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
            hdma_needs_init: &mut $core.hdma_needs_init,

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
    pub ppu: Ppu5C7x<P>,
    pub ssmp: Ssmp,

    pub wram: Box<[u8; WRAM_SIZE]>,
    pub vram: Box<[u16; VRAM_SIZE]>,
    pub cgram: Box<[Color; CGRAM_SIZE]>,
    pub oam: Box<[u8; OAM_SIZE]>,
    pub ppu_regs: PpuRegs,
    pub cpu_regs: CpuIoRegs,
    pub apu_ports: ApuIoPorts,

    pub dma_regs: [DmaRegs; 8],
    pub dma_en: bool,
    pub hdma_en: bool,
    pub hdma_pending: bool,
    pub hdma_needs_init: bool,
    pub dma_active_ch: usize,
    pub hdma_active_ch: usize,

    pub joy1_latch: u16,
    pub joy2_latch: u16,
    pub joy1_data1_auto: u16,
    pub joy2_data1_auto: u16,
    pub joy1_data2_auto: u16,
    pub joy2_data2_auto: u16,
    pub joypad_cmd: Option<JoypadCmd>,
    pub cpu_interrupt: Option<CpuInterrupt>,

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
            hdma_needs_init: false,
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
    
    pub fn init_probe(&mut self) {
        let mut probe = self.probe.take().unwrap();
        probe.init(self);
        self.probe = Some(probe);
    }

    pub fn do_with_probe<F, A>(&mut self, f: F) -> Option<A>
        where F: FnOnce(&mut P, &mut Self) -> A,
    {
        if let Some(mut probe) = self.probe.take() {
            let result = f(&mut probe, self);
            self.probe = Some(probe);
            Some(result)
        } else {
            None
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
        self.hdma_needs_init = true;
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

    pub fn run_frame(&mut self, frame_buffer: Option<&mut [u8]>, audio_buffer: Option<&mut Vec<i16>>) {
        self.frame_ready = false;

        self.ssmp.start_frame();

        if let Some(frame_buffer) = frame_buffer {
            if let Some(audio_buffer) = audio_buffer {
                while !self.frame_ready && !self.probe.as_mut().unwrap().should_stop() {
                    self.cycle(frame_buffer, audio_buffer);
                }
            } else {
                while !self.frame_ready && !self.probe.as_mut().unwrap().should_stop() {
                    self.cycle(frame_buffer, &mut Vec::new());
                }
            }
        } else {
            if let Some(audio_buffer) = audio_buffer {
                while !self.frame_ready && !self.probe.as_mut().unwrap().should_stop() {
                    self.cycle_no_video(audio_buffer);
                }
            } else {
                while !self.frame_ready && !self.probe.as_mut().unwrap().should_stop() {
                    self.cycle_no_video(&mut Vec::new());
                }
            }
        }

        let mut probe = self.probe.take().unwrap();
        probe.on_frame(self);
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
            self.cycle_ppu(frame_buffer);

            probe.on_dot(self);

            if self.ppu.dot == 0 {
                probe.on_scanline(self);
            }
        }

        self.ssmp.cycle(clocks, audio_buffer, &mut self.apu_ports);
        
        probe.on_emulation_cycle(self);

        self.probe = Some(probe);
    }

    pub fn cycle_instruction(&mut self, frame_buffer: Option<&mut [u8]>) {
        let mut audio_buffer = Vec::new();
        let video_out = frame_buffer.is_some();
        let frame_buffer = frame_buffer.unwrap_or_else(|| &mut []);
        
        while self.cpu.clocks > self.ppu.clocks {
            if video_out {
                self.cycle(frame_buffer, &mut audio_buffer);
            } else {
                self.cycle_no_video(&mut audio_buffer);
            }
        }
        
        if video_out {
            self.cycle(frame_buffer, &mut audio_buffer);
        } else {
            self.cycle_no_video(&mut audio_buffer);
        }
    }
    
    fn cycle_cpu(&mut self, probe: &mut P) {
        self.cpu.stopped = false;

        if self.hdma_needs_init && self.ppu.scanline == 0 {
            self.hdma_needs_init = false;
            self.hdma_init_channels(probe);
        }

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

        if self.hdma_pending && self.ppu.scanline < sppu::VBLANK_START_SCANLINE
            && self.ppu.dot == sppu::HBLANK_START_DOT
        {
            self.hdma_en = self.hdma_active_ch < 8;

            if self.hdma_active_ch < 8 {
                self.dma_regs[self.hdma_active_ch].hdma_do_transfer = true;
            }
        }
    }

    fn do_hdma(&mut self, probe: &mut P) {
        let hdma_active_ch = self.hdma_active_ch;
        
        if self.dma_regs[hdma_active_ch].hdma_entry_just_loaded {
            self.dma_regs[hdma_active_ch].hdma_entry_just_loaded = false;
        } else {
            self.dma_regs[hdma_active_ch].scanlines_left -= 1;
        }

        // Table entry finished
        if self.dma_regs[self.hdma_active_ch].scanlines_left == 0 {
            probe.on_hdma_end(hdma_active_ch);
            
            if !self.hdma_load_entry(self.hdma_active_ch, probe) {
                // Channel exhausted, find next active channel
                self.hdma_active_ch = (self.hdma_active_ch + 1..8)
                    .find(|&ch| self.dma_regs[ch].hdma_en)
                    .unwrap_or(8);

                if self.hdma_active_ch == 8 {
                    self.hdma_en = false;
                    self.hdma_pending = false;
                    self.cpu.stopped = false;
                }
            }

            self.hdma_en = false;
            self.cpu.stopped = false;
            return;
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

        let a_bus_addr: scpu::Address;
        let b_bus_addr: scpu::Address;

        if hdma_ch_regs.indirect_hdma {
            a_bus_addr = hdma_ch_regs.hdma_indirect_table_addr;
            b_bus_addr = hdma_ch_regs.get_b_with_offset();

            if hdma_ch_regs.hdma_repeat_flag {
                hdma_ch_regs.hdma_indirect_table_addr.offset += 1;
            }
        } else {
            a_bus_addr = scpu::Address {
                bank: hdma_ch_regs.a_bus_addr.bank,
                offset: hdma_ch_regs.hdma_table_offset,
            };
            b_bus_addr = hdma_ch_regs.get_b_with_offset();

            if hdma_ch_regs.hdma_repeat_flag {
                hdma_ch_regs.hdma_table_offset += 1;
            }
        }

        let (src_addr, dst_addr) = match hdma_ch_regs.direction {
            dma::Direction::AtoB => (a_bus_addr, b_bus_addr),
            dma::Direction::BtoA => (b_bus_addr, a_bus_addr),
        };

        let transfer_pattern_length = match hdma_ch_regs.transfer_pattern {
            // Stop after first byte
            dma::TransferPattern::Pattern0 => 1,
            // Stop after two bytes
            dma::TransferPattern::Pattern1
            | dma::TransferPattern::Pattern2
            | dma::TransferPattern::Pattern6 => 2,
            // Stop after four bytes
            dma::TransferPattern::Pattern3
            | dma::TransferPattern::Pattern4
            | dma::TransferPattern::Pattern5
            | dma::TransferPattern::Pattern7 => 4,
        };

        hdma_ch_regs.transfer_pattern_step += 1;
        hdma_ch_regs.transfer_pattern_step %= transfer_pattern_length;

        if hdma_ch_regs.transfer_pattern_step == 0 {
            // Full pattern transferred for this scanline; stop until next hblank
            self.hdma_en = false;
            self.cpu.stopped = false;

            // Non-repeat: only transfer once per entry. Repeat: transfer every scanline.
            if !hdma_ch_regs.hdma_repeat_flag {
                hdma_ch_regs.hdma_do_transfer = false;
            }
        }

        if hdma_ch_regs.hdma_do_transfer {
            let mut bus = cpu_bus!(self, probe);
            let value = bus.read(src_addr);
            bus.write(dst_addr, value);
            probe.on_hdma_transfer(self.hdma_active_ch, src_addr.to_u32(), dst_addr.to_u32(), value);
        }
    }

    fn do_dma(&mut self, probe: &mut P) {
        let mut dma_ch_regs = &mut self.dma_regs[self.dma_active_ch];

        // HDMA indirect table register is same as DMA byte count register
        let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;

        // Channel's DMA transfer complete
        if byte_count == 0 {
            probe.on_dma_end(self.dma_active_ch);
            
            dma_ch_regs.dma_en = false;
            self.dma_active_ch += 1;

            'seek_active_channel: while self.dma_active_ch < 8 {
                dma_ch_regs = &mut self.dma_regs[self.dma_active_ch];

                let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;

                if dma_ch_regs.dma_en {
                    // Active channel found
                    if byte_count != 0 {
                        probe.on_dma_start(self.dma_active_ch);
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
        let value = bus.read(src_addr);
        bus.write(dst_addr, value);
        
        probe.on_dma_transfer(self.dma_active_ch, src_addr.to_u32(), dst_addr.to_u32(), value);

        // if dst_addr.offset == 0x2118 || dst_addr.offset == 0x2119 {
        //     debug!("DMA transfered 0x{:02X} from ${:06X} to VRAM[{:04X}]", data, src_addr.to_u32(), bus.ppu_regs.vram_addr);
        // }
    }

    pub fn rom_slice(&self) -> &[u8] {
        self.cart.as_ref().unwrap().rom_slice()
    }
}

impl<P: DebugProbe> Snemulator<P> {
    fn cycle_no_video(&mut self, audio_buffer: &mut Vec<i16>) {
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

        self.ssmp.cycle(clocks, audio_buffer, &mut self.apu_ports);

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
    
    /// Helper function to read a non-MMIO address from mapped memory
    pub fn cpu_read_mem(&self, addr: scpu::Address) -> u8 {
        match addr.bank {
            // Banks $00-$3F: LoROM mapping
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                // WRAM mirror (first 8KB)
                0x0000..=0x1FFF => self.wram[addr.offset as usize],

                // Cartridge (LoROM: $8000-$FFFF)
                0x8000..=0xFFFF => self.cart.as_ref().unwrap().read(addr),

                _ => 0, // Open bus
            },

            // Banks $40-$6F: LoROM cartridge
            0x40..=0x6F => self.cart.as_ref().unwrap().read(addr),

            // Banks $70-$7D: SRAM or ROM
            0x70..=0x7D => self.cart.as_ref().unwrap().read(addr),

            // Banks $7E-$7F: WRAM (full 128KB)
            0x7E..=0x7F => {
                let wram_addr = ((addr.bank as usize & 1) << 16) | (addr.offset as usize);
                self.wram[wram_addr]
            }

            // Banks $C0-$FF: HiROM cartridge / mirror
            0xC0..=0xFF => self.cart.as_ref().unwrap().read(addr),
        }
    }

    /// Reads the next HDMA table entry for `ch` into runtime state.
    /// Advances hdma_table_offset past the consumed bytes.
    /// For indirect mode, also reads and stores hdma_indirect_table_addr.
    /// Returns false if scanline_count == 0 (end of table), disabling the channel.
    fn hdma_load_entry(&mut self, ch: usize, probe: &mut P) -> bool {
        let mut bus = cpu_bus!(self, probe);

        let table_addr = scpu::Address {
            bank: bus.dma_regs[ch].a_bus_addr.bank,
            offset: bus.dma_regs[ch].hdma_table_offset,
        };

        let scanline_count = bus.read(table_addr);
        bus.dma_regs[ch].hdma_table_offset += 1;

        if scanline_count == 0 {
            bus.dma_regs[ch].hdma_en = false;
            return false;
        }

        bus.dma_regs[ch].entry_scanline_count = scanline_count & 0x7F;
        bus.dma_regs[ch].scanlines_left = scanline_count & 0x7F;
        bus.dma_regs[ch].hdma_repeat_flag = get_bit_n!(scanline_count, 7);
        bus.dma_regs[ch].hdma_entry_just_loaded = true;
        bus.dma_regs[ch].transfer_pattern_step = 0;
        bus.dma_regs[ch].hdma_do_transfer = true;

        if bus.dma_regs[ch].indirect_hdma {
            let lo_addr = scpu::Address {
                bank: bus.dma_regs[ch].a_bus_addr.bank,
                offset: bus.dma_regs[ch].hdma_table_offset,
            };
            let lo = bus.read(lo_addr);

            let hi_addr = scpu::Address { offset: lo_addr.offset + 1, ..lo_addr };
            let hi = bus.read(hi_addr);

            bus.dma_regs[ch].hdma_table_offset += 2;

            // Bank byte comes from $43n7, already stored in hdma_indirect_table_addr.bank
            let indirect_bank = bus.dma_regs[ch].hdma_indirect_table_addr.bank;
            bus.dma_regs[ch].hdma_indirect_table_addr = scpu::Address {
                bank: indirect_bank,
                offset: u16::from_le_bytes([lo, hi]),
            };
        }

        true
    }

    /// Called once per frame before the first hblank of active display.
    /// Resets table pointers and loads the first entry for every HDMA-enabled channel.
    /// Channels whose first entry has scanline_count == 0 are disabled immediately.
    fn hdma_init_channels(&mut self, probe: &mut P) {
        for ch in 0..8 {
            if !self.dma_regs[ch].hdma_en {
                continue;
            }

            // Reset table pointer to the base A-bus address for this frame
            self.dma_regs[ch].hdma_table_offset = self.dma_regs[ch].a_bus_addr.offset;
            self.dma_regs[ch].hdma_initialized = true;

            if !self.hdma_load_entry(ch, probe) {
                // Channel had an empty table; already disabled inside hdma_load_entry
                continue;
            }

            probe.on_hdma_start(ch);
        }

        // Set hdma_active_ch to first still-enabled channel
        self.hdma_active_ch = (0..8)
            .find(|&ch| self.dma_regs[ch].hdma_en)
            .unwrap_or(8);

        self.hdma_pending = self.hdma_active_ch < 8;
    }
}

impl Snemulator<NullProbe> {
    pub fn new() -> Self {
        Self::with_probe(NullProbe {})
    }
}
