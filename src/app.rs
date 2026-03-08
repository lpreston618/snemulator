use anyhow::Result;
use glow::HasContext;
use log::{info, warn, error, trace};
use rfd::FileDialog;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use sdl3::video::GLProfile;
use std::time::{Duration, Instant};
use crate::core::sysinfo::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::core::snemcore::Snemulator;
use crate::core::controller::{ControllerPlayer, JoypadButton};

pub const FRAME_BUF_SIZE: usize = (2*SCREEN_WIDTH * 2*SCREEN_HEIGHT * 4) as usize;
pub const AUDIO_SAMPLE_HZ: usize = 44100;

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const TARGET_FPS: u32 = 60;
const SECS_BEFORE_HIDE_MENU: f32 = 3.0;
const SECS_BEFORE_HIDE_MOUSE: f32 = 3.0;
const FRAMES_BEFORE_HIDE_MENU: u64 = (SECS_BEFORE_HIDE_MENU * TARGET_FPS as f32) as u64;
const FRAMES_BEFORE_HIDE_MOUSE: u64 = (SECS_BEFORE_HIDE_MOUSE * TARGET_FPS as f32) as u64;

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

fn create_shader_program(gl: &glow::Context) -> Result<(glow::Program, glow::VertexArray, glow::Buffer), String> {
    // Create shader program for rendering game texture
    unsafe {
        // Vertex shader
        let vertex_shader_source = r#"
            #version 330 core
            layout (location = 0) in vec2 aPos;
            layout (location = 1) in vec2 aTexCoord;
            
            out vec2 TexCoord;
            
            void main() {
                gl_Position = vec4(aPos, 0.0, 1.0);
                TexCoord = aTexCoord;
            }
        "#;
        
        // Fragment shader
        let fragment_shader_source = r#"
            #version 330 core
            out vec4 FragColor;
            in vec2 TexCoord;
            
            uniform sampler2D gameTexture;
            
            void main() {
                FragColor = texture(gameTexture, TexCoord);
            }
        "#;
        
        let vertex_shader = gl.create_shader(glow::VERTEX_SHADER)?;
        gl.shader_source(vertex_shader, vertex_shader_source);
        gl.compile_shader(vertex_shader);
        
        let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER)?;
        gl.shader_source(fragment_shader, fragment_shader_source);
        gl.compile_shader(fragment_shader);
        
        let shader_program = gl.create_program()?;
        gl.attach_shader(shader_program, vertex_shader);
        gl.attach_shader(shader_program, fragment_shader);
        gl.link_program(shader_program);
        
        gl.delete_shader(vertex_shader);
        gl.delete_shader(fragment_shader);
        
        // Create VAO and VBO for a fullscreen quad
        let vertices: [f32; 24] = [
            // positions   // texCoords
            -1.0,  1.0,    0.0, 0.0,  // top-left
            -1.0, -1.0,    0.0, 1.0,  // bottom-left
             1.0, -1.0,    1.0, 1.0,  // bottom-right
            
            -1.0,  1.0,    0.0, 0.0,  // top-left
             1.0, -1.0,    1.0, 1.0,  // bottom-right
             1.0,  1.0,    1.0, 0.0,  // top-right
        ];
        
        let vao = gl.create_vertex_array()?;
        let vbo = gl.create_buffer()?;
        
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<f32>(),
            ),
            glow::STATIC_DRAW,
        );
        
        // Position attribute
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 4 * std::mem::size_of::<f32>() as i32, 0);
        gl.enable_vertex_attrib_array(0);
        
        // TexCoord attribute
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 4 * std::mem::size_of::<f32>() as i32, 2 * std::mem::size_of::<f32>() as i32);
        gl.enable_vertex_attrib_array(1);
        
        gl.bind_vertex_array(None);
        
        Ok((shader_program, vao, vbo))
    }
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
    ui_scale: f32,
    window: sdl3::video::Window,
    gl: std::sync::Arc<glow::Context>,
    game_texture: Option<glow::Texture>,
    shader_program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    
    snem_core: Snemulator,
    frame_buffer: Vec<u8>,
    
    start_time: std::time::Instant,
    frame_count: u64,
    last_mouse_input_frame: u64,
    show_menu: bool,
    show_mouse: bool,
    show_settings: bool,
    show_about: bool,
    is_paused: bool,
    is_fullscreen: bool,
    is_minimized: bool,
    pause_on_minimize: bool,
    rom_loaded: bool,
    
    app_action: SnemulatorAppAction,
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
                        func as *const _
                    }
                    None => std::ptr::null(),
                }
            })
        };
        let gl = std::sync::Arc::new(gl);
        
        // Initialize egui
        let egui_context = egui::Context::default();
        
        egui_context.set_pixels_per_point(window.display_scale());
        
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
        
        let (shader_program, vao, vbo) = create_shader_program(&gl).map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(Self {
            sdl_context,
            _gl_context: gl_context,
            egui_context,
            egui_painter,
            ui_scale: window.display_scale(),
            window,
            gl,
            snem_core: Snemulator::new(),
            frame_buffer,
            game_texture,
            shader_program,
            vao,
            vbo,
            start_time: std::time::Instant::now(),
            frame_count: 0,
            last_mouse_input_frame: 0,
            show_menu: true,
            show_mouse: true,
            show_settings: false,
            show_about: false,
            is_paused: false,
            is_fullscreen: false,
            is_minimized: false,
            pause_on_minimize: true,
            rom_loaded: false,
            app_action: SnemulatorAppAction::Continue,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);
        
        'running: loop {
            let frame_start = Instant::now();
            self.app_action = SnemulatorAppAction::Continue;
            
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
            
            self.handle_input(&mut raw_input);
            
            // Emulate one frame
            if self.rom_loaded && !self.is_paused && (!self.is_minimized || !self.pause_on_minimize) {
                self.snem_core.run_frame(&mut self.frame_buffer);
                
                self.update_game_texture();
            }
            
            // Run egui
            let ctx = self.egui_context.clone();
            let mut game_rect = egui::Rect::NOTHING;
            // game_rect = egui::Rect {
            //     min: egui::Pos2::ZERO,
            //     max: egui::Pos2::new(window_width as f32, window_height as f32),
            // };
            let full_output = ctx.run(raw_input, |ctx| {
                game_rect = self.render_ui(ctx);
            });
            
            self.render(game_rect, full_output)?;

            match self.app_action {
                SnemulatorAppAction::Exit => break 'running,
                SnemulatorAppAction::Continue => {}
            }
            
            // Frame timing
            self.frame_count += 1;
            let elapsed = frame_start.elapsed();
            
            // info!("Frame time: {} us, Time left: {} us", elapsed.as_micros(), FRAME_DURATION.as_micros() - elapsed.as_micros());
            
            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
        }
        
        // Cleanup
        self.egui_painter.destroy();

        Ok(())
    }
    
    fn handle_input(&mut self, raw_input: &mut egui::RawInput) {
        let mut event_pump = self.sdl_context.event_pump().expect("Failed to get event pump");

        // Handle SDL events
        for event in event_pump.poll_iter() {
            // Convert SDL event to egui event
            self.handle_sdl_event(&event, raw_input);

            match event {
                Event::Quit { .. } => {
                    info!("Quit event received, exiting");
                    
                    self.app_action = SnemulatorAppAction::Exit;
                }
                
                Event::Window { win_event, .. } => {
                    match win_event {
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
                } => self.handle_keydown(keycode, keymod),
                
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => self.handle_keyup(keycode),
                
                _ => {}
            }
        }
        
        self.show_menu = (self.frame_count - self.last_mouse_input_frame) < FRAMES_BEFORE_HIDE_MENU;
        self.show_mouse = (self.frame_count - self.last_mouse_input_frame) < FRAMES_BEFORE_HIDE_MOUSE;
        
        self.sdl_context.mouse().show_cursor(self.show_mouse);
    }
    
    fn handle_sdl_event(&mut self, event: &Event, raw_input: &mut egui::RawInput) {
        match event {
            Event::MouseMotion { x, y, .. } => {
                // Convert physical pixels to logical pixels
                let logical_x = *x as f32 / self.ui_scale;
                let logical_y = *y as f32 / self.ui_scale;
                raw_input.events.push(egui::Event::PointerMoved(egui::Pos2::new(logical_x, logical_y)));
                self.last_mouse_input_frame = self.frame_count;
            }
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
                    // Convert physical pixels to logical pixels
                    let logical_x = *x as f32 / self.ui_scale;
                    let logical_y = *y as f32 / self.ui_scale;
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::Pos2::new(logical_x, logical_y),
                        button,
                        pressed: true,
                        modifiers: Default::default(),
                    });
                }
                self.last_mouse_input_frame = self.frame_count;
            }
            Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
                    // Convert physical pixels to logical pixels
                    let logical_x = *x as f32 / self.ui_scale;
                    let logical_y = *y as f32 / self.ui_scale;
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::Pos2::new(logical_x, logical_y),
                        button,
                        pressed: false,
                        modifiers: Default::default(),
                    });
                }
                self.last_mouse_input_frame = self.frame_count;
            }
            _ => {}
        }
    }

    fn handle_keydown(&mut self, keycode: Keycode, keymod: Mod) {
        match keycode {
            Keycode::F11 => {
                if let Err(e) = self.toggle_fullscreen() {
                    error!("Failed to toggle fullscreen: {}", e);
                }
            }
            Keycode::Escape => {
                if self.is_fullscreen {
                    if let Err(e) = self.toggle_fullscreen() {
                        error!("Failed to exit fullscreen: {}", e);
                    }
                }
            }
            Keycode::Q => {
                if keymod.contains(Mod::LCTRLMOD) {
                    info!("Ctrl+Q pressed, exiting");
                    
                    self.app_action = SnemulatorAppAction::Exit;
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
    
    fn render_game_screen(&self, available_rect: egui::Rect, window_width: f32, window_height: f32) {
        unsafe {
            if let Some(texture) = self.game_texture {
                // Calculate aspect ratio
                let game_aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
                let available_width = available_rect.width();
                let available_height = available_rect.height();
                let available_aspect = available_width / available_height;
                
                // Calculate size maintaining aspect ratio
                let (render_width, render_height) = if available_aspect > game_aspect {
                    // Available space is wider - fit to height
                    let h = available_height;
                    let w = h * game_aspect;
                    (w, h)
                } else {
                    // Available space is taller - fit to width
                    let w = available_width;
                    let h = w / game_aspect;
                    (w, h)
                };
                
                // Center the game screen in available space
                let x = available_rect.left() + (available_width - render_width) / 2.0;
                let y = available_rect.top() + (available_height - render_height) / 2.0;
                
                // Convert to OpenGL normalized device coordinates
                let ndc_x = (x / window_width) * 2.0 - 1.0;
                let ndc_y = 1.0 - (y / window_height) * 2.0;
                let ndc_w = (render_width / window_width) * 2.0;
                let ndc_h = (render_height / window_height) * 2.0;
                
                // Update vertex positions for this specific area
                let vertices: [f32; 24] = [
                    // positions                    // texCoords
                    ndc_x,          ndc_y,          0.0, 0.0,  // top-left
                    ndc_x,          ndc_y - ndc_h,  0.0, 1.0,  // bottom-left
                    ndc_x + ndc_w,  ndc_y - ndc_h,  1.0, 1.0,  // bottom-right
                    
                    ndc_x,          ndc_y,          0.0, 0.0,  // top-left
                    ndc_x + ndc_w,  ndc_y - ndc_h,  1.0, 1.0,  // bottom-right
                    ndc_x + ndc_w,  ndc_y,          1.0, 0.0,  // top-right
                ];
                
                // Update VBO
                self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
                self.gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    std::slice::from_raw_parts(
                        vertices.as_ptr() as *const u8,
                        vertices.len() * std::mem::size_of::<f32>(),
                    ),
                    glow::STATIC_DRAW,
                );
                
                self.gl.use_program(Some(self.shader_program));
                self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                self.gl.bind_vertex_array(Some(self.vao));
                self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
                self.gl.bind_vertex_array(None);
            }
        }
    }
    
    fn render_ui(&mut self, ctx: &egui::Context) -> egui::Rect {
        // Top menu bar
        if self.show_menu && !self.is_fullscreen {
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {                
                    ui.menu_button("File", |ui| {
                        ui.set_width(100.0);
                        
                        if ui.button("Load Rom").clicked() {
                            // TODO
                            if let Err(e) = self.load_rom() {
                                error!("Failed to load rom: {}", e);
                            }
                        }
                        if ui.button("Recent ROMs").clicked() {
                            warn!("Recent ROMs clicked.");
                        }
                        
                        ui.separator();
                        
                        if button_with_shortcut(ui, "Exit", "Ctrl + Q").clicked() {
                            info!("Exit button clicked, exiting");
                            
                            self.app_action = SnemulatorAppAction::Exit;
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
                                error!("Failed to toggle fullscreen: {}", e);
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
        }

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
        
        ctx.available_rect()
    }
    
    fn render(&mut self, available_rect: egui::Rect, egui_full_output: egui::FullOutput) -> Result<()> {
        let (window_width, window_height) = self.window.size();
        
        unsafe {
            self.gl.viewport(0, 0, window_width as i32, window_height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
        
        // Render snes frame buffer
        self.render_game_screen(available_rect, window_width as f32, window_height as f32);

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
    
    fn load_rom(&mut self) -> Result<()> {
        let romfile = FileDialog::new()
            .add_filter("ROM", &["sfc", "smc"])
            .set_directory("/")
            .pick_file();
        
        if let Some(romfile) = romfile {
            let file_name = romfile.to_str().unwrap();
            let data = std::fs::read(&romfile)?;
            
            self.snem_core.load_rom(data)?;
            
            info!("Loaded rom '{file_name}'");
            
            self.rom_loaded = true;
        }
        
        Ok(())
    }
    
    fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    
        if self.is_paused {
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
    
    fn toggle_fullscreen(&mut self) -> Result<()> {
        self.is_fullscreen = match self.window.fullscreen_state() {
            sdl3::video::FullscreenType::Off => true, // off -> on
            _ => false, // on -> off
        };
        
        trace!("Set fullscreen to {}", self.is_fullscreen);
        
        self.window.set_fullscreen(self.is_fullscreen)?;
        
        Ok(())
    }
}