use anyhow::Result;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::PixelFormat;
use sdl3::sys::render::SDL_LOGICAL_PRESENTATION_LETTERBOX;
use std::time::{Duration, Instant};
use crate::core::sysinfo::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::core::snemcore::{Button, Snemulator};

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const TARGET_FPS: u32 = 60;
const FRAME_BUF_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize;

pub struct EmulatorApp {
    sdl_context: sdl3::Sdl,
    canvas: sdl3::render::Canvas<sdl3::video::Window>,
    snem_core: Snemulator,
    frame_buffer: Vec<u8>,
    // menu_bar: MenuBar,
    // show_menu: bool,
}

impl EmulatorApp {
    pub fn new() -> Result<Self> {
        // Initialize SDL3
        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;

        // Create window
        let window = video_subsystem
            .window("Snemulator", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .resizable()
            .build()?;

        // Create canvas
        let mut canvas = window.into_canvas();
        canvas.set_logical_size(
            SCREEN_WIDTH, 
            SCREEN_HEIGHT, 
            SDL_LOGICAL_PRESENTATION_LETTERBOX)
        .map_err(|e| anyhow::anyhow!(e))?;

        Ok(Self {
            sdl_context,
            canvas,
            snem_core: Snemulator::new(),
            frame_buffer: vec![0; FRAME_BUF_SIZE],
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let frame_duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);
        let texture_creator = self.canvas.texture_creator();
        
        let mut texture = texture_creator.create_texture_streaming(
            PixelFormat::RGBA32,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )?;

        'running: loop {
            let frame_start = Instant::now();
            
            let mut event_pump = self.sdl_context.event_pump()
                .expect("Failed to get event pump");

            // Handle input
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => self.handle_keydown(keycode),
                    
                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => self.handle_keyup(keycode),
                    
                    _ => {}
                }
            }

            // Emulate one frame
            self.snem_core.run_frame(&mut self.frame_buffer);

            // Render
            self.render(&mut texture)?;

            // Frame timing
            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }

        Ok(())
    }

    fn handle_keydown(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Up => self.snem_core.set_button(Button::Up, true),
            Keycode::Down => self.snem_core.set_button(Button::Down, true),
            Keycode::Left => self.snem_core.set_button(Button::Left, true),
            Keycode::Right => self.snem_core.set_button(Button::Right, true),
            Keycode::Z => self.snem_core.set_button(Button::A, true),
            Keycode::X => self.snem_core.set_button(Button::B, true),
            Keycode::Return => self.snem_core.set_button(Button::Start, true),
            Keycode::Backspace => self.snem_core.set_button(Button::Select, true),
            _ => {}
        }
    }

    fn handle_keyup(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Up => self.snem_core.set_button(Button::Up, false),
            Keycode::Down => self.snem_core.set_button(Button::Down, false),
            Keycode::Left => self.snem_core.set_button(Button::Left, false),
            Keycode::Right => self.snem_core.set_button(Button::Right, false),
            Keycode::Z => self.snem_core.set_button(Button::A, false),
            Keycode::X => self.snem_core.set_button(Button::B, false),
            Keycode::Return => self.snem_core.set_button(Button::Start, false),
            Keycode::Backspace => self.snem_core.set_button(Button::Select, false),
            _ => {}
        }
    }

    fn render(&mut self, texture: &mut sdl3::render::Texture) -> Result<()> {
        // Update texture with frame buffer
        texture.update(None, &self.frame_buffer, (SCREEN_WIDTH * 4) as usize)?;

        // Clear and render
        self.canvas.clear();
        self.canvas.copy(texture, None, None)?;
        self.canvas.present();

        Ok(())
    }
}