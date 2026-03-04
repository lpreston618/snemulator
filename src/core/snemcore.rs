use anyhow::Result;
use crate::core::controller::{ControllerPlayer, JoypadButton, SnemController};
use crate::core::sysinfo::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH,
    WRAM_SIZE,
};
use log::{debug, error, info, trace, warn};

// Emulator core
pub struct Snemulator {
    p1_controller: SnemController,
    p2_controller: SnemController,
    
    // cpu: scpu::Cpu65c816,
    // ppu: sppu::Ppu5C7x,
    // ssmp: ssmp::Ssmp,
    
    wram: Box<[u8; WRAM_SIZE]>,
    rom: Vec<u8>,
}

impl Snemulator {
    pub fn new() -> Self {
        Self {
            p1_controller: SnemController::new(),
            p2_controller: SnemController::new(),
            // cpu: scpu::Cpu65c816::new(),
            // ppu: sppu::Ppu5C7x::new(),
            // ssmp: ssmp::Ssmp::new(),
            wram: Box::new([0u8; WRAM_SIZE]),
            rom: Vec::new(),
        }
    }
    
    pub fn load_rom(&mut self, data: Vec<u8>) -> Result<()> {
        
        Ok(())
    }

    pub fn set_button(&mut self, player: ControllerPlayer, button: JoypadButton, pressed: bool) {
        trace!("{player:?} button {button:?} pressed = {pressed}");
        
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
        
        while !self.cycle() {}
        
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
    
    fn cycle(&mut self) -> bool {
        true
    }
}