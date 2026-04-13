#[cfg(feature = "debug")]
use crate::debug::window::DebugWindow;
#[cfg(feature = "debug")]
use crate::debug::debugger::Debugger;

use crate::windows::about::AboutWindow;
use crate::windows::game::MainWindow;
use crate::windows::settings::{Settings, SettingsWindow};
use anyhow::{anyhow, Result};
use rfd::FileDialog;
use ringbuf::HeapRb;
use ringbuf::traits::{Observer, RingBuffer};
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use snemcore::controller::{ControllerPlayer, JoypadButton};
use snemcore::sysinfo::{SCREEN_HEIGHT, SCREEN_WIDTH};
use snemcore::Snemulator;
use std::time::{Duration, Instant};

pub const FRAME_BUF_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize;

pub const WINDOW_WIDTH: u32 = 640;
pub const WINDOW_HEIGHT: u32 = 480;

const TARGET_FPS: u32 = 60;
const PREV_FPS_BUFFER_LEN: usize = TARGET_FPS as usize * 1;
const SECS_BEFORE_HIDE_MENU: f32 = 3.0;
const SECS_BEFORE_HIDE_MOUSE: f32 = 3.0;
const FRAMES_BEFORE_HIDE_MENU: u64 = (SECS_BEFORE_HIDE_MENU * TARGET_FPS as f32) as u64;
const FRAMES_BEFORE_HIDE_MOUSE: u64 = (SECS_BEFORE_HIDE_MOUSE * TARGET_FPS as f32) as u64;

pub enum AppAction {
    Continue,
    TogglePause,
    ToggleFullscreen,
    LoadRom,
    ResetCore,
    PowerOnCore,
    SaveState,
    LoadState,
    OpenAbout,
    OpenSettings,
    Exit,
    
    #[cfg(feature = "debug")]
    OpenDebug,
    #[cfg(feature = "debug")]
    CloseDebug,
}

pub struct AppState {
    pub frame_count: u64,
    pub last_mouse_input_frame: u64,
    pub show_menu: bool,
    pub show_mouse: bool,
    pub is_paused: bool,
    pub is_fullscreen: bool,
    pub is_minimized: bool,
    pub rom_loaded: bool,
    pub fps: f32,
    
    #[cfg(feature = "debug")]
    pub debug_active: bool,
}

pub struct SnemulatorApp {
    sdl_context: sdl3::Sdl,
    video_subsystem: sdl3::VideoSubsystem,
    event_pump: Option<sdl3::EventPump>,

    main_window: MainWindow,
    about_window: Option<AboutWindow>,
    settings_window: Option<SettingsWindow>,
    state: AppState,
    settings: Settings,
    prev_frame_micros: HeapRb<usize>,
    total_frame_micros: usize,
    last_frame: Instant,
    frame_buffer: Box<[u8; FRAME_BUF_SIZE]>,

    #[cfg(not(feature = "debug"))]
    snem_core: Snemulator,
    
    #[cfg(feature = "debug")]
    snem_core: Snemulator<Debugger>,
    #[cfg(feature = "debug")]
    debug_window: Option<DebugWindow>,
}

impl SnemulatorApp {
    pub fn new() -> Result<Self> {
        let state = AppState {
            frame_count: 0,
            last_mouse_input_frame: 0,
            show_menu: true,
            show_mouse: true,
            is_paused: false,
            is_fullscreen: false,
            is_minimized: false,
            rom_loaded: false,
            fps: 0.0,
            
            #[cfg(feature = "debug")]
            debug_active: false,
        };

        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;
        let event_pump = Some(sdl_context.event_pump()?);
        let settings = SnemulatorApp::try_find_settings().unwrap_or_default();
        let frame_buffer = Box::new([0u8; FRAME_BUF_SIZE]);
        let main_window = MainWindow::new(&video_subsystem, &settings)?;
        
        #[cfg(feature = "debug")]
        let snem_core = Snemulator::with_probe(Debugger::new()?);
        #[cfg(not(feature = "debug"))]
        let snem_core = Snemulator::new();
        

        Ok(Self {
            sdl_context,
            video_subsystem,
            event_pump,

            main_window,
            about_window: None,
            settings_window: None,
            state,
            settings,
            prev_frame_micros: HeapRb::new(PREV_FPS_BUFFER_LEN),
            total_frame_micros: 0,
            last_frame: Instant::now(),
            
            snem_core,
            frame_buffer,
            
            #[cfg(feature = "debug")]
            debug_window: None,
        })
    }

    fn try_find_settings() -> Option<Settings> {
        None
    }

    pub fn run(&mut self) -> Result<()> {
        const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);

