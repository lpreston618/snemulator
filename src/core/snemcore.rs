use crate::core::controller::{ControllerPlayer, JoypadButton, SnemController};
use crate::core::sysinfo::{SCREEN_HEIGHT, SCREEN_WIDTH};

// Emulator core
pub struct Snemulator {
    p1_controller: SnemController,
    p2_controller: SnemController,
    // Add your emulator state here (CPU, memory, etc.)
}

impl Snemulator {
    pub fn new() -> Self {
        Self {
            p1_controller: SnemController::new(),
            p2_controller: SnemController::new(),
        }
    }

    pub fn set_button(&mut self, player: ControllerPlayer, button: JoypadButton, pressed: bool) {
        println!("{player:?} button {button:?} pressed = {pressed}");
        
        match player {
            ControllerPlayer::Player1 => self.p1_controller.set_button(button, pressed),
            ControllerPlayer::Player2 => self.p2_controller.set_button(button, pressed),
        }
    }

    pub fn run_frame(&mut self, frame_buffer: &mut [u8]) {
        // TODO: Implement your emulator logic here
        // This should:
        // 1. Execute CPU instructions until a frame is complete
        // 2. Update the frame_buffer with pixel data (RGBA format)
        
        // Example: Fill with a test pattern
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let offset = ((y * SCREEN_WIDTH + x) * 4) as usize;
                frame_buffer[offset] = (x & 0xFF) as u8;     // R
                frame_buffer[offset + 1] = (y & 0xFF) as u8; // G
                frame_buffer[offset + 2] = 128;              // B
                frame_buffer[offset + 3] = 255;              // A
            }
        }
    }
}