use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use anyhow::Result;

use crate::{app::theme::{AppTheme, ThemePreset}, ui_window::UiWindow};

pub const SETTINGS_WINDOW_WIDTH: u32 = 600;
pub const SETTINGS_WINDOW_HEIGHT: u32 = 400;
const MAX_RECENT_ROMS: usize = 5;

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum AspectMode {
    #[default]
    Stretch,
    FourByThree,
    PixelPerfect,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum ScalingFilter {
    #[default]
    Nearest,
    Bilinear,
}

// Placeholder — you'll replace with your own struct per our earlier note.
#[derive(Serialize, Deserialize, Clone)]
pub struct Hotkeys {
    pub save_state: egui::Key,
    pub load_state: egui::Key,
    pub toggle_fast_forward: egui::Key,
    pub toggle_rewind: egui::Key,
    pub reset: egui::Key,
    pub screenshot: egui::Key,
    pub toggle_fullscreen: egui::Key,
    pub toggle_mute: egui::Key,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            save_state: egui::Key::F5,
            load_state: egui::Key::F7,
            toggle_fast_forward: egui::Key::F9,
            toggle_rewind: egui::Key::F8,
            reset: egui::Key::F1,
            screenshot: egui::Key::F12,
            toggle_fullscreen: egui::Key::F11,
            toggle_mute: egui::Key::M,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    // Video settings
    pub ui_scale: f32,
    pub vsync_en: bool,
    pub integer_scaling: bool,
    pub show_fps: bool,
    pub always_show_menu: bool,
    pub aspect_mode: AspectMode,
    pub scaling_filter: ScalingFilter,
    pub theme_preset: ThemePreset,

    // Audio settings
    pub audio_enabled: bool,
    pub master_volume: f32,

    // Input settings
    #[serde(default)]
    pub hotkeys: Hotkeys,

    // Emulation settings
    pub fast_forward_speed: u32,
    // pub rewind_enabled: bool,
    pub pause_on_minimize: bool,

    #[serde(default)]
    pub recent_roms: Vec<PathBuf>,

    #[serde(default)]
    pub roms_library_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            vsync_en: true,
            integer_scaling: false,
            show_fps: false,
            always_show_menu: false,
            aspect_mode: AspectMode::Stretch,
            scaling_filter: ScalingFilter::Nearest,
            theme_preset: ThemePreset::Dark,

            audio_enabled: true,
            master_volume: 1.0,

            hotkeys: Hotkeys::default(),

            fast_forward_speed: 4,
            // rewind_enabled: false,
            pause_on_minimize: true,

            recent_roms: Vec::new(),
            roms_library_dir: None,
        }
    }
}

impl Settings {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("snemulator").join("settings.toml"))
    }

    pub fn data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("snemulator"))
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

    pub fn push_recent_rom(&mut self, path: &PathBuf) {
        self.recent_roms.retain(|p| p.file_name() != path.file_name());
        self.recent_roms.insert(0, path.clone());
        self.recent_roms.truncate(MAX_RECENT_ROMS);
    }

    pub fn remove_recent_rom(&mut self, path: &PathBuf) {
        self.recent_roms.retain(|p| p.file_name() != path.file_name());
    }
}

#[derive(PartialEq, Clone, Copy)]
enum SettingsTab {
    General,
    Video,
    Audio,
    Controls,
    Emulation,
}

pub struct SettingsWindow {
    egui_window: UiWindow,
    current_tab: SettingsTab,
}

impl SettingsWindow {
    pub fn new(egui_window: UiWindow) -> Result<Self> {
        Ok(Self {
            egui_window,
            current_tab: SettingsTab::General,
        })
    }

    pub fn set_theme(&mut self, app_theme: &AppTheme) {
        app_theme.apply(&self.egui_window.egui_ctx);
    }
    
