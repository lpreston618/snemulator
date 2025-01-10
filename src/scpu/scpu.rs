use std::ptr;

use serde::{ser::SerializeStruct, Serialize};

use crate::cartridge::Cartridge;

const WRAM_SIZE: usize = 128 * 1024; // 128 KiB

#[derive(Debug, Clone, Copy)]
pub enum MappingMode {
    LoROM,
    HiROM,
    ExHiROM,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum CpuMode {
    Emulation,
    Native,
}

#[derive(Debug)]
enum RegSize {
    Byte,
    TwoBytes,
}

#[derive(Debug)]
enum MemSel {
    FastROM,
    SlowROM,
}

pub enum Flag {
    FlagC = 1,  // Carry
    FlagZ = 2,  // Zero
    FlagI = 4,  // IRQ Disable
    FlagD = 8,  // Decimal Mode
    FlagX = 16, // X Register Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagM = 32, // Accumulator Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagV = 64, // Overflow
    FlagN = 128, // Negative
    
    // FlagB = FlagX, // Break (Emulation mode only, same place as X flag)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CpuInterrupt {
    IRQ,
    NMI,
    Reset,
    Abort,
}

trait CpuAddress {
    fn bank(self) -> u8;
    fn bank_addr(self) -> u16;
    fn page(self) -> u8;
    fn page_addr(self) -> u8;
    fn with_bank(self, bank: u8) -> Self;
    fn with_bank_addr(self, bank_addr: u16) -> Self;
    fn with_page(self, page: u8) -> Self;
    fn with_page_addr(self, page_addr: u8) -> Self;
    fn from_parts(bank: u8, page: u8, page_addr: u8) -> Self;
}

impl CpuAddress for u32 {
    fn bank(self) -> u8 {
        (self >> 16) as u8
    }
    fn bank_addr(self) -> u16 {
        self as u16
    }
    fn page(self) -> u8 {
        (self >> 8) as u8
    }
    fn page_addr(self) -> u8 {
        self as u8
    }
    fn with_bank(self, bank: u8) -> Self {
        ((bank as u32) << 16) | (self & 0x00FFFF)
    }
    fn with_bank_addr(self, bank_addr: u16) -> Self {
        (self & 0xFF0000) | (bank_addr as u32)
    }
    fn with_page(self, page: u8) -> Self {
        ((page as u32) << 8) | (self & 0xFF00FF)
    }
    fn with_page_addr(self, page_addr: u8) -> Self {
        (self & 0xFFFF00) | (page_addr as u32)
    }
    fn from_parts(bank: u8, page: u8, page_addr: u8) -> Self {
        ((bank as u32) << 16) | ((page as u32) << 8) | (page_addr as u32)
    }
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
    total_clocks: u64,

    wram: [u8; WRAM_SIZE],
    rom: Vec<u8>,
    rom_mirror: usize,
    has_sram: bool,

    debug_nmi: u8,
}

impl Serialize for Cpu65c816 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Cpu65c816", 11)?;
        s.serialize_field("acc", &self.acc)?;
        s.serialize_field("x", &self.x)?;
        s.serialize_field("y", &self.y)?;
        s.serialize_field("pc", &self.pc)?;
        s.serialize_field("stk_ptr", &self.stk_ptr)?;
        s.serialize_field("direct", &self.direct_page)?;
        s.serialize_field("data_bank", &self.data_bank)?;
        s.serialize_field("prg_bank", &self.prg_bank)?;
        s.serialize_field("status", &self.status)?;

        s.serialize_field("mode", &format!("{:?}", self.mode))?;
        s.serialize_field("mapping_mode", &format!("{:?}", self.mapping_mode))?;
        s.serialize_field("mem_sel", &format!("{:?}", self.mem_sel))?;
        s.serialize_field("branch_taken", &self.branch_taken)?;
        s.serialize_field("stopped", &self.stopped)?;
        s.serialize_field("awaiting_interrupt", &self.awaiting_interrupt)?;
        s.serialize_field("total_clocks", &self.total_clocks)?;

        s.end()
    }
}

// SNES System Functionality
impl Cpu65c816 {
    // Creates a new, uninitialized 65c816 CPU
    pub fn new() -> Self {
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
            total_clocks: 0,

            wram: [0; WRAM_SIZE],
            rom: Vec::new(),
            rom_mirror: 0,
            has_sram: false,

            debug_nmi: 0xc2, // for testing porpuses
        }
    }

    pub fn load_cart(&mut self, cart: &Cartridge) {
        self.mapping_mode = cart.mapping_mode();
        self.rom_mirror = cart.rom_size() - 1;
        self.rom = cart.rom_data();
    }

    pub fn reset(&mut self) {
        self.hardware_interrupt(CpuInterrupt::Reset);
    }
}

// Internal Helper Functions
impl Cpu65c816 {
    const ONE_CYCLE: u64 = 6;
    const ONE_CYCLE_SLOW: u64 = 8;
    const TWO_CYCLE: u64 = 12;

    fn add_clocks(&mut self, clocks: u64) {
        self.total_clocks += clocks;
    }

