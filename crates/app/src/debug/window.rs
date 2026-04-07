use anyhow::Result;
use snemcore::cartridge::MappingMode;
use snemcore::Snemulator;

use crate::app;
use crate::debug::debugger::Debugger;
// use crate::core;
use crate::debug::tabs;
use common::UiWindow;
// use crate::core::debug::breakpoints::BreakpointInfo;
// use crate::core::debug::watchpoints::CompiledGraph;
// use crate::core::debug::DebugAction;

const DEBUG_WINDOW_WIDTH: u32 = 800;
const DEBUG_WINDOW_HEIGHT: u32 = 600;
const HYPERSPEED_SPEEDUP: usize = 10;

pub enum DebugAction {
    SingleStep,
    StepFrame,
    TogglePause,
    Reset,
    HardReset,
}

pub struct DebugWindow {
    egui_window: Option<Box<UiWindow>>,
    cpu_tab: Box<tabs::CpuTab>,
    mem_tab: Box<tabs::MemoryTab>,
    ppu_tab: Box<tabs::PpuTab>,
    wp_tab: Box<tabs::WatchpointsTab>,
    selected_tab: tabs::DebugTab,
    jump_to_bps_on_hit: bool,
    jump_to_wps_on_hit: bool,
    hyperspeed_en: bool,
}

impl DebugWindow {
    pub fn new(
        video_subsystem: &sdl3::VideoSubsystem,
        rom_mapping_mode: MappingMode,
    ) -> Result<Self> {
        let mut egui_window = Box::new(UiWindow::new(
            video_subsystem,
            "Debug",
            DEBUG_WINDOW_WIDTH,
            DEBUG_WINDOW_HEIGHT,
        )?);

        log::debug!("Debugging started");

        let mut ppu_tab = None;

        egui_window.with_painter(|_, painter| {
            ppu_tab = Some(tabs::PpuTab::new(painter));
        });

        let ppu_tab = Box::new(ppu_tab.unwrap());

        let mut debug_window = Self {
            egui_window: None,
            cpu_tab: Box::new(tabs::CpuTab::new(rom_mapping_mode)),
            mem_tab: Box::new(tabs::MemoryTab::new()),
            ppu_tab,
            wp_tab: Box::new(tabs::WatchpointsTab::new()),
            selected_tab: tabs::DebugTab::Cpu,
            jump_to_bps_on_hit: true,
            jump_to_wps_on_hit: true,
            hyperspeed_en: false,
        };

        debug_window.egui_window = Some(egui_window);

        Ok(debug_window)
    }

    pub fn update_and_render(
        &mut self,
        core: &mut Snemulator<Debugger>,
        app_state: &mut app::AppState,
        frame_buffer: &mut [u8],
        audio_buffer: &mut Vec<i16>,
    ) -> app::AppAction {
        let mut app_action = app::AppAction::Continue;

        if !app_state.is_paused {
            if self.hyperspeed_en {       
                core.probe.as_mut().unwrap().update_textures = false;
                
                for _ in 0..HYPERSPEED_SPEEDUP-1 {
                    core.run_frame_no_output();
                }
                
                core.probe.as_mut().unwrap().update_textures = true;
                
                let mut fake_audio_buffer = Vec::new();
                core.run_frame(frame_buffer, &mut fake_audio_buffer);
            } else {
                core.probe.as_mut().unwrap().update_textures = true;
                core.run_frame(frame_buffer, audio_buffer);
            }            
            
            if core.probe.as_ref().unwrap().breakpoint_hit {
                core.probe.as_mut().unwrap().breakpoint_hit = false;
                app_state.is_paused = true;
                self.breakpoint_hit(core);
            }
        }

        let mut egui_window = self.egui_window.take().unwrap();
        let mut debug_action: Option<DebugAction> = None;

        let full_output = Some(egui_window.update_ui(|ctx| {
            egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for tab in [
                        tabs::DebugTab::Cpu,
                        tabs::DebugTab::Memory,
                        tabs::DebugTab::Ppu,
                        tabs::DebugTab::Watchpoints,
                    ] {
                        ui.selectable_value(&mut self.selected_tab, tab, tab.label());
                    }
                });
            });

            egui::TopBottomPanel::top("commands").show(ctx, |ui| {
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("Step Instruction").clicked() {
                        debug_action = Some(DebugAction::SingleStep);
                    }

                    if ui.button("Step Frame").clicked() {
                        debug_action = Some(DebugAction::StepFrame);
                    }

                    if ui.button("Reset").clicked() {
                        debug_action = Some(DebugAction::Reset);
                    }

                    if ui.button("Hard Reset").clicked() {
                        debug_action = Some(DebugAction::HardReset);
                    }
                    
                    let hyperspeed_text = if self.hyperspeed_en { "Disable Hyperspeed" } else { "Enable Hyperspeed" };
                    ui.toggle_value(&mut self.hyperspeed_en, hyperspeed_text)
                        .on_hover_text(format!("If enabled, emulator will run at {}x speed, but with no audio and reduced video output", HYPERSPEED_SPEEDUP));

                    if app_state.is_paused && ui.button("Resume").clicked() {
                        debug_action = Some(DebugAction::TogglePause);
                    }

                    if !app_state.is_paused && ui.button("Pause").clicked() {
                        debug_action = Some(DebugAction::TogglePause);
                    }

                    ui.label(format!("Frame: {}", core.frame));

                    ui.label(format!("Cycles: {}", core.total_cycles));
                    
                    ui.label(format!("FPS: {:.0}", app_state.fps));
                });

