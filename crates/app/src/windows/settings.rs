use anyhow::Result;

use crate::windows::ui_window::UiWindow;

const SETTINGS_WINDOW_WIDTH: u32 = 600;
const SETTINGS_WINDOW_HEIGHT: u32 = 400;

#[derive(Default)]
pub struct Settings {
    // Video settings
    // ui_scale: f32,
    pub vsync_en: bool,
    // integer_scaling: bool,
    pub show_fps: bool,
    pub always_show_menu: bool,
    
    // Audio settings
    pub audio_enabled: bool,
    pub master_volume: f32,
    
    // Input settings
    // keyboard_layout: KeyboardLayout,
    
    // Emulation settings
    pub fast_forward_speed: u32,
    // rewind_enabled: bool,
    pub save_state_slots: u32,
    pub pause_on_minimize: bool,
}

pub struct SettingsWindow {
    egui_window: UiWindow,
}

impl SettingsWindow {
    pub fn new(video_subsystem: &sdl3::VideoSubsystem) -> Result<Self> {
        Ok(Self {
            egui_window: UiWindow::new(
                video_subsystem,
                "Settings",
                SETTINGS_WINDOW_WIDTH,
                SETTINGS_WINDOW_HEIGHT,
            )?
        })
    }
    
    pub fn update_and_render(&mut self, settings: &mut Settings) {
        let full_output = self.egui_window.update_ui(|ctx| {
            
        });
        
        self.egui_window.clear();
        self.egui_window.render(full_output);
    }
    
    pub fn id(&self) -> u32 {
        self.egui_window.window().id()
    }
    
    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers) {
        self.egui_window.handle_sdl_mouse_event(event, modifiers);
    }
}