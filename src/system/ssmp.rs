use std::{cell::Cell, rc::Rc};

use crate::utils::GetBits;

// Master Clock runs at 21.4773 MHz, and S-DSP internally clocks at 3.072 MHz
// The ratio of S-DSP Clock Period / Master Clock Period = (1/21477300 Hz) / (1/3072000 Hz)
// = 0.143034739 is approximated closely by 97/678 = 0.143067846608
// with an error of 0.02274%
const MASTER_CLOCK_TIME_UNITS: usize = 97;
const SDSP_CLOCK_TIME_UNITS: usize = 678;

// TIMER2 runs at 64kHz, which translates to one tick per every 96 DSP clocks.
// Timers 0 and 1 each run at 1/8th that speed (8kHz), so we keep a secondary
// counter that rolls over every 8 ticks of the fast TIMER2.
const FAST_TIMER_DSP_CLOCKS: usize = 96;
const SLOW_TIMER_TICKS_MAX: u8 = 8;

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

pub struct ApuIORegs {
    // APU -> CPU regs
    pub apuio0: Cell<u8>,
    pub apuio1: Cell<u8>,
    pub apuio2: Cell<u8>,
    pub apuio3: Cell<u8>,

    // CPU -> APU regs
    pub cpuio0: Cell<u8>,
    pub cpuio1: Cell<u8>,
    pub cpuio2: Cell<u8>,
    pub cpuio3: Cell<u8>,
}

impl ApuIORegs {
    pub fn new() -> ApuIORegs {
        ApuIORegs {
            apuio0: Cell::new(0),
            apuio1: Cell::new(0),
            apuio2: Cell::new(0),
            apuio3: Cell::new(0),
            cpuio0: Cell::new(0),
            cpuio1: Cell::new(0),
            cpuio2: Cell::new(0),
            cpuio3: Cell::new(0),
        }
    }
}

pub struct Spc700 {
    pc: u16,
    sp: u8,
    acc: u8,
    x: u8,
    y: u8,
    status: u8,

    branch_taken: bool,

    dir_page: u16,

    aram: Vec<u8>,

    time_since_last_clock: usize,
    sdsp_clocks: usize,
    spc_clocks_until_instr: usize,

    apuio_regs: Rc<ApuIORegs>,

    // $F1    I.CC .210
    //        | ||  |||
    //        | ||  ||+- Enable timer 0
    //        | ||  |+-- Enable timer 1
    //        | ||  +--- Enable timer 2
    //        | |+------ Clear CPUIO read ports 0 & 1
    //        | +------- Clear CPUIO read ports 2 & 3
    //        +--------- IPL ROM enable
    timer0_en: bool,
    timer1_en: bool,
    timer2_en: bool,
    ipl_read: bool,

    // $F2    RAAA AAAA
    //        |||| ||||
    //        |+++-++++- S-DSP register address
    //        +--------- Read only flag
    sdsp_read_only: bool,
    sdsp_addr: u8,

    // $FA..=$FC    TTTT TTTT
    //              |||| ||||
    //              ++++-++++- Timer target
    timer0_target: u8,
    timer1_target: u8,
    timer2_target: u8,

    // $FD..=$FF    0000 CCCC
    //                   ||||
    //                   ++++- Timer counter
    timer0_counter: u8,
    timer1_counter: u8,
    timer2_counter: u8,

    timer0_internal_counter: u8,
    timer1_internal_counter: u8,
    timer2_internal_counter: u8,

    slow_timer_clocks: u8,
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

    pub fn new(apuio_regs: Rc<ApuIORegs>) -> Spc700 {
        Spc700 {
            pc: 0xFFC0,
            sp: 0,
            acc: 0,
            x: 0,
            y: 0,
            status: 0,
            branch_taken: false,
            dir_page: 0,

            aram: vec![0; 0x10000],

            time_since_last_clock: 0,
            sdsp_clocks: 0,
            spc_clocks_until_instr: 0,

            apuio_regs,

            timer0_en: false,
            timer1_en: false,
            timer2_en: false,
            ipl_read: true,
            sdsp_read_only: false,
            sdsp_addr: 0,
            timer0_target: 0,
            timer1_target: 0,
            timer2_target: 0,
            timer0_counter: 0,
            timer1_counter: 0,
            timer2_counter: 0,
            timer0_internal_counter: 0,
            timer1_internal_counter: 0,
            timer2_internal_counter: 0,
            slow_timer_clocks: 0,
        }
    }

    fn reset(&mut self) {
        self.timer0_en = false;
        self.timer1_en = false;
        self.timer2_en = false;
    }

    pub fn clock(&mut self, master_clocks_elapsed: usize) {
        self.time_since_last_clock += master_clocks_elapsed * MASTER_CLOCK_TIME_UNITS;

        while self.time_since_last_clock > SDSP_CLOCK_TIME_UNITS {
            // self.sdsp.clock();

            self.sdsp_clocks += 1;

            // Spc700 clocks every 3 S-DSP cycles
            if self.sdsp_clocks % 3 == 0 {
                if self.spc_clocks_until_instr == 0 {
                    self.exec_instr();
                }

                self.spc_clocks_until_instr -= 1;
            }

            self.time_since_last_clock -= SDSP_CLOCK_TIME_UNITS;

            if self.sdsp_clocks == FAST_TIMER_DSP_CLOCKS {
                self.clock_fast_timer();

                self.sdsp_clocks = 0;
                self.slow_timer_clocks += 1;

                if self.slow_timer_clocks == SLOW_TIMER_TICKS_MAX {
                    self.clock_slow_timers();

                    self.slow_timer_clocks = 0;
                }
            }
        }
    }

    fn clock_slow_timers(&mut self) {
        if self.timer0_en {
            self.timer0_internal_counter += 1;
            if self.timer0_internal_counter == self.timer0_target {
                self.timer0_counter += 1;
                self.timer0_counter &= 0x0F;
                self.timer0_internal_counter = 0;
            }
        }
        if self.timer1_en {
            self.timer1_internal_counter += 1;
            if self.timer1_internal_counter == self.timer1_target {
                self.timer1_counter += 1;
                self.timer1_counter &= 0x0F;
                self.timer1_internal_counter = 0;
            }
        }
    }

    fn clock_fast_timer(&mut self) {
        if self.timer2_en {
            self.timer2_internal_counter += 1;
            if self.timer2_internal_counter == self.timer2_target {
                self.timer2_counter += 1;
                self.timer2_counter &= 0x0F;
                self.timer2_internal_counter = 0;
            }
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            (0x00F0..=0x00FF) => self.read_sound_regs(address),
            (0xFFC0..=0xFFFF) if self.ipl_read => Spc700::IPL_ROM[(address & 0x3F) as usize],
            _ => self.aram[address as usize],
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            (0xF0..=0xFF) => self.write_sound_regs(address, data),
            _ => self.aram[address as usize] = data,
        }
    }

    fn read_sound_regs(&mut self, address: u16) -> u8 {
        match address & 0xF {
            0x2 => self.sdsp_addr,
            0x3 => 0, // self.sdsp.read_regs(self.sdsp_addr),
            0x4 => {
                println!(
                    "[Spc700] Read 0x{:02X} from cpuio0",
                    self.apuio_regs.cpuio0.get()
                );
                self.apuio_regs.cpuio0.get()
            }
            0x5 => {
                println!(
                    "[Spc700] Read 0x{:02X} from cpuio1",
                    self.apuio_regs.cpuio1.get()
                );
                self.apuio_regs.cpuio1.get()
            }
            0x6 => {
                println!(
                    "[Spc700] Read 0x{:02X} from cpuio2",
                    self.apuio_regs.cpuio2.get()
                );
                self.apuio_regs.cpuio2.get()
            }
            0x7 => {
                println!(
                    "[Spc700] Read 0x{:02X} from cpuio3",
                    self.apuio_regs.cpuio3.get()
                );
                self.apuio_regs.cpuio3.get()
            }
            0x8 => self.aram[0xFF08],
            0x9 => self.aram[0xFF09],
            0xA => self.timer0_target,
            0xB => self.timer1_target,
            0xC => self.timer2_target,
            0xD => {
                let data = self.timer0_counter;
                self.timer0_counter = 0;
                data
            }
            0xE => {
                let data = self.timer1_counter;
                self.timer1_counter = 0;
                data
            }
            0xF => {
                let data = self.timer2_counter;
                self.timer2_counter = 0;
                data
            }
            _ => 0,
        }
    }