                ui.add_space(3.0);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                match self.selected_tab {
                    tabs::DebugTab::Cpu => {
                        self.cpu_tab.render(ui, core, &mut self.jump_to_bps_on_hit)
                    }
                    tabs::DebugTab::Memory => self.mem_tab.render(ui, core),
                    tabs::DebugTab::Ppu => self.ppu_tab.render(ui, core),
                    tabs::DebugTab::Watchpoints => {
                        self.wp_tab.render(ui, core, app_state)
                    }
                    _ => {}
                };
            });
        }));

        let full_output = full_output.unwrap();

        egui_window.clear();
        egui_window.render(full_output);

        self.egui_window = Some(egui_window);

        if let Some(action) = debug_action {
            match action {
                DebugAction::TogglePause => {
                    app_action = app::AppAction::TogglePause;
                }
                DebugAction::Reset => {
                    app_action = app::AppAction::ResetCore;
                }
                DebugAction::HardReset => {
                    app_action = app::AppAction::PowerOnCore;
                }
                DebugAction::SingleStep if app_state.is_paused => {
                    core.probe.as_mut().unwrap().update_textures = true;
                    core.cycle_instruction(frame_buffer);
                    
                    if core.probe.as_ref().unwrap().breakpoint_hit {
                        core.probe.as_mut().unwrap().breakpoint_hit = false;
                        self.breakpoint_hit(core);
                    }
                }
                
                _ => {}
            }
        }
        
        // match debug_action {
        //     DebugAction::SingleStep if app_state.is_paused => {
        //         // let mut layer_buffers = self.ppu_tab.layer_buffers();

        //         // match snem_core.debug_step_instruction(
        //         //     frame_buffer,
        //         //     audio_buffer,
        //         //     self.breakpoints(),
        //         //     self.watchpoints(),
        //         //     &mut layer_buffers,
        //         // ) {
        //         //     DebugAction::BreakpointHit => {
        //         //         self.breakpoint_hit(&snem_core, app_state);
        //         //     }
        //         //     DebugAction::WatchpointHit => {
        //         //         self.watchpoint_hit(app_state);
        //         //     }
        //         //     _ => {}
        //         // }

        //         // self.ppu_tab.restore_buffers(layer_buffers);
        //         // clear_watchpoints = true;
        //     }
        //     DebugAction::StepFrame if app_state.is_paused => {
        //         // let mut layer_buffers = self.ppu_tab.layer_buffers();

        //         // match snem_core.debug_run_frame(
        //         //     frame_buffer,
        //         //     audio_buffer,
        //         //     self.breakpoints(),
        //         //     self.watchpoints(),
        //         //     &mut layer_buffers,
        //         // ) {
        //         //     DebugAction::BreakpointHit => {
        //         //         app_state.is_paused = true;
        //         //         self.breakpoint_hit(&snem_core, app_state);
        //         //     }
        //         //     DebugAction::WatchpointHit => {
        //         //         app_state.is_paused = true;
        //         //         self.watchpoint_hit(app_state);
        //         //     }
        //         //     _ => {}
        //         // }

        //         // self.ppu_tab.restore_buffers(layer_buffers);
        //         // clear_watchpoints = true;
        //     }
        //     DebugAction::TogglePause => {
        //         app_action = app::AppAction::TogglePause;
        //     }
        //     DebugAction::Reset => {
        //         app_action = app::AppAction::ResetCore;
        //     }
        //     DebugAction::HardReset => {
        //         app_action = app::AppAction::PowerOnCore;
        //     }
        //     _ => {}
        // }

        // if clear_watchpoints {
        //     self.wp_tab.clear_compiled_watchpoints();
        // }

        app_action
    }

    pub fn id(&self) -> u32 {
        self.egui_window.as_ref().unwrap().window().id()
    }

    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers) {
        self.egui_window
            .as_mut()
            .unwrap()
            .handle_sdl_mouse_event(event, modifiers);
        self.egui_window
            .as_mut()
            .unwrap()
            .handle_sdl_keyboard_event(event);
    }

    pub fn breakpoint_hit(&mut self, core: &Snemulator<Debugger>) {
        self.cpu_tab.breakpoint_hit((core.cpu.pb as u32) << 16 | core.cpu.pc as u32);
        
        if self.jump_to_bps_on_hit {
            self.selected_tab = tabs::DebugTab::Cpu;
        }
    }

    // pub fn watchpoint_hit(&mut self, app_state: &mut app::AppState) {
    //     if self.wp_tab.watchpoints_enabled() {
    //         app_state.is_paused = true;

    //         if self.jump_to_wps_on_hit {
    //             self.selected_tab = tabs::DebugTab::Watchpoints;
    //         }
    //     }
    // }

    // pub fn breakpoints(&self) -> &std::collections::HashSet<BreakpointInfo> {
    //     &self.cpu_tab.breakpoints()
    // }

    // pub fn watchpoints(&self) -> &CompiledGraph {
    //     self.wp_tab.watchpoints()
    // }

    // fn compile_watchpoints(&mut self, snem_core: &core::snemcore::Snemulator) {
    //     self.wp_tab.compile_watchpoints(snem_core);
    // }
}
