use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use anyhow::Result;

use snemcore::controller::ControllerPlayer;

use crate::{
    app::theme::{AppTheme, ThemePreset},
    app::controller::ControllerManager,
    ui_window::UiWindow,
};

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

/// The 12 logical SNES pad inputs a player can rebind.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SnesInput {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    L,
    R,
    Start,
    Select,
}

impl SnesInput {
    /// SNES-shaped layout order, grouped for the remap grid UI.
    pub const DPAD: [SnesInput; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];
    pub const FACE: [SnesInput; 4] = [Self::Y, Self::X, Self::B, Self::A];
    pub const SHOULDERS: [SnesInput; 2] = [Self::L, Self::R];
    pub const SYSTEM: [SnesInput; 2] = [Self::Select, Self::Start];

    pub fn label(self) -> &'static str {
        match self {
            SnesInput::Up => "D-Pad Up",
            SnesInput::Down => "D-Pad Down",
            SnesInput::Left => "D-Pad Left",
            SnesInput::Right => "D-Pad Right",
            SnesInput::A => "A",
            SnesInput::B => "B",
            SnesInput::X => "X",
            SnesInput::Y => "Y",
            SnesInput::L => "L",
            SnesInput::R => "R",
            SnesInput::Start => "Start",
            SnesInput::Select => "Select",
        }
    }
}

/// Mirrors the subset of `gilrs::Button` we support binding to. Kept as our
/// own type (rather than serializing gilrs's directly) so the settings file
/// format doesn't depend on gilrs's internal representation, and so this
/// module has no gilrs dependency at all -- the conversion lives in
/// controller.rs, next to the crate that owns it.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RemapButton {
    South,
    East,
    North,
    West,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LeftThumb,
    RightThumb,
}

/// Mirrors the subset of `gilrs::Axis` we support binding to (analog sticks
/// used as digital directions).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RemapAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftZ,
    RightZ,
    DPadX,
    DPadY,
}

/// A single physical input that can be bound to an `SnesInput`: either a
/// digital button, or an analog axis pushed past a deadzone in a given
/// direction (e.g. binding the D-pad to a stick).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputSource {
    Button(RemapButton),
    ButtonLow(RemapButton),  // For axis-style d-pads where value ≈ 0 means pressed
    AxisPositive(RemapAxis),
    AxisNegative(RemapAxis),
}

impl InputSource {
    /// Short display label for the remap grid, e.g. "South", "LStick Y −".
    pub fn label(self) -> String {
        match self {
            InputSource::Button(b) => format!("{b:?}"),
            InputSource::ButtonLow(b) => format!("{b:?} (Low)"),
            InputSource::AxisPositive(a) => format!("{} +", axis_label(a)),
            InputSource::AxisNegative(a) => format!("{} \u{2212}", axis_label(a)),
        }
    }
}

fn axis_label(axis: RemapAxis) -> &'static str {
    match axis {
        RemapAxis::LeftStickX => "LStick X",
        RemapAxis::LeftStickY => "LStick Y",
        RemapAxis::RightStickX => "RStick X",
        RemapAxis::RightStickY => "RStick Y",
        RemapAxis::LeftZ => "L Analog",
        RemapAxis::RightZ => "R Analog",
        RemapAxis::DPadX => "DPad X",
        RemapAxis::DPadY => "DPad Y",
    }
}

/// Full remap for one physical controller, keyed in `Settings` by the
/// controller's hex-encoded UUID so it survives reconnects/replugging into
/// a different port.
#[derive(Serialize, Deserialize, Clone)]
pub struct ControllerBinding {
    /// Display name only -- not used for matching, just so the Controls tab
    /// can show something readable even if the controller isn't currently
    /// connected.
    pub display_name: String,
    pub bindings: std::collections::HashMap<SnesInput, InputSource>,
}

impl ControllerBinding {
    /// Standard modern-gamepad layout: South/East/West/North face cluster
    /// as B/A/Y/X, shoulder bumpers as L/R.
    pub fn default_for(display_name: String) -> Self {
        let bindings = std::collections::HashMap::from([
            (SnesInput::B,      InputSource::Button(RemapButton::South)),
            (SnesInput::A,      InputSource::Button(RemapButton::East)),
            (SnesInput::Y,      InputSource::Button(RemapButton::West)),
            (SnesInput::X,      InputSource::Button(RemapButton::North)),
            (SnesInput::L,      InputSource::Button(RemapButton::LeftTrigger)),
            (SnesInput::R,      InputSource::Button(RemapButton::RightTrigger)),
            (SnesInput::Select, InputSource::Button(RemapButton::Select)),
            (SnesInput::Start,  InputSource::Button(RemapButton::Start)),
            (SnesInput::Up,     InputSource::Button(RemapButton::DPadUp)),
            (SnesInput::Down,   InputSource::Button(RemapButton::DPadDown)),
            (SnesInput::Left,   InputSource::Button(RemapButton::DPadLeft)),
            (SnesInput::Right,  InputSource::Button(RemapButton::DPadRight)),
        ]);

        Self { display_name, bindings }
    }
}

