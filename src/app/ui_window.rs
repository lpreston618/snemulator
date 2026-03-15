use anyhow::{Result};
use glow::HasContext;

use crate::app::utils::sdl_to_egui_mouse_button;

// Generic egui window wrapper
pub struct UiWindow {
    window: sdl3::video::Window,
    raw_input: Option<egui::RawInput>,
    egui_ctx: egui::Context,
    egui_painter: egui_glow::Painter,
    gl: std::sync::Arc<glow::Context>,
    gl_context: sdl3::video::GLContext,
    ui_scale: f32,
}

impl UiWindow {
    pub fn new(
        video_subsystem: &sdl3::VideoSubsystem,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        
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
        let egui_painter = egui_glow::Painter::new(gl.clone(), "", None, false)?;

        let ui_scale = window.display_scale();
        
        egui_ctx.set_pixels_per_point(ui_scale);
        
        Ok(Self {
            window,
            raw_input: None,
            egui_ctx,
            egui_painter,
            gl,
            gl_context,
            ui_scale,
        })
    }
    
    /// Updates the UI with the given function and returns the full output to be used during rendering.
    pub fn update_ui<F>(&mut self, ui_func: F) -> egui::FullOutput
    where
        F: FnMut(&egui::Context),
    {   
        self.window.gl_make_current(&self.gl_context).ok();
        
        let raw_input = self.raw_input.take().unwrap_or(self.new_raw_input());
        
        self.egui_ctx.run(raw_input, ui_func)
    }
    
    /// Clears the screen with the default background color. Should be called before rendering.
    pub fn clear(&mut self) {
        let (width, height) = self.window.size();

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.2, 0.2, 0.2, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }
    
    /// Renders the given `egui::FullOutput` to the window.
    pub fn render(&mut self, full_output: egui::FullOutput) {
        let (width, height) = self.window.size();
        
        let clipped = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        self.egui_painter.paint_and_update_textures(
            [width, height],
            full_output.pixels_per_point,
            &clipped,
            &full_output.textures_delta,
        );

        self.window.gl_swap_window();
    }
    
    /// Adds any sdl mouse events to the egui raw input. Returns a bool if the event was handled.
    pub fn handle_sdl_mouse_event(&mut self, event: &sdl3::event::Event) -> bool {
        let mut new_event = None;
        
        match event {
            sdl3::event::Event::MouseMotion { x, y, .. } => {
                let logical_x = *x as f32 / self.ui_scale;
                let logical_y = *y as f32 / self.ui_scale;
                new_event = Some(egui::Event::PointerMoved(egui::Pos2::new(logical_x, logical_y)));
            }
            sdl3::event::Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
                    let logical_x = *x as f32 / self.ui_scale;
                    let logical_y = *y as f32 / self.ui_scale;
                    new_event = Some(egui::Event::PointerButton {
                        pos: egui::Pos2::new(logical_x, logical_y),
                        button,
                        pressed: true,
                        modifiers: Default::default(),
                    });
                }
            }
            sdl3::event::Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                if let Some(button) = sdl_to_egui_mouse_button(*mouse_btn) {
                    let logical_x = *x as f32 / self.ui_scale;
                    let logical_y = *y as f32 / self.ui_scale;
                    new_event = Some(egui::Event::PointerButton {
                        pos: egui::Pos2::new(logical_x, logical_y),
                        button,
                        pressed: false,
                        modifiers: Default::default(),
                    });
                }
            }
            _ => {}
        }
        
        if let Some(event) = new_event {
            if self.raw_input.is_none() {
                self.raw_input = Some(self.new_raw_input());
            }
            
            let raw_input = self.raw_input.as_mut().unwrap();
            raw_input.events.push(event);
            
            return true;
        }
        
        false
    }
    
    fn new_raw_input(&mut self) -> egui::RawInput {
        let (width, height) = self.window.size();
        let scaled_width = width as f32 / self.ui_scale;
        let scaled_height = height as f32 / self.ui_scale;
        
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(scaled_width, scaled_height)
            )),
            ..Default::default()
        }
    }
    
    pub fn window(&self) -> &sdl3::video::Window {
        &self.window
    }
    
    pub fn window_mut(&mut self) -> &mut sdl3::video::Window {
        &mut self.window
    }
    
    pub fn gl(&self) -> &glow::Context {
        &self.gl
    }
    
    pub fn ui_scale(&self) -> f32 {
        self.ui_scale
    }
}

impl Drop for UiWindow {
    fn drop(&mut self) {
        self.window.gl_make_current(&self.gl_context).ok();
        self.egui_painter.destroy();
    }
}