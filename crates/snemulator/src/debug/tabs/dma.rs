use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId};

use crate::app::AppState;
use crate::debug::harness::{MainDebugHarness, StopCondition};
use crate::debug::window::DebugAction;
use crate::app::theme::AppTheme;
use snemcore::Snemulator;
use snemcore::dma::{AddressIncMode, Direction, TransferPattern};

// ─── Struct ──────────────────────────────────────────────────────────────────

pub struct DmaTab {
    channel_open: [bool; 8],
}

impl DmaTab {
    pub fn new() -> Self {
        Self { channel_open: [false; 8] }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        core: &Snemulator,
        harness: &mut MainDebugHarness,
        app_state: &AppState,
        app_theme: &AppTheme,
        debug_action: &mut Option<DebugAction>,
    ) {
        let dma = &core.dma;

        // ── Global status bar ────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.add_enabled_ui(app_state.is_paused, |ui| {
                if ui.button("Run Until DMA Start").clicked() {
                    *debug_action = Some(DebugAction::SetPaused(false));
                    harness.stop_condition = Some(StopCondition::DmaStart { ch: None });
                }

                if ui.button("Run Until H-DMA Init").clicked() {
                    *debug_action = Some(DebugAction::SetPaused(false));
                    harness.stop_condition = Some(StopCondition::HdmaInit { ch: None });
                }
            });
            self.render_status_badge(ui, app_theme, "DMA", dma.dma_active_ch < 8,
                Some(format!("CH{}", dma.dma_active_ch)));
            ui.add_space(8.0);
            self.render_status_badge(ui, app_theme, "HDMA", dma.hdma_active_ch < 8,
                Some(format!("CH{}", dma.hdma_active_ch)));
            ui.add_space(8.0);
            self.render_flag_badge(ui, app_theme, "HDMA PENDING", dma.regs.iter().any(|ch| ch.hdma_en) && !core.cpu_regs.hblank_flag, app_theme.warning);
            ui.add_space(4.0);
            self.render_flag_badge(ui, app_theme, "NEEDS INIT", dma.hdma_needs_init, app_theme.info);
        });

        ui.separator();

        // ── Per-channel rows ─────────────────────────────────────────────────
        egui::ScrollArea::vertical().show(ui, |ui| {
            for ch in 0..8usize {
                let regs = &core.dma.regs[ch];
                let dma_active = regs.dma_en;
                let hdma_active = regs.hdma_en; 
                let is_active = regs.dma_en || regs.hdma_en;

                let header_job = self.format_channel_header(core, app_theme, ch, is_active);

                let resp = egui::CollapsingHeader::new(header_job)
                    .id_salt(format!("dma_ch_{ch}"))
                    .open(Some(self.channel_open[ch]))
                    .show(ui, |ui| {
                        self.render_channel_detail(ui, app_theme, regs);
                    });

                if resp.header_response.clicked() {
                    self.channel_open[ch] = !self.channel_open[ch];
                }

                if app_state.is_paused {
                    resp.header_response.context_menu(|ui| {
                        if dma_active {
                            if ui.button(format!("Run Until DMA{} Ends", ch)).clicked() {
                                *debug_action = Some(DebugAction::SetPaused(false));
                                harness.stop_condition = Some(StopCondition::DmaEnd { ch: Some(ch as u8) });
                                ui.close()
                            }
                        } else {
                            if ui.button(format!("Run Until DMA{} Begins", ch)).clicked() {
                                *debug_action = Some(DebugAction::SetPaused(false));
                                harness.stop_condition = Some(StopCondition::DmaStart { ch: Some(ch as u8) });
                                ui.close()
                            }
                        }

                        if hdma_active {
                            if ui.button(format!("Run Until H-DMA{} Ends", ch)).clicked() {
                                *debug_action = Some(DebugAction::SetPaused(false));
                                harness.stop_condition = Some(StopCondition::HdmaEnd { ch: Some(ch as u8) });
                                ui.close()
                            }

                            if ui.button(format!("Run Until H-DMA{} Entry", ch)).clicked() {
                                *debug_action = Some(DebugAction::SetPaused(false));
                                harness.stop_condition = Some(StopCondition::HdmaEntry { ch: Some(ch as u8) });
                                ui.close()
                            }
                        } else {
                            if ui.button(format!("Run Until H-DMA{} Init", ch)).clicked() {
                                *debug_action = Some(DebugAction::SetPaused(false));
                                harness.stop_condition = Some(StopCondition::HdmaInit { ch: Some(ch as u8) });
                                ui.close()
                            }
                        }

                        if ui.button(format!("Run Until H-DMA{} Transfer", ch)).clicked() {
                            *debug_action = Some(DebugAction::SetPaused(false));
                            harness.stop_condition = Some(StopCondition::HdmaScanline { ch: Some(ch as u8) });
                            ui.close();
                        }
                    });
                }
            }
        });
    }

    // ── Channel header summary ────────────────────────────────────────────────

    fn format_channel_header(
        &self,
        core: &Snemulator,
        t: &AppTheme,
        ch: usize,
        is_active: bool,
    ) -> LayoutJob {
        let regs = &core.dma.regs[ch];

        let mut job = LayoutJob::default();
        let mono = FontId::monospace(13.0);

        let ch_color = if is_active { t.accent } else { t.text_muted };

        append(&mut job, &format!("CH{ch}  "), mono.clone(), ch_color);

        // DMA / HDMA enable badges inline
        append_bool_badge(&mut job, "DMA", regs.dma_en, t, mono.clone());
        append(&mut job, "  ", mono.clone(), t.text_muted);
        append_bool_badge(&mut job, "HDMA", regs.hdma_en, t, mono.clone());
        append(&mut job, "  ", mono.clone(), t.text_muted);

        // Direction
        let dir_str = match regs.direction {
            Direction::AtoB => "A→B",
            Direction::BtoA => "B→A",
        };
        append(&mut job, dir_str, mono.clone(), t.syntax_keyword);
        append(&mut job, "  ", mono.clone(), t.text_muted);

        // A-bus → B-bus
        append(&mut job, &format!("${:02X}:{:04X}", regs.a_bus_addr.bank, regs.a_bus_addr.offset),
            mono.clone(), t.syntax_address);
        append(&mut job, " - ", mono.clone(), t.text_muted);
        
        if regs.b_bus_addr == 0x18 || regs.b_bus_addr == 0x19 {
            append(&mut job, &format!("VRAM[${:04X}]", core.ppu_regs.vram_addr), mono.clone(), t.syntax_address);
        } else {
            append(&mut job, &format!("$21{:02X}", regs.b_bus_addr), mono.clone(), t.syntax_address);
        }

        append(&mut job, "  ", mono.clone(), t.text_muted);

        // Transfer pattern
        append(&mut job, &format_pattern(regs.transfer_pattern), mono.clone(), t.syntax_directive);

        append(&mut job, " ", mono.clone(), t.text_muted);

        append(&mut job, &format!("({} Bytes)", regs.transfer_pattern_length()), mono.clone(), t.syntax_directive);

        job
    }

    // ── Expanded detail view ──────────────────────────────────────────────────

    fn render_channel_detail(
        &self,
        ui: &mut egui::Ui,
        t: &AppTheme,
        regs: &snemcore::dma::DmaRegs,
    ) {
        ui.add_space(4.0);
        ui.columns(2, |cols| {
            // ── Left column: Common / DMA ────────────────────────────────────
            let ui = &mut cols[0];
            ui.label(detail_heading(t, "DMA"));
            ui.add_space(2.0);

            detail_row(ui, t, "DMA En",       &format_bool(regs.dma_en, t));
            detail_row(ui, t, "Direction",    &format_dir(regs.direction, t));
            detail_row(ui, t, "B-Bus",        &format_bbus(regs.b_bus_addr, t));
            detail_row(ui, t, "A-Bus",        &format_addr(regs.a_bus_addr, t));
            detail_row(ui, t, "Pattern",      &format_pattern_colored(regs.transfer_pattern, t));
            detail_row(ui, t, "Pattern Step", &format_usize_dec(regs.transfer_pattern_step as usize, t));
            detail_row(ui, t, "Transferred",  &format_usize_dec(regs.dma_bytes_transferred, t));
            detail_row(ui, t, "Remaining",    &format_usize_dec(regs.hdma_indirect_table_addr.offset as usize, t));
            detail_row(ui, t, "Inc Mode",     &format_inc_mode(regs.inc_mode, t));
            detail_row(ui, t, "Params Raw",   &format_hex_u8(regs.params_raw, t));

            // ── Right column: HDMA ───────────────────────────────────────────
            let ui = &mut cols[1];
            ui.label(detail_heading(t, "HDMA"));
            ui.add_space(2.0);

            detail_row(ui, t, "HDMA En",      &format_bool(regs.hdma_en, t));
            detail_row(ui, t, "Indirect",     &format_bool(regs.indirect_hdma, t));
            detail_row(ui, t, "Ind. Addr",    &format_addr(regs.hdma_indirect_table_addr, t));
            detail_row(ui, t, "Table Offset", &format_hex_u16(regs.hdma_table_offset, t));
            detail_row(ui, t, "Entry Lines",  &format_usize_dec(regs.entry_scanline_count as usize, t));
            detail_row(ui, t, "Lines Left",   &format_scanlines_left(regs.scanlines_left, t));
            detail_row(ui, t, "Repeat",       &format_bool(regs.hdma_repeat_flag, t));
            detail_row(ui, t, "Do Transfer",  &format_bool(regs.hdma_do_transfer, t));
            detail_row(ui, t, "Entry Loaded", &format_bool(regs.hdma_entry_just_loaded, t));
        });
        ui.add_space(4.0);
    }

    // ── Status bar helpers ────────────────────────────────────────────────────

    fn render_status_badge(
        &self,
        ui: &mut egui::Ui,
        t: &AppTheme,
        label: &str,
        active: bool,
        detail: Option<String>,
    ) {
        let (fg, bg) = if active {
            (t.text_primary, t.accent_muted)
        } else {
            (t.text_disabled, t.bg_secondary)
        };

        egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .corner_radius(t.widget_corner_radius as f32)
            .show(ui, |ui| {
                let mut job = LayoutJob::default();
                let mono = FontId::monospace(12.0);
                append(&mut job, label, mono.clone(), fg);
                if active {
                    if let Some(d) = detail {
                        append(&mut job, &format!(": {d}"), mono, t.accent);
                    }
                }
                ui.label(job);
            });
    }

    fn render_flag_badge(&self, ui: &mut egui::Ui, t: &AppTheme, label: &str, active: bool, color: Color32) {
        if !active { return; }
        egui::Frame::NONE
            .fill(t.bg_secondary)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .corner_radius(t.widget_corner_radius as f32)
            .show(ui, |ui| {
                let mut job = LayoutJob::default();
                append(&mut job, label, FontId::monospace(12.0), color);
                ui.label(job);
            });
    }
}

