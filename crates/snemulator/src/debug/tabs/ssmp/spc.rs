use crate::app::theme::AppTheme;
use snemcore::{Snemulator, ssmp::spc::Spc700};

use crate::debug::harness::MainDebugHarness;

mod disassembler;

use disassembler::{disassemble_range, DisasmLine, DisasmOperandKind};

const DISASM_BLOCK_SIZE: usize = 64;

struct SpcDisassemblyView {
    cached_lines: Option<Vec<DisasmLine>>,
    follow_pc: bool,
    current_addr: u16,
}

impl SpcDisassemblyView {
    fn new() -> Self {
        Self { cached_lines: None, follow_pc: true, current_addr: 0 }
    }

    fn update(&mut self, core: &Snemulator) {
        if self.follow_pc {
            self.current_addr = core.ssmp.spc.pc;
        }

        self.cached_lines = Some(disassemble_range(
            core.ssmp.aram.as_slice(),
            &Spc700::IPL_ROM,
            core.ssmp.spc_regs.ipl_read_en,
            self.current_addr,
            DISASM_BLOCK_SIZE,
        ));
    }
}

pub struct SpcTab {
    disasm: SpcDisassemblyView,
    bp_input: String,
}

impl SpcTab {
    pub fn new() -> Self {
        Self { disasm: SpcDisassemblyView::new(), bp_input: String::new() }
    }

    pub fn breakpoint_hit(&mut self, addr: u16) {
        self.disasm.current_addr = addr;
    }

