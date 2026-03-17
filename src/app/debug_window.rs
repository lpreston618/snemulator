use anyhow::Result;
use std::collections::HashSet;

use crate::app::debug::BreakpointInfo;
use crate::app::ui_window::UiWindow;
use crate::core::scpu::disassembler::{MemoryRegion, get_memory_region};
use crate::{app, core};

const DISASM_BLOCK_SIZE: usize = 64;
const DEBUG_WINDOW_WIDTH: u32 = 800;
const DEBUG_WINDOW_HEIGHT: u32 = 600;

pub struct DisassemblyView {
    cached_lines: Option<Vec<core::scpu::disassembler::DisasmLine>>,
    // scroll_offset: usize,
    breakpoints: HashSet<BreakpointInfo>,
    options: core::scpu::disassembler::DisassemblyOptions,
    follow_pc: bool,
    current_addr: u32,
}

impl DisassemblyView {
    fn new(rom_mapping_mode: core::cartridge::MappingMode) -> Self {
        Self {
            cached_lines: None,
            // scroll_offset: 0,
            breakpoints: HashSet::new(),
            options: core::scpu::disassembler::DisassemblyOptions {
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

    /// Call when PC changes significantly or user navigates manually.
    /// Decodes `count` instructions forward from `start_addr`.
    pub fn decode_forward(
        start_addr: u32,
        memory: &[u8],
        memory_region: core::scpu::disassembler::MemoryRegion,
        options: &core::scpu::disassembler::DisassemblyOptions,
        snem_core: &core::snemcore::Snemulator,
    ) -> Vec<core::scpu::disassembler::DisasmLine> {
        let mem = core::scpu::disassembler::MemBlock {
            data: memory,
            start_addr: 0,
            bank: (start_addr >> 16) as u8,
        };

        let flag_e = if options.forced_e.is_some() {
            options.forced_e.unwrap()
        } else {
            snem_core.cpu.e
        };

        let flag_m = if options.forced_flag_m.is_some() {
            options.forced_flag_m.unwrap() | flag_e
        } else {
            snem_core.cpu.is_flag_set(core::scpu::Flag::FlagM) | flag_e
        };

        let flag_x = if options.forced_flag_x.is_some() {
            options.forced_flag_x.unwrap() | flag_e
        } else {
            snem_core.cpu.is_flag_set(core::scpu::Flag::FlagX) | flag_e
        };

        let state = core::scpu::disassembler::ExecuteState {
            dp: snem_core.cpu.dp,
            pc: start_addr as u16,
            flag_m,
            flag_x,
            memory_region,
        };

        core::scpu::disassembler::disassemble_block(&mem, options, Some(state))
    }

    pub fn update(&mut self,
        pc: u32,
        memory: &[u8],
        memory_region: core::scpu::disassembler::MemoryRegion,
        options: &core::scpu::disassembler::DisassemblyOptions,
        snem_core: &core::snemcore::Snemulator
    ) {
        if self.follow_pc {
            self.current_addr = pc;
        }

        self.cached_lines = Some(Self::decode_forward(self.current_addr, memory, memory_region, options, snem_core));
    }
}

// pub struct ChrViewer {
//     texture: Option<glow::Texture>,
//     bpp_mode: core::sppu::ColorDepth,
//     palette_index: usize,
// }

// impl ChrViewer {
//     // Call once during DebugWindow::new(), same pattern as game_texture init
//     pub fn init_texture(gl: &glow::Context) -> Option<glow::Texture> { ... }

//     // Decode VRAM tiles -> RGBA pixels, upload via tex_sub_image_2d
//     pub fn update_texture(&self, gl: &glow::Context, vram: &[u8], cgram: &[u8]) {
//         let mut pixels = vec![0u8; TILES_WIDE * TILES_TALL * 8 * 8 * 4];
//         // decode tiles from vram into pixels using self.bpp_mode
//         // ...
//         unsafe {
//             gl.bind_texture(glow::TEXTURE_2D, self.texture);
//             gl.tex_sub_image_2d( ... pixels ... );
//         }
//     }

//     // In egui, display with egui::Image using a TextureId registered via egui_painter
//     // NOTE: You'll need to register the raw GL texture with egui_glow to get a TextureId
// }

pub struct DebugWindow {
    egui_window: Option<UiWindow>,
    disasm: DisassemblyView,
    // chr_viewer: ChrViewer,
    selected_tab: DebugTab,
    bp_input: String,
}

#[derive(PartialEq, Clone, Copy)]
enum DebugTab { Cpu, Memory, Disassembly, ChrRam, Ppu, Breakpoints }

impl DebugTab {
    fn label(&self) -> &'static str {
        match self {
            DebugTab::Cpu         => "CPU",
            DebugTab::Memory      => "Memory",
            DebugTab::Disassembly => "Disassembly",
            DebugTab::ChrRam      => "CHR RAM",
            DebugTab::Ppu         => "PPU",
            DebugTab::Breakpoints => "Breakpoints",
        }
    }
}

impl DebugWindow {
    pub fn new(video_subsystem: &sdl3::VideoSubsystem, rom_mapping_mode: core::cartridge::MappingMode) -> Result<Self> {
        let egui_window = UiWindow::new(
            video_subsystem,
            "Debug",
            DEBUG_WINDOW_WIDTH,
            DEBUG_WINDOW_HEIGHT,
        )?;

        Ok(Self {
            egui_window: Some(egui_window),
            disasm: DisassemblyView::new(rom_mapping_mode),
            // chr_viewer: ChrViewer::new(),
            selected_tab: DebugTab::Cpu,
            // mem_region: MemoryRegion::default(),
            bp_input: String::new(),
        })
    }

    pub fn update_and_render(
        &mut self,
        snem_core: &core::snemcore::Snemulator,
        app_state: &app::AppState
    ) -> app::DebugAction {
        // let gl = self.egui_window.gl();
        // self.chr_viewer.update_texture(gl, snem_core.vram(), snem_core.cgram());

        let mut egui_window = self.egui_window.take().unwrap();
        let mut debug_action = app::DebugAction::None;

        let full_output = egui_window.update_ui(|ctx| {
            egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for tab in [
                        DebugTab::Cpu,
                        DebugTab::Memory,
                        DebugTab::Disassembly,
                        DebugTab::ChrRam,
                        DebugTab::Ppu,
                        DebugTab::Breakpoints
                    ] {
                        ui.selectable_value(&mut self.selected_tab, tab, tab.label());
                    }
                });
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                debug_action = match self.selected_tab {
                    // DebugTab::Memory     => self.render_memory_viewer(ui, snes),
                    DebugTab::Disassembly => {
                        self.update_disassembly(snem_core);
                        self.render_cpu_tab(ui, snem_core, app_state)
                    },
                    // DebugTab::ChrRam     => self.render_chr_viewer(ui),
                    // DebugTab::Cpu        => self.render_cpu_state(ui, snes),
                    DebugTab::Breakpoints => {
                        // self.render_breakpoints_tab(ui, snem_core);
                        app::DebugAction::None
                    },
                    _ => app::DebugAction::None,
                };
            });
        });

        egui_window.clear();
        egui_window.render(full_output);

        self.egui_window = Some(egui_window);
        debug_action
    }

    fn update_disassembly(&mut self, snem_core: &core::snemcore::Snemulator) {
        let options = self.disasm.options.clone();
        let pc = (snem_core.cpu.pb as u32) << 16 | snem_core.cpu.pc as u32;

        let region = get_memory_region(pc);

        let memory = match region {
            MemoryRegion::LowRamMirror => &snem_core.wram[..0x2000],
            MemoryRegion::Ram => &snem_core.wram[..],
            MemoryRegion::Rom => &snem_core.rom_slice(),
            _ => {
                log::warn!("Tried to disassemble invalid memory region at: {:06X}", pc);
                return;
            },
        };

        self.disasm.update(pc, memory, region, &options, snem_core);
    }

    fn render_cpu_tab(
        &mut self,
        ui: &mut egui::Ui,
        snem_core: &core::snemcore::Snemulator,
        app_state: &app::AppState
    ) -> app::DebugAction {
        let mut debug_action = app::DebugAction::None;

        let pc = (snem_core.cpu.pb as u32) << 16 | snem_core.cpu.pc as u32;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.disasm.options.use_hw_reg_names, "Use HW Reg Names");
                
                ui.checkbox(&mut self.disasm.options.show_rel_addr_dest, "Show Branch Dest Addr");

                ui.checkbox(&mut self.disasm.follow_pc, "Follow PC");

                if ui.button("Go to PC").clicked() {
                    self.disasm.current_addr = pc;
                    self.disasm.follow_pc = true;
                    self.disasm.options.forced_flag_x = None;
                    self.disasm.options.forced_flag_m = None;
                    self.disasm.options.forced_e = None;
                }

                if ui.button("Step Instruction").clicked() {
                    debug_action = app::DebugAction::SingleStep;
                }

                if ui.button("Step Frame").clicked() {
                    debug_action = app::DebugAction::StepFrame;
                }

                let pause_text = if app_state.is_paused { "Resume" } else { "Pause" };
                if ui.button(pause_text).clicked() {
                    debug_action = app::DebugAction::TogglePause;
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt(0)
                    .selected_text(
                        match self.disasm.options.forced_e {
                            Some(true) => "Emulation",
                            Some(false) => "Native",
                            None => if snem_core.cpu.e { "Emulation" } else { "Native" },
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
                    None if snem_core.cpu.e => {
                        ui.disable();

                        ("m8", "x8")
                    }
                    _ => {
                        let m_text = match self.disasm.options.forced_flag_m {
                            Some(true) => "m8",
                            Some(false) => "m16",
                            None => if snem_core.cpu.is_flag_set(core::scpu::Flag::FlagM) { "m8" } else { "m16" },
                        };
                        let x_text = match self.disasm.options.forced_flag_x {
                            Some(true) => "x8",
                            Some(false) => "x16",
                            None => if snem_core.cpu.is_flag_set(core::scpu::Flag::FlagX) { "x8" } else { "x16" },
                        };
                        (m_text, x_text)
                    }
                };

                egui::ComboBox::from_id_salt(1)
                    .selected_text(m_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.disasm.options.forced_flag_m, Some(true), "m8");
                        ui.selectable_value(&mut self.disasm.options.forced_flag_m, Some(false), "m16");
                        ui.selectable_value(&mut self.disasm.options.forced_flag_m, None, "Current in Program");
                    });

                egui::ComboBox::from_id_salt(2)
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
                            let has_bp = self.disasm.breakpoints.contains(&BreakpointInfo::new(line.addr));
                            let addr   = line.addr;

                            ui.horizontal(|ui| {
                                let dot = if has_bp { "🔴" } else { "⚪" };
                                if ui.small_button(dot).clicked() {
                                    if has_bp {
                                        self.disasm.breakpoints.remove(&BreakpointInfo::new(addr));
                                    }
                                    else {
                                        self.add_breakpoint(addr, snem_core);
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
                self.cpu_state_section(ui, snem_core);
                
                ui.add_space(10.0);
                
                self.breakpoints_section(ui, snem_core);
            });
        });

        debug_action
    }
    
    fn cpu_state_section(&mut self, ui: &mut egui::Ui, snem_core: &core::snemcore::Snemulator) {
        ui.heading("CPU State");

        ui.separator();

        ui.horizontal(|ui| {
            let pb_text = egui::RichText::new(format!("PB: {:02X}", snem_core.cpu.pb)).monospace();
            ui.label(pb_text);

            let pc_text = egui::RichText::new(format!("PC: {:04X}", snem_core.cpu.pc)).monospace();
            ui.label(pc_text);

            let sp_text = egui::RichText::new(format!("SP: {:04X}", snem_core.cpu.sp)).monospace();
            ui.label(sp_text);

            let db_text = egui::RichText::new(format!("DB: {:02X}", snem_core.cpu.db)).monospace();
            ui.label(db_text);

            let dp_text = egui::RichText::new(format!("DP: {:04X}", snem_core.cpu.dp)).monospace();
            ui.label(dp_text);
        });

        ui.horizontal(|ui| {
            let a_text = egui::RichText::new(format!("A: {:04X}", snem_core.cpu.a)).monospace();
            ui.label(a_text);

            let x_text = egui::RichText::new(format!("X: {:04X}", snem_core.cpu.x)).monospace();
            ui.label(x_text);

            let y_text = egui::RichText::new(format!("Y: {:04X}", snem_core.cpu.y)).monospace();
            ui.label(y_text);

            let style = egui::Style::default();
            let mut status_str = egui::text::LayoutJob::default();

            let flag_col = |flag| if snem_core.cpu.is_flag_set(flag) { egui::Color32::GREEN } else { egui::Color32::RED };

            let p_text = egui::RichText::new("P: ").color(ui.visuals().text_color()).monospace();
            let n_text = egui::RichText::new("N").color(flag_col(core::scpu::Flag::FlagN)).monospace();
            let v_text = egui::RichText::new("V").color(flag_col(core::scpu::Flag::FlagV)).monospace();
            let m_text = egui::RichText::new("M").color(flag_col(core::scpu::Flag::FlagM)).monospace();
            let x_text = egui::RichText::new("X").color(flag_col(core::scpu::Flag::FlagX)).monospace();
            let d_text = egui::RichText::new("D").color(flag_col(core::scpu::Flag::FlagD)).monospace();
            let i_text = egui::RichText::new("I").color(flag_col(core::scpu::Flag::FlagI)).monospace();
            let z_text = egui::RichText::new("Z").color(flag_col(core::scpu::Flag::FlagZ)).monospace();
            let c_text = egui::RichText::new("C").color(flag_col(core::scpu::Flag::FlagC)).monospace();

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
            let mut halted = snem_core.cpu.halted;
            let mut stopped = snem_core.cpu.stopped;
            let mut waiting_for_interrupt = snem_core.cpu.waiting_for_interrupt;
            
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
            let mut irq_pending = snem_core.cpu.irq_pending;
            let mut nmi_pending = snem_core.cpu.nmi_pending;
            
            ui.add_enabled(false,
                egui::Checkbox::new(&mut irq_pending, "IRQ Pending")
            );
            ui.add_enabled(false,
                egui::Checkbox::new(&mut nmi_pending, "NMI Pending")
            );
        });
    }

    fn breakpoints_section(&mut self, ui: &mut egui::Ui, snem_core: &core::snemcore::Snemulator) {
        ui.horizontal(|ui| {
            ui.heading("Breakpoints");
            if ui.button("Clear All").clicked() {
                self.disasm.breakpoints.clear();
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
                    self.add_breakpoint(addr, snem_core);
                    self.bp_input.clear();
                }
            }
        });
        ui.separator();

        if self.disasm.breakpoints.is_empty() {
            ui.label("No breakpoints set.");
            return;
        }

        let mut to_remove: Option<&BreakpointInfo> = None;
        let mut sorted: Vec<BreakpointInfo> = self.disasm.breakpoints.iter().copied().collect();
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
                                let pc = ((snem_core.cpu.pb as u32) << 16) | snem_core.cpu.pc as u32;
        
                                self.selected_tab = DebugTab::Disassembly;
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
            self.disasm.breakpoints.remove(breakpoint);
        }
    }

    fn add_breakpoint(&mut self, addr: u32, snem_core: &core::snemcore::Snemulator) {
        let mut breakpoint = BreakpointInfo::new(addr);
        breakpoint.force_x = match self.disasm.options.forced_flag_x {
            Some(v) => v,
            None => snem_core.cpu.is_flag_set(core::scpu::Flag::FlagX)
        };
        breakpoint.force_m = match self.disasm.options.forced_flag_m {
            Some(v) => v,
            None => snem_core.cpu.is_flag_set(core::scpu::Flag::FlagM)
        };
        breakpoint.force_e = match self.disasm.options.forced_e {
            Some(v) => v,
            None => snem_core.cpu.e
        };

        self.disasm.breakpoints.insert(breakpoint);
    }

    pub fn id(&self) -> u32 {
        self.egui_window.as_ref().unwrap().window().id()
    }

    pub fn handle_event(&mut self, event: &sdl3::event::Event) {
        self.egui_window.as_mut().unwrap().handle_sdl_mouse_event(event);
        self.egui_window.as_mut().unwrap().handle_sdl_keyboard_event(event);
    }

    pub fn breakpoints(&self) -> &HashSet<BreakpointInfo> {
        &self.disasm.breakpoints
    }
}
