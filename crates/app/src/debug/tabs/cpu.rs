use std::collections::HashSet;

use crate::debug::{breakpoints::BreakpointInfo, debugger::Debugger};
use common::app_utils::monospace_text;
use snemcore::{Snemulator, cartridge, scpu};

const DISASM_BLOCK_SIZE: usize = 64;

struct DisassemblyView {
    cached_lines: Option<Vec<scpu::disassembler::DisasmLine>>,
    // scroll_offset: usize,
    options: scpu::disassembler::DisassemblyOptions,
    follow_pc: bool,
    current_addr: u32,
}

impl DisassemblyView {
    fn new(rom_mapping_mode: cartridge::MappingMode) -> Self {        
        Self {
            cached_lines: None,
            // scroll_offset: 0,
            options: scpu::disassembler::DisassemblyOptions {
                use_hw_reg_names: true,
                show_rel_addr_dest: true,
                max_instr_count: DISASM_BLOCK_SIZE,
                forced_flag_x: None,
                forced_flag_m: None,
                forced_e: None,
                rom_mapping_mode,
            },
            follow_pc: true,
            current_addr: 0,
        }
    }

    fn update(&mut self,
        core: &Snemulator<Debugger>,
        options: &scpu::disassembler::DisassemblyOptions,
    ) {
        if self.follow_pc {
            self.current_addr = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;
        }

        self.cached_lines = Some(scpu::disassembler::disassemble_forward(core, options, self.current_addr));
    }
}

pub struct CpuTab {
    disasm: DisassemblyView,
    bp_input: String,
}

impl CpuTab {
    pub fn new(rom_mapping_mode: cartridge::MappingMode) -> Self {
        Self {
            disasm: DisassemblyView::new(rom_mapping_mode),
            bp_input: String::new(),
        }
    }
    
    pub fn breakpoint_hit(&mut self, addr: u32) {
        self.disasm.current_addr = addr;
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, jump_to_bps_on_hit: &mut bool) {
        self.update_disasm(core);
        
        let pc = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.disasm.options.use_hw_reg_names, "Use HW Reg Names");
                
                ui.checkbox(&mut self.disasm.options.show_rel_addr_dest, "Show Branch Dest Addr");

                ui.checkbox(&mut self.disasm.follow_pc, "Follow PC");

