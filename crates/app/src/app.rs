use crate::SnemulatorArgs;

use crate::app::rom_paths::RomManifest;
use crate::ui_window::UiWindow;
use crate::app::resampler::AudioResampler;
use sdl3::VideoSubsystem;
#[cfg(not(feature="debug"))]
use snemcore::debug::NullHarness;
use snemcore::savestate::SaveState;
#[cfg(feature = "debug")]
use crate::debug::{harness::MainDebugHarness, window::DebugWindow};

use crate::game::MainWindow;
use theme::{AppTheme, ThemePreset};
use settings::{Settings, SettingsWindow};
use rom_paths::RomPaths;
use anyhow::{anyhow, Result};
use rfd::FileDialog;
use ringbuf::HeapRb;
use ringbuf::traits::{Observer, RingBuffer};
use sdl3::audio::{AudioFormat, AudioSpec, AudioStreamOwner};
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use snemcore::controller::{ControllerPlayer, JoypadButton};
use snemcore::sysinfo::{self, AUDIO_SAMPLE_HZ, FRAMES_PER_SECOND, SCREEN_HEIGHT, SCREEN_WIDTH};
use snemcore::Snemulator;
use std::path::PathBuf;
use std::time::{Duration, Instant};

mod resampler;
pub mod settings;
pub mod theme;
mod rom_paths;

pub const FRAME_BUF_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize;

pub const WINDOW_WIDTH: u32 = 640;
pub const WINDOW_HEIGHT: u32 = 480;

const PREV_FPS_BUFFER_LEN: usize = FRAMES_PER_SECOND as usize * 1;
const FRAMES_BEFORE_HIDE_MENU: u64 = (3.0 * FRAMES_PER_SECOND) as u64;
const FRAMES_BEFORE_HIDE_MOUSE: u64 = (3.0 * FRAMES_PER_SECOND) as u64;
const FRAMES_BETWEEN_DISPLAY_FPS_UPDATE: u64 = (1.0 * FRAMES_PER_SECOND) as u64;
const AUDIO_SAMPLES_PER_FRAME: usize = 2 * AUDIO_SAMPLE_HZ / FRAMES_PER_SECOND as usize;

const SECONDS_BETWEEN_AUTO_SRAM_SAVES: f32 = 60.0;
const FRAMES_BETWEEN_AUTO_SRAM_SAVES: u64 = (SECONDS_BETWEEN_AUTO_SRAM_SAVES * FRAMES_PER_SECOND) as u64;

pub const MAX_SAVE_STATE_SLOTS: usize = 10;

#[cfg(feature = "debug")]
fn create_harness() -> MainDebugHarness {
    MainDebugHarness::new()
}

#[cfg(not(feature = "debug"))]
fn create_harness() -> NullHarness {
    NullHarness {}
}

pub enum AppAction {
    Continue,
    TogglePause,
    ToggleFullscreen,
    LoadRom,
    LoadRomFromPath(PathBuf),
    UnloadRom,
    ResetCore,
    PowerOnCore,
    SaveState { slot: usize },
    LoadState { slot: usize },
    OpenSettings,
    Exit,

    #[cfg(feature = "debug")]
    CloseDebug,
    #[cfg(feature = "debug")]
    OpenDebug,
}

pub struct RomMetadata {
    pub crc32_hash: u32,
    pub paths: RomPaths,
    pub used_save_state_slots: [bool; MAX_SAVE_STATE_SLOTS],
}

pub struct AppState {
    pub frame_count: u64,
    pub last_mouse_input_frame: u64,
    pub last_display_fps_update_frame: u64,
    pub last_sram_autosave_frame: u64,
    pub show_menu: bool,
    pub show_mouse: bool,
    pub is_paused: bool,
    pub is_fullscreen: bool,
    pub is_minimized: bool,
    pub fps: f32,
    pub display_fps: usize,
    pub loaded_rom_data: Option<RomMetadata>,

    #[cfg(feature = "debug")]
    pub debug_active: bool,
}

