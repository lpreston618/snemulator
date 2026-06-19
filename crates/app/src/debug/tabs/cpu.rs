use std::collections::HashMap;

use crate::{debug::tabs::cpu::symbols::{LabelEditState, SymbolManager}, theme::AppTheme};
use snemcore::{Snemulator, scpu};

use crate::debug::harness::MainDebugHarness;

mod disassembler;
mod symbols;

const DISASM_BLOCK_SIZE: usize = 64;

struct DisassemblyView {
    cached_lines: Option<Vec<disassembler::DisasmLine>>,
    comments: HashMap<u32, String>,
    symbols: SymbolManager,
    options: disassembler::DisassemblyOptions,
    follow_pc: bool,
    current_addr: u32,
}

impl DisassemblyView {
    fn new() -> Self {        
        Self {
            cached_lines: None,
            comments: HashMap::new(),
            symbols: SymbolManager::new(),
            options: disassembler::DisassemblyOptions {
                use_hw_reg_names: true,
                show_rel_addr_dest: true,
                show_symbols: true,
                max_instr_count: DISASM_BLOCK_SIZE,
                forced_flag_x: None,
                forced_flag_m: None,
                forced_e: None,
            },
            follow_pc: true,
            current_addr: 0,
        }
    }

    fn update(&mut self,
        core: &Snemulator,
        options: &disassembler::DisassemblyOptions,
    ) {
        if self.follow_pc {
            self.current_addr = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;
        }

        self.cached_lines = Some(disassembler::disassemble_forward(core, options, self.current_addr));
    }
}

pub struct CpuTab {
    disasm: DisassemblyView,
    bp_input: String,
    rom_changes: HashMap<u32, u8>,
    label_edit: LabelEditState,
}

impl CpuTab {
    pub fn new() -> Self {
        Self {
            disasm: DisassemblyView::new(),
            bp_input: String::new(),
            rom_changes: std::collections::HashMap::new(),
            label_edit: LabelEditState::new(),
            // rom_edit: None,
        }
    }
    
    pub fn breakpoint_hit(&mut self, addr: u32) {
        self.disasm.current_addr = addr;
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, core: &mut Snemulator, harness: &mut MainDebugHarness, app_theme: &AppTheme) {
        self.update_disasm(core);
        
        let pc = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.disasm.options.use_hw_reg_names, "Use HW Reg Names");

                ui.checkbox(&mut self.disasm.options.show_symbols, "Show Labels");
                
                ui.checkbox(&mut self.disasm.options.show_rel_addr_dest, "Show Branch Dest Addr");

                ui.checkbox(&mut self.disasm.follow_pc, "Follow PC");

