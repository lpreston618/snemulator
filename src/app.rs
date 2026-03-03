use anyhow::Result;
use glow::HasContext;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use sdl3::pixels::PixelFormat;
use sdl3::sys::render::SDL_LOGICAL_PRESENTATION_LETTERBOX;
use sdl3::video::GLProfile;
use std::time::{Duration, Instant};
use crate::core::sysinfo::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::core::snemcore::Snemulator;
use crate::core::controller::{ControllerPlayer, JoypadButton};

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const TARGET_FPS: u32 = 60;
const FRAME_BUF_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize;

fn sdl_to_egui_mouse_button(button: sdl3::mouse::MouseButton) -> Option<egui::PointerButton> {
    match button {
        sdl3::mouse::MouseButton::Left => Some(egui::PointerButton::Primary),
        sdl3::mouse::MouseButton::Right => Some(egui::PointerButton::Secondary),
        sdl3::mouse::MouseButton::Middle => Some(egui::PointerButton::Middle),
        _ => None,
    }
}

fn button_with_shortcut(ui: &mut egui::Ui, label: &str, shortcut: &str) -> egui::Response {
    ui.add(egui::Button::new(label).right_text(egui::RichText::new(shortcut).weak()))
}

enum SnemulatorAppAction {
    Continue,
    Exit,
}

pub struct SnemulatorApp {
    sdl_context: sdl3::Sdl,
    _gl_context: sdl3::video::GLContext,
    egui_context: egui::Context,
    egui_painter: egui_glow::Painter,
    window: sdl3::video::Window,
    gl: std::sync::Arc<glow::Context>,
    snem_core: Snemulator,
    frame_buffer: Vec<u8>,
    game_texture: Option<glow::Texture>,
    
    start_time: std::time::Instant,
    show_menu: bool,
    show_mouse: bool,
    show_about: bool,
    is_paused: bool,
    is_fullscreen: bool,
    is_minimized: bool,
}

