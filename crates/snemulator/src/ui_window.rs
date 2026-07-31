use glow::HasContext;

use crate::app::theme::AppTheme;

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
        F: for<'a> FnMut(&'a mut egui::Ui),
    {   
        self.window.gl_make_current(&self.gl_context).ok();
        
        let raw_input = self.raw_input.take().unwrap_or(self.new_raw_input());
        
        let full_output = self.egui_ctx.run_ui(raw_input, ui_func);
        
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
    pub fn clear(&mut self, app_theme: &AppTheme) {
        let (width, height) = self.window.size();

        let clear_r = (app_theme.bg_primary.r() as f32) / 255.0;
        let clear_g = (app_theme.bg_primary.g() as f32) / 255.0;
        let clear_b = (app_theme.bg_primary.b() as f32) / 255.0;

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(clear_r, clear_g, clear_b, 1.0);
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
                    phase: egui::TouchPhase::Move,
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
}

impl Drop for UiWindow {
    fn drop(&mut self) {
        self.window.gl_make_current(&self.gl_context).ok();
        self.egui_painter.as_mut().unwrap().destroy();
    }
}

pub fn sdl_to_egui_mouse_button(button: sdl3::mouse::MouseButton) -> Option<egui::PointerButton> {
    match button {
        sdl3::mouse::MouseButton::Left => Some(egui::PointerButton::Primary),
        sdl3::mouse::MouseButton::Right => Some(egui::PointerButton::Secondary),
        sdl3::mouse::MouseButton::Middle => Some(egui::PointerButton::Middle),
        _ => None,
    }
}

pub fn sdl_to_egui_keycode(keycode: sdl3::keyboard::Keycode) -> Option<egui::Key> {
    match keycode {
        sdl3::keyboard::Keycode::Down => Some(egui::Key::ArrowDown),
        sdl3::keyboard::Keycode::Left => Some(egui::Key::ArrowLeft),
        sdl3::keyboard::Keycode::Right => Some(egui::Key::ArrowRight),
        sdl3::keyboard::Keycode::Up => Some(egui::Key::ArrowUp),
        
        sdl3::keyboard::Keycode::Escape => Some(egui::Key::Escape),
        sdl3::keyboard::Keycode::Tab => Some(egui::Key::Tab),
        sdl3::keyboard::Keycode::Backspace => Some(egui::Key::Backspace),
        sdl3::keyboard::Keycode::Return => Some(egui::Key::Enter),
        sdl3::keyboard::Keycode::Space => Some(egui::Key::Space),
        sdl3::keyboard::Keycode::Insert => Some(egui::Key::Insert),
        sdl3::keyboard::Keycode::Delete => Some(egui::Key::Delete),
        sdl3::keyboard::Keycode::Home => Some(egui::Key::Home),
        sdl3::keyboard::Keycode::End => Some(egui::Key::End),
        sdl3::keyboard::Keycode::PageUp => Some(egui::Key::PageUp),
        sdl3::keyboard::Keycode::PageDown => Some(egui::Key::PageDown),
        
        // sdl3::keyboard::Keycode::Copy => Some(egui::Key::Copy),
        // sdl3::keyboard::Keycode::Cut => Some(egui::Key::Cut),
        // sdl3::keyboard::Keycode::Paste => Some(egui::Key::Paste),
        // sdl3::keyboard::Keycode::Colon => Some(egui::Key::Colon),
        // sdl3::keyboard::Keycode::Comma => Some(egui::Key::Comma),
        // sdl3::keyboard::Keycode::Backslash => Some(egui::Key::Backslash),
        // sdl3::keyboard::Keycode::Slash => Some(egui::Key::Slash),
        // sdl3::keyboard::Keycode::Pipe => Some(egui::Key::Pipe),
        // sdl3::keyboard::Keycode::Question => Some(egui::Key::Questionmark),
        // sdl3::keyboard::Keycode::Exclaim => Some(egui::Key::Exclamationmark),
        // sdl3::keyboard::Keycode::LeftBracket => Some(egui::Key::OpenBracket),
        // sdl3::keyboard::Keycode::RightBracket => Some(egui::Key::CloseBracket),
        
        // sdl3::keyboard::Keycode::LeftBrace => Some(egui::Key::OpenCurlyBracket),
        // sdl3::keyboard::Keycode::RightBrace => Some(egui::Key::CloseCurlyBracket),
        // sdl3::keyboard::Keycode::Backtick => Some(egui::Key::Backtick),
        // sdl3::keyboard::Keycode::Minus => Some(egui::Key::Minus),
        // sdl3::keyboard::Keycode::Period => Some(egui::Key::Period),
        // sdl3::keyboard::Keycode::Plus => Some(egui::Key::Plus),
        // sdl3::keyboard::Keycode::Equals => Some(egui::Key::Equals),
        // sdl3::keyboard::Keycode::Semicolon => Some(egui::Key::Semicolon),
        // sdl3::keyboard::Keycode::Apostrophe => Some(egui::Key::Quote),
        // sdl3::keyboard::Keycode::Num0 => Some(egui::Key::Num0),
        // sdl3::keyboard::Keycode::Num1 => Some(egui::Key::Num1),
        // sdl3::keyboard::Keycode::Num2 => Some(egui::Key::Num2),
        // sdl3::keyboard::Keycode::Num3 => Some(egui::Key::Num3),
        // sdl3::keyboard::Keycode::Num4 => Some(egui::Key::Num4),
        // sdl3::keyboard::Keycode::Num5 => Some(egui::Key::Num5),
        // sdl3::keyboard::Keycode::Num6 => Some(egui::Key::Num6),
        // sdl3::keyboard::Keycode::Num7 => Some(egui::Key::Num7),
        // sdl3::keyboard::Keycode::Num8 => Some(egui::Key::Num8),
        // sdl3::keyboard::Keycode::Num9 => Some(egui::Key::Num9),
        // sdl3::keyboard::Keycode::A => Some(egui::Key::A),
        // sdl3::keyboard::Keycode::B => Some(egui::Key::B),
        // sdl3::keyboard::Keycode::C => Some(egui::Key::C),
        // sdl3::keyboard::Keycode::D => Some(egui::Key::D),
        // sdl3::keyboard::Keycode::E => Some(egui::Key::E),
        // sdl3::keyboard::Keycode::F => Some(egui::Key::F),
        // sdl3::keyboard::Keycode::G => Some(egui::Key::G),
        // sdl3::keyboard::Keycode::H => Some(egui::Key::H),
        // sdl3::keyboard::Keycode::I => Some(egui::Key::I),
        // sdl3::keyboard::Keycode::J => Some(egui::Key::J),
        // sdl3::keyboard::Keycode::K => Some(egui::Key::K),
        // sdl3::keyboard::Keycode::L => Some(egui::Key::L),
        // sdl3::keyboard::Keycode::M => Some(egui::Key::M),
        // sdl3::keyboard::Keycode::N => Some(egui::Key::N),
        // sdl3::keyboard::Keycode::O => Some(egui::Key::O),
        // sdl3::keyboard::Keycode::P => Some(egui::Key::P),
        // sdl3::keyboard::Keycode::Q => Some(egui::Key::Q),
        // sdl3::keyboard::Keycode::R => Some(egui::Key::R),
        // sdl3::keyboard::Keycode::S => Some(egui::Key::S),
        // sdl3::keyboard::Keycode::T => Some(egui::Key::T),
        // sdl3::keyboard::Keycode::U => Some(egui::Key::U),
        // sdl3::keyboard::Keycode::V => Some(egui::Key::V),
        // sdl3::keyboard::Keycode::W => Some(egui::Key::W),
        // sdl3::keyboard::Keycode::X => Some(egui::Key::X),
        // sdl3::keyboard::Keycode::Y => Some(egui::Key::Y),
        // sdl3::keyboard::Keycode::Z => Some(egui::Key::Z),
        
        // sdl3::keyboard::Keycode::F1 => Some(egui::Key::F1),
        // sdl3::keyboard::Keycode::F2 => Some(egui::Key::F2),
        // sdl3::keyboard::Keycode::F3 => Some(egui::Key::F3),
        // sdl3::keyboard::Keycode::F4 => Some(egui::Key::F4),
        // sdl3::keyboard::Keycode::F5 => Some(egui::Key::F5),
        // sdl3::keyboard::Keycode::F6 => Some(egui::Key::F6),
        // sdl3::keyboard::Keycode::F7 => Some(egui::Key::F7),
        // sdl3::keyboard::Keycode::F8 => Some(egui::Key::F8),
        // sdl3::keyboard::Keycode::F9 => Some(egui::Key::F9),
        // sdl3::keyboard::Keycode::F10 => Some(egui::Key::F10),
        // sdl3::keyboard::Keycode::F11 => Some(egui::Key::F11),
        // sdl3::keyboard::Keycode::F12 => Some(egui::Key::F12),
        // sdl3::keyboard::Keycode::F13 => Some(egui::Key::F13),
        // sdl3::keyboard::Keycode::F14 => Some(egui::Key::F14),
        // sdl3::keyboard::Keycode::F15 => Some(egui::Key::F15),
        // sdl3::keyboard::Keycode::F16 => Some(egui::Key::F16),
        // sdl3::keyboard::Keycode::F17 => Some(egui::Key::F17),
        // sdl3::keyboard::Keycode::F18 => Some(egui::Key::F18),
        // sdl3::keyboard::Keycode::F19 => Some(egui::Key::F19),
        // sdl3::keyboard::Keycode::F20 => Some(egui::Key::F20),
        // sdl3::keyboard::Keycode::F21 => Some(egui::Key::F21),
        // sdl3::keyboard::Keycode::F22 => Some(egui::Key::F22),
        // sdl3::keyboard::Keycode::F23 => Some(egui::Key::F23),
        // sdl3::keyboard::Keycode::F24 => Some(egui::Key::F24),
        // sdl3::keyboard::Keycode::F25 => Some(egui::Key::F25),
        // sdl3::keyboard::Keycode::F26 => Some(egui::Key::F26),
        // sdl3::keyboard::Keycode::F27 => Some(egui::Key::F27),
        // sdl3::keyboard::Keycode::F28 => Some(egui::Key::F28),
        // sdl3::keyboard::Keycode::F29 => Some(egui::Key::F29),
        // sdl3::keyboard::Keycode::F30 => Some(egui::Key::F30),
        // sdl3::keyboard::Keycode::F31 => Some(egui::Key::F31),
        // sdl3::keyboard::Keycode::F32 => Some(egui::Key::F32),
        // sdl3::keyboard::Keycode::F33 => Some(egui::Key::F33),
        // sdl3::keyboard::Keycode::F34 => Some(egui::Key::F34),
        // sdl3::keyboard::Keycode::F35 => Some(egui::Key::F35),
        
        _ => None,
    }
}

pub fn sdl_to_egui_modifiers(keymod: sdl3::keyboard::Mod) -> egui::Modifiers {
    egui::Modifiers {
        alt: keymod.contains(sdl3::keyboard::Mod::LALTMOD | sdl3::keyboard::Mod::RALTMOD),
        ctrl: keymod.contains(sdl3::keyboard::Mod::LCTRLMOD | sdl3::keyboard::Mod::RCTRLMOD),
        shift: keymod.contains(sdl3::keyboard::Mod::LSHIFTMOD | sdl3::keyboard::Mod::RSHIFTMOD),
        mac_cmd: keymod.contains(sdl3::keyboard::Mod::LGUIMOD | sdl3::keyboard::Mod::RGUIMOD),
        command: false,
    }
}