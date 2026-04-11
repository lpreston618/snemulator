use std::marker::PhantomData;

use crate::{probe::DebugProbe, scpu::bus::CpuBus};

pub mod bus;
pub mod disassembler;
pub mod dma;
mod instructions;
pub mod ioregs;
pub mod mult;

pub use bus::Address;

#[derive(Clone, Copy)]
pub enum Flag {
    FlagC = 1 << 0, // Carry
    FlagZ = 1 << 1, // Zero
    FlagI = 1 << 2, // IRQ Disable
    FlagD = 1 << 3, // Decimal Mode
    FlagX = 1 << 4, // X Register Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagM = 1 << 5, // Accumulator Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagV = 1 << 6, // Overflow
    FlagN = 1 << 7, // Negative
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CpuInterrupt {
    IRQ,
    NMI,
    BRK,
    COP,
    Reset,
    Abort,
}

pub struct Cpu65c816<P: DebugProbe> {
    // Internal Registers
    pub a: u16,  // Accumulator
    pub x: u16,  // X index
    pub y: u16,  // Y index
    pub sp: u16, // Stack pointer
    pub pc: u16, // Program counter
    pub pb: u8,  // Program bank
    pub db: u8,  // Data bank
    pub dp: u16, // Direct page
    pub p: u8,   // Processor status
    pub e: bool, // Emulation mode

    // Internal state
    pub halted: bool,
    pub stopped: bool,
    pub waiting_for_interrupt: bool,
    pub irq_pending: bool,
    pub nmi_pending: bool,

    /// The number of clocks before the next instruction is executed
    pub clocks: usize,

    pub branch_taken: bool,
    pub page_crossed: bool,
    
    _phantom_probe: PhantomData<P>,
}

// SNES System Functionality
impl<P: DebugProbe> Cpu65c816<P> {
    /// Number of system clocks in a single slow cpu cycle (e.g. a typical bus read/write)
    const SLOW_CYCLE_CLOCKS: usize = 8;
    /// Number of system clocks in a single cpu cycle
    const CYCLE_CLOCKS: usize = 6;

    // Creates a new, uninitialized 65c816 CPU
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            pb: 0,
            db: 0,
            dp: 0,
            p: 0,
            e: false,

            // Internal state
            halted: false,
            waiting_for_interrupt: false,
            irq_pending: false,
            nmi_pending: false,

            // Cycle tracking
            clocks: 0,

            branch_taken: false,
            page_crossed: false,
            stopped: false,
            
            _phantom_probe: PhantomData {},
        }
    }

    /// Sets the CPU to its proper initial state.
    pub fn power_on(&mut self, bus: &mut CpuBus<P>) {
        self.x = 0;
        self.y = 0;
        self.db = 0;
        self.pb = 0;
        self.dp = 0;
        self.sp = 0x0100;
        self.p = 0x34;
        self.e = true;
        self.halted = false;
        self.waiting_for_interrupt = false;
        self.irq_pending = false;
        self.nmi_pending = false;
        self.stopped = false;
        self.handle_interrupt(bus, CpuInterrupt::Reset); // TODO: Check this?
    }

    pub fn reset(&mut self, bus: &mut CpuBus<P>) {
        self.stopped = false;
        self.irq_pending = false;
        self.nmi_pending = false;
        self.waiting_for_interrupt = false;
        self.handle_interrupt(bus, CpuInterrupt::Reset);
    }

    /// Cycles the cpu for a given number of clocks. If the number of clocks is 0 after cycling, the next instructions is executed.
    pub fn cycle(&mut self, bus: &mut CpuBus<P>) {
        if self.nmi_pending {
            self.handle_interrupt(bus, CpuInterrupt::NMI);
            self.nmi_pending = false;
            self.waiting_for_interrupt = false;
            return;
        }

        if self.irq_pending && !self.is_flag_set(Flag::FlagI) {
            self.handle_interrupt(bus, CpuInterrupt::IRQ);
            self.waiting_for_interrupt = false;
            return;
        }

        if self.stopped || self.halted || self.waiting_for_interrupt {
            self.clocks += Self::CYCLE_CLOCKS;
            return;
        }

        self.execute(bus);
    }

    pub fn handle_interrupt(&mut self, bus: &mut CpuBus<P>, interrupt: CpuInterrupt) {
        bus.probe.on_interrupt(interrupt);

        match interrupt {
            CpuInterrupt::Reset => {
                self.e = true;
                self.set_flag_to_bool(Flag::FlagM, true);
                self.set_flag_to_bool(Flag::FlagX, true);
                self.sp = 0x100 | (self.sp & 0xFF);
            }
            CpuInterrupt::BRK => {
                panic!("BRK");
            }
            _ => {}
        }

        if !self.e {
            self.push(bus, self.pb);
        }

        self.push_word(bus, self.pc);
        self.push(bus, self.p);

        self.set_flag_to_bool(Flag::FlagI, true);
        self.set_flag_to_bool(Flag::FlagD, false);

        let vector_offset = if self.e {
            match interrupt {
                CpuInterrupt::IRQ => 0xFFFE,
                CpuInterrupt::NMI => 0xFFFA,
                CpuInterrupt::BRK => 0xFFFE,
                CpuInterrupt::COP => 0xFFF4,
                CpuInterrupt::Reset => 0xFFFC,
                CpuInterrupt::Abort => 0xFFF8,
            }
        } else {
            match interrupt {
                CpuInterrupt::IRQ => 0xFFEE,
                CpuInterrupt::NMI => 0xFFEA,
                CpuInterrupt::BRK => 0xFFE6,
                CpuInterrupt::COP => 0xFFE4,
                CpuInterrupt::Reset => unreachable!(), // reset interrupt sets e flag
                CpuInterrupt::Abort => 0xFFE8,
            }
        };

        let vector_lo = Address {
            bank: 0,
            offset: vector_offset,
        };
        let vector_hi = Address {
            bank: 0,
            offset: vector_offset + 1,
        };

        self.pb = 0;
        self.pc = self.read_word(bus, vector_lo, vector_hi);
    }
}

