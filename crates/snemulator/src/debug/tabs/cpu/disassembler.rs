use snemcore::{Snemulator, scpu};

use crate::debug::tabs::cpu::symbols::SymbolManager;

#[derive(Clone, Copy)]
enum AddressingMode {
    Implied,
    Accumulator,
    Immediate8,
    Immediate16,
    ImmediateM,      // 8 or 16 bit depending on M flag
    ImmediateX,      // 8 or 16 bit depending on X flag
    Relative8,
    Relative16,
    Direct,
    DirectX,
    DirectY,
    DirectIndirect,
    DirectIndirectLong,
    DirectXIndirect,
    DirectIndirectY,
    DirectIndirectLongY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Long,
    LongX,
    AbsoluteIndirect,
    LongIndirect,
    AbsoluteXIndirect,
    StackRelative,
    StackRelativeIndirectY,
    SrcDst,
}

#[derive(Clone, Copy)]
struct DisassembleData {
    pub mnemonic: &'static str,
    pub addr_mode: AddressingMode,
}

#[derive(Clone, Debug)]
pub struct DisassemblyOptions {
    pub use_hw_reg_names: bool,
    pub show_rel_addr_dest: bool,
    pub show_symbols: bool,
    pub max_instr_count: usize,
    pub forced_flag_x: Option<bool>,
    pub forced_flag_m: Option<bool>,
    pub forced_e: Option<bool>,
}

pub enum DisasmOperandKind {
    Number,
    Address { addr: u32 },
    LabeledAddress { addr: u32 },
    Register,
}

pub struct DisasmLine {
    pub addr: u32,
    pub bytes: Vec<u8>,
    pub mnemonic: &'static str,
    pub operand: Option<DisasmOperand>,
}

pub struct DisasmOperand {
    pub text: String,
    pub kind: DisasmOperandKind,
}

/// Information about the state of the cpu that is assumed while disassembling the block.
/// This can affect things like how many bytes are read for an instruction and which
/// addresses are recognized as hardware registers.
pub struct ExecuteState {
    pub dp: u16,
    pub addr: scpu::Address,
    pub flag_m: bool,
    pub flag_x: bool,
}

