
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
    aram: [u8; 0x10000],
}

// Boot program for the SPC700
const IPS_ROM: [u8; 64] = [ 
    0xCD, 0xEF, 0xBD, 0xE8, 0x00, 0xC6, 0x1D, 0xD0, 0xFC, 0x8F, 0xAA, 0xF4, 0x8F, 0xBB, 0xF5, 0x78,
    0xCC, 0xF4, 0xD0, 0xFB, 0x2F, 0x19, 0xEB, 0xF4, 0xD0, 0xFC, 0x7E, 0xF4, 0xD0, 0x0B, 0xE4, 0xF5,
    0xCB, 0xF4, 0xD7, 0x00, 0xFC, 0xD0, 0xF3, 0xAB, 0x01, 0x10, 0xEF, 0x7E, 0xF4, 0x10, 0xEB, 0xBA,
    0xF6, 0xDA, 0x00, 0xBA, 0xF4, 0xC4, 0xF4, 0xDD, 0x5D, 0xD0, 0xDB, 0x1F, 0x00, 0x00, 0xC0, 0xFF,
];



impl Spc700 {
    fn clock(&mut self) {
        let opcode = self.aram[self.pc as usize];
        match opcode {
            0x84 => {
                self.
            }
            _ => ; // Lance is writing a python script to generate this.
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            (0xF0..=0xFF) => self.read_sound_regs(),
            (0xFFC0..=0xFFFF) if self.read_ipl => IPL_ROM[(address & 0x3F) as usize],
            _ => self.aram[address as usize],
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            (0xF0..=0xFF) => self.write_sound_regs(),
            _ => self.aram[address as usize] = data,
        }
    }

    fn exec_instr(&mut self, opcode: u8) {
        let mut cycles: usize = 0;
        match opcode {
            0x82  => {
            
            },
        }
    }
}


// Helper functions.
impl Spc700 {


    // ADDRESSING MODES - Fetches data from the bus
    // Returns:
    //  - Address of the data needed for the instruction
    //  - Number of extra self.cycles taken to get data
    //
    // Note: Implied addressing mode has no return type because no data is needed for instructions
    // with implied addressing mode.

