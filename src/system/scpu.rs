mod dma;
mod utils;

use dma::{DmaChannel, DmaStatus};
use utils::{CpuAddress, is_mmio_addr};

use crate::log::SnemLogger;
use crate::system::cartridge::Cartridge;
use crate::system::ppu::PpuData;
use crate::system::ssmp::ApuIORegs;
use crate::utils::util_macros::bool2byte;
use crate::utils::{dec_low_byte, inc_low_byte, GetBits};

use std::cell::RefCell;
use std::rc::Rc;

const WRAM_SIZE: usize = 128 * 1024; // 128 KiB

#[derive(Debug, Clone, Copy, Default)]
pub enum MappingMode {
    #[default]
    LoROM,
    HiROM,
    ExHiROM,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum CpuMode {
    Emulation,
    Native,
}

#[derive(Debug, PartialEq)]
enum RegSize {
    Byte,
    TwoBytes,
}

#[derive(Debug)]
enum MemSel {
    FastROM,
    SlowROM,
}

#[derive(Debug)]
enum HVTimerIRQ {
    None,   // Ignore H/V Timers
    HTimer, // IRQ when H counter == HTIME
    VTimer, // IRQ when V counter == VTIME and H counter == 0
    Both,   // IRQ when V counter == VTIME and H counter == HTIME
}

pub enum Flag {
    FlagC = 1,   // Carry
    FlagZ = 2,   // Zero
    FlagI = 4,   // IRQ Disable
    FlagD = 8,   // Decimal Mode
    FlagX = 16,  // X Register Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagM = 32,  // Accumulator Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagV = 64,  // Overflow
    FlagN = 128, // Negative
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CpuInterrupt {
    IRQ,
    NMI,
    Reset,
    Abort,
}

pub struct Cpu65c816 {
    // Internal Registers
    acc: u16,
    x: u16,
    y: u16,
    pc: u16,
    stk_ptr: u16,
    direct_page: u16,
    data_bank: u8,
    prg_bank: u8,
    status: u8,

    mode: CpuMode,
    mapping_mode: MappingMode,
    mem_sel: MemSel,
    branch_taken: bool,
    page_crossed: bool,
    stopped: bool,
    awaiting_interrupt: bool,
    sys_clocks_until_clock: usize,

    wram: [u8; WRAM_SIZE],
    rom: Vec<u8>,
    rom_mirror: usize,
    has_sram: bool,

    dma_status: DmaStatus,
    dma_channels: Vec<DmaChannel>,
    active_channel_idx: usize,

    ppu_data: Rc<PpuData>,
    apuio_regs: Rc<ApuIORegs>,

    hv_timer_irq: HVTimerIRQ,
    vblank_nmi_ignore: bool,

    logger: Rc<RefCell<SnemLogger>>,
}

// SNES System Functionality
impl Cpu65c816 {
    // Creates a new, uninitialized 65c816 CPU
    pub fn new(
        ppu_data: Rc<PpuData>,
        apuio_regs: Rc<ApuIORegs>,
        logger: Rc<RefCell<SnemLogger>>,
    ) -> Self {
        Self {
            acc: 0,
            x: 0,
            y: 0,
            pc: 0,
            stk_ptr: 0,
            direct_page: 0,
            data_bank: 0,
            prg_bank: 0,
            status: 0,

            mode: CpuMode::Emulation,
            mapping_mode: MappingMode::LoROM,
            mem_sel: MemSel::SlowROM,
            branch_taken: false,
            page_crossed: false,
            stopped: false,
            awaiting_interrupt: false,
            sys_clocks_until_clock: 0,

            wram: [0; WRAM_SIZE],
            rom: Vec::new(),
            rom_mirror: 0,
            has_sram: false,

            dma_status: DmaStatus::Off,
            dma_channels: vec![DmaChannel::default(); 8],
            active_channel_idx: 8,

            ppu_data,
            apuio_regs,

            hv_timer_irq: HVTimerIRQ::None,
            vblank_nmi_ignore: true,

            logger,
        }
    }

    /// Sets the CPU to its proper initial state. Can be triggered by an interrupt.
    pub fn initialize(&mut self) {
        self.x = 0;
        self.y = 0;
        self.data_bank = 0;
        self.prg_bank = 0;
        self.direct_page = 0;
        self.stk_ptr = 0x01FF;
        self.status = 0x34;
        self.reset();
    }

    pub fn load_cart(&mut self, cart: Cartridge) {
        self.mapping_mode = cart.mapping_mode();
        self.rom = cart.rom_data();
        self.rom_mirror = self.rom.len() - 1;
    }

    pub fn reset(&mut self) {
        self.trigger_interrupt(CpuInterrupt::Reset);
    }
}

// Internal Helper Functions / Bus Behavior
impl Cpu65c816 {
    const ONE_CYCLE: usize = 6;
    const ONE_CYCLE_SLOW: usize = 8;
    const TWO_CYCLE: usize = 12;
    const THREE_CYCLE: usize = 18;
    const FOUR_CYCLE: usize = 24;

    fn add_clocks(&mut self, clocks: usize) {
        self.sys_clocks_until_clock += clocks;
    }

    fn read(&mut self, address: u32) -> u8 {
        // println!("Read from: 0x{address:06x}");

        let (data, clocks) = match (address.bank(), address.bank_addr()) {
            // Mirror of low RAM
            (0..=0x3F, bank_addr @ 0..=0x1FFF) | (0x80..=0xBF, bank_addr @ 0..=0x1FFF) => {
                let mirrored_addr = bank_addr & 0x1FFF;

                (self.wram[mirrored_addr as usize], Cpu65c816::ONE_CYCLE_SLOW)
            }

            // WRAM
            (0x7E..=0x7F, _) => {
                let ram_addr = address & 0x01FFFF;

                (self.wram[ram_addr as usize], Cpu65c816::ONE_CYCLE_SLOW)
            }

            // MMIO Registers
            (0..=0x3F, bank_addr @ 0x2000..=0x5FFF)
            | (0x80..=0xBF, bank_addr @ 0x2000..=0x5FFF) => {
                (self.read_mmio_regs(bank_addr), Cpu65c816::ONE_CYCLE_SLOW)
            }

            // Anywhere else in the addressable space (dependent on mapping mode)
            _ => match self.mapping_mode {
                MappingMode::LoROM => self.read_lorom(address),
                MappingMode::HiROM => self.read_hirom(address),
                MappingMode::ExHiROM => self.read_exhirom(address),
            },
        };

        match self.dma_status {
            DmaStatus::Off => self.add_clocks(clocks),
            _ => {}
        }

        data
    }

    fn write(&mut self, address: u32, data: u8) {
        let clocks = match (address.bank(), address.bank_addr()) {
            // Mirror of low RAM
            (0..=0x3F, bank_addr @ 0..=0x1FFF) | (0x80..=0xBF, bank_addr @ 0..=0x1FFF) => {
                let mirrored_addr = bank_addr & 0x1FFF;
                self.wram[mirrored_addr as usize] = data;

                Cpu65c816::ONE_CYCLE_SLOW
            }

            // WRAM
            (0x7E..=0x7F, _) => {
                let ram_addr = address & 0x01FFFF;
                self.wram[ram_addr as usize] = data;

                Cpu65c816::ONE_CYCLE_SLOW
            }

            // MMIO Registers
            (0..=0x3F, bank_addr @ 0x2000..=0x5FFF)
            | (0x80..=0xBF, bank_addr @ 0x2000..=0x7FFF) => {
                self.write_mmio_regs(bank_addr, data);

                Cpu65c816::ONE_CYCLE_SLOW
            }

            // Anywhere else in the addressable space (dependent on mapping mode)
            _ => match self.mapping_mode {
                MappingMode::LoROM => self.write_lorom(address, data),
                MappingMode::HiROM => self.write_hirom(address, data),
                MappingMode::ExHiROM => self.write_exhirom(address, data),
            },
        };

        match self.dma_status {
            DmaStatus::Off => self.add_clocks(clocks),
            _ => {}
        };
    }

    fn read_mmio_regs(&mut self, mmio_address: u16) -> u8 {
        match mmio_address {
            0x2100..=0x213F => self.ppu_data.read(mmio_address as u8),

            0x2140 => self.apuio_regs.apuio0.get(),
            0x2141 => self.apuio_regs.apuio1.get(),
            0x2142 => self.apuio_regs.apuio2.get(),
            0x2143 => self.apuio_regs.apuio3.get(),

            // NOTE: This read is only for cpu debugging purposes, and
            // will be removed later.
            0x4210 => {
                let vblank_nmi_bit = if self.ppu_data.cpu_vblank_nmi() {
                    0x80
                } else {
                    0
                };
                let cpu_version_bits = 0;

                self.ppu_data.clear_cpu_vblank_nmi();

                // println!("read 0x{:02X} from vblanknmi", vblank_nmi_bit);

                vblank_nmi_bit | cpu_version_bits
            }

            0x4212 => {
                let vblank_bit = if self.ppu_data.in_vblank() { 0x80 } else { 0 };
                let hblank_bit = if self.ppu_data.in_hblank() { 0x40 } else { 0 };
                let auto_joypad_read_bit = 0;

                vblank_bit | hblank_bit | auto_joypad_read_bit
            }

            0x4300..=0x43FF if ((mmio_address >> 4) & 0xF) < 8 => self.read_dma_regs(mmio_address),

            _ => {
                // if mmio_address != 0x2180 {
                // println!(" ==== Attempt to read mmio reg ${mmio_address:04X}");
                // }

                0
            }
        }
    }

    fn write_mmio_regs(&mut self, mmio_address: u16, data: u8) {
        match mmio_address {
            0x2100..=0x213F => {
                self.ppu_data.write(mmio_address as u8, data);
            }
            0x2140 => { self.apuio_regs.cpuio0.set(data) }
            0x2141 => { self.apuio_regs.cpuio1.set(data) }
            0x2142 => { self.apuio_regs.cpuio2.set(data) }
            0x2143 => { self.apuio_regs.cpuio3.set(data) }
            0x4200 => {
                self.vblank_nmi_ignore = (data & 0x80) == 0;
                self.hv_timer_irq = match (data >> 4) & 3 {
                    0 => HVTimerIRQ::None,
                    1 => HVTimerIRQ::HTimer,
                    2 => HVTimerIRQ::VTimer,
                    3 => HVTimerIRQ::Both,
                    _ => unreachable!(),
                };

                println!(
                    "Vblank NMI ignore set to {} and H/V Timer IRQ to {:?}",
                    self.vblank_nmi_ignore, self.hv_timer_irq
                );
            }

            0x420B => {
                if data == 0 {
                    self.dma_status = DmaStatus::Off;
                } else {
                    self.dma_status = DmaStatus::DMA;
                }

                for (i, dma_channel) in self.dma_channels.iter_mut().enumerate().rev() {
                    dma_channel.active = (data & (1 << i)) != 0;

                    if dma_channel.active {
                        dma_channel.bytes_written = 0;
                        self.active_channel_idx = i;
                    }
                }
            }
            0x420C => {
                // for i in 0..8 {
                //     self.dma_channels[i].active = (data & (1 << i)) != 0;
                // }

                println!("Wrote 0x{data:02X} to HDMA enable");
            }

            0x4300..=0x43FF if ((mmio_address >> 4) & 0xF) < 8 => {
                self.write_dma_regs(mmio_address, data);
            }

            _ => {
                // if mmio_address != 0x2180 {
                // println!(" ==== Attempt to write mmio reg ${mmio_address:04X} with data 0x{data:02X}");
                // }
            }
        }
    }

    fn read_dma_regs(&mut self, reg_address: u16) -> u8 {
        let channel_idx = ((reg_address >> 4) & 7) as usize;
        let reg_address = reg_address & 0xFF0F;

        let dma_channel = &self.dma_channels[channel_idx];

        match reg_address {
            0x4300 => dma_channel.params_raw,
            0x4301 => dma_channel.b_bus_addr,
            0x4302 => dma_channel.a_bus_lo,
            0x4303 => dma_channel.a_bus_hi,
            0x4304 => dma_channel.a_bus_bank,
            // 0x4305 => self.dma_byte_count_lo[channel_idx],
            // 0x4306 => self.dma_byte_count_hi[channel_idx],
            // 0x4307 => self.hdma_indirect_addr_bank[channel_idx],
            // 0x4308 => self.hdma_table_addr_lo[channel_idx],
            // 0x4309 => self.hdma_table_addr_hi[channel_idx],
            // 0x430A => self.hdma_line_counter[channel_idx],
            // 0x430B => self.dma_unused1[channel_idx],
            // 0x430F => self.dma_unused2[channel_idx],
            _ => 0,
        }
    }

    fn write_dma_regs(&mut self, reg_address: u16, data: u8) {
        let channel_idx = ((reg_address >> 4) & 7) as usize;
        let reg_address = reg_address & 0xFF0F;

        let dma_channel = &mut self.dma_channels[channel_idx];

        // println!("Write to DMA reg ${:02X} (ch {}) with 0x{:02X}", reg_address, channel_idx, data);

        match reg_address {
            0x4300 => {
                dma_channel.params_raw = data;
                dma_channel.transfer_pattern = match data & 7 {
                    0 => dma::TransferPattern::Pattern0,
                    1 => dma::TransferPattern::Pattern1,
                    2 => dma::TransferPattern::Pattern2,
                    3 => dma::TransferPattern::Pattern3,
                    4 => dma::TransferPattern::Pattern4,
                    5 => dma::TransferPattern::Pattern5,
                    6 => dma::TransferPattern::Pattern6,
                    _ => dma::TransferPattern::Pattern7,
                };
                dma_channel.inc_mode = match (data >> 3) & 3 {
                    0 => dma::AddressIncMode::Inc,
                    2 => dma::AddressIncMode::Dec,
                    _ => dma::AddressIncMode::Fixed,
                };
                dma_channel.indirect = (data & 0x40) != 0;
                dma_channel.direction = match data >> 7 {
                    0 => dma::Direction::AtoB,
                    _ => dma::Direction::BtoA,
                };

                // println!("Wrote 0x{data:02X} to DMA channel {channel_idx} Params (Transfer Pattern = {:?}, A Inc Mode = {:?}, Indirect = {}, Direction = {:?})",
                //     dma_channel.transfer_pattern,
                //     dma_channel.inc_mode,
                //     dma_channel.indirect,
                //     dma_channel.direction,
                // );
            }
            0x4301 => {
                dma_channel.b_bus_addr = data;
                // println!("Wrote 0x{data:02X} to DMA channel {channel_idx} B-Bus Address");
            }
            0x4302 => {
                dma_channel.a_bus_lo = data;
                // println!("Wrote 0x{data:02X} to DMA channel {channel_idx} A-Bus Address Low");
            }
            0x4303 => {
                dma_channel.a_bus_hi = data;
                // println!("Wrote 0x{data:02X} to DMA channel {channel_idx} A-Bus Address Hi");
            }
            0x4304 => {
                dma_channel.a_bus_bank = data;
                // println!("Wrote 0x{data:02X} to DMA channel {channel_idx} A-Bus Address Bank");
            }
            0x4305 => {
                dma_channel.byte_count = (dma_channel.byte_count & 0xFF00) | (data as u16);
                // println!("Wrote 0x{data:02X} to DMA channel {channel_idx} Byte Count Low");
            }
            0x4306 => {
                dma_channel.byte_count = (dma_channel.byte_count & 0x00FF) | ((data as u16) << 8);
                // println!("Wrote 0x{data:02X} to DMA channel {channel_idx} Byte Count Hi");
            }
            // 0x4307 => { self.hdma_indirect_addr_bank[channel_idx] = data; }
            // 0x4308 => { self.hdma_table_addr_lo[channel_idx] = data; }
            // 0x4309 => { self.hdma_table_addr_hi[channel_idx] = data; }
            // 0x430A => { self.hdma_line_counter[channel_idx] = data; }
            // 0x430B => { self.dma_unused1[channel_idx] = data; }
            // 0x430F => { self.dma_unused2[channel_idx] = data; }
            _ => {}
        }
    }

