use anyhow::{Result};
use glow::HasContext;

// Generic egui window wrapper
pub struct UiWindow {
    pub window: sdl3::video::Window,
    egui_ctx: egui::Context,
    egui_painter: egui_glow::Painter,
    gl: std::sync::Arc<glow::Context>,
    gl_context: sdl3::video::GLContext,
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

        Ok(Self {
            window,
            egui_ctx,
            egui_painter,
            gl,
            gl_context
        })
    }

    pub fn render<F>(&mut self, raw_input: Option<egui::RawInput>, ui_func: F)
    where
        F: FnMut(&egui::Context),
    {   
        self.window.gl_make_current(&self.gl_context).ok();
        
        let (width, height) = self.window.size();
        let raw_input = raw_input.unwrap_or(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width as f32, height as f32)
                )),
                ..Default::default()
            }
        );

        let full_output = self.egui_ctx.run(raw_input, ui_func);

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.2, 0.2, 0.2, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        let clipped = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        self.egui_painter.paint_and_update_textures(
            [width, height],
            full_output.pixels_per_point,
            &clipped,
            &full_output.textures_delta,
        );

        self.window.gl_swap_window();
    }
}

impl Drop for UiWindow {
    fn drop(&mut self) {
        self.window.gl_make_current(&self.gl_context).ok();
        self.egui_painter.destroy();
    }
}