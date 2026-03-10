use anyhow::Result;

use crate::app::ui_window::UiWindow;

const SETTINGS_WINDOW_WIDTH: u32 = 600;
const SETTINGS_WINDOW_HEIGHT: u32 = 400;

#[derive(Default)]
pub struct Settings {
    // Video settings
    // ui_scale: f32,
    pub vsync_en: bool,
    // integer_scaling: bool,
    pub show_fps: bool,
    
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
    pub fn new(
        video_subsystem: &sdl3::VideoSubsystem, 
        gl: std::sync::Arc<glow::Context>, 
        gl_context: std::rc::Rc<sdl3::video::GLContext>) -> Result<Self> {
        Ok(Self {
            egui_window: UiWindow::new(
                video_subsystem,
                gl,
                gl_context,
                "Settings",
                SETTINGS_WINDOW_WIDTH,
                SETTINGS_WINDOW_HEIGHT,
            )?
        })
    }
    
    pub fn render(&mut self, settings: &mut Settings) {
        self.egui_window.render(None, |ui| {
            
        });
    }
    
    pub fn id(&self) -> u32 {
        self.egui_window.window.id()
    }
}