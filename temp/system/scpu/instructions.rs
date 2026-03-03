use crate::system::scpu::Flag;
use crate::utils::{inc_low_byte, GetBits, GetBytes, SetBytes};

use super::Cpu65c816;
use super::CpuAddress;

macro_rules! m8 {
    ($cpu:expr, $block:expr) => {
        if $cpu.is_flag_set(Flag::FlagM) { $block }
    };
}
macro_rules! m16 {
    ($cpu:expr, $block:expr) => {
        if !$cpu.is_flag_set(Flag::FlagM) { $block }
    };
}
macro_rules! m8_or_else {
    ($cpu:expr, $ifblock:expr, $elseblock:expr) => {
        if $cpu.is_flag_set(Flag::FlagM) { $ifblock } else { $elseblock }
    };
}
macro_rules! x8 {
    ($cpu:expr, $block:expr) => {
        if $cpu.is_flag_set(Flag::FlagX) { $block }
    };
}
macro_rules! x16 {
    ($cpu:expr, $block:expr) => {
        if !$cpu.is_flag_set(Flag::FlagX) { $block }
    };
}
macro_rules! x8_or_else {
    ($cpu:expr, $ifblock:expr, $elseblock:expr) => {
        if $cpu.is_flag_set(Flag::FlagX) { $ifblock } else { $elseblock }
    };
}

// Addressing Modes
impl Cpu65c816 {
    pub fn immediate(&mut self) -> CpuAddress {
        let addr = self.pc;
        self.pc += 1;
        (((self.prg_bank as u32) << 16) | (addr as u32)).into()
    }

    pub fn absolute(&mut self) -> CpuAddress {
        let lo = self.read_prg();
        let mi = self.read_prg();
        CpuAddress::from_parts(self.data_bank, mi, lo)
    }

    pub fn absolute_x(&mut self) -> CpuAddress {
        let lo = self.read_prg() as u16;
        let hi = self.read_prg() as u16;
        let bank_addr = ((hi << 8) | lo) + self.x;

        (((self.prg_bank as u32) << 16) | (bank_addr as u32)).into()
    }

    pub fn absolute_y(&mut self) -> CpuAddress {
        let lo = self.read_prg() as u16;
        let hi = self.read_prg() as u16;
        let bank_addr = ((hi << 8) | lo) + self.y;

        (((self.prg_bank as u32) << 16) | (bank_addr as u32)).into()
    }

    pub fn absolute_indirect(&mut self) -> CpuAddress {
        let ll = self.read_prg();
        let hh = self.read_prg();

        let ptr_lo = (hh as u16) << 8 | ll as u16;
        let ptr_hi = ptr_lo + 1;

        let lo = self.read(ptr_lo as u32);
        let mi = self.read(ptr_hi as u32);

        CpuAddress::from_parts(self.data_bank, mi, lo)
    }

    pub fn direct(&mut self) -> CpuAddress {
        CpuAddress( (self.direct_page + self.read_prg() as u16) as u32 )
    }

    pub fn direct_x(&mut self) -> CpuAddress {
        let addr = self.direct();

        if self.emulation_mode && self.direct_page.get_lo() == 0 {
            addr.wrapping_add8(self.x as u8)
        } else {
            addr.wrapping_add16(self.x)
        }
    }

    pub fn direct_y(&mut self) -> CpuAddress {
        let lo = self.read_prg();

        if self.emulation_mode && self.direct_page.get_lo() == 0 {
            CpuAddress::from_parts(0, self.direct_page.get_hi(), lo + self.y as u8)
        } else {
            ((self.direct_page + self.y + lo as u16) as u32).into()
        }
    }

    pub fn direct_indirect(&mut self) -> CpuAddress {
        let ll = self.read_prg();
        let hh = self.read_prg();

        let ptr_lo = self.direct_page + (((hh as u16) << 8) | ll as u16);
        let ptr_mi = if self.emulation_mode && self.direct_page.get_lo() == 0 {
            inc_low_byte(ptr_lo)
        } else {
            ptr_lo + 1
        };

        let lo = self.read(ptr_lo as u32);
        let mi = self.read(ptr_mi as u32);

        CpuAddress::from_parts(self.data_bank, mi, lo)
    }

