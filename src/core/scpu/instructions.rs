use crate::core::scpu::bus::{Address, CpuBus};
use crate::core::scpu::{Cpu65c816, Flag};
use crate::{
    get_byte_n, 
    set_byte_n,
    set_bit_n,
    clr_bit_n,
    get_bit_n,
};

const N_BRK_VECTOR_LO: u16 = 0xFFE6;
const N_BRK_VECTOR_HI: u16 = 0xFFE7;
const E_BRK_VECTOR_LO: u16 = 0xFFFE;
const E_BRK_VECTOR_HI: u16 = 0xFFFF;

const N_COP_VECTOR_LO: u16 = 0xFFE4;
const N_COP_VECTOR_HI: u16 = 0xFFE5;
const E_COP_VECTOR_LO: u16 = 0xFFF4;
const E_COP_VECTOR_HI: u16 = 0xFFF5;

/// Set the N and Z flags based on an 8-bit value.
macro_rules! set_nz8 {
    ($cpu:expr, $val:expr) => {
        $cpu.set_flag_to_bool(Flag::FlagN, get_bit_n!($val, 7));
        $cpu.set_flag_to_bool(Flag::FlagZ, ($val as u8) == 0);
    };
}
/// Set the N and Z flags based on a 16-bit value.
macro_rules! set_nz16 {
    ($cpu:expr, $val:expr) => {
        $cpu.set_flag_to_bool(Flag::FlagN, get_bit_n!($val, 15));
        $cpu.set_flag_to_bool(Flag::FlagZ, $val == 0);
    };
}

macro_rules! cmp_reg8 {
    ($cpu:expr, $reg:expr, $bus:expr, $addr:expr) => {
        let data = $cpu.read($bus, $addr);
        let result = ($reg as u8) - data;

        $cpu.set_flag_to_bool(Flag::FlagC, ($reg as u8) >= data);
        
        set_nz8!($cpu, result);
    };
}

macro_rules! cmp_reg16 {
    ($cpu:expr, $reg:expr, $bus:expr, $addr_lo:expr, $addr_hi:expr) => {
        let data = $cpu.read_word($bus, $addr_lo, $addr_hi) as u16;
        let result = $reg - data;

        $cpu.set_flag_to_bool(Flag::FlagC, $reg >= data);
        
        set_nz16!($cpu, result);
    };
}

macro_rules! dec_idx {
    ($cpu:expr, $reg:expr) => {
        if $cpu.is_flag_set(Flag::FlagX) {
            $reg -= 1;
            $reg &= 0xFF;
            set_nz8!($cpu, $reg);
        } else {
            $reg -= 1;
            set_nz16!($cpu, $reg);
        }
    };
}

macro_rules! inc_idx {
    ($cpu:expr, $reg:expr) => {
        if $cpu.is_flag_set(Flag::FlagX) {
            $reg += 1;
            $reg &= 0xFF;
            set_nz8!($cpu, $reg);
        } else {
            $reg += 1;
            set_nz16!($cpu, $reg);
        }
    };
}

macro_rules! transfer_reg {
    ($cpu:expr, $src:expr, $dst:expr, $flag:expr) => {
        if $cpu.is_flag_set($flag) {
            set_byte_n!($dst, $src & 0xFF, 0);
            set_nz8!($cpu, $dst);
        } else {
            $dst = $src;
            set_nz16!($cpu, $dst);
        }
    };
}

// Addressing Modes
impl Cpu65c816 {
    pub fn immediate(&mut self) -> Address {
        let offset = self.pc;
        self.pc += 1;        
        
        Address {
            bank: self.pb,
            offset,
        }
    }

    pub fn absolute(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let hi = self.read_prg(bus);
        
        Address {
            bank: self.db,
            offset: u16::from_le_bytes([lo, hi]),
        }
    }

    pub fn absolute_x(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let hi = self.read_prg(bus);
        
        Address::from_u32(
            u32::from_le_bytes([lo, hi, self.db, 0]) + self.x as u32
        )
    }

    pub fn absolute_y(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let hi = self.read_prg(bus);
        
        Address::from_u32(
            u32::from_le_bytes([lo, hi, self.db, 0]) + self.y as u32
        )
    }

    pub fn absolute_indirect(&mut self, bus: &mut CpuBus) -> Address {
        let ll = self.read_prg(bus);
        let hh = self.read_prg(bus);

        let ptr_lo = u16::from_le_bytes([ll, hh]);
        let ptr_hi = ptr_lo + 1;

        let lo = self.read(bus, Address { bank: 0, offset: ptr_lo });
        let hi = self.read(bus, Address { bank: 0, offset: ptr_hi });

        Address {
            bank: self.pb,
            offset: u16::from_le_bytes([lo, hi]),
        }
    }
    
