use anyhow::Result;

use crate::app;
use crate::app::debug::LayerBuffers;
use crate::core;
use crate::app::debug::tabs;
use crate::app::ui_window::UiWindow;
use crate::app::debug::breakpoints::BreakpointInfo;
use crate::app::debug::watchpoints::types::CompiledGraph;

const DEBUG_WINDOW_WIDTH: u32 = 800;
const DEBUG_WINDOW_HEIGHT: u32 = 600;

pub enum DebugAction {
    SingleStep,
    StepFrame,
    TogglePause,
    BreakpointHit,
    WatchpointHit,
    Reset,
    HardReset,
    None,
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
}

impl DebugWindow {
    pub fn new(video_subsystem: &sdl3::VideoSubsystem, rom_mapping_mode: core::cartridge::MappingMode) -> Result<Self> {
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
        };
        
        debug_window.egui_window = Some(egui_window);
        
        Ok(debug_window)
    }

    pub fn update_and_render(
        &mut self,
        snem_core: &mut core::snemcore::Snemulator,
        app_state: &mut app::AppState,
        frame_buffer: &mut [u8],
        audio_buffer: &mut Vec<i16>,
    ) -> app::AppAction {
        
        let mut clear_watchpoints = false;
        let mut app_action = app::AppAction::Continue;
        
        if !app_state.is_paused { 
            let mut layer_buffers = self.ppu_tab.layer_buffers();
            
            match snem_core.debug_run_frame(
                frame_buffer, 
                audio_buffer,
                self.breakpoints(),
                self.watchpoints(),
                &mut layer_buffers,
            ) {
                DebugAction::BreakpointHit => {
                    self.breakpoint_hit(&snem_core, app_state);
                    clear_watchpoints = true;
                },
                DebugAction::WatchpointHit => {
                    self.watchpoint_hit(app_state);
                    clear_watchpoints = true;
                }
                _ => {}
            }
            
            self.ppu_tab.restore_buffers(layer_buffers);
        }
        
        self.wp_tab.update_watchpoint_graph();

        let mut egui_window = self.egui_window.take().unwrap();
        let mut debug_action = DebugAction::None;
    
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
                        self.compile_watchpoints(&snem_core);
                        debug_action = DebugAction::SingleStep;
                    }
    
                    if ui.button("Step Frame").clicked() {
                        self.compile_watchpoints(&snem_core);
                        debug_action = DebugAction::StepFrame;
                    }
                    
                    if ui.button("Reset").clicked() {
                        clear_watchpoints = true;
                        debug_action = DebugAction::Reset;
                    }
                    
                    if ui.button("Hard Reset").clicked() {
                        clear_watchpoints = true;
                        debug_action = DebugAction::HardReset;
                    }
                    
                    if app_state.is_paused && ui.button("Resume").clicked() {
                        self.compile_watchpoints(&snem_core);
                        debug_action = DebugAction::TogglePause;
                    }
                    
                    if !app_state.is_paused && ui.button("Pause").clicked() {
                        debug_action = DebugAction::TogglePause;
                        clear_watchpoints = true;
                    }
                    
                    ui.label(format!("Frame: {}", snem_core.frame));
                    
                    ui.label(format!("Cycles: {}", snem_core.total_cycles));
                });
                
                ui.add_space(3.0);
            });
            
            egui::CentralPanel::default().show(ctx, |ui| {
                match self.selected_tab {
                    tabs::DebugTab::Cpu => self.cpu_tab.render(ui, snem_core, &mut self.jump_to_bps_on_hit),
                    tabs::DebugTab::Memory => self.mem_tab.render(ui, snem_core),
                    tabs::DebugTab::Ppu => self.ppu_tab.render(ui, snem_core),
                    tabs::DebugTab::Watchpoints => self.wp_tab.render(ui, snem_core, app_state, &mut self.jump_to_wps_on_hit),
                };
            });
        }));

        let full_output = full_output.unwrap();
        
        egui_window.clear();
        egui_window.render(full_output);

        self.egui_window = Some(egui_window);
        
        match debug_action {
            DebugAction::SingleStep if app_state.is_paused => {
                let mut layer_buffers = self.ppu_tab.layer_buffers();
                
                match snem_core.debug_step_instruction(
                    frame_buffer, 
                    audio_buffer,
                    self.breakpoints(),
                    self.watchpoints(),
                    &mut layer_buffers,
                ) {
                    DebugAction::BreakpointHit => {
                        self.breakpoint_hit(&snem_core, app_state);
                    },
                    DebugAction::WatchpointHit => {
                        self.watchpoint_hit(app_state);
                    }
                    _ => {}
                }
                
                self.ppu_tab.restore_buffers(layer_buffers);
                clear_watchpoints = true;
            }
            DebugAction::StepFrame if app_state.is_paused => {
                let mut layer_buffers = self.ppu_tab.layer_buffers();
                
                match snem_core.debug_run_frame(
                    frame_buffer, 
                    audio_buffer,
                    self.breakpoints(),
                    self.watchpoints(),
                    &mut layer_buffers,
                ) {
                    DebugAction::BreakpointHit => {
                        app_state.is_paused = true;
                        self.breakpoint_hit(&snem_core, app_state);
                    },
                    DebugAction::WatchpointHit => {
                        app_state.is_paused = true;
                        self.watchpoint_hit(app_state);
                    }
                    _ => {}
                }
                
                self.ppu_tab.restore_buffers(layer_buffers);
                clear_watchpoints = true;
            }
            DebugAction::TogglePause => {
                app_action = app::AppAction::TogglePause;
            }
            DebugAction::Reset => {
                app_action = app::AppAction::ResetCore;
            }
            DebugAction::HardReset => {
                app_action = app::AppAction::PowerOnCore;
            }
            _ => {}
        }
        
        if clear_watchpoints {
            self.wp_tab.clear_compiled_watchpoints();
        }
        
        app_action
    }

    pub fn id(&self) -> u32 {
        self.egui_window.as_ref().unwrap().window().id()
    }

    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers) {
        self.egui_window.as_mut().unwrap().handle_sdl_mouse_event(event, modifiers);
        self.egui_window.as_mut().unwrap().handle_sdl_keyboard_event(event);
    }
    
    pub fn breakpoint_hit(&mut self, snem_core: &core::snemcore::Snemulator, app_state: &mut app::AppState) {
        app_state.is_paused = true;
        self.cpu_tab.breakpoint_hit((snem_core.cpu.pb as u32) << 16 | snem_core.cpu.pc as u32);
        
        if self.jump_to_bps_on_hit {
            self.selected_tab = tabs::DebugTab::Cpu;
        }
    }
    
    pub fn watchpoint_hit(&mut self, app_state: &mut app::AppState) {
        if self.wp_tab.watchpoints_enabled() {
            app_state.is_paused = true;
            
            if self.jump_to_wps_on_hit {
                self.selected_tab = tabs::DebugTab::Watchpoints;
            }
        }
    }
    
    pub fn breakpoints(&self) -> &std::collections::HashSet<BreakpointInfo> {
        &self.cpu_tab.breakpoints()
    }
    
    pub fn watchpoints(&self) -> &CompiledGraph {
        self.wp_tab.watchpoints()
    }
    
    fn compile_watchpoints(&mut self, snem_core: &core::snemcore::Snemulator) {
        self.wp_tab.compile_watchpoints(snem_core);
    }
}
