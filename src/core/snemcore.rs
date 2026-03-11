use anyhow::{Result, anyhow};
use crate::core::cartridge::Cartridge;
use crate::core::controller::{ControllerPlayer, JoypadButton, JoypadCmd, SnemController};
use crate::core::scpu::dma::DmaRegs;
use crate::core::scpu::ioregs::CpuIoRegs;
use crate::core::scpu::mult::Mult5A22;
use crate::core::scpu::{self, Cpu65c816, CpuInterrupt};
use crate::core::scpu::bus::CpuBus;
use crate::core::sppu::Ppu5C7x;
use crate::core::sppu::bus::PpuBus;
use crate::core::ssmp::Ssmp;
use crate::core::ssmp::ioports::ApuIoPorts;
use crate::core::sysinfo::{
    CGRAM_SIZE, OAM_SIZE, VRAM_SIZE, WRAM_SIZE
};
use crate::core::sppu::color::Color;
use crate::core::sppu::regs::PpuRegs;
use log::{debug, trace};

macro_rules! cpu_bus {
    ($core:ident) => {
        CpuBus {
            wram: &mut $core.wram,
            vram: &mut $core.vram,
            cgram: &mut $core.cgram,
            oam: &mut $core.oam,
            ppu_regs: &mut $core.ppu_regs,
            cpu_regs: &mut $core.cpu_regs,
            apu_ports: &mut $core.apu_ports,
            
            mult: &mut $core.mult,
            dma_regs: &mut $core.dma_regs,
            
            joy1_in: $core.joy1_latch,
            joy2_in: $core.joy2_latch,
            joy1_data1_auto: $core.joy1_data1_auto,
            joy2_data1_auto: $core.joy2_data1_auto,
            joy1_data2_auto: $core.joy1_data2_auto,
            joy2_data2_auto: $core.joy2_data2_auto,
            joypad_cmd: &mut $core.joypad_cmd,
            cart: $core.cart.as_mut().unwrap(),
        }
    };
}

macro_rules! ppu_bus {
    ($core:ident, $frame_buffer:ident) => {
        PpuBus {
            vram: &mut $core.vram,
            cgram: &mut $core.cgram,
            oam: &mut $core.oam,
            ppu_regs: &mut $core.ppu_regs,
            cpu_regs: &mut $core.cpu_regs,
            $frame_buffer,
            frame_ready: &mut $core.frame_ready,
            interrupt: &mut $core.cpu_interrupt,
        }
    };
}

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
    joypad_cmd: Option<JoypadCmd>,
    cpu_interrupt: Option<CpuInterrupt>,
    
    frame_ready: bool,
    
    cart: Option<Cartridge>,
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
            mult: Mult5A22::default(),
            dma_regs: [DmaRegs::default(); 8],
            joy1_latch: 0,
            joy2_latch: 0,
            joy1_data1_auto: 0,
            joy2_data1_auto: 0,
            joy1_data2_auto: 0,
            joy2_data2_auto: 0,
            joypad_cmd: None,
            cpu_interrupt: None,
            frame_ready: false,
            cart: None,
        }
    }
    
    fn power_on(&mut self) {
        self.ssmp.power_on();
        self.mult.power_on();
        
        let mut bus = cpu_bus!(self);
        self.cpu.power_on(&mut bus);
        
    }
    
    pub fn load_rom(&mut self, data: Vec<u8>) -> Result<()> {
        self.cart = Some(Cartridge::from_rom(data).map_err(|e| anyhow!(e))?);
        
        self.power_on();
        
        Ok(())
    }

    pub fn set_button(&mut self, player: ControllerPlayer, button: JoypadButton, pressed: bool) {
        trace!("{player:?} button {button:?} pressed = {pressed}");
        
        match player {
            ControllerPlayer::Player1 => self.p1_controller.set_button(button, pressed),
            ControllerPlayer::Player2 => self.p2_controller.set_button(button, pressed),
        }
    }

    pub fn run_frame(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>) {
        self.frame_ready = false;
        
        self.ssmp.start_frame();
        
        // if self.ppu.frame == 120 {
        //     for group in self.cgram.chunks(4) {
        //         debug!("{:?} {:?} {:?} {:?}", group[0], group[1], group[2], group[3]);
        //     }
        // }
        
        // trace!("Frame {}", self.ppu.frame);
        
        // if self.ppu.frame > 100 {
        //     for _ in 0..100 {
        //         self.cycle(frame_buffer, audio_buffer);
        //     }
        //     return;
        // }
        
        while !self.frame_ready {
            self.cycle(frame_buffer, audio_buffer);
        }
    }
    
    fn cycle(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>) {
        let clocks = self.cpu.clocks.min(self.ppu.clocks);
        
        self.cpu.clocks -= clocks;
        self.ppu.clocks -= clocks;
        
        if self.cpu.clocks == 0 {
            self.joypad_cmd = None;
            
            let mut bus = cpu_bus!(self);
            
            self.cpu.cycle(&mut bus);
            
            // if self.ppu.frame > 100 {
            //     trace!("{}", scpu::dissasembler::disassemble(&self.cpu, &mut bus));
            // }
            
            match self.joypad_cmd {
                Some(JoypadCmd::ClockJoy1) => { self.joy1_latch >>= 1; },
                Some(JoypadCmd::ClockJoy2) => { self.joy2_latch >>= 1; },
                _ => {},
            }
        }
        
        if self.ppu.clocks == 0 {
            self.cpu_interrupt = None;
            
            let mut bus = ppu_bus!(self, frame_buffer);
            
            self.ppu.cycle(&mut bus);
            
            match self.cpu_interrupt {
                Some(CpuInterrupt::IRQ) => { self.cpu.irq_pending = true; }
                Some(CpuInterrupt::NMI) => { self.cpu.nmi_pending = true; }
                _ => {}
            }
        }
        
        self.ssmp.clock(clocks, audio_buffer, &mut self.apu_ports);
    }
}