    pub fn render(&mut self, ui: &mut egui::Ui, core: &mut Snemulator, harness: &mut MainDebugHarness, app_theme: &AppTheme) {
        self.disasm.update(core);
        let pc = core.ssmp.spc.pc;

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.disasm.follow_pc, "Follow PC");
            if ui.button("Go to PC").clicked() {
                self.disasm.current_addr = pc;
            }
        });

        ui.separator();

        let available_height = ui.available_height();
        const SIDEBAR_WIDTH: f32 = 220.0;
        const BP_SECTION_FRACTION: f32 = 0.30;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.disasm_section(ui, core, harness, app_theme, available_height * (1.0 - BP_SECTION_FRACTION));

                egui::ScrollArea::vertical().id_salt("spc_info_scroll").show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_width(SIDEBAR_WIDTH);
                        egui::CollapsingHeader::new("Registers").default_open(true)
                            .show(ui, |ui| self.spc_state_section(ui, core, app_theme));
                    });
                });
            });

            ui.add_space(5.0);
            self.breakpoints_section(ui, core, harness);
        });
    }

    fn disasm_section(&mut self, ui: &mut egui::Ui, core: &Snemulator, harness: &mut MainDebugHarness, app_theme: &AppTheme, available_height: f32) {
        ui.vertical(|ui| {
            egui::ScrollArea::vertical().id_salt("spc_disasm_scroll").min_scrolled_height(available_height).show(ui, |ui| {
                if let Some(lines) = self.disasm.cached_lines.take() {
                    for line in lines.iter() {
                        ui.horizontal(|ui| {
                            if let Some(addr) = self.disasm_line(ui, core, harness, app_theme, line) {
                                self.disasm.follow_pc = false;
                                self.disasm.current_addr = addr;
                            }
                        });
                    }
                } else {
                    ui.label("No disassembly available");
                }
            });
        });
    }

    fn disasm_line(&mut self, ui: &mut egui::Ui, core: &Snemulator, harness: &mut MainDebugHarness, app_theme: &AppTheme, line: &DisasmLine) -> Option<u16> {
        use egui::text::{LayoutJob, TextFormat};

        let mono = egui::FontId::monospace(13.0);
        let addr = line.addr as u16;
        let is_current = addr == core.ssmp.spc.pc;
        let has_breakpoint = harness.spc_breakpoints.contains(&addr);
        let bg_color = if has_breakpoint { Some(app_theme.breakpoint_bg) }
            else if is_current { Some(app_theme.highlight_line) } else { None };

        let bp_response = app_theme.draw_breakpoint_marker(ui, has_breakpoint, is_current);
        if bp_response.clicked() {
            if has_breakpoint { harness.spc_breakpoints.remove(&addr); }
            else { harness.spc_breakpoints.insert(addr); }
        }

        let make_format = |color: egui::Color32, bg: Option<egui::Color32>| TextFormat {
            font_id: mono.clone(), color, background: bg.unwrap_or(egui::Color32::TRANSPARENT), ..Default::default()
        };

        let mut prefix_job = LayoutJob::default();
        prefix_job.append(&format!("{:04X}  ", addr), 0.0, make_format(app_theme.syntax_address, bg_color));
        let bytes_str: String = line.bytes.iter().take(4).map(|b| format!("{:02X} ", b)).collect();
        prefix_job.append(&format!("{:<12}", bytes_str), 0.0, make_format(app_theme.text_muted, bg_color));
        ui.label(prefix_job);

        let mut jump_target: Option<u16> = None;
        let mut template_parts = line.mnemonic.split("{}");
        let mut operands = line.operands.iter();

        if let Some(first) = template_parts.next() {
            ui.label(egui::RichText::new(first).monospace().color(app_theme.syntax_opcode));
        }

        for part in template_parts {
            if let Some(operand) = operands.next() {
                let (text, target, color) = match operand.kind {
                    DisasmOperandKind::Immediate8(v) =>
                        (format!("#${:02X}", v), None, app_theme.syntax_number),
                    DisasmOperandKind::DirectPage(a) =>
                        (format!("${:02X}", a), Some(a as u16), app_theme.syntax_address),
                    DisasmOperandKind::Absolute(a) =>
                        (format!("${:04X}", a), Some(a), app_theme.syntax_address),
                    DisasmOperandKind::AbsoluteBit(a, bit) =>
                        (format!("${:04X}.{}", a, bit), Some(a), app_theme.syntax_address),
                    DisasmOperandKind::BranchTarget(a) =>
                        (format!("${:04X}", a), Some(a), app_theme.syntax_address), // or a dedicated color, see note below
                };

                if let Some(target) = target {
                    let rich_text = egui::RichText::new(&text).monospace().color(color);
                    let response = ui.add(egui::Label::new(rich_text).sense(egui::Sense::click()));
                    if response.clicked() { jump_target = Some(target); }
                    response.on_hover_cursor(egui::CursorIcon::PointingHand);
                } else {
                    ui.label(egui::RichText::new(&text).monospace().color(color));
                }
            }
            ui.label(egui::RichText::new(part).monospace().color(app_theme.syntax_opcode));
        }

        jump_target
    }

    fn spc_state_section(&mut self, ui: &mut egui::Ui, core: &Snemulator, app_theme: &AppTheme) {
        app_theme.section_header(ui, "SPC700 State");

        ui.horizontal(|ui| {
            ui.label(app_theme.format_register("PC", core.ssmp.spc.pc, 16, false));
            ui.label(app_theme.format_register("SP", core.ssmp.spc.sp as u16, 8, false));
        });
        ui.horizontal(|ui| {
            ui.label(app_theme.format_register("A", core.ssmp.spc.a as u16, 8, false));
            ui.label(app_theme.format_register("X", core.ssmp.spc.x as u16, 8, false));
            ui.label(app_theme.format_register("Y", core.ssmp.spc.y as u16, 8, false));
        });
        ui.horizontal(|ui| {
            ui.label(app_theme.format_register("PSW", core.ssmp.spc.status as u16, 8, false));
            ui.label(app_theme.format_register("DP", core.ssmp.spc.dir_page, 16, false));
        });

        app_theme.debugger_separator(ui);

        ui.horizontal(|ui| {
            let mut stopped = core.ssmp.spc.stopped;
            let mut ipl_read_en = core.ssmp.spc_regs.ipl_read_en;
            let mut sdsp_read_only = core.ssmp.spc_regs.sdsp_read_only;
            ui.add_enabled(false, egui::Checkbox::new(&mut stopped, "Stopped"));
            ui.add_enabled(false, egui::Checkbox::new(&mut ipl_read_en, "IPL Read En."));
            ui.add_enabled(false, egui::Checkbox::new(&mut sdsp_read_only, "SDSP Read Only"));
        });

        // app_theme.debugger_separator(ui);


    }

    fn breakpoints_section(&mut self, ui: &mut egui::Ui, core: &mut Snemulator, harness: &mut MainDebugHarness) {
        let breakpoints = &mut harness.spc_breakpoints;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Breakpoints").strong().size(14.0));
            if ui.button("Clear All").clicked() { breakpoints.clear(); }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Add:");
            let response = ui.add(egui::TextEdit::singleline(&mut self.bp_input).hint_text("XXXX").char_limit(4));
            let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Add").clicked() || submitted {
                if let Ok(addr) = u16::from_str_radix(self.bp_input.trim(), 16) {
                    breakpoints.insert(addr);
                    self.bp_input.clear();
                }
            }
        });

        ui.separator();

        if breakpoints.is_empty() {
            ui.label("No breakpoints set.");
            return;
        }

        let mut to_remove: Option<u16> = None;
        let mut sorted: Vec<u16> = breakpoints.iter().copied().collect();
        sorted.sort();

        egui::ScrollArea::vertical().id_salt("spc_bp_scroll").show(ui, |ui| {
            for group in sorted.chunks(8) {
                ui.horizontal(|ui| {
                    for &breakpoint in group {
                        ui.horizontal(|ui| {
                            if ui.small_button("❌").clicked() { to_remove = Some(breakpoint); }
                            if ui.button(egui::RichText::new(format!("{:04X}", breakpoint)).monospace()).clicked() {
                                self.disasm.follow_pc = breakpoint == core.ssmp.spc.pc;
                                self.disasm.current_addr = breakpoint;
                            }
                        });
                    }
                });
            }
        });

        if let Some(breakpoint) = to_remove {
            breakpoints.remove(&breakpoint);
        }
    }
}