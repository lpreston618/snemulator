use anyhow::Result;
use glow::HasContext;
use log::{info, warn, error, trace};
use rfd::FileDialog;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use sdl3::video::GLProfile;
use std::time::{Duration, Instant};
use crate::app::about_window::AboutWindow;
use crate::app::main_window::MainWindow;
use crate::app::settings::AppSettings;
use crate::core::sysinfo::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::core::snemcore::Snemulator;
use crate::core::controller::{ControllerPlayer, JoypadButton};

mod about_window;
mod main_window;
mod menu;
mod settings;

pub const FRAME_BUF_SIZE: usize = (2*SCREEN_WIDTH * 2*SCREEN_HEIGHT * 4) as usize;
pub const AUDIO_SAMPLE_HZ: usize = 44100;

pub const WINDOW_WIDTH: u32 = 640;
pub const WINDOW_HEIGHT: u32 = 480;

const TARGET_FPS: u32 = 60;
const SECS_BEFORE_HIDE_MENU: f32 = 3.0;
const SECS_BEFORE_HIDE_MOUSE: f32 = 3.0;
const FRAMES_BEFORE_HIDE_MENU: u64 = (SECS_BEFORE_HIDE_MENU * TARGET_FPS as f32) as u64;
const FRAMES_BEFORE_HIDE_MOUSE: u64 = (SECS_BEFORE_HIDE_MOUSE * TARGET_FPS as f32) as u64;

enum SnemulatorAppAction {
    Continue,
    TogglePause,
    ToggleFullscreen,
    LoadRom,
    ResetCore,
    SaveState,
    LoadState,
    ShowAbout,
    ShowSettings,
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
    state: AppState,
    settings: AppSettings,
    
    snem_core: Snemulator,
    frame_buffer: Box<[u8; FRAME_BUF_SIZE]>,
    
    start_time: std::time::Instant,
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
        let settings = SnemulatorApp::try_find_settings().unwrap_or_default();
        
        let main_window = MainWindow::new(&video_subsystem, &settings)?;
        
