use crate::{debug::DebugHarness, savestate, ssmp::spc::{bus::SpcBus, ioregs::SpcIoRegs}};

pub mod bus;
pub mod ioregs;
mod instructions;
mod disassembler;

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

#[derive(Default)]
pub struct Spc700 {
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub dir_page: u16,
    pub stopped: bool,

    branch_taken: bool,

    clocks: usize,

    prg_bytes: Vec<u8>,
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

    pub fn save_state(&self, regs: &SpcIoRegs) -> savestate::SpcState {
        savestate::SpcState {
            pc: self.pc,
            sp: self.sp,
            a: self.a,
            x: self.x,
            y: self.y,
            status: self.status,
            dir_page: self.dir_page,
            stopped: self.stopped,
            ipl_read_en: regs.ipl_read_en,
            sdsp_read_only: regs.sdsp_read_only,
            sdsp_addr: regs.sdsp_addr,
        }
    }

    pub fn load_state(&mut self, regs: &mut SpcIoRegs, state: &savestate::SpcState, _version: u32) {
        self.pc = state.pc;
        self.sp = state.sp;
        self.a = state.a;
        self.x = state.x;
        self.y = state.y;
        self.status = state.status;
        self.dir_page = state.dir_page;
        self.stopped = state.stopped;
        regs.ipl_read_en = state.ipl_read_en;
        regs.sdsp_read_only = state.sdsp_read_only;
        regs.sdsp_addr = state.sdsp_addr;
    }

    pub fn power_on(&mut self) {
        self.pc = 0xFFC0;
        self.sp = 0;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.status = 0;
        self.dir_page = 0;
        self.stopped = false;
        self.branch_taken = false;
        self.clocks = 0;
    }
    
    pub fn reset(&mut self) {
        self.pc = 0xFFC0;
        self.sp = 0;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.status = 0;
        self.dir_page = 0;
        self.stopped = false;
        self.branch_taken = false;
        self.clocks = 0;
    }

    pub fn clock<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        if self.clocks == 0 {            
            self.exec_instr(bus);

            if H::IS_DEBUGGING_HARNESS && H::TRACK_SPC_INSTRUCTIONS {
                bus.harness.on_spc_instruction(self, &self.prg_bytes.clone());
            }
        }

        self.clocks -= 1;
    }
}