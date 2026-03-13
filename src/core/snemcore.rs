use anyhow::{Result, anyhow};
use crate::core::cartridge::Cartridge;
use crate::core::controller::{ControllerPlayer, JoypadButton, JoypadCmd, SnemController};
use crate::core::scpu::dma::{self, DmaRegs};
use crate::core::scpu::ioregs::CpuIoRegs;
use crate::core::scpu::mult::Mult5A22;
use crate::core::scpu::{self, Cpu65c816, CpuInterrupt};
use crate::core::scpu::bus::CpuBus;
use crate::core::sppu::{self, Ppu5C7x};
use crate::core::sppu::bus::PpuBus;
use crate::core::ssmp::Ssmp;
use crate::core::ssmp::ioports::ApuIoPorts;
use crate::core::sysinfo::{
    CGRAM_SIZE, OAM_SIZE, VRAM_SIZE, WRAM_SIZE
};
use crate::core::sppu::color::Color;
use crate::core::sppu::regs::PpuRegs;
use crate::get_bit_n;
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
            dma_en: &mut $core.dma_en,
            hdma_pending: &mut $core.hdma_pending,
            dma_active_ch: &mut $core.dma_active_ch,
            hdma_active_ch: &mut $core.hdma_active_ch,
            
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
    dma_en: bool,
    hdma_en: bool,
    hdma_pending: bool,
    dma_active_ch: usize,
    hdma_active_ch: usize,
    
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
            dma_en: false,
            hdma_en: false,
            hdma_pending: false,
            dma_active_ch: 8,
            hdma_active_ch: 8,
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
        
        while !self.frame_ready {
            self.cycle(frame_buffer, audio_buffer);
        }
    }
    
    fn cycle(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>) {
        let clocks = self.cpu.clocks.min(self.ppu.clocks);
        
        self.cpu.clocks -= clocks;
        self.ppu.clocks -= clocks;
        
        if self.cpu.clocks == 0 {
            self.cycle_cpu();
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
        
        if self.hdma_pending && self.ppu.scanline < sppu::VBLANK_START_SCANLINE && self.ppu.dot == sppu::HBLANK_START_DOT {
            self.hdma_en = true;
            self.dma_regs[self.hdma_active_ch].transfer_pattern_step = 0;
        }
        
        self.ssmp.clock(clocks, audio_buffer, &mut self.apu_ports);
    }
    
    fn cycle_cpu(&mut self) {
        if self.hdma_en {
            self.cpu.stopped = true;
            self.do_hdma();
        }
        
        if !self.hdma_en && self.dma_en {
            self.cpu.stopped = true;
            self.do_dma();
        }
        
        self.joypad_cmd = None;
        
        let mut bus = cpu_bus!(self);
        
        self.cpu.cycle(&mut bus);

        match self.joypad_cmd {
            Some(JoypadCmd::ClockJoy1) => { self.joy1_latch >>= 1; },
            Some(JoypadCmd::ClockJoy2) => { self.joy2_latch >>= 1; },
            _ => {},
        }
    }
    
    fn do_hdma(&mut self) {
        let hdma_ch_regs = &mut self.dma_regs[self.hdma_active_ch];
        
        // check if channel active or find next active channel
        if hdma_ch_regs.scanlines_left == 0 {
            'seek_next_entry: while self.hdma_active_ch < 8 {
                let hdma_ch_regs = &mut self.dma_regs[self.hdma_active_ch];
                
                let mut hdma_table_addr = hdma_ch_regs.a_bus_addr;
                hdma_table_addr.offset = hdma_ch_regs.hdma_table_offset;
                
                let mut bus = cpu_bus!(self);
                
                let scanline_counter = bus.read(hdma_table_addr);
                
                let hdma_ch_regs = &mut self.dma_regs[self.hdma_active_ch];
                
                if scanline_counter != 0 {
                    // Found a valid enty in this HDMA table
                    hdma_ch_regs.transfer_pattern_step = 0;
                    hdma_ch_regs.scanlines_left = scanline_counter & 0x7F;
                    hdma_ch_regs.hdma_reload_flag = get_bit_n!(scanline_counter, 7);
                    break 'seek_next_entry;   
                }
                
                // No more entries in this table, move to next channel
                hdma_ch_regs.hdma_en = false;
                self.hdma_active_ch += 1;
            }
        }
        
        // No active HDMA channel found
        if self.hdma_active_ch == 8 {
            self.hdma_en = false;
            self.hdma_pending = false;
            self.cpu.stopped = false;
            return;
        }
        
        let hdma_ch_regs = &mut self.dma_regs[self.hdma_active_ch];
        
        // do single byte transfer
        
        let a_bus_addr = hdma_ch_regs.hdma_get_a_bus_addr();
        let b_bus_addr = hdma_ch_regs.get_b_with_offset();
        
        let (src_addr, dst_addr) = match hdma_ch_regs.direction {
            dma::Direction::AtoB => (a_bus_addr, b_bus_addr),
            dma::Direction::BtoA => (b_bus_addr, a_bus_addr),
        };
        
        hdma_ch_regs.scanline_counter -= 1;
        hdma_ch_regs.transfer_pattern_step += 1;
        
        let mut bus = cpu_bus!(self);
        
        let data = bus.read(src_addr);
        
        bus.write(dst_addr, data);
        
    }
    
    fn do_dma(&mut self) {
        let dma_ch_regs = &mut self.dma_regs[self.dma_active_ch];
        
        // HDMA indirect table register is same as DMA byte count register
        let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;

        if byte_count == 0 {
            dma_ch_regs.dma_en = false;
            
            self.dma_active_ch += 1;
            
            'seek_active_channel: while self.dma_active_ch < 8 {
                let dma_ch_regs = &mut self.dma_regs[self.dma_active_ch];
                
                let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;
                
                if dma_ch_regs.dma_en {
                    if byte_count == 0 {
                        dma_ch_regs.dma_en = false;
                    } else {
                        break 'seek_active_channel;
                    }
                }
                
                self.dma_active_ch += 1;
            }
        }
        
        // No DMA channels are enabled, disable DMA
        if self.dma_active_ch == 8 {
            self.dma_en = false;
            self.cpu.stopped = false;
            return;
        }
        
        let dma_ch_regs = &mut self.dma_regs[self.dma_active_ch]; // To appease borrow checker
        
        let abus_addr = dma_ch_regs.a_bus_addr;
        let bbus_addr = dma_ch_regs.get_b_with_offset();
        
        let (src_addr, dst_addr) = match dma_ch_regs.direction {
            dma::Direction::AtoB => (abus_addr, bbus_addr),
            dma::Direction::BtoA => (bbus_addr, abus_addr),
        };

        let mut bus = cpu_bus!(self);
        
        let data = bus.read(src_addr);
        
        bus.write(dst_addr, data);
        
        let dma_ch_regs = &mut self.dma_regs[self.dma_active_ch]; // To appease borrow checker
        
        dma_ch_regs.hdma_indirect_table_addr.offset -= 1; // byte_count -= 1
        
        dma_ch_regs.inc_a_bus_addr();
    }
}