    fn write_sound_regs(&mut self, address: u16, data: u8) {
        match address & 0xF {
            0x1 => {
                if !self.timer0_en && data.bit_en(0) {
                    self.timer0_counter = 0;
                    self.timer0_internal_counter = 0;
                }

                if !self.timer1_en && data.bit_en(1) {
                    self.timer1_counter = 0;
                    self.timer1_internal_counter = 0;
                }

                if !self.timer2_en && data.bit_en(2) {
                    self.timer2_counter = 0;
                    self.timer2_internal_counter = 0;
                }

                self.timer0_en = data.bit_en(0);
                self.timer1_en = data.bit_en(1);
                self.timer2_en = data.bit_en(2);
                self.ipl_read = data.bit_en(7);

                if data.bit_en(4) {
                    self.apuio_regs.cpuio0.set(0);
                    self.apuio_regs.cpuio1.set(0);
                }

                if data.bit_en(5) {
                    self.apuio_regs.cpuio2.set(0);
                    self.apuio_regs.cpuio3.set(0);
                }
            }
            0x2 => {
                self.sdsp_read_only = data.bit_en(7);
                self.sdsp_addr = data & 0x7F;
            }
            0x3 => {
                if !self.sdsp_read_only {
                    // self.sdsp.write_regs(self.sdsp_addr, data);
                }
            }
            0x4 => {
                println!("[Spc700] wrote 0x{:02X} to apuio0", data);
                self.apuio_regs.apuio0.set(data);
            }
            0x5 => {
                println!("[Spc700] wrote 0x{:02X} to apuio1", data);
                self.apuio_regs.apuio1.set(data);
            }
            0x6 => {
                println!("[Spc700] wrote 0x{:02X} to apuio2", data);
                self.apuio_regs.apuio2.set(data);
            }
            0x7 => {
                println!("[Spc700] wrote 0x{:02X} to apuio3", data);
                self.apuio_regs.apuio3.set(data);
            }
            0x8 => {
                self.aram[0xFF08] = data;
            }
            0x9 => {
                self.aram[0xFF09] = data;
            }
            0xA => {
                self.timer0_target = data;
            }
            0xB => {
                self.timer1_target = data;
            }
            0xC => {
                self.timer2_target = data;
            }
            0xD => {
                self.timer0_counter = data;
            }
            0xE => {
                self.timer1_counter = data;
            }
            0xF => {
                self.timer2_counter = data;
            }
            _ => {}
        }
    }

    fn read_word(&mut self, addr_lo: u16, addr_hi: u16) -> u16 {
        u16::from_le_bytes([self.read(addr_lo), self.read(addr_hi)])
    }

    fn write_word(&mut self, addr_lo: u16, addr_hi: u16, word: u16) {
        self.write(addr_lo, word as u8);
        self.write(addr_hi, (word >> 8) as u8);
    }

    fn pop(&mut self) -> u8 {
        self.sp += 1;
        self.read(0x100 | self.sp as u16)
    }

    fn push(&mut self, data: u8) {
        self.write(0x100 | self.sp as u16, data);
    }

    fn pop_word(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }

    fn push_word(&mut self, word: u16) {
        self.push(word as u8);
        self.push((word >> 8) as u8);
    }

