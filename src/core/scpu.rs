use crate::core::scpu::bus::{Address, CpuBus};

mod bus;
mod dma;
mod instructions;
mod ioregs;
mod mult;

pub enum Flag {
    FlagC = 1 << 0,   // Carry
    FlagZ = 1 << 1,   // Zero
    FlagI = 1 << 2,   // IRQ Disable
    FlagD = 1 << 3,   // Decimal Mode
    FlagX = 1 << 4,   // X Register Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagM = 1 << 5,   // Accumulator Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagV = 1 << 6,   // Overflow
    FlagN = 1 << 7,   // Negative
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
    pub a: u16,       // Accumulator
    pub x: u16,       // X index
    pub y: u16,       // Y index
    pub sp: u16,      // Stack pointer
    pub pc: u16,      // Program counter
    pub pb: u8,       // Program bank
    pub db: u8,       // Data bank
    pub dp: u16,      // Direct page
    pub p: u8,        // Processor status
    pub e: bool,      // Emulation mode
    
    // Internal state
    pub halted: bool,
    pub waiting_for_irq: bool,
    pub irq_pending: bool,
    pub nmi_pending: bool,
    
    // Cycle tracking
    clocks: usize,
        
    fast_rom_en: bool,
    branch_taken: bool,
    page_crossed: bool,
    stopped: bool,
}

// SNES System Functionality
impl Cpu65c816 {
    const ONE_CYCLE_SLOW: usize = 6;
    const ONE_CYCLE_FAST: usize = 4;
    
    // Creates a new, uninitialized 65c816 CPU
    pub fn new() -> Cpu65c816 {
        Cpu65c816 {
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            pc: 0,
            pb: 0,
            db: 0,
            dp: 0,
            p: 0,
            e: false,
            
            // Internal state
            halted: false,
            waiting_for_irq: false,
            irq_pending: false,
            nmi_pending: false,
            
            // Cycle tracking
            clocks: 0,
                
            fast_rom_en: false,
            branch_taken: false,
            page_crossed: false,
            stopped: false,
        }
    }

    /// Sets the CPU to its proper initial state. Can be triggered by an interrupt.
    pub fn initialize(&mut self) {
        self.x = 0;
        self.y = 0;
        self.db = 0;
        self.pb = 0;
        self.dp = 0;
        self.s = 0x0100;
        self.p = 0x34;
        self.reset();
    }

    pub fn reset(&mut self) {
        // self.trigger_interrupt(CpuInterrupt::Reset);
    }
    
    fn read(&mut self, bus: &mut CpuBus, addr: Address) -> u8 {
        self.clocks += Self::ONE_CYCLE_SLOW;
        bus.read(addr)
    }
    
    fn write(&mut self, bus: &mut CpuBus, addr: Address, value: u8) {
        self.clocks += Self::ONE_CYCLE_SLOW;
        bus.write(addr, value);
    }
    
    fn read_prg(&mut self, bus: &mut CpuBus) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.clocks += Self::ONE_CYCLE_SLOW;
        bus.read(Address { bank: self.pb, offset: pc })
    }
    
    fn read_word(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) -> u16 {
        u16::from_le_bytes([
            self.read(bus, addr_lo),
            self.read(bus, addr_hi),
        ])
    }
    
    fn write_word(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address, value: u16) {
        self.write(bus, addr_lo, value as u8);
        self.write(bus, addr_hi, (value >> 8) as u8);
    }
    
    fn pop(&mut self, bus: &mut CpuBus) -> u8 {
        self.sp += 1;
        
        if self.e {
            self.sp = 0x100 | (self.sp & 0xFF);
        }
        
        self.read(bus, Address { bank: 0, offset: self.sp })
    }
    
    fn push(&mut self, bus: &mut CpuBus, value: u8) {
        self.write(bus, Address { bank: 0, offset: self.sp }, value);
        
        self.sp -= 1;
        
        if self.e {
            self.sp = 0x100 | (self.sp & 0xFF);
        }
    }
    
    fn pop_word(&mut self, bus: &mut CpuBus) -> u16 {
        u16::from_le_bytes([
            self.pop(bus),
            self.pop(bus),
        ])
    }
    
    fn push_word(&mut self, bus: &mut CpuBus, value: u16) {
        self.push(bus, (value >> 8) as u8);
        self.push(bus, value as u8);
    }
    
    fn is_flag_set(&self, flag: Flag) -> bool {
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