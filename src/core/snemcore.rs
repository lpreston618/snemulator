use anyhow::{Result, anyhow};
use crate::core::cartridge::Cartridge;
use crate::core::controller::{ControllerPlayer, JoypadButton, JoypadCmd, SnemController};
use crate::core::scpu::dma::DmaRegs;
use crate::core::scpu::ioregs::CpuIoRegs;
use crate::core::scpu::mult::Mult5A22;
use crate::core::scpu::{Cpu65c816, CpuInterrupt};
use crate::core::scpu::bus::CpuBus;
use crate::core::sppu::Ppu5C7x;
use crate::core::sppu::bus::PpuBus;
use crate::core::ssmp::Ssmp;
use crate::core::ssmp::ioports::ApuIoPorts;
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
    ssmp: Ssmp,
    
    wram: Box<[u8; WRAM_SIZE]>,
    vram: Box<[u16; VRAM_SIZE]>,
    cgram: Box<[Color; CGRAM_SIZE]>,
    oam: Box<[u8; OAM_SIZE]>,
    ppu_regs: PpuRegs,
    cpu_regs: CpuIoRegs,
    apu_ports: ApuIoPorts,
    
    mult: Mult5A22,
    dma_regs: [DmaRegs; 8],
    
    joy1_latch: u16,
    joy2_latch: u16,
    joy1_data1_auto: u16,
    joy2_data1_auto: u16,
    joy1_data2_auto: u16,
    joy2_data2_auto: u16,
    
    frame_ready: bool,
    
    cart: Option<Cartridge>,
    
    total_clocks: u64,
}

impl Snemulator {
    pub fn new() -> Self {
        Self {
            p1_controller: SnemController::new(),
            p2_controller: SnemController::new(),
            cpu: Cpu65c816::new(),
            ppu: Ppu5C7x::new(),
            ssmp: Ssmp::new(),
            wram: Box::new([0u8; WRAM_SIZE]),
            vram: Box::new([0u16; VRAM_SIZE]),
            cgram: Box::new([Color::default(); CGRAM_SIZE]),
            oam: Box::new([0u8; OAM_SIZE]),
            ppu_regs: PpuRegs::default(),
            cpu_regs: CpuIoRegs::default(),
            apu_ports: ApuIoPorts::default(),
            mult: Mult5A22::new(),
            dma_regs: [DmaRegs::default(); 8],
            joy1_latch: 0,
            joy2_latch: 0,
            joy1_data1_auto: 0,
            joy2_data1_auto: 0,
            joy1_data2_auto: 0,
            joy2_data2_auto: 0,
            frame_ready: false,
            cart: None,
            total_clocks: 0,
        }
    }
    
    pub fn load_rom(&mut self, data: Vec<u8>) -> Result<()> {
        match Cartridge::from_rom(data) {
            Ok(cart) => {
                self.cart = Some(cart);
                Ok(())
            },
            Err(err) => {
                error!("Failed to load ROM: {}", err);
                Err(anyhow!(err))
            },
        }
    }

    pub fn set_button(&mut self, player: ControllerPlayer, button: JoypadButton, pressed: bool) {
        trace!("{player:?} button {button:?} pressed = {pressed}");
        
        match player {
            ControllerPlayer::Player1 => self.p1_controller.set_button(button, pressed),
            ControllerPlayer::Player2 => self.p2_controller.set_button(button, pressed),
        }
    }

    pub fn run_frame(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>) {
        // TODO: Implement your emulator logic here
        // This should:
        // 1. Execute CPU instructions until a frame is complete
        // 2. Update the frame_buffer with pixel data (RGBA format)
        
        self.frame_ready = false;
        
        while !self.frame_ready {
            self.cycle(frame_buffer, audio_buffer);
        }
    }
    
    fn cycle(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>) {
        let clocks = self.cpu.clocks.min(self.ppu.clocks);
        
        self.cpu.clocks -= clocks;
        self.ppu.clocks -= clocks;
        self.total_clocks += clocks as u64;
        
        if self.cpu.clocks == 0 {
            let mut joypad_cmd: Option<JoypadCmd> = None;
            
            let mut bus = CpuBus {
                wram: &mut self.wram,
                vram: &mut self.vram,
                cgram: &mut self.cgram,
                oam: &mut self.oam,
                ppu_regs: &mut self.ppu_regs,
                cpu_regs: &mut self.cpu_regs,
                apu_ports: &mut self.apu_ports,
                
                mult: &mut self.mult,
                dma_regs: &mut self.dma_regs,
                
                joy1_in: self.joy1_latch,
                joy2_in: self.joy2_latch,
                joy1_data1_auto: self.joy1_data1_auto,
                joy2_data1_auto: self.joy2_data1_auto,
                joy1_data2_auto: self.joy1_data2_auto,
                joy2_data2_auto: self.joy2_data2_auto,
                joypad_cmd: &mut joypad_cmd,
                cart: self.cart.as_mut().unwrap(),
            };
            
            self.cpu.cycle(&mut bus);
            
            match joypad_cmd {
                Some(JoypadCmd::ClockJoy1) => { self.joy1_latch >>= 1; },
                Some(JoypadCmd::ClockJoy2) => { self.joy2_latch >>= 1; },
                _ => {},
            }
        }
        
        if self.ppu.clocks == 0 {
            let mut interrupt: Option<CpuInterrupt> = None;
            
            let mut bus = PpuBus {
                vram: &mut self.vram,
                cgram: &mut self.cgram,
                oam: &mut self.oam,
                ppu_regs: &mut self.ppu_regs,
                cpu_regs: &mut self.cpu_regs,
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
        
        self.ssmp.clock(clocks, audio_buffer, &mut self.apu_ports);
    }
    
    pub fn get_system_clocks(&self) -> u64 {
        self.total_clocks
    }
}