    fn exec_instr(&mut self) {
        let cycles: usize;

        let opcode = self.read(self.pc);

        self.branch_taken = false;

        match opcode {
            0x00 => {
                self.pc += 1;
                self.nop();
                cycles = 2;
            }
            0x01 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x02 => {
                let addr = self.direct();
                self.pc += 2;
                self.set1(addr, opcode);
                cycles = 4;
            }
            0x03 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbs(addr1, addr2, opcode);
                cycles = 5;
            }
            0x04 => {
                let addr = self.direct();
                self.pc += 2;
                self.or_acc(addr);
                cycles = 3;
            }
            0x05 => {
                let addr = self.absolute();
                self.pc += 3;
                self.or_acc(addr);
                cycles = 4;
            }
            0x06 => {
                let addr = self.indirect();
                self.pc += 1;
                self.or_acc(addr);
                cycles = 3;
            }
            0x07 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.or_acc(addr);
                cycles = 6;
            }
            0x08 => {
                let addr = self.immediate();
                self.pc += 2;
                self.or_acc(addr);
                cycles = 2;
            }
            0x09 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                self.or_mem(addr1, addr2);
                cycles = 6;
            }
            0x0A => {
                let addr = self.absolute();
                self.pc += 3;
                self.or1(addr);
                cycles = 5;
            }
            0x0B => {
                let addr = self.direct();
                self.pc += 2;
                self.asl_mem(addr);
                cycles = 4;
            }
            0x0C => {
                let addr = self.absolute();
                self.pc += 3;
                self.asl_mem(addr);
                cycles = 5;
            }
            0x0D => {
                self.pc += 1;
                self.push_psw();
                cycles = 4;
            }
            0x0E => {
                let addr = self.absolute();
                self.pc += 3;
                self.tset1(addr);
                cycles = 6;
            }
            0x0F => {
                self.pc += 1;
                self.brk();
                cycles = 8;
            }
            0x10 => {
                let addr = self.relative();
                self.pc += 2;
                self.bpl(addr);
                cycles = 2;
            }
            0x11 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x12 => {
                let addr = self.direct();
                self.pc += 2;
                self.clr1(addr, opcode);
                cycles = 4;
            }
            0x13 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbc(addr1, addr2, opcode);
                cycles = 5;
            }
            0x14 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.or_acc(addr);
                cycles = 4;
            }
            0x15 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.or_acc(addr);
                cycles = 5;
            }
            0x16 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.or_acc(addr);
                cycles = 5;
            }
            0x17 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.or_acc(addr);
                cycles = 6;
            }
            0x18 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                self.or_mem(addr1, addr2);
                cycles = 5;
            }
            0x19 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                self.or_mem(addr1, addr2);
                cycles = 5;
            }
            0x1A => {
                let (addr1, addr2) = self.direct_word();
                self.pc += 2;
                self.decw(addr1, addr2);
                cycles = 6;
            }
            0x1B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.asl_mem(addr);
                cycles = 5;
            }
            0x1C => {
                self.pc += 1;
                self.asl_acc();
                cycles = 2;
            }
            0x1D => {
                self.pc += 1;
                self.dex();
                cycles = 2;
            }
            0x1E => {
                let addr = self.absolute();
                self.pc += 3;
                self.cmx(addr);
                cycles = 4;
            }
            0x1F => {
                let addr = self.absolute_x_indirect();
                self.pc += 3;
                self.jmp(addr);
                cycles = 6;
            }
            0x20 => {
                self.pc += 1;
                self.clrp();
                cycles = 2;
            }
            0x21 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x22 => {
                let addr = self.direct();
                self.pc += 2;
                self.set1(addr, opcode);
                cycles = 4;
            }
            0x23 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbs(addr1, addr2, opcode);
                cycles = 5;
            }
            0x24 => {
                let addr = self.direct();
                self.pc += 2;
                self.and_acc(addr);
                cycles = 3;
            }
            0x25 => {
                let addr = self.absolute();
                self.pc += 3;
                self.and_acc(addr);
                cycles = 4;
            }
            0x26 => {
                let addr = self.indirect();
                self.pc += 1;
                self.and_acc(addr);
                cycles = 3;
            }
            0x27 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.and_acc(addr);
                cycles = 6;
            }
            0x28 => {
                let addr = self.immediate();
                self.pc += 2;
                self.and_acc(addr);
                cycles = 2;
            }
            0x29 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                self.and_mem(addr1, addr2);
                cycles = 6;
            }
            0x2A => {
                let addr = self.absolute();
                self.pc += 3;
                self.or1(addr);
                cycles = 5;
            }
            0x2B => {
                let addr = self.direct();
                self.pc += 2;
                self.rol_mem(addr);
                cycles = 4;
            }
            0x2C => {
                let addr = self.absolute();
                self.pc += 3;
                self.rol_mem(addr);
                cycles = 5;
            }
            0x2D => {
                self.pc += 1;
                self.push_acc();
                cycles = 4;
            }
            0x2E => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.cbne(addr1, addr2);
                cycles = 5;
            }
            0x2F => {
                let addr = self.relative();
                self.pc += 2;
                self.bra(addr);
                cycles = 4;
            }
            0x30 => {
                let addr = self.relative();
                self.pc += 2;
                self.bmi(addr);
                cycles = 2;
            }
            0x31 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x32 => {
                let addr = self.direct();
                self.pc += 2;
                self.clr1(addr, opcode);
                cycles = 4;
            }
            0x33 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbc(addr1, addr2, opcode);
                cycles = 5;
            }
            0x34 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.and_acc(addr);
                cycles = 4;
            }
            0x35 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.and_acc(addr);
                cycles = 5;
            }
            0x36 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.and_acc(addr);
                cycles = 5;
            }
            0x37 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.and_acc(addr);
                cycles = 6;
            }
            0x38 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                self.and_mem(addr1, addr2);
                cycles = 5;
            }
            0x39 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                self.and_mem(addr1, addr2);
                cycles = 5;
            }
            0x3A => {
                let (addr1, addr2) = self.direct_word();
                self.pc += 2;
                self.incw(addr1, addr2);
                cycles = 6;
            }
            0x3B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.rol_mem(addr);
                cycles = 5;
            }
            0x3C => {
                self.pc += 1;
                self.rol_acc();
                cycles = 2;
            }
            0x3D => {
                self.pc += 1;
                self.inx();
                cycles = 2;
            }
            0x3E => {
                let addr = self.direct();
                self.pc += 2;
                self.cmx(addr);
                cycles = 3;
            }
            0x3F => {
                let addr = self.absolute();
                self.pc += 3;
                self.call(addr);
                cycles = 8;
            }
            0x40 => {
                self.pc += 1;
                self.setp();
                cycles = 2;
            }
            0x41 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x42 => {
                let addr = self.direct();
                self.pc += 2;
                self.set1(addr, opcode);
                cycles = 4;
            }
            0x43 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbs(addr1, addr2, opcode);
                cycles = 5;
            }
            0x44 => {
                let addr = self.direct();
                self.pc += 2;
                self.eor_acc(addr);
                cycles = 3;
            }
            0x45 => {
                let addr = self.absolute();
                self.pc += 3;
                self.eor_acc(addr);
                cycles = 4;
            }
            0x46 => {
                let addr = self.indirect();
                self.pc += 1;
                self.eor_acc(addr);
                cycles = 3;
            }
            0x47 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.eor_acc(addr);
                cycles = 6;
            }
            0x48 => {
                let addr = self.immediate();
                self.pc += 2;
                self.eor_acc(addr);
                cycles = 2;
            }
            0x49 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                self.eor_mem(addr1, addr2);
                cycles = 6;
            }
            0x4A => {
                let addr = self.absolute();
                self.pc += 3;
                self.and1(addr);
                cycles = 4;
            }
            0x4B => {
                let addr = self.direct();
                self.pc += 2;
                self.lsr_mem(addr);
                cycles = 4;
            }
            0x4C => {
                let addr = self.absolute();
                self.pc += 3;
                self.lsr_mem(addr);
                cycles = 5;
            }
            0x4D => {
                self.pc += 1;
                self.push_x();
                cycles = 4;
            }
            0x4E => {
                let addr = self.absolute();
                self.pc += 3;
                self.tclr1(addr);
                cycles = 6;
            }
            0x4F => {
                let addr = self.immediate();
                self.pc += 1;
                self.pcall(addr);
                cycles = 6;
            }
            0x50 => {
                let addr = self.relative();
                self.pc += 2;
                self.bvc(addr);
                cycles = 2;
            }
            0x51 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x52 => {
                let addr = self.direct();
                self.pc += 2;
                self.clr1(addr, opcode);
                cycles = 4;
            }
            0x53 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbc(addr1, addr2, opcode);
                cycles = 5;
            }
            0x54 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.eor_acc(addr);
                cycles = 4;
            }
            0x55 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.eor_acc(addr);
                cycles = 5;
            }
            0x56 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.eor_acc(addr);
                cycles = 5;
            }
            0x57 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.eor_acc(addr);
                cycles = 6;
            }
            0x58 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                self.eor_mem(addr1, addr2);
                cycles = 5;
            }
            0x59 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                self.eor_mem(addr1, addr2);
                cycles = 5;
            }
            0x5A => {
                let (addr1, addr2) = self.direct_word();
                self.pc += 2;
                self.cmpw(addr1, addr2);
                cycles = 4;
            }
            0x5B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.lsr_mem(addr);
                cycles = 5;
            }
            0x5C => {
                self.pc += 1;
                self.lsr_acc();
                cycles = 2;
            }
            0x5D => {
                self.pc += 1;
                self.tax();
                cycles = 2;
            }
            0x5E => {
                let addr = self.absolute();
                self.pc += 3;
                self.cmy(addr);
                cycles = 4;
            }
            0x5F => {
                let addr = self.absolute();
                self.pc += 3;
                self.jmp(addr);
                cycles = 3;
            }
            0x60 => {
                self.pc += 1;
                self.clrc();
                cycles = 2;
            }
            0x61 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x62 => {
                let addr = self.direct();
                self.pc += 2;
                self.set1(addr, opcode);
                cycles = 4;
            }
            0x63 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbs(addr1, addr2, opcode);
                cycles = 5;
            }
            0x64 => {
                let addr = self.direct();
                self.pc += 2;
                self.cmp_acc(addr);
                cycles = 3;
            }
            0x65 => {
                let addr = self.absolute();
                self.pc += 3;
                self.cmp_acc(addr);
                cycles = 4;
            }
            0x66 => {
                let addr = self.indirect();
                self.pc += 1;
                self.cmp_acc(addr);
                cycles = 3;
            }
            0x67 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.cmp_acc(addr);
                cycles = 6;
            }
            0x68 => {
                let addr = self.immediate();
                self.pc += 2;
                self.cmp_acc(addr);
                cycles = 2;
            }
            0x69 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                self.cmp_mem(addr1, addr2);
                cycles = 6;
            }
            0x6A => {
                let addr = self.absolute();
                self.pc += 3;
                self.and1(addr);
                cycles = 4;
            }
            0x6B => {
                let addr = self.direct();
                self.pc += 2;
                self.ror_mem(addr);
                cycles = 4;
            }
            0x6C => {
                let addr = self.absolute();
                self.pc += 3;
                self.ror_mem(addr);
                cycles = 5;
            }
            0x6D => {
                self.pc += 1;
                self.push_y();
                cycles = 4;
            }
            0x6E => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.dbnz_mem(addr1, addr2);
                cycles = 5;
            }
            0x6F => {
                self.pc += 1;
                self.ret();
                cycles = 5;
            }
            0x70 => {
                let addr = self.relative();
                self.pc += 2;
                self.bvs(addr);
                cycles = 2;
            }
            0x71 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x72 => {
                let addr = self.direct();
                self.pc += 2;
                self.clr1(addr, opcode);
                cycles = 4;
            }
            0x73 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbc(addr1, addr2, opcode);
                cycles = 5;
            }
            0x74 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.cmp_acc(addr);
                cycles = 4;
            }
            0x75 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.cmp_acc(addr);
                cycles = 5;
            }
            0x76 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.cmp_acc(addr);
                cycles = 5;
            }
            0x77 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.cmp_acc(addr);
                cycles = 6;
            }
            0x78 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                self.cmp_mem(addr1, addr2);
                cycles = 5;
            }
            0x79 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                self.cmp_mem(addr1, addr2);
                cycles = 5;
            }
            0x7A => {
                let (addr1, addr2) = self.direct_word();
                self.pc += 2;
                self.addw(addr1, addr2);
                cycles = 5;
            }
            0x7B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.ror_mem(addr);
                cycles = 5;
            }
            0x7C => {
                self.pc += 1;
                self.ror_acc();
                cycles = 2;
            }
            0x7D => {
                self.pc += 1;
                self.txa();
                cycles = 2;
            }
            0x7E => {
                let addr = self.direct();
                self.pc += 2;
                self.cmy(addr);
                cycles = 3;
            }
            0x7F => {
                self.pc += 1;
                self.ret1();
                cycles = 6;
            }
            0x80 => {
                self.pc += 1;
                self.setc();
                cycles = 2;
            }
            0x81 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x82 => {
                let addr = self.direct();
                self.pc += 2;
                self.set1(addr, opcode);
                cycles = 4;
            }
            0x83 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbs(addr1, addr2, opcode);
                cycles = 5;
            }
            0x84 => {
                let addr = self.direct();
                self.pc += 2;
                self.adc_acc(addr);
                cycles = 3;
            }
            0x85 => {
                let addr = self.absolute();
                self.pc += 3;
                self.adc_acc(addr);
                cycles = 4;
            }
            0x86 => {
                let addr = self.indirect();
                self.pc += 1;
                self.adc_acc(addr);
                cycles = 3;
            }
            0x87 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.adc_acc(addr);
                cycles = 6;
            }
            0x88 => {
                let addr = self.immediate();
                self.pc += 2;
                self.adc_acc(addr);
                cycles = 2;
            }
            0x89 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                self.adc_mem(addr1, addr2);
                cycles = 6;
            }
            0x8A => {
                let addr = self.absolute();
                self.pc += 3;
                self.eor1(addr);
                cycles = 5;
            }
            0x8B => {
                let addr = self.direct();
                self.pc += 2;
                self.dec_mem(addr);
                cycles = 4;
            }
            0x8C => {
                let addr = self.absolute();
                self.pc += 3;
                self.dec_mem(addr);
                cycles = 5;
            }
            0x8D => {
                let addr = self.immediate();
                self.pc += 2;
                self.ldy(addr);
                cycles = 2;
            }
            0x8E => {
                self.pc += 1;
                self.pop_psw();
                cycles = 4;
            }
            0x8F => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                self.mov(addr1, addr2);
                cycles = 5;
            }
            0x90 => {
                let addr = self.relative();
                self.pc += 2;
                self.bcc(addr);
                cycles = 2;
            }
            0x91 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0x92 => {
                let addr = self.direct();
                self.pc += 2;
                self.clr1(addr, opcode);
                cycles = 4;
            }
            0x93 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbc(addr1, addr2, opcode);
                cycles = 5;
            }
            0x94 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.adc_acc(addr);
                cycles = 4;
            }
            0x95 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.adc_acc(addr);
                cycles = 5;
            }
            0x96 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.adc_acc(addr);
                cycles = 5;
            }
            0x97 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.adc_acc(addr);
                cycles = 6;
            }
            0x98 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                self.adc_mem(addr1, addr2);
                cycles = 5;
            }
            0x99 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                self.adc_mem(addr1, addr2);
                cycles = 5;
            }
            0x9A => {
                let (addr1, addr2) = self.direct_word();
                self.pc += 2;
                self.subw(addr1, addr2);
                cycles = 5;
            }
            0x9B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.dec_mem(addr);
                cycles = 5;
            }
            0x9C => {
                self.pc += 1;
                self.dec_acc();
                cycles = 2;
            }
            0x9D => {
                self.pc += 1;
                self.tsx();
                cycles = 2;
            }
            0x9E => {
                self.pc += 1;
                self.div();
                cycles = 12;
            }
            0x9F => {
                self.pc += 1;
                self.xcn();
                cycles = 5;
            }
            0xA0 => {
                self.pc += 1;
                self.sei();
                cycles = 3;
            }
            0xA1 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0xA2 => {
                let addr = self.direct();
                self.pc += 2;
                self.set1(addr, opcode);
                cycles = 4;
            }
            0xA3 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbs(addr1, addr2, opcode);
                cycles = 5;
            }
            0xA4 => {
                let addr = self.direct();
                self.pc += 2;
                self.sbc_acc(addr);
                cycles = 3;
            }
            0xA5 => {
                let addr = self.absolute();
                self.pc += 3;
                self.sbc_acc(addr);
                cycles = 4;
            }
            0xA6 => {
                let addr = self.indirect();
                self.pc += 1;
                self.sbc_acc(addr);
                cycles = 3;
            }
            0xA7 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.sbc_acc(addr);
                cycles = 6;
            }
            0xA8 => {
                let addr = self.immediate();
                self.pc += 2;
                self.sbc_acc(addr);
                cycles = 2;
            }
            0xA9 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                self.sbc_mem(addr1, addr2);
                cycles = 6;
            }
            0xAA => {
                let addr = self.absolute();
                self.pc += 3;
                self.ldc(addr);
                cycles = 4;
            }
            0xAB => {
                let addr = self.direct();
                self.pc += 2;
                self.inc_mem(addr);
                cycles = 4;
            }
            0xAC => {
                let addr = self.absolute();
                self.pc += 3;
                self.inc_mem(addr);
                cycles = 5;
            }
            0xAD => {
                let addr = self.immediate();
                self.pc += 2;
                self.cmy(addr);
                cycles = 2;
            }
            0xAE => {
                self.pc += 1;
                self.pop_acc();
                cycles = 4;
            }
            0xAF => {
                let addr = self.indirect_inc();
                self.pc += 1;
                self.sta(addr);
                cycles = 4;
            }
            0xB0 => {
                let addr = self.relative();
                self.pc += 2;
                self.bcs(addr);
                cycles = 2;
            }
            0xB1 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0xB2 => {
                let addr = self.direct();
                self.pc += 2;
                self.clr1(addr, opcode);
                cycles = 4;
            }
            0xB3 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbc(addr1, addr2, opcode);
                cycles = 5;
            }
            0xB4 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.sbc_acc(addr);
                cycles = 4;
            }
            0xB5 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.sbc_acc(addr);
                cycles = 5;
            }
            0xB6 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.sbc_acc(addr);
                cycles = 5;
            }
            0xB7 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.sbc_acc(addr);
                cycles = 6;
            }
            0xB8 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                self.sbc_mem(addr1, addr2);
                cycles = 5;
            }
            0xB9 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                self.sbc_mem(addr1, addr2);
                cycles = 5;
            }
            0xBA => {
                let (addr1, addr2) = self.direct_word();
                self.pc += 2;
                self.ldya(addr1, addr2);
                cycles = 5;
            }
            0xBB => {
                let addr = self.x_direct();
                self.pc += 2;
                self.inc_mem(addr);
                cycles = 5;
            }
            0xBC => {
                self.pc += 1;
                self.inc_acc();
                cycles = 2;
            }
            0xBD => {
                self.pc += 1;
                self.txs();
                cycles = 2;
            }
            0xBE => {
                self.pc += 1;
                self.das();
                cycles = 3;
            }
            0xBF => {
                let addr = self.indirect_inc();
                self.pc += 1;
                self.lda(addr);
                cycles = 4;
            }
            0xC0 => {
                self.pc += 1;
                self.cli();
                cycles = 3;
            }
            0xC1 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0xC2 => {
                let addr = self.direct();
                self.pc += 2;
                self.set1(addr, opcode);
                cycles = 4;
            }
            0xC3 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbs(addr1, addr2, opcode);
                cycles = 5;
            }
            0xC4 => {
                let addr = self.direct();
                self.pc += 2;
                self.sta(addr);
                cycles = 4;
            }
            0xC5 => {
                let addr = self.absolute();
                self.pc += 3;
                self.sta(addr);
                cycles = 5;
            }
            0xC6 => {
                let addr = self.indirect();
                self.pc += 1;
                self.sta(addr);
                cycles = 4;
            }
            0xC7 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.sta(addr);
                cycles = 7;
            }
            0xC8 => {
                let addr = self.immediate();
                self.pc += 2;
                self.cmx(addr);
                cycles = 2;
            }
            0xC9 => {
                let addr = self.absolute();
                self.pc += 3;
                self.stx(addr);
                cycles = 5;
            }
            0xCA => {
                let addr = self.absolute();
                self.pc += 3;
                self.stc(addr);
                cycles = 6;
            }
            0xCB => {
                let addr = self.direct();
                self.pc += 2;
                self.sty(addr);
                cycles = 4;
            }
            0xCC => {
                let addr = self.absolute();
                self.pc += 3;
                self.sty(addr);
                cycles = 5;
            }
            0xCD => {
                let addr = self.immediate();
                self.pc += 2;
                self.ldx(addr);
                cycles = 2;
            }
            0xCE => {
                self.pc += 1;
                self.pop_x();
                cycles = 4;
            }
            0xCF => {
                self.pc += 1;
                self.mul();
                cycles = 9;
            }
            0xD0 => {
                let addr = self.relative();
                self.pc += 2;
                self.bne(addr);
                cycles = 2;
            }
            0xD1 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0xD2 => {
                let addr = self.direct();
                self.pc += 2;
                self.clr1(addr, opcode);
                cycles = 4;
            }
            0xD3 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbc(addr1, addr2, opcode);
                cycles = 5;
            }
            0xD4 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.sta(addr);
                cycles = 5;
            }
            0xD5 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.sta(addr);
                cycles = 6;
            }
            0xD6 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.sta(addr);
                cycles = 6;
            }
            0xD7 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.sta(addr);
                cycles = 7;
            }
            0xD8 => {
                let addr = self.direct();
                self.pc += 2;
                self.stx(addr);
                cycles = 4;
            }
            0xD9 => {
                let addr = self.y_direct();
                self.pc += 2;
                self.stx(addr);
                cycles = 5;
            }
            0xDA => {
                let (addr1, addr2) = self.direct_word();
                self.pc += 2;
                self.stya(addr1, addr2);
                cycles = 5;
            }
            0xDB => {
                let addr = self.x_direct();
                self.pc += 2;
                self.sty(addr);
                cycles = 5;
            }
            0xDC => {
                self.pc += 1;
                self.dey();
                cycles = 2;
            }
            0xDD => {
                self.pc += 1;
                self.tya();
                cycles = 2;
            }
            0xDE => {
                let (addr1, addr2) = self.x_direct_relative();
                self.pc += 3;
                self.cbne(addr1, addr2);
                cycles = 6;
            }
            0xDF => {
                self.pc += 1;
                self.daa();
                cycles = 3;
            }
            0xE0 => {
                self.pc += 1;
                self.clrv();
                cycles = 2;
            }
            0xE1 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0xE2 => {
                let addr = self.direct();
                self.pc += 2;
                self.set1(addr, opcode);
                cycles = 4;
            }
            0xE3 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbs(addr1, addr2, opcode);
                cycles = 5;
            }
            0xE4 => {
                let addr = self.direct();
                self.pc += 2;
                self.lda(addr);
                cycles = 3;
            }
            0xE5 => {
                let addr = self.absolute();
                self.pc += 3;
                self.lda(addr);
                cycles = 4;
            }
            0xE6 => {
                let addr = self.indirect();
                self.pc += 1;
                self.lda(addr);
                cycles = 3;
            }
            0xE7 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.lda(addr);
                cycles = 6;
            }
            0xE8 => {
                let addr = self.immediate();
                self.pc += 2;
                self.lda(addr);
                cycles = 2;
            }
            0xE9 => {
                let addr = self.absolute();
                self.pc += 3;
                self.ldx(addr);
                cycles = 4;
            }
            0xEA => {
                let addr = self.absolute();
                self.pc += 3;
                self.not1(addr);
                cycles = 5;
            }
            0xEB => {
                let addr = self.direct();
                self.pc += 2;
                self.ldy(addr);
                cycles = 3;
            }
            0xEC => {
                let addr = self.absolute();
                self.pc += 3;
                self.ldy(addr);
                cycles = 4;
            }
            0xED => {
                self.pc += 1;
                self.notc();
                cycles = 3;
            }
            0xEE => {
                self.pc += 1;
                self.pop_y();
                cycles = 4;
            }
            0xEF => {
                self.pc += 1;
                self.sleep();
                cycles = 3;
            }
            0xF0 => {
                let addr = self.relative();
                self.pc += 2;
                self.beq(addr);
                cycles = 2;
            }
            0xF1 => {
                self.pc += 1;
                self.tcall(opcode);
                cycles = 8;
            }
            0xF2 => {
                let addr = self.direct();
                self.pc += 2;
                self.clr1(addr, opcode);
                cycles = 4;
            }
            0xF3 => {
                let (addr1, addr2) = self.direct_relative();
                self.pc += 3;
                self.bbc(addr1, addr2, opcode);
                cycles = 5;
            }
            0xF4 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.lda(addr);
                cycles = 4;
            }
            0xF5 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.lda(addr);
                cycles = 5;
            }
            0xF6 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.lda(addr);
                cycles = 5;
            }
            0xF7 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.lda(addr);
                cycles = 6;
            }
            0xF8 => {
                let addr = self.direct();
                self.pc += 2;
                self.ldx(addr);
                cycles = 3;
            }
            0xF9 => {
                let addr = self.y_direct();
                self.pc += 2;
                self.ldx(addr);
                cycles = 4;
            }
            0xFA => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                self.mov(addr1, addr2);
                cycles = 5;
            }
            0xFB => {
                let addr = self.x_direct();
                self.pc += 2;
                self.ldy(addr);
                cycles = 4;
            }
            0xFC => {
                self.pc += 1;
                self.iny();
                cycles = 2;
            }
            0xFD => {
                self.pc += 1;
                self.tay();
                cycles = 2;
            }
            0xFE => {
                let addr = self.relative();
                self.pc += 2;
                self.dbnz_y(addr);
                cycles = 4;
            }
            0xFF => {
                self.pc += 1;
                self.stop();
                cycles = 3;
            }
        }

        self.spc_clocks_until_instr += cycles;

        if self.branch_taken {
            self.spc_clocks_until_instr += 2;
        }
    }
}