pub struct SnemulatorApp {
    sdl_context: sdl3::Sdl,
    video_subsystem: sdl3::VideoSubsystem,
    audio_stream: AudioStreamOwner,
    event_pump: Option<sdl3::EventPump>,

    main_window: MainWindow,
    settings_window: Option<SettingsWindow>,
    state: AppState,
    settings: Settings,
    theme: AppTheme,
    fonts: egui::FontDefinitions,
    prev_frame_micros: HeapRb<usize>,
    total_frame_micros: usize,
    frame_buffer: Box<[u8; FRAME_BUF_SIZE]>,
    audio_buffer: Vec<i16>,
    audio_resampler: Option<AudioResampler>,

    snem_core: Snemulator,
    random_seed: u64,

    #[cfg(feature = "debug")]
    debug_harness: MainDebugHarness,
    #[cfg(not(feature = "debug"))]
    debug_harness: NullHarness,

    #[cfg(feature = "debug")]
    debug_window: Option<DebugWindow>,
}

impl SnemulatorApp {
    pub fn new(args: SnemulatorArgs) -> Result<Self> {
        let state = AppState {
            frame_count: 0,
            last_mouse_input_frame: 0,
            last_display_fps_update_frame: 0,
            last_sram_autosave_frame: 0,
            show_menu: true,
            show_mouse: true,
            is_paused: false,
            is_fullscreen: false,
            is_minimized: false,
            fps: 0.0,
            display_fps: 0,
            loaded_rom_data: None,

            #[cfg(feature = "debug")]
            debug_active: false,
        };

        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;
        let audio_subsystem = sdl_context.audio()?;
        let event_pump = Some(sdl_context.event_pump()?);
        let settings = Settings::load();
        let theme = AppTheme::load_or_preset(ThemePreset::default());
        let fonts = Self::load_fonts();
        let frame_buffer = Box::new([0u8; FRAME_BUF_SIZE]);
        let audio_buffer = Vec::new();

        let main_egui_window = Self::create_window(
            "Snemulator",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            &video_subsystem,
            &fonts,
            &theme
        )?;

        let main_window = MainWindow::new(main_egui_window, &video_subsystem, &settings)?;

        
        if args.resampling {
            
        }
        
        let (audio_stream, audio_resampler) = if args.resampling {
            let audio_spec = AudioSpec {
                freq: None,
                channels: Some(2),
                format: Some(AudioFormat::s16_sys()),
            };
            let audio_device = audio_subsystem.open_playback_device(&audio_spec)?;
            let obtained_spec = audio_device.format()?;
            let output_rate = obtained_spec.0.freq.unwrap() as usize;

            let stream_spec = AudioSpec {
                freq: obtained_spec.0.freq,
                channels: Some(2),
                format: Some(AudioFormat::s16_sys()),
            };

            let audio_stream = audio_device.open_device_stream(Some(&stream_spec))?;
    
            let audio_resampler = AudioResampler::new(32000, output_rate);

            (audio_stream, Some(audio_resampler))
        } else {
            let audio_spec = AudioSpec {
                freq: Some(sysinfo::AUDIO_SAMPLE_HZ as i32),
                channels: Some(2),
                format: Some(AudioFormat::s16_sys()),
            };

            let audio_device = audio_subsystem.open_playback_device(&audio_spec)?;
            let audio_stream = audio_device.open_device_stream(Some(&audio_spec))?;

            (audio_stream, None)
        };

        let snem_core = Snemulator::new();
        let debug_harness = create_harness();

        let mut app = Self {
            sdl_context,
            video_subsystem,
            audio_stream,
            event_pump,

            main_window,
            settings_window: None,
            state,
            settings,
            theme,
            fonts,
            prev_frame_micros: HeapRb::new(PREV_FPS_BUFFER_LEN),
            total_frame_micros: 0,
            random_seed: 0, 
            
            snem_core,
            frame_buffer,
            audio_buffer,
            audio_resampler,

            debug_harness,

            #[cfg(feature = "debug")]
            debug_window: None,
        };

        app.handle_args(args)?;

        app.random_seed = app.snem_core.get_random_seed();

        log::trace!("Random Seed: {}", app.random_seed);

        Ok(app)
    }

