use glow::HasContext;

use crate::{app_utils::{sdl_to_egui_keycode, sdl_to_egui_modifiers, sdl_to_egui_mouse_button}};

// Generic egui window wrapper
pub struct UiWindow {
    pub window: sdl3::video::Window,
    pub raw_input: Option<egui::RawInput>,
    pub text_input: sdl3::keyboard::TextInputUtil,
    pub egui_ctx: egui::Context,
    pub egui_painter: Option<egui_glow::Painter>,
    pub gl: std::sync::Arc<glow::Context>,
    pub gl_context: sdl3::video::GLContext,
    pub ui_scale: f32,
}

impl UiWindow {
    /// Updates the UI with the given function and returns the full output to be used during rendering.
    pub fn update_ui<F>(&mut self, ui_func: F) -> egui::FullOutput
    where
        F: FnMut(&egui::Context),
    {   
        self.window.gl_make_current(&self.gl_context).ok();
        
        let raw_input = self.raw_input.take().unwrap_or(self.new_raw_input());
        
        let full_output = self.egui_ctx.run(raw_input, ui_func);
        
        let wants_text = full_output.platform_output.ime.is_some()
            || self.egui_ctx.memory(|m| m.focused().is_some());
        
        if wants_text {
            self.text_input.start(&self.window);
        } else {
            self.text_input.stop(&self.window);
        }
        
        full_output
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
        self.egui_painter.as_mut().unwrap().paint_and_update_textures(
            [width, height],
            full_output.pixels_per_point,
            &clipped,
            &full_output.textures_delta,
        );

        self.window.gl_swap_window();
    }
    
    /// Adds any sdl mouse events to the egui raw input. Returns a bool if the event was handled.
    pub fn handle_sdl_mouse_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers) -> bool {
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
                        modifiers: egui::Modifiers::default(),
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
                        modifiers: egui::Modifiers::default(),
                    });
                }
            }
            sdl3::event::Event::MouseWheel { y, .. } => {
                new_event = Some(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: egui::Vec2::new(0.0, *y as f32),
                    modifiers: egui::Modifiers::default(),
                });
            }
            _ => {}
        }
        
        if let Some(event) = new_event {
            if self.raw_input.is_none() {
                self.raw_input = Some(self.new_raw_input());
            }
            
            let raw_input = self.raw_input.as_mut().unwrap();
            raw_input.events.push(event);
            raw_input.modifiers = *modifiers;
            
            return true;
        }
        
        false
    }
    
    /// Adds any sdl keyboard events to the egui raw input. Returns a bool if the event was handled.
    pub fn handle_sdl_keyboard_event(&mut self, event: &sdl3::event::Event) -> bool {
        let mut new_event = None;
        
        match event {
            sdl3::event::Event::TextInput { text, .. } => {
                new_event = Some(egui::Event::Text(text.clone()));
            }
            sdl3::event::Event::KeyDown { keycode, keymod, repeat, .. } => {
                if let Some(keycode) = keycode {
                    if let Some(key) = sdl_to_egui_keycode(*keycode) {
                        new_event = Some(egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers: sdl_to_egui_modifiers(*keymod),
                            repeat: *repeat,
                            physical_key: None,
                        });
                    }
                }
            }
            sdl3::event::Event::KeyUp { keycode, keymod, repeat, .. } => {
                if let Some(keycode) = keycode {
                    if let Some(key) = sdl_to_egui_keycode(*keycode) {
                        new_event = Some(egui::Event::Key {
                            key,
                            pressed: false,
                            modifiers: sdl_to_egui_modifiers(*keymod),
                            repeat: *repeat,
                            physical_key: None,
                        });
                    }
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
        self.egui_painter.as_mut().unwrap().destroy();
    }
}