// Helper functions.
impl Spc700 {
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
}

// Addressing Modes
impl Spc700 {
    fn direct(&mut self) -> u16 {
        (self.read(self.pc + 1) as u16) | self.dir_page
    }

    fn direct_word(&mut self) -> (u16, u16) {
        let tmp = self.read(self.pc + 1);
        let lo_addr = tmp as u16 | self.dir_page;
        let hi_addr = (tmp + 1) as u16 | self.dir_page;

        (lo_addr, hi_addr)
    }

    fn x_direct(&mut self) -> u16 {
        ((self.read(self.pc + 1) + self.x) as u16) | self.dir_page
    }

    fn y_direct(&mut self) -> u16 {
        ((self.read(self.pc + 1) + self.y) as u16) | self.dir_page
    }

    fn indirect(&mut self) -> u16 {
        (self.x as u16) | self.dir_page
    }

    fn indirect_inc(&mut self) -> u16 {
        let addr = (self.x as u16) | self.dir_page;
        self.x += 1;

        addr
    }

    fn direct_to_direct(&mut self) -> (u16, u16) {
        let src_addr = (self.read(self.pc + 2) as u16) | self.dir_page;
        let dst_addr = (self.read(self.pc + 1) as u16) | self.dir_page;

        (src_addr, dst_addr)
    }