    fn handle_args(&mut self, args: SnemulatorArgs) -> Result<()> {
        if let Some(seed) = args.seed {
            self.snem_core.set_random_seed(seed);
        }

        if let Some(rom_path) = args.rom {
            log::trace!("Loading ROM from command line argument: '{}'", rom_path);
            self.try_load_rom_from_path(&rom_path.into())?;
        }

        if args.start_paused && !self.state.is_paused {
            self.toggle_pause();
        }

        if args.no_audio {
            self.settings.audio_enabled = false;
        } else {
            self.settings.audio_enabled = true;
        }

        #[cfg(feature = "debug")]
        if args.debug {
            log::trace!("Debug mode enabled from command line argument");
            self.show_debug();
        }

        if let Some(theme) = args.theme {
            let theme_preset = match theme.to_ascii_lowercase().as_str() {
                "dark" => Some(ThemePreset::Dark),
                "light" => Some(ThemePreset::Light),
                "retro" => Some(ThemePreset::Retro),
                _ => None,
            };

            if let Some(preset) = theme_preset {
                log::trace!("Setting theme to preset '{}' from command line arg", theme);

                self.theme = AppTheme::from_preset(preset);
                self.apply_new_theme();
            }
        }

        Ok(())
    }

    fn apply_new_theme(&mut self) {
        self.main_window.set_theme(&self.theme);

        if let Some(settings_window) = &mut self.settings_window {
            settings_window.set_theme(&self.theme);
        }

        #[cfg(feature = "debug")]
        if let Some(debug_window) = &mut self.debug_window {
            debug_window.set_theme(&self.theme);
        }
    }

    fn load_fonts() -> egui::FontDefinitions {
        let mut fonts = egui::FontDefinitions::default();
        
        let mono_data = include_bytes!("../assets/fonts/JetBrainsMonoNL-Bold.ttf");
        fonts.font_data.insert(
            "JetBrains Mono Bold".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(mono_data)),
        );
        