// ─── Formatting helpers ───────────────────────────────────────────────────────

fn append(job: &mut LayoutJob, text: &str, font_id: FontId, color: Color32) {
    job.append(text, 0.0, TextFormat { font_id, color, ..Default::default() });
}

fn append_bool_badge(job: &mut LayoutJob, label: &str, val: bool, t: &AppTheme, font_id: FontId) {
    let (text, color) = if val {
        (format!("[{label} ✓]"), t.success)
    } else {
        (format!("[{label} ✗]"), t.text_disabled)
    };
    append(job, &text, font_id, color);
}

fn detail_heading(t: &AppTheme, text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, text, FontId::monospace(12.0), t.syntax_directive);
    job
}

/// Renders a key: value row with the label left-aligned and value right of it.
fn detail_row(ui: &mut egui::Ui, t: &AppTheme, label: &str, value: &LayoutJob) {
    ui.horizontal(|ui| {
        let mut label_job = LayoutJob::default();
        append(&mut label_job, &format!("{label:<14}"), FontId::monospace(12.0), t.text_secondary);
        ui.label(label_job);
        ui.label(value.clone());
    });
}

fn format_bool(val: bool, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    let (text, color) = if val { ("✓", t.success) } else { ("✗", t.text_disabled) };
    append(&mut job, text, FontId::monospace(12.0), color);
    job
}