    pub fn direct_indirect_long(&mut self) -> CpuAddress {
        let ll = self.read_prg();
        let hh = self.read_prg();

        let ptr_lo = self.direct_page + (((hh as u16) << 8) | ll as u16);
        let ptr_mi = ptr_lo + 1;
        let ptr_hi = ptr_mi + 1;

        let lo = self.read(ptr_lo as u32);
        let mi = self.read(ptr_mi as u32);
        let hi = self.read(ptr_hi as u32);

        CpuAddress::from_parts(hi, mi, lo)
    }

    pub fn direct_x_indirect(&mut self) -> CpuAddress {
        let ptr_lo = self.direct_x();
        let ptr_mi = if self.emulation_mode && self.direct_page.get_lo() == 0 {
            (ptr_lo & 0xFFFF00) | ((ptr_lo + 1) & 0x0000FF)
        } else {
             (ptr_lo + 1) & 0xFFFFFF
        };

        let lo = self.read(ptr_lo);
        let mi = self.read(ptr_mi);

        CpuAddress::from_parts(self.data_bank, mi, lo)
    }

    pub fn direct_indirect_y(&mut self) -> CpuAddress {
        self.direct_indirect() + self.y as u32
    }

    pub fn direct_indirect_long_y(&mut self) -> CpuAddress {
        self.direct_indirect_long() + self.y as u32
    }

    // wrap full 32 bit addr
    pub fn long(&mut self) -> CpuAddress {
        let lo = self.read_prg();
        let mi = self.read_prg();
        let hi = self.read_prg();
        CpuAddress::from_parts(hi, mi, lo)
    }

    // wrap full 32 bit addr
    pub fn long_x(&mut self) -> CpuAddress {
        self.long() + self.x as u32
    }

    pub fn long_indirect(&mut self) -> CpuAddress {
        let ll = self.read_prg();
        let hh = self.read_prg();
        
        let ptr_lo = (hh as u16) << 8 | ll as u16;
        let ptr_mi = ptr_lo + 1;
        let ptr_hi = ptr_mi + 1;

        let lo = self.read(ptr_lo as u32);
        let mi = self.read(ptr_mi as u32);
        let hi = self.read(ptr_hi as u32);

        CpuAddress::from_parts(hi, mi, lo)
    }

    fn relative(&mut self) -> CpuAddress {
        let offset = ((self.read_prg() as i8) as i16) as u16;
        let bank_addr = self.pc + offset;

        (((self.prg_bank as u32) << 16) | (bank_addr as u32)).into()
    }

    fn relative_long(&mut self) -> CpuAddress {
        let offset_lo = self.read_prg() as u16;
        let offset_hi = self.read_prg() as u16;
        let offset = (offset_hi << 8) | offset_lo;
        let bank_addr = self.pc + offset;

        (((self.prg_bank as u32) << 16) | (bank_addr as u32)).into()
    }

    fn source_dest(&mut self) -> (CpuAddress, CpuAddress) {
        let src_bank = self.read_prg() as u32;
        let dst_bank = self.read_prg() as u32;

        (CpuAddress(src_bank | self.x as u32), CpuAddress(dst_bank | self.y as u32))
    }

    pub fn stack_relative(&mut self) -> CpuAddress {
        ((self.stk_ptr + self.read_prg() as u16) as u32).into()
    }

    pub fn stack_relative_indirect_y(&mut self) -> CpuAddress {
        let ptr_lo = self.stack_relative();
        let ptr_mi = ptr_lo.wrapping_add16(1);

        let lo = self.read(ptr_lo);
        let mi = self.read(ptr_mi);

        CpuAddress::from_parts(self.data_bank, mi, lo)
    }


    // fn absolute_indirect(&mut self) -> CpuAddress {
    //     let (ptr_lo, ptr_hi) = self.absolute16();
    //     let address_lo = self.read(ptr_lo.with_bank(0));
    //     let address_hi = self.read(ptr_hi.with_bank(0));
    //     CpuAddress::from_parts(self.prg_bank, address_hi, address_lo)
    // }
    // fn absolute_indirect_long(&mut self) -> CpuAddress {
    //     let (ptr_lo, ptr_mi) = self.absolute16();
    //     let ptr_hi = ptr_mi + 1;
    //     let address_lo = self.read(ptr_lo.with_bank(0));
    //     let address_mi = self.read(ptr_mi.with_bank(0));
    //     let address_hi = self.read(ptr_hi.with_bank(0));
    //     CpuAddress::from_parts(address_hi, address_mi, address_lo)
    // }

