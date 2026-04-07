use anyhow::{anyhow, Result};
use cartridge::Cartridge;
use controller::{ControllerPlayer, JoypadButton, JoypadCmd, SnemController};
use scpu::bus::CpuBus;
use scpu::dma::{self, DmaRegs};
use scpu::ioregs::CpuIoRegs;
use scpu::{Cpu65c816, CpuInterrupt};
use sppu::bus::PpuBus;
use sppu::color::Color;
use sppu::regs::PpuRegs;
use sppu::Ppu5C7x;
use ssmp::ioports::ApuIoPorts;
use ssmp::Ssmp;
use sysinfo::{CGRAM_SIZE, OAM_SIZE, VRAM_SIZE, WRAM_SIZE};

use crate::probe::{DebugProbe, NullProbe};

pub mod cartridge;
pub mod controller;
pub mod probe;
pub mod scpu;
pub mod sppu;
pub mod ssmp;
pub mod sysinfo;
mod utils;

macro_rules! cpu_bus {
    ($core:ident, $probe:expr) => {
        CpuBus {
            wram: &mut $core.wram,
            vram: &mut $core.vram,
            cgram: &mut $core.cgram,
            oam: &mut $core.oam,
            ppu_regs: &mut $core.ppu_regs,
            cpu_regs: &mut $core.cpu_regs,
            apu_ports: &mut $core.apu_ports,

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

            probe: $probe,
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
pub struct Snemulator<P: DebugProbe = NullProbe> {
    p1_controller: SnemController,
    p2_controller: SnemController,

    pub cpu: Cpu65c816<P>,
    pub ppu: Ppu5C7x,
    pub ssmp: Ssmp,

    pub wram: Box<[u8; WRAM_SIZE]>,
    pub vram: Box<[u16; VRAM_SIZE]>,
    pub cgram: Box<[Color; CGRAM_SIZE]>,
    pub oam: Box<[u8; OAM_SIZE]>,
    pub ppu_regs: PpuRegs,
    pub cpu_regs: CpuIoRegs,
    pub apu_ports: ApuIoPorts,

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

    pub frame_ready: bool,

    pub cart: Option<Cartridge>,
    pub total_cycles: u64,
    pub frame: u64,

    pub probe: Option<P>,
}

impl<P: DebugProbe> Snemulator<P> {
    pub fn with_probe(probe: P) -> Self {
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
            total_cycles: 0,
            frame: 0,
            probe: Some(probe),
        }
    }

    pub fn power_on(&mut self) {
        self.clear_regs();

        self.wram.fill(0);
        self.vram.fill(0);
        self.cgram.fill(Color::BLACK);
        self.oam.fill(0);

        self.ppu_regs.power_on();
        self.cpu_regs.power_on();
        self.apu_ports.power_on();

        for regs in self.dma_regs.iter_mut() {
            regs.power_on();
        }

        let mut bus = cpu_bus!(self, self.probe.as_mut().unwrap());
        self.cpu.power_on(&mut bus);

        self.ssmp.power_on();
        self.ppu.power_on();
    }

    pub fn reset(&mut self) {
        self.clear_regs();

        self.ppu_regs.reset();
        self.cpu_regs.reset();
        self.apu_ports.reset();

        for regs in self.dma_regs.iter_mut() {
            regs.reset();
        }

        let mut bus = cpu_bus!(self, self.probe.as_mut().unwrap());
        self.cpu.reset(&mut bus);

        self.ssmp.reset();
        self.ppu.reset();
    }

    fn clear_regs(&mut self) {
        self.dma_en = false;
        self.hdma_en = false;
        self.hdma_pending = false;
        self.dma_active_ch = 8;
        self.hdma_active_ch = 8;

        self.p1_controller = SnemController::new();
        self.p2_controller = SnemController::new();
        self.joy1_latch = 0;
        self.joy2_latch = 0;
        self.joy1_data1_auto = 0;
        self.joy2_data1_auto = 0;
        self.joy1_data2_auto = 0;
        self.joy2_data2_auto = 0;
        self.joypad_cmd = None;
        self.cpu_interrupt = None;
        self.frame_ready = false;
        self.frame = 0;
        self.total_cycles = 0;
    }

    pub fn load_rom(&mut self, data: Vec<u8>) -> Result<()> {
        self.cart = Some(Cartridge::from_rom(data).map_err(|e| anyhow!(e))?);

        self.power_on();

        Ok(())
    }

    pub fn set_button(&mut self, player: ControllerPlayer, button: JoypadButton, pressed: bool) {
        match player {
            ControllerPlayer::Player1 => self.p1_controller.set_button(button, pressed),
            ControllerPlayer::Player2 => self.p2_controller.set_button(button, pressed),
        }
    }

    pub fn run_frame(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>) {
        self.frame_ready = false;

        self.ssmp.start_frame();

        while !self.frame_ready && !self.probe.as_mut().unwrap().should_stop() {
            self.cycle(frame_buffer, audio_buffer);
        }

        let mut probe = self.probe.take().unwrap();
        probe.on_frame_end(self);
        self.probe = Some(probe);

        if self.frame_ready {
            self.frame += 1;
        }
    }

    fn cycle(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>) {
        let mut probe = self.probe.take().unwrap();

        let clocks = self.cpu.clocks.min(self.ppu.clocks);

        self.cpu.clocks -= clocks;
        self.ppu.clocks -= clocks;
        self.total_cycles += clocks as u64;

        if self.cpu.clocks == 0 {
            self.cycle_cpu(&mut probe);

            probe.on_instruction(self);
        }

        if self.ppu.clocks == 0 {
            let scanline = self.ppu.scanline;

            self.cycle_ppu(frame_buffer);

            probe.on_dot(self);

            if self.ppu.scanline != scanline {
                probe.on_scanline(self);
            }
        }

        self.ssmp.cycle(clocks, audio_buffer, &mut self.apu_ports);

        self.probe = Some(probe);
    }

    fn cycle_cpu(&mut self, probe: &mut P) {
        self.cpu.stopped = false;

        if self.hdma_en {
            self.cpu.stopped = true;
            self.do_hdma(probe);
        }

        if !self.hdma_en && self.dma_en {
            self.cpu.stopped = true;
            self.do_dma(probe);
        }

        self.joypad_cmd = None;

        let mut bus = cpu_bus!(self, probe);

        self.cpu.cycle(&mut bus);

        match self.joypad_cmd {
            Some(JoypadCmd::ClockJoy1) => {
                self.joy1_latch >>= 1;
            }
            Some(JoypadCmd::ClockJoy2) => {
                self.joy2_latch >>= 1;
            }
            _ => {}
        }
    }

    fn cycle_ppu(&mut self, frame_buffer: &mut [u8]) {
        self.cpu_interrupt = None;

        let mut bus = ppu_bus!(self, frame_buffer);

        self.ppu.cycle(&mut bus);

        match self.cpu_interrupt {
            Some(CpuInterrupt::IRQ) => {
                self.cpu.irq_pending = true;
            }
            Some(CpuInterrupt::NMI) => {
                self.cpu.nmi_pending = true;
            }
            _ => {}
        }

        if self.hdma_pending
            && self.ppu.scanline < sppu::VBLANK_START_SCANLINE
            && self.ppu.dot == sppu::HBLANK_START_DOT
        {
            self.hdma_en = true;
            self.dma_regs[self.hdma_active_ch].transfer_pattern_step = 0;
        }
    }

    fn do_hdma(&mut self, probe: &mut P) {
        let mut hdma_active_ch = self.hdma_active_ch;
        let mut bus = cpu_bus!(self, probe);

        // Table entry finished
        if bus.dma_regs[hdma_active_ch].scanlines_left == 0 {
            'seek_next_entry: while hdma_active_ch < 8 {
                let mut hdma_table_addr = bus.dma_regs[hdma_active_ch].a_bus_addr;
                hdma_table_addr.offset = bus.dma_regs[hdma_active_ch].hdma_table_offset;

                let scanline_counter = bus.read(hdma_table_addr);

                // Found a valid enty in this HDMA table
                if scanline_counter != 0 {
                    bus.dma_regs[hdma_active_ch].transfer_pattern_step = 0;
                    bus.dma_regs[hdma_active_ch].scanlines_left = scanline_counter & 0x7F;
                    bus.dma_regs[hdma_active_ch].hdma_reload_flag = get_bit_n!(scanline_counter, 7);

                    // Load indirect table address
                    if bus.dma_regs[hdma_active_ch].indirect_hdma {
                        let hdma_indirect_table_addr = u16::from_le_bytes([
                            bus.read(hdma_table_addr),
                            bus.read(hdma_table_addr),
                        ]);

                        bus.dma_regs[hdma_active_ch].hdma_table_offset += 2;

                        bus.dma_regs[hdma_active_ch].hdma_indirect_table_addr = scpu::Address {
                            bank: bus.dma_regs[hdma_active_ch].a_bus_addr.bank,
                            offset: hdma_indirect_table_addr,
                        }
                    }

                    break 'seek_next_entry;
                }

                // No more entries in this table, move to next channel
                bus.dma_regs[hdma_active_ch].hdma_en = false;
                hdma_active_ch += 1;
            }
        }

        self.hdma_active_ch = hdma_active_ch;

        // No active HDMA channel found
        if self.hdma_active_ch == 8 {
            self.hdma_en = false;
            self.hdma_pending = false;
            self.cpu.stopped = false;
            return;
        }

        let hdma_ch_regs = &mut self.dma_regs[self.hdma_active_ch];

        let a_bus_addr = if hdma_ch_regs.indirect_hdma {
            let addr = hdma_ch_regs.hdma_indirect_table_addr;

            hdma_ch_regs.hdma_indirect_table_addr.offset += 1;

            addr
        } else {
            let addr = scpu::Address {
                bank: hdma_ch_regs.a_bus_addr.bank,
                offset: hdma_ch_regs.hdma_table_offset,
            };

            hdma_ch_regs.hdma_table_offset += 1;
            addr
        };
        let b_bus_addr = hdma_ch_regs.get_b_with_offset();

        let (src_addr, dst_addr) = match hdma_ch_regs.direction {
            dma::Direction::AtoB => (a_bus_addr, b_bus_addr),
            dma::Direction::BtoA => (b_bus_addr, a_bus_addr),
        };

        hdma_ch_regs.scanline_counter -= 1;
        hdma_ch_regs.transfer_pattern_step += 1;

        self.hdma_en = match hdma_ch_regs.transfer_pattern {
            // Stop after first byte
            dma::TransferPattern::Pattern0 => false,

            // Stop after two bytes
            dma::TransferPattern::Pattern1
            | dma::TransferPattern::Pattern2
            | dma::TransferPattern::Pattern6 => hdma_ch_regs.transfer_pattern_step < 2,

            // Stop after four bytes
            dma::TransferPattern::Pattern3
            | dma::TransferPattern::Pattern4
            | dma::TransferPattern::Pattern5
            | dma::TransferPattern::Pattern7 => hdma_ch_regs.transfer_pattern_step < 4,
        };

        let mut bus = cpu_bus!(self, probe);
        let data = bus.read(src_addr);
        bus.write(dst_addr, data);
    }

    fn do_dma(&mut self, probe: &mut P) {
        let mut dma_ch_regs = &mut self.dma_regs[self.dma_active_ch];

        // HDMA indirect table register is same as DMA byte count register
        let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;

        // Channel's DMA transfer complete
        if byte_count == 0 {
            dma_ch_regs.dma_en = false;
            self.dma_active_ch += 1;

            'seek_active_channel: while self.dma_active_ch < 8 {
                dma_ch_regs = &mut self.dma_regs[self.dma_active_ch];

                let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;

                if dma_ch_regs.dma_en {
                    // Active channel found
                    if byte_count != 0 {
                        break 'seek_active_channel;
                    }

                    // Enabled channel has no bytes to transfer, disable it
                    dma_ch_regs.dma_en = false;
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

        let dma_ch_regs = &mut self.dma_regs[self.dma_active_ch]; // No longer mutable

        let a_bus_addr = dma_ch_regs.a_bus_addr;
        let b_bus_addr = dma_ch_regs.get_b_with_offset();

        let (src_addr, dst_addr) = match dma_ch_regs.direction {
            dma::Direction::AtoB => (a_bus_addr, b_bus_addr),
            dma::Direction::BtoA => (b_bus_addr, a_bus_addr),
        };

        dma_ch_regs.hdma_indirect_table_addr.offset -= 1; // byte_count -= 1
        dma_ch_regs.transfer_pattern_step += 1;
        dma_ch_regs.inc_a_bus_addr();

        let mut bus = cpu_bus!(self, probe);
        let data = bus.read(src_addr);
        bus.write(dst_addr, data);

        // if dst_addr.offset == 0x2118 || dst_addr.offset == 0x2119 {
        //     debug!("DMA transfered 0x{:02X} from ${:06X} to VRAM[{:04X}]", data, src_addr.to_u32(), bus.ppu_regs.vram_addr);
        // }
    }

    pub fn rom_slice(&self) -> &[u8] {
        self.cart.as_ref().unwrap().rom_slice()
    }
}

impl<P: DebugProbe> Snemulator<P> {
    pub fn run_frame_no_output(&mut self) {
        self.frame_ready = false;

        self.ssmp.start_frame();

        while !self.frame_ready && !self.probe.as_mut().unwrap().should_stop() {
            self.cycle_no_output();
        }

        let mut probe = self.probe.take().unwrap();
        probe.on_frame_end(self);
        self.probe = Some(probe);

        if self.frame_ready {
            self.frame += 1;
        }
    }

    fn cycle_no_output(&mut self) {
        let mut probe = self.probe.take().unwrap();

        let clocks = self.cpu.clocks.min(self.ppu.clocks);

        self.cpu.clocks -= clocks;
        self.ppu.clocks -= clocks;
        self.total_cycles += clocks as u64;

        if self.cpu.clocks == 0 {
            self.cycle_cpu(&mut probe);

            probe.on_instruction(self);
        }

        if self.ppu.clocks == 0 {
            let scanline = self.ppu.scanline;

            self.cycle_ppu_no_output();

            probe.on_dot(self);

            if self.ppu.scanline != scanline {
                probe.on_scanline(self);
            }
        }

        self.ssmp.cycle_no_output(clocks, &mut self.apu_ports);

        self.probe = Some(probe);
    }
    
    fn cycle_ppu_no_output(&mut self) {
        self.cpu_interrupt = None;
        
        let frame_buffer = &mut [];

        let mut bus = ppu_bus!(self, frame_buffer);
        
        self.ppu.cycle_no_output(&mut bus);

        match self.cpu_interrupt {
            Some(CpuInterrupt::IRQ) => {
                self.cpu.irq_pending = true;
            }
            Some(CpuInterrupt::NMI) => {
                self.cpu.nmi_pending = true;
            }
            _ => {}
        }

        if self.hdma_pending
            && self.ppu.scanline < sppu::VBLANK_START_SCANLINE
            && self.ppu.dot == sppu::HBLANK_START_DOT
        {
            self.hdma_en = true;
            self.dma_regs[self.hdma_active_ch].transfer_pattern_step = 0;
        }
    }
    
    pub fn update_layer_buffers(
        &mut self,
        bg1_buffer: &mut [u8],
        bg2_buffer: &mut [u8],
        bg3_buffer: &mut [u8],
        bg4_buffer: &mut [u8],
        obj_buffer: &mut [u8],
    ) {
        let frame_buffer = &mut [];
        
        let mut bus = ppu_bus!(self, frame_buffer);
        
        self.ppu.draw_debug_layers(&mut bus, bg1_buffer, bg2_buffer, bg3_buffer, bg4_buffer, obj_buffer);
    }
}

impl Snemulator<NullProbe> {
    pub fn new() -> Self {
        Self::with_probe(NullProbe {})
    }
}