impl SnemulatorApp {
    pub fn new() -> Result<Self> {
        // Initialize SDL3 with OpenGL
        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;

        // Set OpenGL attributes
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_double_buffer(true);

        // Create window
        let window = video_subsystem
            .window("Emulator with egui", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .resizable()
            .opengl()
            .build()?;
        
        // Create OpenGL context
        let gl_context = window.gl_create_context()?;
        window.gl_make_current(&gl_context)?;
        
        // Load OpenGL functions
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                match video_subsystem.gl_get_proc_address(s) {
                    Some(func) => {
                        unsafe {
                            func as *const _
                        }
                    }
                    None => std::ptr::null(),
                }
            })
        };
        let gl = std::sync::Arc::new(gl);
        
        // Initialize egui
        let egui_context = egui::Context::default();
        let egui_painter = egui_glow::Painter::new(
            gl.clone(),
            "",
            None,
            false,
        ).map_err(|e| anyhow::anyhow!("Failed to create egui painter: {}", e))?;
        
        let frame_buffer = vec![0; FRAME_BUF_SIZE];
        
        // Create OpenGL texture for game screen
        let game_texture = unsafe {
            let texture = gl.create_texture()
                .map_err(|e| anyhow::anyhow!(e))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                SCREEN_WIDTH as i32,
                SCREEN_HEIGHT as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&frame_buffer)),
            );
            Some(texture)
        };
        
        let snem_core = Snemulator::new();
        
        Ok(Self {
            sdl_context,
            _gl_context: gl_context,
            egui_context,
            egui_painter,
            window,
            gl,
            snem_core,
            frame_buffer,
            game_texture,
            start_time: std::time::Instant::now(),
            show_menu: true,
            show_mouse: true,
            show_about: false,
            is_paused: false,
            is_fullscreen: false,
            is_minimized: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let frame_duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);

        'running: loop {
            let frame_start = Instant::now();
            
            let (window_width, window_height) = self.window.size();
            
            // Create egui input
            let mut raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(window_width as f32, window_height as f32),
                )),
                time: Some(self.start_time.elapsed().as_secs_f64()),
                ..Default::default()
            };
            
            let mut action = self.handle_input(&mut raw_input);

            match action {
                SnemulatorAppAction::Exit => break 'running,
                SnemulatorAppAction::Continue => {}
            }
            
            // Emulate one frame
            if !self.is_paused {
                self.snem_core.run_frame(&mut self.frame_buffer);
            }
            
            self.update_game_texture();
            
            // Run egui
            let ctx = self.egui_context.clone();
            let full_output = ctx.run(raw_input, |ctx| {
                action = self.render_ui(ctx);
            });
            
            match action {
                SnemulatorAppAction::Exit => break 'running,
                SnemulatorAppAction::Continue => {}
            }
            
            self.render(full_output)?;

            // Frame timing
            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        
        // Cleanup
        self.egui_painter.destroy();

        Ok(())
    }
    
    fn handle_input(&mut self, raw_input: &mut egui::RawInput) -> SnemulatorAppAction {
        let mut event_pump = self.sdl_context.event_pump().expect("Failed to get event pump");

        // Handle SDL events
        let mut action = SnemulatorAppAction::Continue;
        for event in event_pump.poll_iter() {
            // Convert SDL event to egui event
            self.handle_sdl_event(&event, raw_input);

            match event {
                Event::Quit { .. } => {
                    action = SnemulatorAppAction::Exit;
                }
                
                Event::Window { win_event, .. } => {
                    match win_event {
                        sdl3::event::WindowEvent::Resized(_, _) => {
                            
                        }
                        // sdl3::event::WindowEvent::Maximized => {
                        //     self.is_fullscreen = true;
                        // }
                        sdl3::event::WindowEvent::Minimized => {
                            self.is_minimized = true;
                        }
                        sdl3::event::WindowEvent::Restored
                        | sdl3::event::WindowEvent::Shown => {
                            self.is_minimized = false;
                        }
                        _ => {}
                    }
                }
                
                // Event::KeyDown {
                //     keycode: Some(Keycode::Escape),
                //     ..
                // } => should_quit = true,
                
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    action = self.handle_keydown(keycode, keymod);
                }
                
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => self.handle_keyup(keycode),
                
                _ => {}
            }
        }
        
        action
    }
    
    fn handle_sdl_event(&mut self, event: &Event, raw_input: &mut egui::RawInput) {
        match event {
            Event::MouseMotion { x, y, .. } => {
                raw_input.events.push(egui::Event::PointerMoved(egui::Pos2::new(*x as f32, *y as f32)));
            }
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::Pos2::new(*x as f32, *y as f32),
                        button,
                        pressed: true,
                        modifiers: Default::default(),
                    });
                }
            }
            Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::Pos2::new(*x as f32, *y as f32),
                        button,
                        pressed: false,
                        modifiers: Default::default(),
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_keydown(&mut self, keycode: Keycode, keymod: Mod) -> SnemulatorAppAction {
        let mut action = SnemulatorAppAction::Continue;
        
        match keycode {
            Keycode::F11 => {
                if let Err(e) = self.toggle_fullscreen() {
                    eprintln!("Failed to toggle fullscreen: {}", e);
                }
            }
            Keycode::Escape => {
                if self.is_fullscreen {
                    if let Err(e) = self.toggle_fullscreen() {
                        eprintln!("Failed to exit fullscreen: {}", e);
                    }
                }
            }
            Keycode::Q => {
                if keymod.contains(Mod::LCTRLMOD) {
                    action = SnemulatorAppAction::Exit;
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
        
        action
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
    
    fn update_game_texture(&self) {
        unsafe {
            if let Some(texture) = self.game_texture {
                self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                self.gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    0,
                    SCREEN_WIDTH as i32,
                    SCREEN_HEIGHT as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(&self.frame_buffer)),
                );
            }
        }
    }
    
    fn render_ui(&mut self, ctx: &egui::Context) -> SnemulatorAppAction {
        let mut action = SnemulatorAppAction::Continue;
        
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {                
                ui.menu_button("File", |ui| {
                    ui.set_width(100.0);
                    
                    if ui.button("Load Rom").clicked() {
                        self.load_rom();
                    }
                    if ui.button("Recent ROMs").clicked() {
                        println!("Recent ROMs clicked.");
                    }
                    
                    ui.separator();
                    
                    if button_with_shortcut(ui, "Exit", "Ctrl + Q").clicked() {
                        action = SnemulatorAppAction::Exit;
                    }
                });
                
                ui.menu_button("Emulation", |ui| {
                    ui.set_width(100.0);
                    
                    let pause_text = if self.is_paused { "Resume" } else { "Pause" };
                    if ui.button(pause_text).clicked() {
                        self.toggle_pause();
                        ui.close();
                    }
                    if ui.button("Reset").clicked() {
                        self.reset_emulation();
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Save State").clicked() {
                        self.save_state();
                        ui.close();
                    }
                    if ui.button("Load State").clicked() {
                        self.load_state();
                        ui.close();
                    }
                    
                });
                
                ui.menu_button("View", |ui| {
                    ui.set_width(100.0);
                    
                    let window_size_text = if self.is_fullscreen { "Windowed" } else { "Fullscreen" };
                    if button_with_shortcut(ui, window_size_text, "F11").clicked() {
                        if let Err(e) = self.toggle_fullscreen() {
                            eprintln!("Failed to toggle fullscreen: {}", e);
                        }
                        
                        ui.close();
                    }
                });
                
                ui.menu_button("About", |ui| {
                    ui.set_width(100.0);
                    
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                })
            });

            //     ui.menu_button("View", |ui| {
            //         ui.checkbox(&mut self.show_menu, "Show Controls Panel");
            //     });
        });

        // Side panel with controls
        // if self.show_menu {
        //     egui::SidePanel::right("controls")
        //         .default_width(220.0)
        //         .show(ctx, |ui| {
        //             ui.heading("🎮 Controls");
        //             ui.separator();

        //             ui.label("⬆️⬇️⬅️➡️ Arrow Keys: D-Pad");
        //             ui.label("🅰 Z: A Button");
        //             ui.label("🅱 X: B Button");
        //             ui.label("▶️ Enter: Start");
        //             ui.label("⏹️ Backspace: Select");

        //             ui.separator();
        //             ui.heading("📊 Emulator Info");
        //             ui.label(format!("Frame: {}", self.emulator.frame_count));
        //             ui.label(format!("Status: {}", 
        //                 if self.emulator.paused { "⏸ Paused" } else { "▶️ Running" }
        //             ));

        //             ui.separator();
                    
        //             ui.horizontal(|ui| {
        //                 if ui.button("🔄 Reset").clicked() {
        //                     self.emulator.reset();
        //                 }
        //                 let pause_text = if self.emulator.paused { "▶ Resume" } else { "⏸ Pause" };
        //                 if ui.button(pause_text).clicked() {
        //                     self.emulator.toggle_pause();
        //                 }
        //             });

        //             ui.separator();
        //             ui.heading("💾 Save States");
                    
        //             if ui.button("💾 Save State").clicked() {
        //                 self.emulator.save_state();
        //             }
        //             if ui.button("📂 Load State").clicked() {
        //                 self.emulator.load_state();
        //             }
        //         });
        // }

        // About window
        if self.show_about {
            egui::Window::new("About")
                .open(&mut self.show_about)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("🎮 SNES Emulator");
                    ui.label("Version 0.1.0");
                    ui.separator();
                    ui.label("A Super Nintendo Entertainment System emulator");
                    ui.label("written in Rust using SDL3 and egui.");
                    ui.separator();
                    ui.hyperlink_to("GitHub", "https://github.com/lpreston618/snemulator");
                    ui.separator();
                    // if ui.button("Close").clicked() {
                    //     // maybe send viewport cmd to close?
                    //     self.show_about = false;
                    // }
                });
        }

        // Status bar at bottom
        // egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        //     ui.horizontal(|ui| {
        //         ui.label(format!("FPS: {}", 60));
        //         ui.separator();
        //         ui.label(format!("Frame: {}", self.emulator.frame_count));
        //         ui.separator();
        //         ui.label(if self.emulator.paused { "⏸ PAUSED" } else { "▶️ RUNNING" });
        //     });
        // });

        // Central panel for game display
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.heading("Game Screen");
                ui.label("(Emulator output will render here)");
            });
        });
        
        action
    }
    
    fn render(&mut self, egui_full_output: egui::FullOutput) -> Result<()> {
        let (window_width, window_height) = self.window.size();
        
        unsafe {
            self.gl.viewport(0, 0, window_width as i32, window_height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
        
        // Render game screen (you'd render this properly with shaders)
        // For now we'll just clear to show egui works

        // Render egui
        let clipped_primitives = self.egui_context.tessellate(egui_full_output.shapes, egui_full_output.pixels_per_point);
        
        self.egui_painter.paint_and_update_textures(
            [window_width, window_height],
            egui_full_output.pixels_per_point,
            &clipped_primitives,
            &egui_full_output.textures_delta,
        );

        self.window.gl_swap_window();

        Ok(())
    }
    
    fn load_rom(&mut self) {
        println!("Load ROM called");
    }
    
    fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    
        if self.is_paused {
            println!("Paused emulation");
        } else {
            println!("Resumed emulation");
        }
    }
    
    fn reset_emulation(&mut self) {
        println!("Reset called");
    }
    
    fn save_state(&mut self) {
        println!("Save State called");
    }
    
    fn load_state(&mut self) {
        println!("Load State called");
    }
    
    fn toggle_fullscreen(&mut self) -> Result<()> {
        self.is_fullscreen = match self.window.fullscreen_state() {
            sdl3::video::FullscreenType::Off => true, // off -> on
            _ => false, // on -> off
        };
        
        println!("Fullscreen = {}", self.is_fullscreen);
        
        self.window.set_fullscreen(self.is_fullscreen)?;
        
        Ok(())
    }
}