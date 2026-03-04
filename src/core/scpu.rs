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
    
    /// The number of clocks before the next instruction is executed
    clocks: usize,
        
    fast_rom_en: bool,
    branch_taken: bool,
    page_crossed: bool,
    stopped: bool,
}

// SNES System Functionality
impl Cpu65c816 {
    /// Number of system clocks in a single slow cpu cycle (e.g. a typical bus read/write)
    const SLOW_CYCLE_CLOCKS: usize = 8;
    /// Number of system clocks in a single cpu cycle
    const CYCLE_CLOCKS: usize = 6;
    
    // Creates a new, uninitialized 65c816 CPU
    pub fn new() -> Cpu65c816 {
        Cpu65c816 {
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
        self.sp = 0x0100;
        self.p = 0x34;
        self.reset();
    }

    pub fn reset(&mut self) {
        // self.trigger_interrupt(CpuInterrupt::Reset);
    }
    
    /// Cycles the cpu for a given number of clocks. If the number of clocks is 0 after cycling, the next instructions is executed.
    pub fn cycle(&mut self, bus: &mut CpuBus, clocks: usize) {
        self.clocks -= clocks;
        
        assert!(self.clocks < 250); // make sure no underflow
        
        if self.clocks == 0 {
            self.execute(bus);
        }
    }
}