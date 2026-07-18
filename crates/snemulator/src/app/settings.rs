use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use anyhow::Result;

use snemcore::controller::ControllerPlayer;

use crate::{
    app::{controller::ControllerManager, library::LibraryViewMode, theme::{AppTheme, ThemePreset}}, ui_window::UiWindow,
};

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
            library_view_mode: LibraryViewMode::List,

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
    active_player: ControllerPlayer,
    new_profile_name: String,
}

impl SettingsWindow {
    pub fn new(egui_window: UiWindow, settings: &Settings) -> Result<Self> {
        Ok(Self {
            egui_window,
            current_tab: SettingsTab::General,
            new_settings: settings.clone(),
            active_player: ControllerPlayer::Player1,
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
        app_theme: &AppTheme,
    ) -> Option<Settings> {
        let current_tab = &mut self.current_tab;
        let active_player = &mut self.active_player;
        let mut apply_settings = false;

        let full_output = self.egui_window.update_ui(|ctx| {
            egui::Panel::left("settings_tab_strip")
                .resizable(false)
                .min_size(128.0)
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
                    active_player,
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
        
        self.egui_window.clear(app_theme);
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
        active_player: &mut ControllerPlayer,
        new_profile_name: &mut String,
    ) {
        ui.horizontal(|ui| {
            ui.selectable_value(active_player, ControllerPlayer::Player1, "Player 1");
            ui.selectable_value(active_player, ControllerPlayer::Player2, "Player 2");
        });
        ui.separator();

        let controllers = controller_manager.connected_controllers();
        let selected = controllers.iter().find(|c| c.assigned_player == Some(*active_player));

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Controller:");
                egui::ComboBox::new("controller_select", "")
                    .selected_text(selected.map(|c| c.name.as_str()).unwrap_or("None"))
                    .show_ui(ui, |ui| {
                        for info in &controllers {
                            let is_selected = selected.is_some_and(|s| s.uuid_key == info.uuid_key);
                            if ui.selectable_label(is_selected, &info.name).clicked() {
                                controller_manager.assign_player(info.id, *active_player, settings);
                            }
                        }
                    });
    
                ui.separator();
                
                if ui.add_enabled(selected.is_some(), egui::Button::new("Reset to Defaults")).clicked() {
                    settings.reset_binding(&selected.unwrap().uuid_key);
                }
            });

            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(new_profile_name).hint_text("Profile name").desired_width(120.0));
                let name_valid = !new_profile_name.trim().is_empty() && selected.is_some();
                if ui.add_enabled(name_valid, egui::Button::new("Save as Profile")).clicked() {
                    let info = selected.unwrap();
                    let binding = settings.binding_for(&info.uuid_key, &info.name);
                    settings.save_profile(new_profile_name.trim().to_string(), binding.bindings);
                    new_profile_name.clear();
                }
                if !settings.profiles.is_empty() && selected.is_some() {
                    ui.label("Load Profile:");

                    egui::ComboBox::new("profile_load", "")
                        .selected_text("Choose…")
                        .show_ui(ui, |ui| {
                            for profile in settings.profiles.clone() {
                                if ui.selectable_label(false, &profile.name).clicked() {
                                    let info = selected.unwrap();
                                    settings.apply_profile(&info.uuid_key, &info.name, &profile.name);
                                }
                            }
                        });
                }
            });
        });

        ui.separator();

        let Some(info) = selected else {
            ui.centered_and_justified(|ui| {
                ui.label(if controllers.is_empty() {
                    "Plug in a controller to configure it."
                } else {
                    "Select a controller above to configure this player."
                });
            });
            return;
        };

        let capturing = controller_manager.capturing_for(info.id);
        let binding = settings.binding_for(&info.uuid_key, &info.name);

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
                "Unbound".to_string()
            } else {
                binding.bindings.get(&chip.input).map(|s| s.label()).unwrap_or_else(|| "Unbound".to_string())
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
                    controller_manager.cancel_remap();
                } else {
                    controller_manager.begin_remap(info.id, chip.input);
                }
            }
        }
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