// Master Clock runs at 21.4773 MHz, and S-DSP internally clocks at 3.072 MHz
// The ratio of S-DSP Clock Period / Master Clock Period = (1/21477300 Hz) / (1/3072000 Hz)
// = 0.143034739 is approximated closely by 97/678 = 0.143067846608
// with an error of 0.02274%
const MASTER_CLOCK_TIME_UNITS: usize = 97;
const SDSP_CLOCK_TIME_UNITS: usize = 678;

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

struct Spc700 {
    pc: u16,
    sp: u8,
    acc: u8,
    x: u8,
    y: u8,
    status: u8,
    branch_taken: bool,
    read_ipl: bool,
    dir_page: u16,
    aram: [u8; 0x10000],
    time_since_last_clock: usize,
    sdsp_clocks: usize,
}

impl Spc700 {
    // Boot program for the SPC700
    const IPL_ROM: [u8; 64] = [ 
        0xCD, 0xEF, 0xBD, 0xE8, 0x00, 0xC6, 0x1D, 0xD0, 0xFC, 0x8F, 0xAA, 0xF4, 0x8F, 0xBB, 0xF5, 0x78,
        0xCC, 0xF4, 0xD0, 0xFB, 0x2F, 0x19, 0xEB, 0xF4, 0xD0, 0xFC, 0x7E, 0xF4, 0xD0, 0x0B, 0xE4, 0xF5,
        0xCB, 0xF4, 0xD7, 0x00, 0xFC, 0xD0, 0xF3, 0xAB, 0x01, 0x10, 0xEF, 0x7E, 0xF4, 0x10, 0xEB, 0xBA,
        0xF6, 0xDA, 0x00, 0xBA, 0xF4, 0xC4, 0xF4, 0xDD, 0x5D, 0xD0, 0xDB, 0x1F, 0x00, 0x00, 0xC0, 0xFF,
    ];