/// A user-named, reusable binding layout, decoupled from any specific
/// physical controller.
#[derive(Serialize, Deserialize, Clone)]
pub struct ControllerProfile {
    pub name: String,
    pub bindings: std::collections::HashMap<SnesInput, InputSource>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    // Video settings
    // pub ui_scale: f32,
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

    /// Per-controller button remaps, keyed by the controller's hex-encoded
    /// UUID. Absent entries fall back to `ControllerBinding::default_for`.
    #[serde(default)]
    pub controller_bindings: std::collections::HashMap<String, ControllerBinding>,

    #[serde(default)]
    pub profiles: Vec<ControllerProfile>,
    
    /// uuid_key of the controller that should auto-fill Player 1 / 2 on launch.
    #[serde(default)]
    pub preferred_p1: Option<String>,
    #[serde(default)]
    pub preferred_p2: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // ui_scale: 1.0,
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

            controller_bindings: std::collections::HashMap::new(),
            profiles: Vec::new(),
            preferred_p1: None,
            preferred_p2: None,
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

    pub fn load_or_default() -> Self {
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

    /// Current binding for a controller, falling back to the standard
    /// default layout if it hasn't been customized.
    pub fn binding_for(&self, uuid_key: &str, display_name: &str) -> ControllerBinding {
        self.controller_bindings
            .get(uuid_key)
            .cloned()
            .unwrap_or_else(|| ControllerBinding::default_for(display_name.to_string()))
    }

    /// Rebinds a single SNES input for a controller and saves immediately.
    pub fn set_binding(
        &mut self,
        uuid_key: &str,
        display_name: &str,
        input: SnesInput,
        source: InputSource,
    ) {
        let entry = self
            .controller_bindings
            .entry(uuid_key.to_string())
            .or_insert_with(|| ControllerBinding::default_for(display_name.to_string()));
        entry.display_name = display_name.to_string();
        entry.bindings.insert(input, source);
        self.save();
    }

    /// Resets a controller's bindings back to the default layout and saves.
    pub fn reset_binding(&mut self, uuid_key: &str) {
        self.controller_bindings.remove(uuid_key);
        self.save();
    }

    /// Saves `bindings` as a named profile, overwriting any existing
    /// profile with the same name.
    pub fn save_profile(&mut self, name: String, bindings: std::collections::HashMap<SnesInput, InputSource>) {
        match self.profiles.iter_mut().find(|p| p.name == name) {
            Some(existing) => existing.bindings = bindings,
            None => self.profiles.push(ControllerProfile { name, bindings }),
        }
        self.save();
    }

    /// Copies a saved profile's bindings onto a specific controller.
    pub fn apply_profile(&mut self, uuid_key: &str, display_name: &str, profile_name: &str) {
        let Some(profile) = self.profiles.iter().find(|p| p.name == profile_name) else { return };
        let bindings = profile.bindings.clone();
        self.controller_bindings
            .entry(uuid_key.to_string())
            .or_insert_with(|| ControllerBinding::default_for(display_name.to_string()))
            .bindings = bindings;
        self.save();
    }

    pub fn delete_profile(&mut self, name: &str) {
        self.profiles.retain(|p| p.name != name);
        self.save();
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
    new_settings: Settings,
    /// Which physical controller is currently shown in the Controls tab's
    /// remap grid, identified by its hex UUID key.
    selected_controller: Option<String>,
    new_profile_name: String,
}

impl SettingsWindow {
    pub fn new(egui_window: UiWindow, settings: &Settings) -> Result<Self> {
        Ok(Self {
            egui_window,
            current_tab: SettingsTab::General,
            new_settings: settings.clone(),
            selected_controller: None,
            new_profile_name: String::new(),
        })
    }

    pub fn set_theme(&mut self, app_theme: &AppTheme) {
        app_theme.apply(&self.egui_window.egui_ctx);
    }
    
    /// `controller_manager` and `live_settings` are the app's real, persisted
    /// state (not the buffered `new_settings` draft this window otherwise
    /// edits). Controller rebinds are applied and saved immediately when
    /// captured, so they bypass the draft/Apply flow used by the other tabs.
    pub fn update_and_render(
        &mut self,
        controller_manager: &mut ControllerManager,
        live_settings: &mut Settings,
    ) -> Option<Settings> {
        let current_tab = &mut self.current_tab;
        let selected_controller = &mut self.selected_controller;
        let mut apply_settings = false;

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
                SettingsTab::General => Self::render_general_tab(ui, &mut self.new_settings),
                SettingsTab::Video => Self::render_video_tab(ui, &mut self.new_settings),
                SettingsTab::Audio => Self::render_audio_tab(ui, &mut self.new_settings),
                SettingsTab::Controls => Self::render_controls_tab(
                    ui,
                    live_settings,
                    controller_manager,
                    selected_controller,
                    &mut self.new_profile_name,
                ),
                SettingsTab::Emulation => Self::render_emulation_tab(ui, &mut self.new_settings),
            });

            egui::Area::new("settings_close_button".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-10.0, -10.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            apply_settings = true;
                        }
                    });
                });
        });
        
        self.egui_window.clear();
        self.egui_window.render(full_output);

        if apply_settings {
            Some(self.new_settings.clone())
        } else {
            None
        }
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

    fn render_controls_tab(
        ui: &mut egui::Ui,
        settings: &mut Settings,
        controller_manager: &mut ControllerManager,
        selected: &mut Option<String>,
        new_profile_name: &mut String,
    ) {
        let controllers = controller_manager.connected_controllers();

        if controllers.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label("Plug in a controller to configure it.");
            });
            return;
        }

        // Keep the selection valid; default to the first connected controller.
        let selection_valid = selected
            .as_deref()
            .is_some_and(|sel| controllers.iter().any(|c| c.uuid_key == sel));
        if !selection_valid {
            *selected = Some(controllers[0].uuid_key.clone());
        }

        ui.horizontal(|ui| {
            // Left panel: which physical controller is connected.
            ui.vertical(|ui| {
                ui.set_width(180.0);
                for info in &controllers {
                    let is_selected = selected.as_deref() == Some(info.uuid_key.as_str());
                    let badge = match info.assigned_player {
                        Some(ControllerPlayer::Player1) => " (P1)",
                        Some(ControllerPlayer::Player2) => " (P2)",
                        None => "",
                    };
                    let label = format!("{}{}", info.name, badge);
                    if ui.selectable_label(is_selected, label).clicked() {
                        *selected = Some(info.uuid_key.clone());
                    }
                }
            });

            ui.separator();

            // Right panel: remap grid, shaped like the physical pad so it
            // reads at a glance instead of as a flat settings list.
            ui.vertical(|ui| {
                let Some(sel_key) = selected.clone() else { return };
                let Some(info) = controllers.iter().find(|c| c.uuid_key == sel_key) else {
                    return;
                };

                ui.horizontal(|ui| {
                    ui.heading(&info.name);
                    if ui.button("Set as Player 1").clicked() {
                        controller_manager.assign_player(info.id, ControllerPlayer::Player1, settings);
                    }
                    if ui.button("Set as Player 2").clicked() {
                        controller_manager.assign_player(info.id, ControllerPlayer::Player2, settings);
                    }
                    if ui.button("Reset to Defaults").clicked() {
                        settings.reset_binding(&sel_key);
                    }
                });

                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(new_profile_name).hint_text("Profile name"));
                    let name_valid = !new_profile_name.trim().is_empty();
                    if ui.add_enabled(name_valid, egui::Button::new("Save as Profile")).clicked() {
                        let binding = settings.binding_for(&sel_key, &info.name);
                        settings.save_profile(new_profile_name.trim().to_string(), binding.bindings);
                        new_profile_name.clear();
                    }

                    if !settings.profiles.is_empty() {
                        egui::ComboBox::new("profile_select", "Load Profile")
                            .selected_text("Choose…")
                            .show_ui(ui, |ui| {
                                for profile in settings.profiles.clone() {
                                    if ui.selectable_label(false, &profile.name).clicked() {
                                        settings.apply_profile(&sel_key, &info.name, &profile.name);
                                    }
                                }
                            });
                    }
                });

                ui.separator();

                let capturing = controller_manager.capturing_for(info.id);
                let binding = settings.binding_for(&sel_key, &info.name);

                let mut remap_row = |ui: &mut egui::Ui, input: SnesInput| {
                    ui.horizontal(|ui| {
                        ui.set_width(220.0);
                        ui.label(input.label());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if capturing == Some(input) {
                                ui.colored_label(egui::Color32::YELLOW, "Press a button…");
                                if ui.small_button("Cancel").clicked() {
                                    controller_manager.cancel_remap();
                                }
                            } else {
                                let current = binding
                                    .bindings
                                    .get(&input)
                                    .map(|s| s.label())
                                    .unwrap_or_else(|| "Unbound".to_string());
                                ui.label(current);
                                if ui.small_button("Rebind").clicked() {
                                    controller_manager.begin_remap(info.id, input);
                                }
                            }
                        });
                    });
                };

                ui.label(egui::RichText::new("D-Pad").strong());
                for input in SnesInput::DPAD {
                    remap_row(ui, input);
                }

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Face Buttons").strong());
                for input in SnesInput::FACE {
                    remap_row(ui, input);
                }

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Shoulders").strong());
                for input in SnesInput::SHOULDERS {
                    remap_row(ui, input);
                }

                ui.add_space(8.0);
                ui.label(egui::RichText::new("System").strong());
                for input in SnesInput::SYSTEM {
                    remap_row(ui, input);
                }
            });
        });
    }

    fn render_video_tab(ui: &mut egui::Ui, settings: &mut Settings) {
        ui.checkbox(&mut settings.vsync_en, "Enable VSync");
        ui.checkbox(&mut settings.show_fps, "Show FPS counter");
        ui.checkbox(&mut settings.integer_scaling, "Integer scaling");

        // ui.add(egui::Slider::new(&mut settings.ui_scale, 0.5..=2.0).text("UI scale"));

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
        self.egui_window.handle_sdl_keyboard_event(event);
    }
}