// This table would be defined elsewhere with all 256 entries
static DISASSEMBLE_TABLE: [DisassembleData; 256] = [
    DisassembleData {mnemonic: "brk", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {mnemonic: "cop", addr_mode: AddressingMode::Immediate8},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::StackRelative},
    DisassembleData {mnemonic: "tsb", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "asl", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {mnemonic: "php", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {mnemonic: "asl", addr_mode: AddressingMode::Accumulator},
    DisassembleData {mnemonic: "phd", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "tsb", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "asl", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "bpl", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {mnemonic: "trb", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "asl", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {mnemonic: "clc", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "inc", addr_mode: AddressingMode::Accumulator},
    DisassembleData {mnemonic: "tcs", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "trb", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "asl", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "ora", addr_mode: AddressingMode::LongX},
    DisassembleData {mnemonic: "jsr", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {mnemonic: "jsl", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::StackRelative},
    DisassembleData {mnemonic: "bit", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "rol", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {mnemonic: "plp", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {mnemonic: "rol", addr_mode: AddressingMode::Accumulator},
    DisassembleData {mnemonic: "pld", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "bit", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "rol", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "bmi", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {mnemonic: "bit", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "rol", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {mnemonic: "sec", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "dec", addr_mode: AddressingMode::Accumulator},
    DisassembleData {mnemonic: "tsc", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "bit", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "rol", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "and", addr_mode: AddressingMode::LongX},
    DisassembleData {mnemonic: "rti", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {mnemonic: "wdm", addr_mode: AddressingMode::Immediate8},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::StackRelative},
    DisassembleData {mnemonic: "mvp", addr_mode: AddressingMode::SrcDst},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "lsr", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {mnemonic: "pha", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {mnemonic: "lsr", addr_mode: AddressingMode::Accumulator},
    DisassembleData {mnemonic: "phk", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "jmp", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "lsr", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "bvc", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {mnemonic: "mvn", addr_mode: AddressingMode::SrcDst},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "lsr", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {mnemonic: "cli", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "phy", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "tcd", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "jmp", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "lsr", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "eor", addr_mode: AddressingMode::LongX},
    DisassembleData {mnemonic: "rts", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {mnemonic: "per", addr_mode: AddressingMode::Immediate8},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::StackRelative},
    DisassembleData {mnemonic: "stz", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "ror", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {mnemonic: "pla", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {mnemonic: "ror", addr_mode: AddressingMode::Accumulator},
    DisassembleData {mnemonic: "rtl", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "jmp", addr_mode: AddressingMode::AbsoluteIndirect},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "ror", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "bvs", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {mnemonic: "stz", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "ror", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {mnemonic: "sei", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "ply", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "tdc", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "jmp", addr_mode: AddressingMode::AbsoluteXIndirect},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "ror", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "adc", addr_mode: AddressingMode::LongX},
    DisassembleData {mnemonic: "bra", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {mnemonic: "brl", addr_mode: AddressingMode::Relative16},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::StackRelative},
    DisassembleData {mnemonic: "sty", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "stx", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {mnemonic: "dey", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "bit", addr_mode: AddressingMode::Immediate8},
    DisassembleData {mnemonic: "txa", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "phb", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "sty", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "stx", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "bcc", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {mnemonic: "sty", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "stx", addr_mode: AddressingMode::DirectY},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {mnemonic: "tya", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "txs", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "txy", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "stz", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "stz", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "sta", addr_mode: AddressingMode::LongX},
    DisassembleData {mnemonic: "ldy", addr_mode: AddressingMode::ImmediateX},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {mnemonic: "ldx", addr_mode: AddressingMode::ImmediateX},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::StackRelative},
    DisassembleData {mnemonic: "ldy", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "ldx", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {mnemonic: "tay", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {mnemonic: "tax", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "plb", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "ldy", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "ldx", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "bcs", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {mnemonic: "ldy", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "ldx", addr_mode: AddressingMode::DirectY},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {mnemonic: "clv", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "tsx", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "tyx", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "ldy", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "ldx", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "lda", addr_mode: AddressingMode::LongX},
    DisassembleData {mnemonic: "cpy", addr_mode: AddressingMode::ImmediateX},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {mnemonic: "rep", addr_mode: AddressingMode::Immediate8},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::StackRelative},
    DisassembleData {mnemonic: "cpy", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "dec", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {mnemonic: "iny", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {mnemonic: "dex", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "wai", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "cpy", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "dec", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "bne", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {mnemonic: "pei", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "dec", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {mnemonic: "cld", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "phx", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "stp", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "jmp", addr_mode: AddressingMode::LongIndirect},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "dec", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "cmp", addr_mode: AddressingMode::LongX},
    DisassembleData {mnemonic: "cpx", addr_mode: AddressingMode::ImmediateX},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {mnemonic: "sep", addr_mode: AddressingMode::Immediate8},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::StackRelative},
    DisassembleData {mnemonic: "cpx", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "inc", addr_mode: AddressingMode::Direct},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {mnemonic: "inx", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {mnemonic: "nop", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "xba", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "cpx", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "inc", addr_mode: AddressingMode::Absolute},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::Long},
    DisassembleData {mnemonic: "beq", addr_mode: AddressingMode::Relative8},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {mnemonic: "pea", addr_mode: AddressingMode::Immediate16},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "inc", addr_mode: AddressingMode::DirectX},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {mnemonic: "sed", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {mnemonic: "plx", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "xcefc", addr_mode: AddressingMode::Implied},
    DisassembleData {mnemonic: "jsr", addr_mode: AddressingMode::AbsoluteXIndirect},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "inc", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {mnemonic: "sbc", addr_mode: AddressingMode::LongX},
];

/// Returns the hardware register name for a given SNES MMIO address, if known
fn get_register_name(addr: u32) -> Option<&'static str> {
    // SNES MMIO is mirrored, so we only care about the lower 16 bits
    // and registers are in bank $00 (or mirrored banks)
    let addr = addr & 0xFFFF;
    
    match addr {
        // PPU Registers ($2100-$213F)
        0x2100 => Some("INIDISP"),
        0x2101 => Some("OBSEL"),
        0x2102 => Some("OAMADDL"),
        0x2103 => Some("OAMADDH"),
        0x2104 => Some("OAMDATA"),
        0x2105 => Some("BGMODE"),
        0x2106 => Some("MOSAIC"),
        0x2107 => Some("BG1SC"),
        0x2108 => Some("BG2SC"),
        0x2109 => Some("BG3SC"),
        0x210A => Some("BG4SC"),
        0x210B => Some("BG12NBA"),
        0x210C => Some("BG34NBA"),
        0x210D => Some("BG1HOFS"),
        0x210E => Some("BG1VOFS"),
        0x210F => Some("BG2HOFS"),
        0x2110 => Some("BG2VOFS"),
        0x2111 => Some("BG3HOFS"),
        0x2112 => Some("BG3VOFS"),
        0x2113 => Some("BG4HOFS"),
        0x2114 => Some("BG4VOFS"),
        0x2115 => Some("VMAIN"),
        0x2116 => Some("VMADDL"),
        0x2117 => Some("VMADDH"),
        0x2118 => Some("VMDATAL"),
        0x2119 => Some("VMDATAH"),
        0x211A => Some("M7SEL"),
        0x211B => Some("M7A"),
        0x211C => Some("M7B"),
        0x211D => Some("M7C"),
        0x211E => Some("M7D"),
        0x211F => Some("M7X"),
        0x2120 => Some("M7Y"),
        0x2121 => Some("CGADD"),
        0x2122 => Some("CGDATA"),
        0x2123 => Some("W12SEL"),
        0x2124 => Some("W34SEL"),
        0x2125 => Some("WOBJSEL"),
        0x2126 => Some("WH0"),
        0x2127 => Some("WH1"),
        0x2128 => Some("WH2"),
        0x2129 => Some("WH3"),
        0x212A => Some("WBGLOG"),
        0x212B => Some("WOBJLOG"),
        0x212C => Some("TM"),
        0x212D => Some("TS"),
        0x212E => Some("TMW"),
        0x212F => Some("TSW"),
        0x2130 => Some("CGWSEL"),
        0x2131 => Some("CGADSUB"),
        0x2132 => Some("COLDATA"),
        0x2133 => Some("SETINI"),
        0x2134 => Some("MPYL"),
        0x2135 => Some("MPYM"),
        0x2136 => Some("MPYH"),
        0x2137 => Some("SLHV"),
        0x2138 => Some("OAMDATAREAD"),
        0x2139 => Some("VMDATALREAD"),
        0x213A => Some("VMDATAHREAD"),
        0x213B => Some("CGDATAREAD"),
        0x213C => Some("OPHCT"),
        0x213D => Some("OPVCT"),
        0x213E => Some("STAT77"),
        0x213F => Some("STAT78"),
        
        // APU Registers ($2140-$2143)
        0x2140 => Some("APUIO0"),
        0x2141 => Some("APUIO1"),
        0x2142 => Some("APUIO2"),
        0x2143 => Some("APUIO3"),
        
        // WRAM Access ($2180-$2183)
        0x2180 => Some("WMDATA"),
        0x2181 => Some("WMADDL"),
        0x2182 => Some("WMADDM"),
        0x2183 => Some("WMADDH"),
        
        // CPU Registers ($4200-$421F)
        0x4200 => Some("NMITIMEN"),
        0x4201 => Some("WRIO"),
        0x4202 => Some("WRMPYA"),
        0x4203 => Some("WRMPYB"),
        0x4204 => Some("WRDIVL"),
        0x4205 => Some("WRDIVH"),
        0x4206 => Some("WRDIVB"),
        0x4207 => Some("HTIMEL"),
        0x4208 => Some("HTIMEH"),
        0x4209 => Some("VTIMEL"),
        0x420A => Some("VTIMEH"),
        0x420B => Some("MDMAEN"),
        0x420C => Some("HDMAEN"),
        0x420D => Some("MEMSEL"),
        0x4210 => Some("RDNMI"),
        0x4211 => Some("TIMEUP"),
        0x4212 => Some("HVBJOY"),
        0x4213 => Some("RDIO"),
        0x4214 => Some("RDDIVL"),
        0x4215 => Some("RDDIVH"),
        0x4216 => Some("RDMPYL"),
        0x4217 => Some("RDMPYH"),
        0x4218 => Some("JOY1L"),
        0x4219 => Some("JOY1H"),
        0x421A => Some("JOY2L"),
        0x421B => Some("JOY2H"),
        0x421C => Some("JOY3L"),
        0x421D => Some("JOY3H"),
        0x421E => Some("JOY4L"),
        0x421F => Some("JOY4H"),
        
        // DMA Registers ($4300-$43FF) - Channel 0-7
        addr if (0x4300..=0x437F).contains(&addr) => {
            let channel = (addr >> 4) & 0x7;
            let reg = addr & 0xF;
            match reg {
                0x0 => Some(match channel {
                    0 => "DMAP0", 1 => "DMAP1", 2 => "DMAP2", 3 => "DMAP3",
                    4 => "DMAP4", 5 => "DMAP5", 6 => "DMAP6", _ => "DMAP7",
                }),
                0x1 => Some(match channel {
                    0 => "BBAD0", 1 => "BBAD1", 2 => "BBAD2", 3 => "BBAD3",
                    4 => "BBAD4", 5 => "BBAD5", 6 => "BBAD6", _ => "BBAD7",
                }),
                0x2 => Some(match channel {
                    0 => "A1T0L", 1 => "A1T1L", 2 => "A1T2L", 3 => "A1T3L",
                    4 => "A1T4L", 5 => "A1T5L", 6 => "A1T6L", _ => "A1T7L",
                }),
                0x3 => Some(match channel {
                    0 => "A1T0H", 1 => "A1T1H", 2 => "A1T2H", 3 => "A1T3H",
                    4 => "A1T4H", 5 => "A1T5H", 6 => "A1T6H", _ => "A1T7H",
                }),
                0x4 => Some(match channel {
                    0 => "A1B0", 1 => "A1B1", 2 => "A1B2", 3 => "A1B3",
                    4 => "A1B4", 5 => "A1B5", 6 => "A1B6", _ => "A1B7",
                }),
                0x5 => Some(match channel {
                    0 => "DAS0L", 1 => "DAS1L", 2 => "DAS2L", 3 => "DAS3L",
                    4 => "DAS4L", 5 => "DAS5L", 6 => "DAS6L", _ => "DAS7L",
                }),
                0x6 => Some(match channel {
                    0 => "DAS0H", 1 => "DAS1H", 2 => "DAS2H", 3 => "DAS3H",
                    4 => "DAS4H", 5 => "DAS5H", 6 => "DAS6H", _ => "DAS7H",
                }),
                0x7 => Some(match channel {
                    0 => "DASB0", 1 => "DASB1", 2 => "DASB2", 3 => "DASB3",
                    4 => "DASB4", 5 => "DASB5", 6 => "DASB6", _ => "DASB7",
                }),
                0x8 => Some(match channel {
                    0 => "A2A0L", 1 => "A2A1L", 2 => "A2A2L", 3 => "A2A3L",
                    4 => "A2A4L", 5 => "A2A5L", 6 => "A2A6L", _ => "A2A7L",
                }),
                0x9 => Some(match channel {
                    0 => "A2A0H", 1 => "A2A1H", 2 => "A2A2H", 3 => "A2A3H",
                    4 => "A2A4H", 5 => "A2A5H", 6 => "A2A6H", _ => "A2A7H",
                }),
                0xA => Some(match channel {
                    0 => "NTRL0", 1 => "NTRL1", 2 => "NTRL2", 3 => "NTRL3",
                    4 => "NTRL4", 5 => "NTRL5", 6 => "NTRL6", _ => "NTRL7",
                }),
                0xB => Some(match channel {
                    0 => "UNUSED0", 1 => "UNUSED1", 2 => "UNUSED2", 3 => "UNUSED3",
                    4 => "UNUSED4", 5 => "UNUSED5", 6 => "UNUSED6", _ => "UNUSED7",
                }),
                _ => None,
            }
        }
        
        _ => None,
    }
}

fn format_accumulator() -> DisasmOperand {
    DisasmOperand {
        text: "A".to_string(),
        kind: DisasmOperandKind::Register,
    }
}

/// Formats an absolute address, optionally replacing with register name
fn format_absolute(addr: u32, options: &DisassemblyOptions, symbols: &SymbolManager) -> DisasmOperand {
    if options.use_hw_reg_names {
        if let Some(name) = get_register_name(addr & 0xFFFF) {
            return DisasmOperand {
                text: name.to_string(),
                kind: DisasmOperandKind::Register,
            };
        }
    }

    if options.show_symbols {
        if let Some(label) = symbols.get_address_label(addr & 0xFFFF) {
            return DisasmOperand {
                text: label.to_string(),
                kind: DisasmOperandKind::LabeledAddress { addr },
            }
        }
    }

    DisasmOperand {
        text: format!("${:04X}", addr as u16),
        kind: DisasmOperandKind::Address { addr },
    }
}

fn format_immediate8(byte: u8) -> DisasmOperand {
    DisasmOperand {
        text: format!("#${:02X}", byte),
        kind: DisasmOperandKind::Number,
    }
}

fn format_immediate16(word: u16) -> DisasmOperand {
    DisasmOperand {
        text: format!("#${:04X}", word),
        kind: DisasmOperandKind::Number,
    }
}

/// Formats an absolute long address, optionally replacing with register name
fn format_absolute_long(addr: u32, options: &DisassemblyOptions, symbols: &SymbolManager) -> DisasmOperand {
    if options.use_hw_reg_names && (addr >> 16) <= 0x3F {
        if let Some(name) = get_register_name(addr & 0xFFFF) {
            return DisasmOperand {
                text: name.to_string(),
                kind: DisasmOperandKind::Register,
            };
        }
    }

    if options.show_symbols {
        if let Some(label) = symbols.get_address_label(addr) {
            return DisasmOperand {
                text: label.to_string(),
                kind: DisasmOperandKind::LabeledAddress { addr },
            }
        }
    }

    DisasmOperand {
        text: format!("${:06X}", addr),
        kind: DisasmOperandKind::Address { addr },
    }
}

/// Formats a direct page address, optionally replacing with register name
/// Note: This resolves the effective address using the direct page register
fn format_direct(dp: u16, dp_offset: u8, options: &DisassemblyOptions, symbols: &SymbolManager) -> DisasmOperand {
    let addr = dp + dp_offset as u16;
    
    if options.use_hw_reg_names {
        if let Some(name) = get_register_name(addr as u32) {
            return DisasmOperand {
                text: name.to_string(),
                kind: DisasmOperandKind::Register,
            };
        }
    }

    if options.show_symbols {
        if let Some(label) = symbols.get_address_label(addr as u32) {
            return DisasmOperand {
                text: label.to_string(),
                kind: DisasmOperandKind::LabeledAddress { addr: addr as u32 },
            }
        }
    }

    DisasmOperand {
        text: format!("${:02X}", dp_offset),
        kind: DisasmOperandKind::Address { addr: addr as u32 },
    }
}

fn format_rel8(pb: u8, pc: u16, offset_byte: u8, options: &DisassemblyOptions, symbols: &SymbolManager) -> DisasmOperand {
    let address = pc as u16 + ((offset_byte as i8) as i16) as u16;
    let address = (pb as u32) << 16 | address as u32;
    
    if options.show_symbols {
        if let Some(label) = symbols.get_address_label(address) {
            return DisasmOperand {
                text: label.to_string(),
                kind: DisasmOperandKind::LabeledAddress { addr: address },
            }
        }
    }

    if options.show_rel_addr_dest {
        return DisasmOperand {
            text: format!("${:04X}", address & 0xFFFF),
            kind: DisasmOperandKind::Address { addr: address },
        };
    }
    
    DisasmOperand {
        text: format!("#${:02X}", offset_byte),
        kind: DisasmOperandKind::Number,
    }
}

fn format_rel16(pb: u8, pc: u16, offset_word: u16, options: &DisassemblyOptions, symbols: &SymbolManager) -> DisasmOperand {
    let address = pc as u16 + offset_word as u16;
    let address = (pb as u32) << 16 | address as u32;
    
    if options.show_symbols {
        if let Some(label) = symbols.get_address_label(address) {
            return DisasmOperand {
                text: label.to_string(),
                kind: DisasmOperandKind::LabeledAddress { addr: address },
            }
        }
    }

    if options.show_rel_addr_dest {
        return DisasmOperand {
            text: format!("${:04X}", address & 0xFFFF),
            kind: DisasmOperandKind::Address { addr: address },
        };
    }
    
    DisasmOperand {
        text: format!("#${:04X}", offset_word as u16),
        kind: DisasmOperandKind::Number,
    }
}

fn disassemble(
    prg_bytes: &[u8; 4],
    state: &ExecuteState,
    options: &DisassemblyOptions,
    symbols: &SymbolManager,
) -> DisasmLine {
    let dp = state.dp;
    let flag_x = state.flag_x;
    let flag_m = state.flag_m;

    let arg8 = prg_bytes[1];
    let arg16 = prg_bytes[1] as u16 | (prg_bytes[2] as u16) << 8;
    let arg24 = arg16 as u32 | (prg_bytes[3] as u32) << 16;
    let data = &DISASSEMBLE_TABLE[prg_bytes[0] as usize];
    
    let operand = match data.addr_mode {
        AddressingMode::Implied => None,
        AddressingMode::Accumulator => Some(format_accumulator()),
        AddressingMode::Immediate8  => Some(format_immediate8(arg8)),
        AddressingMode::Immediate16 => Some(format_immediate16(arg16)),
        AddressingMode::ImmediateM if flag_m => Some(format_immediate8(arg8)),
        AddressingMode::ImmediateM           => Some(format_immediate16(arg16)),
        AddressingMode::ImmediateX if flag_x => Some(format_immediate8(arg8)),
        AddressingMode::ImmediateX           => Some(format_immediate16(arg16)),
        AddressingMode::Relative8  => Some(format_rel8(state.addr.bank, state.addr.offset + 2, arg8, options, symbols)),
        AddressingMode::Relative16 => Some(format_rel16(state.addr.bank, state.addr.offset + 2, arg16, options, symbols)),
        AddressingMode::Direct  => Some(format_direct(dp, arg8, options, symbols)),
        AddressingMode::DirectX => {
            let mut operand = format_direct(dp, arg8, options, symbols);
            operand.text = format!("{},X", operand.text);
            Some(operand)
        },
        AddressingMode::DirectY => {
            let mut operand = format_direct(dp, arg8, options, symbols);
            operand.text = format!("{},Y", operand.text);
            Some(operand)
        },
        AddressingMode::DirectIndirect      => {
            let mut operand = format_direct(dp, arg8, options, symbols);
            operand.text = format!("({})", operand.text);
            Some(operand)
        },
        AddressingMode::DirectIndirectLong  => {
            let mut operand = format_direct(dp, arg8, options, symbols);
            operand.text = format!("[{}]", operand.text);
            Some(operand)
        },
        AddressingMode::DirectXIndirect     => {
            let mut operand = format_direct(dp, arg8, options, symbols);
            operand.text = format!("({},X)", operand.text);
            Some(operand)
        },
        AddressingMode::DirectIndirectY     => {
            let mut operand = format_direct(dp, arg8, options, symbols);
            operand.text = format!("({}),Y", operand.text);
            Some(operand)
        },
        AddressingMode::DirectIndirectLongY => {
            let mut operand = format_direct(dp, arg8, options, symbols);
            operand.text = format!("[{}],Y", operand.text);
            Some(operand)
        },
        AddressingMode::Absolute  => Some(format_absolute(arg16 as u32, options, symbols)),
        AddressingMode::AbsoluteX => {
            let mut operand = format_absolute(arg16 as u32, options, symbols);
            operand.text = format!("{},X", operand.text);
            Some(operand)
        },
        AddressingMode::AbsoluteY => {
            let mut operand = format_absolute(arg16 as u32, options, symbols);
            operand.text = format!("{},Y", operand.text);
            Some(operand)
        },
        AddressingMode::Long  => Some(format_absolute_long(arg24, options, symbols)),
        AddressingMode::LongX => {
            let mut operand = format_absolute_long(arg24, options, symbols);
            operand.text = format!("{},X", operand.text);
            Some(operand)
        }
        AddressingMode::AbsoluteIndirect  => {
            let mut operand = format_absolute(arg16 as u32, options, symbols);
            operand.text = format!("({})", operand.text);
            Some(operand)
        }
        AddressingMode::LongIndirect      => {
            let mut operand = format_absolute(arg16 as u32, options, symbols);
            operand.text = format!("[{}]", operand.text);
            Some(operand)
        }
        AddressingMode::AbsoluteXIndirect => {
            let mut operand = format_absolute(arg16 as u32, options, symbols);
            operand.text = format!("({},X)", operand.text);
            Some(operand)
        }
        AddressingMode::StackRelative => Some(DisasmOperand {
            text: format!("${:02X},S", arg8),
            kind: DisasmOperandKind::Number,
        }),
        AddressingMode::StackRelativeIndirectY => Some(DisasmOperand {
            text: format!("(${:02X},S),Y", arg8),
            kind: DisasmOperandKind::Number,
        }),
        AddressingMode::SrcDst => {
            let dst = prg_bytes[1];
            let src = prg_bytes[2];

            Some(DisasmOperand {
                text: format!("${:02X},${:02X}", src, dst),
                kind: DisasmOperandKind::Number,
            })
        }
    };
    
    let num_data_bytes = match data.addr_mode {
        AddressingMode::Implied => 0,
        AddressingMode::Accumulator => 0,
        AddressingMode::Immediate8  => 1,
        AddressingMode::Immediate16 => 2,
        AddressingMode::ImmediateM if flag_m => 1,
        AddressingMode::ImmediateM           => 2,
        AddressingMode::ImmediateX if flag_x => 1,
        AddressingMode::ImmediateX           => 2,
        AddressingMode::Relative8  => 1,
        AddressingMode::Relative16 => 2,
        AddressingMode::Direct  => 1,
        AddressingMode::DirectX => 1,
        AddressingMode::DirectY => 1,
        AddressingMode::DirectIndirect      => 1,
        AddressingMode::DirectIndirectLong  => 1,
        AddressingMode::DirectXIndirect     => 1,
        AddressingMode::DirectIndirectY     => 1,
        AddressingMode::DirectIndirectLongY => 1,
        AddressingMode::Absolute  => 2,
        AddressingMode::AbsoluteX => 2,
        AddressingMode::AbsoluteY => 2,
        AddressingMode::Long  => 3,
        AddressingMode::LongX => 3,
        AddressingMode::AbsoluteIndirect  => 2,
        AddressingMode::LongIndirect      => 2,
        AddressingMode::AbsoluteXIndirect => 2,
        AddressingMode::StackRelative     => 1,
        AddressingMode::StackRelativeIndirectY => 1,
        AddressingMode::SrcDst => 2,
    };
    let num_bytes = 1 + num_data_bytes;
    
    let bytes = prg_bytes[..num_bytes].to_vec();
    
    let disasm_line = DisasmLine {
        addr: state.addr.to_u32(),
        bytes,
        mnemonic: data.mnemonic,
        operand,
    };
    
    disasm_line
}

pub fn disassemble_forward(
    core: &Snemulator,
    options: &DisassemblyOptions,
    symbols: &SymbolManager,
    start_addr: u32,
) -> Vec<DisasmLine> {
    let mut disassembly = Vec::new();
    let mut addr = scpu::Address::from_u32(start_addr);
    
    let flag_e = if options.forced_e.is_some() {
        options.forced_e.unwrap()
    } else {
        core.cpu.e
    };

    let flag_m = if options.forced_flag_m.is_some() {
        options.forced_flag_m.unwrap() | flag_e
    } else {
        core.cpu.is_flag_set(scpu::Flag::FlagM) | flag_e
    };

    let flag_x = if options.forced_flag_x.is_some() {
        options.forced_flag_x.unwrap() | flag_e
    } else {
        core.cpu.is_flag_set(scpu::Flag::FlagX) | flag_e
    };
    
    let mut state = ExecuteState {
        dp: 0,
        addr,
        flag_m,
        flag_x,
    };
    
    let mut instr_count = 0;
    while instr_count < options.max_instr_count {
        let b0 = read_rom_or_ram(core, scpu::Address::from_u32(addr.to_u32() + 0));
        let b1 = read_rom_or_ram(core, scpu::Address::from_u32(addr.to_u32() + 1));
        let b2 = read_rom_or_ram(core, scpu::Address::from_u32(addr.to_u32() + 2));
        let b3 = read_rom_or_ram(core, scpu::Address::from_u32(addr.to_u32() + 3));
        
        let disasm_line = disassemble(
            &[b0, b1, b2, b3],
            &state,
            options,
            symbols,
        );
        
        addr = scpu::Address::from_u32(addr.to_u32() + disasm_line.bytes.len() as u32);
        
        disassembly.push(disasm_line);
        
        state.addr = addr;
        instr_count += 1;
    }
    
    disassembly
}

fn read_rom_or_ram(core: &Snemulator, addr: scpu::Address) -> u8 {
    if addr.bank & 0x7F <= 0x3F {
        if addr.offset >= 0x8000 {
            core.cart.as_ref().unwrap().read(addr)
        } else {
            0
        }
    } else if addr.bank == 0x7E || addr.bank == 0x7F {
        core.wram[addr.to_u32() as usize & 0x1FFFF]
    } else {
        core.cart.as_ref().unwrap().read(addr)
    }
}