    fn clock(&mut self, master_clocks_elapsed: usize) {
        self.time_since_last_clock += master_clocks_elapsed * MASTER_CLOCK_TIME_UNITS;

        while self.time_since_last_clock > SDSP_CLOCK_TIME_UNITS {
            // self.sdsp.clock();
            
            self.sdsp_clocks = (self.sdsp_clocks + 1) % 3;

            // Spc700 clocks every 3 S-DSP cycles
            if self.sdsp_clocks == 0 {
                self.exec_instr();
            }

            self.time_since_last_clock -= SDSP_CLOCK_TIME_UNITS;
        }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            (0xF0..=0xFF) => self.read_sound_regs(),
            (0xFFC0..=0xFFFF) if self.read_ipl => Spc700::IPL_ROM[(address & 0x3F) as usize],
            _ => self.aram[address as usize],
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            (0xF0..=0xFF) => self.write_sound_regs(),
            _ => self.aram[address as usize] = data,
        }
    }

    fn exec_instr(&mut self) {
        let cycles: usize;

        let opcode = self.read(self.pc);

        match opcode {
            0x00 => {
                self.pc += 1;
                self.nop();
                cycles = 2;
            },
            0x01 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x02 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.set1(addr);
                cycles = 4;
            },
            0x03 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x04 => {
                let addr = self.direct();
                self.pc += 2;
                self.ora(addr);
                cycles = 3;
            },
            0x05 => {
                let addr = self.absolute();
                self.pc += 3;
                self.ora(addr);
                cycles = 4;
            },
            0x06 => {
                let addr = self.indirect();
                self.pc += 1;
                self.ora(addr);
                cycles = 3;
            },
            0x07 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.ora(addr);
                cycles = 6;
            },
            0x08 => {
                let addr = self.immediate();
                self.pc += 2;
                self.ora(addr);
                cycles = 2;
            },
            0x09 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                cycles = 6;
            },
            0x0A => {
                let addr = self.absolute_bit();
                self.pc += 3;
                self.or1(addr);
                cycles = 5;
            },
            0x0B => {
                let addr = self.direct();
                self.pc += 2;
                self.asl(addr);
                cycles = 4;
            },
            0x0C => {
                let addr = self.absolute();
                self.pc += 3;
                self.asl(addr);
                cycles = 5;
            },
            0x0D => {
                self.pc += 1;
                self.push();
                cycles = 4;
            },
            0x0E => {
                let addr = self.absolute();
                self.pc += 3;
                self.tset1(addr);
                cycles = 6;
            },
            0x0F => {
                self.pc += 1;
                self.brk();
                cycles = 8;
            },
            0x10 => {
                let addr = self.relative();
                self.pc += 2;
                self.bpl(addr);
                cycles = 2;
            },
            0x11 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x12 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.clr1(addr);
                cycles = 4;
            },
            0x13 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x14 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.ora(addr);
                cycles = 4;
            },
            0x15 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.ora(addr);
                cycles = 5;
            },
            0x16 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.ora(addr);
                cycles = 5;
            },
            0x17 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.ora(addr);
                cycles = 6;
            },
            0x18 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                cycles = 5;
            },
            0x19 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                cycles = 5;
            },
            0x1A => {
                let addr = self.direct();
                self.pc += 2;
                self.decw(addr);
                cycles = 6;
            },
            0x1B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.asl(addr);
                cycles = 5;
            },
            0x1C => {
                self.pc += 1;
                self.asl();
                cycles = 2;
            },
            0x1D => {
                self.pc += 1;
                self.dex();
                cycles = 2;
            },
            0x1E => {
                let addr = self.absolute();
                self.pc += 3;
                self.cpx(addr);
                cycles = 4;
            },
            0x1F => {
                let addr = self.absolute();
                self.pc += 3;
                self.jmp(addr);
                cycles = 6;
            },
            0x20 => {
                self.pc += 1;
                self.clrp();
                cycles = 2;
            },
            0x21 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x22 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.set1(addr);
                cycles = 4;
            },
            0x23 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x24 => {
                let addr = self.direct();
                self.pc += 2;
                self.and(addr);
                cycles = 3;
            },
            0x25 => {
                let addr = self.absolute();
                self.pc += 3;
                self.and(addr);
                cycles = 4;
            },
            0x26 => {
                let addr = self.indirect();
                self.pc += 1;
                self.and(addr);
                cycles = 3;
            },
            0x27 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.and(addr);
                cycles = 6;
            },
            0x28 => {
                let addr = self.immediate();
                self.pc += 2;
                self.and(addr);
                cycles = 2;
            },
            0x29 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                cycles = 6;
            },
            0x2A => {
                let addr = self.absolute_bit();
                self.pc += 3;
                self.or1(addr);
                cycles = 5;
            },
            0x2B => {
                let addr = self.direct();
                self.pc += 2;
                self.rol(addr);
                cycles = 4;
            },
            0x2C => {
                let addr = self.absolute();
                self.pc += 3;
                self.rol(addr);
                cycles = 5;
            },
            0x2D => {
                self.pc += 1;
                self.push();
                cycles = 4;
            },
            0x2E => {
                let addr = self.relative();
                self.pc += 2;
                self.cbne(addr);
                cycles = 5;
            },
            0x2F => {
                let addr = self.relative();
                self.pc += 2;
                self.bra(addr);
                cycles = 4;
            },
            0x30 => {
                let addr = self.relative();
                self.pc += 2;
                self.bmi(addr);
                cycles = 2;
            },
            0x31 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x32 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.clr1(addr);
                cycles = 4;
            },
            0x33 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x34 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.and(addr);
                cycles = 4;
            },
            0x35 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.and(addr);
                cycles = 5;
            },
            0x36 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.and(addr);
                cycles = 5;
            },
            0x37 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.and(addr);
                cycles = 6;
            },
            0x38 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                cycles = 5;
            },
            0x39 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                cycles = 5;
            },
            0x3A => {
                let addr = self.direct();
                self.pc += 2;
                self.incw(addr);
                cycles = 6;
            },
            0x3B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.rol(addr);
                cycles = 5;
            },
            0x3C => {
                self.pc += 1;
                self.rol();
                cycles = 2;
            },
            0x3D => {
                self.pc += 1;
                self.inx();
                cycles = 2;
            },
            0x3E => {
                let addr = self.direct();
                self.pc += 2;
                self.cpx(addr);
                cycles = 3;
            },
            0x3F => {
                let addr = self.absolute();
                self.pc += 3;
                self.call(addr);
                cycles = 8;
            },
            0x40 => {
                self.pc += 1;
                self.setp();
                cycles = 2;
            },
            0x41 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x42 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.set1(addr);
                cycles = 4;
            },
            0x43 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x44 => {
                let addr = self.direct();
                self.pc += 2;
                self.eor(addr);
                cycles = 3;
            },
            0x45 => {
                let addr = self.absolute();
                self.pc += 3;
                self.eor(addr);
                cycles = 4;
            },
            0x46 => {
                let addr = self.indirect();
                self.pc += 1;
                self.eor(addr);
                cycles = 3;
            },
            0x47 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.eor(addr);
                cycles = 6;
            },
            0x48 => {
                let addr = self.immediate();
                self.pc += 2;
                self.eor(addr);
                cycles = 2;
            },
            0x49 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                cycles = 6;
            },
            0x4A => {
                let addr = self.absolute_bit();
                self.pc += 3;
                self.and1(addr);
                cycles = 4;
            },
            0x4B => {
                let addr = self.direct();
                self.pc += 2;
                self.lsr(addr);
                cycles = 4;
            },
            0x4C => {
                let addr = self.absolute();
                self.pc += 3;
                self.lsr(addr);
                cycles = 5;
            },
            0x4D => {
                self.pc += 1;
                self.push();
                cycles = 4;
            },
            0x4E => {
                let addr = self.absolute();
                self.pc += 3;
                self.tclr1(addr);
                cycles = 6;
            },
            0x4F => {
                self.pc += 1;
                self.pcall();
                cycles = 6;
            },
            0x50 => {
                let addr = self.relative();
                self.pc += 2;
                self.bvc(addr);
                cycles = 2;
            },
            0x51 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x52 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.clr1(addr);
                cycles = 4;
            },
            0x53 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x54 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.eor(addr);
                cycles = 4;
            },
            0x55 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.eor(addr);
                cycles = 5;
            },
            0x56 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.eor(addr);
                cycles = 5;
            },
            0x57 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.eor(addr);
                cycles = 6;
            },
            0x58 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                cycles = 5;
            },
            0x59 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                cycles = 5;
            },
            0x5A => {
                let addr = self.direct();
                self.pc += 2;
                self.cmpw(addr);
                cycles = 4;
            },
            0x5B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.lsr(addr);
                cycles = 5;
            },
            0x5C => {
                self.pc += 1;
                self.lsr();
                cycles = 2;
            },
            0x5D => {
                self.pc += 1;
                self.ldx();
                cycles = 2;
            },
            0x5E => {
                let addr = self.absolute();
                self.pc += 3;
                self.cpy(addr);
                cycles = 4;
            },
            0x5F => {
                let addr = self.absolute();
                self.pc += 3;
                self.jmp(addr);
                cycles = 3;
            },
            0x60 => {
                self.pc += 1;
                self.clrc();
                cycles = 2;
            },
            0x61 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x62 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.set1(addr);
                cycles = 4;
            },
            0x63 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x64 => {
                let addr = self.direct();
                self.pc += 2;
                self.cmp(addr);
                cycles = 3;
            },
            0x65 => {
                let addr = self.absolute();
                self.pc += 3;
                self.cmp(addr);
                cycles = 4;
            },
            0x66 => {
                let addr = self.indirect();
                self.pc += 1;
                self.cmp(addr);
                cycles = 3;
            },
            0x67 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.cmp(addr);
                cycles = 6;
            },
            0x68 => {
                let addr = self.immediate();
                self.pc += 2;
                self.cmp(addr);
                cycles = 2;
            },
            0x69 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                cycles = 6;
            },
            0x6A => {
                let addr = self.absolute_bit();
                self.pc += 3;
                self.and1(addr);
                cycles = 4;
            },
            0x6B => {
                let addr = self.direct();
                self.pc += 2;
                self.ror(addr);
                cycles = 4;
            },
            0x6C => {
                let addr = self.absolute();
                self.pc += 3;
                self.ror(addr);
                cycles = 5;
            },
            0x6D => {
                self.pc += 1;
                self.push();
                cycles = 4;
            },
            0x6E => {
                let addr = self.relative();
                self.pc += 2;
                self.dbnz(addr);
                cycles = 5;
            },
            0x6F => {
                self.pc += 1;
                self.ret();
                cycles = 5;
            },
            0x70 => {
                let addr = self.relative();
                self.pc += 2;
                self.bvs(addr);
                cycles = 2;
            },
            0x71 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x72 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.clr1(addr);
                cycles = 4;
            },
            0x73 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x74 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.cmp(addr);
                cycles = 4;
            },
            0x75 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.cmp(addr);
                cycles = 5;
            },
            0x76 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.cmp(addr);
                cycles = 5;
            },
            0x77 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.cmp(addr);
                cycles = 6;
            },
            0x78 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                cycles = 5;
            },
            0x79 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                cycles = 5;
            },
            0x7A => {
                let addr = self.direct();
                self.pc += 2;
                self.addw(addr);
                cycles = 5;
            },
            0x7B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.ror(addr);
                cycles = 5;
            },
            0x7C => {
                self.pc += 1;
                self.ror();
                cycles = 2;
            },
            0x7D => {
                self.pc += 1;
                self.lda();
                cycles = 2;
            },
            0x7E => {
                let addr = self.direct();
                self.pc += 2;
                self.cpy(addr);
                cycles = 3;
            },
            0x7F => {
                self.pc += 1;
                self.ret1();
                cycles = 6;
            },
            0x80 => {
                self.pc += 1;
                self.setc();
                cycles = 2;
            },
            0x81 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x82 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.set1(addr);
                cycles = 4;
            },
            0x83 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x84 => {
                let addr = self.direct();
                self.pc += 2;
                self.adc(addr);
                cycles = 3;
            },
            0x85 => {
                let addr = self.absolute();
                self.pc += 3;
                self.adc(addr);
                cycles = 4;
            },
            0x86 => {
                let addr = self.indirect();
                self.pc += 1;
                self.adc(addr);
                cycles = 3;
            },
            0x87 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.adc(addr);
                cycles = 6;
            },
            0x88 => {
                let addr = self.immediate();
                self.pc += 2;
                self.adc(addr);
                cycles = 2;
            },
            0x89 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                cycles = 6;
            },
            0x8A => {
                let addr = self.absolute_bit();
                self.pc += 3;
                self.eor1(addr);
                cycles = 5;
            },
            0x8B => {
                let addr = self.direct();
                self.pc += 2;
                self.dec(addr);
                cycles = 4;
            },
            0x8C => {
                let addr = self.absolute();
                self.pc += 3;
                self.dec(addr);
                cycles = 5;
            },
            0x8D => {
                let addr = self.immediate();
                self.pc += 2;
                self.ldy(addr);
                cycles = 2;
            },
            0x8E => {
                self.pc += 1;
                self.pop();
                cycles = 4;
            },
            0x8F => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                cycles = 5;
            },
            0x90 => {
                let addr = self.relative();
                self.pc += 2;
                self.bcc(addr);
                cycles = 2;
            },
            0x91 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0x92 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.clr1(addr);
                cycles = 4;
            },
            0x93 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0x94 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.adc(addr);
                cycles = 4;
            },
            0x95 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.adc(addr);
                cycles = 5;
            },
            0x96 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.adc(addr);
                cycles = 5;
            },
            0x97 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.adc(addr);
                cycles = 6;
            },
            0x98 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                cycles = 5;
            },
            0x99 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                cycles = 5;
            },
            0x9A => {
                let addr = self.direct();
                self.pc += 2;
                self.subw(addr);
                cycles = 5;
            },
            0x9B => {
                let addr = self.x_direct();
                self.pc += 2;
                self.dec(addr);
                cycles = 5;
            },
            0x9C => {
                self.pc += 1;
                self.dec();
                cycles = 2;
            },
            0x9D => {
                self.pc += 1;
                self.ldx();
                cycles = 2;
            },
            0x9E => {
                self.pc += 1;
                self.div();
                cycles = 12;
            },
            0x9F => {
                self.pc += 1;
                self.xcn();
                cycles = 5;
            },
            0xA0 => {
                self.pc += 1;
                self.sei();
                cycles = 3;
            },
            0xA1 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0xA2 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.set1(addr);
                cycles = 4;
            },
            0xA3 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0xA4 => {
                let addr = self.direct();
                self.pc += 2;
                self.sbc(addr);
                cycles = 3;
            },
            0xA5 => {
                let addr = self.absolute();
                self.pc += 3;
                self.sbc(addr);
                cycles = 4;
            },
            0xA6 => {
                let addr = self.indirect();
                self.pc += 1;
                self.sbc(addr);
                cycles = 3;
            },
            0xA7 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.sbc(addr);
                cycles = 6;
            },
            0xA8 => {
                let addr = self.immediate();
                self.pc += 2;
                self.sbc(addr);
                cycles = 2;
            },
            0xA9 => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                cycles = 6;
            },
            0xAA => {
                let addr = self.absolute_bit();
                self.pc += 3;
                self.mov1(addr);
                cycles = 4;
            },
            0xAB => {
                let addr = self.direct();
                self.pc += 2;
                self.inc(addr);
                cycles = 4;
            },
            0xAC => {
                let addr = self.absolute();
                self.pc += 3;
                self.inc(addr);
                cycles = 5;
            },
            0xAD => {
                let addr = self.immediate();
                self.pc += 2;
                self.cpy(addr);
                cycles = 2;
            },
            0xAE => {
                self.pc += 1;
                self.pop();
                cycles = 4;
            },
            0xAF => {
                let addr = self.indirect_inc();
                self.pc += 1;
                self.sta(addr);
                cycles = 4;
            },
            0xB0 => {
                let addr = self.relative();
                self.pc += 2;
                self.bcs(addr);
                cycles = 2;
            },
            0xB1 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0xB2 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.clr1(addr);
                cycles = 4;
            },
            0xB3 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0xB4 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.sbc(addr);
                cycles = 4;
            },
            0xB5 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.sbc(addr);
                cycles = 5;
            },
            0xB6 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.sbc(addr);
                cycles = 5;
            },
            0xB7 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.sbc(addr);
                cycles = 6;
            },
            0xB8 => {
                let (addr1, addr2) = self.immediate_to_direct();
                self.pc += 3;
                cycles = 5;
            },
            0xB9 => {
                let (addr1, addr2) = self.indirect_to_indirect();
                self.pc += 1;
                cycles = 5;
            },
            0xBA => {
                let addr = self.direct();
                self.pc += 2;
                self.ldya(addr);
                cycles = 5;
            },
            0xBB => {
                let addr = self.x_direct();
                self.pc += 2;
                self.inc(addr);
                cycles = 5;
            },
            0xBC => {
                self.pc += 1;
                self.inc();
                cycles = 2;
            },
            0xBD => {
                self.pc += 1;
                self.stx();
                cycles = 2;
            },
            0xBE => {
                self.pc += 1;
                self.das();
                cycles = 3;
            },
            0xBF => {
                let addr = self.indirect_inc();
                self.pc += 1;
                self.lda(addr);
                cycles = 4;
            },
            0xC0 => {
                self.pc += 1;
                self.cli();
                cycles = 3;
            },
            0xC1 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0xC2 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.set1(addr);
                cycles = 4;
            },
            0xC3 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0xC4 => {
                let addr = self.direct();
                self.pc += 2;
                self.sta(addr);
                cycles = 4;
            },
            0xC5 => {
                let addr = self.absolute();
                self.pc += 3;
                self.sta(addr);
                cycles = 5;
            },
            0xC6 => {
                let addr = self.indirect();
                self.pc += 1;
                self.sta(addr);
                cycles = 4;
            },
            0xC7 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.sta(addr);
                cycles = 7;
            },
            0xC8 => {
                let addr = self.immediate();
                self.pc += 2;
                self.cpx(addr);
                cycles = 2;
            },
            0xC9 => {
                let addr = self.absolute();
                self.pc += 3;
                self.stx(addr);
                cycles = 5;
            },
            0xCA => {
                let addr = self.absolute_bit();
                self.pc += 3;
                self.mov1(addr);
                cycles = 6;
            },
            0xCB => {
                let addr = self.direct();
                self.pc += 2;
                self.sty(addr);
                cycles = 4;
            },
            0xCC => {
                let addr = self.absolute();
                self.pc += 3;
                self.sty(addr);
                cycles = 5;
            },
            0xCD => {
                let addr = self.immediate();
                self.pc += 2;
                self.ldx(addr);
                cycles = 2;
            },
            0xCE => {
                self.pc += 1;
                self.pop();
                cycles = 4;
            },
            0xCF => {
                self.pc += 1;
                self.mul();
                cycles = 9;
            },
            0xD0 => {
                let addr = self.relative();
                self.pc += 2;
                self.bne(addr);
                cycles = 2;
            },
            0xD1 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0xD2 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.clr1(addr);
                cycles = 4;
            },
            0xD3 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0xD4 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.sta(addr);
                cycles = 5;
            },
            0xD5 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.sta(addr);
                cycles = 6;
            },
            0xD6 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.sta(addr);
                cycles = 6;
            },
            0xD7 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.sta(addr);
                cycles = 7;
            },
            0xD8 => {
                let addr = self.direct();
                self.pc += 2;
                self.stx(addr);
                cycles = 4;
            },
            0xD9 => {
                let addr = self.y_direct();
                self.pc += 2;
                self.stx(addr);
                cycles = 5;
            },
            0xDA => {
                let addr = self.direct();
                self.pc += 2;
                self.stya(addr);
                cycles = 5;
            },
            0xDB => {
                let addr = self.x_direct();
                self.pc += 2;
                self.sty(addr);
                cycles = 5;
            },
            0xDC => {
                self.pc += 1;
                self.dey();
                cycles = 2;
            },
            0xDD => {
                self.pc += 1;
                self.lda();
                cycles = 2;
            },
            0xDE => {
                let addr = self.x_direct();
                self.pc += 2;
                self.cbne(addr);
                cycles = 6;
            },
            0xDF => {
                self.pc += 1;
                self.daa();
                cycles = 3;
            },
            0xE0 => {
                self.pc += 1;
                self.clrv();
                cycles = 2;
            },
            0xE1 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0xE2 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.set1(addr);
                cycles = 4;
            },
            0xE3 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0xE4 => {
                let addr = self.direct();
                self.pc += 2;
                self.lda(addr);
                cycles = 3;
            },
            0xE5 => {
                let addr = self.absolute();
                self.pc += 3;
                self.lda(addr);
                cycles = 4;
            },
            0xE6 => {
                let addr = self.indirect();
                self.pc += 1;
                self.lda(addr);
                cycles = 3;
            },
            0xE7 => {
                let addr = self.x_indirect();
                self.pc += 2;
                self.lda(addr);
                cycles = 6;
            },
            0xE8 => {
                let addr = self.immediate();
                self.pc += 2;
                self.lda(addr);
                cycles = 2;
            },
            0xE9 => {
                let addr = self.absolute();
                self.pc += 3;
                self.ldx(addr);
                cycles = 4;
            },
            0xEA => {
                let addr = self.absolute_bit();
                self.pc += 3;
                self.not1(addr);
                cycles = 5;
            },
            0xEB => {
                let addr = self.direct();
                self.pc += 2;
                self.ldy(addr);
                cycles = 3;
            },
            0xEC => {
                let addr = self.absolute();
                self.pc += 3;
                self.ldy(addr);
                cycles = 4;
            },
            0xED => {
                self.pc += 1;
                self.notc();
                cycles = 3;
            },
            0xEE => {
                self.pc += 1;
                self.pop();
                cycles = 4;
            },
            0xEF => {
                self.pc += 1;
                self.sleep();
                cycles = 0;
            },
            0xF0 => {
                let addr = self.relative();
                self.pc += 2;
                self.beq(addr);
                cycles = 2;
            },
            0xF1 => {
                self.pc += 1;
                self.tcall();
                cycles = 8;
            },
            0xF2 => {
                let addr = self.direct_bit();
                self.pc += 2;
                self.clr1(addr);
                cycles = 4;
            },
            0xF3 => {
                let (addr1, addr2) = self.direct_bit_relative();
                self.pc += 3;
                cycles = 5;
            },
            0xF4 => {
                let addr = self.x_direct();
                self.pc += 2;
                self.lda(addr);
                cycles = 4;
            },
            0xF5 => {
                let addr = self.x_absolute();
                self.pc += 3;
                self.lda(addr);
                cycles = 5;
            },
            0xF6 => {
                let addr = self.y_absolute();
                self.pc += 3;
                self.lda(addr);
                cycles = 5;
            },
            0xF7 => {
                let addr = self.indirect_y();
                self.pc += 2;
                self.lda(addr);
                cycles = 6;
            },
            0xF8 => {
                let addr = self.direct();
                self.pc += 2;
                self.ldx(addr);
                cycles = 3;
            },
            0xF9 => {
                let addr = self.y_direct();
                self.pc += 2;
                self.ldx(addr);
                cycles = 4;
            },
            0xFA => {
                let (addr1, addr2) = self.direct_to_direct();
                self.pc += 3;
                cycles = 5;
            },
            0xFB => {
                let addr = self.x_direct();
                self.pc += 2;
                self.ldy(addr);
                cycles = 4;
            },
            0xFC => {
                self.pc += 1;
                self.iny();
                cycles = 2;
            },
            0xFD => {
                self.pc += 1;
                self.ldy();
                cycles = 2;
            },
            0xFE => {
                let addr = self.relative();
                self.pc += 2;
                self.dbnz(addr);
                cycles = 4;
            },
            0xFF => {
                self.pc += 1;
                self.stop();
                cycles = 0;
            },
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

        if flag == Flag::FlagP {
            self.dir_page = 0x100;
        }
    }
    fn clear_flag(&mut self, flag: Flag) {
        self.status &= !(flag as u8);

        if flag == Flag::FlagP {
            self.dir_page = 0;
        }
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
    fn direct(&self) -> u16 {
        (self.read(self.pc + 1) as u16) | self.dir_page
    }

    fn x_direct(&self) -> u16 {
        ((self.read(self.pc + 1) + self.x) as u16) | self.dir_page
    }

    fn y_direct(&self) -> u16 {
        ((self.read(self.pc + 1) + self.y) as u16) | self.dir_page
    }

    fn indirect(&self) -> u16 {
        (self.x as u16) | self.dir_page
    }

    fn indirect_inc(&mut self) -> u16 {
        let addr = (self.x as u16) | self.dir_page;
        self.x += 1;

        addr
    }

    fn direct_to_direct(&self) -> (u16, u16) {
        let src_addr = (self.read(self.pc + 2) as u16) | self.dir_page;
        let dst_addr = (self.read(self.pc + 1) as u16) | self.dir_page;

        (src_addr, dst_addr)
    }

    fn indirect_to_indirect(&self) -> (u16, u16) {
        let src_addr = (self.y as u16) | self.dir_page;
        let dst_addr = (self.x as u16) | self.dir_page;

        (src_addr, dst_addr)
    }

    fn immediate_to_direct(&self) -> (u16, u16) {
        let src_addr = ((self.pc + 2) as u16) | self.dir_page;
        let dst_addr = (self.read(self.pc + 1) as u16) | self.dir_page;

        (src_addr, dst_addr)
    }

    fn direct_bit(&self) -> u16 {
        self.direct()
    }

    fn direct_bit_relative(&self) -> (u16, u16) {
        let data_addr = self.direct();
        let rel_addr = self.pc + (self.read(self.pc + 2)) as u16;

        (data_addr, rel_addr)
    }

    fn absolute_bit(&self) -> u16 {
        self.absolute()
    }

    fn absolute(&self) -> u16 {
        u16::from_le_bytes([
            self.read(self.pc + 1),
            self.read(self.pc + 2),
        ])
    }

    fn absolute_x_indirect(&self) -> u16 {
        let ptr_addr = self.x_direct();

        self.read(ptr_addr) as u16
    }

    fn x_absolute(&self) -> u16 {
        self.absolute() + (self.x as u16)
    }

    fn y_absolute(&self) -> u16 {
        self.absolute() + (self.y as u16)
    }

    fn x_indirect(&self) -> u16 {
        self.read(self.x_direct()) as u16
    }

    fn indirect_y(&self) -> u16 {
        self.indirect() + (self.y as u16)
    }

    fn relative(&self) -> u16 {
        self.pc + (self.read(self.pc + 1) as u16)
    }

    fn immediate(&self) -> u16 {
        self.pc + 1
    }
}