    pub fn absolute_x_indirect(&mut self, bus: &mut CpuBus) -> Address {
        let ll = self.read_prg(bus);
        let hh = self.read_prg(bus);

        let ptr_lo = u16::from_le_bytes([ll, hh]);
        let ptr_hi = ptr_lo + 1;

        let lo = self.read(bus, Address { bank: self.db, offset: ptr_lo });
        let hi = self.read(bus, Address { bank: self.db, offset: ptr_hi });

        Address {
            bank: self.pb,
            offset: u16::from_le_bytes([lo, hi]),
        }
    }
        
    pub fn direct(&mut self, bus: &mut CpuBus) -> Address {
        Address {
            bank: 0,
            offset: self.dp + self.read_prg(bus) as u16,
        }
    }
        
    pub fn direct_x(&mut self, bus: &mut CpuBus) -> Address {
        let data = self.read_prg(bus);
        
        let offset = if self.e && (self.dp & 0xFF) == 0 {
            self.dp | ((data + self.x as u8) as u16)
        } else {
            self.dp + (data as u16) + self.x
        };
        
        Address {
            bank: 0,
            offset,
        }
    }
        
    pub fn direct_y(&mut self, bus: &mut CpuBus) -> Address {
        let data = self.read_prg(bus);
        
        let offset = if self.e && (self.dp & 0xFF) == 0 {
            self.dp | ((data + self.y as u8) as u16)
        } else {
            self.dp + (data as u16) + self.y
        };
        
        Address {
            bank: 0,
            offset,
        }
    }
        
    pub fn direct_indirect(&mut self, bus: &mut CpuBus) -> Address {
        let data = self.read_prg(bus);

        let ptr_lo = self.dp + data as u16;
        let ptr_hi = if self.e && (self.dp & 0xFF) == 0 {
            self.dp + (data + 1) as u16
        } else {
            self.dp + (data as u16) + 1
        };

        let lo = self.read(bus, Address { bank: 0, offset: ptr_lo });
        let hi = self.read(bus, Address { bank: 0, offset: ptr_hi });

        Address {
            bank: self.db,
            offset: u16::from_le_bytes([lo, hi]),
        }
    }
        
    pub fn direct_indirect_long(&mut self, bus: &mut CpuBus) -> Address {
        let ll = self.read_prg(bus);
        let hh = self.read_prg(bus);

        let ptr_lo = self.dp + u16::from_le_bytes([ll, hh]);
        let ptr_mi = ptr_lo + 1;
        let ptr_hi = ptr_mi + 1;

        let lo = self.read(bus, Address { bank: 0, offset: ptr_lo });
        let mi = self.read(bus, Address { bank: 0, offset: ptr_mi });
        let hi = self.read(bus, Address { bank: 0, offset: ptr_hi });

        Address {
            bank: hi,
            offset: u16::from_le_bytes([lo, mi]),
        }
    }
        
    pub fn direct_x_indirect(&mut self, bus: &mut CpuBus) -> Address {
        let data = self.read_prg(bus);
        
        let ptr_lo = if self.e && (self.dp & 0xFF) == 0 {
            self.dp | (data + self.x as u8) as u16
        } else {
            self.dp + (data as u16) + self.x
        };
        let ptr_hi = if self.e && (self.dp & 0xFF) == 0 {
            self.dp | (data + self.x as u8 + 1) as u16
        } else {
            self.dp + (data as u16) + self.x + 1
        };

        let lo = self.read(bus, Address { bank: 0, offset: ptr_lo });
        let hi = self.read(bus, Address { bank: 0, offset: ptr_hi });

        Address {
            bank: self.db,
            offset: u16::from_le_bytes([lo, hi]),
        }
    }
        
    pub fn direct_indirect_y(&mut self, bus: &mut CpuBus) -> Address {
        Address::from_u32(
            self.direct_indirect(bus).to_u32() + self.y as u32
        )
    }
        
    pub fn direct_indirect_long_y(&mut self, bus: &mut CpuBus) -> Address {
        Address::from_u32(
            self.direct_indirect_long(bus).to_u32() + self.y as u32
        )
    }
        
    pub fn long(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let mi = self.read_prg(bus);
        let hi = self.read_prg(bus);
        
        Address {
            bank: hi,
            offset: u16::from_le_bytes([lo, mi]),
        }
    }
        
    pub fn long_x(&mut self, bus: &mut CpuBus) -> Address {
        Address::from_u32(
            self.long(bus).to_u32() + self.x as u32
        )
    }
        