                if ui.button("Go to PC").clicked() {
                    self.disasm.current_addr = pc;
                    self.disasm.options.forced_flag_x = None;
                    self.disasm.options.forced_flag_m = None;
                    self.disasm.options.forced_e = None;
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("cpu_mode_sel")
                    .selected_text(
                        match self.disasm.options.forced_e {
                            Some(true) => "Emulation",
                            Some(false) => "Native",
                            None => if core.cpu.e { "Emulation" } else { "Native" },
                        })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.disasm.options.forced_e, Some(true), "Emulation");
                        ui.selectable_value(&mut self.disasm.options.forced_e, Some(false), "Native");
                        ui.selectable_value(&mut self.disasm.options.forced_e, None, "Current in Program");
                    });

                let (m_text, x_text, mx_en) = match self.disasm.options.forced_e {
                    Some(true) => {
                        ("m8", "x8", false)
                    }
                    None if core.cpu.e => {
                        ("m8", "x8", false)
                    }
                    _ => {
                        let m_text = match self.disasm.options.forced_flag_m {
                            Some(true) => "m8",
                            Some(false) => "m16",
                            None => if core.cpu.is_flag_set(scpu::Flag::FlagM) { "m8" } else { "m16" },
                        };
                        let x_text = match self.disasm.options.forced_flag_x {
                            Some(true) => "x8",
                            Some(false) => "x16",
                            None => if core.cpu.is_flag_set(scpu::Flag::FlagX) { "x8" } else { "x16" },
                        };
                        (m_text, x_text, true)
                    }
                };

                ui.add_enabled_ui(mx_en, |ui| {
                    egui::ComboBox::from_id_salt("m_flag_sel")
                        .selected_text(m_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.disasm.options.forced_flag_m, Some(true), "m8");
                            ui.selectable_value(&mut self.disasm.options.forced_flag_m, Some(false), "m16");
                            ui.selectable_value(&mut self.disasm.options.forced_flag_m, None, "Current in Program");
                        });
    
                    egui::ComboBox::from_id_salt("x_flag_sel")
                        .selected_text(x_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.disasm.options.forced_flag_x, Some(true), "x8");
                            ui.selectable_value(&mut self.disasm.options.forced_flag_x, Some(false), "x16");
                            ui.selectable_value(&mut self.disasm.options.forced_flag_x, None, "Current in Program");
                        });
                });
                
                ui.add_enabled_ui(!self.rom_changes.is_empty(), |ui| {
                    if ui.button("Reset ROM Data").clicked() {
                        let cart = core.cart.as_mut().unwrap();
                        
                        for (addr, value) in self.rom_changes.iter() {
                            cart.force_write(scpu::Address::from_u32(*addr), *value);
                        }
                        
                        self.rom_changes.clear();
                    }
                });
            });
        });

        ui.separator();

        let available_height = ui.available_height();

        const SIDEBAR_WIDTH: f32 = 220.0;
        const BP_SECTION_HEIGHT: f32 = 140.0;

        ui.horizontal(|ui| {
            self.disasm_section(ui, core, harness, app_theme, available_height - BP_SECTION_HEIGHT);

            ui.vertical(|ui| {
                ui.set_width(SIDEBAR_WIDTH);

                egui::CollapsingHeader::new("Registers")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.cpu_state_section(ui, core, app_theme);
                    });

                ui.add_space(5.0);

                egui::CollapsingHeader::new("Stack")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.stack_section(ui, core, harness, app_theme);
                    });
            });
        });

        ui.add_space(5.0);

        self.breakpoints_section(ui, core, harness);

        self.label_edit_popup(ui.ctx());
    }
    
    fn label_edit_popup(&mut self, ctx: &egui::Context) {
        if !self.label_edit.open {
            return;
        }

        let mut should_close = false;
        let mut should_save = false;

        egui::Window::new("Edit Label")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Address:");
                    ui.label(
                        egui::RichText::new(format!("${:06X}", self.label_edit.address))
                            .monospace()
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Label:");
                    let response = ui.text_edit_singleline(&mut self.label_edit.input);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        should_save = true;
                    }
                    response.request_focus();
                });

                if let Some(error) = &self.label_edit.error {
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        should_save = true;
                    }
                    if ui.button("Remove").clicked() {
                        let _ = self.disasm.symbols.set_address_label(self.label_edit.address, None);
                        should_close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                });
            });

        if should_save {
            let label = self.label_edit.input.trim();
            if label.is_empty() {
                let _ = self.disasm.symbols.set_address_label(self.label_edit.address, None);
                should_close = true;
            } else {
                match self.disasm.symbols.set_address_label(self.label_edit.address, Some(label.to_string())) {
                    Ok(_) => should_close = true,
                    Err(e) => self.label_edit.error = Some(e.to_string()),
                }
            }
        }

        if should_close {
            self.label_edit.close();
        }
    }

    fn disasm_section(
        &mut self,
        ui: &mut egui::Ui,
        core: &mut Snemulator,
        harness: &mut MainDebugHarness,
        app_theme: &AppTheme,
        available_height: f32
    ) {
        ui.vertical(|ui| {
            egui::ScrollArea::vertical().id_salt("disasm_scroll").min_scrolled_height(available_height).show(ui, |ui| {
                let lines = self.disasm.cached_lines.take();

                if let Some(lines) = lines {
                    for line in lines.iter() {
                        if self.disasm.options.show_symbols {
                            let label = self.disasm.symbols.get_address_label(line.addr);
                        
                            // Render label line if present
                            if let Some(label) = label {
                                ui.horizontal(|ui| {
                                    // Empty space for breakpoint gutter alignment
                                    // ui.add_space(20.0); // Adjust this to match your breakpoint marker width
                                    
                                    let label_job = self.format_label_line(app_theme, label);
                                    ui.label(label_job);
                                });
                            }
                        }
                        
                        ui.horizontal(|ui| {
                            self.disasm_line(ui, core, harness, app_theme, line);
                        });
                    }
                } else {
                    ui.label("No disassembly available");
                }
            });
        });
    }
    
    fn disasm_line(
        &mut self,
        ui: &mut egui::Ui,
        core: &mut Snemulator,
        harness: &mut MainDebugHarness,
        app_theme: &AppTheme,
        line: &disassembler::DisasmLine,
    ) {
        let pc = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;
        
        let addr = line.addr;
        let is_current = addr == pc;
        let has_breakpoint = harness.breakpoints.contains(&addr);
        let is_modified = self.rom_changes.contains_key(&addr);
        
        // Breakpoint gutter
        let bp_response = app_theme.draw_breakpoint_marker(ui, has_breakpoint, is_current);
        if bp_response.clicked() {
            if has_breakpoint {
                harness.breakpoints.remove(&addr);
            } else {
                harness.breakpoints.insert(addr);
            }
        }
        
        // Format with theme
        let job = self.format_disassembly_line(
            app_theme,
            line,
            None,
            is_current,
            has_breakpoint,
            is_modified,
        );
        
        ui.label(job)
            .context_menu(|ui| self.disasm_context_menu(ui, core, line));
    }
    
    fn disasm_context_menu(&mut self, ui: &mut egui::Ui, core: &mut Snemulator, line: &disassembler::DisasmLine) {
        const NOP: u8 = 0xEA;
        
        let is_changed = self.rom_changes.contains_key(&line.addr);
        
        if is_changed {
            if ui.button("Revert").clicked() {
                let mut earliest_in_changed = line.addr;
                while self.rom_changes.contains_key(&(earliest_in_changed - 1)) {
                    earliest_in_changed -= 1;
                }
                
                let cart = core.cart.as_mut().unwrap();
                
                let mut addr = earliest_in_changed;
                while self.rom_changes.contains_key(&addr) {
                    cart.force_write(scpu::Address::from_u32(addr), self.rom_changes[&addr]);
                    self.rom_changes.remove(&addr);
                    addr += 1;
                }
                
                ui.close();
            }
        } else {
            if ui.button("Replace with NOPs").clicked() {
                let cart = core.cart.as_mut().unwrap();
    
                for i in 0..line.bytes.len() {
                    let addr = line.addr + i as u32;
                    self.rom_changes.insert(addr, cart.read(scpu::Address::from_u32(addr)));
                    cart.force_write(scpu::Address::from_u32(addr), NOP);
                }
                
                ui.close();
            }
        }

        // Label for current instruction address
        let current_label = self.disasm.symbols.get_address_label(line.addr);
        let current_label_text = if current_label.is_some() {
            "Edit Label"
        } else {
            "Add Label"
        };
        
        if ui.button(current_label_text).clicked() {
            self.label_edit.open_for(line.addr, current_label);
            ui.close();
        }

        // Label for destination address (if operand is an address)
        if let Some(operand) = &line.operand {
            if matches!(operand.kind, disassembler::DisasmOperandKind::Address) {
                let dest_addr = operand.value;
                let dest_label = self.disasm.symbols.get_address_label(dest_addr);
                let dest_label_text = if dest_label.is_some() {
                    format!("Edit Argument Label")
                } else {
                    format!("Add Argument Label")
                };
                
                if ui.button(dest_label_text).clicked() {
                    self.label_edit.open_for(dest_addr, dest_label);
                    ui.close();
                }
            }
        }
    }
    
    fn cpu_state_section(&mut self, ui: &mut egui::Ui, core: &Snemulator, app_theme: &AppTheme) {
        app_theme.section_header(ui, "CPU State");

        ui.horizontal(|ui| {
            ui.label(app_theme.format_register("PB", core.cpu.pb as u16, 8, false));
            ui.label(app_theme.format_register("PC", core.cpu.pc, 16, false));
            ui.label(app_theme.format_register("SP", core.cpu.sp, 16, false));
            ui.label(app_theme.format_register("DB", core.cpu.db as u16, 8, false));
            ui.label(app_theme.format_register("DP", core.cpu.dp, 16, false));
        });

        ui.horizontal(|ui| {
            ui.label(app_theme.format_register("A", core.cpu.a, 16, false));
            ui.label(app_theme.format_register("X", core.cpu.x, 16, false));
            ui.label(app_theme.format_register("Y", core.cpu.y, 16, false));

            ui.label(app_theme.format_status_flags(core));
        });
        
        app_theme.debugger_separator(ui);

        ui.horizontal(|ui| {
            let mut halted = core.cpu.halted;
            let mut stopped = core.cpu.stopped;
            let mut waiting_for_interrupt = core.cpu.waiting_for_interrupt;

            ui.add_enabled(false,
                egui::Checkbox::new(&mut halted, "Halted")
            );
            ui.add_enabled(false,
                egui::Checkbox::new(&mut stopped, "Stopped")
            );
            ui.add_enabled(false,
                egui::Checkbox::new(&mut waiting_for_interrupt, "Waiting for Interrupt")
            );
        });

        ui.horizontal(|ui| {
            let mut irq_pending = core.cpu_regs.hv_timer_irq_flag;
            let mut nmi_pending = core.cpu.nmi_pending;

            ui.add_enabled(false,
                egui::Checkbox::new(&mut irq_pending, "IRQ Pending")
            );
            ui.add_enabled(false,
                egui::Checkbox::new(&mut nmi_pending, "NMI Pending")
            );
        });

        app_theme.debugger_separator(ui);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("(APU→CPU)").monospace().color(app_theme.text_muted));
            ui.label(app_theme.format_register("APUIO0", core.apu_ports.apuio0 as u16, 8, false));
            ui.label(app_theme.format_register("APUIO1", core.apu_ports.apuio1 as u16, 8, false));
            ui.label(app_theme.format_register("APUIO2", core.apu_ports.apuio2 as u16, 8, false));
            ui.label(app_theme.format_register("APUIO3", core.apu_ports.apuio3 as u16, 8, false));
        });

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("(CPU→APU)").monospace().color(app_theme.text_muted));
            ui.label(app_theme.format_register("CPUIO0", core.apu_ports.cpuio0 as u16, 8, false));
            ui.label(app_theme.format_register("CPUIO1", core.apu_ports.cpuio1 as u16, 8, false));
            ui.label(app_theme.format_register("CPUIO2", core.apu_ports.cpuio2 as u16, 8, false));
            ui.label(app_theme.format_register("CPUIO3", core.apu_ports.cpuio3 as u16, 8, false));
        });
    }
    
    fn stack_section(&mut self, ui: &mut egui::Ui, core: &Snemulator, harness: &MainDebugHarness, app_theme: &AppTheme) {
        const STACK_DEPTH: usize = 8;
 
        let sp = core.cpu.sp;
 
        ui.label(app_theme.format_register("S", sp, 16, false));
        app_theme.debugger_separator(ui);
 
        egui::Grid::new("stack_grid").num_columns(3).striped(true).show(ui, |ui| {
            // Walk from the most-recently-pushed byte (S+1) downward through
            // STACK_DEPTH rows. Each row's address is bank-0-only by definition
            // (65816 stack never leaves bank 0), so this can't wrap into another bank.
            for i in 0..STACK_DEPTH {
                let addr = sp.wrapping_add(1).wrapping_add(i as u16);
 
                // NOTE: bypassing the bus here intentionally. The stack is always
                // bank 0, never MMIO, and there's no side-effect-free peek available —
                // going through a real .read() risks mutating PPU/APU latch state just
                // to render this panel. This assumes core.aram is indexed identically
                // to bank-0 bus addresses (aram[addr] == bus address $00:addr).
                let value = core.wram[addr as usize];
 
                let tag = harness.stack_tracker.tag_at(addr);
 
                let is_top = i == 0;
                let addr_color = if is_top { app_theme.modified } else { app_theme.syntax_address };
                let addr_text = egui::RichText::new(format!("${:04X}", addr))
                    .monospace()
                    .color(addr_color);
                ui.label(addr_text);
 
                let value_color = if is_top { app_theme.modified } else { app_theme.syntax_number };
                ui.label(egui::RichText::new(format!("{:02X}", value)).monospace().color(value_color));
 
                let label = match tag {
                    Some(t) => t.cause.byte_label(t.offset_in_group, t.group_size),
                    None => "--".to_string(),
                };
                let label_color = if is_top { app_theme.modified } else { app_theme.text_muted };
                ui.label(egui::RichText::new(label).monospace().color(label_color));
 
                ui.end_row();
            }
        });
    }
    
    fn breakpoints_section(&mut self, ui: &mut egui::Ui, core: &mut Snemulator, harness: &mut MainDebugHarness) {
        let breakpoints = &mut harness.breakpoints;
        
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Breakpoints")
                    .strong()
                    .size(14.0)
            );
            
            if ui.button("Clear All").clicked() {
                breakpoints.clear();
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Add:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.bp_input).hint_text("XXXXXX").char_limit(6)
            );
            let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Add").clicked() || submitted {
                if let Ok(addr) = u32::from_str_radix(self.bp_input.trim(), 16) {
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

        let mut to_remove: Option<u32> = None;
        let mut sorted: Vec<u32> = breakpoints.iter().copied().collect();

        sorted.sort();

        egui::ScrollArea::vertical().id_salt("bp_scroll").show(ui, |ui| {
            for group in sorted.chunks(8) {
                ui.horizontal(|ui| {
                    for &breakpoint in group {
                        ui.horizontal(|ui| {
                            if ui.small_button("❌").clicked() {
                                to_remove = Some(breakpoint);
                            }
                            // Clicking the address jumps the disassembly view to it
                            if ui.button(egui::RichText::new(format!("{:06X}", breakpoint)).monospace()).clicked() {
                                let pc = ((core.cpu.pb as u32) << 16) | core.cpu.pc as u32;
                                self.disasm.follow_pc = breakpoint == pc;
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
    
    fn update_disasm(&mut self, core: &Snemulator) {
        let options = self.disasm.options.clone();

        self.disasm.update(core, &options);
    }

    fn format_label_line(&self, app_theme: &AppTheme, label: &str) -> egui::text::LayoutJob {
        use egui::text::{LayoutJob, TextFormat};
        
        let mut job = LayoutJob::default();
        let mono = egui::FontId::monospace(13.0);
        
        job.append(
            &format!(".{}:", label),
            0.0,
            TextFormat {
                font_id: mono,
                color: app_theme.syntax_label,
                ..Default::default()
            },
        );
        
        job
    }

    /// Create a highlighted job for disassembly view
    pub fn format_disassembly_line(
        &self,
        app_theme: &AppTheme,
        line: &disassembler::DisasmLine,
        comment: Option<&str>,
        is_current: bool,
        has_breakpoint: bool,
        is_modified: bool,
    ) -> egui::text::LayoutJob {
        use egui::text::{LayoutJob, TextFormat};
        
        let mut job = LayoutJob::default();
        let mono = egui::FontId::monospace(13.0);
        
        // Address
        job.append(
            &format!("{:06X}  ", line.addr),
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: app_theme.syntax_address,
                ..Default::default()
            },
        );
        
        // Raw bytes (up to 4 bytes shown)
        let bytes_str: String = line.bytes
            .iter()
            .take(4)
            .map(|b| format!("{:02X} ", b))
            .collect();
        job.append(
            &format!("{:<12}", bytes_str),
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: app_theme.text_muted,
                ..Default::default()
            },
        );
        
        // Mnemonic
        job.append(
            &format!("{:<6}", line.mnemonic),
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: app_theme.syntax_opcode,
                ..Default::default()
            },
        );

        if let Some(operand) = line.operand.as_ref() {
            let (text, color) = match operand.kind {
                disassembler::DisasmOperandKind::Address => {
                    if self.disasm.options.show_symbols {
                        if let Some(label) = self.disasm.symbols.get_address_label(operand.value) {
                            (label, app_theme.syntax_label)
                        } else {
                            (operand.text.as_str(), app_theme.syntax_address)
                        }
                    } else {
                        (operand.text.as_str(), app_theme.syntax_address)
                    }
                }
                disassembler::DisasmOperandKind::Number => (operand.text.as_str(), app_theme.syntax_number),
                disassembler::DisasmOperandKind::Register => (operand.text.as_str(), app_theme.syntax_register),
            };

            // Operand (with register/number highlighting)
            job.append(
                text,
                0.0,
                TextFormat {
                    font_id: mono.clone(),
                    color,
                    ..Default::default()
                },
            );
        }
        
        // Comment
        if let Some(comment) = comment {
            job.append(
                &format!("  ; {}", comment),
                0.0,
                TextFormat {
                    font_id: mono.clone(),
                    color: app_theme.syntax_comment,
                    ..Default::default()
                },
            );
        }
        
        // Background highlighting for current line or breakpoint
        if is_current || has_breakpoint || is_modified {
            let color = if has_breakpoint {
                app_theme.breakpoint_bg
            } else if is_modified {
                app_theme.modified_bg
            } else {
                app_theme.highlight_line
            };

            job.sections.iter_mut().for_each(|section| {
                section.format.background = color;
            });
        }
        
        job
    }
}

/// Parse a combined disassembly string into mnemonic and operand
fn parse_disasm_str(disasm_str: &str) -> (&str, &str) {
    match disasm_str.split_once(char::is_whitespace) {
        Some((mnemonic, operand)) => (mnemonic, operand.trim()),
        None => (disasm_str, ""),
    }
}