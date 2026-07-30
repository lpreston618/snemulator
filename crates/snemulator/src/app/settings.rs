use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use anyhow::Result;

use snemcore::controller::ControllerPlayer;

use crate::{
    app::{AppAction, controller::ControllerManager, library::LibraryViewMode, theme::{AppTheme, ThemePreset}}, ui_window::UiWindow,
};

use crate::app::controller::{PlayerInputDevice, default_scancode_for};

pub const SETTINGS_WINDOW_WIDTH: u32 = 780;
pub const SETTINGS_WINDOW_HEIGHT: u32 = 560;
const MAX_RECENT_ROMS: usize = 5;

#[derive(Clone)]
struct ChipAnchor {
    pub input: SnesInput,
    pub button_pos: (f32, f32),   // normalized (x, y) on the controller image
    pub midpoint_pos: Option<(f32, f32)>, // normalized (x, y) for a midpoint pos of the line between button and label
    pub label_pos: (f32, f32),    // normalized (x, y) for the chip; may fall outside 0..1
}

const CHIP_LAYOUT: &[ChipAnchor] = &[
    ChipAnchor { input: SnesInput::L,      button_pos: (0.300, 0.200), midpoint_pos: None, label_pos: (0.35, 0.00) },
    ChipAnchor { input: SnesInput::R,      button_pos: (0.700, 0.200), midpoint_pos: None, label_pos: (0.65, 0.00) },
    ChipAnchor { input: SnesInput::Up,     button_pos: (0.247, 0.420), midpoint_pos: None, label_pos: (0.05, 0.10) },
    ChipAnchor { input: SnesInput::Left,   button_pos: (0.200, 0.500), midpoint_pos: None, label_pos: (-0.05, 0.35) },
    ChipAnchor { input: SnesInput::Down,   button_pos: (0.247, 0.580), midpoint_pos: None, label_pos: (-0.05, 0.60) },
    ChipAnchor { input: SnesInput::Right,  button_pos: (0.295, 0.500), midpoint_pos: Some((0.295, 0.700)), label_pos: (0.08, 0.88) },
    ChipAnchor { input: SnesInput::Select, button_pos: (0.440, 0.550), midpoint_pos: None, label_pos: (0.38, 0.92) },
    ChipAnchor { input: SnesInput::Start,  button_pos: (0.530, 0.550), midpoint_pos: None, label_pos: (0.62, 0.92) },
    ChipAnchor { input: SnesInput::B,      button_pos: (0.755, 0.620), midpoint_pos: None, label_pos: (1.05, 0.60) },
    ChipAnchor { input: SnesInput::A,      button_pos: (0.820, 0.505), midpoint_pos: None, label_pos: (1.05, 0.35) },
    ChipAnchor { input: SnesInput::X,      button_pos: (0.741, 0.380), midpoint_pos: None, label_pos: (0.95, 0.10) },
    ChipAnchor { input: SnesInput::Y,      button_pos: (0.675, 0.495), midpoint_pos: Some((0.675, 0.680)), label_pos: (0.92, 0.88) },
];

pub const SNES_CONTROLLER: egui::ImageSource<'static> = egui::include_image!("../../assets/snes_controller.svg");

fn chip_label(input: SnesInput) -> &'static str {
    match input {
        SnesInput::Up => "Up", SnesInput::Down => "Down",
        SnesInput::Left => "Left", SnesInput::Right => "Right",
        _ => input.label(),
    }
}

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Hotkeys {
    pub save_state: u32,
    pub load_state: u32,
    pub toggle_pause: u32,
    pub toggle_fast_forward: u32,
    // pub toggle_rewind: u32,
    pub reset: u32,
    // pub screenshot: u32,
    pub toggle_fullscreen: u32,
    pub toggle_mute: u32,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            save_state: sdl3::keyboard::Keycode::F5 as u32,
            load_state: sdl3::keyboard::Keycode::F7 as u32,
            toggle_pause: sdl3::keyboard::Keycode::P as u32,
            toggle_fast_forward: sdl3::keyboard::Keycode::F9 as u32,
            // toggle_rewind: sdl3::keyboard::Scancode::F8.to_i32(),
            reset: sdl3::keyboard::Keycode::F1 as u32,
            // screenshot: sdl3::keyboard::Scancode::F12.to_i32(),
            toggle_fullscreen: sdl3::keyboard::Keycode::F11 as u32,
            toggle_mute: sdl3::keyboard::Keycode::M as u32,
        }
    }
}