    fn indirect_to_indirect(&mut self) -> (u16, u16) {
        let src_addr = (self.y as u16) | self.dir_page;
        let dst_addr = (self.x as u16) | self.dir_page;

        (src_addr, dst_addr)
    }

    fn immediate_to_direct(&mut self) -> (u16, u16) {
        let src_addr = ((self.pc + 2) as u16) | self.dir_page;
        let dst_addr = (self.read(self.pc + 1) as u16) | self.dir_page;

        (src_addr, dst_addr)
    }

    fn direct_relative(&mut self) -> (u16, u16) {
        let data_addr = self.direct();
        let offset = self.read(self.pc + 2);
        let branch_addr = ((self.pc as i32) + ((offset as i8) as i32)) as u16;

        (data_addr, branch_addr)
    }

    fn absolute(&mut self) -> u16 {
        u16::from_le_bytes([self.read(self.pc + 1), self.read(self.pc + 2)])
    }

    fn absolute_x_indirect(&mut self) -> u16 {
        let ptr_addr = self.x_direct();

        self.read(ptr_addr) as u16
    }

    fn x_absolute(&mut self) -> u16 {
        self.absolute() + (self.x as u16)
    }

    fn y_absolute(&mut self) -> u16 {
        self.absolute() + (self.y as u16)
    }

