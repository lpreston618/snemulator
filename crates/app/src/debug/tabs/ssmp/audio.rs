use egui::{Color32, FontId, Pos2, Rect, Stroke, Vec2};
use egui::text::LayoutJob;
use sdl3::audio::AudioStreamOwner;

use crate::app::AppState;
use crate::debug::harness::{DSP_SAMPLE_RATE, MainDebugHarness, SAMPLE_HISTORY_LEN, SAMPLE_HISTORY_SECONDS};
use crate::theme::AppTheme;
use super::{append, RingBuffer};

// ─── Constants ────────────────────────────────────────────────────────────────

const WAVEFORM_HEIGHT: f32 = 48.0;
const WAVEFORM_DISPLAY_SAMPLES: usize = 4096;

// ─── Struct ───────────────────────────────────────────────────────────────────

pub struct SdspAudioTab {
    pub playing_voice: Option<usize>, // Which voice is currently playing, if any
}

impl SdspAudioTab {
    pub fn new() -> Self {
        Self {
            playing_voice: None,
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        harness: &mut MainDebugHarness,
        stream: &mut AudioStreamOwner,
        app_state: &AppState,
        app_theme: &AppTheme,
    ) {
        if stream.queued_bytes().unwrap_or(0) <= 128 {
            self.playing_voice = None;
        }

        // ── Toolbar ──────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui.button("Clear All").clicked() {
                self.clear_all(harness);
            }

            if let Some(v) = self.playing_voice {
                ui.add_space(8.0);
                let mut stop_job = LayoutJob::default();
                let label = if v == usize::MAX { "■ Stop (MIX)".to_string() } else { format!("■ Stop (V{v})") };
                append(&mut stop_job, &label, FontId::monospace(12.0), app_theme.warning);
                if ui.button(stop_job).clicked() {
                    self.playing_voice = None;
                    let _ = stream.clear();
                }
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ── Per-voice strips ──────────────────────────────────────────────
            for v in 0..8usize {
                self.render_voice_strip(ui, app_state, app_theme, v, stream, harness);
                ui.add_space(4.0);
            }

            ui.separator();

            // ── Master mix strip ─────────────────────────────────────────────
            self.render_mix_strip(ui, app_theme, stream, harness);
        });
    }

    // ── Voice strip ───────────────────────────────────────────────────────────

    fn render_voice_strip(
        &mut self,
        ui: &mut egui::Ui,
        app_state: &AppState,
        app_theme: &AppTheme,
        v: usize,
        stream: &mut AudioStreamOwner,
        harness: &mut MainDebugHarness,
    ) {
        let buf_secs = seconds_buffered(&harness.voice_buffers[v].0);

        ui.horizontal(|ui| {
            // Voice label
            let mut label_job = LayoutJob::default();
            append(&mut label_job, &format!("V{v} "), FontId::monospace(13.0), app_theme.accent);
            ui.label(label_job);

            // Play button
            let is_playing = self.playing_voice == Some(v);
            let play_label = if is_playing { "■" } else { "▶" };
            let play_color = if is_playing { app_theme.warning } else { app_theme.success };
            let has_audio  = harness.voice_buffers[v].0.len > 0;

            ui.add_enabled_ui(has_audio && app_state.is_paused, |ui| {
                if ui.add(egui::Button::new({
                    let mut j = LayoutJob::default();
                    append(&mut j, play_label, FontId::monospace(12.0), play_color);
                    j
                })).clicked() {
                    if is_playing {
                        self.playing_voice = None;
                        let _ = stream.clear();
                    } else {
                        self.playing_voice = Some(v);
                        self.upload_voice_to_stream(v, stream, harness);
                    }
                }
            });

            // Buffer length indicator
            let mut buf_job = LayoutJob::default();
            append(&mut buf_job, &format!("{buf_secs:.1}s / {}s", SAMPLE_HISTORY_SECONDS),
                FontId::monospace(11.0), app_theme.text_muted);
            ui.label(buf_job);

            // Clear this voice
            if ui.small_button("✕").clicked() {
                harness.voice_buffers[v].0 = RingBuffer::new();
                harness.voice_buffers[v].1 = RingBuffer::new();
                if self.playing_voice == Some(v) {
                    self.playing_voice = None;
                    let _ = stream.clear();
                }
            }
        });

        // Waveform — left channel
        let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), WAVEFORM_HEIGHT));
        self.paint_waveform(ui, app_theme, rect, &harness.voice_buffers[v].0, app_theme.accent, "L");

        // Waveform — right channel
        let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), WAVEFORM_HEIGHT));
        self.paint_waveform(ui, app_theme, rect, &harness.voice_buffers[v].1, app_theme.info, "R");
    }

    // ── Master mix strip ──────────────────────────────────────────────────────

    fn render_mix_strip(
        &mut self,
        ui: &mut egui::Ui,
        app_theme: &AppTheme,
        stream: &mut AudioStreamOwner,
        harness: &mut MainDebugHarness,
    ) {
        let buf_secs = seconds_buffered(&harness.mix_buffers.0);
        let is_playing = self.playing_voice == Some(usize::MAX); // sentinel for mix

        ui.horizontal(|ui| {
            let mut label_job = LayoutJob::default();
            append(&mut label_job, "MIX", FontId::monospace(13.0), app_theme.syntax_label);
            ui.label(label_job);

            let play_label = "▶";
            let play_color = if is_playing { app_theme.warning } else { app_theme.success };
            let has_audio  = harness.mix_buffers.0.len > 0;

            ui.add_enabled_ui(has_audio, |ui| {
                if ui.add(egui::Button::new({
                    let mut j = LayoutJob::default();
                    append(&mut j, play_label, FontId::monospace(12.0), play_color);
                    j
                })).clicked() {
                    if is_playing {
                        self.playing_voice = None;
                        let _ = stream.clear();
                    } else {
                        self.playing_voice = Some(usize::MAX);
                        self.upload_mix_to_stream(stream, harness);
                    }
                }
            });

            let mut buf_job = LayoutJob::default();
            append(&mut buf_job, &format!("{buf_secs:.1}s / {}s", SAMPLE_HISTORY_SECONDS),
                FontId::monospace(11.0), app_theme.text_muted);
            ui.label(buf_job);

            if ui.small_button("✕").clicked() {
                harness.mix_buffers.0 = RingBuffer::new();
                harness.mix_buffers.1 = RingBuffer::new();
                if is_playing {
                    self.playing_voice = None;
                    let _ = stream.clear();
                }
            }
        });

        let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), WAVEFORM_HEIGHT));
        self.paint_waveform(ui, app_theme, rect, &harness.mix_buffers.0, app_theme.success, "L");

        let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), WAVEFORM_HEIGHT));
        self.paint_waveform(ui, app_theme, rect, &harness.mix_buffers.1, app_theme.success, "R");
    }

    // ── Waveform painter ──────────────────────────────────────────────────────

    fn paint_waveform(
        &self,
        ui: &egui::Ui,
        app_theme: &AppTheme,
        rect: Rect,
        buf: &RingBuffer<SAMPLE_HISTORY_LEN>,
        color: Color32,
        channel_label: &str,
    ) {
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, app_theme.corner_radius as f32, app_theme.bg_tertiary);

        let w = rect.width();
        let h = rect.height();
        let mid_y = rect.center().y;

        // Zero line
        painter.line_segment(
            [Pos2::new(rect.left(), mid_y), Pos2::new(rect.right(), mid_y)],
            Stroke::new(1.0, app_theme.border),
        );

        let samples: Vec<f32> = buf.tail(WAVEFORM_DISPLAY_SAMPLES)
            .map(|s| s as f32 / i16::MAX as f32)
            .collect();
        let n = samples.len();

        if n >= 2 {
            let points: Vec<Pos2> = samples.iter().enumerate().map(|(i, &s)| {
                let x = rect.left() + (i as f32 / (n - 1) as f32) * w;
                let y = mid_y - s.clamp(-1.0, 1.0) * (h * 0.5);
                Pos2::new(x, y)
            }).collect();
            painter.add(egui::Shape::line(points, Stroke::new(1.0, color)));
        }

        // Channel label overlay
        painter.text(
            Pos2::new(rect.left() + 4.0, rect.top() + 2.0),
            egui::Align2::LEFT_TOP,
            channel_label,
            FontId::monospace(10.0),
            app_theme.text_muted,
        );

        painter.rect_stroke(
            rect,
            app_theme.corner_radius as f32,
            Stroke::new(1.0, app_theme.border),
            egui::StrokeKind::Outside,
        );
    }

    // ── SDL3 upload ───────────────────────────────────────────────────────────

    /// Interleaves left/right samples and uploads to the SDL3 stream.
    fn upload_voice_to_stream(&self, v: usize, stream: &mut AudioStreamOwner, harness: &mut MainDebugHarness) {
        let samples = self.interleave_stereo(
            &harness.voice_buffers[v].0,
            &harness.voice_buffers[v].1,
        );
        let _ = stream.put_data_i16(&samples);
    }

    fn upload_mix_to_stream(&self, stream: &mut AudioStreamOwner, harness: &mut MainDebugHarness) {
        let samples = self.interleave_stereo(&harness.mix_buffers.0, &harness.mix_buffers.1);
        let _ = stream.put_data_i16(&samples);
    }

    /// Interleaves two mono ring buffers into a stereo i16 vec [L, R, L, R, ...].
    fn interleave_stereo(&self, left: &RingBuffer<SAMPLE_HISTORY_LEN>, right: &RingBuffer<SAMPLE_HISTORY_LEN>) -> Vec<i16> {
        let n = left.len.min(right.len);
        let mut out = Vec::with_capacity(n * 2);
        for (l, r) in left.iter_chronological().zip(right.iter_chronological()) {
            out.push(l);
            out.push(r);
        }
        out
    }

    fn clear_all(&mut self, harness: &mut MainDebugHarness) {
        for v in 0..8usize {
            harness.voice_buffers[v].0 = RingBuffer::new();
            harness.voice_buffers[v].1 = RingBuffer::new();
        }
        harness.mix_buffers.0 = RingBuffer::new();
        harness.mix_buffers.1 = RingBuffer::new();
        self.playing_voice = None;
    }
}

fn seconds_buffered(buf: &RingBuffer<SAMPLE_HISTORY_LEN>) -> f32 {
    buf.len as f32 / DSP_SAMPLE_RATE as f32
}