    pub fn update_and_render(&mut self, settings: &mut Settings) -> bool {
        let current_tab = &mut self.current_tab;
        let mut close_window = false;

        let full_output = self.egui_window.update_ui(|ctx| {
            egui::SidePanel::left("settings_tab_strip")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.selectable_value(current_tab, SettingsTab::General, "⚙ General");
                    ui.selectable_value(current_tab, SettingsTab::Video, "🖵 Video");
                    ui.selectable_value(current_tab, SettingsTab::Audio, "🔊 Audio");
                    ui.selectable_value(current_tab, SettingsTab::Controls, "🎮 Controls");
                    ui.selectable_value(current_tab, SettingsTab::Emulation, "⏱ Emulation");
                });

            egui::CentralPanel::default().show(ctx, |ui| match current_tab {
                SettingsTab::General => Self::render_general_tab(ui, settings),
                SettingsTab::Video => Self::render_video_tab(ui, settings),
                SettingsTab::Audio => Self::render_audio_tab(ui, settings),
                SettingsTab::Controls => {
                    ui.label("Controller mapping coming soon.");
                }
                SettingsTab::Emulation => Self::render_emulation_tab(ui, settings),
            });

            egui::Area::new("settings_close_button".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-10.0, -10.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Ok").clicked() {
                            close_window = true;
                        }
                    });
                });
        });
        
        self.egui_window.clear();
        self.egui_window.render(full_output);

        close_window
    }

    fn render_general_tab(ui: &mut egui::Ui, settings: &mut Settings) {
        ui.checkbox(&mut settings.always_show_menu, "Always show menu bar");
        ui.checkbox(&mut settings.pause_on_minimize, "Pause when window minimized");

        egui::ComboBox::new("theme_select", "Theme")
            .selected_text(settings.theme_preset.name())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.theme_preset, ThemePreset::Dark, "Dark");
                ui.selectable_value(&mut settings.theme_preset, ThemePreset::Light, "Light");
                ui.selectable_value(&mut settings.theme_preset, ThemePreset::Retro, "Retro");
            });

        ui.separator();
        ui.label("ROM Library");
        ui.horizontal(|ui| {
            let path_str = settings
                .roms_library_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Not set".to_string());
            ui.label(path_str);
            if ui.button("Browse...").clicked() {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    settings.roms_library_dir = Some(dir);
                }
            }
        });
    }

    fn render_video_tab(ui: &mut egui::Ui, settings: &mut Settings) {
        ui.checkbox(&mut settings.vsync_en, "Enable VSync");
        ui.checkbox(&mut settings.show_fps, "Show FPS counter");
        ui.checkbox(&mut settings.integer_scaling, "Integer scaling");

        ui.add(egui::Slider::new(&mut settings.ui_scale, 0.5..=2.0).text("UI scale"));

        ui.separator();
        ui.label("Aspect ratio");
        ui.radio_value(&mut settings.aspect_mode, AspectMode::Stretch, "Stretch");
        ui.radio_value(&mut settings.aspect_mode, AspectMode::FourByThree, "4:3");
        ui.radio_value(&mut settings.aspect_mode, AspectMode::PixelPerfect, "Pixel-perfect");

        ui.separator();
        ui.label("Scaling filter");
        ui.radio_value(&mut settings.scaling_filter, ScalingFilter::Nearest, "Nearest");
        ui.radio_value(&mut settings.scaling_filter, ScalingFilter::Bilinear, "Bilinear");
    }

    fn render_audio_tab(ui: &mut egui::Ui, settings: &mut Settings) {
        ui.checkbox(&mut settings.audio_enabled, "Enable audio");
        ui.add_enabled(
            settings.audio_enabled,
            egui::Slider::new(&mut settings.master_volume, 0.0..=1.0).text("Master volume"),
        );
    }

    fn render_emulation_tab(ui: &mut egui::Ui, settings: &mut Settings) {
        ui.add(
            egui::Slider::new(&mut settings.fast_forward_speed, 2..=8)
                .text("Fast forward speed (x)"),
        );
        // ui.checkbox(&mut settings.rewind_enabled, "Enable rewind");
        // ui.add(
        //     egui::Slider::new(&mut settings.save_state_slots, 1..=10).text("Save state slots"),
        // );
    }
    
    pub fn id(&self) -> u32 {
        self.egui_window.window().id()
    }
    
    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers) {
        self.egui_window.handle_sdl_mouse_event(event, modifiers);
    }
}