    // Accumulator - like implied, no extra data needed. Accumulator is used as data
    fn accumulator(&self) -> u16 {
        0
    }
    // Implied - no extra data needed for this instruction, read no extra bytes
    fn implied(&self) -> u16 {
        0
    }
    // Immediate - data immediatly follows instruction
    fn immediate(&self) -> u16 {
        self.pc + 1
    }
    // Absolute - The next 2 bytes are the address in memory of the data to retrieve
    fn absolute(&self) -> u16 {
        let abs_address = self.read_word(self.pc + 1);
        abs_address
    }
    // Indexed Addressing (X) - Like Absolute, but adds the x register to the abs address to get
    // the "effective address," and uses that to fetch data from memory.
    // Also known as Absolute X addressing.
    fn absolute_x(&self) -> u16 {
        let abs_address = self.read_word(self.pc + 1);
        let effective_address = abs_address.wrapping_add(self.get_x_reg() as u16);

        let page_boundary_crossed: bool = (abs_address & 0xFF00) != (effective_address & 0xFF00);

        effective_address
    }
    // Indexed Addressing (Y) - Like same as Indexed x, but used the y register instead.
    // Also known as Absolute Y addressing.
    fn absolute_y(&self) -> u16 {
        let abs_address = self.read_word(self.pc + 1);
        let effective_address = abs_address.wrapping_add(self.get_y_reg() as u16);

        let page_boundary_crossed: bool = (abs_address & 0xFF00) != (effective_address & 0xFF00);

        effective_address
    }
    // Zero Page - Like absolute, but uses only 1 byte for address & uses 0x00 for the high byte of the address
    fn zero_page(&self) -> u16 {
        let zpage_address = self.read(self.pc + 1) as u16;
        zpage_address
    }
    // Indexed Addressing Zero-Page (X) - Like Indexed x, but uses only the single next byte as the
    // low order byte of the absolute address and fills the top byte w/ 0x00. Then adds x to get
    // the effective address. Note that the effective address will never go off the zero-page, if
    // the address exceeds 0x00FF, it will loop back around to 0x0000.
    // Also known as Zero Page X addressing.
    fn zpage_x(&self) -> u16 {
        let zpage_address = self.read(self.pc + 1);
        let effective_zpage_address = zpage_address.wrapping_add(self.get_x_reg()) as u16;
        
        effective_zpage_address
    }
    // Indexed Addressing Zero-Page (Y) - Like Indexed Z-Page x, but uses the y register instead
    // Also known as Zero Page Y addressing.
    fn zpage_y(&self) -> u16 {
        let zpage_address = self.read(self.pc + 1);
        let effective_zpage_address = zpage_address.wrapping_add(self.get_y_reg()) as u16;
        
        effective_zpage_address
    }
    // Indirect Addressing - Uses the next 2 bytes as the abs address, then reads the byte that
    // points to in memory and the one after (in LLHH order) and uses those as the effective
    // address where the data will be read from.
    // Note: This mode has a hardware bug where a page boundary cannot be crossed by 
    // the reading of 2 bytes from abs_address, and therefore it can take no
    // additional clock cycles.
    fn indirect(&self) -> u16 {
        let abs_address = self.read_word(self.pc + 1);

        let effective_lo = self.read(abs_address) as u16;
        let effective_hi = if abs_address & 0xFF == 0xFF {
            self.read(abs_address & 0xFF00)
        } else {
            self.read(abs_address + 1)
        } as u16;

        let effective_address = (effective_hi << 8) | effective_lo;

        effective_address
    }
    // Pre-Indexed Indirect Zero-Page (X) - Like Indexed Z-Page x, but as in Indirect addressing, another
    // address is read from memory instead of the data. The address read is the effective
    // address of the actual data. Note that if the z-page address is 0x00FF, then the bytes at
    // 0x00FF and 0x0000 are read and used as low and high bytes of the effective address, respectively
    fn indirect_x(&self) -> u16 {
        let zpage_address = self.read(self.pc + 1).wrapping_add(self.get_x_reg());
        let effective_address = self.read_zpage_word(zpage_address);
        
        effective_address
    }
    // Post-Indexed Indirect Zero-Page (Y) - Like Indirect Indexed Z-Page x, but with two major differences:
    // First, the y register is used instead of x. Second, the register is added to the address
    // retrieved from the z-page, not the address used to access the z-page.
    fn indirect_y(&self) -> u16 {
        let zpage_address = self.read(self.pc + 1);
        let abs_address = self.read_zpage_word(zpage_address);
        let effective_address = abs_address.wrapping_add(self.get_y_reg() as u16);
        
        let page_boundary_crossed = (abs_address & 0xFF00) != (effective_address & 0xFF00);

        effective_address
    }
    // Relative Addressing - Data used is next byte
    fn relative(&self) -> u16 {
       self.pc + 1
    }

    fn direct_page(&self) -> u16 {
        let zpage_address = self.read(self.pc + 1) as u16;
        if self.is_flag_set(Flag::FlagP) {
            zpage_address | 0x100
        } else {
            zpage_address
        }

    }

    // FLAG GETTERS AND SETTERS
    //
    //
    //
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

// CPU Instructions
impl Spc700 {
    fn adc(&mut self, arg1: u8, arg2: u8) -> u8 {
        let result = (arg1 as u16) + (arg2 as u16) + if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        self.set_flag_to_bool(Flag::FlagC, result & 0xFF00 > 0);
        self.set_flag_to_bool(Flag::FlagZ, result & 0xFF == 0);
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagH, result & 0xFF >= 0xA); // This flag used by DAA
        
        // Set V flag if acc and data are same sign, but result is different sign
        let a = arg1 & 0x80 != 0;
        let r = (result & 0x80) != 0;
        let d = arg2 & 0x80 != 0;
        self.set_flag_to_bool(Flag::FlagV, !(a^d)&(a^r) ); // Trust, bro
        (result as u8);
    }


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
