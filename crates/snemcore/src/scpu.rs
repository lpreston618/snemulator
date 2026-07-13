use crate::{debug::DebugHarness, savestate, scpu::bus::CpuBus};

mod instructions;
mod tests;
pub mod bus;
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CpuInterrupt {
    IRQ,
    NMI,
    BRK,
    COP,
    Reset,
    Abort,
}

pub struct Cpu65c816 {
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

    /// The number of clocks before the next instruction is executed
    pub clocks: usize,

    pub branch_taken: bool,
    pub page_crossed: bool,

    pub prg_bytes: Vec<u8>,
}

// SNES System Functionality
impl Cpu65c816 {
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

            // Cycle tracking
            clocks: 0,

            branch_taken: false,
            page_crossed: false,
            stopped: false,

            prg_bytes: Vec::with_capacity(4),
        }
    }

    pub fn save_state(&self) -> savestate::CpuState {
        savestate::CpuState {
            a: self.a,
            x: self.x,
            y: self.y,
            sp: self.sp,
            pc: self.pc,
            pb: self.pb,
            db: self.db,
            dp: self.dp,
            p: self.p,
            e: self.e,
            halted: self.halted,
            stopped: self.stopped,
            waiting_for_interrupt: self.waiting_for_interrupt,
            clocks: self.clocks,
        }
    }

    pub fn load_state(&mut self, state: &savestate::CpuState, _version: u32) {
        self.a = state.a;
        self.x = state.x;
        self.y = state.y;
        self.sp = state.sp;
        self.pc = state.pc;
        self.pb = state.pb;
        self.db = state.db;
        self.dp = state.dp;
        self.p = state.p;
        self.e = state.e;
        self.halted = state.halted;
        self.stopped = state.stopped;
        self.waiting_for_interrupt = state.waiting_for_interrupt;
        self.clocks = state.clocks;
    }

    /// Sets the CPU to its proper initial state.
    pub fn power_on<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) {
        self.a = 0;
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
        self.stopped = false;
        self.handle_interrupt(bus, CpuInterrupt::Reset);
    }

    pub fn reset<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) {
        self.stopped = false;
        self.waiting_for_interrupt = false;
        self.handle_interrupt(bus, CpuInterrupt::Reset);
    }

    /// Cycles the cpu for a given number of clocks. If the number of clocks is 0 after cycling, the next instructions is executed.
    pub fn cycle<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) {
        if H::IS_DEBUGGING_HARNESS {
            self.prg_bytes.clear();
        }

        if bus.cpu_regs.nmi_pending {
            self.handle_interrupt(bus, CpuInterrupt::NMI);
            bus.cpu_regs.nmi_pending = false;
            self.waiting_for_interrupt = false;
            return;
        }

        if bus.cpu_regs.hv_timer_irq_flag && !self.is_flag_set(Flag::FlagI) {
            self.handle_interrupt(bus, CpuInterrupt::IRQ);
            self.waiting_for_interrupt = false;
            return;
        }

        if self.stopped || self.halted || self.waiting_for_interrupt {
            self.clocks += Self::CYCLE_CLOCKS;
            return;
        }

        self.execute(bus);

        if H::IS_DEBUGGING_HARNESS && H::TRACK_CPU_INSTRUCTIONS {
            bus.harness.on_instruction(self, &self.prg_bytes.clone());
        }
    }

    pub fn handle_interrupt<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, interrupt: CpuInterrupt) {
        match interrupt {
            CpuInterrupt::Reset => {
                self.e = true;
                self.set_flag_to_bool(Flag::FlagM, true);
                self.set_flag_to_bool(Flag::FlagX, true);
                self.sp = 0x100 | (self.sp & 0xFF);
                self.db = 0;
                self.dp = 0;
            }
            _ => {
                if !self.e {
                    self.push(bus, self.pb);
                }

                self.push_word(bus, self.pc);
                self.push(bus, self.p);
            }
        }

        self.set_flag_to_bool(Flag::FlagI, true);
        self.set_flag_to_bool(Flag::FlagD, false);

        let vector = bus.cart.interrupt_vector(interrupt, self.e);

        self.pb = 0;
        self.pc = vector;

        if H::IS_DEBUGGING_HARNESS && H::TRACK_CPU_INTERRUPTS {
            bus.harness.on_interrupt(self, interrupt);
        }
    }
}