impl Hotkeys {
    pub fn to_app_action(&self, scancode: sdl3::keyboard::Keycode) -> Option<AppAction> {
        let scancode = scancode as u32;

        if scancode == self.save_state {
            Some(AppAction::SaveState { slot: 0 })
        } else if scancode == self.load_state {
            Some(AppAction::LoadState { slot: 0 })
        } else if scancode == self.toggle_pause {
            Some(AppAction::TogglePaused)
        } else if scancode == self.toggle_fast_forward {
            Some(AppAction::ToggleFastForward)
        } else if scancode == self.reset {
            Some(AppAction::ResetCore)
        } else if scancode == self.toggle_fullscreen {
            Some(AppAction::ToggleFullscreen)
        } else if scancode == self.toggle_mute {
            Some(AppAction::ToggleMute)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HotkeyAction {
    SaveState,
    LoadState,
    TogglePause,
    ToggleFastForward,
    Reset,
    ToggleFullscreen,
    ToggleMute,
}

impl HotkeyAction {
    const ALL: [HotkeyAction; 7] = [
        HotkeyAction::SaveState,
        HotkeyAction::LoadState,
        HotkeyAction::TogglePause,
        HotkeyAction::ToggleFastForward,
        HotkeyAction::Reset,
        HotkeyAction::ToggleFullscreen,
        HotkeyAction::ToggleMute,
    ];

    fn label(&self) -> &'static str {
        match self {
            HotkeyAction::SaveState => "Save State",
            HotkeyAction::LoadState => "Load State",
            HotkeyAction::TogglePause => "Toggle Pause",
            HotkeyAction::ToggleFastForward => "Toggle Fast Forward",
            HotkeyAction::Reset => "Reset",
            HotkeyAction::ToggleFullscreen => "Toggle Fullscreen",
            HotkeyAction::ToggleMute => "Toggle Mute",
        }
    }

    fn get(&self, hotkeys: &Hotkeys) -> u32 {
        match self {
            HotkeyAction::SaveState => hotkeys.save_state,
            HotkeyAction::LoadState => hotkeys.load_state,
            HotkeyAction::TogglePause => hotkeys.toggle_pause,
            HotkeyAction::ToggleFastForward => hotkeys.toggle_fast_forward,
            HotkeyAction::Reset => hotkeys.reset,
            HotkeyAction::ToggleFullscreen => hotkeys.toggle_fullscreen,
            HotkeyAction::ToggleMute => hotkeys.toggle_mute,
        }
    }

    fn set(&self, hotkeys: &mut Hotkeys, code: u32) {
        match self {
            HotkeyAction::SaveState => hotkeys.save_state = code,
            HotkeyAction::LoadState => hotkeys.load_state = code,
            HotkeyAction::TogglePause => hotkeys.toggle_pause = code,
            HotkeyAction::ToggleFastForward => hotkeys.toggle_fast_forward = code,
            HotkeyAction::Reset => hotkeys.reset = code,
            HotkeyAction::ToggleFullscreen => hotkeys.toggle_fullscreen = code,
            HotkeyAction::ToggleMute => hotkeys.toggle_mute = code,
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
    pub library_view_mode: LibraryViewMode,

    // Audio settings
    pub audio_enabled: bool,
    pub master_volume: f32,

    // Input settings
    #[serde(default)]
    pub hotkeys: Hotkeys,

    // Emulation settings
    pub fast_forward_en: bool,
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
    pub controller_bindings: HashMap<String, ControllerBinding>,
    #[serde(default)]
    pub keyboard_bindings: HashMap<SnesInput, i32>,

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
            library_view_mode: LibraryViewMode::List,

            audio_enabled: true,
            master_volume: 1.0,

            hotkeys: Hotkeys::default(),

            fast_forward_en: false,
            fast_forward_speed: 4,
            // rewind_enabled: false,
            pause_on_minimize: true,

            recent_roms: Vec::new(),
            roms_library_dir: None,

            controller_bindings: HashMap::new(),
            keyboard_bindings: HashMap::new(),
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
}

#[derive(PartialEq, Clone, Copy)]
enum SettingsTab {
    General,
    Video,
    Audio,
    Controls,
    Hotkeys,
    Emulation,
}

pub struct SettingsWindow {
    egui_window: UiWindow,
    current_tab: SettingsTab,
    new_settings: Settings,
    active_player: ControllerPlayer,
    new_profile_name: String,
    rebinding_hotkey: Option<HotkeyAction>,
}

impl SettingsWindow {
    pub fn new(egui_window: UiWindow, settings: &Settings) -> Result<Self> {
        Ok(Self {
            egui_window,
            current_tab: SettingsTab::General,
            new_settings: settings.clone(),
            active_player: ControllerPlayer::Player1,
            new_profile_name: String::new(),
            rebinding_hotkey: None,
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
        app_theme: &AppTheme,
    ) -> Option<AppAction> {
        let current_tab = &mut self.current_tab;
        let active_player = &mut self.active_player;
        let mut app_action: Option<AppAction> = None;

        let full_output = self.egui_window.update_ui(|ctx| {
            egui::Panel::left("settings_tab_strip")
                .resizable(false)
                .min_size(128.0)
                .show(ctx, |ui| {
                    ui.selectable_value(current_tab, SettingsTab::General, "⚙ General");
                    ui.selectable_value(current_tab, SettingsTab::Video, "🖵 Video");
                    ui.selectable_value(current_tab, SettingsTab::Audio, "🔊 Audio");
                    ui.selectable_value(current_tab, SettingsTab::Controls, "🎮 Controls");
                    ui.selectable_value(current_tab, SettingsTab::Hotkeys, "🖮 Hotkeys");
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
                    active_player,
                    &mut self.new_profile_name,
                ),
                SettingsTab::Hotkeys => Self::render_hotkeys_tab(ui, &mut self.new_settings, &mut self.rebinding_hotkey),
                SettingsTab::Emulation => Self::render_emulation_tab(ui, &mut self.new_settings),
            });

            egui::Area::new("settings_close_button".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-10.0, -10.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            app_action = Some(AppAction::ApplySettings(Box::new(self.new_settings.clone())));
                        }

                        if ui.button("Cancel").clicked() {
                            app_action = Some(AppAction::CloseSettings(None));
                        }

                        if ui.button("Ok").clicked() {
                            app_action = Some(AppAction::CloseSettings(Some(Box::new(self.new_settings.clone()))));
                        }
                    });
                });
        });
        
        self.egui_window.clear(app_theme);
        self.egui_window.render(full_output);

        app_action
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
        active_player: &mut ControllerPlayer,
        new_profile_name: &mut String,
    ) {
        ui.horizontal(|ui| {
            ui.selectable_value(active_player, ControllerPlayer::Player1, "Player 1");
            ui.selectable_value(active_player, ControllerPlayer::Player2, "Player 2");
        });
        ui.separator();

        let controllers = controller_manager.connected_controllers();
        let current_device = controller_manager.device_for(*active_player);

        // The gamepad info backing `current_device`, if any. Profiles and the
        // per-button binding lookup below are keyed off a gamepad's
        // uuid/name, so both need this rather than `current_device` directly.
        let selected_gamepad = match current_device {
            Some(PlayerInputDevice::Gamepad(id)) => controllers.iter().find(|c| c.id == id),
            _ => None,
        };

        let selected_text = match current_device {
            Some(PlayerInputDevice::Keyboard) => "Keyboard".to_string(),
            Some(PlayerInputDevice::Gamepad(_)) => selected_gamepad
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown Gamepad".to_string()),
            None => "None".to_string(),
        };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Controller:");

                egui::ComboBox::new("controller_select", "")
                    .selected_text(&selected_text)
                    .show_ui(ui, |ui| {
                        let keyboard_selected = current_device == Some(PlayerInputDevice::Keyboard);
                        if ui.selectable_label(keyboard_selected, "Keyboard").clicked() {
                            controller_manager.assign_player(PlayerInputDevice::Keyboard, *active_player, settings);
                        }

                        for info in &controllers {
                            let is_selected = current_device == Some(PlayerInputDevice::Gamepad(info.id));

                            if ui.selectable_label(is_selected, &info.name).clicked() {
                                controller_manager.assign_player(PlayerInputDevice::Gamepad(info.id), *active_player, settings);
                            }
                        }
                    });

                ui.separator();

                if ui.add_enabled(current_device.is_some(), egui::Button::new("Reset to Defaults")).clicked() {
                    match current_device {
                        Some(PlayerInputDevice::Gamepad(id)) => {
                            if let Some(info) = controllers.iter().find(|c| c.id == id) {
                                settings.reset_binding(&info.uuid_key);
                            }
                        }
                        Some(PlayerInputDevice::Keyboard) => {
                            settings.keyboard_bindings.clear();
                            settings.save();
                        }
                        None => {}
                    }
                }
            });