    fn x_direct_relative(&mut self) -> (u16, u16) {
        let data_addr = self.x_direct();
        let offset = self.read(self.pc + 2);
        let branch_addr = ((self.pc as i32) + ((offset as i8) as i32)) as u16;

        (data_addr, branch_addr)
    }

    fn x_indirect(&mut self) -> u16 {
        let temp = self.x_direct();
        self.read(temp) as u16
    }

    fn indirect_y(&mut self) -> u16 {
        self.indirect() + (self.y as u16)
    }

    fn relative(&mut self) -> u16 {
        let offset = self.read(self.pc + 1);

        ((self.pc as i32) + ((offset as i8) as i32)) as u16
    }

    fn immediate_relative(&mut self) -> (u16, u16) {
        let data_addr = self.pc + 1;
        let offset = self.read(self.pc + 2);
        let branch_addr = ((self.pc as i32) + ((offset as i8) as i32)) as u16;

        (data_addr, branch_addr)
    }

    fn immediate(&mut self) -> u16 {
        self.pc + 1
    }
}

// CPU Instructions
impl Spc700 {
    fn add_16_base(&mut self, arg1: u16, arg2: u16) -> u16 {
        let result =
            (arg1 as u32) + (arg2 as u32) + if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        let half_result = (arg1 & 0xFFF) + (arg2 & 0xFFF);

        self.set_flag_to_bool(Flag::FlagC, result > 0xFFFF);
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagH, half_result >= 0xFFF);
        self.set_flag_to_bool(Flag::FlagZ, result & 0xFF == 0);

        // Set V flag if acc and data are same sign, but result is different sign
        let a = arg1.bit_en(15);
        let d = arg2.bit_en(15);
        let r = (result & 0x8000) != 0;
        self.set_flag_to_bool(Flag::FlagV, !(a ^ d) & (a ^ r));