fn format_hex_u8(val: u8, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("${val:02X}"), FontId::monospace(12.0), t.syntax_number);
    job
}

fn format_hex_u16(val: u16, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("${val:04X}"), FontId::monospace(12.0), t.syntax_number);
    job
}

fn format_usize_dec(val: usize, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("{val}"), FontId::monospace(12.0), t.syntax_number);
    job
}

fn format_addr(addr: snemcore::scpu::Address, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("${:02X}:{:04X}", addr.bank, addr.offset),
        FontId::monospace(12.0), t.syntax_address);
    job
}

fn format_bbus(b_bus_addr: u8, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format!("$21{b_bus_addr:02X}"), FontId::monospace(12.0), t.syntax_address);
    job
}

fn format_dir(dir: Direction, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    let text = match dir {
        Direction::AtoB => "A→B",
        Direction::BtoA => "B→A",
    };
    append(&mut job, text, FontId::monospace(12.0), t.syntax_keyword);
    job
}

fn format_inc_mode(mode: AddressIncMode, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    let text = match mode {
        AddressIncMode::Inc => "Increment",
        AddressIncMode::Dec => "Decrement",
        AddressIncMode::Fixed    => "Fixed",
    };
    append(&mut job, text, FontId::monospace(12.0), t.syntax_keyword);
    job
}

fn format_pattern(pattern: TransferPattern) -> String {
    match pattern {
        TransferPattern::Pattern0 => "1-reg",
        TransferPattern::Pattern1 => "2-reg-seq",
        TransferPattern::Pattern2 | TransferPattern::Pattern6 => "1-reg-x2",
        TransferPattern::Pattern3 | TransferPattern::Pattern7 => "2-reg-x2-seq",
        TransferPattern::Pattern4 => "4-reg",
        TransferPattern::Pattern5 => "1-reg-x4",
    }.to_string()
}

fn format_pattern_colored(pattern: TransferPattern, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    append(&mut job, &format_pattern(pattern), FontId::monospace(12.0), t.syntax_directive);
    job
}

/// Colors scanlines_left with a warning tint when it's 1 (about to tick).
fn format_scanlines_left(val: u8, t: &AppTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    let color = if val == 1 { t.warning } else { t.syntax_number };
    append(&mut job, &format!("{val}"), FontId::monospace(12.0), color);
    job
}