        fonts.families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "JetBrains Mono Bold".to_owned());
        
        fonts.families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "JetBrains Mono Bold".to_owned());
        
        fonts
    }

    pub fn run(&mut self) -> Result<()> {
        let frame_duration = Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND);
        let spin_threshold = Duration::from_millis(2);

        'running: loop {
            let frame_start = Instant::now();

            let app_action = self.handle_input();

            match app_action {
                AppAction::Continue => {}
                AppAction::Exit => break 'running,
                _ => {
                    self.do_action(app_action);
                }
            }

            self.update_emulator();

            self.render_audio();

            let app_action = self.main_window.update_and_render(
                &self.state,
                &mut self.settings,
                &self.frame_buffer[..],
            );

            match app_action {
                AppAction::Continue => {}
                AppAction::Exit => break 'running,
                _ => {
                    self.do_action(app_action);
                }
            }

            if let Some(settings_window) = &mut self.settings_window {
                settings_window.update_and_render(&mut self.settings);
            }

            #[cfg(feature = "debug")]
            self.update_debug_window();

            self.state.show_menu = self.settings.always_show_menu
                || (self.state.frame_count - self.state.last_mouse_input_frame
                    < FRAMES_BEFORE_HIDE_MENU);
            self.state.show_mouse = match self.sdl_context.mouse().focused_window_id() {
                Some(id) => {
                    id != self.main_window.id()
                        || (self.state.frame_count - self.state.last_mouse_input_frame
                            < FRAMES_BEFORE_HIDE_MOUSE)
                }
                _ => true,
            };

            self.sdl_context.mouse().show_cursor(self.state.show_mouse);

            if (self.state.frame_count - self.state.last_display_fps_update_frame) > FRAMES_BETWEEN_DISPLAY_FPS_UPDATE {
                self.state.last_display_fps_update_frame = self.state.frame_count;
                self.state.display_fps = self.state.fps as usize;
            }

            if (self.state.frame_count - self.state.last_sram_autosave_frame) > FRAMES_BETWEEN_AUTO_SRAM_SAVES {
                self.state.last_sram_autosave_frame = self.state.frame_count;
                self.save_cartridge_save_ram(true);
            }

            // Frame timing
            self.state.frame_count += 1;

            let deadline = frame_start + frame_duration;

            // Sleep until we're close to the deadline, avoiding overshoot
            let now = Instant::now();
            if let Some(sleep_duration) = deadline.checked_duration_since(now) {
                if sleep_duration > spin_threshold {
                    std::thread::sleep(sleep_duration - spin_threshold);
                }
            }

            // Spin-wait the remaining time for precision
            while Instant::now() < deadline {}

            self.update_fps(frame_start.elapsed());
        }

        self.save_cartridge_save_ram(false);

        Ok(())
    }
    
    fn update_fps(&mut self, elapsed: Duration) {
        let prev = self.prev_frame_micros.push_overwrite(elapsed.as_micros() as usize);
        
        if let Some(prev_micros) = prev {
            self.total_frame_micros -= prev_micros;
        }
        
        self.total_frame_micros += elapsed.as_micros() as usize;
        
        if self.prev_frame_micros.occupied_len() > 0 {
            let avg_micros = self.total_frame_micros / self.prev_frame_micros.occupied_len() as usize;
            let avg_secs = avg_micros as f32 / 1000000.0;
            let avg_fps = 1.0 / avg_secs;
            self.state.fps = avg_fps;
        } else {
            self.state.fps = 0.0;
        }
    }
    
    fn update_emulator(&mut self) {
        if self.state.loaded_rom_data.is_some() && !self.state.is_paused && self.settings_window.is_none()
            && (!self.state.is_minimized || !self.settings.pause_on_minimize)
        {
            // let audio_buf = if self.settings.audio_enabled { Some(&mut self.audio_buffer) } else { None };

            self.snem_core.run_frame(&mut self.frame_buffer[..], &mut self.audio_buffer, &mut self.debug_harness);
        }
    }

    fn render_audio(&mut self) {
        if self.audio_buffer.is_empty() {
            return;
        }

        if let Some(resampler) = &mut self.audio_resampler {
            // Called once per frame, after SDSP finishes
            while self.audio_buffer.len() >= resampler.input_frames_needed() * 2 {
                let needed = resampler.input_frames_needed() * 2;
                let chunk: Vec<i16> = self.audio_buffer.drain(..needed).collect();
                let resampled = resampler.resample(&chunk);
                
                if let Err(e) = self.audio_stream.put_data_i16(&resampled) {
                    log::warn!("Audio stream write failed: {e}");
                }
            }
        } else {
            if let Err(e) = self.audio_stream.put_data_i16(&self.audio_buffer) {
                log::warn!("Audio stream write failed: {e}");
            }

            self.audio_buffer.clear();
        }
    }

    fn handle_input(&mut self) -> AppAction {
        let mut app_action = AppAction::Continue;

        let mut event_pump = self.event_pump.take().unwrap();
        let keyboard_state = event_pump.keyboard_state();

        let modifiers = egui::Modifiers {
            alt: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LAlt)
                || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RAlt),
            ctrl: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LCtrl)
                || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RCtrl),
            shift: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LShift)
                || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RShift),
            mac_cmd: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LGui)
                || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RGui),
            command: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LGui)
                || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RGui),
        };

        for event in event_pump.poll_iter() {
            // Route events to windows
            if let Some(event_win_id) = event.get_window_id() {
                if let Some(settings_window) = &mut self.settings_window {
                    if event_win_id == settings_window.id() {
                        self.handle_settings_window_event(&event, &modifiers);
                        continue;
                    }
                }

                #[cfg(feature = "debug")]
                if let Some(debug_window) = &mut self.debug_window {
                    if event_win_id == debug_window.id() {
                        self.handle_debug_window_event(&event, &modifiers);
                        continue;
                    }
                }
            }

            // Event is for main window
            self.main_window
                .handle_event(&event, &modifiers, &mut self.state);

            match event {
                Event::Quit { .. } => {
                    log::info!("Quit event received, exiting.");
                    
                    self.settings.save();
                    self.settings_window = None;

                    app_action = AppAction::Exit;
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    app_action = self.handle_keydown(keycode, keymod);
                }

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => self.handle_keyup(keycode),
                _ => {}
            }
        }

        self.event_pump = Some(event_pump);

        app_action
    }

    fn handle_settings_window_event(&mut self, event: &Event, modifiers: &egui::Modifiers) {
        match &event {
            Event::Window {
                win_event: sdl3::event::WindowEvent::CloseRequested,
                ..
            } => {
                self.settings_window = None;
            }
            _ => {
                self.settings_window
                    .as_mut()
                    .unwrap()
                    .handle_event(event, modifiers);
            }
        }
    }

    fn do_action(&mut self, app_action: AppAction) {
        match app_action {
            AppAction::LoadRom => self.load_rom(),
            AppAction::LoadRomFromPath(path) => {
                if let Err(_) = self.try_load_rom_from_path(&path) {
                    self.settings.remove_recent_rom(&path);
                    
                    let file_name = path
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid file name"))
                        .unwrap()
                        .to_string();

                    log::error!("Failed to load ROM '{}'", file_name);
                }
            }
            AppAction::UnloadRom if self.state.loaded_rom_data.is_some() => {
                self.save_cartridge_save_ram(false);
                self.snem_core.unload_rom();
                self.state.loaded_rom_data = None;
                self.clear_frame_buf();
                self.audio_stream.clear().ok();
                
                if !self.state.is_paused {
                    self.toggle_pause();
                }

                log::info!("Unloaded ROM");
            }
            AppAction::LoadState { slot } => {
                if let Err(e) = self.try_load_state(slot) {
                    log::error!("Failed to load state: {e}");
                }
            },
            AppAction::SaveState { slot } => {
                if let Err(e) = self.try_save_state(slot) {
                    log::error!("Failed to save state: {e}")
                }
            },
            AppAction::ResetCore => self.reset_emulation(false),
            AppAction::PowerOnCore => self.reset_emulation(true),
            AppAction::OpenSettings => self.show_settings(),
            AppAction::ToggleFullscreen => self.toggle_fullscreen(),
            AppAction::TogglePause => self.toggle_pause(),
            #[cfg(feature = "debug")]
            AppAction::OpenDebug => self.show_debug(),
            #[cfg(feature = "debug")]
            AppAction::CloseDebug => { self.debug_window = None; }
            
            _ => {}
        }
    }

    fn handle_keydown(&mut self, keycode: Keycode, keymod: Mod) -> AppAction {
        let mut app_action = AppAction::Continue;

        match keycode {
            Keycode::F11 => {
                app_action = AppAction::ToggleFullscreen;
            }
            Keycode::Escape => {
                if self.state.is_fullscreen {
                    app_action = AppAction::ToggleFullscreen;
                }
            }
            Keycode::Q => {
                if keymod.contains(Mod::LCTRLMOD) {
                    log::info!("Ctrl+Q pressed, exiting");

                    app_action = AppAction::Exit;
                }
            }

            Keycode::Up => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Up, true)
            }
            Keycode::Down => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Down, true)
            }
            Keycode::Left => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Left, true)
            }
            Keycode::Right => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Right, true)
            }
            Keycode::Z => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::A, true)
            }
            Keycode::X => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::B, true)
            }
            Keycode::Return => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Start, true)
            }
            Keycode::RShift => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Select, true)
            }

            _ => {}
        }

        app_action
    }

    fn handle_keyup(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Up => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Up, false)
            }
            Keycode::Down => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Down, false)
            }
            Keycode::Left => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Left, false)
            }
            Keycode::Right => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Right, false)
            }
            Keycode::Z => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::A, false)
            }
            Keycode::X => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::B, false)
            }
            Keycode::Return => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Start, false)
            }
            Keycode::RShift => {
                self.snem_core
                    .set_button(ControllerPlayer::Player1, JoypadButton::Select, false)
            }
            _ => {}
        }
    }

    fn clear_frame_buf(&mut self) {
        self.frame_buffer.chunks_mut(4).for_each(|pixel| {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            pixel[3] = 255;
        });
    }

    fn load_rom(&mut self) {
        if let Err(e) = self.try_load_rom() {
            log::error!("Failed to load rom: {}", e);
        }
    }

    fn try_load_rom(&mut self) -> Result<()> {
        let start_dir = self.settings.default_rom_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("/"));

        let romfile = FileDialog::new()
            .add_filter("ROM", &["sfc", "smc"])
            .set_directory(start_dir)
            .pick_file();

        if let Some(romfile) = romfile {
            let file_name = romfile
                .to_str()
                .ok_or_else(|| anyhow!("Invalid file name"))?
                .to_string();

            log::info!("Trying to load rom '{}'", file_name);

            self.try_load_rom_from_path(&romfile)?;
        }

        Ok(())
    }

    fn try_load_rom_from_path(&mut self, path: &PathBuf) -> Result<()> {
        let data = std::fs::read(path)?;
        let crc = crc32fast::hash(&data);

        let rom_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid ROM filename"))?;

        // Prefer existing folder by hash, fall back to name
        let rom_paths = RomPaths::find_by_hash(crc)
            .or_else(|| RomPaths::new(rom_name))
            .ok_or_else(|| anyhow!("Could not resolve data directory"))?;

        rom_paths.ensure_dirs()?;
        rom_paths.write_manifest(&RomManifest {
            rom_crc: crc,
            display_name: rom_name.to_string(),
        });

        self.snem_core.load_rom(data, crc)?;

        self.snem_core.power_on(&mut self.debug_harness);

        self.settings.push_recent_rom(path);
        self.settings.save();

        self.audio_buffer.extend([0; AUDIO_SAMPLES_PER_FRAME]);
        self.render_audio();
        self.audio_stream.resume()?;

        let used_save_state_slots: [bool; MAX_SAVE_STATE_SLOTS] = std::array::from_fn(|slot| {
            rom_paths.state_path(slot as u32).exists()
        });

        self.state.loaded_rom_data = Some(RomMetadata {
            crc32_hash: crc,
            paths: rom_paths,
            used_save_state_slots,
        });

        log::info!("Loaded rom '{}'", rom_name);

        self.load_save_ram();
        
        Ok(())
    }

    fn load_save_ram(&mut self) {
        if !self.snem_core.cartridge_has_save_ram() {
            return;
        }

        let Some(path) = self.cartridge_save_ram_path() else {
            return;
        };

        let save_data = match std::fs::read(&path) {
            Ok(data) => data,
            Err(e) => {
                log::error!("Failed to read save data from file '{}': {e}", path.to_string_lossy());
                return;
            }
        };

        if let Err(e) = self.snem_core.load_save_ram(save_data) {
            log::error!("Could not load previous save: {e}");
        } else {
            log::info!("Loaded previous save from '{}'", path.to_string_lossy());
        }
    }

    #[cfg(not(feature = "debug"))]
    fn toggle_pause(&mut self) {
        self.state.is_paused = !self.state.is_paused;

        if self.state.is_paused {
            self.audio_stream.pause().unwrap();
            log::trace!("Paused emulation");
        } else {
            self.audio_stream.resume().unwrap();
            log::trace!("Resumed emulation");
        }
    }

    #[cfg(feature = "debug")]
    fn toggle_pause(&mut self) {
        self.state.is_paused = !self.state.is_paused;

        if self.state.is_paused {
            self.debug_harness.stop_emulation = false;
            self.debug_harness.stop_condition = None;
            
            if let Err(e) = self.audio_stream.clear() {
                log::error!("failed to clear audio stream: {}", e);
            }

            log::trace!("Paused emulation");
        } else {
            if let Err(e) = self.audio_stream.clear() {
                log::error!("failed to clear audio stream: {}", e);
            }

            if let Some(debug_window) = &mut self.debug_window {
                debug_window.resume();
            }

            log::trace!("Resumed emulation");
        }
    }

    fn reset_emulation(&mut self, hard_reset: bool) {
        self.audio_buffer.clear();
        self.audio_stream.pause().unwrap();
        self.audio_stream.clear().unwrap();
        self.audio_stream.put_data_i16(&[0; AUDIO_SAMPLES_PER_FRAME]).unwrap();
        self.audio_stream.resume().unwrap();

        self.clear_frame_buf();

        if hard_reset {
            log::info!("Reset core to power-on state");

            self.snem_core.power_on(&mut self.debug_harness);
        } else {
            log::info!("Soft reset core");

            self.snem_core.reset(&mut self.debug_harness);
        }
    }

    fn cartridge_save_ram_path(&mut self) -> Option<PathBuf> {
        let path = self.state.loaded_rom_data.as_ref()?.paths.sav_path();
        Some(path)
    }

    fn save_cartridge_save_ram(&mut self, is_auto: bool) {
        let Some(loaded_rom) = &self.state.loaded_rom_data else {
            return;
        };

        if !self.snem_core.cartridge_has_save_ram() {
            return;
        }

        if is_auto && !self.snem_core.sram_changed() {
            log::info!("S-RAM is clean, skipping autosave.");
            return;
        }

        let sram = self.snem_core.get_cart_save_ram();

        if sram.len() == 0 {
            return;
        }

        let path = loaded_rom.paths.sav_path();

        match std::fs::write(path.clone(), sram) {
            Err(e) => {
                let message = format!("Failed to write save to '{}': {e}", path.to_string_lossy());
            
                if is_auto {
                    log::warn!("{}", message);
                } else {
                    log::error!("{}", message);
                }
            }
            _ => {
                if is_auto {
                    log::info!("Autosaved to '{}'", path.to_string_lossy());
                } else {
                    log::info!("Saved game to '{}'", path.to_string_lossy());
                }
            }
        }
    }

    fn try_save_state(&mut self, slot: usize) -> Result<()> {
        let Some(loaded_rom) = &mut self.state.loaded_rom_data else {
            return Err(anyhow!("cannot save state with no rom loaded"));
        };

        let path = loaded_rom.paths.state_path(slot as u32);
        let state = self.snem_core.save_state();
        let config = bincode_next::config::standard();
        let bytes: Vec<u8> = bincode_next::serde::encode_to_vec(state, config)?;

        std::fs::write(path.clone(), bytes)?;

        loaded_rom.used_save_state_slots[slot] = true;

        log::info!("Saved state '{}'", path.to_string_lossy());

        Ok(())
    }

    fn try_load_state(&mut self, slot: usize) -> Result<()> {
        let Some(loaded_rom) = &self.state.loaded_rom_data else {
            return Err(anyhow!("cannot load state with no rom loaded"));
        };

        let path = loaded_rom.paths.state_path(slot as u32);

        let bytes = std::fs::read(path.clone())?;
        let config = bincode_next::config::standard();
        let (state, _bytes_read): (SaveState, usize) = bincode_next::serde::decode_from_slice(&bytes, config)?;

        self.snem_core.try_load_state(state)?;

        log::info!("Loaded state from '{}'", path.to_string_lossy());

        Ok(())
    }

    fn toggle_fullscreen(&mut self) {
        self.state.is_fullscreen = !self.state.is_fullscreen;

        if let Err(e) = self.main_window.set_fullscreen(self.state.is_fullscreen) {
            self.state.is_fullscreen = !self.state.is_fullscreen;

            log::error!("Failed to toggle fullscreen: {}", e);
        }
    }

    fn show_settings(&mut self) {
        if self.settings_window.is_some() {
            return;
        }

        if !self.state.is_paused {
            self.toggle_pause();
        }

        let settings_egui_window = Self::create_window(
            "Settings",
            settings::SETTINGS_WINDOW_WIDTH,
            settings::SETTINGS_WINDOW_HEIGHT,
            &self.video_subsystem,
            &self.fonts,
            &self.theme,
        );

        if let Err(e) = settings_egui_window {
            log::error!("Failed to create settings window: {}", e);
            return;
        }

        match SettingsWindow::new(settings_egui_window.unwrap()) {
            Ok(window) => self.settings_window = Some(window),
            Err(e) => log::error!("Failed to create settings window: {}", e),
        }
    }
}

