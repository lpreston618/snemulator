pub mod audio;
pub mod state;

use egui::{Color32, FontId};
use egui::text::{LayoutJob, TextFormat};
use crate::theme::AppTheme;

// ─── Ring Buffer ──────────────────────────────────────────────────────────────

const DSP_SAMPLE_RATE: usize = 32_000;
pub const RING_BUFFER_SECONDS: f32 = 10.0;
pub const RING_BUFFER_LEN: usize = (RING_BUFFER_SECONDS as usize) * DSP_SAMPLE_RATE;

/// A fixed-capacity circular buffer for audio samples.
pub struct RingBuffer {
    data: Vec<i16>,
    /// Index of the next write position.
    head: usize,
    /// Number of valid samples currently stored.
    pub len: usize,
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            data: vec![0i16; RING_BUFFER_LEN],
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, sample: i16) {
        self.data[self.head] = sample;
        self.head = (self.head + 1) % RING_BUFFER_LEN;
        self.len = (self.len + 1).min(RING_BUFFER_LEN);
    }

    /// Iterates samples from oldest to newest.
    pub fn iter_chronological(&self) -> impl Iterator<Item = i16> + '_ {
        let start = if self.len < RING_BUFFER_LEN {
            0
        } else {
            self.head
        };
        (0..self.len).map(move |i| self.data[(start + i) % RING_BUFFER_LEN])
    }

    /// Returns the most recent `n` samples, oldest first.
    pub fn tail(&self, n: usize) -> impl Iterator<Item = i16> + '_ {
        let n = n.min(self.len);
        let start = (self.head + RING_BUFFER_LEN - n) % RING_BUFFER_LEN;
        (0..n).map(move |i| self.data[(start + i) % RING_BUFFER_LEN])
    }

    pub fn seconds_buffered(&self) -> f32 {
        self.len as f32 / DSP_SAMPLE_RATE as f32
    }
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