    pub fn long_indirect(&mut self, bus: &mut CpuBus) -> Address {
        let ll = self.read_prg(bus);
        let hh = self.read_prg(bus);
        
        let ptr_lo = u16::from_le_bytes([ll, hh]);
        let ptr_mi = ptr_lo + 1;
        let ptr_hi = ptr_mi + 1;

        let lo = self.read(bus, Address { bank: 0, offset: ptr_lo });
        let mi = self.read(bus, Address { bank: 0, offset: ptr_mi });
        let hi = self.read(bus, Address { bank: 0, offset: ptr_hi });

        Address {
            bank: hi,
            offset: u16::from_le_bytes([lo, mi]),
        }
    }
        
    fn relative(&mut self, bus: &mut CpuBus) -> Address {
        let rel_offset = ((self.read_prg(bus) as i8) as i16) as u16; // sign extend u8 to u16
        let offset = self.pc + rel_offset;

        Address {
            bank: self.pb,
            offset,
        }
    }
        
    fn relative_long(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let hi = self.read_prg(bus);
        let rel_offset = u16::from_le_bytes([lo, hi]);
        let offset = self.pc + rel_offset;

        Address {
            bank: self.pb,
            offset,
        }
    }
     
    fn source(&mut self, bus: &mut CpuBus) -> Address {
        Address {
            bank: self.read_prg(bus),
            offset: self.x,
        }
    }
    
    fn destination(&mut self, bus: &mut CpuBus) -> Address {
        Address {
            bank: self.read_prg(bus),
            offset: self.y,
        }
    }
    
    fn stack_relative(&mut self, bus: &mut CpuBus) -> Address {
        Address {
            bank: 0,
            offset: self.sp + self.read_prg(bus) as u16,
        }
    }
    
    fn stack_relative_indirect_y(&mut self, bus: &mut CpuBus) -> Address {
        let ptr_lo = self.sp + self.read_prg(bus) as u16;
        let ptr_hi = ptr_lo + 1;

        let lo = self.read(bus, Address { bank: 0, offset: ptr_lo });
        let hi = self.read(bus, Address { bank: 0, offset: ptr_hi });
        
        Address::from_u32(
            u32::from_le_bytes([lo, hi, self.db, 0]) + self.y as u32
        )
    }
}

// Instructions
impl Cpu65c816 {
    fn adc_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let mut result: u16;
        let data = self.read(bus, addr) as u16;
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        if self.is_flag_set(Flag::FlagD) {
            result = (self.a & 0x0F) + (data & 0x0F) + c;

            if result >= 0xA {
                result += 0x6;
            }

            let c = if result > 0xF { 0x10 } else { 0 };
            result = (self.a & 0xF0) + (data & 0xF0) + c + (result & 0xF);
        } else {
            result = self.a + data + c;
        };

        self.set_flag_to_bool(Flag::FlagV, get_bit_n!(!(self.a ^ data) & (data ^ result), 7));

        if self.is_flag_set(Flag::FlagD) && result >= 0xA0 {
            result += 0x60;
        }

        self.set_flag_to_bool(Flag::FlagC, result > 0xFF);

        set_byte_n!(self.a, result, 0);