#[cfg(feature = "debug")]
impl SnemulatorApp {
    fn show_debug(&mut self) {
        if self.debug_window.is_some() {
            return;
        }

        if self.snem_core.cart.is_none() {
            if let Err(e) = self.try_load_rom() {
                log::error!("Cannot debug without ROM loaded: {}", e);
                return;
            }
        }

        // File dialog closed without selecting a ROM
        if self.snem_core.cart.is_none() {
            return;
        }

        let debug_egui_window = Self::create_window(
            "Debug",
            crate::debug::window::DEBUG_WINDOW_WIDTH,
            crate::debug::window::DEBUG_WINDOW_HEIGHT,
            &self.video_subsystem,
            &self.fonts,
            &self.theme,
        );

        if let Err(e) = debug_egui_window {
            log::error!("Failed to create debug window: {}", e);
            return;
        }

        match DebugWindow::new(debug_egui_window.unwrap()) {
            Ok(window) => self.debug_window = Some(window),
            Err(e) => log::error!("Failed to create debug window: {}", e),
        }

        if self.debug_window.is_some() {
            self.state.is_paused = true;
        }
    }

    fn update_debug_window(&mut self) {
        if self.debug_window.is_none() {
            return;
        }

        if self.debug_harness.stop_emulation && !self.state.is_paused {
            self.toggle_pause();
            self.debug_harness.stop_emulation = false;
            self.debug_harness.stop_condition = None;
        }

        let debug_action = self.debug_window.as_mut().unwrap().update_and_render(
            &mut self.snem_core,
            &mut self.state,
            &self.theme,
            &mut self.debug_harness,
            &mut self.audio_stream,
        );

        match debug_action {
            AppAction::TogglePause => {
                self.toggle_pause();
            }
            AppAction::ResetCore => {
                self.reset_emulation(false);
            }
            AppAction::PowerOnCore => {
                self.reset_emulation(true);
            }
            _ => {}
        }
    }

