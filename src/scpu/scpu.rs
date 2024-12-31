use serde::{ser::SerializeStruct, Serialize};

#[derive(Default, Clone, Copy)]
enum CpuMode {
    #[default]
    Emulation,
    Native8Bit,
    Native16Bit,
}

pub enum Flag {
    FlagC = 1,   // Carry
    FlagZ = 2,   // Zero
    FlagI = 4,   // IRQ Disable
    FlagD = 8,   // Decimal Mode
    FlagX = 16,  // X Register Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagM = 32,  // Accumulator Size (Native mode only; 0: 16-bit, 1: 8-bit)
    FlagV = 64,  // Overflow
    FlagN = 128, // Negative
    // FLAG_B = 16, // Break (Emulation mode only, same place as X flag)
}

trait CpuAddress {
    fn bank(self) -> u8;
    fn bank_addr(self) -> u16;
    fn with_bank(self, bank: u8) -> Self;
    fn with_bank_addr(self, bank_addr: u16) -> Self;
}

impl CpuAddress for u32 {
    fn bank(self) -> u8 { (self >> 16) as u8 }
    fn bank_addr(self) -> u16 { self as u16 }
    fn with_bank(self, bank: u8) -> Self { (self & 0x00FFFF) | (bank as u32) }
    fn with_bank_addr(self, bank_addr: u16) -> Self { (self & 0xFF0000) | (bank_addr as u32) }
}

#[derive(Default)]
pub struct Cpu65c816 {
    // Internal Registers
    acc: u16,
    x: u16,
    y: u16,
    pc: u16,
    stk_ptr: u16,
    direct: u16,
    data_bank: u8,
    prg_bank: u8,
    status: u8,

    mode: CpuMode,
    branch_taken: bool,

    wram: Vec<u8>,
}

impl Serialize for Cpu65c816 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
                let mut s = serializer.serialize_struct("Cpu65c816", 11)?;
                s.serialize_field("acc", &self.acc)?;
                s.serialize_field("x", &self.x);
                s.serialize_field("y", &self.y);
                s.serialize_field("pc", &self.pc);
                s.serialize_field("stk_ptr", &self.stk_ptr);
                s.serialize_field("direct", &self.direct);
                s.serialize_field("data_bank", &self.data_bank);
                s.serialize_field("prg_bank", &self.prg_bank);
                s.serialize_field("status", &self.status);
                s.serialize_field("mode", &(self.mode as u8));
                s.serialize_field("branch_taken", &self.branch_taken);
                s.end()
    }
}

// SNES System Functionality
impl Cpu65c816 {
    // Creates a new, uninitialized 65c816 CPU
    pub fn new() -> Self {
        Self::default()
    }
}

// Internal Helper Functions
impl Cpu65c816 {
    fn read8(&self, address: u32) -> u8 { self.wram[address as usize] }
    fn write8(&mut self, address: u32, data: u8) { self.wram[address as usize] = data; }
    fn read16(&self, address_lo: u32, address_hi: u32) -> u16 {
        u16::from_le_bytes([
            self.read8(address_lo),
            self.read8(address_hi)
        ])
    }
    fn write16(&mut self, address_lo: u32, address_hi: u32, data: u16) {
        self.write8(address_lo, data as u8);
        self.write8(address_hi, (data >> 8) as u8);
    }

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

    fn get_acc_hi(&self) -> u8 { (self.acc >> 8) as u8 }
    fn get_acc_lo(&self) -> u8 { self.acc as u8 }
    fn set_acc_hi(&mut self, val: u8) {
        self.acc = ((val as u16) << 8) | (self.acc & 0x00FF);
    }
    fn set_acc_lo(&mut self, val: u8) {
        self.acc = (self.acc & 0xFF00) | val as u16;
    }

    fn get_x_hi(&self) -> u8 { (self.x >> 8) as u8 }
    fn get_x_lo(&self) -> u8 { self.x as u8 }
    fn set_x_hi(&mut self, val: u8) {
        self.x = ((val as u16) << 8) | (self.x & 0x00FF);
    }
    fn set_x_lo(&mut self, val: u8) {
        self.x = (self.x & 0xFF00) | val as u16;
    }

    fn get_y_hi(&self) -> u8 { (self.y >> 8) as u8 }
    fn get_y_lo(&self) -> u8 { self.y as u8 }
    fn set_y_hi(&mut self, val: u8) {
        self.y = ((val as u16) << 8) | (self.y & 0x00FF);
    }
    fn set_y_lo(&mut self, val: u8) {
        self.y = (self.y & 0xFF00) | val as u16;
    }

}


// Helper functions
macro_rules! bool2byte {
	($val:expr) => {
		if $val { 1 } else { 0 }
	};
}

// Computes lhs + rhs + carry and outputs a new BCD digit. Alters the carry variable with the new carry value.
fn bcd_add_digit(lhs: u8, rhs: u8, carry: &mut bool) -> u8 {
	let mut result = lhs + rhs + bool2byte!(*carry);
    *carry = false;

	// If the resulting digit is 10-15, make it wrap back around starting at 0
	if result >= 10 {
		result -= 10;
		*carry = true;
	}

    result
}