            // Profiles store gamepad button/axis bindings (InputSource), which
            // don't map onto keyboard scancodes -- so profile save/load is
            // only available when a gamepad, not the keyboard, is selected.
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(new_profile_name)
                        .hint_text("Profile name")
                        .desired_width(120.0)
                    );

                let name_valid = !new_profile_name.trim().is_empty() && selected_gamepad.is_some();

                if ui.add_enabled(name_valid, egui::Button::new("Save as Profile")).clicked() {
                    let info = selected_gamepad.unwrap();
                    let binding = settings.binding_for(&info.uuid_key, &info.name);
                    settings.save_profile(new_profile_name.trim().to_string(), binding.bindings);
                    new_profile_name.clear();
                }

                if !settings.profiles.is_empty() && selected_gamepad.is_some() {
                    ui.label("Load Profile:");

                    egui::ComboBox::new("profile_load", "")
                        .selected_text("Choose…")
                        .show_ui(ui, |ui| {
                            for profile in settings.profiles.clone() {
                                if ui.selectable_label(false, &profile.name).clicked() {
                                    let info = selected_gamepad.unwrap();
                                    settings.apply_profile(&info.uuid_key, &info.name, &profile.name);
                                }
                            }
                        });
                }
            });
        });

        ui.separator();

        if current_device.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label(if controllers.is_empty() {
                    "Plug in a controller to configure it."
                } else {
                    "Select a controller above to configure this player."
                });
            });
            return;
        }

        // Which SnesInput (if any) is currently waiting for a press, resolved
        // per device type.
        let capturing = match current_device {
            Some(PlayerInputDevice::Gamepad(id)) => controller_manager.capturing_for(id),
            Some(PlayerInputDevice::Keyboard) => controller_manager.keyboard_capturing(),
            None => None,
        };

        let gamepad_binding = selected_gamepad.map(|info| settings.binding_for(&info.uuid_key, &info.name));

        let bound_label = |input: SnesInput| -> String {
            match current_device {
                Some(PlayerInputDevice::Gamepad(_)) => gamepad_binding
                    .as_ref()
                    .and_then(|b| b.bindings.get(&input))
                    .map(|s| s.label())
                    .unwrap_or_else(|| "Unbound".to_string()),
                Some(PlayerInputDevice::Keyboard) => settings
                    .keyboard_bindings
                    .get(&input)
                    .and_then(|&code| sdl3::keyboard::Scancode::from_i32(code as i32))
                    .unwrap_or_else(|| default_scancode_for(input))
                    .name()
                    .to_string(),
                None => "Unbound".to_string(),
            }
        };

        const SIDE_MARGIN: f32 = 90.0;
        const TOP_MARGIN: f32 = 70.0;

        ui.add_space(TOP_MARGIN);
        let image_rect = ui.horizontal(|ui| {
            ui.add_space(SIDE_MARGIN);
            let image_w = (ui.available_width() - SIDE_MARGIN).clamp(280.0, 460.0);
            let image_h = image_w * (240.0 / 400.0);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(image_w, image_h), egui::Sense::hover());
            egui::Image::new(SNES_CONTROLLER).paint_at(ui, rect);
            rect
        }).inner;

        for chip in CHIP_LAYOUT {
            let button_pt = image_rect.min + image_rect.size() * egui::vec2(chip.button_pos.0, chip.button_pos.1);
            let label_pt = image_rect.min + image_rect.size() * egui::vec2(chip.label_pos.0, chip.label_pos.1);

            if let Some(midpt) = chip.midpoint_pos {
                let mid_pt = image_rect.min + image_rect.size() * egui::vec2(midpt.0, midpt.1);

                ui.painter().line_segment([button_pt, mid_pt], (1.0, egui::Color32::GRAY));
                ui.painter().line_segment([mid_pt, label_pt], (1.0, egui::Color32::GRAY));
            } else {
                ui.painter().line_segment([button_pt, label_pt], (1.0, egui::Color32::GRAY));
            }

            let is_capturing = capturing == Some(chip.input);
            let bound_text = if is_capturing {
                "Waiting…".to_string()
            } else {
                bound_label(chip.input)
            };

            let bound_text = if bound_text.len() <= 11 { bound_text } else {
                bound_text.chars().take(10).collect::<String>() + "…"
            };

            let chip_rect = egui::Rect::from_center_size(label_pt, egui::vec2(100.0, 32.0));
            let resp = ui.put(
                chip_rect,
                egui::Button::new(format!("{}\n{}", chip_label(chip.input), bound_text)).small(),
            );
            if resp.clicked() {
                if is_capturing {
                    match current_device {
                        Some(PlayerInputDevice::Gamepad(_)) => controller_manager.cancel_remap(),
                        Some(PlayerInputDevice::Keyboard) => controller_manager.cancel_keyboard_remap(),
                        None => {}
                    }
                } else {
                    match current_device {
                        Some(PlayerInputDevice::Gamepad(id)) => controller_manager.begin_remap(id, chip.input),
                        Some(PlayerInputDevice::Keyboard) => controller_manager.begin_keyboard_remap(chip.input),
                        None => {}
                    }
                }
            }
        }
    }

    fn render_hotkeys_tab(
        ui: &mut egui::Ui,
        settings: &mut Settings,
        rebinding: &mut Option<HotkeyAction>,
    ) {
        ui.label("Click \"Rebind\" then press the desired key. Press Esc to cancel.");
        ui.separator();

        egui::Grid::new("hotkeys_grid")
            .num_columns(3)
            .spacing([16.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                for action in HotkeyAction::ALL {
                    let is_rebinding = *rebinding == Some(action);

                    ui.label(action.label());

                    if is_rebinding {
                        ui.colored_label(egui::Color32::YELLOW, "Press a key...");
                    } else {
                        let raw = action.get(&settings.hotkeys);
                        let keycode = sdl3::keyboard::Keycode::from_u32(raw);
                        let text = keycode.map_or(format!("Unknown {raw}"), |k| {
                            format!("{k}")
                        });

                        ui.label(text);
                    }

                    let button_text = if is_rebinding { "Cancel" } else { "Rebind" };
                    if ui.button(button_text).clicked() {
                        *rebinding = if is_rebinding { None } else { Some(action) };
                    }

                    ui.end_row();
                }
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
        ui.checkbox(&mut settings.fast_forward_en, "Enable fast forward");
        ui.add(
            egui::Slider::new(&mut settings.fast_forward_speed, 2..=8)
                .text("Fast forward speed (x)"),
        );
        // ui.checkbox(&mut settings.rewind_enabled, "Enable rewind");
    }
    
    pub fn id(&self) -> u32 {
        self.egui_window.window().id()
    }
    
    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers) {
        self.egui_window.handle_sdl_mouse_event(event, modifiers);
        self.egui_window.handle_sdl_keyboard_event(event);

        match event {
            sdl3::event::Event::KeyDown {
                keycode,
                ..
            } => {
                match self.current_tab {
                    SettingsTab::Hotkeys => {
                        if let Some(action) = self.rebinding_hotkey {
                            if Some(sdl3::keyboard::Keycode::Escape) == *keycode {
                                self.rebinding_hotkey = None;
                                return;
                            }
        
                            if let Some(keycode) = *keycode {
                                action.set(&mut self.new_settings.hotkeys, keycode as u32);
                                self.rebinding_hotkey = None;
                            }
                        }
                    }

                    _ => {}
                }
            }

            _ => {}
        }
    }
}