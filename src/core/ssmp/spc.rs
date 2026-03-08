// use std::cell::Cell;
// use std::rc::Rc;

// use crate::core::ssmp::spc::bus::SpcBus;

use crate::core::ssmp::spc::bus::SpcBus;

pub mod bus;
pub mod ioregs;
mod instructions;

#[derive(PartialEq)]
pub enum Flag {
    FlagC = 1,   // Carry
    FlagZ = 2,   // Zero
    FlagI = 4,   // IRQ Disable
    FlagH = 8,   // Half-carry
    FlagB = 16,  // Break
    FlagP = 32,  // Direct Page
    FlagV = 64,  // Overflow
    FlagN = 128, // Negative
}

pub struct Spc700 {
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    status: u8,
    dir_page: u16,
    stopped: bool,

    branch_taken: bool,

    clocks: usize,
}

impl Spc700 {
    // Boot program for the SPC700
    const IPL_ROM: [u8; 64] = [
        0xCD, 0xEF, 0xBD, 0xE8, 0x00, 0xC6, 0x1D, 0xD0, 0xFC, 0x8F, 0xAA, 0xF4, 0x8F, 0xBB, 0xF5,
        0x78, 0xCC, 0xF4, 0xD0, 0xFB, 0x2F, 0x19, 0xEB, 0xF4, 0xD0, 0xFC, 0x7E, 0xF4, 0xD0, 0x0B,
        0xE4, 0xF5, 0xCB, 0xF4, 0xD7, 0x00, 0xFC, 0xD0, 0xF3, 0xAB, 0x01, 0x10, 0xEF, 0x7E, 0xF4,
        0x10, 0xEB, 0xBA, 0xF6, 0xDA, 0x00, 0xBA, 0xF4, 0xC4, 0xF4, 0xDD, 0x5D, 0xD0, 0xDB, 0x1F,
        0x00, 0x00, 0xC0, 0xFF,
    ];

    pub fn new() -> Spc700 {
        Spc700 {
            pc: 0xFFC0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            status: 0,
            stopped: false,
            branch_taken: false,
            dir_page: 0,

            clocks: 0,
        }
    }

//     fn reset(&mut self) {
//         todo!()
//     }

    pub fn clock(&mut self, bus: &mut SpcBus) {
        if self.clocks == 0 {
            self.exec_instr(bus);
        }

        self.clocks -= 1;
    }
}