use serde::{ser::SerializeStruct, Serialize};

#[derive(Default, Clone, Copy, PartialEq)]
enum CpuMode {
    #[default]
    Emulation,
    Native
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
    direct_page: u16,
    data_bank: u8,
    prg_bank: u8,
    status: u8,

    mode: CpuMode,
    branch_taken: bool,
    stopped: bool,
    awaiting_interrupt: bool,

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
                s.serialize_field("direct", &self.direct_page);
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

    fn pop8_n(&mut self) -> u8 {
        self.stk_ptr += 1;
        self.read8(self.stk_ptr as u32)
    }
    fn pop16_n(&mut self) -> u16 {
        u16::from_le_bytes([
            self.pop8_n(),
            self.pop8_n()
        ])
    }
    fn pop8_e(&mut self) -> u8 {
        self.stk_ptr = inc_low_byte(self.stk_ptr);
        self.read8(self.stk_ptr as u32)
    }
    fn pop16_e(&mut self) -> u16 {
        u16::from_le_bytes([
            self.pop8_e(),
            self.pop8_e()
        ])
    }
    
    fn push8_n(&mut self, data: u8) {
        self.write8(self.stk_ptr as u32, data);
        self.stk_ptr -= 1;
    }
    fn push16_n(&mut self, data: u16) {
        self.push8_n((data >> 8) as u8);
        self.push8_n(data as u8);
    }
    fn push8_e(&mut self, data: u8) {
        self.write8(self.stk_ptr as u32, data);
        self.stk_ptr = dec_low_byte(self.stk_ptr);
    }
    fn push16_e(&mut self, data: u16) {
        self.push8_e((data >> 8) as u8);
        self.push8_e(data as u8);
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
fn inc_low_byte(value: u16) -> u16 {
    (value & 0xFF00) | ((value + 1) & 0x00FF)
}
fn dec_low_byte(value: u16) -> u16 {
    (value & 0xFF00) | ((value - 1) & 0x00FF)
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

    fn bit_m16(&mut self, address_lo: u32, address_hi: u32) {
        let result = self.acc & self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn bmi_all(&mut self, address: u32) {
        if self.is_flag_set(Flag::FlagN) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bpl_all(&mut self, address: u32) {
        if !self.is_flag_set(Flag::FlagN) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bra_all(&mut self, address: u32) {
        self.pc = address.bank_addr();
        self.branch_taken = true;
    }

    fn brk_n(&mut self) {
        self.push8_n(self.prg_bank);
        self.push16_n(self.pc + 1); // push the address of the brk instruction + 2 (1 has already been added to pc)
        self.push8_n(self.status);
        self.set_flag(Flag::FlagI);

        const N_BRK_VECTOR_LO: u32 = 0x00FFE6;
        const N_BRK_VECTOR_HI: u32 = 0x00FFE7;

        self.pc = self.read16(N_BRK_VECTOR_LO, N_BRK_VECTOR_HI);
    }
    fn brk_e(&mut self) {
        self.push16_n(self.pc + 1); // push the address of the brk instruction + 2 (1 has already been added to pc)
        self.push8_n(self.status);
        self.set_flag(Flag::FlagI);

        const E_BRK_VECTOR_LO: u32 = 0x00FFFE;
        const E_BRK_VECTOR_HI: u32 = 0x00FFFF;

        self.pc = self.read16(E_BRK_VECTOR_LO, E_BRK_VECTOR_HI);
    }

    fn bvc_all(&mut self, address: u32) {
        if !self.is_flag_set(Flag::FlagV) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn bvs_all(&mut self, address: u32) {
        if !self.is_flag_set(Flag::FlagV) {
            self.pc = address.bank_addr();
            self.branch_taken = true;
        }
    }

    fn clc_all(&mut self) { self.clear_flag(Flag::FlagC); }

    fn cld_all(&mut self) { self.clear_flag(Flag::FlagD); }

    fn cli_all(&mut self) { self.clear_flag(Flag::FlagI); }

    fn clv_all(&mut self) { self.clear_flag(Flag::FlagV); }

    fn cmp_m8(&mut self, address: u32) {
        let data = self.read8(address);
        let result = self.get_acc_lo() - data;

        self.set_flag_to_bool(Flag::FlagC, self.get_acc_lo() >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn cmp_m16(&mut self, address_lo: u32, address_hi: u32) {
        let data = self.read16(address_lo, address_hi);
        let result = self.acc - data;

        self.set_flag_to_bool(Flag::FlagC, self.acc >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn cop_n(&mut self, address: u32) {
        let _ = self.read8(address); // read is discarded here

        self.push8_n(self.prg_bank);
        self.push16_n(self.pc); // push the address of the COP instruction + 2 (2 has already been added to pc)
        self.push8_n(self.status);
        self.set_flag(Flag::FlagI);

        const N_COP_VECTOR_LO: u32 = 0x00FFE4;
        const N_COP_VECTOR_HI: u32 = 0x00FFE5;

        self.pc = self.read16(N_COP_VECTOR_LO, N_COP_VECTOR_HI);
    }
    fn cop_e(&mut self, address: u32) {
        let _ = self.read8(address); // read is discarded here

        self.push16_n(self.pc); // push the address of the COP instruction + 2 (2 has already been added to pc)
        self.push8_n(self.status);
        self.set_flag(Flag::FlagI);

        const E_COP_VECTOR_LO: u32 = 0x00FFF4;
        const E_COP_VECTOR_HI: u32 = 0x00FFF5;

        self.pc = self.read16(E_COP_VECTOR_LO, E_COP_VECTOR_HI);
    }

    fn cpx_x8(&mut self, address: u32) {
        let data = self.read8(address);
        let result = self.get_x_lo() - data;

        self.set_flag_to_bool(Flag::FlagC, self.get_x_lo() >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn cpx_x16(&mut self, address_lo: u32, address_hi: u32) {
        let data = self.read16(address_lo, address_hi);
        let result = self.x - data;

        self.set_flag_to_bool(Flag::FlagC, self.x >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn cpy_x8(&mut self, address: u32) {
        let data = self.read8(address);
        let result = self.get_y_lo() - data;

        self.set_flag_to_bool(Flag::FlagC, self.get_y_lo() >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn cpy_x16(&mut self, address_lo: u32, address_hi: u32) {
        let data = self.read16(address_lo, address_hi);
        let result = self.y - data;

        self.set_flag_to_bool(Flag::FlagC, self.y >= data);
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn dec_acc_m8(&mut self) {
        self.acc = dec_low_byte(self.acc);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn dec_acc_m16(&mut self) {
        self.acc -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn dec_mem_m8(&mut self, address: u32) {
        let result = self.read8(address) - 1;

        self.write8(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn dec_mem_m16(&mut self, address_lo: u32, address_hi: u32) {
        let result = self.read16(address_lo, address_hi) - 1;

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn dex_x8(&mut self) {
        self.x = dec_low_byte(self.x);

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn dex_x16(&mut self) {
        self.x -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn dey_x8(&mut self) {
        self.y = dec_low_byte(self.y);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn dey_x16(&mut self) {
        self.y -= 1;

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn eor_m8(&mut self, address: u32) {
        let result = self.get_acc_lo() ^ self.read8(address);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        
        self.set_acc_lo(result);
    }
    fn eor_m16(&mut self, address_lo: u32, address_hi: u32) {
        let result = self.acc ^ self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        
        self.acc = result;
    }

    fn inc_acc_m8(&mut self) {
        self.acc = inc_low_byte(self.acc);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn inc_acc_m16(&mut self) {
        self.acc += 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn inc_mem_m8(&mut self, address: u32) {
        let result = self.read8(address) + 1;

        self.write8(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn inc_mem_m16(&mut self, address_lo: u32, address_hi: u32) {
        let result = self.read16(address_lo, address_hi) + 1;

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn inx_x8(&mut self) {
        self.x = inc_low_byte(self.x);
        
        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn inx_x16(&mut self) {
        self.x += 1;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);    
    }

    fn iny_x8(&mut self) {
        self.y = inc_low_byte(self.y);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn iny_x16(&mut self) {
        self.y += 1;

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn jmp_all(&mut self, address: u32) {
        self.pc = address.bank_addr();
        self.branch_taken = true;
    }

    fn jsr_n(&mut self, address: u32) {
        self.push16_n(self.pc - 1); // push the address of the brk instruction + 2 (3 has already been added to pc)
        self.pc = address.bank_addr();
        self.branch_taken = true;
    }
    fn jsr_e(&mut self, address: u32) {
        self.push16_e(self.pc - 1); // push the address of the brk instruction + 2 (3 has already been added to pc)
        self.pc = address.bank_addr();
        self.branch_taken = true;
    }

    fn jsl_n(&mut self, address: u32) {
        self.push8_n(self.prg_bank);
        self.push16_n(self.pc - 1); // push the address of the JSL instruction + 3 (4 has already been added to pc)

        self.pc = address.bank_addr();
        self.prg_bank = address.bank();

        self.branch_taken = true;
    }
    fn jsl_e(&mut self, address: u32) {
        self.push8_e(self.prg_bank);
        self.push16_e(self.pc - 1); // push the address of the JSL instruction + 3 (4 has already been added to pc)

        self.pc = address.bank_addr();
        self.prg_bank = address.bank();

        self.branch_taken = true;
    }

    fn lda_m8(&mut self, address: u32) {
        self.set_acc_lo(self.read8(address));

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn lda_m16(&mut self, address_lo: u32, address_hi: u32) {
        self.acc = self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn ldx_x8(&mut self, address: u32) {
        self.set_x_lo(self.read8(address));

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn ldx_x16(&mut self, address_lo: u32, address_hi: u32) {
        self.x = self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn ldy_x8(&mut self, address: u32) {
        self.set_y_lo(self.read8(address));

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn ldy_x16(&mut self, address_lo: u32, address_hi: u32) {
        self.y = self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn lsr_acc_m8(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc & 1 != 0);
        self.clear_flag(Flag::FlagN); // 0 shifted into high bit, result always positive

        self.acc >>= 1;

        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn lsr_acc_m16(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc & 1 != 0);
        self.clear_flag(Flag::FlagN); // 0 shifted into high bit, result always positive

        self.acc >>= 1;

        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn lsr_mem_m8(&mut self, address: u32) {
        let data = self.read8(address);
        let result = data >> 1;

        self.set_flag_to_bool(Flag::FlagC, data & 1 != 0);
        self.clear_flag(Flag::FlagN); // 0 shifted into high bit, result always positive

        self.write8(address, result);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn lsr_mem_m16(&mut self, address_lo: u32, address_hi: u32) {
        let data = self.read16(address_lo, address_hi);
        let result = data >> 1;

        self.set_flag_to_bool(Flag::FlagC, data & 1 != 0);
        self.clear_flag(Flag::FlagN); // 0 shifted into high bit, result always positive

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn mvn_all(&mut self, src_address: u32, dest_address: u32) {
        // Idx registers incremented in block move negative (it's backwards, I know)
        // "Negative" actually refers to where the destination address is relative
        // to the source address.
        self.x += 1;
        self.y += 1;

        self.write8(dest_address, self.read8(src_address));

        self.acc -= 1;

        // This instruction essensially jumps to itself until it has moved self.acc + 1
        // bytes. self.acc will be 0xFFFF after this instruction. The addressing mode
        // of this instruction is always BlockMove, so the instruction is always 3 bytes.
        if self.acc != 0xFFFF {
            self.pc -= 3;
        } else {
            // Finished moving data
            self.data_bank = dest_address.bank(); // overself.write8s the dataBank register with the destination bank when finished
        }
    }

    fn mvp_all(&mut self, src_address: u32, dest_address: u32) {
        // Idx registers decremented in block move positive (it's backwards, I know)
        // "Positive" actually refers to where the destination address is relative
        // to the source address.
        self.x -= 1;
        self.y -= 1;

        self.write8(dest_address, self.read8(src_address));

        self.acc -= 1;

        // This instruction essensially jumps to itself until it has moved self.acc + 1
        // bytes. self.acc will be 0xFFFF after this instruction. The addressing mode
        // of this instruction is always BlockMove, so the instruction is always 3 bytes.
        if self.acc != 0xFFFF {
            self.pc -= 3;
        } else {
            // Finished moving data
            self.data_bank = dest_address.bank(); // overself.write8s the dataBank register with the destination bank when finished
        }
    }

    fn nop_all(&mut self) {}

    fn ora_m8(&mut self, address: u32) {
        let result = self.get_acc_lo() | self.read8(address);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        
        self.set_acc_lo(result);
    }
    fn ora_m16(&mut self, address_lo: u32, address_hi: u32) {
        let result = self.acc | self.read16(address_lo, address_hi);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        
        self.acc = result;
    }

    fn pex_n(&mut self, address: u32) {
        self.push16_n(address.bank_addr());
    }
    fn pex_e(&mut self, address: u32) {
        self.push16_e(address.bank_addr());
    }

    fn pha_m8(&mut self) {
        self.push8_n(self.get_acc_lo());
    }
    fn pha_m16(&mut self) {
        self.push16_n(self.acc);
    }
    fn pha_e(&mut self) {
        self.push8_e(self.get_acc_lo());
    }

    fn phb_n(&mut self) {
        self.push8_n(self.data_bank);
    }
    fn phb_e(&mut self) {
        self.push8_e(self.data_bank);
    }

    fn phd_n(&mut self) {
        self.push16_n(self.direct_page);
    }
    fn phd_e(&mut self) {
        self.push16_e(self.direct_page);
    }

    fn phk_n(&mut self) {
        self.push8_n(self.prg_bank);
    }
    fn phk_e(&mut self) {
        self.push8_e(self.prg_bank);
    }

    fn php_n(&mut self) {
        self.push8_n(self.status);
    }
    fn php_e(&mut self) {
        self.push8_e(self.status);
    }

    fn phx_x8(&mut self) {
        self.push8_n(self.get_x_lo());
    }
    fn phx_x16(&mut self) {
        self.push16_n(self.x);
    }
    fn phx_e(&mut self) {
        self.push8_e(self.get_x_lo());
    }

    fn phy_x8(&mut self) {
        self.push8_n(self.get_y_lo());
    }
    fn phy_x16(&mut self) {
        self.push16_n(self.y);
    }
    fn phy_e(&mut self) {
        self.push8_e(self.get_y_lo());
    }

    fn pla_m8(&mut self) {
        let data = self.pop8_n();
        self.set_acc_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn pla_m16(&mut self) {
        self.acc = self.pop16_n();

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn pla_e(&mut self) {
        let data = self.pop8_e();
        self.set_acc_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn plb_n(&mut self) {
        self.data_bank = self.pop8_n();

        self.set_flag_to_bool(Flag::FlagN, self.data_bank & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.data_bank == 0);
    }
    fn plb_e(&mut self) {
        self.data_bank = self.pop8_e();

        self.set_flag_to_bool(Flag::FlagN, self.data_bank & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.data_bank == 0);
    }

    fn pld_n(&mut self) {
        self.direct_page = self.pop16_n();

        self.set_flag_to_bool(Flag::FlagN, self.direct_page & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.direct_page == 0);
    }
    fn pld_e(&mut self) {
        self.direct_page = self.pop16_e();

        self.set_flag_to_bool(Flag::FlagN, self.direct_page & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.direct_page == 0);
    }

    fn plp_n(&mut self) {
        self.status = self.pop8_n();
    }
    fn plp_e(&mut self) {
        self.status = self.pop8_e();
        // Emulation mode forces these flags on
        self.set_flag(Flag::FlagM);
        self.set_flag(Flag::FlagX);
    }

    fn plx_x8(&mut self) {
        let data = self.pop8_n();
        self.set_x_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn plx_x16(&mut self) {
        self.x = self.pop16_n();

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn plx_e(&mut self) {
        let data = self.pop8_e();
        self.set_x_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn ply_x8(&mut self) {
        let data = self.pop8_n();
        self.set_y_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn ply_x16(&mut self) {
        self.y = self.pop16_n();

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }
    fn ply_e(&mut self) {
        let data = self.pop8_e();
        self.set_y_lo(data);

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn rep_n(&mut self, address: u32) {
        self.status &= !self.read8(address);
    }
    fn rep_e(&mut self, address: u32) {
        self.status &= !self.read8(address);
        self.set_flag(Flag::FlagM);
        self.set_flag(Flag::FlagX);
    }

    fn rol_acc_m8(&mut self) {
        let c = self.is_flag_set(Flag::FlagC);
        self.set_flag_to_bool(Flag::FlagC, self.acc & 0x80 != 0);

        self.set_acc_lo(self.get_acc_lo() << 1);
        self.acc |= bool2byte!(c);

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn rol_acc_m16(&mut self) {
        let c = self.is_flag_set(Flag::FlagC);
        self.set_flag_to_bool(Flag::FlagC, self.acc & 0x8000 != 0);

        self.acc <<= 1;
        self.acc |= bool2byte!(c);

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn rol_mem_m8(&mut self, address: u32) {
        let c = self.is_flag_set(Flag::FlagC);
        let data = self.read8(address);
        let result = (data << 1) | bool2byte!(c);

        self.set_flag_to_bool(Flag::FlagC, data & 0x80 != 0);

        self.write8(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn rol_mem_m16(&mut self, address_lo: u32, address_hi: u32) {
        let c = self.is_flag_set(Flag::FlagC);
        let data = self.read16(address_lo, address_hi);
        let result = (data << 1) | bool2byte!(c);

        self.set_flag_to_bool(Flag::FlagC, data & 0x8000 != 0);

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn ror_acc_m8(&mut self) {
        let c = self.is_flag_set(Flag::FlagC);
        self.set_flag_to_bool(Flag::FlagC, self.acc & 1 != 0);

        self.acc >>= 1;
        self.acc |= bool2byte!(c) << 7;

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn ror_acc_m16(&mut self) {
        let c = self.is_flag_set(Flag::FlagC);
        self.set_flag_to_bool(Flag::FlagC, self.acc & 1 != 0);

        self.acc >>= 1;
        self.acc |= bool2byte!(c) << 15;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn ror_mem_m8(&mut self, address: u32) {
        let c = self.is_flag_set(Flag::FlagC);

        let data = self.read8(address);
        let result = (data >> 1) | (bool2byte!(c) << 7);

        self.set_flag_to_bool(Flag::FlagC, data & 1 != 0);

        self.write8(address, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn ror_mem_m16(&mut self, address_lo: u32, address_hi: u32) {
        let c = self.is_flag_set(Flag::FlagC);

        let data = self.read16(address_lo, address_hi);
        let result = (data >> 1) | (bool2byte!(c) << 15);

        self.set_flag_to_bool(Flag::FlagC, data & 1 != 0);

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn rti_n(&mut self) {
        self.status = self.pop8_n();
        self.pc = self.pop16_n();
        self.prg_bank = self.pop8_n();
    }
    fn rti_e(&mut self) {
        self.status = self.pop8_e();
        self.set_flag(Flag::FlagM);
        self.set_flag(Flag::FlagX);
        self.pc = self.pop16_e();
    }

    fn rtl_n(&mut self) {
        self.pc = self.pop16_n() + 1;
        self.prg_bank = self.pop8_n();
    }
    fn rtl_e(&mut self) {
        self.pc = self.pop16_e() + 1;
        self.prg_bank = self.pop8_e();
    }

    fn rts_n(&mut self) {
        self.pc = self.pop16_n() + 1;
    }
    fn rts_e(&mut self) {
        self.pc = self.pop16_e() + 1;
    }

    fn sbc_m8(&mut self, address: u32) {
        let data = self.read8(address);
        let result;

        if self.is_flag_set(Flag::FlagD) {
            // One's place, ten's place
            let o_place: u8;
            let t_place: u8;
            let mut borrow = !self.is_flag_set(Flag::FlagC);

            o_place = bcd_sub_digit(self.get_acc_lo()&0x0F, data&0x0F, &mut borrow);
            t_place = bcd_sub_digit((self.get_acc_lo() >> 4)&0x0F, (data >> 4)&0x0F, &mut borrow);

            result = (t_place << 4) | o_place;

            self.set_flag_to_bool(Flag::FlagC, borrow);
        } else {
            result = self.get_acc_lo() - data - bool2byte!(!self.is_flag_set(Flag::FlagC));

            self.set_flag_to_bool(Flag::FlagC, result > self.get_acc_lo());
        }

        self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagV, (!(self.get_acc_lo() ^ data))&(data ^ result)&0x80 != 0);

        self.set_acc_lo(result);
    }
    fn sbc_m16(&mut self, address_lo: u32, address_hi: u32) {
        let data = self.read16(address_lo, address_hi);
        let result;

        if self.is_flag_set(Flag::FlagD) {
            // One's place, ten's place, hundred's place, thousand's place
            let o_place: u16;
            let t_place: u16;
            let h_place: u16;
            let th_place: u16;
            let mut borrow = !self.is_flag_set(Flag::FlagC);

            o_place = bcd_sub_digit(self.get_acc_lo()&0x0F, (data&0x0F) as u8, &mut borrow) as u16;
            t_place = bcd_sub_digit((self.get_acc_lo() >> 4)&0x0F, ((data >> 4)&0x0F) as u8, &mut borrow) as u16;
            h_place = bcd_sub_digit(self.get_acc_hi()&0x0F, ((data >> 8)&0x0F) as u8, &mut borrow) as u16;
            th_place = bcd_sub_digit((self.get_acc_hi() >> 4)&0x0F, ((data >> 12)&0x0F) as u8, &mut borrow) as u16;

            result = (th_place << 12) | (h_place << 8) | (t_place << 4) | o_place;

            self.set_flag_to_bool(Flag::FlagC, borrow);
        } else {
            result = self.acc - data - bool2byte!(!self.is_flag_set(Flag::FlagC));

            self.set_flag_to_bool(Flag::FlagC, result > self.acc);
        }

        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagV, (!(self.acc ^ data))&(data ^ result)&0x8000 != 0);

        self.acc = result;
    }

    fn sec_all(&mut self) { self.set_flag(Flag::FlagC); }

    fn sed_all(&mut self) { self.set_flag(Flag::FlagD); }

    fn sei_all(&mut self) { self.set_flag(Flag::FlagI); }

    fn sep_all(&mut self, address: u32) {
        self.status |= self.read8(address);
    }

    fn sta_m8(&mut self, address: u32) {
        self.write8(address, self.get_acc_lo());
    }
    fn sta_m16(&mut self, address_lo: u32, address_hi: u32) {
        self.write16(address_lo, address_hi, self.acc)
    }

    fn stp_all(&mut self) { self.stopped = true; }

    fn stx_x8(&mut self, address: u32) {
        self.write8(address, self.get_x_lo());
    }
    fn stx_x16(&mut self, address_lo: u32, address_hi: u32) {
        self.write16(address_lo, address_hi, self.x)
    }

    fn sty_x8(&mut self, address: u32) {
        self.write8(address, self.get_y_lo());
    }
    fn sty_x16(&mut self, address_lo: u32, address_hi: u32) {
        self.write16(address_lo, address_hi, self.y)
    }

    fn stz_m8(&mut self, address: u32) {
        self.write8(address, 0);
    }
    fn stz_m16(&mut self, address_lo: u32, address_hi: u32) {
        self.write16(address_lo, address_hi, 0)
    }

    fn tax_x8(&mut self) {
        self.set_x_lo(self.get_acc_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_x_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_x_lo() == 0);
    }
    fn tax_x16(&mut self) {
        self.x = self.acc;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn tay_x8(&mut self) {
        self.set_y_lo(self.get_acc_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_y_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_y_lo() == 0);
    }
    fn tay_x16(&mut self) {
        self.y = self.acc;

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn tcd_all(&mut self) {
        self.direct_page = self.acc;

        self.set_flag_to_bool(Flag::FlagN, self.direct_page & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.direct_page == 0);
    }

    fn tcs_n(&mut self) { self.stk_ptr = self.acc; }
    fn tcs_e(&mut self) { self.stk_ptr = 0x100 | (self.acc & 0xFF); }

    fn tdc_all(&mut self) {
        self.acc = self.direct_page;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn trb_m8(&mut self, address: u32) {
        let result = self.read8(address) & (!self.get_acc_lo());

        self.write8(address, result);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn trb_m16(&mut self, address_lo: u32, address_hi: u32) {
        let result = self.read16(address_lo, address_hi) & (!self.acc);

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tsb_m8(&mut self, address: u32) {
        let result = self.read8(address) | self.get_acc_lo();

        self.write8(address, result);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    fn tsb_m16(&mut self, address_lo: u32, address_hi: u32) {
        let result = self.read16(address_lo, address_hi) | self.acc;

        self.write16(address_lo, address_hi, result);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tsc_m8(&mut self) {
        self.acc = self.stk_ptr & 0xFF; // 8-bit mode forces acc hi to 0

        self.clear_flag(Flag::FlagN); // the value transfered is always positive
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn tsc_m16(&mut self) {
        self.acc = self.stk_ptr;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
    fn tsc_e(&mut self) {
        self.acc = self.stk_ptr & 0xFF; // Emulation mode forces self.get_acc_hi() to 0

        self.clear_flag(Flag::FlagN); // the value transfered is always positive
        self.clear_flag(Flag::FlagZ); // the value transfered is always non-zero
    }

    fn tsx_x8(&mut self) {
        self.x = self.stk_ptr & 0xFF; // 8-bit mode forces self.xHi to 0

        self.clear_flag(Flag::FlagN); // the value transfered is always positive
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn tsx_x16(&mut self) {
        self.x = self.stk_ptr;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }
    fn tsx_e(&mut self) {
        self.x = self.stk_ptr & 0xFF; // Emulation mode forces self.xHi to 0

        self.clear_flag(Flag::FlagN); // the value transfered is always positive
        self.clear_flag(Flag::FlagZ); // the value transfered is always non-zero
    }

    fn txa_m8(&mut self) {
        self.set_acc_lo(self.get_x_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn txa_m16(&mut self) {
        self.acc = self.x;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn txs_n(&mut self) { self.stk_ptr = self.x; }
    fn txs_e(&mut self) { self.stk_ptr = 0x100 & (self.x & 0xFF); }

    fn txy_x8(&mut self) {
        self.set_y_lo(self.get_x_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_y_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_y_lo() == 0);
    }
    fn txy_x16(&mut self) {
        self.y = self.x;

        self.set_flag_to_bool(Flag::FlagN, self.y & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn tya_m8(&mut self) {
        self.set_acc_lo(self.get_y_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_acc_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_acc_lo() == 0);
    }
    fn tya_m16(&mut self) {
        self.acc = self.y;

        self.set_flag_to_bool(Flag::FlagN, self.acc & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }

    fn tyx_x8(&mut self) {
        self.set_x_lo(self.get_y_lo());

        self.set_flag_to_bool(Flag::FlagN, self.get_x_lo() & 0x80 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.get_x_lo() == 0);
    }
    fn tyx_x16(&mut self) {
        self.x = self.y;

        self.set_flag_to_bool(Flag::FlagN, self.x & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn wai_all(&mut self) { self.awaiting_interrupt = true; }

    fn wdm_all(&mut self) {}

    fn xba_m8(&mut self) {
        self.acc = 0; // Has the effect of zeroing the accumulator in 8-bit mode (i think)
    }
    fn xba_m16(&mut self) {
        self.acc = self.acc.swap_bytes();
    }

    fn xce_all(&mut self) {
        if self.is_flag_set(Flag::FlagC) {
            self.set_flag_to_bool(Flag::FlagC, self.mode == CpuMode::Emulation);
            self.mode = CpuMode::Emulation;

            self.set_acc_hi(0);
            self.set_x_hi(0);
            self.set_y_hi(0);
            self.set_flag(Flag::FlagM);
            self.set_flag(Flag::FlagX);
            self.stk_ptr = 0x100 | (self.stk_ptr & 0xFF);
        } else {
            self.set_flag_to_bool(Flag::FlagC, self.mode == CpuMode::Emulation);
            self.mode = CpuMode::Native;
        }
    }
}