    // fn absolute_x_indirect8(&mut self) -> CpuAddress {
    //     let ptr_lo = self.absolute_x8().with_bank(self.prg_bank);
    //     let ptr_hi = (ptr_lo + 1).with_bank(self.prg_bank); // Wrap on bank
    //     let address_lo = self.read(ptr_lo);
    //     let address_hi = self.read(ptr_hi);
    //     CpuAddress::from_parts(self.prg_bank, address_hi, address_lo)
    // }

    // fn immediate8(&self) -> CpuAddress {
    //     ((self.prg_bank as u32) << 16) | (self.pc + 1) as u32
    // }
    // fn immediate16(&self) -> (u32, u32) {
    //     (
    //         ((self.prg_bank as u32) << 16) | (self.pc + 1) as u32,
    //         ((self.prg_bank as u32) << 16) | (self.pc + 2) as u32,
    //     )
    // }

    // fn direct8(&mut self) -> CpuAddress {
    //     // Direct addressing takes an extra cycle when DL != 0
    //     if self.direct_page & 0xFF != 0 {
    //         self.add_clocks(Cpu65c816::ONE_CYCLE);
    //     }

    //     (self.direct_page + self.read(self.immediate8()) as u16) as u32
    // }
    // fn direct16(&mut self) -> (u32, u32) {
    //     let direct = self.direct8();
    //     ((direct) as u32, (direct + 1) as u32)
    // }

    // fn direct_x8(&mut self) -> CpuAddress {
    //     match self.mode {
    //         CpuMode::Emulation => {
    //             let addr = self.direct8();

    //             if self.direct_page & 0xFF == 0 {
    //                 addr.with_page_addr(addr.page_addr() + self.x.get_lo())
    //             } else {
    //                 (addr + self.x as u32).with_bank(0)
    //             }
    //         }

    //         CpuMode::Native => (self.direct8() + self.x as u32).with_bank(0),
    //     }
    // }
    // fn direct_x16(&mut self) -> (u32, u32) {
    //     let addr = (self.direct8() + self.x as u32).with_bank(0);
    //     (addr, (addr + 1).with_bank(0))
    // }

    // fn direct_y8(&mut self) -> CpuAddress {
    //     match self.mode {
    //         CpuMode::Emulation => {
    //             let addr = self.direct8();

    //             if self.direct_page & 0xFF == 0 {
    //                 addr.with_page_addr(addr.page_addr() + self.y.get_lo())
    //             } else {
    //                 (addr + self.y as u32).with_bank(0)
    //             }
    //         }

    //         CpuMode::Native => (self.direct8() + self.y as u32).with_bank(0),
    //     }
    // }
    // fn direct_y16(&mut self) -> (u32, u32) {
    //     let addr = (self.direct8() + self.y as u32).with_bank(0);
    //     (addr, (addr + 1).with_bank(0))
    // }

    // fn direct_indirect8(&mut self) -> CpuAddress {
    //     let ptr_lo = self.direct8();
    //     let ptr_hi = match self.mode {
    //         CpuMode::Native => (ptr_lo + 1).with_bank(0),
    //         CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
    //     };

    //     CpuAddress::from_parts(self.data_bank, self.read(ptr_hi), self.read(ptr_lo))
    // }
    // fn direct_indirect16(&mut self) -> (u32, u32) {
    //     let ptr_lo = self.direct8();
    //     let ptr_hi = (ptr_lo + 1).with_bank(0);

    //     let address_lo = CpuAddress::from_parts(self.data_bank, self.read(ptr_hi), self.read(ptr_lo));
    //     let address_hi = (address_lo + 1) & 0xFFFFFF;

    //     (address_lo, address_hi)
    // }

