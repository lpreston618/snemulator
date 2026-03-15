use anyhow::{Result, anyhow};
use log::{info, warn, error, trace};
use rfd::FileDialog;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use std::time::{Duration, Instant};
use crate::app::about_window::AboutWindow;
use crate::app::main_window::MainWindow;
use crate::app::settings::{Settings, SettingsWindow};
use crate::core::sysinfo::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::core::snemcore::Snemulator;
use crate::core::controller::{ControllerPlayer, JoypadButton};

mod about_window;
mod debug_window;
mod main_window;
mod ui_window;
mod menu;
mod settings;
mod utils;

pub const FRAME_BUF_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize;
pub const AUDIO_SAMPLE_HZ: usize = 44100;

pub const WINDOW_WIDTH: u32 = 640;
pub const WINDOW_HEIGHT: u32 = 480;

const TARGET_FPS: u32 = 60;
const SECS_BEFORE_HIDE_MENU: f32 = 3.0;
const SECS_BEFORE_HIDE_MOUSE: f32 = 3.0;
const FRAMES_BEFORE_HIDE_MENU: u64 = (SECS_BEFORE_HIDE_MENU * TARGET_FPS as f32) as u64;
const FRAMES_BEFORE_HIDE_MOUSE: u64 = (SECS_BEFORE_HIDE_MOUSE * TARGET_FPS as f32) as u64;

enum AppAction {
    Continue,
    TogglePause,
    ToggleFullscreen,
    LoadRom,
    ResetCore,
    SaveState,
    LoadState,
    OpenAbout,
    OpenSettings,
    Exit,
}

pub struct AppState {
    frame_count: u64,
    last_mouse_input_frame: u64,
    show_menu: bool,
    show_mouse: bool,
    is_paused: bool,
    is_fullscreen: bool,
    is_minimized: bool,
    rom_loaded: bool,
}

pub struct SnemulatorApp {
    sdl_context: sdl3::Sdl,
    video_subsystem: sdl3::VideoSubsystem,
    
    main_window: MainWindow,
    about_window: Option<AboutWindow>,
    settings_window: Option<SettingsWindow>,
    state: AppState,
    settings: Settings,
    
