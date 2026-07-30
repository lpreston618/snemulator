use anyhow::Result;
use snemcore::Snemulator;

use crate::app::{self, AppAction};
use crate::app::audio::AudioManager;
use crate::debug::harness::{MainDebugHarness, StopCondition};
// use crate::core;
use crate::debug::tabs;
use crate::debug::icons::{self, IconExt};
use crate::app::theme::AppTheme;
use crate::ui_window::UiWindow;

pub const DEBUG_WINDOW_WIDTH: u32 = 800;
pub const DEBUG_WINDOW_HEIGHT: u32 = 600;

#[derive(Clone, Copy, PartialEq)]
enum DebugTab {
    Cpu,
    Spc,
    Dma,
    Memory,
    Ppu,
    Sdsp,
    Audio,
    Brr,
}

impl DebugTab {
    fn label(self) -> &'static str {
        match self {
            DebugTab::Cpu => "CPU",
            DebugTab::Spc => "SPC",
            DebugTab::Dma => "DMA",
            DebugTab::Memory => "Memory",
            DebugTab::Ppu => "PPU",
            DebugTab::Sdsp => "S-DSP",
            DebugTab::Audio => "Audio",
            DebugTab::Brr => "BRR",
        }
    }

    fn all() -> &'static [Self] {
        &[
            DebugTab::Cpu,
            DebugTab::Spc,
            DebugTab::Dma,
            DebugTab::Memory,
            DebugTab::Ppu,
            DebugTab::Sdsp,
            DebugTab::Audio,
            DebugTab::Brr,
        ]
    }
}

pub enum DebugAction {
    SingleStep,
    StepFrame,
    SetPaused(bool),
    Reset,
    HardReset,
}

pub struct DebugWindow {
    egui_window: Option<Box<UiWindow>>,
    cpu_tab: Box<tabs::cpu::CpuTab>,
    spc_tab: Box<tabs::ssmp::spc::SpcTab>,
    dma_tab: Box<tabs::dma::DmaTab>,
    mem_tab: Box<tabs::mem::MemoryTab>,
    ppu_tab: Box<tabs::ppu::PpuTab>,
    sdsp_tab: Box<tabs::ssmp::sdsp::SdspTab>,
    audio_tab: Box<tabs::ssmp::audio::SdspAudioTab>,
    brr_tab: Box<tabs::ssmp::brr::SdspBrrTab>,
    // wp_tab: Box<tabs::WatchpointsTab>,
    selected_tab: DebugTab,
    // jump_to_bps_on_hit: bool,
    // jump_to_wps_on_hit: bool,
    // ff_frames: f32,
}

impl DebugWindow {
    pub fn new(
        egui_window: UiWindow,
    ) -> Result<Self> {
        log::trace!("Debugging started");

        let mut debug_window = Self {
            egui_window: None,
            cpu_tab: Box::new(tabs::cpu::CpuTab::new()),
            spc_tab: Box::new(tabs::ssmp::spc::SpcTab::new()),
            dma_tab: Box::new(tabs::dma::DmaTab::new()),
            mem_tab: Box::new(tabs::mem::MemoryTab::new()),
            ppu_tab: Box::new(tabs::ppu::PpuTab::new()),
            sdsp_tab: Box::new(tabs::ssmp::sdsp::SdspTab::new()),
            audio_tab: Box::new(tabs::ssmp::audio::SdspAudioTab::new()),
            brr_tab: Box::new(tabs::ssmp::brr::SdspBrrTab::new()),
            // wp_tab: Box::new(tabs::WatchpointsTab::new()),
            selected_tab: DebugTab::Cpu,
            // jump_to_bps_on_hit: true,
            // jump_to_wps_on_hit: true,
            // ff_frames: 0.0,
        };

        debug_window.egui_window = Some(Box::new(egui_window));

        Ok(debug_window)
    }

    pub fn resume(&mut self) {
        self.audio_tab.playing_track = None;
    }

    pub fn set_theme(&mut self, app_theme: &AppTheme) {
        app_theme.apply(&self.egui_window.as_ref().unwrap().egui_ctx);
    }