    // fn direct_indirect_long8(&mut self) -> CpuAddress {
    //     let ptr_lo = self.direct8();
    //     let ptr_mi = (ptr_lo + 1).with_bank(0);
    //     let ptr_hi = (ptr_lo + 2).with_bank(0);

    //     CpuAddress::from_parts(self.read(ptr_hi), self.read(ptr_mi), self.read(ptr_lo))
    // }
    // fn direct_indirect_long16(&mut self) -> (u32, u32) {
    //     let ptr_lo = self.direct8();
    //     let ptr_mi = (ptr_lo + 1).with_bank(0);
    //     let ptr_hi = (ptr_lo + 2).with_bank(0);

    //     let address_lo = CpuAddress::from_parts(self.read(ptr_hi), self.read(ptr_mi), self.read(ptr_lo));
    //     let address_hi = (address_lo + 1) & 0xFFFFFF;

    //     (address_lo, address_hi)
    // }

    // fn direct_x_indirect8(&mut self) -> CpuAddress {
    //     let ptr_lo = self.direct_x8();
    //     let ptr_hi = match self.mode {
    //         CpuMode::Native => (ptr_lo + 1).with_bank(0),
    //         CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
    //     };

    //     let address_hi = self.read(ptr_hi);
    //     let address_lo = self.read(ptr_lo);

    //     CpuAddress::from_parts(self.data_bank, address_hi, address_lo)
    // }
    // fn direct_x_indirect16(&mut self) -> (u32, u32) {
    //     let ptr_lo = self.direct_x8();
    //     let ptr_hi = match self.mode {
    //         CpuMode::Native => (ptr_lo + 1).with_bank(0),
    //         CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
    //     };

    //     let address_hi = self.read(ptr_hi);
    //     let address_lo = self.read(ptr_lo);

    //     let addr = CpuAddress::from_parts(self.data_bank, address_hi, address_lo);

    //     (addr, (addr + 1) & 0xFFFFFF)
    // }

    // fn direct_indirect_y8(&mut self) -> CpuAddress {
    //     let ptr_lo = self.direct8();
    //     let ptr_hi = match self.mode {
    //         CpuMode::Native => (ptr_lo + 1).with_bank(0),
    //         CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
    //     };

    //     let original_addr = CpuAddress::from_parts(self.data_bank, self.read(ptr_hi), self.read(ptr_lo));
    //     let indexed_addr = (original_addr + self.y as u32) & 0xFFFFFF;

    //     self.page_crossed = original_addr.page() != indexed_addr.page();

    //     indexed_addr
    // }
    // fn direct_indirect_y16(&mut self) -> (u32, u32) {
    //     let ptr_lo = self.direct8();
    //     let ptr_hi = match self.mode {
    //         CpuMode::Native => (ptr_lo + 1).with_bank(0),
    //         CpuMode::Emulation => ptr_lo.with_page_addr(ptr_lo.page_addr() + 1),
    //     };

    //     let addr = (CpuAddress::from_parts(self.data_bank, self.read(ptr_hi), self.read(ptr_lo))
    //         + self.y as u32)
    //         & 0xFFFFFF;

    //     (addr, (addr + 1) & 0xFFFFFF)
    // }

    // fn direct_indirect_long_y8(&mut self) -> CpuAddress {
    //     let ptr_lo = self.direct8();
    //     let ptr_mi = (ptr_lo + 1).with_bank(0);
    //     let ptr_hi = (ptr_lo + 2).with_bank(0);

    //     (CpuAddress::from_parts(self.read(ptr_hi), self.read(ptr_mi), self.read(ptr_lo)) + self.y as u32)
    //         & 0xFFFFFF
    // }
    // fn direct_indirect_long_y16(&mut self) -> (u32, u32) {
    //     let ptr_lo = self.direct8();
    //     let ptr_mi = (ptr_lo + 1).with_bank(0);
    //     let ptr_hi = (ptr_lo + 2).with_bank(0);

    //     let addr = (CpuAddress::from_parts(self.read(ptr_hi), self.read(ptr_mi), self.read(ptr_lo))
    //         + self.y as u32)
    //         & 0xFFFFFF;

    //     (addr, (addr + 1) & 0xFFFFFF)
    // }

