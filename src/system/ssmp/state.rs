use std::cell::Cell;

use crate::system::ssmp::{sdsp, ARAM_SIZE};

/// Contains all data (memory, registers, etc.) shared by the SPC700 and S-DSP.
pub struct SmpData {
    pub aram: Vec<Cell<u8>>,
    pub sdsp_regs: sdsp::SdspRegisters,
}

impl SmpData {
    pub fn new() -> SmpData {
        SmpData {
            aram: vec![Cell::new(0); ARAM_SIZE],
            sdsp_regs: sdsp::SdspRegisters::new(),
        }
    }
}