        'running: loop {
            #[cfg(feature = "debug")]
            {
                self.state.debug_active = self.debug_window.is_some();
            }

            let app_action = self.handle_input();

            match app_action {
                AppAction::Continue => {}
                AppAction::Exit => break 'running,
                _ => {
                    self.do_action(app_action);
                }
            }
            
            #[cfg(feature = "debug")]
            self.debug_update_emulator();
            #[cfg(not(feature = "debug"))]
            self.update_emulator();

            let app_action = self.main_window.update_and_render(
                &self.state,
                &self.settings,
                &self.frame_buffer[..],
            );

            match app_action {
                AppAction::Continue => {}
                AppAction::Exit => break 'running,
                _ => {
                    self.do_action(app_action);
                }
            }

            if let Some(about_window) = &mut self.about_window {
                about_window.update_and_render();
            }

            if let Some(settings_window) = &mut self.settings_window {
                settings_window.update_and_render(&mut self.settings);
            }

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

            // Frame timing
            self.state.frame_count += 1;
            let last_frame = self.last_frame;
            self.last_frame = Instant::now();
            
            let elapsed = self.last_frame - last_frame;
            
            self.update_fps(elapsed);

            // info!("Frame time: {} us, Time left: {} us", elapsed.as_micros(), FRAME_DURATION.as_micros() - elapsed.as_micros());

            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
        }

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
    
    #[cfg(feature = "debug")]
    fn debug_update_emulator(&mut self) {
        let mut temp = Vec::new();
        
        if let Some(debug_window) = &mut self.debug_window {
            let app_action = debug_window.update_and_render(
                &mut self.snem_core,
                &mut self.state,
                &mut self.frame_buffer[..],
                &mut temp,
            );

            match app_action {
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
        } else {
            self.update_emulator();
        }
    }
    
    fn update_emulator(&mut self) {
        let mut temp = Vec::new();
        
        if self.state.rom_loaded {
            if !self.state.is_paused
                && (!self.state.is_minimized || !self.settings.pause_on_minimize)
            {
                self.snem_core.run_frame(Some(&mut self.frame_buffer[..]), Some(&mut temp));
            }
        } else {
            self.render_load_rom_screen();
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
            let event_window_id = match &event {
                Event::Window { window_id, .. } => Some(*window_id),
                Event::MouseMotion { window_id, .. } => Some(*window_id),
                Event::MouseWheel { window_id, .. } => Some(*window_id),
                Event::MouseButtonDown { window_id, .. } => Some(*window_id),
                Event::MouseButtonUp { window_id, .. } => Some(*window_id),
                Event::KeyDown { window_id, .. } => Some(*window_id),
                Event::KeyUp { window_id, .. } => Some(*window_id),
                Event::TextInput { window_id, .. } => Some(*window_id),
                _ => None,
            };

            // Check if event is for about window
            if let Some(event_win_id) = event_window_id {
                if let Some(about_window) = &mut self.about_window {
                    if event_win_id == about_window.id() {
                        self.handle_about_window_event(&event, &modifiers);
                        continue;
                    }
                }

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
                    
                    self.about_window = None;
                    self.settings_window = None;
                    
                    #[cfg(feature = "debug")]
                    {
                        self.debug_window = None;
                    }

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

    fn handle_about_window_event(&mut self, event: &Event, modifiers: &egui::Modifiers) {
        match &event {
            Event::Window {
                win_event: sdl3::event::WindowEvent::CloseRequested,
                ..
            } => {
                self.about_window = None;
            }
            _ => {
                self.about_window
                    .as_mut()
                    .unwrap()
                    .handle_event(event, modifiers);
            }
        }
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

    #[cfg(feature = "debug")]
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

    fn do_action(&mut self, app_action: AppAction) {
        match app_action {
            AppAction::LoadRom => self.load_rom(),
            AppAction::LoadState => self.load_state(),
            AppAction::SaveState => self.save_state(),
            AppAction::ResetCore => self.reset_emulation(false),
            AppAction::PowerOnCore => self.reset_emulation(true),
            AppAction::OpenAbout => self.show_about(),
            AppAction::OpenSettings => self.show_settings(),
            AppAction::ToggleFullscreen => self.toggle_fullscreen(),
            AppAction::TogglePause => self.toggle_pause(),
            
            #[cfg(feature = "debug")]
            AppAction::OpenDebug => self.show_debug(),
            #[cfg(feature = "debug")]
            AppAction::CloseDebug => self.debug_window = None,
            
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
                    .set_button(ControllerPlayer::Player2, JoypadButton::A, true)
            }
            Keycode::X => {
                self.snem_core
                    .set_button(ControllerPlayer::Player2, JoypadButton::B, true)
            }
            Keycode::Return => {
                self.snem_core
                    .set_button(ControllerPlayer::Player2, JoypadButton::Start, true)
            }
            Keycode::Backspace => {
                self.snem_core
                    .set_button(ControllerPlayer::Player2, JoypadButton::Select, true)
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
                    .set_button(ControllerPlayer::Player2, JoypadButton::A, false)
            }
            Keycode::X => {
                self.snem_core
                    .set_button(ControllerPlayer::Player2, JoypadButton::B, false)
            }
            Keycode::Return => {
                self.snem_core
                    .set_button(ControllerPlayer::Player2, JoypadButton::Start, false)
            }
            Keycode::Backspace => {
                self.snem_core
                    .set_button(ControllerPlayer::Player2, JoypadButton::Select, false)
            }
            _ => {}
        }
    }

    fn render_load_rom_screen(&mut self) {
        self.frame_buffer.chunks_mut(4).for_each(|pixel| {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            pixel[3] = 255;
        });

        self.frame_buffer
            .chunks_mut(4)
            .take(SCREEN_WIDTH as usize)
            .for_each(|pixel| {
                pixel[0] = 255;
            });

        self.frame_buffer
            .chunks_mut(4)
            .skip((SCREEN_HEIGHT - 1) as usize * SCREEN_WIDTH as usize)
            .for_each(|pixel| {
                pixel[0] = 255;
            });

        self.frame_buffer
            .chunks_mut(4 * SCREEN_WIDTH as usize)
            .for_each(|row| {
                let lpixel = &mut row[0..4];
                lpixel[0] = 255;

                let rpixel = &mut row[4 * (SCREEN_WIDTH as usize - 1)..];
                rpixel[0] = 255;
            });
    }

    fn load_rom(&mut self) {
        if let Err(e) = self.try_load_rom() {
            log::error!("Failed to load rom: {}", e);
        }
    }

    fn try_load_rom(&mut self) -> Result<()> {
        let romfile = FileDialog::new()
            .add_filter("ROM", &["sfc", "smc"])
            .set_directory("/")
            .pick_file();

        if let Some(romfile) = romfile {
            let file_name = romfile
                .to_str()
                .ok_or_else(|| anyhow!("Invalid file name"))?
                .to_string();

            log::info!("Trying to load rom '{}'", file_name);

            let data = std::fs::read(&romfile)?;

            self.snem_core.load_rom(data)?;

            log::info!("Loaded rom '{file_name}'");

            self.state.rom_loaded = true;
        }

        Ok(())
    }

    fn toggle_pause(&mut self) {
        self.state.is_paused = !self.state.is_paused;

        if self.state.is_paused {
            log::trace!("Paused emulation");
        } else {
            log::trace!("Resumed emulation");
        }
    }

    fn reset_emulation(&mut self, hard_reset: bool) {
        if hard_reset {
            log::info!("Reset core to power-on state");

            self.snem_core.power_on();
        } else {
            log::info!("Soft reset core");

            self.snem_core.reset();
        }
    }

    fn save_state(&mut self) {
        log::warn!("Save State called");
    }

    fn load_state(&mut self) {
        log::warn!("Load State called");
    }

    fn toggle_fullscreen(&mut self) {
        self.state.is_fullscreen = !self.state.is_fullscreen;

        if let Err(e) = self.main_window.set_fullscreen(self.state.is_fullscreen) {
            self.state.is_fullscreen = !self.state.is_fullscreen;

            log::error!("Failed to toggle fullscreen: {}", e);
        }
    }

    fn show_about(&mut self) {
        if self.about_window.is_some() {
            return;
        }

        match AboutWindow::new(&self.video_subsystem) {
            Ok(window) => self.about_window = Some(window),
            Err(e) => log::error!("Failed to create about window: {}", e),
        }
    }

    fn show_settings(&mut self) {
        if self.settings_window.is_some() {
            return;
        }

        match SettingsWindow::new(&self.video_subsystem) {
            Ok(window) => self.settings_window = Some(window),
            Err(e) => log::error!("Failed to create settings window: {}", e),
        }
    }

    #[cfg(feature = "debug")]
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

        let mapping_mode = self.snem_core.cart.as_ref().unwrap().mapping_mode();

        match DebugWindow::new(&self.video_subsystem, mapping_mode) {
            Ok(window) => self.debug_window = Some(window),
            Err(e) => log::error!("Failed to create debug window: {}", e),
        }

        if self.debug_window.is_some() {
            self.state.is_paused = true;
            self.snem_core.init_probe();
        }
    }
}
