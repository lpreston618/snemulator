use anyhow::Result;

use crate::app;
use crate::core;
use crate::app::debug::tabs;
use crate::app::ui_window::UiWindow;
use crate::app::debug::breakpoints::BreakpointInfo;
use crate::app::debug::watchpoints::types::CompiledGraph;

const DEBUG_WINDOW_WIDTH: u32 = 800;
const DEBUG_WINDOW_HEIGHT: u32 = 600;

pub struct DebugWindow {
    egui_window: Option<Box<UiWindow>>,
    cpu_tab: Box<tabs::CpuTab>,
    mem_tab: Box<tabs::MemoryTab>,
    chr_tab: Box<tabs::ChrTab>,
    wp_tab: Box<tabs::WatchpointsTab>,
    selected_tab: tabs::DebugTab,
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
            egui_window: Some(Box::new(egui_window)),
            cpu_tab: Box::new(tabs::CpuTab::new(rom_mapping_mode)),
            mem_tab: Box::new(tabs::MemoryTab::new()),
            chr_tab: Box::new(tabs::ChrTab::new()),
            wp_tab: Box::new(tabs::WatchpointsTab::new()),
            selected_tab: tabs::DebugTab::Cpu,
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
                        tabs::DebugTab::Cpu,
                        tabs::DebugTab::Memory,
                        tabs::DebugTab::ChrRam,
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
                        debug_action = app::DebugAction::SingleStep;
                    }
    
                    if ui.button("Step Frame").clicked() {
                        self.compile_watchpoints(&snem_core);
                        debug_action = app::DebugAction::StepFrame;
                    }
                    
                    if app_state.is_paused && ui.button("Unpause").clicked() {
                        self.compile_watchpoints(&snem_core);
                        debug_action = app::DebugAction::TogglePause;
                    }
                    
                    if !app_state.is_paused && ui.button("Pause").clicked() {
                        debug_action = app::DebugAction::TogglePause;
                    }
                    
                    ui.label(format!("Frame: {}", snem_core.frame));
                    
                    ui.label(format!("Cycles: {}", snem_core.total_cycles));
                });
                
                ui.add_space(3.0);
            });
            
            egui::CentralPanel::default().show(ctx, |ui| {
                match self.selected_tab {
                    tabs::DebugTab::Cpu => self.cpu_tab.render(ui, snem_core),
                    tabs::DebugTab::Memory => self.mem_tab.render(ui, snem_core),
                    tabs::DebugTab::ChrRam => self.chr_tab.render(ui, snem_core),
                    tabs::DebugTab::Watchpoints => self.wp_tab.render(ui, snem_core, app_state),
                    _ => {},
                };
            });
        });

        egui_window.clear();
        egui_window.render(full_output);

        self.egui_window = Some(egui_window);
        debug_action
    }

    pub fn id(&self) -> u32 {
        self.egui_window.as_ref().unwrap().window().id()
    }

    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers) {
        self.egui_window.as_mut().unwrap().handle_sdl_mouse_event(event, modifiers);
        self.egui_window.as_mut().unwrap().handle_sdl_keyboard_event(event);
    }
    
    pub fn breakpoint_hit(&mut self, snem_core: &core::snemcore::Snemulator) {
        self.cpu_tab.breakpoint_hit((snem_core.cpu.pb as u32) << 16 | snem_core.cpu.pc as u32);
        self.selected_tab = tabs::DebugTab::Cpu;
    }
    
    pub fn watchpoint_hit(&mut self) {
        self.selected_tab = tabs::DebugTab::Watchpoints;
    }
    
    pub fn breakpoints(&self) -> &std::collections::HashSet<BreakpointInfo> {
        &self.cpu_tab.breakpoints()
    }
    
    pub fn watchpoints(&self) -> &CompiledGraph {
        self.wp_tab.watchpoints()
    }
    
    pub fn compile_watchpoints(&mut self, snem_core: &core::snemcore::Snemulator) {
        self.wp_tab.compile_watchpoints(snem_core);
    }
}