        set_nz8!(self, self.a);
    }
    
    fn adc_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let mut result: u32;
        let a = self.a as u32;
        let d = self.read_word(bus, addr_lo, addr_hi) as u32;
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        if self.is_flag_set(Flag::FlagD) {
            result = (a & 0x000F) + (d & 0x000F) + c;

            if result >= 0xA {
                result += 6;
            }

            let c = if result > 0xF { 0x10 } else { 0 };
            result = (a & 0x00F0) + (d & 0x00F0) + c + (result & 0xF);

            if result >= 0xA0 {
                result += 0x60;
            }

            let c = if result > 0xFF { 0x100 } else { 0 };
            result = (a & 0x0F00) + (d & 0x0F00) + c + (result & 0xFF);

            if result >= 0xA00 {
                result += 0x600;
            }

            let c = if result > 0xFFF { 0x1000 } else { 0 };
            result = (a & 0xF000) + (d & 0xF000) + c + (result & 0xFFF);
        } else {
            result = a + d + c;
        }

        self.set_flag_to_bool(Flag::FlagV, get_bit_n!(!(a ^ d) & (d ^ result), 15));

        if self.is_flag_set(Flag::FlagD) && result >= 0xA000 {
            result += 0x6000;
        }

        self.set_flag_to_bool(Flag::FlagC, result > 0xFFFF);

        self.a = result as u16;

        set_nz16!(self, self.a);
    }
    
    fn and_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        set_byte_n!(self.a, self.a & self.read(bus, addr) as u16, 0);
        set_nz8!(self, self.a);
    }
    
    fn and_m16(&mut self, bus: &mut CpuBus, addr1: Address, addr2: Address) {
        self.a &= self.read_word(bus, addr1, addr2);
        set_nz16!(self, self.a);
    }
    
    fn asl_all(&mut self) {
        if self.is_flag_set(Flag::FlagM) {
            self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 7));
            set_byte_n!(self.a, self.a << 1, 0);
            set_nz8!(self, self.a);
        } else {
            self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 15));
            self.a <<= 1;
            set_nz16!(self, self.a);
        }
    }

    fn asl_mem_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let data = self.read(bus, addr);
        let result = data << 1;

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 7));

        self.write(bus, addr, result);

        set_nz8!(self, result);
    }

    fn asl_mem_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        let result = data << 1;

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 15));

        self.write_word(bus, addr_lo, addr_hi, result);

        set_nz16!(self, result);
    }
    
    fn bcc_all(&mut self, addr: Address) {
        if !self.is_flag_set(Flag::FlagC) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bcs_all(&mut self, addr: Address) {
        if self.is_flag_set(Flag::FlagC) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn beq_all(&mut self, addr: Address) {
        if self.is_flag_set(Flag::FlagZ) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bit_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let data = self.read(bus, addr);
        let result = (self.a as u8) & data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(data, 7));
        self.set_flag_to_bool(Flag::FlagV, get_bit_n!(data, 6));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    
    fn bit_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        let result = self.a & data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(data, 15));
        self.set_flag_to_bool(Flag::FlagV, get_bit_n!(data, 14));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    
    fn bit_imm_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let data = self.read(bus, addr);
        let result = (self.a as u8) & data;

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    
    fn bit_imm_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        let result = self.a & data;

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn bmi_all(&mut self, addr: Address) {
        if self.is_flag_set(Flag::FlagN) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bne_all(&mut self, addr: Address) {
        if !self.is_flag_set(Flag::FlagZ) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bpl_all(&mut self, addr: Address) {
        if !self.is_flag_set(Flag::FlagN) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bra_all(&mut self, addr: Address) {
        self.pc = addr.offset;
        self.branch_taken = true;
    }

    fn brk_all(&mut self, bus: &mut CpuBus) {   
        if !self.e {
            self.push(bus, self.pb);
        }
        
        self.push_word(bus, self.pc + 1);
        self.push(bus, self.p);
        self.set_flag_to_bool(Flag::FlagI, true);
        self.set_flag_to_bool(Flag::FlagD, false);

        if self.e {
            self.pc = self.read_word(bus,
                Address { bank: 0, offset: E_BRK_VECTOR_LO },
                Address { bank: 0, offset: E_BRK_VECTOR_HI },
            );
        } else {
            self.pc = self.read_word(bus,
                Address { bank: 0, offset: N_BRK_VECTOR_LO },
                Address { bank: 0, offset: N_BRK_VECTOR_HI },
            );
        }
    }

    fn bvc_all(&mut self, addr: Address) {
        if !self.is_flag_set(Flag::FlagV) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bvs_all(&mut self, addr: Address) {
        if self.is_flag_set(Flag::FlagV) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }
    
    fn clc_all(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, false);
    }

    fn cld_all(&mut self) {
        self.set_flag_to_bool(Flag::FlagD, false);
    }

    fn cli_all(&mut self) {
        self.set_flag_to_bool(Flag::FlagI, false);
    }

    fn clv_all(&mut self) {
        self.set_flag_to_bool(Flag::FlagV, false);
    }

    fn cmp_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        cmp_reg8!(self, self.a, bus, addr);
    }
    
    fn cmp_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        cmp_reg16!(self, self.a, bus, addr_lo, addr_hi);
    }

    fn cop_all(&mut self, bus: &mut CpuBus, addr: Address) {
        let _ = self.read(bus, addr); // read is discarded here

        if !self.e {
            self.push(bus, self.pb);
        }
        
        self.push_word(bus, self.pc);
        self.push(bus, self.p);
        self.set_flag_to_bool(Flag::FlagI, true);
        self.set_flag_to_bool(Flag::FlagD, false);

        if self.e {
            self.pc = self.read_word(bus,
                Address { bank: 0, offset: E_COP_VECTOR_LO },
                Address { bank: 0, offset: E_COP_VECTOR_HI },
            );
        } else {
            self.pc = self.read_word(bus,
                Address { bank: 0, offset: N_COP_VECTOR_LO },
                Address { bank: 0, offset: N_COP_VECTOR_HI },
            );
        }
    }

    fn cpx_x8(&mut self, bus: &mut CpuBus, addr: Address) {
        cmp_reg8!(self, self.x, bus, addr);
    }
    
    fn cpx_x16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        cmp_reg16!(self, self.x, bus, addr_lo, addr_hi);
    }

    fn cpy_x8(&mut self, bus: &mut CpuBus, addr: Address) {
        cmp_reg8!(self, self.y, bus, addr);
    }
    
    fn cpy_x16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        cmp_reg16!(self, self.y, bus, addr_lo, addr_hi);
    }
    
    fn dec_all(&mut self) {
        if self.is_flag_set(Flag::FlagM) {
            set_byte_n!(self.a, self.a - 1, 0);
            set_nz8!(self, self.a);
        } else {
            self.a -= 1;
            set_nz16!(self, self.a);
        }
    }
    
    fn dec_mem_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let result = self.read(bus, addr) - 1;

        self.write(bus, addr, result);

        set_nz8!(self, result);
    }
    
    fn dec_mem_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let result = self.read_word(bus, addr_lo, addr_hi) - 1;

        self.write_word(bus, addr_lo, addr_hi, result);

        set_nz16!(self, result);
    }

    fn dex_all(&mut self) {
        dec_idx!(self, self.x);
    }

    fn dey_all(&mut self) {
        dec_idx!(self, self.y);
    }
    
    fn eor_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.a ^= self.read(bus, addr) as u16;

        set_nz8!(self, self.a);
    }
    
    fn eor_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.a ^= self.read_word(bus, addr_lo, addr_hi);

        set_nz16!(self, self.a);
    }
    
    fn inc_all(&mut self) {
        if self.is_flag_set(Flag::FlagM) {
            set_byte_n!(self.a, self.a + 1, 0);
            set_nz8!(self, self.a);
        } else {
            self.a += 1;
            set_nz16!(self, self.a);
        }
    }

    fn inc_mem_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let result = self.read(bus, addr) + 1;

        self.write(bus, addr, result);

        set_nz8!(self, result);
    }
    
    fn inc_mem_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let result = self.read_word(bus, addr_lo, addr_hi) + 1;

        self.write_word(bus, addr_lo, addr_hi, result);

        set_nz16!(self, result);
    }

    fn inx_all(&mut self) {
        inc_idx!(self, self.x);
    }

    fn iny_all(&mut self) {
        inc_idx!(self, self.y);
    }
    
    fn jmp_all(&mut self, addr: Address) {
        self.pc = addr.offset;
    }

    fn jmp_long_all(&mut self, addr: Address) {
        self.pb = addr.bank;
        self.pc = addr.offset;
    }

    fn jsr_all(&mut self, bus: &mut CpuBus, addr: Address) {
        self.push_word(bus, self.pc - 1);
        self.pc = addr.offset;
    }

    fn jsl_all(&mut self, bus: &mut CpuBus, addr: Address) {
        self.push(bus, self.pb);
        self.push_word(bus, self.pc - 1);

        self.pb = addr.bank;
        self.pc = addr.offset;
    }
    
    fn lda_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        set_byte_n!(self.a, self.read(bus, addr) as u16, 0);
        set_nz8!(self, self.a);
    }
    
    fn lda_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.a = self.read_word(bus, addr_lo, addr_hi);
        set_nz16!(self, self.a);
    }

    fn ldx_x8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.x = self.read(bus, addr) as u16;
        set_nz8!(self, self.x);
    }
    
    fn ldx_x16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.x = self.read_word(bus, addr_lo, addr_hi);
        set_nz16!(self, self.x);
    }

    fn ldy_x8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.y = self.read(bus, addr) as u16;
        set_nz8!(self, self.y);
    }
    
    fn ldy_x16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.y = self.read_word(bus, addr_lo, addr_hi);
        set_nz16!(self, self.y);
    }

    fn lsr_all(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 0));
        self.set_flag_to_bool(Flag::FlagN, false); // 0 shifted into high bit, result always positive
        
        if self.is_flag_set(Flag::FlagM) {
            set_byte_n!(self.a, (self.a >> 1) & 0x7F, 0);
            
            self.set_flag_to_bool(Flag::FlagZ, (self.a & 0xFF) == 0);
        } else {
            self.a >>= 1;
            
            self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
        }
    }
   
    fn lsr_mem_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let data = self.read(bus, addr);
        let result = data >> 1;
        
        self.write(bus, addr, result);

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 0));
        self.set_flag_to_bool(Flag::FlagN, false); // 0 shifted into high bit, result always positive
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    
    fn lsr_mem_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        let result = data >> 1;

        self.write_word(bus, addr_lo, addr_hi, result);

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 0));
        self.set_flag_to_bool(Flag::FlagN, false); // 0 shifted into high bit, result always positive
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn mvn_all(&mut self, bus: &mut CpuBus, src_addr: Address, dst_addr: Address) {
        // Idx registers incremented in block move negative (it's backwards, I know)
        // "Negative" actually refers to where the destination address is relative
        // to the source address.
        self.x += 1;
        self.y += 1;

        if self.is_flag_set(Flag::FlagX) {
            self.x &= 0xFF;
            self.y &= 0xFF;
        }

        let data = self.read(bus, src_addr);
        self.write(bus, dst_addr, data);

        self.a -= 1;

        // This instruction essensially jumps to itself until it has moved self.acc + 1
        // bytes. self.acc will be 0xFFFF after this instruction. The addressing mode
        // of this instruction is always BlockMove, so the instruction is always 3 bytes.
        if self.a != 0xFFFF {
            self.pc -= 3;
        } else {
            // Finished moving data
            self.db = dst_addr.bank; // overwrites the dataBank register with the destination bank when finished
        }
    }

    fn mvp_all(&mut self, bus: &mut CpuBus, src_addr: Address, dst_addr: Address) {
        // Idx registers decremented in block move positive (it's backwards, I know)
        // "Positive" actually refers to where the destination address is relative
        // to the source address.
        self.x -= 1;
        self.y -= 1;

        if self.is_flag_set(Flag::FlagX) {
            self.x &= 0xFF;
            self.y &= 0xFF;
        }

        let data = self.read(bus, src_addr);
        self.write(bus, dst_addr, data);

        self.a -= 1;

        // This instruction essensially jumps to itself until it has moved self.acc + 1
        // bytes. self.acc will be 0xFFFF after this instruction. The addressing mode
        // of this instruction is always BlockMove, so the instruction is always 3 bytes.
        if self.a != 0xFFFF {
            self.pc -= 3;
        } else {
            // Finished moving data
            self.db = dst_addr.bank; // overwrites the dataBank register with the destination bank when finished
        }
    }
    
    fn nop_all(&mut self) {
    }
    
    fn ora_m8(&mut self, bus: &mut CpuBus, address: Address) {
        self.a |= self.read(bus, address) as u16;
        set_nz8!(self, self.a);
    }
    
    fn ora_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.a |= self.read_word(bus, addr_lo, addr_hi) as u16;
        set_nz16!(self, self.a);
    }
    
    fn pex_all(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        self.push_word(bus, data);
    }

    fn per_all(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let offset = self.read_word(bus, addr_lo, addr_hi);
        self.push_word(bus, self.pc + offset);
    }

    fn pha_all(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagM) {
            self.push(bus, self.a as u8);
        } else {
            self.push_word(bus, self.a);
        }
    }

    fn phb_all(&mut self, bus: &mut CpuBus) {
        self.push(bus, self.db);
    }

    fn phd_all(&mut self, bus: &mut CpuBus) {
        self.push_word(bus, self.dp);
    }

    fn phk_all(&mut self, bus: &mut CpuBus) {
        self.push(bus, self.pb);
    }

    fn php_all(&mut self, bus: &mut CpuBus) {
        self.push(bus, self.p);
    }

    fn phx_all(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagX) {
            self.push(bus, self.x as u8);
        } else {
            self.push_word(bus, self.x);
        }
    }

    fn phy_all(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagX) {
            self.push(bus, self.y as u8);
        } else {
            self.push_word(bus, self.y);
        }
    }

    fn pla_all(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagM) {
            set_byte_n!(self.a, self.pop(bus) as u16, 0);
            set_nz8!(self, self.a);
        } else {
            self.a = self.pop_word(bus);
            set_nz16!(self, self.a);
        }
    }

    fn plb_all(&mut self, bus: &mut CpuBus) {
        self.db = self.pop(bus);
        set_nz8!(self, self.db);
    }

    fn pld_all(&mut self, bus: &mut CpuBus) {
        self.dp = self.pop_word(bus);
        set_nz16!(self, self.dp);
    }

    fn plp_all(&mut self, bus: &mut CpuBus) {
        self.p = self.pop(bus);
        
        if self.e {
            self.set_flag_to_bool(Flag::FlagX, true);
            self.set_flag_to_bool(Flag::FlagM, true);
        }

        if self.is_flag_set(Flag::FlagX) {
            self.x &= 0xFF;
            self.y &= 0xFF;
        }
    }

    fn plx_all(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagX) {
            self.x = self.pop(bus) as u16;
            set_nz8!(self, self.x);
        } else {
            self.x = self.pop_word(bus);
            set_nz16!(self, self.x);
        }
    }

    fn ply_all(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagM) {
            self.y = self.pop(bus) as u16;
            set_nz8!(self, self.y);
        } else {
            self.y = self.pop_word(bus);
            set_nz16!(self, self.y);
        }
    }
    
    fn rep_all(&mut self, bus: &mut CpuBus, addr: Address) {
        self.p &= !self.read(bus, addr);
        
        if self.e {
            self.set_flag_to_bool(Flag::FlagM, true);
            self.set_flag_to_bool(Flag::FlagX, true);
        }
    }

    fn rol_all(&mut self) {
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        
        if self.is_flag_set(Flag::FlagM) {
            self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 7));
    
            set_byte_n!(self.a, self.a << 1, 0);
            self.a |= c;
    
            set_nz8!(self, self.a);
        } else {
            self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 15));
    
            self.a <<= 1;
            self.a |= c;
    
            set_nz16!(self, self.a);
        }
    }
    
    fn rol_mem_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        let data = self.read(bus, addr);
        let result = (data << 1) | c;
        
        self.write(bus, addr, result);

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 7));

        set_nz8!(self, result);
    }
    
    fn rol_mem_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        let data = self.read_word(bus, addr_lo, addr_hi);
        let result = (data << 1) | c;

        self.write_word(bus, addr_lo, addr_hi, result);

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 15));

        set_nz16!(self, result);
    }

    fn ror_all(&mut self) {
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 0));
        
        if self.is_flag_set(Flag::FlagM) {    
            set_byte_n!(self.a, (self.a >> 1) | (c << 7), 0);
            set_nz8!(self, self.a);
        } else {
            self.a = (self.a >> 1) | (c << 15);
            set_nz16!(self, self.a);
        }
    }

    fn ror_mem_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        let data = self.read(bus, addr);
        let result = (data >> 1) | (c << 7);

        self.write(bus, addr, result);

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 0));
        
        set_nz8!(self, result);
    }
    
    fn ror_mem_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        let data = self.read_word(bus, addr_lo, addr_hi);
        let result = (data >> 1) | (c << 15);

        self.write_word(bus, addr_lo, addr_hi, result);

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 0));
        
        set_nz16!(self, result);
    }

    fn rti_all(&mut self, bus: &mut CpuBus) {
        self.p = self.pop(bus);
        self.pc = self.pop_word(bus);
        
        if self.e {
            self.set_flag_to_bool(Flag::FlagM, true);
            self.set_flag_to_bool(Flag::FlagX, true);
        } else {
            self.pb = self.pop(bus);
        }
        
        if self.is_flag_set(Flag::FlagX) {
            self.x &= 0xFF;
            self.y &= 0xFF;
        }
    }

    fn rtl_all(&mut self, bus: &mut CpuBus) {
        self.pc = self.pop_word(bus) + 1;
        self.pb = self.pop(bus);
    }

    fn rts_all(&mut self, bus: &mut CpuBus) {
        self.pc = self.pop_word(bus) + 1;
    }
    
    fn sbc_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let data = self.read(bus, addr);
        let comp = !data;
        let mut result: u16;
        let a = self.a & 0xFF;
        let d = comp as u16;
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        if self.is_flag_set(Flag::FlagD) {
            result = (a & 0x0F) + (d & 0x0F) + c;

            if result <= 0x0F {
                result -= 0x06;
            }

            let c = if result >= 0x10 { 0x10 } else { 0 };
            result = (a & 0xF0) + (d & 0xF0) + c + (result & 0xF);
        } else {
            result = a + d + c;
        }

        self.set_flag_to_bool(Flag::FlagV, get_bit_n!(!(a ^ d) & (d ^ result), 7));

        if self.is_flag_set(Flag::FlagD) && result <= 0xFF {
            result -= 0x60;
        }

        self.set_flag_to_bool(Flag::FlagC, result > 0xFF);

        set_byte_n!(self.a, result, 0);

        set_nz8!(self, self.a);
    }
    
    fn sbc_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        let comp = !data;
        let mut result: u32;
        let a = self.a as u32;
        let d = comp as u32;
        let c = if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        if self.is_flag_set(Flag::FlagD) {
            result = (a & 0x000F) + (d & 0x000F) + c;

            if result <= 0xF {
                result -= 6;
            }

            let c = if result >= 0x10 { 0x10 } else { 0 };
            result = (a & 0x00F0) + (d & 0x00F0) + c + (result & 0xF);

            if result <= 0xFF {
                result -= 0x60;
            }

            let c = if result >= 0x100 { 0x100 } else { 0 };
            result = (a & 0x0F00) + (d & 0x0F00) + c + (result & 0xFF);

            if result <= 0xFFF {
                result -= 0x600;
            }

            let c = if result >= 0x1000 { 0x1000 } else { 0 };
            result = (a & 0xF000) + (d & 0xF000) + c + (result & 0xFFF);
        } else {
            result = a + d + c;
        }

        self.set_flag_to_bool(Flag::FlagV, get_bit_n!(!(a ^ d) & (d ^ result), 15));

        if self.is_flag_set(Flag::FlagD) && result <= 0xFFFF {
            result -= 0x6000;
        }

        self.set_flag_to_bool(Flag::FlagC, result > 0xFFFF);

        self.a = result as u16;

        set_nz16!(self, self.a);
    }

    fn sec_all(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, true);
    }

    fn sed_all(&mut self) {
        self.set_flag_to_bool(Flag::FlagD, true);
    }

    fn sei_all(&mut self) {
        self.set_flag_to_bool(Flag::FlagI, true);
    }

    fn sep_all(&mut self, bus: &mut CpuBus, addr: Address) {
        self.p |= self.read(bus, addr);

        if self.is_flag_set(Flag::FlagX) {
            self.x &= 0xFF;
            self.y &= 0xFF;
        }
    }

    fn sta_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.write(bus, addr, self.a as u8);
    }
    
    fn sta_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.write_word(bus, addr_lo, addr_hi, self.a)
    }

    fn stp_all(&mut self) {
        self.stopped = true;
    }

    fn stx_x8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.write(bus, addr, self.x as u8);
    }
    
    fn stx_x16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.write_word(bus, addr_lo, addr_hi, self.x)
    }

    fn stz_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.write(bus, addr, 0);
    }
    
    fn stz_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.write_word(bus, addr_lo, addr_hi, 0)
    }
    
    fn tax_all(&mut self) {
        transfer_reg!(self, self.a, self.x, Flag::FlagX);
    }

    fn tay_x8(&mut self) {
        transfer_reg!(self, self.a, self.y, Flag::FlagX);
    }

    fn tcd_all(&mut self) {
        self.dp = self.a;
        set_nz16!(self, self.dp);
    }

    fn tcs_n(&mut self) {
        if self.e {
            self.sp = 0x100 | (self.a & 0xFF);
        } else {
            self.sp = self.a;
        }
    }

    fn tdc_all(&mut self) {
        self.a = self.dp;
        set_nz16!(self, self.a);
    }

    fn trb_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let data = self.read(bus, addr);
        let result = data & (self.a as u8);

        self.write(bus, addr, data & !(self.a as u8));

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }
    
    fn trb_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        let result = data & self.a;

        self.write_word(bus, addr_lo, addr_hi, data & !self.a);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tsb_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        let data = self.read(bus, addr);
        let result = data & (self.a as u8);

        self.write(bus, addr, data | (self.a as u8));

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tsb_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        let result = data & self.a;

        self.write_word(bus, addr_lo, addr_hi, data | self.a);

        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn tsc_all(&mut self) {
        self.a = self.sp;
        set_nz16!(self, self.a);
    }

    fn tsx_all(&mut self) {
        transfer_reg!(self, self.sp, self.x, Flag::FlagX);
    }

    fn txa_all(&mut self) {
        transfer_reg!(self, self.x, self.a, Flag::FlagM);
    }

    fn txs_all(&mut self) {
        if self.e {
            self.sp = 0x100 | (self.x & 0xFF);
        } else {
            self.sp = self.x;
        }
    }

    fn txy_all(&mut self) {
        transfer_reg!(self, self.x, self.y, Flag::FlagX);
    }

    fn tya_all(&mut self) {
        transfer_reg!(self, self.y, self.a, Flag::FlagM);
    }

    fn tyx_all(&mut self) {
        transfer_reg!(self, self.y, self.x, Flag::FlagX);
    }
    
    fn wai_all(&mut self) {
        self.waiting_for_irq = true;
    }

    fn wdm_all(&mut self, bus: &mut CpuBus, addr: Address) {
        let _ = self.read(bus, addr);
    }

    fn xba_all(&mut self) {
        self.a = self.a.swap_bytes();
        set_nz8!(self, self.a); // Flags are always based on the low byte of A for this instr
    }

    fn xce_all(&mut self) {
        self.e = !self.e;
        
        if self.e {
            self.x &= 0xFF;
            self.y &= 0xFF;
            self.sp = 0x100 | (self.sp & 0xFF);
        }
    }
}