    fn read(&mut self, address: u32) -> u8 {
        let data: u8;
        let clocks: u64;

        match self.mapping_mode {
            MappingMode::LoROM => {
                // Memory map diagram here: https://snes.nesdev.org/wiki/Memory_map#LoROM
                match (address.bank(), address.bank_addr()) {
                    // ROM Mirror (Slow)
                    (bank @ 0x00..=0x7D, bank_addr @ 0x8000..=0xFFFF) => {
                        let addr = ((bank as u32) << 15) | ((bank_addr - 0x8000) as u32);
                        data = self.rom[(addr as usize) & self.rom_mirror];
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // ROM Mirror (SLow)
                    (bank @ 0x40..=0x6F, bank_addr @ 0x0000..=0x7FFF) => {
                        let addr = ((bank as u32) << 15) | (bank_addr as u32);
                        data = self.rom[(addr as usize) & self.rom_mirror];
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // SRAM or ROM Mirror (Slow)
                    (bank @ 0x70..=0x7F, bank_addr @ 0x0000..=0x7FFF) => {
                        if self.has_sram {
                            todo!("Access SRAM");
                        } else {
                            let addr = ((bank as u32) << 15) | (bank_addr as u32);
                            data = self.rom[(addr as usize) & self.rom_mirror];
                        }
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // Work RAM
                    (0x7E..=0x7F, ..) => {
                        data = self.wram[(address - 0x7E0000) as usize];
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // ROM Mirror (Slow)
                    (bank @ 0xC0..=0xFF, bank_addr @ 0x0000..=0x7FFF) => {
                        let addr = (((bank - 0x80) as u32) << 15) | (bank_addr as u32);
                        data = self.rom[(addr as usize) & self.rom_mirror];
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // ROM (Fast)
                    (bank @ 0x80..=0xFF, bank_addr @ 0x8000..=0xFFFF) => {
                        let addr = (((bank - 0x80) as u32) << 15) | ((bank_addr - 0x8000) as u32);
                        data = self.rom[(addr as usize) & self.rom_mirror];

                        clocks = match self.mem_sel {
                            MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                            MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
                        };
                    }
                    // Mirror of Low RAM
                    (0x00..=0x3F, bank_addr @ 0x0000..=0x1FFF)
                    | (0x80..=0xBF, bank_addr @ 0x0000..=0x1FFF) => {
                        data = self.wram[bank_addr as usize];
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // PPU Registers
                    (0x00..=0x3F, 0x2100..=0x21FF) | (0x80..=0xBF, 0x2100..=0x21FF) => {
                        data = 0;
                        clocks = 0;
                        // todo!("PPU Registers");
                    }

                    // NOTE: This read is only for cpu debugging purposes, and
                    // will be removed later.
                    (0x00, 0x4210) | (0x80, 0x4210) => {
                        data = self.debug_nmi;
                        clocks = Cpu65c816::ONE_CYCLE;
                    }

                    // CPU Registers
                    (0x00..=0x3F, 0x4200..=0x43FF) | (0x80..=0xBF, 0x4200..=0x43FF) => {
                        data = 0;
                        clocks = 0;
                        // todo!("CPU Registers");
                    }
                    // Controller Registers
                    (bank @ 0x00..=0x3F, bank_addr @ 0x4016)
                    | (bank @ 0x00..=0x3F, bank_addr @ 0x4017)
                    | (bank @ 0x80..=0xBF, bank_addr @ 0x4016)
                    | (bank @ 0x80..=0xBF, bank_addr @ 0x4017) => {
                        data = 0;
                        clocks = 0;
                        // todo!("Controller Registers");
                    }
                    _ => {
                        data = 0;
                        clocks = 0;
                    }
                }
            }

            // Notes: wram always 0x7E000..=0x7FFFFF regardless of mapping mode
            MappingMode::HiROM => {
                data = 0;
                clocks = 0;
                todo!("HiROM Mapping");
            }
            MappingMode::ExHiROM => {
                data = 0;
                clocks = 0;
                todo!("ExHiROM Mapping");
            }
        }

        self.add_clocks(clocks);

        data
    }

    fn write(&mut self, address: u32, data: u8) {
        let clocks: u64;

        match self.mapping_mode {
            MappingMode::LoROM => {
                // Memory map diagram here: https://snes.nesdev.org/wiki/Memory_map#LoROM
                match (address.bank(), address.bank_addr()) {
                    // ROM Mirror (Slow)
                    (bank @ 0x00..=0x7D, bank_addr @ 0x8000..=0xFFFF) => {
                        // let addr = ((bank as u32) << 15) | ((bank_addr - 0x8000) as u32);
                        // self.rom[(addr as usize) & self.rom_mirror] = data;
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // ROM Mirror (SLow)
                    (bank @ 0x40..=0x6F, bank_addr @ 0x0000..=0x7FFF) => {
                        // let addr = ((bank as u32) << 15) | (bank_addr as u32);
                        // self.rom[(addr as usize) & self.rom_mirror] = data;
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // SRAM or ROM Mirror (Slow)
                    (bank @ 0x70..=0x7F, bank_addr @ 0x0000..=0x7FFF) => {
                        if self.has_sram {
                            todo!("Access SRAM");
                        }
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // Work RAM
                    (0x7E..=0x7F, ..) => {
                        self.wram[(address - 0x7E0000) as usize] = data;
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // ROM Mirror (Slow)
                    (bank @ 0xC0..=0xFF, bank_addr @ 0x0000..=0x7FFF) => {
                        // let addr = (((bank - 0x80) as u32) << 15) | (bank_addr as u32);
                        // self.rom[(addr as usize) & self.rom_mirror] = data;
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // ROM (Fast)
                    (bank @ 0x80..=0xFF, bank_addr @ 0x8000..=0xFFFF) => {
                        // let addr = (((bank - 0x80) as u32) << 15) | ((bank_addr - 0x8000) as u32);
                        // self.rom[(addr as usize) & self.rom_mirror] = data;

                        clocks = match self.mem_sel {
                            MemSel::FastROM => Cpu65c816::ONE_CYCLE,
                            MemSel::SlowROM => Cpu65c816::ONE_CYCLE_SLOW,
                        };
                    }
                    // Mirror of Low RAM
                    (0x00..=0x3F, bank_addr @ 0x0000..=0x1FFF)
                    | (0x80..=0xBF, bank_addr @ 0x0000..=0x1FFF) => {
                        self.wram[bank_addr as usize] = data;
                        clocks = Cpu65c816::ONE_CYCLE_SLOW;
                    }
                    // PPU Registers
                    (0x00..=0x3F, 0x2100..=0x21FF) | (0x80..=0xBF, 0x2100..=0x21FF) => {
                        clocks = 0;
                        // todo!("PPU Registers");
                    }
                    // CPU Registers
                    (0x00..=0x3F, 0x4200..=0x43FF) | (0x80..=0xBF, 0x4200..=0x43FF) => {
                        clocks = 0;
                        // todo!("CPU Registers");
                    }
                    // Controller Registers
                    (bank @ 0x00..=0x3F, bank_addr @ 0x4016)
                    | (bank @ 0x00..=0x3F, bank_addr @ 0x4017)
                    | (bank @ 0x80..=0xBF, bank_addr @ 0x4016)
                    | (bank @ 0x80..=0xBF, bank_addr @ 0x4017) => {
                        clocks = 0;
                        // todo!("Controller Registers");
                    }
                    _ => {
                        clocks = 0;
                    }
                }
            }

            // Notes: wram always 0x7E000..=0x7FFFFF regardless of mapping mode
            MappingMode::HiROM => {
                clocks = 0;
                todo!("HiROM Mapping");
            }
            MappingMode::ExHiROM => {
                clocks = 0;
                todo!("ExHiROM Mapping");
            }
        }

        self.add_clocks(clocks);
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

    fn hardware_interrupt(&mut self, interrupt: CpuInterrupt) {
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

        self.pc = self.read16(vector_lo, vector_hi)
    }
}

// Helper functions
macro_rules! bool2byte {
    ($val:expr) => {
        if $val {
            1
        } else {
            0
        }
    };
}
fn inc_low_byte(value: u16) -> u16 {
    (value & 0xFF00) | ((value + 1) & 0x00FF)
}
fn dec_low_byte(value: u16) -> u16 {
    (value & 0xFF00) | ((value - 1) & 0x00FF)
}

// Computes lhs + rhs + carry and outputs a new BCD digit. Alters the carry variable with the new carry value.
fn bcd_add_digit(lhs: u8, rhs: u8, carry: &mut bool) -> u8 {
    let mut result = lhs + rhs + bool2byte!(*carry);
    *carry = false;

    // If the resulting digit is 10-15, make it wrap back around starting at 0
    if result >= 10 {
        result -= 10;
        *carry = true;
    }

    result
}

// Computes lhs - rhs - borrow and outputs a new BCD digit. Alters the borrow variable with the new borrow value.
fn bcd_sub_digit(lhs: u8, rhs: u8, borrow: &mut bool) -> u8 {
    let mut rhs = rhs;
    let mut lhs = lhs;

    rhs += bool2byte!(*borrow);
    *borrow = false;

    // If result of subtraction would be negative, make it wrap around starting at 9
    if rhs > lhs {
        lhs += 10;
        *borrow = true;
    }

    lhs - rhs
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
        (
            (direct) as u32,
            (direct + 1) as u32,
        )
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

        let original_addr = u32::from_parts(
            self.data_bank, 
            self.read(ptr_hi), 
            self.read(ptr_lo)
        );
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
        let result: u8;

        // Decimal Mode
        if self.is_flag_set(Flag::FlagD) {
            // One's place, ten's place
            let o_place: u8;
            let t_place: u8;

            let mut carry = self.is_flag_set(Flag::FlagC);

            o_place = bcd_add_digit(self.get_acc_lo() & 0x0F, data & 0x0F, &mut carry);
            t_place = bcd_add_digit(
                self.get_acc_lo() >> 4,
                data >> 4,
                &mut carry,
            );

            result = (t_place << 4) | o_place;

            self.set_flag_to_bool(Flag::FlagC, carry);
        } else {
            result = self.get_acc_lo() + data + bool2byte!(self.is_flag_set(Flag::FlagC));

            self.set_flag_to_bool(Flag::FlagC, result < self.get_acc_lo());
            self.set_flag_to_bool(Flag::FlagV, !(self.get_acc_lo() ^ data) & (data ^ result) & 0x80 != 0);
        }

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.set_acc_lo(result);
    }

    fn adc_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let result: u16;

        // Decimal Mode
        if self.is_flag_set(Flag::FlagD) {
            // One's place, ten's place
            let o_place: u16;
            let t_place: u16;
            let h_place: u16;
            let th_place: u16;

            let mut carry = self.is_flag_set(Flag::FlagC);

            o_place = bcd_add_digit(self.get_acc_lo() & 0x0F, (data & 0x0F) as u8, &mut carry) as u16;
            t_place = bcd_add_digit(
                self.get_acc_lo() >> 4,
                (data as u8) >> 4,
                &mut carry,
            ) as u16;
            h_place = bcd_add_digit(
                self.get_acc_hi() & 0x0F,
                ((data >> 8) & 0x0F) as u8,
                &mut carry,
            ) as u16;
            th_place = bcd_add_digit(
                self.get_acc_hi() >> 4,
                (data >> 12) as u8,
                &mut carry,
            ) as u16;

            result = (th_place << 12) | (h_place << 8) | (t_place << 4) | o_place;

            self.set_flag_to_bool(Flag::FlagC, carry);
        } else {
            let temp = (self.acc as i32) + (data as i32) + bool2byte!(self.is_flag_set(Flag::FlagC));

            result = temp as u16;

            self.set_flag_to_bool(Flag::FlagC, result < self.acc);
            self.set_flag_to_bool(Flag::FlagV, !(self.acc ^ data) & (data ^ result) & 0x8000 != 0);
        }

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
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
        let ones_comp = !data;
        let result;

        if self.is_flag_set(Flag::FlagD) {
            // One's place, ten's place
            let o_place: u8;
            let t_place: u8;
            let mut borrow = !self.is_flag_set(Flag::FlagC);

            o_place = bcd_sub_digit(self.get_acc_lo() & 0x0F, data & 0x0F, &mut borrow);
            t_place = bcd_sub_digit(
                (self.get_acc_lo() >> 4) & 0x0F,
                (data >> 4) & 0x0F,
                &mut borrow,
            );

            result = (t_place << 4) | o_place;

            self.set_flag_to_bool(Flag::FlagC, !borrow);
        } else {
            result = self.get_acc_lo() + ones_comp + bool2byte!(self.is_flag_set(Flag::FlagC));

            self.set_flag_to_bool(Flag::FlagC, self.get_acc_lo() >= data);
        }

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagV, !(self.get_acc_lo() ^ ones_comp) & (ones_comp ^ result) & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.set_acc_lo(result);
    }
    fn sbc_m16(&mut self, (address_lo, address_hi): (u32, u32)) {
        let data = self.read16(address_lo, address_hi);
        let ones_comp = !data;
        let result;

        if self.is_flag_set(Flag::FlagD) {
            // One's place, ten's place, hundred's place, thousand's place
            let o_place: u16;
            let t_place: u16;
            let h_place: u16;
            let th_place: u16;
            let mut borrow = !self.is_flag_set(Flag::FlagC);

            o_place =
                bcd_sub_digit(self.get_acc_lo() & 0x0F, (data & 0x0F) as u8, &mut borrow) as u16;
            t_place = bcd_sub_digit(
                (self.get_acc_lo() >> 4) & 0x0F,
                ((data >> 4) & 0x0F) as u8,
                &mut borrow,
            ) as u16;
            h_place = bcd_sub_digit(
                self.get_acc_hi() & 0x0F,
                ((data >> 8) & 0x0F) as u8,
                &mut borrow,
            ) as u16;
            th_place = bcd_sub_digit(
                (self.get_acc_hi() >> 4) & 0x0F,
                ((data >> 12) & 0x0F) as u8,
                &mut borrow,
            ) as u16;

            result = (th_place << 12) | (h_place << 8) | (t_place << 4) | o_place;

            self.set_flag_to_bool(Flag::FlagC, !borrow);
        } else {
            result = self.acc + ones_comp + bool2byte!(self.is_flag_set(Flag::FlagC));

            self.set_flag_to_bool(Flag::FlagC, self.acc >= data);
        }

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagV, !(self.acc ^ ones_comp) & (ones_comp ^ result) & 0x8000 != 0);
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
    fn exec_instr(&mut self) {
        let opcode = self.read_prg();

        match (opcode, self.mode, self.acc_size(), self.idx_size()) {
            // brk, imp
            (0x00, CpuMode::Emulation, ..) => {
                self.brk_e();
            }
            (0x00, CpuMode::Native, ..) => {
                self.brk_n();
            }

            // ora, (dir,X)
            (0x01, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x01, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // cop, imm
            (0x02, CpuMode::Emulation, ..) => {
                let addr = self.immediate8();
                self.cop_e(addr);
                // self.pc += 2;
            }
            (0x02, CpuMode::Native, ..) => {
                let addr = self.immediate8();
                self.cop_n(addr);
                // self.pc += 2;
            }

            // ora, stk,S
            (0x03, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x03, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // tsb, dir
            (0x04, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.tsb_m8(addr);
                self.pc += 2;
            }
            (0x04, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.tsb_m16(addr);
                self.pc += 2;
            }

            // ora, dir
            (0x05, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x05, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // asl, dir
            (0x06, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.asl_mem_m8(addr);
                self.pc += 2;
            }
            (0x06, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.asl_mem_m16(addr);
                self.pc += 2;
            }

            // ora, [dir]
            (0x07, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x07, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // php, imp
            (0x08, CpuMode::Emulation, ..) => {
                self.php_e();
                self.pc += 1;
            }
            (0x08, CpuMode::Native, ..) => {
                self.php_n();
                self.pc += 1;
            }

            // ora, imm
            (0x09, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x09, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.ora_m16(addr);
                self.pc += 3;
            }

            // asl, acc
            (0x0A, _, RegSize::Byte, _) => {
                self.asl_acc_m8();
                self.pc += 1;
            }
            (0x0A, _, RegSize::TwoBytes, _) => {
                self.asl_acc_m16();
                self.pc += 1;
            }

            // phd, imp
            (0x0B, CpuMode::Emulation, ..) => {
                self.phd_e();
                self.pc += 1;
            }
            (0x0B, CpuMode::Native, ..) => {
                self.phd_n();
                self.pc += 1;
            }

            // tsb, abs
            (0x0C, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.tsb_m8(addr);
                self.pc += 3;
            }
            (0x0C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.tsb_m16(addr);
                self.pc += 3;
            }

            // ora, abs
            (0x0D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.ora_m8(addr);
                self.pc += 3;
            }
            (0x0D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.ora_m16(addr);
                self.pc += 3;
            }

            // asl, abs
            (0x0E, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.asl_mem_m8(addr);
                self.pc += 3;
            }
            (0x0E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.asl_mem_m16(addr);
                self.pc += 3;
            }

            // ora, long
            (0x0F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.ora_m8(addr);
                self.pc += 4;
            }
            (0x0F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.ora_m16(addr);
                self.pc += 4;
            }

            // bpl, rel8
            (0x10, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bpl_all(addr);
            }

            // ora, (dir),Y
            (0x11, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x11, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // ora, (dir)
            (0x12, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x12, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // ora, (stk,S),Y
            (0x13, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x13, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // trb, dir
            (0x14, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.trb_m8(addr);
                self.pc += 2;
            }
            (0x14, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.trb_m16(addr);
                self.pc += 2;
            }

            // ora, dir,X
            (0x15, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x15, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // asl, dir,X
            (0x16, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.asl_mem_m8(addr);
                self.pc += 2;
            }
            (0x16, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.asl_mem_m16(addr);
                self.pc += 2;
            }

            // ora, [dir],Y
            (0x17, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.ora_m8(addr);
                self.pc += 2;
            }
            (0x17, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.ora_m16(addr);
                self.pc += 2;
            }

            // clc, imp
            (0x18, ..) => {
                self.clc_all();
                self.pc += 1;
            }

            // ora, abs,Y
            (0x19, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.ora_m8(addr);
                self.pc += 3;
            }
            (0x19, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.ora_m16(addr);
                self.pc += 3;
            }

            // inc, acc
            (0x1A, _, RegSize::Byte, _) => {
                self.inc_acc_m8();
                self.pc += 1;
            }
            (0x1A, _, RegSize::TwoBytes, _) => {
                self.inc_acc_m16();
                self.pc += 1;
            }

            // tcs, imp
            (0x1B, CpuMode::Emulation, ..) => {
                self.tcs_e();
                self.pc += 1;
            }
            (0x1B, CpuMode::Native, ..) => {
                self.tcs_n();
                self.pc += 1;
            }

            // trb, abs
            (0x1C, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.trb_m8(addr);
                self.pc += 3;
            }
            (0x1C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.trb_m16(addr);
                self.pc += 3;
            }

            // ora, abs,X
            (0x1D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.ora_m8(addr);
                self.pc += 3;
            }
            (0x1D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.ora_m16(addr);
                self.pc += 3;
            }

            // asl, abs,X
            (0x1E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.asl_mem_m8(addr);
                self.pc += 3;
            }
            (0x1E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.asl_mem_m16(addr);
                self.pc += 3;
            }

            // ora, long,X
            (0x1F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.ora_m8(addr);
                self.pc += 4;
            }
            (0x1F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.ora_m16(addr);
                self.pc += 4;
            }

            // jsr, abs
            (0x20, CpuMode::Emulation, ..) => {
                let addr = self.absolute8();
                self.jsr_e(addr);
            }
            (0x20, CpuMode::Native, ..) => {
                let addr = self.absolute8();
                self.jsr_n(addr);
            }

            // and, (dir,X)
            (0x21, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x21, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // jsl, long
            (0x22, CpuMode::Emulation, ..) => {
                let addr = self.absolute8();
                self.jsl_e(addr);
            }
            (0x22, CpuMode::Native, ..) => {
                let addr = self.absolute8();
                self.jsl_n(addr);
            }

            // and, stk,S
            (0x23, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x23, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // bit, dir
            (0x24, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.bit_m8(addr);
                self.pc += 2;
            }
            (0x24, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.bit_m16(addr);
                self.pc += 2;
            }

            // and, dir
            (0x25, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x25, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // rol, dir
            (0x26, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.rol_mem_m8(addr);
                self.pc += 2;
            }
            (0x26, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.rol_mem_m16(addr);
                self.pc += 2;
            }

            // and, [dir]
            (0x27, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x27, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // plp, imp
            (0x28, CpuMode::Emulation, ..) => {
                self.plp_e();
                self.pc += 1;
            }
            (0x28, CpuMode::Native, ..) => {
                self.plp_n();
                self.pc += 1;
            }

            // and, imm
            (0x29, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x29, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.and_m16(addr);
                self.pc += 3;
            }

            // rol, acc
            (0x2A, _, RegSize::Byte, _) => {
                self.rol_acc_m8();
                self.pc += 1;
            }
            (0x2A, _, RegSize::TwoBytes, _) => {
                self.rol_acc_m16();
                self.pc += 1;
            }

            // pld, imp
            (0x2B, CpuMode::Emulation, ..) => {
                self.pld_e();
                self.pc += 1;
            }
            (0x2B, CpuMode::Native, ..) => {
                self.pld_n();
                self.pc += 1;
            }

            // bit, abs
            (0x2C, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.bit_m8(addr);
                self.pc += 3;
            }
            (0x2C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.bit_m16(addr);
                self.pc += 3;
            }

            // and, abs
            (0x2D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.and_m8(addr);
                self.pc += 3;
            }
            (0x2D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.and_m16(addr);
                self.pc += 3;
            }

            // rol, abs
            (0x2E, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.rol_mem_m8(addr);
                self.pc += 3;
            }
            (0x2E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.rol_mem_m16(addr);
                self.pc += 3;
            }

            // and, long
            (0x2F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.and_m8(addr);
                self.pc += 4;
            }
            (0x2F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.and_m16(addr);
                self.pc += 4;
            }

            // bmi, rel8
            (0x30, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bmi_all(addr);
            }

            // and, (dir),Y
            (0x31, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x31, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // and, (dir)
            (0x32, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x32, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // and, (stk,S),Y
            (0x33, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x33, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // bit, dir,X
            (0x34, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.bit_m8(addr);
                self.pc += 2;
            }
            (0x34, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.bit_m16(addr);
                self.pc += 2;
            }

            // and, dir,X
            (0x35, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x35, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // rol, dir,X
            (0x36, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.rol_mem_m8(addr);
                self.pc += 2;
            }
            (0x36, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.rol_mem_m16(addr);
                self.pc += 2;
            }

            // and, [dir],Y
            (0x37, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.and_m8(addr);
                self.pc += 2;
            }
            (0x37, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.and_m16(addr);
                self.pc += 2;
            }

            // sec, imp
            (0x38, ..) => {
                self.sec_all();
                self.pc += 1;
            }

            // and, abs,Y
            (0x39, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.and_m8(addr);
                self.pc += 3;
            }
            (0x39, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.and_m16(addr);
                self.pc += 3;
            }

            // dec, acc
            (0x3A, _, RegSize::Byte, _) => {
                self.dec_acc_m8();
                self.pc += 1;
            }
            (0x3A, _, RegSize::TwoBytes, _) => {
                self.dec_acc_m16();
                self.pc += 1;
            }

            // tsc, imp
            (0x3B, CpuMode::Emulation, ..) => {
                self.tsc_e();
                self.pc += 1;
            }
            (0x3B, _, RegSize::Byte, _) => {
                self.tsc_m8();
                self.pc += 1;
            }
            (0x3B, _, RegSize::TwoBytes, _) => {
                self.tsc_m16();
                self.pc += 1;
            }

            // bit, abs,X
            (0x3C, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.bit_m8(addr);
                self.pc += 3;
            }
            (0x3C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.bit_m16(addr);
                self.pc += 3;
            }

            // and, abs,X
            (0x3D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.and_m8(addr);
                self.pc += 3;
            }
            (0x3D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.and_m16(addr);
                self.pc += 3;
            }

            // rol, abs,X
            (0x3E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.rol_mem_m8(addr);
                self.pc += 3;
            }
            (0x3E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.rol_mem_m16(addr);
                self.pc += 3;
            }

            // and, long,X
            (0x3F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.and_m8(addr);
                self.pc += 4;
            }
            (0x3F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.and_m16(addr);
                self.pc += 4;
            }

            // rti, imp
            (0x40, CpuMode::Emulation, ..) => {
                self.rti_e();
            }
            (0x40, CpuMode::Native, ..) => {
                self.rti_n();
            }

            // eor, (dir,X)
            (0x41, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x41, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // wdm, imm
            (0x42, ..) => {
                let addr = self.immediate8();
                self.wdm_all(addr);
                self.pc += 2;
            }

            // eor, stk,S
            (0x43, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x43, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // mvp, src,dest
            (0x44, ..) => {
                let addr = self.src_dst();
                self.mvp_all(addr);
                self.pc += 3;
            }

            // eor, dir
            (0x45, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x45, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // lsr, dir
            (0x46, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.lsr_mem_m8(addr);
                self.pc += 2;
            }
            (0x46, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.lsr_mem_m16(addr);
                self.pc += 2;
            }

            // eor, [dir]
            (0x47, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x47, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // pha, imp
            (0x48, CpuMode::Emulation, ..) => {
                self.pha_e();
                self.pc += 1;
            }
            (0x48, _, RegSize::Byte, _) => {
                self.pha_m8();
                self.pc += 1;
            }
            (0x48, _, RegSize::TwoBytes, _) => {
                self.pha_m16();
                self.pc += 1;
            }

            // eor, imm
            (0x49, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x49, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.eor_m16(addr);
                self.pc += 3;
            }

            // lsr, acc
            (0x4A, _, RegSize::Byte, _) => {
                self.lsr_acc_m8();
                self.pc += 1;
            }
            (0x4A, _, RegSize::TwoBytes, _) => {
                self.lsr_acc_m16();
                self.pc += 1;
            }

            // phk, imp
            (0x4B, CpuMode::Emulation, ..) => {
                self.phk_e();
                self.pc += 1;
            }
            (0x4B, CpuMode::Native, ..) => {
                self.phk_n();
                self.pc += 1;
            }

            // jmp, abs
            (0x4C, ..) => {
                let addr = self.absolute8();
                self.jmp_all(addr);
            }

            // eor, abs
            (0x4D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.eor_m8(addr);
                self.pc += 3;
            }
            (0x4D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.eor_m16(addr);
                self.pc += 3;
            }

            // lsr, abs
            (0x4E, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.lsr_mem_m8(addr);
                self.pc += 3;
            }
            (0x4E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.lsr_mem_m16(addr);
                self.pc += 3;
            }

            // eor, long
            (0x4F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.eor_m8(addr);
                self.pc += 4;
            }
            (0x4F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.eor_m16(addr);
                self.pc += 4;
            }

            // bvc, rel8
            (0x50, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bvc_all(addr);
            }

            // eor, (dir),Y
            (0x51, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x51, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // eor, (dir)
            (0x52, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x52, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // eor, (stk,S),Y
            (0x53, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x53, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // mvn, src,dest
            (0x54, ..) => {
                let addr = self.src_dst();
                self.mvn_all(addr);
                self.pc += 3;
            }

            // eor, dir,X
            (0x55, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x55, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // lsr, dir,X
            (0x56, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.lsr_mem_m8(addr);
                self.pc += 2;
            }
            (0x56, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.lsr_mem_m16(addr);
                self.pc += 2;
            }

            // eor, [dir],Y
            (0x57, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.eor_m8(addr);
                self.pc += 2;
            }
            (0x57, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.eor_m16(addr);
                self.pc += 2;
            }

            // cli, imp
            (0x58, ..) => {
                self.cli_all();
                self.pc += 1;
            }

            // eor, abs,Y
            (0x59, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.eor_m8(addr);
                self.pc += 3;
            }
            (0x59, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.eor_m16(addr);
                self.pc += 3;
            }

            // phy, imp
            (0x5A, CpuMode::Emulation, ..) => {
                self.phy_e();
                self.pc += 1;
            }
            (0x5A, _, _, RegSize::Byte) => {
                self.phy_x8();
                self.pc += 1;
            }
            (0x5A, _, _, RegSize::TwoBytes) => {
                self.phy_x16();
                self.pc += 1;
            }

            // tcd, imp
            (0x5B, ..) => {
                self.tcd_all();
                self.pc += 1;
            }

            // jmp, long
            (0x5C, ..) => {
                let addr = self.absolute8();
                self.jmp_all(addr);
            }

            // eor, abs,X
            (0x5D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.eor_m8(addr);
                self.pc += 3;
            }
            (0x5D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.eor_m16(addr);
                self.pc += 3;
            }

            // lsr, abs,X
            (0x5E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.lsr_mem_m8(addr);
                self.pc += 3;
            }
            (0x5E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.lsr_mem_m16(addr);
                self.pc += 3;
            }

            // eor, long,X
            (0x5F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.eor_m8(addr);
                self.pc += 4;
            }
            (0x5F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.eor_m16(addr);
                self.pc += 4;
            }

            // rts, imp
            (0x60, CpuMode::Emulation, ..) => {
                self.rts_e();
            }
            (0x60, CpuMode::Native, ..) => {
                self.rts_n();
            }

            // adc, (dir,X)
            (0x61, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x61, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // per, imm
            (0x62, CpuMode::Emulation, ..) => {
                let addr = self.immediate16();
                self.per_e(addr);
                self.pc += 3;
            }
            (0x62, CpuMode::Native, ..) => {
                let addr = self.immediate16();
                self.per_n(addr);
                self.pc += 3;
            }

            // adc, stk,S
            (0x63, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x63, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // stz, dir
            (0x64, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.stz_m8(addr);
                self.pc += 2;
            }
            (0x64, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.stz_m16(addr);
                self.pc += 2;
            }

            // adc, dir
            (0x65, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x65, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // ror, dir
            (0x66, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.ror_mem_m8(addr);
                self.pc += 2;
            }
            (0x66, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.ror_mem_m16(addr);
                self.pc += 2;
            }

            // adc, [dir]
            (0x67, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x67, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // pla, imp
            (0x68, CpuMode::Emulation, ..) => {
                self.pla_e();
                self.pc += 1;
            }
            (0x68, _, RegSize::Byte, _) => {
                self.pla_m8();
                self.pc += 1;
            }
            (0x68, _, RegSize::TwoBytes, _) => {
                self.pla_m16();
                self.pc += 1;
            }

            // adc, imm
            (0x69, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x69, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.adc_m16(addr);
                self.pc += 3;
            }

            // ror, acc
            (0x6A, _, RegSize::Byte, _) => {
                self.ror_acc_m8();
                self.pc += 1;
            }
            (0x6A, _, RegSize::TwoBytes, _) => {
                self.ror_acc_m16();
                self.pc += 1;
            }

            // rtl, imp
            (0x6B, CpuMode::Emulation, ..) => {
                self.rtl_e();
            }
            (0x6B, CpuMode::Native, ..) => {
                self.rtl_n();
            }

            // jmp, (abs)
            (0x6C, ..) => {
                let addr = self.absolute_indirect();
                self.jmp_all(addr);
            }

            // adc, abs
            (0x6D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.adc_m8(addr);
                self.pc += 3;
            }
            (0x6D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.adc_m16(addr);
                self.pc += 3;
            }

            // ror, abs
            (0x6E, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.ror_mem_m8(addr);
                self.pc += 3;
            }
            (0x6E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.ror_mem_m16(addr);
                self.pc += 3;
            }

            // adc, long
            (0x6F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.adc_m8(addr);
                self.pc += 4;
            }
            (0x6F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.adc_m16(addr);
                self.pc += 4;
            }

            // bvs, rel8
            (0x70, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bvs_all(addr);
            }

            // adc, (dir),Y
            (0x71, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x71, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // adc, (dir)
            (0x72, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x72, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // adc, (stk,S),Y
            (0x73, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x73, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // stz, dir,X
            (0x74, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.stz_m8(addr);
                self.pc += 2;
            }
            (0x74, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.stz_m16(addr);
                self.pc += 2;
            }

            // adc, dir,X
            (0x75, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x75, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // ror, dir,X
            (0x76, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.ror_mem_m8(addr);
                self.pc += 2;
            }
            (0x76, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.ror_mem_m16(addr);
                self.pc += 2;
            }

            // adc, [dir],Y
            (0x77, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.adc_m8(addr);
                self.pc += 2;
            }
            (0x77, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.adc_m16(addr);
                self.pc += 2;
            }

            // sei, imp
            (0x78, ..) => {
                self.sei_all();
                self.pc += 1;
            }

            // adc, abs,Y
            (0x79, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.adc_m8(addr);
                self.pc += 3;
            }
            (0x79, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.adc_m16(addr);
                self.pc += 3;
            }

            // ply, imp
            (0x7A, CpuMode::Emulation, ..) => {
                self.ply_e();
                self.pc += 1;
            }
            (0x7A, _, _, RegSize::Byte) => {
                self.ply_x8();
                self.pc += 1;
            }
            (0x7A, _, _, RegSize::TwoBytes) => {
                self.ply_x16();
                self.pc += 1;
            }

            // tdc, imp
            (0x7B, ..) => {
                self.tdc_all();
                self.pc += 1;
            }

            // jmp, (abs,X)
            (0x7C, ..) => {
                let addr = self.absolute_x_indirect8();
                self.jmp_all(addr);
            }

            // adc, abs,X
            (0x7D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.adc_m8(addr);
                self.pc += 3;
            }
            (0x7D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.adc_m16(addr);
                self.pc += 3;
            }

            // ror, abs,X
            (0x7E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.ror_mem_m8(addr);
                self.pc += 3;
            }
            (0x7E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.ror_mem_m16(addr);
                self.pc += 3;
            }

            // adc, long,X
            (0x7F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.adc_m8(addr);
                self.pc += 4;
            }
            (0x7F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.adc_m16(addr);
                self.pc += 4;
            }

            // bra, rel8
            (0x80, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bra_all(addr);
            }

            // sta, (dir,X)
            (0x81, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x81, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // bra, rel16
            (0x82, ..) => {
                let addr = self.relative16();
                self.pc += 3;
                self.bra_all(addr);
            }

            // sta, stk,S
            (0x83, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x83, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // sty, dir
            (0x84, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.sty_x8(addr);
                self.pc += 2;
            }
            (0x84, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.sty_x16(addr);
                self.pc += 2;
            }

            // sta, dir
            (0x85, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x85, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // stx, dir
            (0x86, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.stx_x8(addr);
                self.pc += 2;
            }
            (0x86, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.stx_x16(addr);
                self.pc += 2;
            }

            // sta, [dir]
            (0x87, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x87, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // dey, imp
            (0x88, _, _, RegSize::Byte) => {
                self.dey_x8();
                self.pc += 1;
            }
            (0x88, _, _, RegSize::TwoBytes) => {
                self.dey_x16();
                self.pc += 1;
            }

            // bit, imm
            (0x89, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.bit_imm_m8(addr);
                self.pc += 2;
            }
            (0x89, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.bit_imm_m16(addr);
                self.pc += 3;
            }

            // txa, imp
            (0x8A, _, RegSize::Byte, _) => {
                self.txa_m8();
                self.pc += 1;
            }
            (0x8A, _, RegSize::TwoBytes, _) => {
                self.txa_m16();
                self.pc += 1;
            }

            // phb, imp
            (0x8B, CpuMode::Emulation, ..) => {
                self.phb_e();
                self.pc += 1;
            }
            (0x8B, CpuMode::Native, ..) => {
                self.phb_n();
                self.pc += 1;
            }

            // sty, abs
            (0x8C, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.sty_x8(addr);
                self.pc += 3;
            }
            (0x8C, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.sty_x16(addr);
                self.pc += 3;
            }

            // sta, abs
            (0x8D, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.sta_m8(addr);
                self.pc += 3;
            }
            (0x8D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.sta_m16(addr);
                self.pc += 3;
            }

            // stx, abs
            (0x8E, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.stx_x8(addr);
                self.pc += 3;
            }
            (0x8E, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.stx_x16(addr);
                self.pc += 3;
            }

            // sta, long
            (0x8F, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.sta_m8(addr);
                self.pc += 4;
            }
            (0x8F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.sta_m16(addr);
                self.pc += 4;
            }

            // bcc, rel8
            (0x90, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bcc_all(addr);
            }

            // sta, (dir),Y
            (0x91, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x91, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // sta, (dir)
            (0x92, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x92, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // sta, (stk,S),Y
            (0x93, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x93, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // sty, dir,X
            (0x94, _, _, RegSize::Byte) => {
                let addr = self.direct_x8();
                self.sty_x8(addr);
                self.pc += 2;
            }
            (0x94, _, _, RegSize::TwoBytes) => {
                let addr = self.direct_x16();
                self.sty_x16(addr);
                self.pc += 2;
            }

            // sta, dir,X
            (0x95, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x95, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // stx, dir,Y
            (0x96, _, _, RegSize::Byte) => {
                let addr = self.direct_y8();
                self.stx_x8(addr);
                self.pc += 2;
            }
            (0x96, _, _, RegSize::TwoBytes) => {
                let addr = self.direct_y16();
                self.stx_x16(addr);
                self.pc += 2;
            }

            // sta, [dir],Y
            (0x97, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.sta_m8(addr);
                self.pc += 2;
            }
            (0x97, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.sta_m16(addr);
                self.pc += 2;
            }

            // tya, imp
            (0x98, _, RegSize::Byte, _) => {
                self.tya_m8();
                self.pc += 1;
            }
            (0x98, _, RegSize::TwoBytes, _) => {
                self.tya_m16();
                self.pc += 1;
            }

            // sta, abs,Y
            (0x99, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.sta_m8(addr);
                self.pc += 3;
            }
            (0x99, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.sta_m16(addr);
                self.pc += 3;
            }

            // txs, imp
            (0x9A, CpuMode::Emulation, ..) => {
                self.txs_e();
                self.pc += 1;
            }
            (0x9A, CpuMode::Native, ..) => {
                self.txs_n();
                self.pc += 1;
            }

            // txy, imp
            (0x9B, _, _, RegSize::Byte) => {
                self.txy_x8();
                self.pc += 1;
            }
            (0x9B, _, _, RegSize::TwoBytes) => {
                self.txy_x16();
                self.pc += 1;
            }

            // stz, abs
            (0x9C, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.stz_m8(addr);
                self.pc += 3;
            }
            (0x9C, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.stz_m16(addr);
                self.pc += 3;
            }

            // sta, abs,X
            (0x9D, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.sta_m8(addr);
                self.pc += 3;
            }
            (0x9D, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.sta_m16(addr);
                self.pc += 3;
            }

            // stz, abs,X
            (0x9E, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.stz_m8(addr);
                self.pc += 3;
            }
            (0x9E, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.stz_m16(addr);
                self.pc += 3;
            }

            // sta, long,X
            (0x9F, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.sta_m8(addr);
                self.pc += 4;
            }
            (0x9F, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.sta_m16(addr);
                self.pc += 4;
            }

            // ldy, imm
            (0xA0, _, _, RegSize::Byte) => {
                let addr = self.immediate8();
                self.ldy_x8(addr);
                self.pc += 2;
            }
            (0xA0, _, _, RegSize::TwoBytes) => {
                let addr = self.immediate16();
                self.ldy_x16(addr);
                self.pc += 3;
            }

            // lda, (dir,X)
            (0xA1, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xA1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // ldx, imm
            (0xA2, _, _, RegSize::Byte) => {
                let addr = self.immediate8();
                self.ldx_x8(addr);
                self.pc += 2;
            }
            (0xA2, _, _, RegSize::TwoBytes) => {
                let addr = self.immediate16();
                self.ldx_x16(addr);
                self.pc += 3;
            }

            // lda, stk,S
            (0xA3, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xA3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // ldy, dir
            (0xA4, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.ldy_x8(addr);
                self.pc += 2;
            }
            (0xA4, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.ldy_x16(addr);
                self.pc += 2;
            }

            // lda, dir
            (0xA5, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xA5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // ldx, dir
            (0xA6, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.ldx_x8(addr);
                self.pc += 2;
            }
            (0xA6, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.ldx_x16(addr);
                self.pc += 2;
            }

            // lda, [dir]
            (0xA7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xA7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // tay, imp
            (0xA8, _, _, RegSize::Byte) => {
                self.tay_x8();
                self.pc += 1;
            }
            (0xA8, _, _, RegSize::TwoBytes) => {
                self.tay_x16();
                self.pc += 1;
            }

            // lda, imm
            (0xA9, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xA9, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.lda_m16(addr);
                self.pc += 3;
            }

            // tax, imp
            (0xAA, _, _, RegSize::Byte) => {
                self.tax_x8();
                self.pc += 1;
            }
            (0xAA, _, _, RegSize::TwoBytes) => {
                self.tax_x16();
                self.pc += 1;
            }

            // plb, imp
            (0xAB, CpuMode::Emulation, ..) => {
                self.plb_e();
                self.pc += 1;
            }
            (0xAB, CpuMode::Native, ..) => {
                self.plb_n();
                self.pc += 1;
            }

            // ldy, abs
            (0xAC, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.ldy_x8(addr);
                self.pc += 3;
            }
            (0xAC, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.ldy_x16(addr);
                self.pc += 3;
            }

            // lda, abs
            (0xAD, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.lda_m8(addr);
                self.pc += 3;
            }
            (0xAD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.lda_m16(addr);
                self.pc += 3;
            }

            // ldx, abs
            (0xAE, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.ldx_x8(addr);
                self.pc += 3;
            }
            (0xAE, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.ldx_x16(addr);
                self.pc += 3;
            }

            // lda, long
            (0xAF, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.lda_m8(addr);
                self.pc += 4;
            }
            (0xAF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.lda_m16(addr);
                self.pc += 4;
            }

            // bcs, rel8
            (0xB0, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bcs_all(addr);
            }

            // lda, (dir),Y
            (0xB1, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xB1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // lda, (dir)
            (0xB2, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xB2, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // lda, (stk,S),Y
            (0xB3, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xB3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // ldy, dir,X
            (0xB4, _, _, RegSize::Byte) => {
                let addr = self.direct_x8();
                self.ldy_x8(addr);
                self.pc += 2;
            }
            (0xB4, _, _, RegSize::TwoBytes) => {
                let addr = self.direct_x16();
                self.ldy_x16(addr);
                self.pc += 2;
            }

            // lda, dir,X
            (0xB5, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xB5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // ldx, dir,Y
            (0xB6, _, _, RegSize::Byte) => {
                let addr = self.direct_y8();
                self.ldx_x8(addr);
                self.pc += 2;
            }
            (0xB6, _, _, RegSize::TwoBytes) => {
                let addr = self.direct_y16();
                self.ldx_x16(addr);
                self.pc += 2;
            }

            // lda, [dir],Y
            (0xB7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.lda_m8(addr);
                self.pc += 2;
            }
            (0xB7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.lda_m16(addr);
                self.pc += 2;
            }

            // clv, imp
            (0xB8, ..) => {
                self.clv_all();
                self.pc += 1;
            }

            // lda, abs,Y
            (0xB9, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.lda_m8(addr);
                self.pc += 3;
            }
            (0xB9, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.lda_m16(addr);
                self.pc += 3;
            }

            // tsx, imp
            (0xBA, _, _, RegSize::Byte) => {
                self.tsx_x8();
                self.pc += 1;
            }
            (0xBA, _, _, RegSize::TwoBytes) => {
                self.tsx_x16();
                self.pc += 1;
            }

            // tyx, imp
            (0xBB, _, _, RegSize::Byte) => {
                self.tyx_x8();
                self.pc += 1;
            }
            (0xBB, _, _, RegSize::TwoBytes) => {
                self.tyx_x16();
                self.pc += 1;
            }

            // ldy, abs,X
            (0xBC, _, _, RegSize::Byte) => {
                let addr = self.absolute_x8();
                self.ldy_x8(addr);
                self.pc += 3;
            }
            (0xBC, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute_x16();
                self.ldy_x16(addr);
                self.pc += 3;
            }

            // lda, abs,X
            (0xBD, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.lda_m8(addr);
                self.pc += 3;
            }
            (0xBD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.lda_m16(addr);
                self.pc += 3;
            }

            // ldx, abs,Y
            (0xBE, _, _, RegSize::Byte) => {
                let addr = self.absolute_y8();
                self.ldx_x8(addr);
                self.pc += 3;
            }
            (0xBE, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute_y16();
                self.ldx_x16(addr);
                self.pc += 3;
            }

            // lda, long,X
            (0xBF, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.lda_m8(addr);
                self.pc += 4;
            }
            (0xBF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.lda_m16(addr);
                self.pc += 4;
            }

            // cpy, imm
            (0xC0, _, _, RegSize::Byte) => {
                let addr = self.immediate8();
                self.cpy_x8(addr);
                self.pc += 2;
            }
            (0xC0, _, _, RegSize::TwoBytes) => {
                let addr = self.immediate16();
                self.cpy_x16(addr);
                self.pc += 3;
            }

            // cmp, (dir,X)
            (0xC1, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xC1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // rep, imm
            (0xC2, CpuMode::Emulation, ..) => {
                let addr = self.immediate8();
                self.rep_e(addr);
                self.pc += 2;
            }
            (0xC2, CpuMode::Native, ..) => {
                let addr = self.immediate8();
                self.rep_n(addr);
                self.pc += 2;
            }

            // cmp, stk,S
            (0xC3, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xC3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // cpy, dir
            (0xC4, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.cpy_x8(addr);
                self.pc += 2;
            }
            (0xC4, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.cpy_x16(addr);
                self.pc += 2;
            }

            // cmp, dir
            (0xC5, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xC5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // dec, dir
            (0xC6, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.dec_mem_m8(addr);
                self.pc += 2;
            }
            (0xC6, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.dec_mem_m16(addr);
                self.pc += 2;
            }

            // cmp, [dir]
            (0xC7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xC7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // iny, imp
            (0xC8, _, _, RegSize::Byte) => {
                self.iny_x8();
                self.pc += 1;
            }
            (0xC8, _, _, RegSize::TwoBytes) => {
                self.iny_x16();
                self.pc += 1;
            }

            // cmp, imm
            (0xC9, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xC9, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.cmp_m16(addr);
                self.pc += 3;
            }

            // dex, imp
            (0xCA, _, _, RegSize::Byte) => {
                self.dex_x8();
                self.pc += 1;
            }
            (0xCA, _, _, RegSize::TwoBytes) => {
                self.dex_x16();
                self.pc += 1;
            }

            // wai, imp
            (0xCB, ..) => {
                self.wai_all();
                self.pc += 1;
            }

            // cpy, abs
            (0xCC, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.cpy_x8(addr);
                self.pc += 3;
            }
            (0xCC, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.cpy_x16(addr);
                self.pc += 3;
            }

            // cmp, abs
            (0xCD, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.cmp_m8(addr);
                self.pc += 3;
            }
            (0xCD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.cmp_m16(addr);
                self.pc += 3;
            }

            // dec, abs
            (0xCE, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.dec_mem_m8(addr);
                self.pc += 3;
            }
            (0xCE, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.dec_mem_m16(addr);
                self.pc += 3;
            }

            // cmp, long
            (0xCF, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.cmp_m8(addr);
                self.pc += 4;
            }
            (0xCF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.cmp_m16(addr);
                self.pc += 4;
            }

            // bne, rel8
            (0xD0, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.bne_all(addr);
            }

            // cmp, (dir),Y
            (0xD1, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xD1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // cmp, (dir)
            (0xD2, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xD2, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // cmp, (stk,S),Y
            (0xD3, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xD3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // pex, dir
            (0xD4, CpuMode::Emulation, ..) => {
                let addr = self.direct16();
                self.pex_e(addr);
                self.pc += 2;
            }
            (0xD4, CpuMode::Native, ..) => {
                let addr = self.direct16();
                self.pex_n(addr);
                self.pc += 2;
            }

            // cmp, dir,X
            (0xD5, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xD5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // dec, dir,X
            (0xD6, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.dec_mem_m8(addr);
                self.pc += 2;
            }
            (0xD6, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.dec_mem_m16(addr);
                self.pc += 2;
            }

            // cmp, [dir],Y
            (0xD7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.cmp_m8(addr);
                self.pc += 2;
            }
            (0xD7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.cmp_m16(addr);
                self.pc += 2;
            }

            // cld, imp
            (0xD8, ..) => {
                self.cld_all();
                self.pc += 1;
            }

            // cmp, abs,Y
            (0xD9, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.cmp_m8(addr);
                self.pc += 3;
            }
            (0xD9, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.cmp_m16(addr);
                self.pc += 3;
            }

            // phx, imp
            (0xDA, CpuMode::Emulation, ..) => {
                self.phx_e();
                self.pc += 1;
            }
            (0xDA, _, _, RegSize::Byte) => {
                self.phx_x8();
                self.pc += 1;
            }
            (0xDA, _, _, RegSize::TwoBytes) => {
                self.phx_x16();
                self.pc += 1;
            }

            // stp, imp
            (0xDB, ..) => {
                self.stp_all();
                self.pc += 1;
            }

            // jmp, [abs]
            (0xDC, ..) => {
                let addr = self.absolute_indirect_long();
                self.jmp_all(addr);
            }

            // cmp, abs,X
            (0xDD, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.cmp_m8(addr);
                self.pc += 3;
            }
            (0xDD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.cmp_m16(addr);
                self.pc += 3;
            }

            // dec, abs,X
            (0xDE, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.dec_mem_m8(addr);
                self.pc += 3;
            }
            (0xDE, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.dec_mem_m16(addr);
                self.pc += 3;
            }

            // cmp, long,X
            (0xDF, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.cmp_m8(addr);
                self.pc += 4;
            }
            (0xDF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.cmp_m16(addr);
                self.pc += 4;
            }

            // cpx, imm
            (0xE0, _, _, RegSize::Byte) => {
                let addr = self.immediate8();
                self.cpx_x8(addr);
                self.pc += 2;
            }
            (0xE0, _, _, RegSize::TwoBytes) => {
                let addr = self.immediate16();
                self.cpx_x16(addr);
                self.pc += 3;
            }

            // sbc, (dir,X)
            (0xE1, _, RegSize::Byte, _) => {
                let addr = self.direct_x_indirect8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xE1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x_indirect16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // sep, imm
            (0xE2, ..) => {
                let addr = self.immediate8();
                self.sep_all(addr);
                self.pc += 2;
            }

            // sbc, stk,S
            (0xE3, _, RegSize::Byte, _) => {
                let addr = self.stack_s8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xE3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_s16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // cpx, dir
            (0xE4, _, _, RegSize::Byte) => {
                let addr = self.direct8();
                self.cpx_x8(addr);
                self.pc += 2;
            }
            (0xE4, _, _, RegSize::TwoBytes) => {
                let addr = self.direct16();
                self.cpx_x16(addr);
                self.pc += 2;
            }

            // sbc, dir
            (0xE5, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xE5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // inc, dir
            (0xE6, _, RegSize::Byte, _) => {
                let addr = self.direct8();
                self.inc_mem_m8(addr);
                self.pc += 2;
            }
            (0xE6, _, RegSize::TwoBytes, _) => {
                let addr = self.direct16();
                self.inc_mem_m16(addr);
                self.pc += 2;
            }

            // sbc, [dir]
            (0xE7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xE7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // inx, imp
            (0xE8, _, _, RegSize::Byte) => {
                self.inx_x8();
                self.pc += 1;
            }
            (0xE8, _, _, RegSize::TwoBytes) => {
                self.inx_x16();
                self.pc += 1;
            }

            // sbc, imm
            (0xE9, _, RegSize::Byte, _) => {
                let addr = self.immediate8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xE9, _, RegSize::TwoBytes, _) => {
                let addr = self.immediate16();
                self.sbc_m16(addr);
                self.pc += 3;
            }

            // nop, imp
            (0xEA, ..) => {
                self.nop_all();
                self.pc += 1;
            }

            // xba, imp
            (0xEB, ..) => {
                self.xba_all();
                self.pc += 1;
            }

            // cpx, abs
            (0xEC, _, _, RegSize::Byte) => {
                let addr = self.absolute8();
                self.cpx_x8(addr);
                self.pc += 3;
            }
            (0xEC, _, _, RegSize::TwoBytes) => {
                let addr = self.absolute16();
                self.cpx_x16(addr);
                self.pc += 3;
            }

            // sbc, abs
            (0xED, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.sbc_m8(addr);
                self.pc += 3;
            }
            (0xED, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.sbc_m16(addr);
                self.pc += 3;
            }

            // inc, abs
            (0xEE, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.inc_mem_m8(addr);
                self.pc += 3;
            }
            (0xEE, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.inc_mem_m16(addr);
                self.pc += 3;
            }

            // sbc, long
            (0xEF, _, RegSize::Byte, _) => {
                let addr = self.absolute8();
                self.sbc_m8(addr);
                self.pc += 4;
            }
            (0xEF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute16();
                self.sbc_m16(addr);
                self.pc += 4;
            }

            // beq, rel8
            (0xF0, ..) => {
                let addr = self.relative8();
                self.pc += 2;
                self.beq_all(addr);
            }

            // sbc, (dir),Y
            (0xF1, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_y8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xF1, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_y16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // sbc, (dir)
            (0xF2, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xF2, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // sbc, (stk,S),Y
            (0xF3, _, RegSize::Byte, _) => {
                let addr = self.stack_indirect_y8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xF3, _, RegSize::TwoBytes, _) => {
                let addr = self.stack_indirect_y16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // pex, imm
            (0xF4, CpuMode::Emulation, ..) => {
                let addr = self.immediate16();
                self.pex_e(addr);
                self.pc += 3;
            }
            (0xF4, CpuMode::Native, ..) => {
                let addr = self.immediate16();
                self.pex_n(addr);
                self.pc += 3;
            }

            // sbc, dir,X
            (0xF5, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xF5, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // inc, dir,X
            (0xF6, _, RegSize::Byte, _) => {
                let addr = self.direct_x8();
                self.inc_mem_m8(addr);
                self.pc += 2;
            }
            (0xF6, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_x16();
                self.inc_mem_m16(addr);
                self.pc += 2;
            }

            // sbc, [dir],Y
            (0xF7, _, RegSize::Byte, _) => {
                let addr = self.direct_indirect_long_y8();
                self.sbc_m8(addr);
                self.pc += 2;
            }
            (0xF7, _, RegSize::TwoBytes, _) => {
                let addr = self.direct_indirect_long_y16();
                self.sbc_m16(addr);
                self.pc += 2;
            }

            // sed, imp
            (0xF8, ..) => {
                self.sed_all();
                self.pc += 1;
            }

            // sbc, abs,Y
            (0xF9, _, RegSize::Byte, _) => {
                let addr = self.absolute_y8();
                self.sbc_m8(addr);
                self.pc += 3;
            }
            (0xF9, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_y16();
                self.sbc_m16(addr);
                self.pc += 3;
            }

            // plx, imp
            (0xFA, CpuMode::Emulation, ..) => {
                self.plx_e();
                self.pc += 1;
            }
            (0xFA, _, _, RegSize::Byte) => {
                self.plx_x8();
                self.pc += 1;
            }
            (0xFA, _, _, RegSize::TwoBytes) => {
                self.plx_x16();
                self.pc += 1;
            }

            // xce, imp
            (0xFB, ..) => {
                self.xce_all();
                self.pc += 1;
            }

            // jsr, (abs,X)
            (0xFC, CpuMode::Emulation, ..) => {
                let addr = self.absolute_x_indirect8();
                self.jsr_e(addr);
            }
            (0xFC, CpuMode::Native, ..) => {
                let addr = self.absolute_x_indirect8();
                self.jsr_n(addr);
            }

            // sbc, abs,X
            (0xFD, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.sbc_m8(addr);
                self.pc += 3;
            }
            (0xFD, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.sbc_m16(addr);
                self.pc += 3;
            }

            // inc, abs,X
            (0xFE, _, RegSize::Byte, _) => {
                let addr = self.absolute_x8();
                self.inc_mem_m8(addr);
                self.pc += 3;
            }
            (0xFE, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_x16();
                self.inc_mem_m16(addr);
                self.pc += 3;
            }

            // sbc, long,X
            (0xFF, _, RegSize::Byte, _) => {
                let addr = self.absolute_long_x8();
                self.sbc_m8(addr);
                self.pc += 4;
            }
            (0xFF, _, RegSize::TwoBytes, _) => {
                let addr = self.absolute_long_x16();
                self.sbc_m16(addr);
                self.pc += 4;
            }
        }
    
        if self.branch_taken {
            self.add_clocks(Cpu65c816::ONE_CYCLE);

            if self.page_crossed && self.mode == CpuMode::Emulation {
                self.add_clocks(Cpu65c816::ONE_CYCLE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::cartridge::Cartridge;

    /// Prints out a slice of bytes in hex and ASCII format, side by side. When
    /// startval is specified, indices beginning at the startval will be printed
    /// before each line. If startval is unspecified, indeces start at 0.
    pub fn hexdump_at(bytes: &[u8], startval: usize) {
        const CHUNK_SIZE: usize = 16;

        let mut index = startval;
        println!();
        for chunk in bytes.chunks(CHUNK_SIZE) {
            let l = chunk.len();
            print!("{:08X}: ", index);
            for b in chunk.iter() {
                print!("{b:02X} ");
            }

            print!("{:>width$} ", "|", width = (CHUNK_SIZE - l) * 3 + 1);
            for b in chunk.iter() {
                match b {
                    32..=126 => print!("{}", *b as char),
                    _ => print!("."),
                }
            }
            println!();
            index += CHUNK_SIZE;
        }
    }

    /// Prints out a slice of bytes in hex and ASCII format, side by side. When
    /// startval is specified, indeces beginning at the startval will be printed
    /// before each line. If startval is unspecified, indeces start at 0.
    pub fn hexdump(bytes: &[u8]) {
        hexdump_at(bytes, 0);
    }

    /// Find the subvector "needle" in the vector "haystack"
    fn find_subvec(haystack: &Vec<u8>, needle: &Vec<u8>) -> Option<usize> {
        (0..haystack.len() - needle.len() + 1)
            .filter(|&i| haystack[i..i + needle.len()] == needle[..])
            .next()
    }

    fn cpu_status_str(cpu: &Cpu65c816) -> String {
        let mut status_str = String::new();
        status_str.push(if cpu.is_flag_set(Flag::FlagN) {
            'N'
        } else {
            'n'
        });
        status_str.push(if cpu.is_flag_set(Flag::FlagV) {
            'V'
        } else {
            'v'
        });
        if cpu.mode == CpuMode::Emulation {
            status_str.push('1');
            status_str.push(if cpu.is_flag_set(Flag::FlagX) {
                'B'
            } else {
                'b'
            });
        } else {
            status_str.push(if cpu.is_flag_set(Flag::FlagM) {
                'M'
            } else {
                'm'
            });
            status_str.push(if cpu.is_flag_set(Flag::FlagX) {
                'X'
            } else {
                'x'
            });
        }
        status_str.push(if cpu.is_flag_set(Flag::FlagD) {
            'D'
        } else {
            'd'
        });
        status_str.push(if cpu.is_flag_set(Flag::FlagI) {
            'I'
        } else {
            'i'
        });
        status_str.push(if cpu.is_flag_set(Flag::FlagZ) {
            'Z'
        } else {
            'z'
        });
        status_str.push(if cpu.is_flag_set(Flag::FlagC) {
            'C'
        } else {
            'c'
        });

        status_str
    }

    fn lemon_cpu_str(cpu: &Cpu65c816) -> String {
        format!(
            "{:02x}{:04x} A:{:04x} X:{:04x} Y:{:04x} S:{:04x} D:{:04x} DB:{:02x} {} ",
            cpu.prg_bank,
            cpu.pc,
            cpu.acc,
            cpu.x,
            cpu.y,
            cpu.stk_ptr,
            cpu.direct_page,
            cpu.data_bank,
            cpu_status_str(cpu)
        )
    }

    const INSTR_NAMES: [&str; 256] = [
        "BRK", "ORA", "COP", "ORA", "TSB", "ORA", "ASL", "ORA", "PHP", "ORA", "ASL", "PHD", "TSB",
        "ORA", "ASL", "ORA", "BPL", "ORA", "ORA", "ORA", "TRB", "ORA", "ASL", "ORA", "CLC", "ORA",
        "INC", "TCS", "TRB", "ORA", "ASL", "ORA", "JSR", "AND", "JSL", "AND", "BIT", "AND", "ROL",
        "AND", "PLP", "AND", "ROL", "PLD", "BIT", "AND", "ROL", "AND", "BMI", "AND", "AND", "AND",
        "BIT", "AND", "ROL", "AND", "SEC", "AND", "DEC", "TSC", "BIT", "AND", "ROL", "AND", "RTI",
        "EOR", "WDM", "EOR", "MVP", "EOR", "LSR", "EOR", "PHA", "EOR", "LSR", "PHK", "JMP", "EOR",
        "LSR", "EOR", "BVC", "EOR", "EOR", "EOR", "MVN", "EOR", "LSR", "EOR", "CLI", "EOR", "PHY",
        "TCD", "JMP", "EOR", "LSR", "EOR", "RTS", "ADC", "PEX", "ADC", "STZ", "ADC", "ROR", "ADC",
        "PLA", "ADC", "ROR", "RTL", "JMP", "ADC", "ROR", "ADC", "BVS", "ADC", "ADC", "ADC", "STZ",
        "ADC", "ROR", "ADC", "SEI", "ADC", "PLY", "TDC", "JMP", "ADC", "ROR", "ADC", "BRA", "STA",
        "BRA", "STA", "STY", "STA", "STX", "STA", "DEY", "BIT", "TXA", "PHB", "STY", "STA", "STX",
        "STA", "BCC", "STA", "STA", "STA", "STY", "STA", "STX", "STA", "TYA", "STA", "TXS", "TXY",
        "STZ", "STA", "STZ", "STA", "LDY", "LDA", "LDX", "LDA", "LDY", "LDA", "LDX", "LDA", "TAY",
        "LDA", "TAX", "PLB", "LDY", "LDA", "LDX", "LDA", "BCS", "LDA", "LDA", "LDA", "LDY", "LDA",
        "LDX", "LDA", "CLV", "LDA", "TSX", "TYX", "LDY", "LDA", "LDX", "LDA", "CPY", "CMP", "REP",
        "CMP", "CPY", "CMP", "DEC", "CMP", "INY", "CMP", "DEX", "WAI", "CPY", "CMP", "DEC", "CMP",
        "BNE", "CMP", "CMP", "CMP", "PEX", "CMP", "DEC", "CMP", "CLD", "CMP", "PHX", "STP", "JMP",
        "CMP", "DEC", "CMP", "CPX", "SBC", "SEP", "SBC", "CPX", "SBC", "INC", "SBC", "INX", "SBC",
        "NOP", "XBA", "CPX", "SBC", "INC", "SBC", "BEQ", "SBC", "SBC", "SBC", "PEX", "SBC", "INC",
        "SBC", "SED", "SBC", "PLX", "XCE", "JSR", "SBC", "INC", "SBC",
    ];

    #[test]
    fn test_lorom_title() {
        let test_path = Path::new("tests/blarggs/test_adc_sbc/test_adc.smc");
        let cart = Cartridge::from_path_with_mode(test_path, MappingMode::LoROM).unwrap();

        let mut cpu = Cpu65c816::new();

        cpu.load_cart(&cart);

        // let expected_name = b"65C816 TEST          ";

        // println!("ROM Size: {}", cpu.rom.len());

        hexdump_at(&cpu.rom[0x8000..0x8000 + 0x1000], 0x8000);

        // for i in 0..21 {
        //     let expected_char = expected_name[i];
        //     let actual_char = cpu.read(0x00FFC0 + i as u32);

        //     assert_eq!(expected_char, actual_char);
        // }
    }

    // #[test]
    // fn test_cpubasic() {
    //     let test_path = Path::new("tests/blarggs/test_adc_sbc/test_adc.smc");
    //     let cart = Cartridge::from_path(test_path).unwrap();

    //     let mut cpu = Cpu65c816::new();

    //     cpu.load_cart(&cart);

    //     // cpu.reset();
    //     cpu.pc = 0x8000;

    //     for _ in 0..100 {
    //         let opcode = cpu.read_prg();
    //         let (addr_lo, addr_hi) = cpu.immediate16();
    //         let val1 = cpu.read(addr_lo);
    //         let val2 = cpu.read(addr_hi);
    //         println!("PRG BANK: 0x{:02X}, PC: 0x{:04X}, INSTR: {} (0x{:02X}), IMM16: 0x{:02X} 0x{:02X}, IDX SIZE: {:?}, ACC SIZE: {:?}, X: 0x{:04X}, Y: 0x{:04X}, A: 0x{:04X}", cpu.prg_bank, cpu.pc, INSTR_NAMES[opcode as usize], opcode, val1, val2, cpu.idx_size(), cpu.acc_size(), cpu.x, cpu.y, cpu.acc);
    //         // let dir8 = cpu.direct8();
    //         // let val3 = cpu.read(dir8);
    //         // println!("    STK PTR: 0x{:04X}, DIR PAGE: 0x{:04X}, DATA BANK: 0x{:02X}, DIR8: 0x{:02X}", cpu.stk_ptr, cpu.direct_page, cpu.data_bank, val3);
    //         cpu.exec_instr();
    //     }
    // }

    fn run_lemon_test(test_name: &str) {
        let test_path_str = format!("tests/lemons/CPUTest/{test_name}.sfc");
        let test_path = Path::new(&test_path_str);
        let cart = Cartridge::from_path(test_path).unwrap();

        let log_path_str = format!("tests/lemons/CPUTest/{test_name}-trace_compare.log");
        let log_path = Path::new(&log_path_str);
        let log_lines: Vec<String> = std::fs::read_to_string(log_path)
            .unwrap()
            .lines()
            .map(String::from)
            .collect();

        let mut cpu = Cpu65c816::new();
        cpu.load_cart(&cart);

        cpu.stk_ptr = 0x1ff;
        cpu.status = 0x34;

        cpu.rom_mirror = cpu.rom.len() - 1;

        cpu.pc = 0x8000;

        cpu.wram[0] = 0xb5;

        for (i, line) in log_lines.iter().enumerate() {
            let opcode = cpu.read_prg();
            let (addr_lo, addr_hi) = cpu.immediate16();
            let val1 = cpu.read(addr_lo);
            let val2 = cpu.read(addr_hi);
            // let cpu_str = format!("ADDR: 0x{:02X}{:04X}, INSTR: {} (0x{:02X}), IMM16: 0x{:02X} 0x{:02X}, X: 0x{:04X}, Y: 0x{:04X}, A: 0x{:04X}, SP: {:04X}, P: {}, D: 0x{:04X}", 
            //     cpu.prg_bank,
            //     cpu.pc,
            //     INSTR_NAMES[opcode as usize],
            //     opcode,
            //     val1,
            //     val2,
            //     cpu.x,
            //     cpu.y,
            //     cpu.acc,
            //     cpu.stk_ptr,
            //     cpu_status_str(&cpu),
            //     cpu.direct_page,
            // );
            // println!("{}", cpu_str);

            // Quick hack for running this test
            if opcode == 0x2C && val1 == 0x10 && val2 == 0x42 {
                cpu.debug_nmi = if log_lines[i+1].as_bytes()[48] == b'N' {
                    0xc2
                } else {
                    0x42
                }
            }

            assert_eq!(*line, lemon_cpu_str(&cpu));
            
            cpu.exec_instr();
        }
    }


    #[test]
    fn test_lemon_all() {
        let paths = std::fs::read_dir("./tests/lemons/CPUTest").unwrap();

        for path in paths {
            if let Ok(path) = path {
                let file_name = String::from(path.file_name().to_str().unwrap());
                     
                if let Some(test_name) = file_name.strip_suffix(".sfc") {
                    if test_name == "CPUMSC" {
                        println!("cpumsc [[SKIPPED - PPU Dependent]]");
                        continue;
                    }
    
                    run_lemon_test(test_name);
                
                    println!("{} [[PASSED]]", test_name.to_lowercase());
                }
            }
        }
    }
}