    pub fn update_and_render(
        &mut self,
        core: &mut Snemulator,
        app_state: &mut app::AppState,
        app_theme: &AppTheme,
        harness: &mut MainDebugHarness,
        audio_manager: &mut AudioManager,
    ) -> Option<AppAction> {
        let mut app_action: Option<AppAction> = None;

        let mut egui_window = self.egui_window.take().unwrap();
        let mut debug_action: Option<DebugAction> = None;

        let full_output = egui_window.update_ui(|ui| {
            egui::Panel::top("tabs").show(ui, |ui| {
                ui.horizontal(|ui| {
                    for &tab in DebugTab::all() {
                        ui.selectable_value(&mut self.selected_tab, tab, tab.label());
                    }
                });
            });

            debug_action = self.show_toolbar(ui, app_state, app_theme, core, harness);

            egui::CentralPanel::default().show(ui, |ui| {
                match self.selected_tab {
                    DebugTab::Cpu => self.cpu_tab.render(ui, core, harness, app_theme),
                    DebugTab::Spc => self.spc_tab.render(ui, core, harness, app_theme),
                    DebugTab::Dma => self.dma_tab.render(ui, core, harness, app_state, app_theme, &mut debug_action),
                    DebugTab::Memory => self.mem_tab.render(ui, core, app_theme),
                    DebugTab::Ppu => self.ppu_tab.render(ui, core, harness, app_theme),
                    DebugTab::Sdsp => self.sdsp_tab.render(ui, core, harness, app_state, app_theme, &mut debug_action),
                    DebugTab::Audio => self.audio_tab.render(ui, harness, audio_manager, app_state, app_theme),
                    DebugTab::Brr => self.brr_tab.render(ui, core, harness, app_theme, app_state, &mut debug_action),
                    // DebugTab::Watchpoints => {
                    //     self.wp_tab.render(ui, core, app_state)
                    // }
                    // _ => {}
                };
            });
        });

        egui_window.clear(app_theme);
        egui_window.render(full_output);

        self.egui_window = Some(egui_window);

        if let Some(action) = debug_action {
            match action {
                DebugAction::SetPaused(paused) => {
                    app_action = Some(AppAction::SetPaused(paused));
                }
                DebugAction::Reset => {
                    app_action = Some(AppAction::ResetCore);
                }
                DebugAction::HardReset => {
                    app_action = Some(AppAction::PowerOnCore);
                }
                DebugAction::SingleStep if app_state.is_paused => {
                    // core.cycle_instruction(frame_buffer);
                    
                    // if core.probe.as_ref().unwrap().control.breakpoint_hit {
                    //     core.probe.as_mut().unwrap().control.breakpoint_hit = false;
                    //     self.breakpoint_hit(core);
                    // }
                    
                    // if core.probe.as_ref().unwrap().control.watchpoint_hit {
                    //     core.probe.as_mut().unwrap().control.watchpoint_hit = false;
                    // }
                }
                DebugAction::StepFrame if app_state.is_paused => {
                    // core.probe.as_mut().unwrap().control.update_textures = true;
                    // core.run_frame(frame_buffer, None);
                    
                    // if core.probe.as_ref().unwrap().control.breakpoint_hit {
                    //     core.probe.as_mut().unwrap().control.breakpoint_hit = false;
                    //     self.breakpoint_hit(core);
                    // }
                    
                    // if core.probe.as_ref().unwrap().control.watchpoint_hit {
                    //     core.probe.as_mut().unwrap().control.watchpoint_hit = false;
                    // }
                }
                
                _ => {}
            }
        } else {
            // if !app_state.is_paused {
            //     let mut probe = core.probe.take().unwrap();
                
            //     if probe.control.ff_en {
            //         probe.control.update_textures = false;
                    
            //         self.ff_frames += probe.control.ff_speed;
                    
            //         let frames_to_run = self.ff_frames as usize;
                    
            //         self.ff_frames -= frames_to_run as f32;
                    
            //         let frames_to_run = frames_to_run.saturating_sub(1);
                    
            //         core.probe = Some(probe);
                    
            //         for _ in 0..frames_to_run {
            //             core.run_frame(None, None);
            //         }
                    
            //         core.probe.as_mut().unwrap().control.update_textures = true;
                    
            //         core.run_frame(frame_buffer, None);
            //     } else {
            //         probe.control.update_textures = true;
    
            //         core.probe = Some(probe);
                    
            //         core.run_frame(frame_buffer, audio_buffer);
            //     }
                
            //     if core.probe.as_ref().unwrap().control.should_stop {
            //         app_state.is_paused = true;
            //     }
                
            //     self.handle_probe_events(core);
            // }
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

    fn show_toolbar(&mut self, ui: &mut egui::Ui, app_state: &app::AppState, app_theme: &AppTheme, core: &mut Snemulator, harness: &mut MainDebugHarness) -> Option<DebugAction> {
        let mut debug_action = None;

        egui::Panel::top("commands").show(ui, |ui| {
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                let (pause_continue_icon, pause_continue_text, pause_continue_tint) = if app_state.is_paused {
                    (icons::CONTINUE, "Continue", app_theme.icon_secondary)
                } else {
                    (icons::PAUSE, "Pause", app_theme.icon_tertiary)
                };

                if ui.icon_button_with_tint(pause_continue_icon, pause_continue_tint)
                    .on_hover_text(pause_continue_text)
                    .clicked() {
                    debug_action = Some(DebugAction::SetPaused(!app_state.is_paused));
                }

                let (single_step_text, single_step_stop_cond) = match self.selected_tab {
                    DebugTab::Brr | DebugTab::Audio | DebugTab::Sdsp => ("Next Sample", StopCondition::SampleGenerated),
                    DebugTab::Spc => ("Step SPC700", StopCondition::SpcInstruction),
                    _ => ("Single Step", StopCondition::AnyScpuCycle),
                };

                if ui.add_enabled_ui(app_state.is_paused, |ui| {
                        ui.icon_button(icons::SINGLE_STEP, app_theme)
                    }).inner
                    .on_hover_text(single_step_text)
                    .clicked() {
                    debug_action = Some(DebugAction::SetPaused(false));
                    harness.stop_condition = Some(single_step_stop_cond);
                }

                for (icon, text, stop_cond) in [
                    (icons::STEP_OVER, "Step Over", StopCondition::StepOverSubroutine { depth: 0 }),
                    (icons::STEP_INTO, "Step Into", StopCondition::ScpuInstruction),
                    (icons::STEP_OUT, "Step Out", StopCondition::StepOverSubroutine { depth: 1 }),
                    (icons::RUN_FRAME, "Run Frame", StopCondition::Frame),
                    (icons::RUN_UNTIL_INTERRUPT, "Run Until Interrupt", StopCondition::Interrupt),
                ] {
                    if ui.add_enabled_ui(app_state.is_paused, |ui| {
                            ui.icon_button(icon, app_theme)
                        }).inner
                        .on_hover_text(text)
                        .clicked() {
                        debug_action = Some(DebugAction::SetPaused(false));
                        harness.stop_condition = Some(stop_cond);
                    }
                }

                if ui.button("Reset").clicked() {
                    debug_action = Some(DebugAction::Reset);
                }

                if ui.button("Power On").clicked() {
                    debug_action = Some(DebugAction::HardReset);
                }
                
                ui.label(format!("Frame: {}", core.frame));

                ui.label(format!("Cycles: {}", core.total_cycles));
                
                ui.label(format!("FPS: {:.0}", app_state.fps));
            });

            ui.add_space(3.0);
        });

        debug_action
    }
    
    // fn handle_probe_events(&mut self, core: &mut Snemulator) -> AppAction {
    //     let action = core.do_with_probe(|probe, core| {
    //         let mut app_action = AppAction::Continue;
            
    //         if probe.control.breakpoint_hit {
    //             probe.control.breakpoint_hit = false;
    //             self.breakpoint_hit(core);
    //         }
            
    //         if probe.control.watchpoint_hit {
    //             probe.control.watchpoint_hit = false;
    //         }
            
    //         probe.resume_emulation();
            
    //         if probe.control.should_reset {
    //             probe.control.should_reset = false;
    //             app_action = AppAction::ResetCore;
    //         }
            
    //         app_action
    //     }).unwrap();
        
    //     action
    // }

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

    // pub fn breakpoint_hit(&mut self, core: &Snemulator<Debugger>) {
    //     self.cpu_tab.breakpoint_hit((core.cpu.pb as u32) << 16 | core.cpu.pc as u32);
        
    //     if self.jump_to_bps_on_hit {
    //         self.selected_tab = tabs::DebugTab::Cpu;
    //     }
    // }

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