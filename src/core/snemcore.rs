use crate::core::sysinfo::{SCREEN_HEIGHT, SCREEN_WIDTH};

// Emulator core
pub struct Snemulator {
    buttons: [bool; 8],
    // Add your emulator state here (CPU, memory, etc.)
}

#[derive(Debug, Clone, Copy)]
pub enum Button {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    A = 4,
    B = 5,
    Start = 6,
    Select = 7,
}

impl Snemulator {
    pub fn new() -> Self {
        Self {
            buttons: [false; 8],
        }
    }

    pub fn set_button(&mut self, button: Button, pressed: bool) {
        self.buttons[button as usize] = pressed;
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