// Bus/flag access
impl Cpu65c816 {
    /// Read a byte from the bus at a given address. Adds to cpu clocks.
    fn read<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, addr: Address) -> u8 {
        let cycles_taken = if bus.cpu_regs.fast_rom_en {
            if addr.bank >= 0xC0 || (addr.bank >= 0x80 && addr.offset >= 0x8000) {
                Self::CYCLE_CLOCKS
            } else {
                Self::SLOW_CYCLE_CLOCKS
            }
        } else {
            Self::SLOW_CYCLE_CLOCKS
        };

        self.clocks += cycles_taken;
        let value = bus.read(addr);

        if H::IS_DEBUGGING_HARNESS && H::TRACK_MEMORY {
            bus.harness.on_memory_read(self, addr, value);
        }

        value
    }

    /// Write a byte to the bus at a given address. Adds to cpu clocks.
    fn write<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, addr: Address, value: u8) {
        let cycles_taken = if bus.cpu_regs.fast_rom_en {
            if addr.bank >= 0xC0 || (addr.bank >= 0x80 && addr.offset >= 0x8000) {
                Self::CYCLE_CLOCKS
            } else {
                Self::SLOW_CYCLE_CLOCKS
            }
        } else {
            Self::SLOW_CYCLE_CLOCKS
        };

        self.clocks += cycles_taken;
        bus.write(addr, value);

        if H::IS_DEBUGGING_HARNESS && H::TRACK_MEMORY {
            bus.harness.on_memory_write(self, addr, value);
        }
    }

    fn read_prg<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.clocks += Self::SLOW_CYCLE_CLOCKS;
        let value = bus.read(Address {
            bank: self.pb,
            offset: pc,
        });

        if H::IS_DEBUGGING_HARNESS {
            self.prg_bytes.push(value);
        }

        value
    }

    fn read_word<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, addr_lo: Address, addr_hi: Address) -> u16 {
        u16::from_le_bytes([self.read(bus, addr_lo), self.read(bus, addr_hi)])
    }

    fn write_word<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, addr_lo: Address, addr_hi: Address, value: u16) {
        self.write(bus, addr_lo, value as u8);
        self.write(bus, addr_hi, (value >> 8) as u8);
    }

    // Pop a byte from the stack, wrapping the stack pointer in emulation mode if necessary.
    fn pop<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) -> u8 {
        self.sp += 1;

        if self.e {
            self.sp = 0x100 | (self.sp & 0xFF);
        }

        let value = self.read(
            bus,
            Address {
                bank: 0,
                offset: self.sp,
            },
        );

        if H::IS_DEBUGGING_HARNESS && H::TRACK_STACK {
            bus.harness.on_stack_pop(self, value);
        }

        value
    }

    // Pop a byte from the stack without wrapping the stack pointer in emulation mode.
    fn pop_no_wrap<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) -> u8 {
        self.sp += 1;

        let value = self.read(
            bus,
            Address {
                bank: 0,
                offset: self.sp,
            },
        );

        if H::IS_DEBUGGING_HARNESS && H::TRACK_STACK {
            bus.harness.on_stack_pop(self, value);
        }

        value
    }

    // Pop a word from the stack, wrapping the stack pointer in emulation mode if necessary.
    fn pop_word<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) -> u16 {
        u16::from_le_bytes([self.pop(bus), self.pop(bus)])
    }

    // Pop a word from the stack without wrapping the stack pointer in emulation mode.
    fn pop_word_no_wrap<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) -> u16 {
        u16::from_le_bytes([self.pop_no_wrap(bus), self.pop_no_wrap(bus)])
    }

    // Push a byte onto the stack, wrapping the stack pointer in emulation mode if necessary.
    fn push<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, value: u8) {
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

        if H::IS_DEBUGGING_HARNESS && H::TRACK_STACK {
            bus.harness.on_stack_push(self, value);
        }
    }

    /// Push a byte onto the stack without wrapping the stack pointer in emulation mode.
    fn push_no_wrap<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, value: u8) {
        self.write(
            bus,
            Address {
                bank: 0,
                offset: self.sp,
            },
            value,
        );

        self.sp -= 1;

        if H::IS_DEBUGGING_HARNESS && H::TRACK_STACK {
            bus.harness.on_stack_push(self, value);
        }
    }

    // Push a word onto the stack, wrapping the stack pointer in emulation mode if necessary.
    fn push_word<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, value: u16) {
        self.push(bus, (value >> 8) as u8);
        self.push(bus, value as u8);
    }

    /// Push a word onto the stack without wrapping the stack pointer in emulation mode.
    fn push_word_no_wrap<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>, value: u16) {
        self.push_no_wrap(bus, (value >> 8) as u8);
        self.push_no_wrap(bus, value as u8);
    }

    pub fn is_flag_set(&self, flag: Flag) -> bool {
        self.p & (flag as u8) != 0
    }

    pub fn set_flag_to_bool(&mut self, flag: Flag, value: bool) {
        if value {
            self.p |= flag as u8;
        } else {
            self.p &= !(flag as u8);
        }
    }
}
