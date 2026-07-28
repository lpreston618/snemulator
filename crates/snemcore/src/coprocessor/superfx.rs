use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

use crate::{get_bit_n, sysinfo::MASTER_CLOCK_HZ};

const GSU_CLOCK_HZ_SLOW: usize = 10_738_636; // CLSR.bit0 = 0
const GSU_CLOCK_HZ_FAST: usize = 21_477_272; // CLSR.bit0 = 1 (pipelined)

/// SFR (Status Flag Register) bit positions.
mod sfr_bits {
    const _: u16 = 1 << 0; // No bit 0
    pub const Z: u16 = 1 << 1; // Zero flag
    pub const CY: u16 = 1 << 2; // Carry flag
    pub const S: u16 = 1 << 3; // Sign flag
    pub const OV: u16 = 1 << 4; // Overflow flag
    pub const G: u16 = 1 << 5; // 1 = GSU running. CPU can clear this to stop the chip.
    pub const R: u16 = 1 << 6; // Set while a ROM buffer read (via R14/ROMB) is in flight.
    const _: u16 = 1 << 7; // No bit 7
    pub const ALT1: u16 = 1 << 8; // Prefix flag set by the ALT1 opcode ($3D).
    pub const ALT2: u16 = 1 << 9; // Prefix flag set by the ALT2 opcode ($3E). ALT1+ALT2 = ALT3.
    pub const IL: u16 = 1 << 10; // Immediate lower 8-bit flag.
    pub const IH: u16 = 1 << 11; // Immediate higher 8-bit flag.
    pub const B: u16 = 1 << 12; // Set to 1 when the WITH instruction is executed 
    const _: u16 = 1 << 13; // No bit 13
    const _: u16 = 1 << 14; // No bit 14
    pub const IRQ: u16 = 1 << 15; // Set to 1 when GSU caused an interrupt. Set to 0 when read by 65c816.
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SuperFx {
    /// R0-R13 are general purpose. R14 doubles as the ROM buffer address latch
    /// (any write to it kicks off a ROM fetch into the buffer). R15 is the PC;
    /// writing its high byte from the CPU side ($301F) is what starts the GSU.
    pub r: [u16; 16],

    /// Status Flag Register
    pub sfr: u16,
    /// Backup RAM Register
    pub bramr: u8,
    /// Program Bank Register
    pub pbr: u8,
    /// ROM Bank Register
    pub rombr: u8,
    /// Control Flags Register
    pub cfgr: u8,
    /// Screen Base Register
    pub scbr: u8,
    /// Clock Speed Register
    pub clsr: u8,
    /// Screen Mode Register
    pub scmr: u8,
    /// Version Code Register
    pub vcr: u8,
    /// RAM Bank Register
    pub rambr: u8,
    /// Cache Base Register
    pub cbr: u8,

    // 32 lines of 16 bytes. `cbr` (Cache Base Register) is the 16-byte-aligned GSU
    // program address the cache currently represents; `valid[i]` tracks
    // whether line i has been filled since the last CACHE invalidation.
    //
    // Simplification: real hardware fills cache lines transparently as code
    // executes within the CBR window; here we invalidate the whole cache on
    // CACHE and lazily fill lines on miss. Functionally equivalent for the
    // common "CACHE; loop body; LOOP" pattern, but not cycle-accurate for
    // partial-cache scenarios - revisit if a game depends on that.
    #[serde(with = "BigArray")]
    cache: [u8; 512],
    cache_valid: [bool; 32],

    // Real hardware: writing R14 starts an asynchronous ROM fetch that takes
    // a few cycles; GETB blocks if called before it's ready. We model it as
    // an immediate load for now and can add the latency later if a game
    // turns out to depend on the timing.
    rom_buffer: u8,

    running: bool,

    clock_accumulator: usize,
    clocks: usize,
}

impl SuperFx {
    pub fn new() -> Self {
        Self {
            r: [0u16; 16],
            sfr: 0u16,
            bramr: 0u8,
            pbr: 0u8,
            rombr: 0u8,
            cfgr: 0u8,
            scbr: 0u8,
            clsr: 0u8,
            scmr: 0u8,
            vcr: 0u8,
            rambr: 0u8,
            cbr: 0u8,
            cache: [0u8; 512],
            cache_valid: [false; 32],
            rom_buffer: 0u8,
            running: false,
            clock_accumulator: 0usize,
            clocks: 0usize,
        }
    }

    pub fn power_on(&mut self) {

    }

    pub fn reset(&mut self) {

    }

    pub fn cycle(&mut self, clocks: usize, rom: &[u8], ram: &mut [u8]) -> usize {
        if !self.running {
            return 0;
        }

        self.clock_accumulator += clocks * MASTER_CLOCK_HZ;

        let gsu_hz = if get_bit_n!(self.clsr, 0) { GSU_CLOCK_HZ_FAST } else { GSU_CLOCK_HZ_SLOW };

        while self.clock_accumulator > gsu_hz {
            self.clock_accumulator -= MASTER_CLOCK_HZ;
            
            if self.clocks == 0 {
                self.step(rom, ram);
            }
            
            self.clocks -= 1;
        }

        0 // GSU doesn't steal S-CPU cycles
    }

    fn step(&mut self, rom: &[u8], ram: &mut [u8]) {
        let opcode = self.fetch(rom, ram);


    }

    fn fetch(&mut self, rom: &[u8], ram: &mut [u8]) {
        
    }
}