// Computes lhs - rhs - borrow and outputs a new BCD digit. Alters the borrow variable with the new borrow value.
fn bcd_sub_digit(lhs: u8, rhs: u8, borrow: &mut bool) -> u8 {
    let mut rhs = rhs;
    let mut lhs = lhs;
    
    rhs += bool2byte!(*borrow);
    *borrow = false;

	// If result of subtraction would be negative, make it wrap around starting at 9
	if rhs > lhs {
		lhs += 10;
		*borrow = true;
	}

	lhs - rhs
}


// INSTRUCTIONS

impl Cpu65c816 {
    fn adc_m8(&mut self, address: u32) {
        let data = self.read8(address);
        let result: u8;

        // Decimal Mode
        if self.is_flag_set(Flag::FlagD) {
            // One's place, ten's place
            let o_place: u8;
            let t_place: u8;

            let mut carry = self.is_flag_set(Flag::FlagC);

            o_place = bcd_add_digit(self.get_acc_lo(), data&0x0F, &mut carry);
            t_place = bcd_add_digit((self.get_acc_lo() >> 4)&0x0F, (data >> 4)&0x0F, &mut carry);

            result = (t_place << 4) | o_place;

            self.set_flag_to_bool(Flag::FlagC, carry);
        } else {
            result = self.get_acc_lo() + data + bool2byte!(self.is_flag_set(Flag::FlagC));

            self.set_flag_to_bool(Flag::FlagC, result < self.get_acc_lo());
        }

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagV, (!(self.get_acc_lo() ^ data))&(data ^ result)&0x80 != 0);

        self.acc = result as u16;
    }

    fn adc_m16(&mut self, address_lo: u32, address_hi: u32) {
        let data = self.read16(address_lo, address_hi);
        let result: u16;

        // Decimal Mode
        if self.is_flag_set(Flag::FlagD) {
            // One's place, ten's place
            let o_place: u16;
            let t_place: u16;
            let h_place: u16;
            let th_place: u16;

            let mut carry = self.is_flag_set(Flag::FlagC);

            o_place = bcd_add_digit(self.get_acc_lo(), (data&0x0F) as u8, &mut carry) as u16;
            t_place = bcd_add_digit((self.get_acc_lo() >> 4)&0x0F, ((data >> 4)&0x0F) as u8, &mut carry) as u16;
            h_place = bcd_add_digit(self.get_acc_hi()&0x0F, ((data >> 8)&0x0F) as u8, &mut carry) as u16;
            th_place = bcd_add_digit((self.get_acc_hi() >> 4)&0x0F, ((data >> 12)&0x0F) as u8, &mut carry) as u16;

            result = (th_place << 12) | (h_place << 8) | (t_place << 4) | o_place;

            self.set_flag_to_bool(Flag::FlagC, carry);
        } else {
            result = self.acc + data + bool2byte!(self.is_flag_set(Flag::FlagC));

            self.set_flag_to_bool(Flag::FlagC, result < self.acc);
        }

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagV, (!(self.acc ^ data))&(data ^ result)&0x8000 != 0);

        self.acc = result;
    }

    fn and_m8(&mut self, address: u32) {
        let result = self.get_acc_lo() & self.read8(address);
        
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        
        self.set_acc_lo(result);
    }

    fn and_m16(&mut self, address_lo: u32, address_hi: u32) {
        let result = self.acc & self.read16(address_lo, address_hi);
        
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        
        self.acc = result;
    }

    fn asl_acc_m8(&mut self, address: u32) {
        self.set_flag_to_bool(Flag::FlagC, self.get_acc_lo() & 0x80 != 0);
        
        self.set_acc_lo(self.get_acc_lo() << 1);

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }

    fn asl_acc_m16(&mut self, address_lo: u32, address_hi: u32) {
        self.set_flag_to_bool(Flag::FlagC, self.acc & 0x8000 != 0);

        self.acc <<= 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn asl_mem_m8(&mut self, address: u32) {
        let data = self.read8(address);
        let result = data << 1;
        
        self.set_flag_to_bool(Flag::FlagC, data & 0x80 != 0);
        
        self.write8(address, result);
        
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn asl_mem_m16(&mut self, address_lo: u32, address_hi: u32) {
        let data = self.read16(address_lo, address_hi);
        let result = data << 1;
        
        self.set_flag_to_bool(Flag::FlagC, data & 0x80 != 0);
        
        self.write16(address_lo, address_hi, result);
        
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn bcc_all(&mut self, address: u32) {
        if !self.is_flag_set(Flag::FlagC) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bcs_all(&mut self, address: u32) {
        if self.is_flag_set(Flag::FlagC) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn beq_all(&mut self, address: u32) {
        if self.is_flag_set(Flag::FlagZ) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bit_m8(&mut self, address: u32) {
        let result = self.get_acc_lo() & self.read8(address);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
}