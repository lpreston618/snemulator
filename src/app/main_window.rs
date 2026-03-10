use glow::HasContext;
use anyhow::Result;
use log::info;
use sdl3::video::GLProfile;

use crate::{app::{self, AppState, AppAction, WINDOW_HEIGHT, WINDOW_WIDTH, settings::Settings}, core::sysinfo::{SCREEN_HEIGHT, SCREEN_WIDTH}};

fn sdl_to_egui_mouse_button(button: sdl3::mouse::MouseButton) -> Option<egui::PointerButton> {
    match button {
        sdl3::mouse::MouseButton::Left => Some(egui::PointerButton::Primary),
        sdl3::mouse::MouseButton::Right => Some(egui::PointerButton::Secondary),
        sdl3::mouse::MouseButton::Middle => Some(egui::PointerButton::Middle),
        _ => None,
    }
}

pub struct MainWindow {
    pub window: sdl3::video::Window,
    menu: app::menu::MainMenuBar,
    gl: std::sync::Arc<glow::Context>,
    gl_context: std::rc::Rc<sdl3::video::GLContext>,
    shader_program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    game_texture: Option<glow::Texture>,
}

impl MainWindow {
    pub fn new(
        video_subsystem: &sdl3::VideoSubsystem,
        settings: &Settings) -> Result<Self> {
            
        // Set OpenGL attributes
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_context_flags().forward_compatible().set();
        gl_attr.set_double_buffer(true);
        
        // Create window
        let mut window = video_subsystem
            .window("Snemulator", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .resizable()
            .opengl()
            .build()?;
    
        let window_width = (WINDOW_WIDTH as f32 * window.display_scale()) as u32;
        let window_height = (WINDOW_HEIGHT as f32 * window.display_scale()) as u32;
        
        window.set_minimum_size(SCREEN_WIDTH, SCREEN_HEIGHT)?;
        window.set_size(window_width, window_height)?;
        window.set_position(
            sdl3::video::WindowPos::Centered,
            sdl3::video::WindowPos::Centered
        );
        let window = window; // No longer mutable
        
        // Create the shared GL context
        let gl_context = window.gl_create_context()?;
        let gl_context = std::rc::Rc::new(gl_context);
        
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                match video_subsystem.gl_get_proc_address(s) {
                    Some(ptr) => ptr as *const _,
                    None => return std::ptr::null(),
                }
            })
        };
        let gl = std::sync::Arc::new(gl);
        
        window.gl_make_current(gl_context.as_ref())?;
        
        video_subsystem.gl_set_swap_interval(
            if settings.vsync_en {
                sdl3::video::SwapInterval::VSync
            } else {
                sdl3::video::SwapInterval::Immediate
            }
        )?;
        
        let menu = app::menu::MainMenuBar::new(
            window.display_scale(),
            gl.clone()
        )?;
        
        info!("Main window scale: {}", window.display_scale());
        
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
                glow::PixelUnpackData::Slice(None),
            );
            Some(texture)
        };
        
        let (shader_program, vao, vbo) = Self::create_shader_program(&gl)?;
        
        Ok(Self {
            window,
            menu,
            gl,
            gl_context,
            shader_program,
            vao,
            vbo,
            game_texture,
        })
    }

    fn create_shader_program(gl: &glow::Context) -> Result<(glow::Program, glow::VertexArray, glow::Buffer)> {
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
            
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).map_err(|e| anyhow::anyhow!(e))?;
            gl.shader_source(vertex_shader, vertex_shader_source);
            gl.compile_shader(vertex_shader);
            
            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).map_err(|e| anyhow::anyhow!(e))?;
            gl.shader_source(fragment_shader, fragment_shader_source);
            gl.compile_shader(fragment_shader);
            
            let shader_program = gl.create_program().map_err(|e| anyhow::anyhow!(e))?;
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
            
            let vao = gl.create_vertex_array().map_err(|e| anyhow::anyhow!(e))?;
            let vbo = gl.create_buffer().map_err(|e| anyhow::anyhow!(e))?;
            
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
    
    pub fn handle_sdl_event(&mut self, event: &sdl3::event::Event, raw_input: &mut egui::RawInput, app_state: &mut AppState) {        
        match event {
            sdl3::event::Event::MouseMotion { x, y, .. } => {
                let logical_x = *x as f32 / self.menu.ui_scale;
                let logical_y = *y as f32 / self.menu.ui_scale;
                raw_input.events.push(egui::Event::PointerMoved(egui::Pos2::new(logical_x, logical_y)));
                app_state.last_mouse_input_frame = app_state.frame_count;
            }
            sdl3::event::Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
                    let logical_x = *x as f32 / self.menu.ui_scale;
                    let logical_y = *y as f32 / self.menu.ui_scale;
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::Pos2::new(logical_x, logical_y),
                        button,
                        pressed: true,
                        modifiers: Default::default(),
                    });
                }
                app_state.last_mouse_input_frame = app_state.frame_count;
            }
            sdl3::event::Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
                    let logical_x = *x as f32 / self.menu.ui_scale;
                    let logical_y = *y as f32 / self.menu.ui_scale;
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::Pos2::new(logical_x, logical_y),
                        button,
                        pressed: false,
                        modifiers: Default::default(),
                    });
                }
                app_state.last_mouse_input_frame = app_state.frame_count;
            }
            _ => {}
        }
    }
    
    fn update_game_texture(&self, frame_buffer: &[u8]) {
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
                    glow::PixelUnpackData::Slice(Some(frame_buffer)),
                );
            }
        }
    }

    fn render_game_screen(&self, available_rect: egui::Rect) {
        let (window_width, window_height) = self.window.size();
        let window_width = window_width as f32;
        let window_height = window_height as f32;
                
        unsafe {
            if let Some(texture) = self.game_texture {
                let display_scale = self.window.display_scale();
                
                // Only scale top y because the menu bar is the only thing we are accouting for
                let available_rect = egui::Rect::from_min_max(
                    egui::pos2(
                        available_rect.min.x,
                        available_rect.min.y * display_scale
                    ),
                    egui::pos2(
                        available_rect.max.x,
                        available_rect.max.y
                    )
                );
                            
                let game_aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
                let available_width = available_rect.width();
                let available_height = available_rect.height();
                let available_aspect = available_width / available_height;
                
                let (render_width, render_height) = if available_aspect > game_aspect {
                    let h = available_height;
                    let w = h * game_aspect;
                    (w, h)
                } else {
                    let w = available_width;
                    let h = w / game_aspect;
                    (w, h)
                };
                
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
    
    pub fn render(&mut self, app_state: &AppState, raw_input: egui::RawInput, frame_buffer: &[u8]) -> Result<AppAction> {
        self.window.gl_make_current(self.gl_context.as_ref()).ok();
        
        let (window_width, window_height) = self.window.size();

        // Update game texture
        self.update_game_texture(frame_buffer);

        // Run egui
        let mut app_action = AppAction::Continue;
        let mut game_rect = egui::Rect::NOTHING;
        let full_output = self.menu.egui_context.run(raw_input, |ctx| {
            if app_state.show_menu {
                app_action = self.menu.render(app_state);
            }
            
            game_rect = ctx.available_rect();
        });

        // Render
        unsafe {
            self.gl.viewport(0, 0, window_width as i32, window_height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
        
        // Render game screen
        self.render_game_screen(game_rect);

        // Render egui
        let clipped_primitives = self.menu.egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);
        
        self.menu.egui_painter.paint_and_update_textures(
            [window_width, window_height],
            full_output.pixels_per_point,
            &clipped_primitives,
            &full_output.textures_delta,
        );

        self.window.gl_swap_window();

        Ok(app_action)
    }
    
    pub fn gl(&self) -> std::sync::Arc<glow::Context> {
        self.gl.clone()
    }
    
    pub fn gl_context(&self) -> std::rc::Rc<sdl3::video::GLContext> {
        self.gl_context.clone()
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        self.menu.egui_painter.destroy();
    }
}