    fn map_lorom_addr(&self, address: u32) -> u32 {
        ((address & 0x7F0000) >> 1) | (address & 0x007FFF)
    }

    fn map_hirom_addr(&self, address: u32) -> u32 {
        address & 0x3FFFFF
    }

    fn map_exhirom_addr(&self, address: u32) -> u32 {
        (((address & 0x800000) ^ 0x800000) >> 1) | (address & 0x3FFFFF)
    }

    /// Read from ROM (or SRAM) in LoROM mapping mode
    /// Memory map diagram here: https://snes.nesdev.org/wiki/Memory_map#LoROM
    fn read_lorom(&mut self, address: u32) -> (u8, usize) {
        if 0xF0 <= (address.bank() | 0x80) && address.bank_addr() <= 0x7FFF && self.has_sram {
            todo!("Read SRAM");
        }

        let mirror_addr = if (address & 0x00FFFF) >= 0x8000 {
            address | 0x800000
        } else {
            (address & 0xBFFFFF) | 0x008000
        };

        // 0 if mapped_addr > rom.len() ? or mirror ?
        let mapped_addr = (self.map_lorom_addr(mirror_addr) as usize) & self.rom_mirror;

        let data = self.rom[mapped_addr];

        let clocks = if address.bank() >= 0x80 {
            match self.mem_sel {
                MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
            }
        } else {
            Cpu65c816::ONE_CYCLE_SLOW
        };

        (data, clocks)
    }

    /// Write to SRAM (ROM writes are ignored but still take cycles)
    /// Memory map diagram here: https://snes.nesdev.org/wiki/Memory_map#LoROM
    fn write_lorom(&mut self, address: u32, data: u8) -> usize {
        if 0xF0 <= (address.bank() | 0x80) && address.bank_addr() <= 0x7FFF && self.has_sram {
            todo!("Write SRAM");
        }

        let clocks = if address.bank() >= 0x80 {
            match self.mem_sel {
                MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
            }
        } else {
            Cpu65c816::ONE_CYCLE_SLOW
        };

        clocks
    }

    fn read_hirom(&mut self, address: u32) -> (u8, usize) {
        if (address.bank() & 0x7F) < 0x80 && 0x6000 <= address.bank_addr()
            && address.bank_addr() <= 0x7FFF && self.has_sram {

            let sram_addr = (address - 0x6000) & 0xFFFF;
            todo!("Read SRAM");
        }

        let mirror_addr = address | 0xC00000;

        // 0 if mapped_addr > rom.len() ? or mirror ?
        let mapped_addr = (self.map_hirom_addr(mirror_addr) as usize) & self.rom_mirror;

        let data = self.rom[mapped_addr];

        let clocks = if address.bank() >= 0x80 {
            match self.mem_sel {
                MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
            }
        } else {
            Cpu65c816::ONE_CYCLE_SLOW
        };

        (data, clocks)
    }

    fn write_hirom(&mut self, address: u32, data: u8) -> usize {
        if (address.bank() & 0x7F) < 0x80 && 0x6000 <= address.bank_addr()
            && address.bank_addr() <= 0x7FFF && self.has_sram {

            let sram_addr = (address - 0x6000) & 0xFFFF;
            todo!("Read SRAM");
        }

        let clocks = if address.bank() >= 0x80 {
            match self.mem_sel {
                MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
            }
        } else {
            Cpu65c816::ONE_CYCLE_SLOW
        };

        clocks
    }

    fn read_exhirom(&mut self, address: u32) -> (u8, usize) {
        let data: u8;
        let clocks: usize;

        match (address.bank(), address.bank_addr()) {
            // Lower half of ROM (stored in the upper part of memory, hence ExHiROM).
            // Fast ROM region.
            (bank @ 0xC0..=0xFF, bank_addr) => {
                data = self.rom[(address as usize) & self.rom_mirror];
                clocks = match self.mem_sel {
                    MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
                    MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                };
            }
            // Upper half of ROM (stored lower in memory: bank 0x40 directly follows bank 0xFF).
            // Slow ROM region.
            (bank @ 0x40..=0x7D, bank_addr) => {
                let addr = (address as usize + 0x40000) & self.rom_mirror;
                data = self.rom[addr];
                clocks = Cpu65c816::ONE_CYCLE_SLOW;
            }
            // Very last bit of ROM - and of course, it's the lowest in memory.
            // Only fills the upper half of banks 3E and 3F.
            (bank @ 0x3E..=0x3F, bank_addr @ 0x8000..=0xFFFF) => {
                let addr = (address as usize + 0x80000) & self.rom_mirror;
                data = self.rom[addr];
                clocks = Cpu65c816::ONE_CYCLE_SLOW;
            }
            // Mirror of ROM banks 0xC0-0xFF.
            // Fast ROM region.
            (bank @ 0x80..=0xBF, bank_addr @ 0x8000..=0xFFFF) => {
                let addr = (bank as usize + 0x40) << 8 | bank_addr as usize;
                data = self.rom[addr & self.rom_mirror];

                clocks = match self.mem_sel {
                    MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
                    MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                };
            }
            // Mirror of ROM banks 0x40-0x7D
            (bank @ 0x00..=0x3D, bank_addr @ 0x8000..=0xFFFF) => {
                let addr = (bank as usize + 0x40) << 8 | bank_addr as usize;
                data = self.rom[addr & self.rom_mirror];

                clocks = match self.mem_sel {
                    MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
                    MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                };
            }
            // LoRAM mirrors
            (bank @ 0x00..=0x3F, bank_addr @ 0x0000..=0x1FFF)
            | (bank @ 0x80..=0xBF, bank_addr @ 0x0000..=0x1FFF) => {
                data = self.wram[bank_addr as usize];
                clocks = Cpu65c816::ONE_CYCLE_SLOW;
            }
            // Save RAM
            (bank @ 0x80..=0xBF, bank_addr @ 0x6000..=0x7FFF) => {
                data = 0;
                clocks = 0;
                // TODO: implement SRAM
            }
            _ => {
                data = 0;
                clocks = 0;
            }
        }

        (data, clocks)
    }

    fn write_exhirom(&mut self, address: u32, data: u8) -> usize {
        let clocks: usize;

        match (address.bank(), address.bank_addr()) {
            // Lower half of ROM (stored in the upper part of memory, hence ExHiROM).
            // Fast ROM region.
            (bank @ 0xC0..=0xFF, bank_addr) => {
                clocks = match self.mem_sel {
                    MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
                    MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                };
            }
            // Upper half of ROM (stored lower in memory: bank 0x40 directly follows bank 0xFF).
            // Slow ROM region.
            (bank @ 0x40..=0x7D, bank_addr) => {
                clocks = Cpu65c816::ONE_CYCLE_SLOW;
            }
            // Very last bit of ROM - and of course, it's the lowest in memory.
            // Only fills the upper half of banks 3E and 3F.
            (bank @ 0x3E..=0x3F, bank_addr @ 0x8000..=0xFFFF) => {
                clocks = Cpu65c816::ONE_CYCLE_SLOW;
            }
            // Mirror of ROM banks 0xC0-0xFF.
            // Fast ROM region.
            (bank @ 0x80..=0xBF, bank_addr @ 0x8000..=0xFFFF) => {
                clocks = match self.mem_sel {
                    MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
                    MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                };
            }
            // Mirror of ROM banks 0x40-0x7D
            (bank @ 0x00..=0x3D, bank_addr @ 0x8000..=0xFFFF) => {
                clocks = match self.mem_sel {
                    MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
                    MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                };
            }
            // LoRAM mirrors
            (bank @ 0x00..=0x3F, bank_addr @ 0x0000..=0x1FFF)
            | (bank @ 0x80..=0xBF, bank_addr @ 0x0000..=0x1FFF) => {
                self.wram[bank_addr as usize] = data;
                clocks = Cpu65c816::ONE_CYCLE_SLOW;
            }
            // Save RAM
            (bank @ 0x80..=0xBF, bank_addr @ 0x6000..=0x7FFF) => {
                if self.has_sram {
                    todo!("implement SRAM");
                }
                clocks = 0;
            }
            _ => {
                clocks = 0;
            }
        }

        clocks
    }

    fn read_prg(&mut self) -> u8 {
        self.read(((self.prg_bank as u32) << 16) | (self.pc as u32))
    }
    fn read16(&mut self, address_lo: u32, address_hi: u32) -> u16 {
        u16::from_le_bytes([self.read(address_lo), self.read(address_hi)])
    }
    fn write16(&mut self, address_lo: u32, address_hi: u32, data: u16) {
        self.write(address_lo, data as u8);
        self.write(address_hi, (data >> 8) as u8);
    }

    fn pop8_n(&mut self) -> u8 {
        self.stk_ptr += 1;
        self.read(self.stk_ptr as u32)
    }
    fn pop16_n(&mut self) -> u16 {
        u16::from_le_bytes([self.pop8_n(), self.pop8_n()])
    }
    fn pop8_e(&mut self) -> u8 {
        self.stk_ptr = inc_low_byte(self.stk_ptr);
        self.read(self.stk_ptr as u32)
    }
    fn pop16_e(&mut self) -> u16 {
        u16::from_le_bytes([self.pop8_e(), self.pop8_e()])
    }

    fn push8_n(&mut self, data: u8) {
        self.write(self.stk_ptr as u32, data);
        self.stk_ptr -= 1;
    }
    fn push16_n(&mut self, data: u16) {
        self.push8_n((data >> 8) as u8);
        self.push8_n(data as u8);
    }
    fn push8_e(&mut self, data: u8) {
        self.write(self.stk_ptr as u32, data);
        self.stk_ptr = dec_low_byte(self.stk_ptr);
    }
    fn push16_e(&mut self, data: u16) {
        self.push8_e((data >> 8) as u8);
        self.push8_e(data as u8);
    }

