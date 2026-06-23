use egui::text::LayoutJob;
use egui::{Color32, FontId, Pos2, Rect, Stroke, Vec2};
use snemcore::ssmp::sdsp::{ADSRStage, GainMode};

use crate::debug::harness::{ENVELOPE_HISTORY_LEN, MainDebugHarness};
use crate::app::theme::AppTheme;
use crate::debug::tabs::ssmp::{append, detail_heading, detail_row, fmt_bool, fmt_hex_u8, fmt_hex_u16, fmt_i16_signed, fmt_u8_dec};
use snemcore::Snemulator;

// ─── Constants ────────────────────────────────────────────────────────────────

const ENVELOPE_PAINTER_HEIGHT: f32 = 40.0;

// ─── Struct ───────────────────────────────────────────────────────────────────

pub struct SdspTab {
    voice_open: [bool; 8],
}

impl SdspTab {
    pub fn new() -> Self {
        Self {
            voice_open: [false; 8],
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        core: &mut Snemulator,
        harness: &mut MainDebugHarness,
        app_theme: &AppTheme,
    ) {
        let regs = &core.ssmp.sdsp_regs;
        let voice_regs = &core.ssmp.voice_regs;

        // ── Global DSP state ─────────────────────────────────────────────────
        egui::Frame::NONE
            .fill(app_theme.bg_secondary)
            .inner_margin(egui::Margin::same(6))
            .corner_radius(app_theme.corner_radius as f32)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.global_kv(
                        ui,
                        app_theme,
                        "Vol L/R",
                        &format!("${:02X}/${:02X}", regs.lmain_volume, regs.rmain_volume),
                    );
                    ui.separator();
                    self.global_kv(
                        ui,
                        app_theme,
                        "Echo L/R",
                        &format!("${:02X}/${:02X}", regs.lecho_volume, regs.recho_volume),
                    );
                    ui.separator();
                    self.global_kv(
                        ui,
                        app_theme,
                        "Echo FB",
                        &format!("${:02X}", regs.echo_feedback),
                    );
                    ui.separator();
                    self.global_kv(
                        ui,
                        app_theme,
                        "Noise Freq",
                        &format!("${:02X}", regs.noise_freq),
                    );
                    ui.separator();
                    self.global_kv(
                        ui,
                        app_theme,
                        "Echo Delay",
                        &format!("${:02X}", regs.echo_delay_time),
                    );
                    ui.separator();
                    self.global_kv(
                        ui,
                        app_theme,
                        "Echo Page",
                        &format!("${:02X}", regs.echo_page),
                    );
                    ui.separator();
                    self.global_kv(
                        ui,
                        app_theme,
                        "Sample Dir",
                        &format!("${:02X}", regs.sample_directory_page),
                    );
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    // FIR coefficients
                    let fir_str = regs
                        .fir_regs
                        .iter()
                        .map(|b| format!("{:02X}", *b as u8))
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.global_kv(ui, app_theme, "FIR", &fir_str);
                    ui.separator();

                    // Global flags
                    self.flag_badge(ui, app_theme, "RESET", regs.soft_reset, app_theme.error);
                    self.flag_badge(ui, app_theme, "MUTE", regs.mute_all, app_theme.warning);
                    self.flag_badge(ui, app_theme, "ECHO", regs.echo_en, app_theme.success);
                    ui.separator();

                    self.global_kv(ui, app_theme, "Key On", &format!("0b{:08b}", regs.key_on));
                    ui.separator();
                    self.global_kv(ui, app_theme, "Key Off", &format!("0b{:08b}", regs.key_off));
                });
            });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        // ── Per-voice panels ─────────────────────────────────────────────────
        egui::ScrollArea::vertical().show(ui, |ui| {
            for v in 0..8usize {
                let vr = &voice_regs[v];
                let is_active = vr.envelope != 0;

                if harness.voices_just_keyed_on[v] && !self.voice_open[v] {
                    harness.voices_just_keyed_on[v] = false;
                    self.voice_open[v] = true;
                }

                let header_job = self.format_voice_header(app_theme, v, vr, is_active);

                let resp = egui::CollapsingHeader::new(header_job)
                    .id_salt(format!("sdsp_voice_{v}"))
                    .open(Some(self.voice_open[v]))
                    .show(ui, |ui| {
                        self.render_voice_detail(ui, harness, app_theme, v, vr);
                    });

                if resp.header_response.clicked() {
                    self.voice_open[v] = !self.voice_open[v];
                }

                resp.header_response.context_menu(|ui| {
                    self.render_voice_context_menu(ui, v);
                });
            }
        });
    }

    // ── Voice header summary ──────────────────────────────────────────────────

    fn format_voice_header(
        &self,
        app_theme: &AppTheme,
        v: usize,
        vr: &snemcore::ssmp::sdsp::voices::VoiceRegs,
        is_active: bool,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();
        let mono = FontId::monospace(13.0);

        let ch_color = if is_active {
            app_theme.accent
        } else {
            app_theme.text_muted
        };
        append(&mut job, &format!("VOICE {v}  "), mono.clone(), ch_color);

        self.append_flag_badge(&mut job, "ADSR", vr.adsr_en, app_theme, mono.clone());
        append(&mut job, " ", mono.clone(), app_theme.text_muted);
        self.append_flag_badge(&mut job, "NOISE", vr.noise_en, app_theme, mono.clone());
        append(&mut job, " ", mono.clone(), app_theme.text_muted);
        self.append_flag_badge(&mut job, "ECHO", vr.echo_en, app_theme, mono.clone());
        append(&mut job, " ", mono.clone(), app_theme.text_muted);
        self.append_flag_badge(
            &mut job,
            "PITCHMOD",
            vr.pitchmod_en,
            app_theme,
            mono.clone(),
        );
        append(&mut job, "  ", mono.clone(), app_theme.text_muted);

        let (stage_str, stage_color) = adsr_stage_fmt(vr.adsr_stage, app_theme);
        append(&mut job, "Stage: ", mono.clone(), app_theme.text_secondary);
        append(&mut job, stage_str, mono.clone(), stage_color);
        append(&mut job, "  ", mono.clone(), app_theme.text_muted);

        append(&mut job, "Src: ", mono.clone(), app_theme.text_secondary);
        append(
            &mut job,
            &format!("${:02X}", vr.sample_source),
            mono.clone(),
            app_theme.syntax_number,
        );

        job
    }

    fn append_flag_badge(
        &self,
        job: &mut LayoutJob,
        label: &str,
        val: bool,
        app_theme: &AppTheme,
        font_id: FontId,
    ) {
        let (text, color) = if val {
            (format!("[{label} ✓]"), app_theme.success)
        } else {
            (format!("[{label} ✗]"), app_theme.text_disabled)
        };
        append(job, &text, font_id, color);
    }

    // ── Voice expanded detail ─────────────────────────────────────────────────

    fn render_voice_detail(
        &self,
        ui: &mut egui::Ui,
        harness: &MainDebugHarness,
        app_theme: &AppTheme,
        v: usize,
        vr: &snemcore::ssmp::sdsp::voices::VoiceRegs,
    ) {
        ui.add_space(4.0);

        ui.columns(2, |cols| {
            // ── Left: Pitch / Volume / Output ────────────────────────────────
            let ui = &mut cols[0];
            ui.label(detail_heading(app_theme, "Output"));
            ui.add_space(2.0);
            detail_row(ui, app_theme, "Vol L/R", &{
                let mut j = LayoutJob::default();
                append(
                    &mut j,
                    &format!("${:02X} / ${:02X}", vr.lchannel_volume, vr.rchannel_volume),
                    FontId::monospace(12.0),
                    app_theme.syntax_number,
                );
                j
            });
            detail_row(ui, app_theme, "Pitch", &fmt_hex_u16(vr.pitch, app_theme));
            detail_row(
                ui,
                app_theme,
                "Envelope",
                &fmt_hex_u16(vr.envelope as u16, app_theme),
            );
            detail_row(
                ui,
                app_theme,
                "Sample Out",
                &fmt_i16_signed(vr.sample_out_high, app_theme),
            );
            detail_row(
                ui,
                app_theme,
                "EOS",
                &fmt_bool(vr.end_of_sample_flag, app_theme),
            );
            detail_row(ui, app_theme, "Loop", &fmt_bool(vr.loop_flag, app_theme));

            ui.add_space(6.0);
            ui.label(detail_heading(app_theme, "Gain"));
            ui.add_space(2.0);
            detail_row(
                ui,
                app_theme,
                "Mode",
                &fmt_gain_mode(vr.gain_mode, app_theme),
            );
            detail_row(ui, app_theme, "Rate", &fmt_u8_dec(vr.gain_rate, app_theme));
            detail_row(
                ui,
                app_theme,
                "Fixed",
                &fmt_hex_u8(vr.gain_fixed, app_theme),
            );
            detail_row(
                ui,
                app_theme,
                "Raw",
                &fmt_hex_u8(vr.gain_reg_raw, app_theme),
            );

            // ── Right: ADSR ──────────────────────────────────────────────────
            let ui = &mut cols[1];
            let (stage_str, stage_color) = adsr_stage_fmt(vr.adsr_stage, app_theme);
            ui.label({
                let mut j = LayoutJob::default();
                append(
                    &mut j,
                    "ADSR  ",
                    FontId::monospace(12.0),
                    app_theme.syntax_directive,
                );
                append(&mut j, stage_str, FontId::monospace(12.0), stage_color);
                j
            });
            ui.add_space(2.0);
            detail_row(ui, app_theme, "ADSR En", &fmt_bool(vr.adsr_en, app_theme));
            detail_row(
                ui,
                app_theme,
                "Attack",
                &fmt_u8_dec(vr.adsr_attack, app_theme),
            );
            detail_row(
                ui,
                app_theme,
                "Decay",
                &fmt_u8_dec(vr.adsr_decay, app_theme),
            );
            detail_row(
                ui,
                app_theme,
                "Sus. Level",
                &fmt_u8_dec(vr.adsr_sustain_level, app_theme),
            );
            detail_row(
                ui,
                app_theme,
                "Sus. Rate",
                &fmt_u8_dec(vr.adsr_sustain_rate, app_theme),
            );

            ui.add_space(6.0);
            ui.label(detail_heading(app_theme, "Flags"));
            ui.add_space(2.0);
            detail_row(ui, app_theme, "Noise", &fmt_bool(vr.noise_en, app_theme));
            detail_row(ui, app_theme, "Echo", &fmt_bool(vr.echo_en, app_theme));
            detail_row(
                ui,
                app_theme,
                "Pitch Mod",
                &fmt_bool(vr.pitchmod_en, app_theme),
            );
        });

        ui.add_space(6.0);

        // ── Envelope history painter ─────────────────────────────────────────
        let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), ENVELOPE_PAINTER_HEIGHT));
        self.paint_envelope(ui, harness, app_theme, v, rect);

        ui.add_space(4.0);
    }

    // ── Envelope painter ──────────────────────────────────────────────────────

    fn paint_envelope(
        &self,
        ui: &egui::Ui,
        harness: &MainDebugHarness,
        app_theme: &AppTheme,
        v: usize,
        rect: Rect,
    ) {
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, app_theme.corner_radius as f32, app_theme.bg_tertiary);

        let history = &harness.envelope_history[v];
        let w = rect.width();
        let h = rect.height();

        let points: Vec<Pos2> = history
            .iter_chronological()
            .enumerate()
            .map(|(i, sample)| {
                // envelope is i16 in range 0..=0x7FF (2047)
                let norm = (sample as f32 / 2047.0).clamp(0.0, 1.0);
                let x = rect.left() + (i as f32 / (ENVELOPE_HISTORY_LEN - 1) as f32) * w;
                let y = rect.bottom() - norm * h;
                Pos2::new(x, y)
            })
            .collect();

        if points.len() >= 2 {
            painter.add(egui::Shape::line(
                points,
                Stroke::new(1.5, app_theme.accent),
            ));
        }

        // Border
        painter.rect_stroke(
            rect,
            app_theme.corner_radius as f32,
            Stroke::new(1.0, app_theme.border),
            egui::StrokeKind::Outside,
        );
    }

    // ── Context menu ──────────────────────────────────────────────────────────

    fn render_voice_context_menu(&mut self, ui: &mut egui::Ui, v: usize) {
        ui.label(format!("Voice {v}"));
        ui.separator();
        // Add context menu items here
    }

    // ── Global state helpers ──────────────────────────────────────────────────

    fn global_kv(&self, ui: &mut egui::Ui, app_theme: &AppTheme, label: &str, value: &str) {
        ui.horizontal(|ui| {
            let mut job = LayoutJob::default();
            append(
                &mut job,
                &format!("{label}: "),
                FontId::monospace(12.0),
                app_theme.text_secondary,
            );
            append(
                &mut job,
                value,
                FontId::monospace(12.0),
                app_theme.syntax_number,
            );
            ui.label(job);
        });
    }

    fn flag_badge(
        &self,
        ui: &mut egui::Ui,
        app_theme: &AppTheme,
        label: &str,
        active: bool,
        color: Color32,
    ) {
        let (fg, bg) = if active {
            (color, app_theme.bg_elevated)
        } else {
            (app_theme.text_disabled, app_theme.bg_secondary)
        };
        egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .corner_radius(app_theme.widget_corner_radius as f32)
            .show(ui, |ui| {
                let mut job = LayoutJob::default();
                append(&mut job, label, FontId::monospace(12.0), fg);
                ui.label(job);
            });
    }
}

// ─── Free formatting helpers ──────────────────────────────────────────────────

fn adsr_stage_fmt(stage: ADSRStage, app_theme: &AppTheme) -> (&'static str, Color32) {
    match stage {
        ADSRStage::Attack => ("ATTACK", app_theme.warning),
        ADSRStage::Decay => ("DECAY", app_theme.info),
        ADSRStage::Sustain => ("SUSTAIN", app_theme.success),
        ADSRStage::Release => ("RELEASE", app_theme.text_muted),
    }
}

fn fmt_gain_mode(mode: GainMode, app_theme: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    let text = match mode {
        GainMode::Increase => "Linear Inc",
        GainMode::BentIncrease => "Bent Inc",
        GainMode::Decrease => "Linear Dec",
        GainMode::ExpDecrease => "Exp. Dec",
        GainMode::Fixed => "Fixed",
    };
    append(
        &mut job,
        text,
        FontId::monospace(12.0),
        app_theme.syntax_keyword,
    );
    job
}