// CPU Instructions
impl Spc700 {
    fn adc_base(&mut self, arg1: u8, arg2: u8) -> u8 {
        let result = (arg1 as u16) + (arg2 as u16) + if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        let half_result = (arg1 & 0xF) + (arg2 & 0xF);

        self.set_flag_to_bool(Flag::FlagC, result & 0xFF00 > 0);
        self.set_flag_to_bool(Flag::FlagZ, result & 0xFF == 0);
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagH, half_result >= 0xA);
        
        // Set V flag if acc and data are same sign, but result is different sign
        let a = arg1 & 0x80 != 0;
        let d = arg2 & 0x80 != 0;
        let r = (result & 0x80) != 0;
        self.set_flag_to_bool(Flag::FlagV, !(a^d)&(a^r) ); // Trust, bro
        
        result as u8
    }

    // ADC
    // ADDW
    // AND
    // AND1
    // ASL
    // BBC
    // BBS
    // BCC
    // BCS
    // BEQ
    // BMI
    // BNE
    // BPL
    // BRA
    // BRK
    // BVC
    // BVS
    // CALL
    // CBNE
    // CLI
    // CLR1
    // CLRC
    // CLRP
    // CLRV
    // CMP
    // CMPW
    // CPX
    // CPY
    // DAA
    // DAS
    // DBNZ
    // DEC
    // DECW
    // DEX
    // DEY
    // DIV
    // EOR
    // EOR1
    // INC
    // INCW
    // INX
    // INY
    // JMP
    // LDA
    // LDX
    // LDY
    // LDYA
    // LSR
    // MOV
    // MOV1
    // MUL
    // NOP
    // NOT1
    // NOTC
    // OR
    // OR1
    // ORA
    // PCALL
    // POP
    // PUSH
    // RET
    // RET1
    // ROL
    // ROR
    // SBC
    // SEI
    // SET1
    // SETC
    // SETP
    // SLEEP
    // STA
    // STOP
    // STX
    // STY
    // STYA
    // SUBW
    // TCALL
    // TCLR1
    // TSET1
    // XCN
}