    fn is_flag_set(&self, flag: Flag) -> bool {
        (self.status & flag as u8) != 0
    }
    fn set_flag(&mut self, flag: Flag) {
        self.status |= flag as u8;
    }
    fn clear_flag(&mut self, flag: Flag) {
        self.status &= !(flag as u8);
    }
    fn set_flag_to_bool(&mut self, flag: Flag, val: bool) {
        if val {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    fn get_acc_hi(&self) -> u8 {
        (self.acc >> 8) as u8
    }
    fn get_acc_lo(&self) -> u8 {
        self.acc as u8
    }
    fn set_acc_hi(&mut self, val: u8) {
        self.acc = ((val as u16) << 8) | (self.acc & 0x00FF);
    }
    fn set_acc_lo(&mut self, val: u8) {
        self.acc = (self.acc & 0xFF00) | val as u16;
    }

    fn get_x_hi(&self) -> u8 {
        (self.x >> 8) as u8
    }
    fn get_x_lo(&self) -> u8 {
        self.x as u8
    }
    fn set_x_hi(&mut self, val: u8) {
        self.x = ((val as u16) << 8) | (self.x & 0x00FF);
    }
    fn set_x_lo(&mut self, val: u8) {
        self.x = (self.x & 0xFF00) | val as u16;
    }

    fn get_y_hi(&self) -> u8 {
        (self.y >> 8) as u8
    }
    fn get_y_lo(&self) -> u8 {
        self.y as u8
    }
    fn set_y_hi(&mut self, val: u8) {
        self.y = ((val as u16) << 8) | (self.y & 0x00FF);
    }
    fn set_y_lo(&mut self, val: u8) {
        self.y = (self.y & 0xFF00) | val as u16;
    }

    fn acc_size(&self) -> RegSize {
        if self.is_flag_set(Flag::FlagM) {
            RegSize::Byte
        } else {
            RegSize::TwoBytes
        }
    }

    fn idx_size(&self) -> RegSize {
        if self.is_flag_set(Flag::FlagX) {
            RegSize::Byte
        } else {
            RegSize::TwoBytes
        }
    }

    fn set_mode(&mut self, mode: CpuMode) {
        self.mode = mode;

        match mode {
            CpuMode::Native => {}

            CpuMode::Emulation => {
                self.set_flag(Flag::FlagM);
                self.set_flag(Flag::FlagX);

                self.x &= 0x00FF;
                self.y &= 0x00FF;
                self.stk_ptr = 0x100 | (self.stk_ptr & 0x00FF);
            }
        }
    }

    pub fn trigger_interrupt(&mut self, interrupt: CpuInterrupt) {
        if interrupt == CpuInterrupt::Reset {
            self.set_mode(CpuMode::Emulation);
        }

        let vector_lo: u32;
        let vector_hi: u32;

        match self.mode {
            CpuMode::Native => {
                self.push8_n(self.prg_bank);
                self.push16_n(self.pc);
                self.push8_n(self.status);

                (vector_lo, vector_hi) = match interrupt {
                    CpuInterrupt::IRQ => (0x00FFEE, 0x00FFEF),
                    CpuInterrupt::NMI => (0x00FFEA, 0x00FFEB),
                    CpuInterrupt::Abort => (0x00FFE8, 0x00FFE9),
                    _ => {
                        unreachable!()
                    } // reset sets mode to emulation
                }
            }

            CpuMode::Emulation => {
                self.push16_e(self.pc);
                self.push8_e(self.status);

                (vector_lo, vector_hi) = match interrupt {
                    CpuInterrupt::IRQ => (0x00FFFE, 0x00FFFF),
                    CpuInterrupt::NMI => (0x00FFFA, 0x00FFFB),
                    CpuInterrupt::Reset => (0x00FFFC, 0x00FFFD),
                    CpuInterrupt::Abort => (0x00FFF8, 0x00FFF9),
                }
            }
        }

        self.pc = self.read16(vector_lo, vector_hi);
        self.add_clocks(Cpu65c816::ONE_CYCLE);
    }
}

// Addressing Modes
impl Cpu65c816 {
    fn absolute8(&mut self) -> u32 {
        let (lo, hi) = self.immediate16();
        let address_lo = self.read(lo);
        let address_hi = self.read(hi);
        u32::from_parts(self.data_bank, address_hi, address_lo)
    }
    fn absolute16(&mut self) -> (u32, u32) {
        let address_lo = self.absolute8();
        (address_lo, (address_lo + 1) & 0xFFFFFF)
    }

    fn absolute_long8(&mut self) -> u32 {
        let (lo, mi) = self.immediate16();
        let hi = (mi + 1).with_bank(mi.bank());
        let address_lo = self.read(lo);
        let address_mi = self.read(mi);
        let address_hi = self.read(hi);
        u32::from_parts(address_hi, address_mi, address_lo)
    }
    fn absolute_long16(&mut self) -> (u32, u32) {
        let (lo, mi) = self.immediate16();
        let hi = (mi + 1).with_bank(mi.bank());
        let address_lo = self.read(lo);
        let address_mi = self.read(mi);
        let address_hi = self.read(hi);

        let addr = u32::from_parts(address_hi, address_mi, address_lo);
        (addr, (addr + 1) & 0xFFFFFF)
    }

    fn absolute_long_x8(&mut self) -> u32 {
        (self.absolute_long8() + self.x as u32) & 0xFFFFFF
    }
    fn absolute_long_x16(&mut self) -> (u32, u32) {
        let (address_lo, address_hi) = self.absolute_long16();
        (
            (address_lo + self.x as u32) & 0xFFFFFF,
            (address_hi + self.x as u32) & 0xFFFFFF,
        )
    }

    fn absolute_x8(&mut self) -> u32 {
        let original_addr = self.absolute8();
        let indexed_addr = (original_addr + self.x as u32) & 0xFFFFFF;

        self.page_crossed = original_addr.page() != indexed_addr.page();

        indexed_addr
    }
    fn absolute_x16(&mut self) -> (u32, u32) {
        let address_lo = self.absolute_x8();
        (address_lo, (address_lo + 1) & 0xFFFFFF)
    }

    fn absolute_y8(&mut self) -> u32 {
        let original_addr = self.absolute8();
        let indexed_addr = (original_addr + self.y as u32) & 0xFFFFFF;

        self.page_crossed = original_addr.page() != indexed_addr.page();

        indexed_addr
    }
    fn absolute_y16(&mut self) -> (u32, u32) {
        let address_lo = self.absolute_y8();
        (address_lo, (address_lo + 1) & 0xFFFFFF)
    }

    fn absolute_indirect(&mut self) -> u32 {
        let (ptr_lo, ptr_hi) = self.absolute16();
        let address_lo = self.read(ptr_lo.with_bank(0));
        let address_hi = self.read(ptr_hi.with_bank(0));
        u32::from_parts(self.prg_bank, address_hi, address_lo)
    }
    fn absolute_indirect_long(&mut self) -> u32 {
        let (ptr_lo, ptr_mi) = self.absolute16();
        let ptr_hi = ptr_mi + 1;
        let address_lo = self.read(ptr_lo.with_bank(0));
        let address_mi = self.read(ptr_mi.with_bank(0));
        let address_hi = self.read(ptr_hi.with_bank(0));
        u32::from_parts(address_hi, address_mi, address_lo)
    }

    fn absolute_x_indirect8(&mut self) -> u32 {
        let ptr_lo = self.absolute_x8().with_bank(self.prg_bank);
        let ptr_hi = (ptr_lo + 1).with_bank(self.prg_bank); // Wrap on bank
        let address_lo = self.read(ptr_lo);
        let address_hi = self.read(ptr_hi);
        u32::from_parts(self.prg_bank, address_hi, address_lo)
    }

    fn immediate8(&self) -> u32 {
        ((self.prg_bank as u32) << 16) | (self.pc + 1) as u32
    }
    fn immediate16(&self) -> (u32, u32) {
        (
            ((self.prg_bank as u32) << 16) | (self.pc + 1) as u32,
            ((self.prg_bank as u32) << 16) | (self.pc + 2) as u32,
        )
    }

    fn direct8(&mut self) -> u32 {
        // Direct addressing takes an extra cycle when DL != 0
        if self.direct_page & 0xFF != 0 {
            self.add_clocks(Cpu65c816::ONE_CYCLE);
        }

        (self.direct_page + self.read(self.immediate8()) as u16) as u32
    }
    fn direct16(&mut self) -> (u32, u32) {
        let direct = self.direct8();
        ((direct) as u32, (direct + 1) as u32)
    }

    fn direct_x8(&mut self) -> u32 {
        match self.mode {
            CpuMode::Emulation => {
                let addr = self.direct8();

                if self.direct_page & 0xFF == 0 {
                    addr.with_page_addr(addr.page_addr() + self.get_x_lo())
                } else {
                    (addr + self.x as u32).with_bank(0)
                }
            }

            CpuMode::Native => (self.direct8() + self.x as u32).with_bank(0),
        }
    }
    fn direct_x16(&mut self) -> (u32, u32) {
        let addr = (self.direct8() + self.x as u32).with_bank(0);
        (addr, (addr + 1).with_bank(0))
    }

    fn direct_y8(&mut self) -> u32 {
        match self.mode {
            CpuMode::Emulation => {
                let addr = self.direct8();

                if self.direct_page & 0xFF == 0 {
                    addr.with_page_addr(addr.page_addr() + self.get_y_lo())
                } else {
                    (addr + self.y as u32).with_bank(0)
                }
            }

            CpuMode::Native => (self.direct8() + self.y as u32).with_bank(0),
        }
    }
    fn direct_y16(&mut self) -> (u32, u32) {
        let addr = (self.direct8() + self.y as u32).with_bank(0);
        (addr, (addr + 1).with_bank(0))
    }

    fn direct_indirect8(&mut self) -> u32 {
        let ptr_lo = self.direct8();
        let ptr_hi = match self.mode {
            CpuMode::Native => (ptr_lo + 1).with_bank(0),
            CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
        };

        u32::from_parts(self.data_bank, self.read(ptr_hi), self.read(ptr_lo))
    }
    fn direct_indirect16(&mut self) -> (u32, u32) {
        let ptr_lo = self.direct8();
        let ptr_hi = (ptr_lo + 1).with_bank(0);

        let address_lo = u32::from_parts(self.data_bank, self.read(ptr_hi), self.read(ptr_lo));
        let address_hi = (address_lo + 1) & 0xFFFFFF;

        (address_lo, address_hi)
    }

    fn direct_indirect_long8(&mut self) -> u32 {
        let ptr_lo = self.direct8();
        let ptr_mi = (ptr_lo + 1).with_bank(0);
        let ptr_hi = (ptr_lo + 2).with_bank(0);

        u32::from_parts(self.read(ptr_hi), self.read(ptr_mi), self.read(ptr_lo))
    }
    fn direct_indirect_long16(&mut self) -> (u32, u32) {
        let ptr_lo = self.direct8();
        let ptr_mi = (ptr_lo + 1).with_bank(0);
        let ptr_hi = (ptr_lo + 2).with_bank(0);

        let address_lo = u32::from_parts(self.read(ptr_hi), self.read(ptr_mi), self.read(ptr_lo));
        let address_hi = (address_lo + 1) & 0xFFFFFF;

        (address_lo, address_hi)
    }

    fn direct_x_indirect8(&mut self) -> u32 {
        let ptr_lo = self.direct_x8();
        let ptr_hi = match self.mode {
            CpuMode::Native => (ptr_lo + 1).with_bank(0),
            CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
        };

        let address_hi = self.read(ptr_hi);
        let address_lo = self.read(ptr_lo);

        u32::from_parts(self.data_bank, address_hi, address_lo)
    }
    fn direct_x_indirect16(&mut self) -> (u32, u32) {
        let ptr_lo = self.direct_x8();
        let ptr_hi = match self.mode {
            CpuMode::Native => (ptr_lo + 1).with_bank(0),
            CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
        };

        let address_hi = self.read(ptr_hi);
        let address_lo = self.read(ptr_lo);

        let addr = u32::from_parts(self.data_bank, address_hi, address_lo);

        (addr, (addr + 1) & 0xFFFFFF)
    }

    fn direct_indirect_y8(&mut self) -> u32 {
        let ptr_lo = self.direct8();
        let ptr_hi = match self.mode {
            CpuMode::Native => (ptr_lo + 1).with_bank(0),
            CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
        };

        let original_addr = u32::from_parts(self.data_bank, self.read(ptr_hi), self.read(ptr_lo));
        let indexed_addr = (original_addr + self.y as u32) & 0xFFFFFF;

        self.page_crossed = original_addr.page() != indexed_addr.page();

        indexed_addr
    }
    fn direct_indirect_y16(&mut self) -> (u32, u32) {
        let ptr_lo = self.direct8();
        let ptr_hi = match self.mode {
            CpuMode::Native => (ptr_lo + 1).with_bank(0),
            CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
        };

        let addr = (u32::from_parts(self.data_bank, self.read(ptr_hi), self.read(ptr_lo))
            + self.y as u32)
            & 0xFFFFFF;

        (addr, (addr + 1) & 0xFFFFFF)
    }

    fn direct_indirect_long_y8(&mut self) -> u32 {
        let ptr_lo = self.direct8();
        let ptr_mi = (ptr_lo + 1).with_bank(0);
        let ptr_hi = (ptr_lo + 2).with_bank(0);

        (u32::from_parts(self.read(ptr_hi), self.read(ptr_mi), self.read(ptr_lo)) + self.y as u32)
            & 0xFFFFFF
    }
    fn direct_indirect_long_y16(&mut self) -> (u32, u32) {
        let ptr_lo = self.direct8();
        let ptr_mi = (ptr_lo + 1).with_bank(0);
        let ptr_hi = (ptr_lo + 2).with_bank(0);

        let addr = (u32::from_parts(self.read(ptr_hi), self.read(ptr_mi), self.read(ptr_lo))
            + self.y as u32)
            & 0xFFFFFF;

        (addr, (addr + 1) & 0xFFFFFF)
    }

    fn relative8(&mut self) -> u32 {
        let offset = (self.read(self.immediate8()) as i8) as u16;
        let original_addr = ((self.pc + 2) as u32).with_bank(self.prg_bank);
        let offset_addr = original_addr.with_bank_addr(original_addr.bank_addr() + offset);

        self.page_crossed = original_addr.page() != offset_addr.page();

        offset_addr
    }
    fn relative16(&mut self) -> u32 {
        let (offset_lo, offset_hi) = self.immediate16();
        let offset = self.read16(offset_lo, offset_hi);
        ((self.pc + offset + 3) as u32).with_bank(self.prg_bank)
    }

    fn src_dst(&mut self) -> (u32, u32) {
        let (address_src, address_dst) = self.immediate16();

        let src_bank = self.read(address_src);
        let src = (self.x as u32).with_bank(src_bank);

        let dst_bank = self.read(address_dst);
        let dst = (self.y as u32).with_bank(dst_bank);

        (src, dst)
    }

    fn stack_s8(&mut self) -> u32 {
        let val = self.read(self.immediate8()) as u16;

        (val + self.stk_ptr) as u32
    }
    fn stack_s16(&mut self) -> (u32, u32) {
        let val = self.read(self.immediate8()) as u16;

        ((val + self.stk_ptr) as u32, (val + self.stk_ptr + 1) as u32)
    }

    fn stack_indirect_y8(&mut self) -> u32 {
        let (ptr_lo, ptr_hi) = self.stack_s16();

        let address_lo = self.read(ptr_lo);
        let address_hi = self.read(ptr_hi);

        let addr = u32::from_parts(self.data_bank, address_hi, address_lo);

        (addr + self.y as u32) & 0xFFFFFF
    }
    fn stack_indirect_y16(&mut self) -> (u32, u32) {
        let (ptr_lo, ptr_hi) = self.stack_s16();

        let address_lo = self.read(ptr_lo);
        let address_hi = self.read(ptr_hi);

        let addr = u32::from_parts(self.data_bank, address_hi, address_lo);

        (
            (addr + self.y as u32) & 0xFFFFFF,
            (addr + self.y as u32 + 1) & 0xFFFFFF,
        )
    }
}

// Instructions
impl Cpu65c816 {
    fn adc_m8(&mut self, address: u32) {
        let data = self.read(address);
        let mut result: u16;
        let a = self.acc & 0xFF;
        let d = data as u16;
        let c = bool2byte!(self.is_flag_set(Flag::FlagC));

        if self.is_flag_set(Flag::FlagD) {
            result = (a & 0x0F) + (d & 0x0F) + c;

            if result >= 0xA {
                result += 0x6;
            }

            let c = if result > 0xF { 0x10 } else { 0 };
            result = (a & 0xF0) + (d & 0xF0) + c + (result & 0xF);
        } else {
            result = a + d + c;
        };

        self.set_flag_to_bool(Flag::FlagV, !(a ^ d) & (d ^ result) & 0x80 != 0);

        if self.is_flag_set(Flag::FlagD) && result >= 0xA0 {
            result += 0x60;
        }

        self.set_flag_to_bool(Flag::FlagC, result > 0xFF);

        let result = result as u8;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.set_acc_lo(result);
    }

    fn adc_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let mut result: u32;
        let a = self.acc as u32;
        let d = data as u32;
        let c = bool2byte!(self.is_flag_set(Flag::FlagC));

        if self.is_flag_set(Flag::FlagD) {
            result = (a & 0x000F) + (d & 0x000F) + c;

            if result >= 0xA {
                result += 6;
            }

            let c = if result > 0xF { 0x10 } else { 0 };
            result = (a & 0x00F0) + (d & 0x00F0) + c + (result & 0xF);

            if result >= 0xA0 {
                result += 0x60;
            }

            let c = if result > 0xFF { 0x100 } else { 0 };
            result = (a & 0x0F00) + (d & 0x0F00) + c + (result & 0xFF);

            if result >= 0xA00 {
                result += 0x600;
            }

            let c = if result > 0xFFF { 0x1000 } else { 0 };
            result = (a & 0xF000) + (d & 0xF000) + c + (result & 0xFFF);
        } else {
            result = a + d + c;
        }

        self.set_flag_to_bool(
            Flag::FlagV,
            !(self.acc ^ data) & (data ^ (result as u16)) & 0x8000 != 0,
        );

        if self.is_flag_set(Flag::FlagD) && result >= 0xA000 {
            result += 0x6000;
        }

        self.set_flag_to_bool(Flag::FlagC, result > 0xFFFF);

        let result = result as u16;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(15));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.acc = result;
    }

