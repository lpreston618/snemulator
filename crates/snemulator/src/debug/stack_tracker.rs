use std::collections::HashMap;

use snemcore::scpu::{Cpu65c816, CpuInterrupt, Flag};

/// What wrote a given stack byte, for debug display purposes only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PushCause {
    /// PHA / PHX / PHY / PHP / PHB / PHD / PHK
    Reg(&'static str),
    /// PEA / PEI / PER
    Effective(&'static str),
    /// JSR / JSR (indirect)
    JsrReturn,
    /// JSL
    JslReturn,
    /// A hardware or software interrupt entry.
    Interrupt(CpuInterrupt),
    /// on_stack_push fired but we never matched it to a cause (e.g. manual
    /// SP writes followed by an unrelated store, or a gap in our opcode table).
    Unknown,
}

impl PushCause {
    pub fn label(&self) -> String {
        match self {
            PushCause::Reg(name) => name.to_string(),
            PushCause::Effective(name) => name.to_string(),
            PushCause::JsrReturn => "JSR ret".to_string(),
            PushCause::JslReturn => "JSL ret".to_string(),
            PushCause::Interrupt(kind) => format!("{kind:?} entry"),
            PushCause::Unknown => "?".to_string(),
        }
    }
 
    /// A more specific label for a single byte within a multi-byte group,
    /// using its position (offset_in_group, group_size) to say e.g. "ret addr hi"
    /// instead of just "JSR ret" for both bytes.
    ///
    /// Push order (confirmed for this core): JSR pushes high byte first, then low.
    /// JSL pushes PB first, then high, then low.
    pub fn byte_label(&self, offset_in_group: u8, group_size: u8) -> String {
        match self {
            PushCause::JsrReturn if group_size == 2 => {
                if offset_in_group == 0 { "ret addr hi".to_string() } else { "ret addr lo".to_string() }
            }
            PushCause::JslReturn if group_size == 3 => {
                match offset_in_group {
                    0 => "ret bank".to_string(),
                    1 => "ret addr hi".to_string(),
                    _ => "ret addr lo".to_string(),
                }
            }
            _ => self.label(),
        }
    }
}

/// Metadata attached to a single byte currently sitting in the stack region.
#[derive(Clone, Copy, Debug)]
pub struct StackEntryTag {
    pub cause: PushCause,
    pub value: u8,
    /// 0-indexed position within a multi-byte push, in push order.
    /// e.g. for JSR ret addr: offset 0 = high byte (pushed first), offset 1 = low byte.
    pub offset_in_group: u8,
    pub group_size: u8,
}

/// A push whose cause isn't known yet — buffered until the owning
/// on_instruction/on_interrupt callback fires and claims it.
struct PendingPush {
    sp: u16,
    value: u8,
}

const OPCODE_JSL: u8 = 0x22;
const OPCODE_JSR: u8 = 0x20;
const OPCODE_JSR_INDIRECT: u8 = 0xFC;
const OPCODE_PEA: u8 = 0xF4;
const OPCODE_PEI: u8 = 0xD4;
const OPCODE_PER: u8 = 0x62;
const OPCODE_PHA: u8 = 0x48;
const OPCODE_PHX: u8 = 0xDA;
const OPCODE_PHY: u8 = 0x5A;
const OPCODE_PHB: u8 = 0x8B;
const OPCODE_PHD: u8 = 0x0B;
const OPCODE_PHK: u8 = 0x4B;
const OPCODE_PHP: u8 = 0x08;
const OPCODE_BRK: u8 = 0x00;
const OPCODE_COP: u8 = 0x02;

pub struct StackTracker {
    tags: HashMap<u16, StackEntryTag>,
    pending: Vec<PendingPush>,
}

impl StackTracker {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            pending: Vec::new(),
        }
    }

    pub fn tag_at(&self, sp: u16) -> Option<&StackEntryTag> {
        self.tags.get(&sp)
    }

    pub fn clear(&mut self) {
        self.tags.clear();
        self.pending.clear();
    }

    /// Feed from DebugHarness::on_stack_push. SP has already been decremented
    /// by the time this fires (per trait doc: "SP is now the address of the value - 1"),
    /// so the byte lives at sp + 1.
    pub fn on_stack_push(&mut self, cpu: &Cpu65c816, value: u8) {
        let sp = cpu.sp.wrapping_add(1);
        self.pending.push(PendingPush { sp, value });
    }

    /// Feed from DebugHarness::on_stack_pop. We don't remove the tag —
    /// the byte is still physically present until something overwrites it —
    /// but logically it's now free, so we drop its label and leave the raw value.
    /// This matches real hardware: pop only moves SP, it never clears memory.
    pub fn on_stack_pop(&mut self, cpu: &Cpu65c816, value: u8) {
        let sp = cpu.sp;
        if let Some(tag) = self.tags.get_mut(&sp) {
            tag.cause = PushCause::Unknown;
        }
        // If a pending (not-yet-claimed) push exists at this address, drop it too —
        // this can happen if e.g. a push and pop land in the same instruction window
        // before on_instruction fires (shouldn't occur in practice for any real opcode,
        // but cheap to guard against).
        self.pending.retain(|p| p.sp != sp);
        let _ = value;
    }

    /// Claim the trailing N pending pushes for `cause`, in the order they occurred,
    /// and commit them into the tag map. No-op if fewer than N are pending (a sign
    /// our byte-count table doesn't match what actually happened — better to drop
    /// the data than mislabel it).
    fn claim_trailing(&mut self, count: u8, cause_for: impl Fn(u8, u8) -> PushCause) {
        let count = count as usize;
        if count == 0 || self.pending.len() < count {
            self.pending.clear();
            return;
        }

        let start = self.pending.len() - count;
        let claimed: Vec<PendingPush> = self.pending.drain(start..).collect();

        for (i, p) in claimed.into_iter().enumerate() {
            self.tags.insert(p.sp, StackEntryTag {
                cause: cause_for(i as u8, count as u8),
                value: p.value,
                offset_in_group: i as u8,
                group_size: count as u8,
            });
        }
    }

    /// Feed from DebugHarness::on_instruction, after the opcode has fully executed.
    pub fn on_instruction(&mut self, cpu: &Cpu65c816, prg_bytes: &[u8]) {
        let opcode = prg_bytes[0];

        // BRK/COP are claimed by on_interrupt instead — don't double-tag them here.
        if opcode == OPCODE_BRK || opcode == OPCODE_COP {
            return;
        }

        let m8 = cpu.is_flag_set(Flag::FlagM);
        let x8 = cpu.is_flag_set(Flag::FlagX);

        match opcode {
            OPCODE_JSR | OPCODE_JSR_INDIRECT => {
                self.claim_trailing(2, |_, _| PushCause::JsrReturn);
            }
            OPCODE_JSL => {
                self.claim_trailing(3, |_, _| PushCause::JslReturn);
            }
            OPCODE_PEA => self.claim_trailing(2, |_, _| PushCause::Effective("PEA")),
            OPCODE_PEI => self.claim_trailing(2, |_, _| PushCause::Effective("PEI")),
            OPCODE_PER => self.claim_trailing(2, |_, _| PushCause::Effective("PER")),
            OPCODE_PHA => self.claim_trailing(if m8 { 1 } else { 2 }, |_, _| PushCause::Reg("PHA")),
            OPCODE_PHX => self.claim_trailing(if x8 { 1 } else { 2 }, |_, _| PushCause::Reg("PHX")),
            OPCODE_PHY => self.claim_trailing(if x8 { 1 } else { 2 }, |_, _| PushCause::Reg("PHY")),
            OPCODE_PHB => self.claim_trailing(1, |_, _| PushCause::Reg("PHB")),
            OPCODE_PHD => self.claim_trailing(2, |_, _| PushCause::Reg("PHD")),
            OPCODE_PHK => self.claim_trailing(1, |_, _| PushCause::Reg("PHK")),
            OPCODE_PHP => self.claim_trailing(1, |_, _| PushCause::Reg("PHP")),
            _ => {
                // Not a push-producing opcode we recognize. If pushes are sitting
                // unclaimed at this point, something pushed bytes we didn't expect
                // (e.g. a manual SP/memory trick) — leave them tagged Unknown rather
                // than guess.
                if !self.pending.is_empty() {
                    self.claim_trailing(self.pending.len() as u8, |_, _| PushCause::Unknown);
                }
            }
        }
    }

    /// Feed from DebugHarness::on_interrupt, after the interrupt entry sequence
    /// has finished pushing. Standard 65816 behavior: emulation mode pushes
    /// PC-hi, PC-lo, P (2... wait, 3 incl. P -> see byte count below);
    /// native mode additionally pushes PB first. Reset pushes nothing.
    ///
    /// ASSUMPTION (unverified against this core): Abort follows the same
    /// native/emulation byte-count rule as IRQ/NMI/BRK/COP. If this core's
    /// Abort or Reset handling differs, this will mislabel/drop those pushes —
    /// flagging, not fixing blind.
    pub fn on_interrupt(&mut self, cpu: &Cpu65c816, kind: CpuInterrupt) {
        let count: u8 = match kind {
            CpuInterrupt::Reset => 0,
            _ => if cpu.e { 2 } else { 3 },
        };

        self.claim_trailing(count, move |_, _| PushCause::Interrupt(kind));
    }
}