        result as u16
    }

    fn adc_base(&mut self, arg1: u8, arg2: u8) -> u8 {
        let result =
            (arg1 as u16) + (arg2 as u16) + if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        let half_result = (arg1 & 0xF) + (arg2 & 0xF);

        self.set_flag_to_bool(Flag::FlagC, result > 0xFF);
        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagH, half_result >= 0xA);
        self.set_flag_to_bool(Flag::FlagZ, result & 0xFF == 0);

        // Set V flag if acc and data are same sign, but result is different sign
        let a = arg1.bit_en(7);
        let d = arg2.bit_en(7);
        let r = result.bit_en(7);
        self.set_flag_to_bool(Flag::FlagV, !(a ^ d) & (a ^ r));

        result as u8
    }

    fn adc_acc(&mut self, address: u16) {
        let data = self.read(address);
        self.acc = self.adc_base(self.acc, data);
    }

    fn adc_mem(&mut self, addr1: u16, addr2: u16) {
        let arg1 = self.read(addr1);
        let arg2 = self.read(addr2);

        let result = self.adc_base(arg1, arg2);

        self.write(addr1, result);
    }

    fn addw(&mut self, addr1: u16, addr2: u16) {
        let data = self.read_word(addr1, addr2);
        let ya = ((self.y as u16) << 8) | (self.acc as u16);
        let result = self.add_16_base(ya, data);

        self.y = (result >> 8) as u8;
        self.acc = result as u8;
    }

    // AND - AND Memory with Accumulator
    fn and_acc(&mut self, address: u16) {
        let data = self.read(address);
        let result = self.acc & data;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.acc = result;
    }

    fn and_mem(&mut self, addr1: u16, addr2: u16) {
        let arg1 = self.read(addr1);
        let arg2 = self.read(addr2);
        let result = arg1 & arg2;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write(addr1, result);
    }

    fn and1(&mut self, address: u16) {
        let data = self.read(address & 0x1FFF);
        let b = (address >> 13) as u8;

        self.set_flag_to_bool(Flag::FlagC, data.bit_en(b));
    }

    // ASL - Shift Left One Bit (Accumulator version)
    fn asl_acc(&mut self) {
        let result = self.acc << 1;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, self.acc.bit_en(7));

        self.acc = result;
    }

    // ASL - Shift Left One Bit (Memory version)
    fn asl_mem(&mut self, address: u16) {
        let data = self.read(address);
        let result = data << 1;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, data.bit_en(7));

        self.write(address, result);
    }

    // BBC - Branch if Bit Clear
    fn bbc(&mut self, data_addr: u16, branch_addr: u16, opcode: u8) {
        let b = opcode >> 5;
        let data = self.read(data_addr);
        if data & b == 0 {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BBS - Branch if Bit Set
    fn bbs(&mut self, data_addr: u16, branch_addr: u16, opcode: u8) {
        let b = opcode >> 5;
        let data = self.read(data_addr);
        if data.bit_en(b) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BCC - Branch if Carry Clear
    fn bcc(&mut self, branch_addr: u16) {
        if !self.is_flag_set(Flag::FlagC) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BCS - Branch if Carry Set
    fn bcs(&mut self, branch_addr: u16) {
        if self.is_flag_set(Flag::FlagC) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BEQ - Branch if EQual
    fn beq(&mut self, branch_addr: u16) {
        if self.is_flag_set(Flag::FlagZ) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BMI - Branch MInus
    fn bmi(&mut self, branch_addr: u16) {
        if self.is_flag_set(Flag::FlagN) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BNE - Branch if Not Equal
    fn bne(&mut self, branch_addr: u16) {
        if !self.is_flag_set(Flag::FlagZ) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BPL - Branch PLus (if positive)
    fn bpl(&mut self, branch_addr: u16) {
        if !self.is_flag_set(Flag::FlagN) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BRA - BRanch Always
    fn bra(&mut self, branch_addr: u16) {
        self.pc = branch_addr;
        self.branch_taken = true;
    }

    // BRK - Break
    // TODO: make sure it actually works this way
    fn brk(&mut self) {
        self.set_flag(Flag::FlagB);
    }

    // BVC - Branch if OVerflow Clear
    fn bvc(&mut self, branch_addr: u16) {
        if !self.is_flag_set(Flag::FlagV) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BVS - Branch if OVerflow Set
    fn bvs(&mut self, branch_addr: u16) {
        if self.is_flag_set(Flag::FlagV) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // CALL - call a subroutine
    fn call(&mut self, new_addr: u16) {
        self.push_word(self.pc + 1);
        self.pc = new_addr;
    }

    // CBNE - Compare and Branch if Not Equal
    fn cbne(&mut self, address: u16, branch_addr: u16) {
        self.cmp_acc(address);
        self.bne(branch_addr);

        if !self.is_flag_set(Flag::FlagZ) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // CMP - Compare Memory with Accumulator
    fn cmp_acc(&mut self, address: u16) {
        let data = self.read(address);

        let result = (self.acc as i16) - (data as i16);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagN, result < 0);
        self.set_flag_to_bool(Flag::FlagC, result >= 0);
    }

    fn cmp_mem(&mut self, addr1: u16, addr2: u16) {
        let arg1 = self.read(addr1);
        let arg2 = self.read(addr2);

        let result = (arg2 as i16) - (arg1 as i16);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagN, result < 0);
        self.set_flag_to_bool(Flag::FlagC, result >= 0);
    }

    // CLI - CLear Interrupt flag (called DI in SPC700 documentation)
    fn cli(&mut self) {
        self.clear_flag(Flag::FlagI);
    }

    // CLR1 - clears a single bit in the direct page
    fn clr1(&mut self, address: u16, opcode: u8) {
        let data = self.read(address);
        let b = 1 << (opcode >> 5);

        self.write(address, data & !b);
    }

    // CLRC - clear carry flag
    fn clrc(&mut self) {
        self.clear_flag(Flag::FlagC);
    }

    // CLRP - clear direct page flag
    fn clrp(&mut self) {
        self.clear_flag(Flag::FlagP);
        self.dir_page = 0;
    }

    // CLRV - clear carry flag
    fn clrv(&mut self) {
        self.clear_flag(Flag::FlagV);
    }

    // CMPW - Compare Word with YA
    fn cmpw(&mut self, addr_lo: u16, addr_hi: u16) {
        let lo = self.read(addr_lo);
        let hi = self.read(addr_hi);
        let data = ((hi as u16) << 8) | (lo as u16);
        let ya = ((self.y as u16) << 8) | (self.acc as u16);
        let result = (ya as i32) - (data as i32);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagN, result < 0);
        self.set_flag_to_bool(Flag::FlagC, result >= 0);
    }

    // CMX - Compare Memory with X
    fn cmx(&mut self, address: u16) {
        let data = self.read(address);

        let result = (self.x as i16) - (data as i16);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagN, result < 0);
        self.set_flag_to_bool(Flag::FlagC, result >= 0);
    }

    // CMY - Compare Memory with X
    fn cmy(&mut self, address: u16) {
        let data = self.read(address);

        let result = (self.y as i16) - (data as i16);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagN, result < 0);
        self.set_flag_to_bool(Flag::FlagC, result >= 0);
    }

    // DAA - Decimal Adjust Addition
    fn daa(&mut self) {
        if self.is_flag_set(Flag::FlagH) {
            self.acc += 6;
        }
    }

    // DAS - Decimal Adjust Subtraction
    fn das(&mut self) {
        if self.is_flag_set(Flag::FlagH) {
            self.acc -= 6;
        }
    }

    // DBNZ - Decrement and Branch if Not Zero (y register)
    fn dbnz_y(&mut self, branch_addr: u16) {
        self.y -= 1;
        if self.y != 0 {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // DBNZ - Decrement and Branch if Not Zero (memory)
    fn dbnz_mem(&mut self, address: u16, branch_addr: u16) {
        let data = self.read(address) - 1;
        self.write(address, data);
        if data != 0 {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // DEC - decrement (accumulator)
    fn dec_acc(&mut self) {
        self.acc -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    // DEC - decrement (memory)
    fn dec_mem(&mut self, address: u16) {
        let data = self.read(address) - 1;

        self.set_flag_to_bool(Flag::FlagN, data.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, data == 0);

        self.write(address, data);
    }

    fn decw(&mut self, addr1: u16, addr2: u16) {
        let data = self.read_word(addr1, addr2);
        let result = data - 1;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write_word(addr1, addr2, result);
    }

    fn dex(&mut self) {
        self.x -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.x.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn dey(&mut self) {
        self.y -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.y.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn div(&mut self) {
        let ya = ((self.y as u16) << 8) | (self.acc as u16);

        self.set_flag_to_bool(Flag::FlagH, (self.y & 0xF) >= (self.x & 0xF));
        self.set_flag_to_bool(Flag::FlagV, self.y >= self.x);

        if (self.y as u16) < ((self.x as u16) << 1) {
            let div_result = ya / self.x as u16;
            let mod_result = ya % self.x as u16;

            self.acc = div_result as u8;
            self.y = mod_result as u8;
        } else {
            self.acc = (255 - (ya - ((self.x as u16) << 9)) / (256 - (self.x as u16))) as u8;
            self.y =
                ((self.x as u16) + (ya - ((self.x as u16) << 9)) % (256 - (self.x as u16))) as u8;
        }

        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
    }

    fn eor_acc(&mut self, address: u16) {
        self.acc ^= self.read(address);

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn eor_mem(&mut self, addr1: u16, addr2: u16) {
        let arg1 = self.read(addr1);
        let arg2 = self.read(addr2);
        let result = arg1 ^ arg2;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write(addr1, result);
    }

    fn eor1(&mut self, address: u16) {
        let addr = address & 0x1FFF;
        let data = self.read(addr);
        let b = (address >> 13) as u8;
        let result = self.is_flag_set(Flag::FlagC) ^ (data.bit_en(b));

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn inc_acc(&mut self) {
        self.acc += 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn inc_mem(&mut self, address: u16) {
        let result = self.read(address) + 1;

        self.write(address, result);

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn incw(&mut self, addr_lo: u16, addr_hi: u16) {
        let result = self.read_word(addr_lo, addr_hi) + 1;

        self.write_word(addr_lo, addr_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn inx(&mut self) {
        self.x += 1;

        self.set_flag_to_bool(Flag::FlagN, self.x.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn iny(&mut self) {
        self.y += 1;

        self.set_flag_to_bool(Flag::FlagN, self.y.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    fn lda(&mut self, address: u16) {
        self.acc = self.read(address);

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0 && self.acc == 0);
    }

    fn ldc(&mut self, address: u16) {
        let addr = address & 0x1FFF;
        let data = self.read(addr);
        let b = (address >> 13) as u8;

        self.set_flag_to_bool(Flag::FlagC, data.bit_en(b));
    }

    fn ldx(&mut self, address: u16) {
        self.x = self.read(address);

        self.set_flag_to_bool(Flag::FlagN, self.x.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0 && self.acc == 0);
    }

    fn ldy(&mut self, address: u16) {
        self.y = self.read(address);

        self.set_flag_to_bool(Flag::FlagN, self.y.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0 && self.acc == 0);
    }

    fn ldya(&mut self, addr_lo: u16, addr_hi: u16) {
        self.y = self.read(addr_hi);
        self.acc = self.read(addr_lo);

        self.set_flag_to_bool(Flag::FlagN, self.y.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0 && self.acc == 0);
    }

    fn lsr_acc(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc.bit_en(0));

        self.acc >>= 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn lsr_mem(&mut self, address: u16) {
        let data = self.read(address);
        let result = data >> 1;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, data.bit_en(0));

        self.write(address, result);
    }

    fn mov(&mut self, src_addr: u16, dst_addr: u16) {
        let data = self.read(src_addr);

        self.write(dst_addr, data);
    }

    fn mul(&mut self) {
        let result = (self.y as u16) * (self.acc as u16);

        self.y = (result >> 8) as u8;
        self.acc = result as u8;

        self.set_flag_to_bool(Flag::FlagN, self.y.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn nop(&self) {}

    fn not1(&mut self, address: u16) {
        let addr = address & 0x1FFF;
        let data = self.read(addr);
        let b = (address >> 13) as u8;
        let result = data ^ b;

        self.write(addr, result);
    }

    fn notc(&mut self) {
        self.status ^= Flag::FlagC as u8;
    }

    fn or1(&mut self, address: u16) {
        let addr = address & 0x1FFF;
        let data = self.read(addr);
        let b = (address >> 13) as u8;
        let result = self.is_flag_set(Flag::FlagC) || !(data.bit_en(b));

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn or1_inv(&mut self, address: u16) {
        let addr = address & 0x1FFF;
        let data = self.read(addr);
        let b = (address >> 13) as u8;
        let result = self.is_flag_set(Flag::FlagC) || (data.bit_en(b));

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn or_acc(&mut self, address: u16) {
        let data = self.read(address);
        let result = self.acc | data;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn or_mem(&mut self, addr1: u16, addr2: u16) {
        let arg1 = self.read(addr1);
        let arg2 = self.read(addr2);
        let result = arg1 | arg2;

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn pcall(&mut self, address: u16) {
        let call_addr = 0xFF00 | self.read(address) as u16;

        self.push_word(self.pc);

        self.pc = call_addr;
    }

    fn pop_acc(&mut self) {
        self.acc = self.pop();
    }

    fn pop_x(&mut self) {
        self.x = self.pop();
    }

    fn pop_y(&mut self) {
        self.y = self.pop();
    }

    fn pop_psw(&mut self) {
        self.status = self.pop();

        if self.is_flag_set(Flag::FlagP) {
            self.dir_page = 0x100;
        } else {
            self.dir_page = 0;
        }
    }

    fn push_acc(&mut self) {
        self.push(self.acc);
    }

    fn push_x(&mut self) {
        self.push(self.x);
    }

    fn push_y(&mut self) {
        self.push(self.y);
    }

    fn push_psw(&mut self) {
        self.push(self.status);
    }

    fn ret(&mut self) {
        self.pc = self.pop_word();
    }

    fn ret1(&mut self) {
        self.status = self.pop();
        self.pc = self.pop_word();

        if self.is_flag_set(Flag::FlagP) {
            self.dir_page = 0x100;
        } else {
            self.dir_page = 0;
        }
    }

    fn rol_acc(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc.bit_en(7));

        self.acc <<= 1;
        self.acc |= if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn rol_mem(&mut self, address: u16) {
        let data = self.read(address);
        let result = (data << 1) | if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, data.bit_en(7));

        self.write(address, result);
    }

    fn ror_acc(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc.bit_en(0));

        self.acc >>= 1;
        self.acc |= if self.is_flag_set(Flag::FlagC) {
            0x80
        } else {
            0
        };

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn ror_mem(&mut self, address: u16) {
        let data = self.read(address);
        let result = (if self.is_flag_set(Flag::FlagC) {
            0x80
        } else {
            0
        }) | (data >> 1);

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, data.bit_en(0));

        self.write(address, result);
    }

    fn sbc_acc(&mut self, address: u16) {
        let data = self.read(address);
        let comp = (-1 * (data as i8)) as u8;
        self.acc = self.adc_base(self.acc, comp);
    }

    fn sbc_mem(&mut self, addr1: u16, addr2: u16) {
        let arg1 = self.read(addr1);
        let arg2 = self.read(addr2);
        let comp1 = (-1 * (arg1 as i8)) as u8;
        let comp2 = (-1 * (arg2 as i8)) as u8;
        let result = self.adc_base(comp1, comp2);

        self.write(addr1, result);
    }

    fn sei(&mut self) {
        self.set_flag(Flag::FlagI)
    }

    fn set1(&mut self, address: u16, opcode: u8) {
        let data = self.read(address);
        let b = 1 << (opcode >> 5);

        self.write(address, data | b);
    }

    fn setc(&mut self) {
        self.set_flag(Flag::FlagC);
    }

    fn setp(&mut self) {
        self.set_flag(Flag::FlagP);
        self.dir_page = 0x100;
    }

    fn sleep(&self) {}

    fn sta(&mut self, address: u16) {
        self.write(address, self.acc);
    }

    // MOV1 alias
    fn stc(&mut self, address: u16) {
        let addr = address & 0x1FFF;
        let bit = (address >> 8) as u8;

        if self.is_flag_set(Flag::FlagC) {
            self.set1(addr, bit);
        } else {
            self.clr1(addr, bit);
        }
    }

    fn stop(&self) {}

    fn stx(&mut self, address: u16) {
        self.write(address, self.y);
    }

    fn sty(&mut self, address: u16) {
        self.write(address, self.y);
    }

    fn stya(&mut self, addr_lo: u16, addr_hi: u16) {
        self.write(addr_lo, self.acc);
        self.write(addr_hi, self.y);
    }

    fn subw(&mut self, addr1: u16, addr2: u16) {
        let data = self.read_word(addr1, addr2);
        let comp = (-1 * (data as i16)) as u16;
        let ya = ((self.y as u16) << 8) | (self.acc as u16);
        let result = self.add_16_base(ya, comp);

        self.y = (result >> 8) as u8;
        self.acc = result as u8;
    }

    fn tax(&mut self) {
        self.x = self.acc;

        self.set_flag_to_bool(Flag::FlagN, self.x.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn tay(&mut self) {
        self.y = self.acc;

        self.set_flag_to_bool(Flag::FlagN, self.y.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn tcall(&mut self, opcode: u8) {
        let n = (opcode >> 4) as u16;
        let addr = 0xFFDE - (n << 1);

        self.push_word(self.pc);
        self.pc = self.read_word(addr, addr + 1);
    }

    fn tclr1(&mut self, address: u16) {
        let data = self.read(address);
        let result = data & self.acc;

        self.write(address, data & !self.acc);

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tset1(&mut self, address: u16) {
        let data = self.read(address);
        let result = data & self.acc;

        self.write(address, data | self.acc);

        self.set_flag_to_bool(Flag::FlagN, result.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tsx(&mut self) {
        self.x = self.sp;

        self.set_flag_to_bool(Flag::FlagN, self.x.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn txa(&mut self) {
        self.acc = self.x;

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn txs(&mut self) {
        self.sp = self.x;
    }

    fn tya(&mut self) {
        self.acc = self.y;

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn xcn(&mut self) {
        self.acc = (self.acc >> 4) | ((self.acc & 0xF) << 4);

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
}

