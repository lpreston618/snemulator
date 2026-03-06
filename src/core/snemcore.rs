use anyhow::Result;
use crate::core::controller::{ControllerPlayer, JoypadButton, SnemController};
use crate::core::scpu::{Cpu65c816, CpuInterrupt};
use crate::core::scpu::bus::CpuBus;
use crate::core::sppu::Ppu5C7x;
use crate::core::sppu::bus::{PpuBus, FRAME_BUF_SIZE};
use crate::core::sysinfo::{
    CGRAM_SIZE, OAM_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, VRAM_SIZE, WRAM_SIZE
};
use crate::core::sppu::color::Color;
use crate::core::sppu::regs::PpuRegs;
use log::{debug, error, info, trace, warn};

// Emulator core
pub struct Snemulator {
    p1_controller: SnemController,
    p2_controller: SnemController,
    
    cpu: Cpu65c816,
    ppu: Ppu5C7x,
    // ssmp: ssmp::Ssmp,
    
    wram: Box<[u8; WRAM_SIZE]>,
    vram: Box<[u16; VRAM_SIZE]>,
    cgram: Box<[Color; CGRAM_SIZE]>,
    oam: Box<[u8; OAM_SIZE]>,
    ppu_regs: PpuRegs,
    
    frame_ready: bool,
    
    rom: Vec<u8>,
}

impl Snemulator {
    pub fn new() -> Self {
        Self {
            p1_controller: SnemController::new(),
            p2_controller: SnemController::new(),
            cpu: Cpu65c816::new(),
            ppu: Ppu5C7x::new(),
            // ssmp: ssmp::Ssmp::new(),
            wram: Box::new([0u8; WRAM_SIZE]),
            vram: Box::new([0u16; VRAM_SIZE]),
            cgram: Box::new([Color::default(); CGRAM_SIZE]),
            oam: Box::new([0u8; OAM_SIZE]),
            ppu_regs: PpuRegs::default(),
            frame_ready: false,
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
        
        self.frame_ready = false;
        
        while !self.frame_ready {
            self.cycle(frame_buffer);
        }
        
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
    
    fn cycle(&mut self, frame_buffer: &mut [u8]) {
        let clocks = self.cpu.clocks.min(self.ppu.clocks);
        
        self.cpu.clocks -= clocks;
        self.ppu.clocks -= clocks;
        
        if self.cpu.clocks == 0 {
            let mut bus = CpuBus {
                wram: &mut self.wram,
                vram: &mut self.vram,
                cgram: &mut self.cgram,
                oam: &mut self.oam,
                ppu_regs: &mut self.ppu_regs,
            };
            
            self.cpu.cycle(&mut bus);
        }
        
        if self.ppu.clocks == 0 {
            let mut interrupt: Option<CpuInterrupt> = None;
            
            let mut bus = PpuBus {
                vram: &mut self.vram,
                cgram: &mut self.cgram,
                oam: &mut self.oam,
                ppu_regs: &mut self.ppu_regs,
                frame_buffer,
                frame_ready: &mut self.frame_ready,
                interrupt: &mut interrupt,
            };
            
            self.ppu.cycle(&mut bus);
            
            match interrupt {
                Some(CpuInterrupt::IRQ) => { self.cpu.irq_pending = true; }
                Some(CpuInterrupt::NMI) => { self.cpu.nmi_pending = true; }
                _ => {}
            }
        }
    }
}