        Ok(Self {
            sdl_context,
            video_subsystem,
            
            main_window,
            about_window: None,
            state,
            settings,
            
            snem_core: Snemulator::new(),
            frame_buffer: Box::new([0u8; FRAME_BUF_SIZE]),
            
            start_time: std::time::Instant::now(),
        })
    }
    
    fn try_find_settings() -> Option<AppSettings> {
        None
    }

    pub fn run(&mut self) -> Result<()> {
        const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);
        
        'running: loop {
            let frame_start = Instant::now();
            
            let mut raw_input = egui::RawInput::default();
            
            let app_action = self.handle_input(&mut raw_input);
            
            match app_action {
                SnemulatorAppAction::Continue => {},
                SnemulatorAppAction::Exit => break 'running,
                _ => { self.do_action(app_action); }
            }
            
            // Emulate one frame
            if self.state.rom_loaded && !self.state.is_paused && (!self.state.is_minimized || !self.settings.pause_on_minimize) {
                let mut temp = Vec::new();
                
                self.snem_core.run_frame(&mut self.frame_buffer[..], &mut temp);
            }
            
            let app_action = self.main_window.render(&self.state, raw_input, &self.frame_buffer[..])?;

            match app_action {
                SnemulatorAppAction::Continue => {}
                SnemulatorAppAction::Exit => break 'running,
                _ => { self.do_action(app_action); }
            }
            
            // Frame timing
            self.state.frame_count += 1;
            let elapsed = frame_start.elapsed();
            
            // info!("Frame time: {} us, Time left: {} us", elapsed.as_micros(), FRAME_DURATION.as_micros() - elapsed.as_micros());
            
            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
        }
        
        // Cleanup
        self.main_window.cleanup();

        Ok(())
    }
    
    fn handle_input(&mut self, raw_input: &mut egui::RawInput) -> SnemulatorAppAction {
        let mut app_action = SnemulatorAppAction::Continue;
        
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
                    if event_win_id == about_window.window.id() {
                        self.handle_about_window_event(&event);
                        continue;
                    }
                }
                
                if event_win_id != self.main_window.window.id() {
                    continue;
                }
            }

            // Event is for main window
            self.main_window.handle_sdl_event(&event, raw_input);

            match event {
                Event::Quit { .. } => {
                    info!("Quit event received, exiting.");
                    
                    app_action = SnemulatorAppAction::Exit;
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
            _ => {}
        }
    }
    
    fn do_action(&mut self, app_action: SnemulatorAppAction) {
        match app_action {
            SnemulatorAppAction::LoadRom => self.load_rom(),
            SnemulatorAppAction::LoadState => self.save_state(),
            SnemulatorAppAction::SaveState => self.load_state(),
            SnemulatorAppAction::ResetCore => self.reset_emulation(),
            SnemulatorAppAction::ShowAbout => self.show_about(),
            SnemulatorAppAction::ShowSettings => self.show_settings(),
            SnemulatorAppAction::ToggleFullscreen => self.toggle_fullscreen(),
            SnemulatorAppAction::TogglePause => self.toggle_pause(),
            _ => {}
        }
    }
    
    // fn handle_sdl_event(&mut self, event: &Event, raw_input: &mut egui::RawInput) {
    //     match event {
    //         Event::MouseMotion { x, y, .. } => {
    //             // Convert physical pixels to logical pixels
    //             let logical_x = *x as f32 / self.ui_scale;
    //             let logical_y = *y as f32 / self.ui_scale;
    //             raw_input.events.push(egui::Event::PointerMoved(egui::Pos2::new(logical_x, logical_y)));
    //             self.last_mouse_input_frame = self.frame_count;
    //         }
    //         Event::MouseButtonDown { mouse_btn, x, y, .. } => {
    //             if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
    //                 // Convert physical pixels to logical pixels
    //                 let logical_x = *x as f32 / self.ui_scale;
    //                 let logical_y = *y as f32 / self.ui_scale;
    //                 raw_input.events.push(egui::Event::PointerButton {
    //                     pos: egui::Pos2::new(logical_x, logical_y),
    //                     button,
    //                     pressed: true,
    //                     modifiers: Default::default(),
    //                 });
    //             }
    //             self.last_mouse_input_frame = self.frame_count;
    //         }
    //         Event::MouseButtonUp { mouse_btn, x, y, .. } => {
    //             if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
    //                 // Convert physical pixels to logical pixels
    //                 let logical_x = *x as f32 / self.ui_scale;
    //                 let logical_y = *y as f32 / self.ui_scale;
    //                 raw_input.events.push(egui::Event::PointerButton {
    //                     pos: egui::Pos2::new(logical_x, logical_y),
    //                     button,
    //                     pressed: false,
    //                     modifiers: Default::default(),
    //                 });
    //             }
    //             self.last_mouse_input_frame = self.frame_count;
    //         }
    //         _ => {}
    //     }
    // }

    fn handle_keydown(&mut self, keycode: Keycode, keymod: Mod) -> SnemulatorAppAction {
        let mut app_action = SnemulatorAppAction::Continue;
        
        match keycode {
            Keycode::F11 => { app_action = SnemulatorAppAction::ToggleFullscreen; },
            Keycode::Escape => {
                if self.state.is_fullscreen {
                    app_action = SnemulatorAppAction::ToggleFullscreen;
                }
            }
            Keycode::Q => {
                if keymod.contains(Mod::LCTRLMOD) {
                    info!("Ctrl+Q pressed, exiting");
                    
                    app_action = SnemulatorAppAction::Exit;
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
    
    // fn update_game_texture(&self) {
    //     unsafe {
    //         if let Some(texture) = self.game_texture {
    //             self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
    //             self.gl.tex_sub_image_2d(
    //                 glow::TEXTURE_2D,
    //                 0,
    //                 0,
    //                 0,
    //                 SCREEN_WIDTH as i32,
    //                 SCREEN_HEIGHT as i32,
    //                 glow::RGBA,
    //                 glow::UNSIGNED_BYTE,
    //                 glow::PixelUnpackData::Slice(Some(&self.frame_buffer)),
    //             );
    //         }
    //     }
    // }
    
    // fn render_game_screen(&self, available_rect: egui::Rect, window_width: f32, window_height: f32) {
    //     unsafe {
    //         if let Some(texture) = self.game_texture {
    //             // Calculate aspect ratio
    //             let game_aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
    //             let available_width = available_rect.width();
    //             let available_height = available_rect.height();
    //             let available_aspect = available_width / available_height;
                
    //             // Calculate size maintaining aspect ratio
    //             let (render_width, render_height) = if available_aspect > game_aspect {
    //                 // Available space is wider - fit to height
    //                 let h = available_height;
    //                 let w = h * game_aspect;
    //                 (w, h)
    //             } else {
    //                 // Available space is taller - fit to width
    //                 let w = available_width;
    //                 let h = w / game_aspect;
    //                 (w, h)
    //             };
                
    //             // Center the game screen in available space
    //             let x = available_rect.left() + (available_width - render_width) / 2.0;
    //             let y = available_rect.top() + (available_height - render_height) / 2.0;
                
    //             // Convert to OpenGL normalized device coordinates
    //             let ndc_x = (x / window_width) * 2.0 - 1.0;
    //             let ndc_y = 1.0 - (y / window_height) * 2.0;
    //             let ndc_w = (render_width / window_width) * 2.0;
    //             let ndc_h = (render_height / window_height) * 2.0;
                
    //             // Update vertex positions for this specific area
    //             let vertices: [f32; 24] = [
    //                 // positions                    // texCoords
    //                 ndc_x,          ndc_y,          0.0, 0.0,  // top-left
    //                 ndc_x,          ndc_y - ndc_h,  0.0, 1.0,  // bottom-left
    //                 ndc_x + ndc_w,  ndc_y - ndc_h,  1.0, 1.0,  // bottom-right
                    
    //                 ndc_x,          ndc_y,          0.0, 0.0,  // top-left
    //                 ndc_x + ndc_w,  ndc_y - ndc_h,  1.0, 1.0,  // bottom-right
    //                 ndc_x + ndc_w,  ndc_y,          1.0, 0.0,  // top-right
    //             ];
                
    //             // Update VBO
    //             self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
    //             self.gl.buffer_data_u8_slice(
    //                 glow::ARRAY_BUFFER,
    //                 std::slice::from_raw_parts(
    //                     vertices.as_ptr() as *const u8,
    //                     vertices.len() * std::mem::size_of::<f32>(),
    //                 ),
    //                 glow::STATIC_DRAW,
    //             );
                
    //             self.gl.use_program(Some(self.shader_program));
    //             self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
    //             self.gl.bind_vertex_array(Some(self.vao));
    //             self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
    //             self.gl.bind_vertex_array(None);
    //         }
    //     }
    // }
    
    // fn render_ui(&mut self, ctx: &egui::Context) -> egui::Rect {
    //     // Top menu bar
    //     if self.show_menu && !self.is_fullscreen {
    //         egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
    //             egui::MenuBar::new().ui(ui, |ui| {                
    //                 ui.menu_button("File", |ui| {
    //                     ui.set_width(100.0);
                        
    //                     if ui.button("Load Rom").clicked() {
    //                         // TODO
    //                         if let Err(e) = self.load_rom() {
    //                             error!("Failed to load rom: {}", e);
    //                         }
    //                     }
    //                     if ui.button("Recent ROMs").clicked() {
    //                         warn!("Recent ROMs clicked.");
    //                     }
                        
    //                     ui.separator();
                        
    //                     if button_with_shortcut(ui, "Exit", "Ctrl + Q").clicked() {
    //                         info!("Exit button clicked, exiting");
                            
    //                         self.app_action = SnemulatorAppAction::Exit;
    //                     }
    //                 });
                    
    //                 ui.menu_button("Emulation", |ui| {
    //                     ui.set_width(100.0);
                        
    //                     let pause_text = if self.is_paused { "Resume" } else { "Pause" };
    //                     if ui.button(pause_text).clicked() {
    //                         self.toggle_pause();
    //                         ui.close();
    //                     }
    //                     if ui.button("Reset").clicked() {
    //                         self.reset_emulation();
    //                         ui.close();
    //                     }
                        
    //                     ui.separator();
                        
    //                     if ui.button("Save State").clicked() {
    //                         self.save_state();
    //                         ui.close();
    //                     }
    //                     if ui.button("Load State").clicked() {
    //                         self.load_state();
    //                         ui.close();
    //                     }
                        
    //                 });
                    
    //                 ui.menu_button("View", |ui| {
    //                     ui.set_width(100.0);
                        
    //                     let window_size_text = if self.is_fullscreen { "Windowed" } else { "Fullscreen" };
    //                     if button_with_shortcut(ui, window_size_text, "F11").clicked() {
    //                         if let Err(e) = self.toggle_fullscreen() {
    //                             error!("Failed to toggle fullscreen: {}", e);
    //                         }
                            
    //                         ui.close();
    //                     }
    //                 });
                    
    //                 ui.menu_button("About", |ui| {
    //                     ui.set_width(100.0);
                        
    //                     if ui.button("About").clicked() {
    //                         self.show_about = true;
    //                         ui.close();
    //                     }
    //                 })
    //             });
    
    //             //     ui.menu_button("View", |ui| {
    //             //         ui.checkbox(&mut self.show_menu, "Show Controls Panel");
    //             //     });
    //         });
    //     }

    //     // Side panel with controls
    //     // if self.show_menu {
    //     //     egui::SidePanel::right("controls")
    //     //         .default_width(220.0)
    //     //         .show(ctx, |ui| {
    //     //             ui.heading("🎮 Controls");
    //     //             ui.separator();

    //     //             ui.label("⬆️⬇️⬅️➡️ Arrow Keys: D-Pad");
    //     //             ui.label("🅰 Z: A Button");
    //     //             ui.label("🅱 X: B Button");
    //     //             ui.label("▶️ Enter: Start");
    //     //             ui.label("⏹️ Backspace: Select");

    //     //             ui.separator();
    //     //             ui.heading("📊 Emulator Info");
    //     //             ui.label(format!("Frame: {}", self.emulator.frame_count));
    //     //             ui.label(format!("Status: {}", 
    //     //                 if self.emulator.paused { "⏸ Paused" } else { "▶️ Running" }
    //     //             ));

    //     //             ui.separator();
                    
    //     //             ui.horizontal(|ui| {
    //     //                 if ui.button("🔄 Reset").clicked() {
    //     //                     self.emulator.reset();
    //     //                 }
    //     //                 let pause_text = if self.emulator.paused { "▶ Resume" } else { "⏸ Pause" };
    //     //                 if ui.button(pause_text).clicked() {
    //     //                     self.emulator.toggle_pause();
    //     //                 }
    //     //             });

    //     //             ui.separator();
    //     //             ui.heading("💾 Save States");
                    
    //     //             if ui.button("💾 Save State").clicked() {
    //     //                 self.emulator.save_state();
    //     //             }
    //     //             if ui.button("📂 Load State").clicked() {
    //     //                 self.emulator.load_state();
    //     //             }
    //     //         });
    //     // }

    //     // About window
    //     if self.show_about {
    //         egui::Window::new("About")
    //             .open(&mut self.show_about)
    //             .collapsible(false)
    //             .resizable(false)
    //             .show(ctx, |ui| {
    //                 ui.heading("🎮 SNES Emulator");
    //                 ui.label("Version 0.1.0");
    //                 ui.separator();
    //                 ui.label("A Super Nintendo Entertainment System emulator");
    //                 ui.label("written in Rust using SDL3 and egui.");
    //                 ui.separator();
    //                 ui.hyperlink_to("GitHub", "https://github.com/lpreston618/snemulator");
    //                 ui.separator();
    //                 // if ui.button("Close").clicked() {
    //                 //     // maybe send viewport cmd to close?
    //                 //     self.show_about = false;
    //                 // }
    //             });
    //     }

    //     // Status bar at bottom
    //     // egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
    //     //     ui.horizontal(|ui| {
    //     //         ui.label(format!("FPS: {}", 60));
    //     //         ui.separator();
    //     //         ui.label(format!("Frame: {}", self.emulator.frame_count));
    //     //         ui.separator();
    //     //         ui.label(if self.emulator.paused { "⏸ PAUSED" } else { "▶️ RUNNING" });
    //     //     });
    //     // });
        
    //     ctx.available_rect()
    // }
    
    // fn render(&mut self, available_rect: egui::Rect, egui_full_output: egui::FullOutput) -> Result<()> {
    //     let (window_width, window_height) = self.main_window.size();
        
    //     unsafe {
    //         self.gl.viewport(0, 0, window_width as i32, window_height as i32);
    //         self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
    //         self.gl.clear(glow::COLOR_BUFFER_BIT);
    //     }
        
    //     // Render snes frame buffer
    //     self.render_game_screen(available_rect, window_width as f32, window_height as f32);

    //     // Render egui
    //     let clipped_primitives = self.egui_context.tessellate(egui_full_output.shapes, egui_full_output.pixels_per_point);
        
    //     self.egui_painter.paint_and_update_textures(
    //         [window_width, window_height],
    //         egui_full_output.pixels_per_point,
    //         &clipped_primitives,
    //         &egui_full_output.textures_delta,
    //     );

    //     self.main_window.gl_swap_window();

    //     Ok(())
    // }
    
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
            let file_name = romfile.to_str().unwrap();
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
        if let Err(e) = self.try_toggle_fullscreen() {
            error!("Failed to toggle fullscreen: {}", e);
        }
    }
    
    fn try_toggle_fullscreen(&mut self) -> Result<()> {
        self.state.is_fullscreen = match self.main_window.window.fullscreen_state() {
            sdl3::video::FullscreenType::Off => true, // off -> on
            _ => false, // on -> off
        };
        
        self.main_window.window.set_fullscreen(self.state.is_fullscreen)?;
        
        Ok(())
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
        // match SettingsWindow::new(&self.video_subsystem) {
        //     Ok(window) => self.settings_window = Some(window),
        //     Err(e) => error!("Failed to create settings window: {}", e),
        // }
    }
}