                if ui.button("Go to PC").clicked() {
                    self.disasm.current_addr = pc;
                    // self.disasm.follow_pc = true;
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

                let (m_text, x_text) = match self.disasm.options.forced_e {
                    Some(true) => {
                        ui.disable();

                        ("m8", "x8")
                    }
                    None if core.cpu.e => {
                        ui.disable();

                        ("m8", "x8")
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
                        (m_text, x_text)
                    }
                };

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
        });
        ui.separator();

        let available_height = ui.available_height();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                egui::ScrollArea::vertical().id_salt("disasm_scroll").min_scrolled_height(available_height).show(ui, |ui| {
                    let lines = self.disasm.cached_lines.take();
                    
                    if let Some(lines) = lines {
                        for line in lines {                            
                            let is_pc  = line.addr == pc;
                            let has_bp = core.probe.as_ref().unwrap().breakpoints.contains(&BreakpointInfo::new(line.addr));
                            let addr   = line.addr;

                            ui.horizontal(|ui| {
                                let dot = if has_bp { "🔴" } else { "⚪" };
                                if ui.small_button(dot).clicked() {
                                    if has_bp {
                                        core.probe.as_mut().unwrap().breakpoints.remove(&BreakpointInfo::new(addr));
                                    }
                                    else {
                                        self.add_breakpoint(addr, core);
                                    }
                                }

                                ui.label(" ");

                                let addr_text_col = if has_bp {
                                    egui::Color32::RED
                                } else if is_pc {
                                    egui::Color32::YELLOW
                                } else {
                                    ui.visuals().text_color()
                                };
                                let addr_text = egui::RichText::new(format!("{:06X}", line.addr))
                                    .monospace()
                                    .color(addr_text_col);
                                ui.label(addr_text);

                                let disasm_col = if has_bp && addr == self.disasm.current_addr {
                                    egui::Color32::RED
                                } else if is_pc {
                                    egui::Color32::YELLOW
                                } else {
                                    ui.visuals().text_color()
                                };
                                let bytes_str: String = line.bytes.iter().map(|b| format!("{:02X} ", b)).collect();
                                ui.label(egui::RichText::new(format!("{:<12}", bytes_str))
                                    .monospace()
                                    .color(disasm_col)
                                    .weak());

                                let disasm_text = egui::RichText::new(&line.disasm_str)
                                    .monospace()
                                    .color(disasm_col);
                                ui.label(disasm_text);

                                ui.add_space(10.0);
                            });
                        }
                    } else {
                        ui.label("No disassembly available");
                    }
                });
            });

            ui.vertical(|ui| {
                self.cpu_state_section(ui, core);
                
                ui.add_space(10.0);
                
                self.breakpoints_section(ui, core, jump_to_bps_on_hit);
            });
        });
    }
    
    fn cpu_state_section(&mut self, ui: &mut egui::Ui, core: &Snemulator<Debugger>) {
        ui.heading("CPU State");
        
        ui.separator();
        
        ui.horizontal(|ui| {
            let pb_text = egui::RichText::new(format!("PB: {:02X}", core.cpu.pb)).monospace();
            ui.label(pb_text);
            
            let pc_text = egui::RichText::new(format!("PC: {:04X}", core.cpu.pc)).monospace();
            ui.label(pc_text);
            
            let sp_text = egui::RichText::new(format!("SP: {:04X}", core.cpu.sp)).monospace();
            ui.label(sp_text);
            
            let db_text = egui::RichText::new(format!("DB: {:02X}", core.cpu.db)).monospace();
            ui.label(db_text);
            
            let dp_text = egui::RichText::new(format!("DP: {:04X}", core.cpu.dp)).monospace();
            ui.label(dp_text);
        });
        
        ui.horizontal(|ui| {
            let a_text = egui::RichText::new(format!("A: {:04X}", core.cpu.a)).monospace();
            ui.label(a_text);
            
            let x_text = egui::RichText::new(format!("X: {:04X}", core.cpu.x)).monospace();
            ui.label(x_text);
            
            let y_text = egui::RichText::new(format!("Y: {:04X}", core.cpu.y)).monospace();
            ui.label(y_text);
            
            let style = egui::Style::default();
            let mut status_str = egui::text::LayoutJob::default();
            
            let flag_col = |flag| if core.cpu.is_flag_set(flag) { egui::Color32::GREEN } else { egui::Color32::RED };
            
            let p_text = egui::RichText::new("P: ").color(ui.visuals().text_color()).monospace();
            let n_text = egui::RichText::new("N").color(flag_col(scpu::Flag::FlagN)).monospace();
            let v_text = egui::RichText::new("V").color(flag_col(scpu::Flag::FlagV)).monospace();
            let m_text = egui::RichText::new("M").color(flag_col(scpu::Flag::FlagM)).monospace();
            let x_text = egui::RichText::new("X").color(flag_col(scpu::Flag::FlagX)).monospace();
            let d_text = egui::RichText::new("D").color(flag_col(scpu::Flag::FlagD)).monospace();
            let i_text = egui::RichText::new("I").color(flag_col(scpu::Flag::FlagI)).monospace();
            let z_text = egui::RichText::new("Z").color(flag_col(scpu::Flag::FlagZ)).monospace();
            let c_text = egui::RichText::new("C").color(flag_col(scpu::Flag::FlagC)).monospace();
            
            p_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            n_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            v_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            m_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            x_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            d_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            i_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            z_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            c_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            
            ui.label(status_str);
        });
        
        ui.separator();
        
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
            let mut irq_pending = core.cpu.irq_pending;
            let mut nmi_pending = core.cpu.nmi_pending;
            
            ui.add_enabled(false,
                egui::Checkbox::new(&mut irq_pending, "IRQ Pending")
            );
            ui.add_enabled(false,
                egui::Checkbox::new(&mut nmi_pending, "NMI Pending")
            );
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label(monospace_text("(APU→CPU)".to_string()));
            ui.label(monospace_text(format!("APUIO0: {:02X}", core.apu_ports.apuio0)));
            ui.label(monospace_text(format!("APUIO1: {:02X}", core.apu_ports.apuio1)));
            ui.label(monospace_text(format!("APUIO2: {:02X}", core.apu_ports.apuio2)));
            ui.label(monospace_text(format!("APUIO3: {:02X}", core.apu_ports.apuio3)));
        });
        
        ui.horizontal(|ui| {
            ui.label(monospace_text("(CPU→APU)".to_string()));
            ui.label(monospace_text(format!("CPUIO0: {:02X}", core.apu_ports.cpuio0)));
            ui.label(monospace_text(format!("CPUIO1: {:02X}", core.apu_ports.cpuio1)));
            ui.label(monospace_text(format!("CPUIO2: {:02X}", core.apu_ports.cpuio2)));
            ui.label(monospace_text(format!("CPUIO3: {:02X}", core.apu_ports.cpuio3)));
        });
    }
    
    fn breakpoints_section(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, jump_to_bps_on_hit: &mut bool) {
        let breakpoints = &mut core.probe.as_mut().unwrap().breakpoints;
        
        ui.horizontal(|ui| {
            ui.heading("Breakpoints");
            
            if ui.button("Clear All").clicked() {
                breakpoints.clear();
            }
            
            ui.add_space(5.0);
            
            ui.checkbox(jump_to_bps_on_hit, "Show Breakpoints on Hit");
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
                    self.add_breakpoint(addr, core);
                    self.bp_input.clear();
                }
            }
        });
        ui.separator();
        
        let breakpoints = &mut core.probe.as_mut().unwrap().breakpoints; // To appease borrow checker

        if breakpoints.is_empty() {
            ui.label("No breakpoints set.");
            return;
        }

        let mut to_remove: Option<&BreakpointInfo> = None;
        let mut sorted: Vec<BreakpointInfo> = breakpoints.iter().copied().collect();
        sorted.sort_unstable_by_key(|bp| bp.addr);

        egui::ScrollArea::vertical().id_salt("bp_scroll").show(ui, |ui| {
            for group in sorted.chunks(5) {
                ui.horizontal(|ui| {
                    for breakpoint in group {
                        ui.horizontal(|ui| {
                            if ui.small_button("❌").clicked() {
                                to_remove = Some(breakpoint);
                            }
                            // Clicking the address jumps the disassembly view to it
                            if ui.button(egui::RichText::new(format!("{:06X}", breakpoint.addr)).monospace()).clicked() {
                                let pc = ((core.cpu.pb as u32) << 16) | core.cpu.pc as u32;
                                self.disasm.follow_pc = breakpoint.addr == pc;
                                self.disasm.current_addr = breakpoint.addr;
                                self.disasm.options.forced_flag_m = Some(breakpoint.force_m);
                                self.disasm.options.forced_flag_x = Some(breakpoint.force_x);
                                self.disasm.options.forced_e = Some(breakpoint.force_e);
                            }
                        });
                    }
                });
            }
            
        });

        if let Some(breakpoint) = to_remove {
            core.probe.as_mut().unwrap().breakpoints.remove(breakpoint);
        }
    }
    
    fn update_disasm(&mut self, core: &Snemulator<Debugger>) {
        let options = self.disasm.options.clone();

        self.disasm.update(core, &options);
    }

    fn add_breakpoint(&mut self, addr: u32, core: &mut Snemulator<Debugger>) {
        let mut breakpoint = BreakpointInfo::new(addr);
        breakpoint.force_x = match self.disasm.options.forced_flag_x {
            Some(v) => v,
            None => core.cpu.is_flag_set(scpu::Flag::FlagX)
        };
        breakpoint.force_m = match self.disasm.options.forced_flag_m {
            Some(v) => v,
            None => core.cpu.is_flag_set(scpu::Flag::FlagM)
        };
        breakpoint.force_e = match self.disasm.options.forced_e {
            Some(v) => v,
            None => core.cpu.e
        };

        core.probe.as_mut().unwrap().breakpoints.insert(breakpoint);
    }
}