// AND - AND Memory with Accumulator
fn and(&mut self, address: u16) {
    let data = self.read(address);

    let result = self.acc & data;
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.acc = result;

}
// ASL - Shift Left One Bit (Accumulator version)
fn asl_acc(&mut self, _address: u16) {
    let result = self.acc << 1;
    self.set_flag_to_bool(Flag::FlagC, (cpu.acc & 0x80) != 0);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.acc = result;


}
// ASL - Shift Left One Bit (Memory version)
fn asl_mem(&mut self, address: u16) {
    let data = self.read(address);
    let result = data << 1;
    self.set_flag_to_bool(Flag::FlagC, (data & 0x80) != 0);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.write(address, result);

}
// BCC - Branch on Carry Clear
fn bcc(&mut self, address: u16) {
    let offset = self.read(address) as i8;

    if !self.carry() {
        let prev_pc = self.get_pc();
        self.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == self.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }

}
// BCS - Branch on Carry Set
fn bcs(&mut self, address: u16) {
    let offset = self.read(address) as i8;

    if self.carry() {
        let prev_pc = self.get_pc();
        self.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == self.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }


}
// BEQ - Branch on Equal (Zero flag set)
fn beq(&mut self, address: u16) {
    let offset = self.read(address) as i8;

    if self.zero() {
        let prev_pc = self.get_pc();
        self.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == self.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }
    

}
// BIT - Test Bits in Memory with Accumulator
fn bit(&mut self, address: u16) {
    let data = self.read(address);

    self.set_flag_to_bool(Flag::FlagN, data & 0x80 != 0);
    self.set_flag_to_bool(Flag::FlagV, data & 0x40 != 0);
    self.set_flag_to_bool(Flag::FlagZ, data & cpu.acc == 0);

}
// BMI - Branch on Result Minus (Negative flag set)
fn bmi(&mut self, address: u16) {
    let offset = self.read(address) as i8;

    if self.negative() {
        let prev_pc = self.get_pc();
        self.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == self.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }

}
// BNE - Branch on Not Equal (Zero flag NOT set)
fn bne(&mut self, address: u16) {
    let offset = self.read(address) as i8;

    if !self.zero() {
        let prev_pc = self.get_pc();
        self.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == self.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }

}
// BPL - Branch on Result Plus (Negative flag NOT set)
fn bpl(&mut self, address: u16) {
    let offset = self.read(address) as i8;

    if !self.negative() {
        let prev_pc = self.get_pc();
        self.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == self.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }

}
// BRK - Force Break (Initiate interrupt)
fn brk(&mut self, _address: u16) {
    self.irq();

}
// BVC - Branch on Overflow clear
fn bvc(&mut self, address: u16) {
    let offset = self.read(address) as i8;

    if !self.overflow() {
        let prev_pc = self.get_pc();
        self.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == self.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }

}
// BVS - Branch on Overflow set
fn bvs(&mut self, address: u16) {
    let offset = self.read(address) as i8;

    if self.overflow() {
        let prev_pc = self.get_pc();
        self.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == self.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }

}
// CLC - Clear Carry Flag
fn clc(&mut self, _address: u16) {
    self.set_flag_to_bool(Flag::FlagC, false);

}
// CLD - Clear Decimal Mode
fn cld(&mut self, _address: u16) {
    self.set_decimal(false);

}
// CLI - Clear Interrupt Disable Bit
fn cli(&mut self, _address: u16) {
    self.set_flag_to_bool(Flag::FlagI, false);

}
// CLV - Clear Overflow Flag
fn clv(&mut self, _address: u16) {
    self.set_flag_to_bool(Flag::FlagV, false);

}
// CMP - Compare Memory with Accumulator
fn cmp(&mut self, address: u16) {
    let data = self.read(address);

    let result = (self.acc as i16) - (data as i16);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.set_flag_to_bool(Flag::FlagC, result >= 0);

}
// CPX - Compare Memory and Index X
fn cpx(&mut self, address: u16) {
    let data = self.read(address);

    let result = (self.x as i16) - (data as i16);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.set_flag_to_bool(Flag::FlagC, result >= 0);

}
// CPY - Compare Memory and Index Y
fn cpy(&mut self, address: u16) {
    let data = self.read(address);

    let result = (self.y as i16) - (data as i16);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.set_flag_to_bool(Flag::FlagC, result >= 0);

}
// DEC - Decrement Memory
fn dec(&mut self, address: u16) {
    let data = self.read(address);

    let result = data.wrapping_sub(1);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.write(address, result);

}
// DEX - Decrement X Register
fn dex(&mut self, _address: u16) {
    let result = self.x.wrapping_sub(1);
    
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.x = result;

}
// DEY - Decrement Y Register
fn dey(&mut self, _address: u16) {
    let result = self.y.wrapping_sub(1);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.y = result;

}
// EOR - Exclusive OR
fn eor(&mut self, address: u16) {
    let data = self.read(address);

    let result = self.acc ^ data;
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.acc = result;

}
// INC - Increment Memory
fn inc(&mut self, address: u16) {
    let data = self.read(address);

    // NOTE: The nesdev page on MMC1 (mapper 1) notes that the inc instr writes to
    // memory twice. Once before the increment (with the unchanged data), and once after.
    // This may be a source of error with mapper 1, as we aren't doing that rn.

    let result = data.wrapping_add(1);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.write(address, result);

}
// INX - Increment X Register
fn inx(&mut self, _address: u16) {
    let result = self.x.wrapping_add(1);

    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.x = result;

}
// INY - Increment Y Register
fn iny(&mut self, _address: u16) {
    let result = self.y.wrapping_add(1);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.y = result;

}
// JMP - Jump
fn jmp(&mut self, address: u16) {
    self.set_pc(address);

}
// JSR - Jump to Subroutine
fn jsr(&mut self, address: u16) {
    let return_point = self.get_pc().wrapping_sub(1); // Return point is pc - 1
    let hi = (return_point >> 8) as u8;
    let lo = return_point as u8;

    self.push_to_stack(hi);
    self.push_to_stack(lo);
    self.set_pc(address);


}
// LDA - Load Accumulator
fn lda(&mut self, address: u16) {
    let data = self.read(address);

    self.set_flag_to_bool(Flag::FlagZ, data == 0);
    self.set_flag_to_bool(Flag::FlagN, data & 0x80 != 0);
    self.acc = data;

}
// LDX - Load X Register
fn ldx(&mut self, address: u16) {
    let data = self.read(address);

    self.set_flag_to_bool(Flag::FlagZ, data == 0);
    self.set_flag_to_bool(Flag::FlagN, data & 0x80 != 0);
    self.x = data;

}
// LDY - Load Y Register
fn ldy(&mut self, address: u16) {
    let data = self.read(address);

    self.set_flag_to_bool(Flag::FlagZ, data == 0);
    self.set_flag_to_bool(Flag::FlagN, data & 0x80 != 0);
    self.y = data;

}
// LSR - Logical Shift Right (Accumulator version)
fn lsr_acc(&mut self, _address: u16) {
    let result = self.acc >> 1;
    self.set_flag_to_bool(Flag::FlagC, cpu.acc & 0x01 == 1);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, false); // result will always have bit 7 == 0
    self.acc = result;

}
// LSR - Logical Shift Right (Memory version)
fn lsr_mem(&mut self, address: u16) {
    let data = self.read(address);
    let result = data >> 1;
    self.set_flag_to_bool(Flag::FlagC, data & 0x01 == 1);
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, false); // result will always have bit 7 == 0
    self.write(address, result);

}
// NOP - No Operation
fn nop(_: &mut self.502, _address: u16) { 0 }
// ORA - Logical Inclusive OR
fn ora(&mut self, address: u16) {
    let data = self.read(address);

    let result = self.acc | data;
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.acc = result;

}
// PHA - Push Accumulator
fn pha(&mut self, _address: u16) {
    self.push_to_stack(cpu.acc);

}
// PHP - Push Processor Status
fn php(&mut self, _address: u16) {
    // Bit 5 (unused flag) is always set to 1 when status pushed to stack
    // Bit 4 (break flag) is set when push to stk caused by php or brk
    self.push_to_stack(cpu.get_status() | 0x30);

}
// PLA - Pull Accumulator
fn pla(&mut self, _address: u16) {
    let result = self.pop_from_stack();
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.acc = result;

}
// PLP - Pull Processor Status
fn plp(&mut self, _address: u16) {
    // Bit 5 is ignored when pulling into processor status
    // Bit 4 is cleared
    let data = self.pop_from_stack() & 0xCF;
    self.set_status(data | (cpu.get_status() & 0x20));

}
// ROL - Rotate Left (Accumulator version)
fn rol_acc(&mut self, _address: u16) {
    let result = (self.acc << 1) | if cpu.carry() { 1 } else { 0 };
    self.set_flag_to_bool(Flag::FlagC, cpu.acc >> 7 == 1); // old bit 7 becomes new carry
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.acc = result;

}
// ROL - Rotate Left (Memory version)
fn rol_mem(&mut self, address: u16) {
    let data = self.read(address);
    let result = (data << 1) | if self.carry() { 1 } else { 0 };
    self.set_flag_to_bool(Flag::FlagC, data >> 7 == 1); // old bit 7 becomes new carry
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.write(address, result);

}
// ROR - Rotate Right (Accumulator version)
fn ror_acc(&mut self, _address: u16) {
    let result = (if self.carry() { 1 } else { 0 } << 7) | (cpu.acc >> 1);
    self.set_flag_to_bool(Flag::FlagC, cpu.acc & 0x01 == 1); // old bit 0 becomes new carry
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.acc = result;

}
// ROR - Rotate Right (Memory version)
fn ror_mem(&mut self, address: u16) {
    let data = self.read(address);
    let result = (if self.carry() { 1 } else { 0 } << 7) | (data >> 1);
    self.set_flag_to_bool(Flag::FlagC, data & 0x01 == 1); // old bit 0 becomes new carry
    self.set_flag_to_bool(Flag::FlagZ, result == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    self.write(address, result);

}
// RTI - Return from Interrupt
fn rti(&mut self, _address: u16) {
    // Restore processer status (Bit 5 ignored, bit 4 cleared)
    let prev_status = self.pop_from_stack() & 0xCF;
    self.set_status(prev_status | (cpu.get_status() & 0x20));
    // Return to previous PC
    let lo = self.pop_from_stack() as u16;
    let hi = self.pop_from_stack() as u16;
    self.set_pc((hi << 8) | lo);


}
// RTS - Return from Subroutine
fn rts(&mut self, _address: u16) {
    let lo = self.pop_from_stack() as u16;
    let hi = self.pop_from_stack() as u16;
    let new_pc = (hi << 8) | lo;
    self.set_pc(new_pc.wrapping_add(1));

}
// SBC - Subtract with Carry
//  Note: instr 0xEB (illegal opcode) executes the same as 0xE9, which is legal.
//        0xEB is differentiated in the table only by the name "USBC" for "Undocumented SBC"
fn sbc(&mut self, address: u16) {
    let data = self.read(address);
    // Add with carry: A + M + C
    // Sub with carry: A - M - (1 - C) == A + (-M - 1) + C
    let twos_comp = (data as i8).overflowing_mul(-1).0.wrapping_sub(1) as u8;
    // let twos_comp = (data as i8 * -1).wrapping_sub(1) as u8; // errors
    // let (twos_comp, _) = data.overflowing_neg();       // wrong behavior

    // ADC w/ two's compliment instead of original data
    let result = (twos_comp as u16) + (self.acc as u16) + (cpu.carry() as u16);
    self.set_flag_to_bool(Flag::FlagC, result & 0xFF00 > 0);
    self.set_flag_to_bool(Flag::FlagZ, result & 0xFF == 0);
    self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
    
    // Set V flag if acc and data are same sign, but result is different sign
    let a = self.acc & 0x80 != 0;
    let r = (result & 0x80) != 0;
    let d = twos_comp & 0x80 != 0;
    self.set_flag_to_bool(Flag::FlagV,  !(a^d)&(a^r) ); // Trust, bro

    self.acc = result as u8;


}
// SEC - Set Carry Flag
fn sec(&mut self, _address: u16) {
    self.set_flag_to_bool(Flag::FlagC, true);

}
// SED - Set Decimal Flag
fn sed(&mut self, _address: u16) {
    self.set_decimal(true);

}
// SEI - Set Interrupt Disable
fn sei(&mut self, _address: u16) {
    self.set_flag_to_bool(Flag::FlagI, true);

}
// STA - Store Accumulator
fn sta(&mut self, address: u16) {
    self.write(address, cpu.acc);

}
// STX - Store X Register
fn stx(&mut self, address: u16) {
    self.write(address, cpu.x);

}
// STY - Store Y Register
fn sty(&mut self, address: u16) {
    self.write(address, cpu.y);

}
// TAX - Transfer Accumulator to X
fn tax(&mut self, _address: u16) {
    self.x = cpu.acc;
    self.set_flag_to_bool(Flag::FlagZ, cpu.x == 0);
    self.set_flag_to_bool(Flag::FlagN, cpu.x & 0x80 != 0);

}
// TAY - Transfer Accumulator to Y
fn tay(&mut self, _address: u16) {
    self.y = cpu.acc;
    self.set_flag_to_bool(Flag::FlagZ, cpu.y == 0);
    self.set_flag_to_bool(Flag::FlagN, cpu.y & 0x80 != 0);

}
// TSX - Transfer Stack Pointer to X
fn tsx(&mut self, _address: u16) {
    self.x = cpu.get_sp();
    self.set_flag_to_bool(Flag::FlagZ, cpu.x == 0);
    self.set_flag_to_bool(Flag::FlagN, cpu.x & 0x80 != 0);

}
// TXA - Transfer X to Accumulator
fn txa(&mut self, _address: u16) {
    self.acc = cpu.x;
    self.set_flag_to_bool(Flag::FlagZ, cpu.acc == 0);
    self.set_flag_to_bool(Flag::FlagN, cpu.acc & 0x80 != 0);

}
// TXS - Transfer X to Stack Pointer
fn txs(&mut self, _address: u16) {
    self.set_sp(cpu.x);

}
// TYA - Transfer Y to Accumulator
fn tya(&mut self, _address: u16) {
    self.acc = cpu.y;
    self.set_flag_to_bool(Flag::FlagZ, cpu.acc == 0);
    self.set_flag_to_bool(Flag::FlagN, cpu.acc & 0x80 != 0);

}


/// ======== ILLEGAL OPCODES ========

// INVALID OPCODE - An unimplemented opcode not recognized by the self.
//                  Placeholder for all unimplemented illegal opcodes.
fn xxx(_: &mut self.502, _address: u16) { 0 }


// LAX - Load Accumulator and X Register
fn lax(&mut self, address: u16) {
    let data = self.read(address);

    self.set_flag_to_bool(Flag::FlagZ, data == 0);
    self.set_flag_to_bool(Flag::FlagN, data & 0x80 != 0);
    self.acc = data;
    self.x = data;

}

// SAX - Store Accumulator & X Register (bitwise acc & x)
fn sax(&mut self, address: u16) {
    let result = self.acc & cpu.x;
    self.write(address, result);


}

// Doing these illegal opcodes using other opcode functions may result in ppu
// registers being incorrectly incremented. definitely something to be aware of.

// DCP - Decrement Memory and Compare with Accumulator
fn dcp(&mut self, address: u16) -> usize {
    dec(self. address);
    cmp(self. address);
    0
}

// ISC - Increment Memory and Subtract with Carry
fn isc(&mut self, address: u16) -> usize {
    inc(self. address);    
    sbc(self. address);
    0
}

// SLO - Arithmetic Shift Left then Logical Inclusive OR
fn slo(&mut self, address: u16) -> usize {
    asl_mem(self. address);
    ora(self. address);
    0
}

// RLA - Rotate Left then Logical AND with Accumulator
fn rla(&mut self, address: u16) -> usize {
    rol_mem(self. address);
    and(self. address);
    0
}

// SRE - Logical Shift Right then "Exclusive OR" Memory with Accumulator
fn sre(&mut self, address: u16) -> usize {
    lsr_mem(self. address);
    eor(self. address);
    0
}

// RRA - Rotate Right and Add Memory to Accumulator
fn rra(&mut self, address: u16) -> usize {
    ror_mem(self. address);
    adc(self. address);
    0
}

// ANC - Bitwise AND Memory with Accumulator then Move Negative Flag to Carry Flag
fn anc(&mut self, address: u16) -> usize {
    and(self. address);

    self.status.set_carry(cpu.status.negative());

    0
}

// ASR - Bitwise AND Memory with Accumulator then Logical Shift Right
fn asr(&mut self, address: u16) -> usize {
    and(self. address);
    lsr_acc(self. address);
    0
}

// ARR - Bitwise AND Memory with Accumulator then Rotate Right
fn arr(&mut self, address: u16) -> usize {
    and(self. address);
    ror_acc(self. address);
    0
}

// LXA - Load Accumulator and Index Register X From Memory
fn lxa(&mut self, address: u16) -> usize {
    // const RAND_CONST: u8 = 0xEE; // This instruction is highly unstable, and
                                 // this constant may take on a value of 00, FF, 
                                 // EE, etc. depending on the state of the
                                 // device (like temperature)   

    // let result = (self.get_acc() | RAND_CONST) & cpu.get_x_reg();
    // self.status.set_zero(result == 0);
    // self.status.set_negative(result & 0x80 != 0);
    // self.set_acc(result);

    0
}

// SHY - Store Index Register Y Bitwise AND Value
fn shy(&mut self, address: u16) -> usize {
    let data = (address.wrapping_sub(self.get_x_reg() as u16) >> 8) as u8; // undo shift from x offset

    let result = self.get_y_reg() & data.wrapping_add(1);

    self.write(address, result);

    0
}

// SHX - Store Index Register X Bitwise AND Value
fn shx(&mut self, address: u16) -> usize {
    let data = (address.wrapping_sub(self.get_y_reg() as u16) >> 8) as u8; // undo shift from x offset

    let result = self.get_x_reg() & data.wrapping_add(1);

    self.write(address, result);

    0
}


/// Adds two 8-bit integers and returns the result
fn add_with_carry(arg1: u8, arg2: u8, cin: bool) -> (u8, bool) {
    let result = (arg1 as u16) + (arg2 as u16) + if cin { 1 } else { 0 };
    (result as u8, result > 255)
}
