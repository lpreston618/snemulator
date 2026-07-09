pub mod audio;
pub mod sdsp;
pub mod brr;
pub mod spc;

use egui::{Color32, FontId};
use egui::text::{LayoutJob, TextFormat};
use crate::debug::harness::{MainDebugHarness, StopCondition};
use crate::app::theme::AppTheme;

/// A fixed-capacity circular buffer for audio samples or envelopes.
pub struct RingBuffer<const CAPACITY: usize> {
    data: Vec<i16>,
    /// Index of the next write position.
    head: usize,
    /// Number of valid samples/envelopes currently stored.
    pub len: usize,
}

impl<const CAPACITY: usize> RingBuffer<CAPACITY> {
    pub fn new() -> Self {
        Self {
            data: vec![0i16; CAPACITY],
            head: 0,
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.len = 0;
        self.head = 0;
    }

    pub fn push(&mut self, sample: i16) {
        self.data[self.head] = sample;
        self.head = (self.head + 1) % CAPACITY;
        self.len = (self.len + 1).min(CAPACITY);
    }

    /// Iterates samples from oldest to newest.
    pub fn iter_chronological(&self) -> impl Iterator<Item = i16> + '_ {
        let start = if self.len < CAPACITY {
            0
        } else {
            self.head
        };
        (0..self.len).map(move |i| self.data[(start + i) % CAPACITY])
    }

    /// Returns the most recent `n` samples, oldest first.
    pub fn tail(&self, n: usize) -> impl Iterator<Item = i16> + '_ {
        let n = n.min(self.len);
        let start = (self.head + CAPACITY - n) % CAPACITY;
        (0..n).map(move |i| self.data[(start + i) % CAPACITY])
    }
}

// ─── Shared voice context menu helper ─────────────────────────────────────────

/// Returns a bool for whether the app needs to unpause
pub fn voice_context_menu(
    ui: &mut egui::Ui,
    core: &snemcore::Snemulator,
    harness: &mut MainDebugHarness,
    voice: usize,
) -> bool {
    let voice_mask = 1 << voice;
    let is_on = core.ssmp.sdsp_regs.key_on & voice_mask != 0 && core.ssmp.sdsp_regs.key_off & voice_mask == 0;

    let (text, stop_cond) = if is_on {
        ("Run Until Key Off", StopCondition::KeyOff { v: voice as u8 })
    } else {
        ("Run Until Key On", StopCondition::KeyOn { v: voice as u8 })
    };

    let clicked = ui.button(text).clicked();

    if clicked {
        harness.stop_condition = Some(stop_cond);
    }

    clicked
}

// ─── Shared formatting helpers ────────────────────────────────────────────────

pub fn append(job: &mut LayoutJob, text: &str, font_id: FontId, color: Color32) {
    job.append(text, 0.0, TextFormat { font_id, color, ..Default::default() });
}

pub fn fmt_hex_u8(val: u8, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("${val:02X}"), FontId::monospace(12.0), t.syntax_number);
    job
}

pub fn fmt_hex_u16(val: u16, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("${val:04X}"), FontId::monospace(12.0), t.syntax_number);
    job
}

pub fn fmt_i8_signed(val: i8, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("{val:+}"), FontId::monospace(12.0), t.syntax_number);
    job
}

pub fn fmt_i16_signed(val: i16, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("{val:+}"), FontId::monospace(12.0), t.syntax_number);
    job
}

pub fn fmt_bool(val: bool, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    let (text, color) = if val { ("✓", t.success) } else { ("✗", t.text_disabled) };
    append(&mut job, text, FontId::monospace(12.0), color);
    job
}

pub fn fmt_u8_dec(val: u8, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("{val}"), FontId::monospace(12.0), t.syntax_number);
    job
}

pub fn detail_row(ui: &mut egui::Ui, t: &AppTheme, label: &str, value: &LayoutJob) {
    ui.horizontal(|ui| {
        let mut label_job = LayoutJob::default();
        append(&mut label_job, &format!("{label:<14}"), FontId::monospace(12.0), t.text_secondary);
        ui.label(label_job);
        ui.label(value.clone());
    });
}

pub fn detail_heading(t: &AppTheme, text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, text, FontId::monospace(12.0), t.syntax_directive);
    job
}
