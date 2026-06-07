use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use anyhow::Result;

use common::UiWindow;

const SETTINGS_WINDOW_WIDTH: u32 = 600;
const SETTINGS_WINDOW_HEIGHT: u32 = 400;
const MAX_RECENT_ROMS: usize = 5;

#[derive(Default, Serialize, Deserialize)]
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

    #[serde(default)]
    pub recent_roms: Vec<PathBuf>,
}

impl Settings {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("snemulator").join("settings.toml"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else { return Self::default() };
        let Ok(text) = std::fs::read_to_string(&path) else { return Self::default() };
        
        log::trace!("Loaded settings from {}: {}", path.display(), text);
        
        let mut settings: Self = toml::from_str(&text).unwrap_or_default();
        settings.recent_roms.retain(|p| p.exists());
        settings
    }

    pub fn save(&self) {
        let Some(path) = Self::config_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, text);
        }

        log::trace!("Saved settings to {}", path.display());
    }

    pub fn push_recent_rom(&mut self, path: PathBuf) {
        self.recent_roms.retain(|p| p.file_name() != path.file_name());
        self.recent_roms.insert(0, path);
        self.recent_roms.truncate(MAX_RECENT_ROMS);
    }
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