    snem_core: Snemulator,
    frame_buffer: Box<[u8; FRAME_BUF_SIZE]>,
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
        };
        
        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;
        let mut settings = SnemulatorApp::try_find_settings().unwrap_or_default();
        let frame_buffer = Box::new([0u8; FRAME_BUF_SIZE]);
        
        let main_window = MainWindow::new(&video_subsystem, &settings)?;
        
        // settings.always_show_menu = true;
        
        Ok(Self {
            sdl_context,
            video_subsystem,
            
            main_window,
            about_window: None,
            settings_window: None,
            state,
            settings,
            
            snem_core: Snemulator::new(),
            frame_buffer,
        })
    }
    
    fn try_find_settings() -> Option<Settings> {
        None
    }

    pub fn run(&mut self) -> Result<()> {
        const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);
        
        'running: loop {
            let frame_start = Instant::now();
            
            let app_action = self.handle_input();
            
            match app_action {
                AppAction::Continue => {},
                AppAction::Exit => break 'running,
                _ => { self.do_action(app_action); }
            }
            
            // Emulate one frame
            if self.state.rom_loaded {
                if !self.state.is_paused && (!self.state.is_minimized || !self.settings.pause_on_minimize){
                    let mut temp = Vec::new();
                    
                    self.snem_core.run_frame(&mut self.frame_buffer[..], &mut temp);
                }
            } else {
                self.render_load_rom_screen();
            }
            
            let app_action = self.main_window.update_and_render(&self.state, &self.settings, &self.frame_buffer[..]);

            match app_action {
                AppAction::Continue => {}
                AppAction::Exit => break 'running,
                _ => { self.do_action(app_action); }
            }
            
            if let Some(about_window) = &mut self.about_window {
                about_window.update_and_render();
            }
            
            if let Some(settings_window) = &mut self.settings_window {
                settings_window.update_and_render(&mut self.settings);
            }
            
            self.state.show_menu = self.settings.always_show_menu || (self.state.frame_count - self.state.last_mouse_input_frame < FRAMES_BEFORE_HIDE_MENU);
            self.state.show_mouse = match self.sdl_context.mouse().focused_window_id() {
                Some(id) => {
                    id != self.main_window.id() || (self.state.frame_count - self.state.last_mouse_input_frame < FRAMES_BEFORE_HIDE_MOUSE)
                }
                _ => true,
            };
            
            self.sdl_context.mouse().show_cursor(self.state.show_mouse);
            
            // Frame timing
            self.state.frame_count += 1;
            let elapsed = frame_start.elapsed();
            
            info!("Frame time: {} us, Time left: {} us", elapsed.as_micros(), FRAME_DURATION.as_micros() - elapsed.as_micros());
            
            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
        }

        Ok(())
    }
    
    fn handle_input(&mut self) -> AppAction {
        let mut app_action = AppAction::Continue;
        
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        
        for event in event_pump.poll_iter() {            
            // Route events to windows
            let event_window_id = match &event {
                Event::Window { window_id, .. } => Some(*window_id),
                Event::MouseMotion { window_id, .. } => Some(*window_id),
                Event::MouseButtonDown { window_id, .. } => Some(*window_id),
                Event::MouseButtonUp { window_id, .. } => Some(*window_id),
                _ => None,
            };

            // Check if event is for about window
            if let Some(event_win_id) = event_window_id {
                if let Some(about_window) = &mut self.about_window {
                    if event_win_id == about_window.id() {
                        self.handle_about_window_event(&event);
                        continue;
                    }
                }
                
                if let Some(settings_window) = &mut self.settings_window {
                    if event_win_id == settings_window.id() {
                        self.handle_settings_window_event(&event);
                        continue;
                    }
                }
                
                if event_win_id != self.main_window.id() {
                    continue;
                }
            }

            // Event is for main window
            self.main_window.handle_event(&event, &mut self.state);

            match event {
                Event::Quit { .. } => {
                    info!("Quit event received, exiting.");
                    
                    app_action = AppAction::Exit;
                }

                Event::KeyDown { keycode: Some(keycode), keymod, .. } => {
                    app_action = self.handle_keydown(keycode, keymod);
                },
                
                Event::KeyUp { keycode: Some(keycode), .. } => self.handle_keyup(keycode),
                _ => {}
            }
        }
        
        app_action
    }
    
    fn handle_about_window_event(&mut self, event: &Event) {
        match &event {
            Event::Window { win_event: sdl3::event::WindowEvent::CloseRequested, .. } => {
                self.about_window = None;
            }
            _ => {
                self.about_window.as_mut().unwrap().handle_event(event);
            }
        }
    }
    
    fn handle_settings_window_event(&mut self, event: &Event) {
        match &event {
            Event::Window { win_event: sdl3::event::WindowEvent::CloseRequested, .. } => {
                self.settings_window = None;
            }
            _ => {
                self.settings_window.as_mut().unwrap().handle_event(event);
            }
        }
    }
    
    fn do_action(&mut self, app_action: AppAction) {
        match app_action {
            AppAction::LoadRom => self.load_rom(),
            AppAction::LoadState => self.save_state(),
            AppAction::SaveState => self.load_state(),
            AppAction::ResetCore => self.reset_emulation(),
            AppAction::OpenAbout => self.show_about(),
            AppAction::OpenSettings => self.show_settings(),
            AppAction::ToggleFullscreen => self.toggle_fullscreen(),
            AppAction::TogglePause => self.toggle_pause(),
            _ => {}
        }
    }

    fn handle_keydown(&mut self, keycode: Keycode, keymod: Mod) -> AppAction {
        let mut app_action = AppAction::Continue;
        
        match keycode {
            Keycode::F11 => { app_action = AppAction::ToggleFullscreen; },
            Keycode::Escape => {
                if self.state.is_fullscreen {
                    app_action = AppAction::ToggleFullscreen;
                }
            }
            Keycode::Q => {
                if keymod.contains(Mod::LCTRLMOD) {
                    info!("Ctrl+Q pressed, exiting");
                    
                    app_action = AppAction::Exit;
                }
            }
         
            Keycode::Up => self.snem_core.set_button(ControllerPlayer::Player1, JoypadButton::Up, true),
            Keycode::Down => self.snem_core.set_button(ControllerPlayer::Player1, JoypadButton::Down, true),
            Keycode::Left => self.snem_core.set_button(ControllerPlayer::Player1, JoypadButton::Left, true),
            Keycode::Right => self.snem_core.set_button(ControllerPlayer::Player1, JoypadButton::Right, true),
            Keycode::Z => self.snem_core.set_button(ControllerPlayer::Player2, JoypadButton::A, true),
            Keycode::X => self.snem_core.set_button(ControllerPlayer::Player2, JoypadButton::B, true),
            Keycode::Return => self.snem_core.set_button(ControllerPlayer::Player2, JoypadButton::Start, true),
            Keycode::Backspace => self.snem_core.set_button(ControllerPlayer::Player2, JoypadButton::Select, true),
            
            _ => {}
        }
        
        app_action
    }

    fn handle_keyup(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Up => self.snem_core.set_button(ControllerPlayer::Player1, JoypadButton::Up, false),
            Keycode::Down => self.snem_core.set_button(ControllerPlayer::Player1, JoypadButton::Down, false),
            Keycode::Left => self.snem_core.set_button(ControllerPlayer::Player1, JoypadButton::Left, false),
            Keycode::Right => self.snem_core.set_button(ControllerPlayer::Player1, JoypadButton::Right, false),
            Keycode::Z => self.snem_core.set_button(ControllerPlayer::Player2, JoypadButton::A, false),
            Keycode::X => self.snem_core.set_button(ControllerPlayer::Player2, JoypadButton::B, false),
            Keycode::Return => self.snem_core.set_button(ControllerPlayer::Player2, JoypadButton::Start, false),
            Keycode::Backspace => self.snem_core.set_button(ControllerPlayer::Player2, JoypadButton::Select, false),
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
        
        self.frame_buffer.chunks_mut(4).take(SCREEN_WIDTH as usize).for_each(|pixel| {
            pixel[0] = 255;
        });
        
        self.frame_buffer.chunks_mut(4).skip((SCREEN_HEIGHT - 1) as usize * SCREEN_WIDTH as usize).for_each(|pixel| {
            pixel[0] = 255;
        });
        
        self.frame_buffer.chunks_mut(4 * SCREEN_WIDTH as usize).for_each(|row| {
            let lpixel = &mut row[0..4];
            lpixel[0] = 255;
            
            let rpixel = &mut row[4 * (SCREEN_WIDTH as usize - 1)..];
            rpixel[0] = 255;
        });
    }
    
    fn load_rom(&mut self) {
        if let Err(e) = self.try_load_rom() {
            error!("Failed to load rom: {}", e);
        }
    }
    
    fn try_load_rom(&mut self) -> Result<()> {
        let romfile = FileDialog::new()
            .add_filter("ROM", &["sfc", "smc"])
            .set_directory("/")
            .pick_file();
        
        if let Some(romfile) = romfile {            
            let file_name = romfile.to_str()
                .ok_or_else(|| anyhow!("Invalid file name"))?
                .to_string();
            
            info!("Trying to load rom '{}'", file_name);
            
            let data = std::fs::read(&romfile)?;
            
            self.snem_core.load_rom(data)?;
            
            info!("Loaded rom '{file_name}'");
            
            self.state.rom_loaded = true;
        }
        
        Ok(())
    }
    
    fn toggle_pause(&mut self) {
        self.state.is_paused = !self.state.is_paused;
    
        if self.state.is_paused {
            trace!("Paused emulation");
        } else {
            trace!("Resumed emulation");
        }
    }
    
    fn reset_emulation(&mut self) {
        warn!("Reset called");
    }
    
    fn save_state(&mut self) {
        warn!("Save State called");
    }
    
    fn load_state(&mut self) {
        warn!("Load State called");
    }
    
    fn toggle_fullscreen(&mut self) {
        self.state.is_fullscreen = !self.state.is_fullscreen;
        
        if let Err(e) = self.main_window.set_fullscreen(self.state.is_fullscreen) {
            self.state.is_fullscreen = !self.state.is_fullscreen;
            
            error!("Failed to toggle fullscreen: {}", e);
        }
    }
    
    fn show_about(&mut self) {
        if self.about_window.is_some() {
            return;
        }
        
        match AboutWindow::new(&self.video_subsystem) {
            Ok(window) => self.about_window = Some(window),
            Err(e) => error!("Failed to create about window: {}", e),
        }
    }
    
    fn show_settings(&mut self) {
        if self.settings_window.is_some() {
            return;
        }
        
        match SettingsWindow::new(&self.video_subsystem) {
            Ok(window) => self.settings_window = Some(window),
            Err(e) => error!("Failed to create settings window: {}", e),
        }
    }
}