    fn and_m8(&mut self, address: u32) {
        let result = self.get_acc_lo() & self.read(address);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.set_acc_lo(result);
    }

    fn and_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let result = self.acc & self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.acc = result;
    }

    fn asl_acc_m8(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.get_acc_lo() & 0x80 != 0);

        self.set_acc_lo(self.get_acc_lo() << 1);

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }

    fn asl_acc_m16(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc & 0x8000 != 0);

        self.acc <<= 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn asl_mem_m8(&mut self, address: u32) {
        let data = self.read(address);
        let result = data << 1;

        self.set_flag_to_bool(Flag::FlagC, data & 0x80 != 0);

        self.write(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn asl_mem_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = data << 1;

        self.set_flag_to_bool(Flag::FlagC, data & 0x8000 != 0);

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn bcc_all(&mut self, address: u32) {
        if !self.is_flag_set(Flag::FlagC) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bcs_all(&mut self, address: u32) {
        if self.is_flag_set(Flag::FlagC) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn beq_all(&mut self, address: u32) {
        if self.is_flag_set(Flag::FlagZ) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bit_m8(&mut self, address: u32) {
        let data = self.read(address);
        let result = self.get_acc_lo() & data;

        self.set_flag_to_bool(Flag::FlagN, data & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagV, data & 0x40 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn bit_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = self.acc & data;

        self.set_flag_to_bool(Flag::FlagN, data & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagV, data & 0x4000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn bit_imm_m8(&mut self, address: u32) {
        let data = self.read(address);
        let result = self.get_acc_lo() & data;

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn bit_imm_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = self.acc & data;

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn bmi_all(&mut self, address: u32) {
        if self.is_flag_set(Flag::FlagN) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bne_all(&mut self, address: u32) {
        if !self.is_flag_set(Flag::FlagZ) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bpl_all(&mut self, address: u32) {
        if !self.is_flag_set(Flag::FlagN) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bra_all(&mut self, address: u32) {
        self.pc = address.bank_addr();
        self.branch_taken = true;
    }

    fn brk_n(&mut self) {
        self.push8_n(self.prg_bank);
        self.push16_n(self.pc + 2); // push the address of the brk instruction + 2
        self.push8_n(self.status);
        self.set_flag(Flag::FlagI);
        self.clear_flag(Flag::FlagD);

        const N_BRK_VECTOR_LO: u32 = 0x00FFE6;
        const N_BRK_VECTOR_HI: u32 = 0x00FFE7;

        self.pc = self.read16(N_BRK_VECTOR_LO, N_BRK_VECTOR_HI);
    }
    fn brk_e(&mut self) {
        self.push16_e(self.pc + 2); // push the address of the brk instruction + 2
        self.push8_e(self.status | Flag::FlagX as u8); // Pushes status to the stack with B flag (same place as X flag) set
        self.set_flag(Flag::FlagI);
        self.clear_flag(Flag::FlagD);

        const E_BRK_VECTOR_LO: u32 = 0x00FFFE;
        const E_BRK_VECTOR_HI: u32 = 0x00FFFF;

        self.pc = self.read16(E_BRK_VECTOR_LO, E_BRK_VECTOR_HI);
    }

    fn bvc_all(&mut self, address: u32) {
        if !self.is_flag_set(Flag::FlagV) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bvs_all(&mut self, address: u32) {
        if self.is_flag_set(Flag::FlagV) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn clc_all(&mut self) {
        self.clear_flag(Flag::FlagC);
    }

    fn cld_all(&mut self) {
        self.clear_flag(Flag::FlagD);
    }

    fn cli_all(&mut self) {
        self.clear_flag(Flag::FlagI);
    }

    fn clv_all(&mut self) {
        self.clear_flag(Flag::FlagV);
    }

    fn cmp_m8(&mut self, address: u32) {
        let data = self.read(address);
        let result = self.get_acc_lo() - data;

        self.set_flag_to_bool(Flag::FlagC, self.get_acc_lo() >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn cmp_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = self.acc - data;

        self.set_flag_to_bool(Flag::FlagC, self.acc >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn cop_n(&mut self, address: u32) {
        let _ = self.read(address); // read is discarded here

        self.push8_n(self.prg_bank);
        self.push16_n(self.pc + 2); // push the address of the COP instruction + 2
        self.push8_n(self.status);
        self.set_flag(Flag::FlagI);
        self.clear_flag(Flag::FlagD);

        const N_COP_VECTOR_LO: u32 = 0x00FFE4;
        const N_COP_VECTOR_HI: u32 = 0x00FFE5;

        self.pc = self.read16(N_COP_VECTOR_LO, N_COP_VECTOR_HI);
    }
    fn cop_e(&mut self, address: u32) {
        let _ = self.read(address); // read is discarded here

        self.push16_e(self.pc + 2); // push the address of the COP instruction + 2
        self.push8_e(self.status);
        self.set_flag(Flag::FlagI);
        self.clear_flag(Flag::FlagD);

        const E_COP_VECTOR_LO: u32 = 0x00FFF4;
        const E_COP_VECTOR_HI: u32 = 0x00FFF5;

        self.pc = self.read16(E_COP_VECTOR_LO, E_COP_VECTOR_HI);
    }

    fn cpx_x8(&mut self, address: u32) {
        let data = self.read(address);
        let result = self.get_x_lo() - data;

        self.set_flag_to_bool(Flag::FlagC, self.get_x_lo() >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn cpx_x16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = self.x - data;

        self.set_flag_to_bool(Flag::FlagC, self.x >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn cpy_x8(&mut self, address: u32) {
        let data = self.read(address);
        let result = self.get_y_lo() - data;

        self.set_flag_to_bool(Flag::FlagC, self.get_y_lo() >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn cpy_x16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = self.y - data;

        self.set_flag_to_bool(Flag::FlagC, self.y >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn dec_acc_m8(&mut self) {
        self.acc = dec_low_byte(self.acc);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn dec_acc_m16(&mut self) {
        self.acc -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn dec_mem_m8(&mut self, address: u32) {
        let result = self.read(address) - 1;

        self.write(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn dec_mem_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let result = self.read16(address_lo, address_hi) - 1;

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn dex_x8(&mut self) {
        self.x = dec_low_byte(self.x);

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn dex_x16(&mut self) {
        self.x -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn dey_x8(&mut self) {
        self.y = dec_low_byte(self.y);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn dey_x16(&mut self) {
        self.y -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn eor_m8(&mut self, address: u32) {
        let result = self.get_acc_lo() ^ self.read(address);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.set_acc_lo(result);
    }
    fn eor_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let result = self.acc ^ self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.acc = result;
    }

    fn inc_acc_m8(&mut self) {
        self.acc = inc_low_byte(self.acc);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn inc_acc_m16(&mut self) {
        self.acc += 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn inc_mem_m8(&mut self, address: u32) {
        let result = self.read(address) + 1;

        self.write(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn inc_mem_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let result = self.read16(address_lo, address_hi) + 1;

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn inx_x8(&mut self) {
        self.x = inc_low_byte(self.x);

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn inx_x16(&mut self) {
        self.x += 1;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn iny_x8(&mut self) {
        self.y = inc_low_byte(self.y);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn iny_x16(&mut self) {
        self.y += 1;

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn jmp_all(&mut self, address: u32) {
        self.pc = address.bank_addr();
    }

    fn jsr_n(&mut self, address: u32) {
        self.push16_n(self.pc + 2); // push the address of the brk instruction + 2
        self.pc = address.bank_addr();
    }
    fn jsr_e(&mut self, address: u32) {
        self.push16_e(self.pc + 2); // push the address of the brk instruction + 2
        self.pc = address.bank_addr();
    }

    fn jsl_n(&mut self, address: u32) {
        self.push8_n(self.prg_bank);
        self.push16_n(self.pc + 3); // push the address of the JSL instruction + 3

        self.pc = address.bank_addr();
        self.prg_bank = address.bank();
    }
    fn jsl_e(&mut self, address: u32) {
        self.push8_e(self.prg_bank);
        self.push16_e(self.pc + 3); // push the address of the JSL instruction + 3

        self.pc = address.bank_addr();
        self.prg_bank = address.bank();
    }

    fn lda_m8(&mut self, address: u32) {
        let data = self.read(address);
        self.set_acc_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn lda_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        self.acc = self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn ldx_x8(&mut self, address: u32) {
        self.x = self.read(address) as u16;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn ldx_x16(&mut self, (address_lo, address_hi): (u32, u32)) {
        self.x = self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn ldy_x8(&mut self, address: u32) {
        let data = self.read(address);
        self.set_y_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn ldy_x16(&mut self, (address_lo, address_hi): (u32, u32)) {
        self.y = self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn lsr_acc_m8(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc & 1 != 0);
        self.clear_flag(Flag::FlagN); // 0 shifted into high bit, result always positive

        self.set_acc_lo(self.get_acc_lo() >> 1);

        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn lsr_acc_m16(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc & 1 != 0);
        self.clear_flag(Flag::FlagN); // 0 shifted into high bit, result always positive

        self.acc >>= 1;

        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn lsr_mem_m8(&mut self, address: u32) {
        let data = self.read(address);
        let result = data >> 1;

        self.set_flag_to_bool(Flag::FlagC, data & 1 != 0);
        self.clear_flag(Flag::FlagN); // 0 shifted into high bit, result always positive

        self.write(address, result);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn lsr_mem_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = data >> 1;

        self.set_flag_to_bool(Flag::FlagC, data & 1 != 0);
        self.clear_flag(Flag::FlagN); // 0 shifted into high bit, result always positive

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn mvn_all(&mut self, (src_address, dest_address): (u32, u32)) {
        // Idx registers incremented in block move negative (it's backwards, I know)
        // "Negative" actually refers to where the destination address is relative
        // to the source address.
        self.x += 1;
        self.y += 1;

        let data = self.read(src_address);
        self.write(dest_address, data);

        self.acc -= 1;

        // This instruction essensially jumps to itself until it has moved self.acc + 1
        // bytes. self.acc will be 0xFFFF after this instruction. The addressing mode
        // of this instruction is always BlockMove, so the instruction is always 3 bytes.
        if self.acc != 0xFFFF {
            self.pc -= 3;
        } else {
            // Finished moving data
            self.data_bank = dest_address.bank(); // overself.write8s the dataBank register with the destination bank when finished
        }
    }

    fn mvp_all(&mut self, (src_address, dest_address): (u32, u32)) {
        // Idx registers decremented in block move positive (it's backwards, I know)
        // "Positive" actually refers to where the destination address is relative
        // to the source address.
        self.x -= 1;
        self.y -= 1;

        let data = self.read(src_address);
        self.write(dest_address, data);

        self.acc -= 1;

        // This instruction essensially jumps to itself until it has moved self.acc + 1
        // bytes. self.acc will be 0xFFFF after this instruction. The addressing mode
        // of this instruction is always BlockMove, so the instruction is always 3 bytes.
        if self.acc != 0xFFFF {
            self.pc -= 3;
        } else {
            // Finished moving data
            self.data_bank = dest_address.bank(); // overself.write8s the dataBank register with the destination bank when finished
        }
    }

    fn nop_all(&mut self) {}

    fn ora_m8(&mut self, address: u32) {
        let result = self.get_acc_lo() | self.read(address);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.set_acc_lo(result);
    }
    fn ora_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let result = self.acc | self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.acc = result;
    }

    fn pex_n(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);

        self.push16_n(data);
    }
    fn pex_e(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);

        self.push16_e(data);
    }

    fn per_n(&mut self, (address_lo, address_hi): (u32, u32)) {
        let offset = self.read16(address_lo, address_hi);

        self.push16_n(self.pc + offset + 3);
    }
    fn per_e(&mut self, (address_lo, address_hi): (u32, u32)) {
        let offset = self.read16(address_lo, address_hi);

        self.push16_e(self.pc + offset + 3);
    }

    // fn pex_n(&mut self, address: u32) {
    //     self.push16_n(address.bank_addr());
    // }
    // fn pex_e(&mut self, address: u32) {
    //     self.push16_e(address.bank_addr());
    // }

    fn pha_m8(&mut self) {
        self.push8_n(self.get_acc_lo());
    }
    fn pha_m16(&mut self) {
        self.push16_n(self.acc);
    }
    fn pha_e(&mut self) {
        self.push8_e(self.get_acc_lo());
    }

    fn phb_n(&mut self) {
        self.push8_n(self.data_bank);
    }
    fn phb_e(&mut self) {
        self.push8_e(self.data_bank);
    }

    fn phd_n(&mut self) {
        self.push16_n(self.direct_page);
    }
    fn phd_e(&mut self) {
        self.push16_e(self.direct_page);
    }

    fn phk_n(&mut self) {
        self.push8_n(self.prg_bank);
    }
    fn phk_e(&mut self) {
        self.push8_e(self.prg_bank);
    }

    fn php_n(&mut self) {
        self.push8_n(self.status);
    }
    fn php_e(&mut self) {
        self.push8_e(self.status);
    }

    fn phx_x8(&mut self) {
        self.push8_n(self.get_x_lo());
    }
    fn phx_x16(&mut self) {
        self.push16_n(self.x);
    }
    fn phx_e(&mut self) {
        self.push8_e(self.get_x_lo());
    }

    fn phy_x8(&mut self) {
        self.push8_n(self.get_y_lo());
    }
    fn phy_x16(&mut self) {
        self.push16_n(self.y);
    }
    fn phy_e(&mut self) {
        self.push8_e(self.get_y_lo());
    }

    fn pla_m8(&mut self) {
        let data = self.pop8_n();
        self.set_acc_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn pla_m16(&mut self) {
        self.acc = self.pop16_n();

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn pla_e(&mut self) {
        let data = self.pop8_e();
        self.set_acc_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn plb_n(&mut self) {
        self.data_bank = self.pop8_n();

        self.set_flag_to_bool(Flag::FlagN, self.data_bank & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.data_bank == 0);
    }
    fn plb_e(&mut self) {
        self.data_bank = self.pop8_e();

        self.set_flag_to_bool(Flag::FlagN, self.data_bank & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.data_bank == 0);
    }

    fn pld_n(&mut self) {
        self.direct_page = self.pop16_n();

        self.set_flag_to_bool(Flag::FlagN, self.direct_page & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.direct_page == 0);
    }
    fn pld_e(&mut self) {
        self.direct_page = self.pop16_e();

        self.set_flag_to_bool(Flag::FlagN, self.direct_page & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.direct_page == 0);
    }

    fn plp_n(&mut self) {
        self.status = self.pop8_n();
    }
    fn plp_e(&mut self) {
        self.status = self.pop8_e();
        // Emulation mode forces these flags on
        self.set_flag(Flag::FlagM);
        self.set_flag(Flag::FlagX);
    }

    fn plx_x8(&mut self) {
        let data = self.pop8_n();
        self.set_x_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.get_x_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_x_lo() == 0);
    }
    fn plx_x16(&mut self) {
        self.x = self.pop16_n();

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn plx_e(&mut self) {
        let data = self.pop8_e();
        self.set_x_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn ply_x8(&mut self) {
        let data = self.pop8_n();
        self.set_y_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn ply_x16(&mut self) {
        self.y = self.pop16_n();

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn ply_e(&mut self) {
        let data = self.pop8_e();
        self.set_y_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn rep_n(&mut self, address: u32) {
        self.status &= !self.read(address);
    }
    fn rep_e(&mut self, address: u32) {
        self.status &= !self.read(address);
        self.set_flag(Flag::FlagM);
        self.set_flag(Flag::FlagX);
    }

    fn rol_acc_m8(&mut self) {
        let c = self.is_flag_set(Flag::FlagC);
        self.set_flag_to_bool(Flag::FlagC, self.acc & 0x80 != 0);

        self.set_acc_lo(self.get_acc_lo() << 1);
        self.acc |= bool2byte!(c);

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn rol_acc_m16(&mut self) {
        let c = self.is_flag_set(Flag::FlagC);
        self.set_flag_to_bool(Flag::FlagC, self.acc & 0x8000 != 0);

        self.acc <<= 1;
        self.acc |= bool2byte!(c);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn rol_mem_m8(&mut self, address: u32) {
        let c = self.is_flag_set(Flag::FlagC);
        let data = self.read(address);
        let result = (data << 1) | bool2byte!(c);

        self.set_flag_to_bool(Flag::FlagC, data & 0x80 != 0);

        self.write(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn rol_mem_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let c = self.is_flag_set(Flag::FlagC);
        let data = self.read16(address_lo, address_hi);
        let result = (data << 1) | bool2byte!(c);

        self.set_flag_to_bool(Flag::FlagC, data & 0x8000 != 0);

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn ror_acc_m8(&mut self) {
        let c = self.is_flag_set(Flag::FlagC);
        self.set_flag_to_bool(Flag::FlagC, self.acc & 1 != 0);

        self.acc >>= 1;
        self.acc |= bool2byte!(c) << 7;

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn ror_acc_m16(&mut self) {
        let c = self.is_flag_set(Flag::FlagC);
        self.set_flag_to_bool(Flag::FlagC, self.acc & 1 != 0);

        self.acc >>= 1;
        self.acc |= bool2byte!(c) << 15;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn ror_mem_m8(&mut self, address: u32) {
        let c = self.is_flag_set(Flag::FlagC);

        let data = self.read(address);
        let result = (data >> 1) | (bool2byte!(c) << 7);

        self.set_flag_to_bool(Flag::FlagC, data & 1 != 0);

        self.write(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn ror_mem_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let c = self.is_flag_set(Flag::FlagC);

        let data = self.read16(address_lo, address_hi);
        let result = (data >> 1) | (bool2byte!(c) << 15);

        self.set_flag_to_bool(Flag::FlagC, data & 1 != 0);

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn rti_n(&mut self) {
        self.status = self.pop8_n();
        self.pc = self.pop16_n();
        self.prg_bank = self.pop8_n();
    }
    fn rti_e(&mut self) {
        self.status = self.pop8_e();
        self.set_flag(Flag::FlagM);
        self.set_flag(Flag::FlagX);
        self.pc = self.pop16_e();
    }

    fn rtl_n(&mut self) {
        self.pc = self.pop16_n() + 1;
        self.prg_bank = self.pop8_n();
    }
    fn rtl_e(&mut self) {
        self.pc = self.pop16_e() + 1;
        self.prg_bank = self.pop8_e();
    }

    fn rts_n(&mut self) {
        self.pc = self.pop16_n() + 1;
    }
    fn rts_e(&mut self) {
        self.pc = self.pop16_e() + 1;
    }

    fn sbc_m8(&mut self, address: u32) {
        let data = self.read(address);
        let comp = !data;
        let mut result: u16;
        let a = self.acc & 0xFF;
        let d = comp as u16;
        let c = bool2byte!(self.is_flag_set(Flag::FlagC));

        if self.is_flag_set(Flag::FlagD) {
            result = (a & 0x0F) + (d & 0x0F) + c;

            if result <= 0x0F {
                result -= 0x06;
            }

            let c = if result >= 0x10 { 0x10 } else { 0 };
            result = (a & 0xF0) + (d & 0xF0) + c + (result & 0xF);
        } else {
            result = a + d + c;
        }

        self.set_flag_to_bool(Flag::FlagV, !(a ^ d) & (d ^ result) & 0x80 != 0);

        if self.is_flag_set(Flag::FlagD) && result <= 0xFF {
            result -= 0x60;
        }

        self.set_flag_to_bool(Flag::FlagC, result > 0xFF);

        let result = result as u8;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.set_acc_lo(result);
    }
    fn sbc_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let comp = !data;
        let mut result: u32;
        let a = self.acc as u32;
        let d = comp as u32;
        let c = bool2byte!(self.is_flag_set(Flag::FlagC));

        if self.is_flag_set(Flag::FlagD) {
            result = (a & 0x000F) + (d & 0x000F) + c;

            if result <= 0xF {
                result -= 6;
            }

            let c = if result >= 0x10 { 0x10 } else { 0 };
            result = (a & 0x00F0) + (d & 0x00F0) + c + (result & 0xF);

            if result <= 0xFF {
                result -= 0x60;
            }

            let c = if result >= 0x100 { 0x100 } else { 0 };
            result = (a & 0x0F00) + (d & 0x0F00) + c + (result & 0xFF);

            if result <= 0xFFF {
                result -= 0x600;
            }

            let c = if result >= 0x1000 { 0x1000 } else { 0 };
            result = (a & 0xF000) + (d & 0xF000) + c + (result & 0xFFF);
        } else {
            result = a + d + c;
        }

        self.set_flag_to_bool(Flag::FlagV, !(a ^ d) & (d ^ result) & 0x8000 != 0);

        if self.is_flag_set(Flag::FlagD) && result <= 0xFFFF {
            result -= 0x6000;
        }

        self.set_flag_to_bool(Flag::FlagC, result > 0xFFFF);

        let result = result as u16;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(15));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.acc = result;
    }

    fn sec_all(&mut self) {
        self.set_flag(Flag::FlagC);
    }

    fn sed_all(&mut self) {
        self.set_flag(Flag::FlagD);
    }

    fn sei_all(&mut self) {
        self.set_flag(Flag::FlagI);
    }

    fn sep_all(&mut self, address: u32) {
        self.status |= self.read(address);

        match self.idx_size() {
            RegSize::Byte => {
                self.set_x_hi(0);
                self.set_y_hi(0);
            }
            _ => {}
        }
    }

    fn sta_m8(&mut self, address: u32) {
        self.write(address, self.get_acc_lo());
    }
    fn sta_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        self.write16(address_lo, address_hi, self.acc)
    }

    fn stp_all(&mut self) {
        self.stopped = true;
    }

    fn stx_x8(&mut self, address: u32) {
        self.write(address, self.get_x_lo());
    }
    fn stx_x16(&mut self, (address_lo, address_hi): (u32, u32)) {
        self.write16(address_lo, address_hi, self.x)
    }

    fn sty_x8(&mut self, address: u32) {
        self.write(address, self.get_y_lo());
    }
    fn sty_x16(&mut self, (address_lo, address_hi): (u32, u32)) {
        self.write16(address_lo, address_hi, self.y)
    }

    fn stz_m8(&mut self, address: u32) {
        self.write(address, 0);
    }
    fn stz_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        self.write16(address_lo, address_hi, 0)
    }

    fn tax_x8(&mut self) {
        self.set_x_lo(self.get_acc_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_x_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_x_lo() == 0);
    }
    fn tax_x16(&mut self) {
        self.x = self.acc;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn tay_x8(&mut self) {
        self.set_y_lo(self.get_acc_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_y_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_y_lo() == 0);
    }
    fn tay_x16(&mut self) {
        self.y = self.acc;

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn tcd_all(&mut self) {
        self.direct_page = self.acc;

        self.set_flag_to_bool(Flag::FlagN, self.direct_page & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.direct_page == 0);
    }

    fn tcs_n(&mut self) {
        self.stk_ptr = self.acc;
    }
    fn tcs_e(&mut self) {
        self.stk_ptr = 0x100 | (self.acc & 0xFF);
    }

    fn tdc_all(&mut self) {
        self.acc = self.direct_page;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn trb_m8(&mut self, address: u32) {
        let data = self.read(address);
        let result = data & self.get_acc_lo();

        self.write(address, data & (!self.get_acc_lo()));

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn trb_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = data & self.acc;

        self.write16(address_lo, address_hi, data & (!self.acc));

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tsb_m8(&mut self, address: u32) {
        let data = self.read(address);
        let result = data & self.get_acc_lo();

        self.write(address, data | self.get_acc_lo());

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn tsb_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result = data & self.acc;

        self.write16(address_lo, address_hi, data | self.acc);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tsc_m8(&mut self) {
        self.set_acc_lo(self.stk_ptr as u8);

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        // self.clear_flag(Flag::FlagN); // the value transfered is always positive
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn tsc_m16(&mut self) {
        self.acc = self.stk_ptr;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn tsc_e(&mut self) {
        self.set_acc_lo(self.stk_ptr as u8);

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
        // self.clear_flag(Flag::FlagN); // the value transfered is always positive
        // self.clear_flag(Flag::FlagZ); // the value transfered is always non-zero
    }

    fn tsx_x8(&mut self) {
        self.x = self.stk_ptr & 0xFF;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn tsx_x16(&mut self) {
        self.x = self.stk_ptr;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn txa_m8(&mut self) {
        self.set_acc_lo(self.get_x_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn txa_m16(&mut self) {
        self.acc = self.x;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn txs_n(&mut self) {
        self.stk_ptr = self.x;
    }
    fn txs_e(&mut self) {
        self.stk_ptr = 0x100 | self.get_x_lo() as u16;
    }

    fn txy_x8(&mut self) {
        self.set_y_lo(self.get_x_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_y_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_y_lo() == 0);
    }
    fn txy_x16(&mut self) {
        self.y = self.x;

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn tya_m8(&mut self) {
        self.set_acc_lo(self.get_y_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn tya_m16(&mut self) {
        self.acc = self.y;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn tyx_x8(&mut self) {
        self.set_x_lo(self.get_y_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_x_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_x_lo() == 0);
    }
    fn tyx_x16(&mut self) {
        self.x = self.y;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn wai_all(&mut self) {
        self.awaiting_interrupt = true;
    }

    fn wdm_all(&mut self, address: u32) {
        let _ = self.read(address);
    }

    fn xba_all(&mut self) {
        self.acc = self.acc.swap_bytes();

        // Flags are always based on the low byte of the acc for this instr
        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }

    fn xce_all(&mut self) {
        let new_mode = match self.is_flag_set(Flag::FlagC) {
            true => CpuMode::Emulation,
            false => CpuMode::Native,
        };
        self.set_flag_to_bool(Flag::FlagC, self.mode == CpuMode::Emulation);
        self.set_mode(new_mode);
    }
}

// Cycle Functionality
impl Cpu65c816 {
    pub fn remove_clocks(&mut self, clocks: usize) {
        self.sys_clocks_until_clock -= clocks;
    }
    pub fn sys_clocks_left(&self) -> usize {
        self.sys_clocks_until_clock
    }

    pub fn clock(&mut self) {
        self.sys_clocks_until_clock = 0;

        if self.ppu_data.cpu_vblank_nmi() {
            if !self.vblank_nmi_ignore {
                self.trigger_interrupt(CpuInterrupt::NMI);
                self.ppu_data.clear_cpu_vblank_nmi(); // ?
            }
        }

        match self.dma_status {
            DmaStatus::Off => self.exec_instr(),
            DmaStatus::DMA => self.do_dma(),
            DmaStatus::HDMA => self.do_hdma(),
            DmaStatus::LayeredHDMA => self.do_hdma(),
        }
    }

    fn do_dma(&mut self) {
        let dma_channel = &mut self.dma_channels[self.active_channel_idx];

        let a_bus_addr = dma_channel.a_bus_addr();
        let b_bus_addr = 0x2100 | dma_channel.get_b_with_offset() as u32;
        let dma_direction = dma_channel.direction.clone();

        dma_channel.inc_a_bus_addr();

        dma_channel.bytes_written += 1;
        dma_channel.byte_count -= 1;

        if dma_channel.byte_count == 0 {
            dma_channel.active = false;
            dma_channel.bytes_written = 0;

            self.dma_status = DmaStatus::Off;

            // Go to next active DMA channel
            for i in self.active_channel_idx+1..8 {
                if self.dma_channels[i].active {
                    self.active_channel_idx = i;
                    self.dma_status = DmaStatus::DMA;
                    break;
                }
            }
        }

        // Cannot perform DMA to/from MMIO addresses for A Bus
        if !is_mmio_addr(a_bus_addr) {
            let (src_addr, dst_addr) = match dma_direction {
                dma::Direction::AtoB => (a_bus_addr, b_bus_addr),
                dma::Direction::BtoA => (b_bus_addr, a_bus_addr),
            };

            let data = self.read(src_addr);
            self.write(dst_addr, data);
        }

        self.add_clocks(Cpu65c816::ONE_CYCLE);
    }

    fn do_hdma(&mut self) {
        self.add_clocks(Cpu65c816::ONE_CYCLE);
        // let hdma_channel_idx = self.hdma_enable.ilog2() as usize;

        // let hdma_indirect = self.dma_params[hdma_channel_idx] & 0x40 != 0;
        // let dst_addr = 0x002100 | self.get_b_with_offset(hdma_channel_idx) as u32;
        // let mut hdma_table_addr = self.hdma_table_addr[hdma_channel_idx];

        // // Starting a new HDMA channel transfer
        // if hdma_channel_idx as u8 != self.hdma_current_channel {
        //     self.hdma_current_channel = hdma_channel_idx as u8;
        //     self.hdma_bytes_written = 0;

        //     let control_byte = self.read(hdma_table_addr);

        //     self.hdma_line_counter[hdma_channel_idx] = self.read(hdma_table_addr);
        // }

        // hdma_table_addr += 1;

        // self.hdma_table_addr_lo[hdma_channel_idx] = hdma_table_addr as u8;
        // self.hdma_table_addr_hi[hdma_channel_idx] = (hdma_table_addr >> 8) as u8;
    }

    fn exec_instr(&mut self) {
        let opcode = self.read_prg();
        let extra_clocks: usize;

        // println!("Opcode 0x{opcode:02X} from PC ${:04X}", self.pc);

        match (opcode, self.mode, self.acc_size(), self.idx_size()) {
            // brk, imp
            (0x00, CpuMode::Emulation, ..) => {
                self.brk_e();
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x00, CpuMode::Native, ..) => {
                self.brk_n();
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ora, (dir,X)
            (0x01, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x01, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.ora_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // cop, imm
            (0x02, CpuMode::Emulation, ..) => {
                let addr = self.immediate8();
                self.cop_e(addr);
                extra_clocks = 0;
            }
            (0x02, CpuMode::Native, ..) => {
                let addr = self.immediate8();
                self.cop_n(addr);
                extra_clocks = 0;
            }

            // ora, stk,S
            (0x03, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x03, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.ora_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // tsb, dir
            (0x04, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.tsb_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x04, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.tsb_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ora, dir
            (0x05, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x05, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.ora_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // asl, dir
            (0x06, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.asl_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x06, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.asl_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ora, [dir]
            (0x07, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x07, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.ora_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // php, imp
            (0x08, CpuMode::Emulation, ..) => {
                self.php_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x08, CpuMode::Native, ..) => {
                self.php_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ora, imm
            (0x09, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0x09, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.ora_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // asl, acc
            (0x0A, _, RegSize::Byte, _) => {
                self.asl_acc_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x0A, _, RegSize::TwoBytes, _) => {
                self.asl_acc_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // phd, imp
            (0x0B, CpuMode::Emulation, ..) => {
                self.phd_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x0B, CpuMode::Native, ..) => {
                self.phd_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // tsb, abs
            (0x0C, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.tsb_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x0C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.tsb_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ora, abs
            (0x0D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.ora_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x0D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.ora_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // asl, abs
            (0x0E, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.asl_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x0E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.asl_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ora, long
            (0x0F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.ora_m8(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x0F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.ora_m16(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bpl, rel8
            (0x10, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bpl_all(addr);
                extra_clocks = 0;
            }

            // ora, (dir),Y
            (0x11, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.ora_m8(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }
            (0x11, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.ora_m16(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }

            // ora, (dir)
            (0x12, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x12, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.ora_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ora, (stk,S),Y
            (0x13, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x13, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.ora_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // trb, dir
            (0x14, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.trb_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x14, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.trb_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ora, dir,X
            (0x15, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x15, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.ora_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // asl, dir,X
            (0x16, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.asl_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }
            (0x16, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.asl_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }

            // ora, [dir],Y
            (0x17, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.ora_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x17, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.ora_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // clc, imp
            (0x18, ..) => {
                self.clc_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ora, abs,Y
            (0x19, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.ora_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x19, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.ora_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // inc, acc
            (0x1A, _, RegSize::Byte, _) => {
                self.inc_acc_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x1A, _, RegSize::TwoBytes, _) => {
                self.inc_acc_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // tcs, imp
            (0x1B, CpuMode::Emulation, ..) => {
                self.tcs_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x1B, CpuMode::Native, ..) => {
                self.tcs_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // trb, abs
            (0x1C, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.trb_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x1C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.trb_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ora, abs,X
            (0x1D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.ora_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x1D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.ora_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // asl, abs,X
            (0x1E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.asl_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x1E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.asl_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ora, long,X
            (0x1F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.ora_m8(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
            (0x1F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.ora_m16(addr);
                self.pc += 4;
                extra_clocks = 0;
            }

            // jsr, abs
            (0x20, CpuMode::Emulation, ..) => {
                let addr = self.absolute8();
                self.jsr_e(addr);
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x20, CpuMode::Native, ..) => {
                let addr = self.absolute8();
                self.jsr_n(addr);
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // and, (dir,X)
            (0x21, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x21, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.and_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // jsl, long
            (0x22, CpuMode::Emulation, ..) => {
                let addr = self.absolute8();
                self.jsl_e(addr);
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x22, CpuMode::Native, ..) => {
                let addr = self.absolute8();
                self.jsl_n(addr);
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // and, stk,S
            (0x23, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x23, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.and_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bit, dir
            (0x24, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.bit_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x24, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.bit_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // and, dir
            (0x25, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x25, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.and_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // rol, dir
            (0x26, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.rol_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x26, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.rol_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // and, [dir]
            (0x27, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x27, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.and_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // plp, imp
            (0x28, CpuMode::Emulation, ..) => {
                self.plp_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x28, CpuMode::Native, ..) => {
                self.plp_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // and, imm
            (0x29, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0x29, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.and_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // rol, acc
            (0x2A, _, RegSize::Byte, _) => {
                self.rol_acc_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x2A, _, RegSize::TwoBytes, _) => {
                self.rol_acc_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // pld, imp
            (0x2B, CpuMode::Emulation, ..) => {
                self.pld_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x2B, CpuMode::Native, ..) => {
                self.pld_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // bit, abs
            (0x2C, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.bit_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x2C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.bit_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // and, abs
            (0x2D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.and_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x2D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.and_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // rol, abs
            (0x2E, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.rol_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x2E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.rol_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // and, long
            (0x2F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.and_m8(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x2F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.and_m16(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bmi, rel8
            (0x30, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bmi_all(addr);
                extra_clocks = 0;
            }

            // and, (dir),Y
            (0x31, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.and_m8(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }
            (0x31, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.and_m16(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }

            // and, (dir)
            (0x32, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x32, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.and_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // and, (stk,S),Y
            (0x33, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x33, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.and_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // bit, dir,X
            (0x34, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.bit_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x34, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.bit_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // and, dir,X
            (0x35, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x35, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.and_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // rol, dir,X
            (0x36, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.rol_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }
            (0x36, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.rol_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }

            // and, [dir],Y
            (0x37, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.and_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x37, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.and_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sec, imp
            (0x38, ..) => {
                self.sec_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // and, abs,Y
            (0x39, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.and_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x39, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.and_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // dec, acc
            (0x3A, _, RegSize::Byte, _) => {
                self.dec_acc_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x3A, _, RegSize::TwoBytes, _) => {
                self.dec_acc_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // tsc, imp
            (0x3B, CpuMode::Emulation, ..) => {
                self.tsc_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x3B, _, RegSize::Byte, _) => {
                self.tsc_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x3B, _, RegSize::TwoBytes, _) => {
                self.tsc_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bit, abs,X
            (0x3C, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.bit_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x3C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.bit_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // and, abs,X
            (0x3D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.and_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x3D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.and_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // rol, abs,X
            (0x3E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.rol_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x3E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.rol_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // and, long,X
            (0x3F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.and_m8(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
            (0x3F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.and_m16(addr);
                self.pc += 4;
                extra_clocks = 0;
            }

            // rti, imp
            (0x40, CpuMode::Emulation, ..) => {
                self.rti_e();
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x40, CpuMode::Native, ..) => {
                self.rti_n();
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // eor, (dir,X)
            (0x41, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x41, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.eor_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // wdm, imm
            (0x42, ..) => {
                let addr = self.immediate8();
                self.wdm_all(addr);
                self.pc += 2;
                extra_clocks = 0;
            }

            // eor, stk,S
            (0x43, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x43, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.eor_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // mvp, src,dest
            (0x44, ..) => {
                let addr = self.src_dst();
                self.mvp_all(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // eor, dir
            (0x45, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x45, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.eor_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // lsr, dir
            (0x46, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.lsr_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x46, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.lsr_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // eor, [dir]
            (0x47, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x47, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.eor_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // pha, imp
            (0x48, CpuMode::Emulation, ..) => {
                self.pha_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x48, _, RegSize::Byte, _) => {
                self.pha_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x48, _, RegSize::TwoBytes, _) => {
                self.pha_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // eor, imm
            (0x49, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0x49, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.eor_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // lsr, acc
            (0x4A, _, RegSize::Byte, _) => {
                self.lsr_acc_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x4A, _, RegSize::TwoBytes, _) => {
                self.lsr_acc_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // phk, imp
            (0x4B, CpuMode::Emulation, ..) => {
                self.phk_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x4B, CpuMode::Native, ..) => {
                self.phk_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // jmp, abs
            (0x4C, ..) => {
                let addr = self.absolute8();
                self.jmp_all(addr);
                extra_clocks = 0;
            }

            // eor, abs
            (0x4D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.eor_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x4D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.eor_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // lsr, abs
            (0x4E, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.lsr_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x4E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.lsr_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // eor, long
            (0x4F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.eor_m8(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x4F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.eor_m16(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bvc, rel8
            (0x50, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bvc_all(addr);
                extra_clocks = 0;
            }

            // eor, (dir),Y
            (0x51, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.eor_m8(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }
            (0x51, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.eor_m16(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }

            // eor, (dir)
            (0x52, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x52, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.eor_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // eor, (stk,S),Y
            (0x53, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x53, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.eor_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // mvn, src,dest
            (0x54, ..) => {
                let addr = self.src_dst();
                self.mvn_all(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // eor, dir,X
            (0x55, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x55, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.eor_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // lsr, dir,X
            (0x56, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.lsr_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }
            (0x56, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.lsr_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }

            // eor, [dir],Y
            (0x57, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.eor_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x57, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.eor_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cli, imp
            (0x58, ..) => {
                self.cli_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // eor, abs,Y
            (0x59, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.eor_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x59, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.eor_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // phy, imp
            (0x5A, CpuMode::Emulation, ..) => {
                self.phy_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x5A, _, _, RegSize::Byte) => {
                self.phy_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x5A, _, _, RegSize::TwoBytes) => {
                self.phy_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // tcd, imp
            (0x5B, ..) => {
                self.tcd_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // jmp, long
            (0x5C, ..) => {
                let addr = self.absolute8();
                self.jmp_all(addr);
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // eor, abs,X
            (0x5D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.eor_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x5D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.eor_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // lsr, abs,X
            (0x5E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.lsr_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x5E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.lsr_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // eor, long,X
            (0x5F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.eor_m8(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
            (0x5F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.eor_m16(addr);
                self.pc += 4;
                extra_clocks = 0;
            }

            // rts, imp
            (0x60, CpuMode::Emulation, ..) => {
                self.rts_e();
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }
            (0x60, CpuMode::Native, ..) => {
                self.rts_n();
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }

            // adc, (dir,X)
            (0x61, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x61, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.adc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // per, imm
            (0x62, CpuMode::Emulation, ..) => {
                let addr = self.immediate16();
                self.per_e(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x62, CpuMode::Native, ..) => {
                let addr = self.immediate16();
                self.per_n(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // adc, stk,S
            (0x63, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x63, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.adc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // stz, dir
            (0x64, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.stz_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x64, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.stz_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // adc, dir
            (0x65, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x65, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.adc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ror, dir
            (0x66, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.ror_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x66, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.ror_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // adc, [dir]
            (0x67, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x67, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.adc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // pla, imp
            (0x68, CpuMode::Emulation, ..) => {
                self.pla_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x68, _, RegSize::Byte, _) => {
                self.pla_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x68, _, RegSize::TwoBytes, _) => {
                self.pla_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // adc, imm
            (0x69, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0x69, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.adc_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // ror, acc
            (0x6A, _, RegSize::Byte, _) => {
                self.ror_acc_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x6A, _, RegSize::TwoBytes, _) => {
                self.ror_acc_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // rtl, imp
            (0x6B, CpuMode::Emulation, ..) => {
                self.rtl_e();
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x6B, CpuMode::Native, ..) => {
                self.rtl_n();
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // jmp, (abs)
            (0x6C, ..) => {
                let addr = self.absolute_indirect();
                self.jmp_all(addr);
                extra_clocks = 0;
            }

            // adc, abs
            (0x6D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.adc_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x6D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.adc_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // ror, abs
            (0x6E, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.ror_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x6E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.ror_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // adc, long
            (0x6F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.adc_m8(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x6F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.adc_m16(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bvs, rel8
            (0x70, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bvs_all(addr);
                extra_clocks = 0;
            }

            // adc, (dir),Y
            (0x71, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.adc_m8(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }
            (0x71, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.adc_m16(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }

            // adc, (dir)
            (0x72, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x72, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.adc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // adc, (stk,S),Y
            (0x73, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x73, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.adc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // stz, dir,X
            (0x74, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.stz_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x74, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.stz_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // adc, dir,X
            (0x75, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x75, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.adc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ror, dir,X
            (0x76, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.ror_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }
            (0x76, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.ror_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }

            // adc, [dir],Y
            (0x77, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.adc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x77, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.adc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sei, imp
            (0x78, ..) => {
                self.sei_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // adc, abs,Y
            (0x79, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.adc_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x79, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.adc_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // ply, imp
            (0x7A, CpuMode::Emulation, ..) => {
                self.ply_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x7A, _, _, RegSize::Byte) => {
                self.ply_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x7A, _, _, RegSize::TwoBytes) => {
                self.ply_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // tdc, imp
            (0x7B, ..) => {
                self.tdc_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // jmp, (abs,X)
            (0x7C, ..) => {
                let addr = self.absolute_x_indirect8();
                self.jmp_all(addr);
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // adc, abs,X
            (0x7D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.adc_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0x7D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.adc_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // ror, abs,X
            (0x7E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.ror_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x7E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.ror_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // adc, long,X
            (0x7F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.adc_m8(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
            (0x7F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.adc_m16(addr);
                self.pc += 4;
                extra_clocks = 0;
            }

            // bra, rel8
            (0x80, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bra_all(addr);
                extra_clocks = 0;
            }

            // sta, (dir,X)
            (0x81, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x81, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // bra, rel16
            (0x82, ..) => {
                let addr = self.relative16();
                self.pc += 3;
                self.bra_all(addr);
                extra_clocks = 0;
            }

            // sta, stk,S
            (0x83, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x83, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sty, dir
            (0x84, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.sty_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x84, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.sty_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sta, dir
            (0x85, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x85, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // stx, dir
            (0x86, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.stx_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x86, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.stx_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sta, [dir]
            (0x87, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x87, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // dey, imp
            (0x88, _, _, RegSize::Byte) => {
                self.dey_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x88, _, _, RegSize::TwoBytes) => {
                self.dey_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bit, imm
            (0x89, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.bit_imm_m8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0x89, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.bit_imm_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // txa, imp
            (0x8A, _, RegSize::Byte, _) => {
                self.txa_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x8A, _, RegSize::TwoBytes, _) => {
                self.txa_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // phb, imp
            (0x8B, CpuMode::Emulation, ..) => {
                self.phb_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x8B, CpuMode::Native, ..) => {
                self.phb_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sty, abs
            (0x8C, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.sty_x8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x8C, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.sty_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // sta, abs
            (0x8D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.sta_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x8D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.sta_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // stx, abs
            (0x8E, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.stx_x8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x8E, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.stx_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // sta, long
            (0x8F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.sta_m8(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x8F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.sta_m16(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bcc, rel8
            (0x90, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bcc_all(addr);
                extra_clocks = 0;
            }

            // sta, (dir),Y
            (0x91, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x91, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // sta, (dir)
            (0x92, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x92, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sta, (stk,S),Y
            (0x93, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x93, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // sty, dir,X
            (0x94, _, _, RegSize::Byte) => {
                let addr = self.direct_x8();
                self.sty_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x94, _, _, RegSize::TwoBytes) => {
                let addr = self.direct_x16();
                self.sty_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // sta, dir,X
            (0x95, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x95, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // stx, dir,Y
            (0x96, _, _, RegSize::Byte) => {
                let addr = self.direct_y8();
                self.stx_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0x96, _, _, RegSize::TwoBytes) => {
                let addr = self.direct_y16();
                self.stx_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // sta, [dir],Y
            (0x97, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.sta_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x97, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.sta_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // tya, imp
            (0x98, _, RegSize::Byte, _) => {
                self.tya_m8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x98, _, RegSize::TwoBytes, _) => {
                self.tya_m16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sta, abs,Y
            (0x99, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.sta_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x99, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.sta_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // txs, imp
            (0x9A, CpuMode::Emulation, ..) => {
                self.txs_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x9A, CpuMode::Native, ..) => {
                self.txs_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // txy, imp
            (0x9B, _, _, RegSize::Byte) => {
                self.txy_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x9B, _, _, RegSize::TwoBytes) => {
                self.txy_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // stz, abs
            (0x9C, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.stz_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0x9C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.stz_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // sta, abs,X
            (0x9D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.sta_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x9D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.sta_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // stz, abs,X
            (0x9E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.stz_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0x9E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.stz_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sta, long,X
            (0x9F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.sta_m8(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
            (0x9F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.sta_m16(addr);
                self.pc += 4;
                extra_clocks = 0;
            }

            // ldy, imm
            (0xA0, _, _, RegSize::Byte) => {
                let addr = self.immediate8();
                self.ldy_x8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0xA0, _, _, RegSize::TwoBytes) => {
                let addr = self.immediate16();
                self.ldy_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // lda, (dir,X)
            (0xA1, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xA1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.lda_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ldx, imm
            (0xA2, _, _, RegSize::Byte) => {
                let addr = self.immediate8();
                self.ldx_x8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0xA2, _, _, RegSize::TwoBytes) => {
                let addr = self.immediate16();
                self.ldx_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // lda, stk,S
            (0xA3, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xA3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.lda_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ldy, dir
            (0xA4, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.ldy_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xA4, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.ldy_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // lda, dir
            (0xA5, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xA5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.lda_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ldx, dir
            (0xA6, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.ldx_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xA6, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.ldx_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // lda, [dir]
            (0xA7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xA7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.lda_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // tay, imp
            (0xA8, _, _, RegSize::Byte) => {
                self.tay_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xA8, _, _, RegSize::TwoBytes) => {
                self.tay_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // lda, imm
            (0xA9, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0xA9, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.lda_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // tax, imp
            (0xAA, _, _, RegSize::Byte) => {
                self.tax_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xAA, _, _, RegSize::TwoBytes) => {
                self.tax_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // plb, imp
            (0xAB, CpuMode::Emulation, ..) => {
                self.plb_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xAB, CpuMode::Native, ..) => {
                self.plb_n();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ldy, abs
            (0xAC, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.ldy_x8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0xAC, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.ldy_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // lda, abs
            (0xAD, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.lda_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0xAD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.lda_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // ldx, abs
            (0xAE, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.ldx_x8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0xAE, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.ldx_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // lda, long
            (0xAF, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.lda_m8(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xAF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.lda_m16(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bcs, rel8
            (0xB0, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bcs_all(addr);
                extra_clocks = 0;
            }

            // lda, (dir),Y
            (0xB1, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.lda_m8(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }
            (0xB1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.lda_m16(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }

            // lda, (dir)
            (0xB2, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xB2, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.lda_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // lda, (stk,S),Y
            (0xB3, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xB3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.lda_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ldy, dir,X
            (0xB4, _, _, RegSize::Byte) => {
                let addr = self.direct_x8();
                self.ldy_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xB4, _, _, RegSize::TwoBytes) => {
                let addr = self.direct_x16();
                self.ldy_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // lda, dir,X
            (0xB5, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xB5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.lda_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // ldx, dir,Y
            (0xB6, _, _, RegSize::Byte) => {
                let addr = self.direct_y8();
                self.ldx_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xB6, _, _, RegSize::TwoBytes) => {
                let addr = self.direct_y16();
                self.ldx_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // lda, [dir],Y
            (0xB7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.lda_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xB7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.lda_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // clv, imp
            (0xB8, ..) => {
                self.clv_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // lda, abs,Y
            (0xB9, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.lda_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0xB9, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.lda_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // tsx, imp
            (0xBA, _, _, RegSize::Byte) => {
                self.tsx_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xBA, _, _, RegSize::TwoBytes) => {
                self.tsx_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // tyx, imp
            (0xBB, _, _, RegSize::Byte) => {
                self.tyx_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xBB, _, _, RegSize::TwoBytes) => {
                self.tyx_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // ldy, abs,X
            (0xBC, _, _, RegSize::Byte) => {
                let addr = self.absolute_x8();
                self.ldy_x8(addr);
                self.pc += 3;

                if self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0xBC, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute_x16();
                self.ldy_x16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // lda, abs,X
            (0xBD, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.lda_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0xBD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.lda_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // ldx, abs,Y
            (0xBE, _, _, RegSize::Byte) => {
                let addr = self.absolute_y8();
                self.ldx_x8(addr);
                self.pc += 3;

                if self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0xBE, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute_y16();
                self.ldx_x16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // lda, long,X
            (0xBF, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.lda_m8(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
            (0xBF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.lda_m16(addr);
                self.pc += 4;
                extra_clocks = 0;
            }

            // cpy, imm
            (0xC0, _, _, RegSize::Byte) => {
                let addr = self.immediate8();
                self.cpy_x8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0xC0, _, _, RegSize::TwoBytes) => {
                let addr = self.immediate16();
                self.cpy_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // cmp, (dir,X)
            (0xC1, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xC1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.cmp_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // rep, imm
            (0xC2, CpuMode::Emulation, ..) => {
                let addr = self.immediate8();
                self.rep_e(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xC2, CpuMode::Native, ..) => {
                let addr = self.immediate8();
                self.rep_n(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cmp, stk,S
            (0xC3, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xC3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.cmp_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cpy, dir
            (0xC4, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.cpy_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xC4, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.cpy_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cmp, dir
            (0xC5, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xC5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.cmp_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // dec, dir
            (0xC6, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.dec_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xC6, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.dec_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // cmp, [dir]
            (0xC7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xC7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.cmp_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // iny, imp
            (0xC8, _, _, RegSize::Byte) => {
                self.iny_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xC8, _, _, RegSize::TwoBytes) => {
                self.iny_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cmp, imm
            (0xC9, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0xC9, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.cmp_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // dex, imp
            (0xCA, _, _, RegSize::Byte) => {
                self.dex_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xCA, _, _, RegSize::TwoBytes) => {
                self.dex_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // wai, imp
            (0xCB, ..) => {
                self.wai_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // cpy, abs
            (0xCC, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.cpy_x8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0xCC, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.cpy_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // cmp, abs
            (0xCD, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.cmp_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0xCD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.cmp_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // dec, abs
            (0xCE, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.dec_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xCE, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.dec_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cmp, long
            (0xCF, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.cmp_m8(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xCF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.cmp_m16(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // bne, rel8
            (0xD0, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bne_all(addr);
                extra_clocks = 0;
            }

            // cmp, (dir),Y
            (0xD1, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.cmp_m8(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }
            (0xD1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.cmp_m16(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }

            // cmp, (dir)
            (0xD2, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xD2, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.cmp_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cmp, (stk,S),Y
            (0xD3, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xD3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.cmp_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // pex, dir
            (0xD4, CpuMode::Emulation, ..) => {
                let addr = self.direct16();
                self.pex_e(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xD4, CpuMode::Native, ..) => {
                let addr = self.direct16();
                self.pex_n(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cmp, dir,X
            (0xD5, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xD5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.cmp_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // dec, dir,X
            (0xD6, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.dec_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }
            (0xD6, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.dec_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }

            // cmp, [dir],Y
            (0xD7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.cmp_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xD7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.cmp_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cld, imp
            (0xD8, ..) => {
                self.cld_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cmp, abs,Y
            (0xD9, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.cmp_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0xD9, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.cmp_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // phx, imp
            (0xDA, CpuMode::Emulation, ..) => {
                self.phx_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xDA, _, _, RegSize::Byte) => {
                self.phx_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xDA, _, _, RegSize::TwoBytes) => {
                self.phx_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // stp, imp
            (0xDB, ..) => {
                self.stp_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // jmp, [abs]
            (0xDC, ..) => {
                let addr = self.absolute_indirect_long();
                self.jmp_all(addr);
                extra_clocks = 0;
            }

            // cmp, abs,X
            (0xDD, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.cmp_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0xDD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.cmp_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // dec, abs,X
            (0xDE, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.dec_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xDE, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.dec_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // cmp, long,X
            (0xDF, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.cmp_m8(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
            (0xDF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.cmp_m16(addr);
                self.pc += 4;
                extra_clocks = 0;
            }

            // cpx, imm
            (0xE0, _, _, RegSize::Byte) => {
                let addr = self.immediate8();
                self.cpx_x8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0xE0, _, _, RegSize::TwoBytes) => {
                let addr = self.immediate16();
                self.cpx_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // sbc, (dir,X)
            (0xE1, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xE1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.sbc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // sep, imm
            (0xE2, ..) => {
                let addr = self.immediate8();
                self.sep_all(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sbc, stk,S
            (0xE3, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xE3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.sbc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // cpx, dir
            (0xE4, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.cpx_x8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xE4, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.cpx_x16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sbc, dir
            (0xE5, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xE5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.sbc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // inc, dir
            (0xE6, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.inc_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xE6, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.inc_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // sbc, [dir]
            (0xE7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xE7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.sbc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // inx, imp
            (0xE8, _, _, RegSize::Byte) => {
                self.inx_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xE8, _, _, RegSize::TwoBytes) => {
                self.inx_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sbc, imm
            (0xE9, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = 0;
            }
            (0xE9, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.sbc_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // nop, imp
            (0xEA, ..) => {
                self.nop_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // xba, imp
            (0xEB, ..) => {
                self.xba_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // cpx, abs
            (0xEC, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.cpx_x8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0xEC, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.cpx_x16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // sbc, abs
            (0xED, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.sbc_m8(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0xED, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.sbc_m16(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // inc, abs
            (0xEE, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.inc_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xEE, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.inc_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sbc, long
            (0xEF, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.sbc_m8(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xEF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.sbc_m16(addr);
                self.pc += 4;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // beq, rel8
            (0xF0, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.beq_all(addr);
                extra_clocks = 0;
            }

            // sbc, (dir),Y
            (0xF1, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.sbc_m8(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }
            (0xF1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.sbc_m16(addr);
                self.pc += 2;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }
            }

            // sbc, (dir)
            (0xF2, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xF2, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.sbc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sbc, (stk,S),Y
            (0xF3, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xF3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.sbc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // pex, imm
            (0xF4, CpuMode::Emulation, ..) => {
                let addr = self.immediate16();
                self.pex_e(addr);
                self.pc += 3;
                extra_clocks = 0;
            }
            (0xF4, CpuMode::Native, ..) => {
                let addr = self.immediate16();
                self.pex_n(addr);
                self.pc += 3;
                extra_clocks = 0;
            }

            // sbc, dir,X
            (0xF5, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xF5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.sbc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // inc, dir,X
            (0xF6, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.inc_mem_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }
            (0xF6, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.inc_mem_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::THREE_CYCLE;
            }

            // sbc, [dir],Y
            (0xF7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.sbc_m8(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xF7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.sbc_m16(addr);
                self.pc += 2;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sed, imp
            (0xF8, ..) => {
                self.sed_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sbc, abs,Y
            (0xF9, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.sbc_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0xF9, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.sbc_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // plx, imp
            (0xFA, CpuMode::Emulation, ..) => {
                self.plx_e();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xFA, _, _, RegSize::Byte) => {
                self.plx_x8();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xFA, _, _, RegSize::TwoBytes) => {
                self.plx_x16();
                self.pc += 1;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // xce, imp
            (0xFB, ..) => {
                self.xce_all();
                self.pc += 1;
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // jsr, (abs,X)
            (0xFC, CpuMode::Emulation, ..) => {
                let addr = self.absolute_x_indirect8();
                self.jsr_e(addr);
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }
            (0xFC, CpuMode::Native, ..) => {
                let addr = self.absolute_x_indirect8();
                self.jsr_n(addr);
                extra_clocks = Cpu65c816::ONE_CYCLE;
            }

            // sbc, abs,X
            (0xFD, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.sbc_m8(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }
            (0xFD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.sbc_m16(addr);
                self.pc += 3;

                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = 0;
                } else {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                }
            }

            // inc, abs,X
            (0xFE, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.inc_mem_m8(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }
            (0xFE, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.inc_mem_m16(addr);
                self.pc += 3;
                extra_clocks = Cpu65c816::TWO_CYCLE;
            }

            // sbc, long,X
            (0xFF, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.sbc_m8(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
            (0xFF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.sbc_m16(addr);
                self.pc += 4;
                extra_clocks = 0;
            }
        }

        if self.branch_taken {
            self.add_clocks(Cpu65c816::ONE_CYCLE);

            if self.page_crossed && self.mode == CpuMode::Emulation {
                self.add_clocks(Cpu65c816::ONE_CYCLE);
            }
        }

        self.add_clocks(extra_clocks);
    }
}

// impl Cpu65c816 {
//     pub fn temp_load_test(&mut self) {
//         use std::path::Path;

//         // C:\Users\lance\Desktop\Pet Projects\RustProjects\snemulator\games\Super Mario World (USA).sfc
//         let test_path_str = format!("tests/lemons/CPUTest/CPUADC.sfc");
//         // let test_path_str = format!("tests/ppu_tests/test_hello.sfc");
//         // let test_path_str = format!("games/Super Mario World (USA).sfc");
//         // let test_path_str = format!("games/SNES Test Program.sfc");
//         let test_path = Path::new(&test_path_str);
//         let cart = Cartridge::from_path(test_path).unwrap();

//         self.load_cart(cart);

//         self.stk_ptr = 0x1ff;
//         self.status = 0x34;

//         self.rom_mirror = self.rom.len() - 1;

//         self.reset();
//     }

//     fn cpu_status_str(&self) -> String {
//         let mut status_str = String::new();
//         status_str.push(if self.is_flag_set(Flag::FlagN) {
//             'N'
//         } else {
//             'n'
//         });
//         status_str.push(if self.is_flag_set(Flag::FlagV) {
//             'V'
//         } else {
//             'v'
//         });
//         if self.mode == CpuMode::Emulation {
//             status_str.push('1');
//             status_str.push(if self.is_flag_set(Flag::FlagX) {
//                 'B'
//             } else {
//                 'b'
//             });
//         } else {
//             status_str.push(if self.is_flag_set(Flag::FlagM) {
//                 'M'
//             } else {
//                 'm'
//             });
//             status_str.push(if self.is_flag_set(Flag::FlagX) {
//                 'X'
//             } else {
//                 'x'
//             });
//         }
//         status_str.push(if self.is_flag_set(Flag::FlagD) {
//             'D'
//         } else {
//             'd'
//         });
//         status_str.push(if self.is_flag_set(Flag::FlagI) {
//             'I'
//         } else {
//             'i'
//         });
//         status_str.push(if self.is_flag_set(Flag::FlagZ) {
//             'Z'
//         } else {
//             'z'
//         });
//         status_str.push(if self.is_flag_set(Flag::FlagC) {
//             'C'
//         } else {
//             'c'
//         });

//         status_str
//     }

//     fn lemon_cpu_str(&self) -> String {
//         format!(
//             "{:02x}{:04x} A:{:04x} X:{:04x} Y:{:04x} S:{:04x} D:{:04x} DB:{:02x} {} ",
//             self.prg_bank,
//             self.pc,
//             self.acc,
//             self.x,
//             self.y,
//             self.stk_ptr,
//             self.direct_page,
//             self.data_bank,
//             self.cpu_status_str()
//         )
//     }
// }

// #[cfg(test)]
// mod tests {
//     use std::path::Path;

//     use libretro_rs::retro::{
//         framebuf::ResizableFrameBuffer, log::PlatformLogger, pixel::format::XRGB8888,
//     };

//     // Note this useful idiom: importing names from outer (for mod tests) scope.
//     use super::*;
//     use crate::system::{cartridge::Cartridge, ppu::Ppu5C7x};

//     /// Prints out a slice of bytes in hex and ASCII format, side by side. When
//     /// startval is specified, indices beginning at the startval will be printed
//     /// before each line. If startval is unspecified, indeces start at 0.
//     pub fn hexdump_at(bytes: &[u8], startval: usize) {
//         const CHUNK_SIZE: usize = 16;

//         let mut index = startval;
//         println!();
//         for chunk in bytes.chunks(CHUNK_SIZE) {
//             let l = chunk.len();
//             print!("{:08X}: ", index);
//             for b in chunk.iter() {
//                 print!("{b:02X} ");
//             }

//             print!("{:>width$} ", "|", width = (CHUNK_SIZE - l) * 3 + 1);
//             for b in chunk.iter() {
//                 match b {
//                     32..=126 => print!("{}", *b as char),
//                     _ => print!("."),
//                 }
//             }
//             println!();
//             index += CHUNK_SIZE;
//         }
//     }

//     /// Prints out a slice of bytes in hex and ASCII format, side by side. When
//     /// startval is specified, indeces beginning at the startval will be printed
//     /// before each line. If startval is unspecified, indeces start at 0.
//     pub fn hexdump(bytes: &[u8]) {
//         hexdump_at(bytes, 0);
//     }

//     /// Find the subvector "needle" in the vector "haystack"
//     fn find_subvec(haystack: &Vec<u8>, needle: &Vec<u8>) -> Option<usize> {
//         (0..haystack.len() - needle.len() + 1)
//             .filter(|&i| haystack[i..i + needle.len()] == needle[..])
//             .next()
//     }

//     fn cpu_status_str(cpu: &Cpu65c816) -> String {
//         let mut status_str = String::new();
//         status_str.push(if cpu.is_flag_set(Flag::FlagN) {
//             'N'
//         } else {
//             'n'
//         });
//         status_str.push(if cpu.is_flag_set(Flag::FlagV) {
//             'V'
//         } else {
//             'v'
//         });
//         if cpu.mode == CpuMode::Emulation {
//             status_str.push('1');
//             status_str.push(if cpu.is_flag_set(Flag::FlagX) {
//                 'B'
//             } else {
//                 'b'
//             });
//         } else {
//             status_str.push(if cpu.is_flag_set(Flag::FlagM) {
//                 'M'
//             } else {
//                 'm'
//             });
//             status_str.push(if cpu.is_flag_set(Flag::FlagX) {
//                 'X'
//             } else {
//                 'x'
//             });
//         }
//         status_str.push(if cpu.is_flag_set(Flag::FlagD) {
//             'D'
//         } else {
//             'd'
//         });
//         status_str.push(if cpu.is_flag_set(Flag::FlagI) {
//             'I'
//         } else {
//             'i'
//         });
//         status_str.push(if cpu.is_flag_set(Flag::FlagZ) {
//             'Z'
//         } else {
//             'z'
//         });
//         status_str.push(if cpu.is_flag_set(Flag::FlagC) {
//             'C'
//         } else {
//             'c'
//         });

//         status_str
//     }

//     fn lemon_cpu_str(cpu: &Cpu65c816) -> String {
//         format!(
//             "{:02x}{:04x} A:{:04x} X:{:04x} Y:{:04x} S:{:04x} D:{:04x} DB:{:02x} {} ",
//             cpu.prg_bank,
//             cpu.pc,
//             cpu.acc,
//             cpu.x,
//             cpu.y,
//             cpu.stk_ptr,
//             cpu.direct_page,
//             cpu.data_bank,
//             cpu_status_str(cpu)
//         )
//     }

//     const INSTR_NAMES: [&str; 256] = [
//         "BRK", "ORA", "COP", "ORA", "TSB", "ORA", "ASL", "ORA", "PHP", "ORA", "ASL", "PHD", "TSB",
//         "ORA", "ASL", "ORA", "BPL", "ORA", "ORA", "ORA", "TRB", "ORA", "ASL", "ORA", "CLC", "ORA",
//         "INC", "TCS", "TRB", "ORA", "ASL", "ORA", "JSR", "AND", "JSL", "AND", "BIT", "AND", "ROL",
//         "AND", "PLP", "AND", "ROL", "PLD", "BIT", "AND", "ROL", "AND", "BMI", "AND", "AND", "AND",
//         "BIT", "AND", "ROL", "AND", "SEC", "AND", "DEC", "TSC", "BIT", "AND", "ROL", "AND", "RTI",
//         "EOR", "WDM", "EOR", "MVP", "EOR", "LSR", "EOR", "PHA", "EOR", "LSR", "PHK", "JMP", "EOR",
//         "LSR", "EOR", "BVC", "EOR", "EOR", "EOR", "MVN", "EOR", "LSR", "EOR", "CLI", "EOR", "PHY",
//         "TCD", "JMP", "EOR", "LSR", "EOR", "RTS", "ADC", "PEX", "ADC", "STZ", "ADC", "ROR", "ADC",
//         "PLA", "ADC", "ROR", "RTL", "JMP", "ADC", "ROR", "ADC", "BVS", "ADC", "ADC", "ADC", "STZ",
//         "ADC", "ROR", "ADC", "SEI", "ADC", "PLY", "TDC", "JMP", "ADC", "ROR", "ADC", "BRA", "STA",
//         "BRA", "STA", "STY", "STA", "STX", "STA", "DEY", "BIT", "TXA", "PHB", "STY", "STA", "STX",
//         "STA", "BCC", "STA", "STA", "STA", "STY", "STA", "STX", "STA", "TYA", "STA", "TXS", "TXY",
//         "STZ", "STA", "STZ", "STA", "LDY", "LDA", "LDX", "LDA", "LDY", "LDA", "LDX", "LDA", "TAY",
//         "LDA", "TAX", "PLB", "LDY", "LDA", "LDX", "LDA", "BCS", "LDA", "LDA", "LDA", "LDY", "LDA",
//         "LDX", "LDA", "CLV", "LDA", "TSX", "TYX", "LDY", "LDA", "LDX", "LDA", "CPY", "CMP", "REP",
//         "CMP", "CPY", "CMP", "DEC", "CMP", "INY", "CMP", "DEX", "WAI", "CPY", "CMP", "DEC", "CMP",
//         "BNE", "CMP", "CMP", "CMP", "PEX", "CMP", "DEC", "CMP", "CLD", "CMP", "PHX", "STP", "JMP",
//         "CMP", "DEC", "CMP", "CPX", "SBC", "SEP", "SBC", "CPX", "SBC", "INC", "SBC", "INX", "SBC",
//         "NOP", "XBA", "CPX", "SBC", "INC", "SBC", "BEQ", "SBC", "SBC", "SBC", "PEX", "SBC", "INC",
//         "SBC", "SED", "SBC", "PLX", "XCE", "JSR", "SBC", "INC", "SBC",
//     ];

//     #[test]
//     fn test_lorom_title() {
//         let test_path = Path::new("tests/blarggs/test_adc_sbc/test_adc.smc");
//         // let cart = Cartridge::from_path_with_mode(test_path, MappingMode::LoROM).unwrap();

//         let ppu_data = Rc::new(PpuData::new());
//         let mut cpu = Cpu65c816::new(ppu_data);

//         cpu.load_cart(cart);

//         hexdump_at(&cpu.rom[0x8000..0x8000 + 0x1000], 0x8000);
//     }

//     fn run_lemon_test(test_name: &str) {
//         let test_path_str = format!("tests/lemons/CPUTest/{test_name}.sfc");
//         let test_path = Path::new(&test_path_str);
//         let cart = Cartridge::from_path(test_path).unwrap();

//         let log_path_str = format!("tests/lemons/CPUTest/{test_name}-trace_compare.log");
//         let log_path = Path::new(&log_path_str);
//         let log_lines: Vec<String> = std::fs::read_to_string(log_path)
//             .unwrap()
//             .lines()
//             .map(String::from)
//             .collect();

//         let ppu_data = Rc::new(PpuData::new());
//         let mut cpu = Cpu65c816::new(ppu_data);
//         cpu.load_cart(cart);

//         cpu.stk_ptr = 0x1ff;
//         cpu.status = 0x34;

//         cpu.rom_mirror = cpu.rom.len() - 1;

//         cpu.pc = 0x8000;

//         cpu.wram[0] = 0xb5;

//         for (i, line) in log_lines.iter().enumerate() {
//             let opcode = cpu.read_prg();
//             let (addr_lo, addr_hi) = cpu.immediate16();
//             let val1 = cpu.read(addr_lo);
//             let val2 = cpu.read(addr_hi);

//             // Quick hack for running this test
//             if opcode == 0x2C && val1 == 0x10 && val2 == 0x42 {
//                 cpu.debug_nmi = if log_lines[i + 1].as_bytes()[48] == b'N' {
//                     0xc2
//                 } else {
//                     0x42
//                 }
//             }

//             assert_eq!(*line, lemon_cpu_str(&cpu));

//             cpu.exec_instr();
//         }
//     }

//     #[test]
//     fn test_lemon_all() {
//         let paths = std::fs::read_dir("./tests/lemons/CPUTest").unwrap();

//         for path in paths {
//             if let Ok(path) = path {
//                 let file_name = String::from(path.file_name().to_str().unwrap());

//                 if let Some(test_name) = file_name.strip_suffix(".sfc") {
//                     if test_name == "CPUMSC" {
//                         println!("cpumsc [[SKIPPED - PPU Dependent]]");
//                         continue;
//                     }

//                     run_lemon_test(test_name);

//                     println!("{} [[PASSED]]", test_name.to_lowercase());
//                 }
//             }
//         }
//     }
// }