// Bus/flag access
impl<P: DebugProbe> Cpu65c816<P> {
    /// Read a byte from the bus at a given address. Adds to cpu clocks.
    fn read(&mut self, bus: &mut CpuBus<P>, addr: Address) -> u8 {
        self.clocks += Self::SLOW_CYCLE_CLOCKS;
        let value = bus.read(addr);
        bus.probe.on_memory_read(addr.to_u32(), value);
        value
    }

    /// Write a byte to the bus at a given address. Adds to cpu clocks.
    fn write(&mut self, bus: &mut CpuBus<P>, addr: Address, value: u8) {
        self.clocks += Self::SLOW_CYCLE_CLOCKS;
        bus.probe.on_memory_write(addr.to_u32(), value);
        bus.write(addr, value);
    }

    fn read_prg(&mut self, bus: &mut CpuBus<P>) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.clocks += Self::SLOW_CYCLE_CLOCKS;
        bus.read(Address {
            bank: self.pb,
            offset: pc,
        })
    }

    fn read_word(&mut self, bus: &mut CpuBus<P>, addr_lo: Address, addr_hi: Address) -> u16 {
        u16::from_le_bytes([self.read(bus, addr_lo), self.read(bus, addr_hi)])
    }

    fn write_word(&mut self, bus: &mut CpuBus<P>, addr_lo: Address, addr_hi: Address, value: u16) {
        self.write(bus, addr_lo, value as u8);
        self.write(bus, addr_hi, (value >> 8) as u8);
    }

    fn pop(&mut self, bus: &mut CpuBus<P>) -> u8 {
        self.sp += 1;

        if self.e {
            self.sp = 0x100 | (self.sp & 0xFF);
        }

        self.read(
            bus,
            Address {
                bank: 0,
                offset: self.sp,
            },
        )
    }

    fn push(&mut self, bus: &mut CpuBus<P>, value: u8) {
        self.write(
            bus,
            Address {
                bank: 0,
                offset: self.sp,
            },
            value,
        );

        self.sp -= 1;

        if self.e {
            self.sp = 0x100 | (self.sp & 0xFF);
        }
    }

    fn pop_word(&mut self, bus: &mut CpuBus<P>) -> u16 {
        u16::from_le_bytes([self.pop(bus), self.pop(bus)])
    }

    fn push_word(&mut self, bus: &mut CpuBus<P>, value: u16) {
        self.push(bus, (value >> 8) as u8);
        self.push(bus, value as u8);
    }

    pub fn is_flag_set(&self, flag: Flag) -> bool {
        self.p & (flag as u8) != 0
    }

    fn set_flag_to_bool(&mut self, flag: Flag, value: bool) {
        if value {
            self.p |= flag as u8;
        } else {
            self.p &= !(flag as u8);
        }
    }
}