    fn handle_debug_window_event(&mut self, event: &Event, modifiers: &egui::Modifiers) {
        match &event {
            Event::Window {
                win_event: sdl3::event::WindowEvent::CloseRequested,
                ..
            } => {
                self.debug_window = None;
            }
            _ => {
                self.debug_window
                    .as_mut()
                    .unwrap()
                    .handle_event(event, modifiers);
            }
        }
    }
}

impl SnemulatorApp {
    pub fn create_window(
        title: &str,
        width: u32,
        height: u32,
        video_subsystem: &VideoSubsystem,
        fonts: &egui::FontDefinitions,
        theme: &AppTheme,
    ) -> Result<UiWindow> {
        
        let mut window = video_subsystem
            .window(title, width, height)
            .opengl()
            .resizable()
            .build()?;
        
        let win_scale = window.display_scale();
        
        window.set_size(
            ((width as f32) * win_scale) as u32,
            ((height as f32) * win_scale) as u32
        )?;
        window.set_position(
            sdl3::video::WindowPos::Centered,
            sdl3::video::WindowPos::Centered
        );
        let window = window; // No longer mutable
        
        let text_input = video_subsystem.text_input();
        let gl_context = window.gl_create_context()?;

        window.gl_make_current(&gl_context)?;

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                match video_subsystem.gl_get_proc_address(s) {
                    Some(ptr) => ptr as *const _,
                    None => std::ptr::null(),
                }
            })
        };
        
        let gl = std::sync::Arc::new(gl);
        let egui_ctx = egui::Context::default();

        egui_extras::install_image_loaders(&egui_ctx);
        
        egui_ctx.set_fonts(fonts.clone());
        theme.apply(&egui_ctx);

        let egui_painter = egui_glow::Painter::new(gl.clone(), "", None, false)?;
        let ui_scale = window.display_scale();
        
        egui_ctx.set_pixels_per_point(ui_scale);
        
        Ok(UiWindow {
            window,
            raw_input: None,
            text_input,
            egui_ctx: egui_ctx,
            egui_painter: Some(egui_painter),
            gl,
            gl_context,
            ui_scale,
        })
    }
}