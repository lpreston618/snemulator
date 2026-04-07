use glow::HasContext;
use anyhow::Result;
use sdl3::video::GLProfile;

use crate::app;
use snemcore::sysinfo;
use crate::windows::menu::MainMenuBar;
use crate::windows::settings::Settings;
use common::UiWindow;

pub struct MainWindow {
    egui_window: UiWindow,
    menu: MainMenuBar,
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
        let egui_window = UiWindow::new(
            video_subsystem, 
            "Snemulator", 
            app::WINDOW_WIDTH, 
            app::WINDOW_HEIGHT
        )?;
        
        video_subsystem.gl_set_swap_interval(
            if settings.vsync_en {
                sdl3::video::SwapInterval::VSync
            } else {
                sdl3::video::SwapInterval::Immediate
            }
        )?;
        
        let menu = MainMenuBar::new();
        
        // Create OpenGL texture for game screen
        let gl = egui_window.gl();
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
                sysinfo::SCREEN_WIDTH as i32,
                sysinfo::SCREEN_HEIGHT as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            Some(texture)
        };
        
        let (shader_program, vao, vbo) = Self::create_shader_program(&gl)?;
        
        Ok(Self {
            egui_window,
            menu,
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
    
    fn update_game_texture(&self, frame_buffer: &[u8]) {
        let gl = self.egui_window.gl();
        
        unsafe {
            if let Some(texture) = self.game_texture {
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    0,
                    sysinfo::SCREEN_WIDTH as i32,
                    sysinfo::SCREEN_HEIGHT as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(frame_buffer)),
                );
            }
        }
    }

    fn render_game_screen(&self, available_rect: egui::Rect) {
        let (window_width, window_height) = self.egui_window.window().size();
        let window_width = window_width as f32;
        let window_height = window_height as f32;
                
        unsafe {
            if let Some(texture) = self.game_texture {                
                let game_aspect = sysinfo::SCREEN_WIDTH as f32 / sysinfo::SCREEN_HEIGHT as f32;
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
                
                let gl = self.egui_window.gl();
                
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    std::slice::from_raw_parts(
                        vertices.as_ptr() as *const u8,
                        vertices.len() * std::mem::size_of::<f32>(),
                    ),
                    glow::STATIC_DRAW,
                );
                
                gl.use_program(Some(self.shader_program));
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                gl.bind_vertex_array(Some(self.vao));
                gl.draw_arrays(glow::TRIANGLES, 0, 6);
                gl.bind_vertex_array(None);
            }
        }
    }
    
    pub fn update_and_render(&mut self, app_state: &app::AppState, app_settings: &Settings, frame_buffer: &[u8]) -> app::AppAction {
        let mut app_action = app::AppAction::Continue;
        let mut game_rect = egui::Rect::NOTHING;
        let ui_scale = self.egui_window.ui_scale();
        
        let full_output = self.egui_window.update_ui(|ctx| {
            if app_state.show_menu {
                app_action = self.menu.render(ctx, app_state);
            }
            
            if app_settings.always_show_menu {
                game_rect = ctx.available_rect();
            } else {
                game_rect = ctx.viewport_rect();
            }
            
            game_rect = game_rect * ui_scale;
        });
        
        self.egui_window.clear();
        
        self.update_game_texture(frame_buffer);
        self.render_game_screen(game_rect);
        
        self.egui_window.render(full_output);
        
        app_action
    }
    
    pub fn id(&self) -> u32 {
        self.egui_window.window().id()
    }
    
    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers, app_state: &mut app::AppState) {        
        if self.egui_window.handle_sdl_mouse_event(event, modifiers) {
            app_state.last_mouse_input_frame = app_state.frame_count;
        }
    }
    
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        self.egui_window.window_mut().set_fullscreen(fullscreen).map_err(|e| e.into())
    }
}