    // fn relative8(&mut self) -> CpuAddress {
    //     let offset = (self.read(self.immediate8()) as i8) as u16;
    //     let new_pc = self.pc + 2 + offset;
    //     let relative_addr = (new_pc as u32).with_bank(self.prg_bank);

    //     self.page_crossed = relative_addr.page() != relative_addr.page();

    //     relative_addr
    // }
    // fn relative16(&mut self) -> CpuAddress {
    //     let (offset_lo, offset_hi) = self.immediate16();
    //     let offset = self.read16(offset_lo, offset_hi);
    //     ((self.pc + offset + 3) as u32).with_bank(self.prg_bank)
    // }

    // fn src_dst(&mut self) -> (u32, u32) {
    //     let (address_src, address_dst) = self.immediate16();

    //     let src_bank = self.read(address_src);
    //     let src = (self.x as u32).with_bank(src_bank);

    //     let dst_bank = self.read(address_dst);
    //     let dst = (self.y as u32).with_bank(dst_bank);

    //     (src, dst)
    // }

    // fn stack_s8(&mut self) -> CpuAddress {
    //     let val = self.read(self.immediate8()) as u16;

    //     (val + self.stk_ptr) as u32
    // }
    // fn stack_s16(&mut self) -> (u32, u32) {
    //     let val = self.read(self.immediate8()) as u16;

    //     ((val + self.stk_ptr) as u32, (val + self.stk_ptr + 1) as u32)
    // }

    // fn stack_indirect_y8(&mut self) -> CpuAddress {
    //     let (ptr_lo, ptr_hi) = self.stack_s16();

    //     let address_lo = self.read(ptr_lo);
    //     let address_hi = self.read(ptr_hi);

    //     let addr = CpuAddress::from_parts(self.data_bank, address_hi, address_lo);

    //     (addr + self.y as u32) & 0xFFFFFF
    // }
    // fn stack_indirect_y16(&mut self) -> (u32, u32) {
    //     let (ptr_lo, ptr_hi) = self.stack_s16();

    //     let address_lo = self.read(ptr_lo);
    //     let address_hi = self.read(ptr_hi);

    //     let addr = CpuAddress::from_parts(self.data_bank, address_hi, address_lo);

    //     (
    //         (addr + self.y as u32) & 0xFFFFFF,
    //         (addr + self.y as u32 + 1) & 0xFFFFFF,
    //     )
    // }
}

macro_rules! set_nz {
    ($cpu:expr, $reg_flag:expr, $val:expr) => {
        if $cpu.is_flag_set($reg_flag) {
            $cpu.set_flag_to_bool(Flag::FlagN, $val.bit_en(7));
            $cpu.set_flag_to_bool(Flag::FlagZ, ($val as u8) == 0);
        } else {
            $cpu.set_flag_to_bool(Flag::FlagN, $val.bit_en(15));
            $cpu.set_flag_to_bool(Flag::FlagZ, $val == 0);
        }
    };
}

// Instructions
impl Cpu65c816 {
    fn and(&mut self, addr1: CpuAddress, addr2: CpuAddress) {
        m8!(self, self.acc.set_lo(self.acc.get_lo() & self.read(addr1)));
        m16!(self, self.acc = self.acc & self.read_word(addr1, addr2));
        set_nz!(self, Flag::FlagM, self.acc)
    }

    fn asl_acc(&mut self, addr1: CpuAddress, addr2: CpuAddress) {
        self.set_flag_to_bool(Flag::FlagC, self.acc.bit_en(m8_or_else!(self, 7, 15)));

        m8_or_else!(self.)
    }

    fn asl_acc_m8(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc.get_lo().bit_en(7));

        self.acc.set_lo(self.acc.get_lo() << 1);

        self.set_flag_to_bool(Flag::FlagN, self.acc.get_lo().bit_en(7));
        self.set_flag_to_bool(Flag::FlagZ, self.acc.get_lo() == 0);
    }

    fn asl_acc_m16(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, self.acc.bit_en(15));

        self.acc <<= 1;

        self.set_flag_to_bool(Flag::FlagN, self.acc.bit_en(15));
        self.set_flag_to_bool(Flag::FlagZ, self.acc == 0);
    }
}