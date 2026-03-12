use crate::core::scpu::bus::{Address, CpuBus};
use crate::core::scpu::{Cpu65c816, CpuInterrupt, Flag};
use crate::{
    set_byte_n,
    get_bit_n,
};
use log::{debug, trace};
use paste::paste;

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

macro_rules! inc_addr {
    ($addr:expr) => {
        Address::from_u32($addr.to_u32() + 1)
    };
}

macro_rules! inc_addr16 {
    ($addr:expr) => {
        Address { bank: $addr.bank, offset: $addr.offset + 1 }
    };
}

macro_rules! op_case {
    ($cpu:ident, $bus:ident, $addr_mode:ident, $instr:ident) => {
        {
            let addr = $cpu.$addr_mode($bus);
            $cpu.$instr($bus, addr);
        }
    };
}

macro_rules! op_case_imm {
    ($cpu:ident, $bus:ident, $instr:ident) => {
        {
            let addr = $cpu.immediate();
            $cpu.$instr($bus, addr);
        }
    };
}

macro_rules! op_case_br {
    ($cpu:ident, $bus:ident, $addr_mode:ident, $instr:ident) => {
        {
            let addr = $cpu.$addr_mode($bus);
            $cpu.$instr(addr);
        }
    };
}

macro_rules! op_case_src_dst {
    ($cpu:ident, $bus:ident, $instr:ident) => {
        {
            let src_addr = $cpu.source($bus);
            let dst_addr = $cpu.destination($bus);
            $cpu.$instr($bus, src_addr, dst_addr);
        }
    };
}

macro_rules! op_case_long {
    ($cpu:ident, $bus:ident, $addr_mode:ident, $instr:ident, $wrap_add:ident) => {
        {
            let addr_lo = $cpu.$addr_mode($bus);
            let addr_hi = $wrap_add!(addr_lo);
            $cpu.$instr($bus, addr_lo, addr_hi);
        }
    };
}

macro_rules! op_case_long_imm {
    ($cpu:ident, $bus:ident, $instr:ident) => {
        {
            let addr_lo = $cpu.immediate();
            let addr_hi = $cpu.immediate();
            $cpu.$instr($bus, addr_lo, addr_hi);
        }
    };
}

macro_rules! op_case_flagm {
    ($cpu:ident, $bus:ident, $addr_mode:ident, $instr:ident, $wrap_add:ident) => {
        paste!( {
            let addr = $cpu.$addr_mode($bus);
        
            if $cpu.is_flag_set(Flag::FlagM) {
                $cpu.[<$instr _m8>]($bus, addr);
            } else {
                $cpu.[<$instr _m16>]($bus, addr, $wrap_add!(addr));
            }
        } )
    };
}

macro_rules! op_case_flagm_imm {
    ($cpu:ident, $bus:ident, $instr:ident) => {
        paste!( {
            let addr = $cpu.immediate();
        
            if $cpu.is_flag_set(Flag::FlagM) {
                $cpu.[<$instr _m8>]($bus, addr);
            } else {
                let addr2 = $cpu.immediate();
                $cpu.[<$instr _m16>]($bus, addr, addr2);
            }
        } )
    };
}

macro_rules! op_case_flagx {
    ($cpu:ident, $bus:ident, $addr_mode:ident, $instr:ident, $wrap_add:ident) => {
        paste!( {
            let addr = $cpu.$addr_mode($bus);
        
            if $cpu.is_flag_set(Flag::FlagX) {
                $cpu.[<$instr _x8>]($bus, addr);
            } else {
                $cpu.[<$instr _x16>]($bus, addr, $wrap_add!(addr));
            }
        } )
    };
}

macro_rules! op_case_flagx_imm {
    ($cpu:ident, $bus:ident, $instr:ident) => {
        paste!( {
            let addr = $cpu.immediate();
        
            if $cpu.is_flag_set(Flag::FlagX) {
                $cpu.[<$instr _x8>]($bus, addr);
            } else {
                let addr2 = $cpu.immediate();
                $cpu.[<$instr _x16>]($bus, addr, addr2);
            }
        } )
    };
}


// Instr execution
impl Cpu65c816 {
    /// Some instructions take extra cycles beyond what is spent reading/writing memory.
    /// This is a lookup for the extra cycles needed for each opcode.
    const EXTRA_CYCLES_LOOKUP: [u8; 256] = [
        1, 2, 0, 1, 2, 1, 2, 1, 1, 0, 1, 1, 1, 0, 1, 1,
        0, 2, 1, 2, 2, 2, 3, 1, 1, 1, 1, 1, 1, 1, 2, 0,
        1, 2, 2, 1, 1, 1, 2, 1, 2, 0, 1, 2, 0, 0, 1, 1,
        0, 2, 1, 2, 2, 2, 3, 1, 1, 1, 1, 1, 1, 1, 2, 0,
        2, 2, 0, 1, 2, 1, 2, 1, 1, 0, 1, 1, 0, 0, 1, 1,
        0, 2, 1, 2, 2, 2, 3, 1, 1, 1, 1, 1, 1, 1, 2, 0,
        4, 2, 1, 1, 1, 1, 2, 1, 2, 0, 1, 3, 0, 0, 1, 1,
        0, 2, 1, 2, 2, 2, 3, 1, 1, 1, 2, 1, 1, 1, 2, 0,
        0, 2, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1,
        0, 2, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 1, 1, 0,
        0, 2, 0, 1, 1, 1, 1, 1, 1, 0, 1, 2, 0, 0, 0, 1,
        0, 2, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0,
        0, 2, 1, 1, 1, 1, 2, 1, 1, 0, 1, 2, 0, 0, 1, 1,
        0, 2, 1, 2, 1, 2, 3, 1, 1, 1, 1, 2, 0, 1, 2, 0,
        0, 2, 1, 1, 1, 1, 2, 1, 1, 0, 1, 2, 0, 0, 1, 1,
        0, 2, 1, 2, 0, 2, 3, 1, 1, 1, 2, 1, 1, 1, 2, 0,
    ];

    /// Fetch, decode, and execute a single instruction. The number of system clocks taken
    /// to complete the instruction is added to the CPU's internal clock counter.
    pub fn execute(&mut self, bus: &mut CpuBus) {
        let opcode = self.read_prg(bus);
        
        if opcode == 0x60 {
            let ret_addr = self.pop_word(bus);
            self.sp -= 2;
            self.debug_cnt -= 1;
            debug!("${:02X}{:04X}: RTS, Stk: {:04X}, depth: {}", self.pb, self.pc, ret_addr, self.debug_cnt);
        }

        self.branch_taken = false;
        
        match opcode {
            0x00 => self.brk(bus),
            0x01 => op_case_flagm!(self, bus, direct_x_indirect, ora, inc_addr),
            0x02 => op_case_imm!(self, bus, cop),
            0x03 => op_case_flagm!(self, bus, stack_relative, ora, inc_addr16),
            0x04 => op_case_flagm!(self, bus, direct, tsb, inc_addr16),
            0x05 => op_case_flagm!(self, bus, direct, ora, inc_addr16),
            0x06 => op_case_flagm!(self, bus, direct, asl_mem, inc_addr16),
            0x07 => op_case_flagm!(self, bus, direct_indirect_long, ora, inc_addr),
            0x08 => self.php(bus),
            0x09 => op_case_flagm_imm!(self, bus, ora),
            0x0A => self.asl(),
            0x0B => self.phd(bus),
            0x0C => op_case_flagm!(self, bus, absolute, tsb, inc_addr),
            0x0D => op_case_flagm!(self, bus, absolute, ora, inc_addr),
            0x0E => op_case_flagm!(self, bus, absolute, asl_mem, inc_addr),
            0x0F => op_case_flagm!(self, bus, long, ora, inc_addr),
            0x10 => op_case_br!(self, bus, relative, bpl),
            0x11 => op_case_flagm!(self, bus, direct_indirect_y, ora, inc_addr),
            0x12 => op_case_flagm!(self, bus, direct_indirect, ora, inc_addr),
            0x13 => op_case_flagm!(self, bus, stack_relative_indirect_y, ora, inc_addr),
            0x14 => op_case_flagm!(self, bus, direct, trb, inc_addr16),
            0x15 => op_case_flagm!(self, bus, direct_x, ora, inc_addr16),
            0x16 => op_case_flagm!(self, bus, direct_x, asl_mem, inc_addr16),
            0x17 => op_case_flagm!(self, bus, direct_indirect_long_y, ora, inc_addr),
            0x18 => self.clc(),
            0x19 => op_case_flagm!(self, bus, absolute_y, ora, inc_addr),
            0x1A => self.inc(),
            0x1B => self.tcs(),
            0x1C => op_case_flagm!(self, bus, absolute, trb, inc_addr),
            0x1D => op_case_flagm!(self, bus, absolute_x, ora, inc_addr),
            0x1E => op_case_flagm!(self, bus, absolute_x, asl_mem, inc_addr),
            0x1F => op_case_flagm!(self, bus, long_x, ora, inc_addr),
            0x20 => op_case!(self, bus, absolute, jsr),
            0x21 => op_case_flagm!(self, bus, direct_x_indirect, and, inc_addr),
            0x22 => op_case!(self, bus, long, jsl),
            0x23 => op_case_flagm!(self, bus, stack_relative, and, inc_addr16),
            0x24 => op_case_flagm!(self, bus, direct, bit, inc_addr16),
            0x25 => op_case_flagm!(self, bus, direct, and, inc_addr16),
            0x26 => op_case_flagm!(self, bus, direct, rol_mem, inc_addr16),
            0x27 => op_case_flagm!(self, bus, direct_indirect_long, and, inc_addr),
            0x28 => self.plp(bus),
            0x29 => op_case_flagm_imm!(self, bus, and),
            0x2A => self.rol(),
            0x2B => self.pld(bus),
            0x2C => op_case_flagm!(self, bus, absolute, bit, inc_addr),
            0x2D => op_case_flagm!(self, bus, absolute, and, inc_addr),
            0x2E => op_case_flagm!(self, bus, absolute, rol_mem, inc_addr),
            0x2F => op_case_flagm!(self, bus, long, and, inc_addr),
            0x30 => op_case_br!(self, bus, relative, bmi),
            0x31 => op_case_flagm!(self, bus, direct_indirect_y, and, inc_addr),
            0x32 => op_case_flagm!(self, bus, direct_indirect, and, inc_addr),
            0x33 => op_case_flagm!(self, bus, stack_relative_indirect_y, and, inc_addr),
            0x34 => op_case_flagm!(self, bus, direct_x, bit, inc_addr16),
            0x35 => op_case_flagm!(self, bus, direct_x, and, inc_addr16),
            0x36 => op_case_flagm!(self, bus, direct_x, rol_mem, inc_addr16),
            0x37 => op_case_flagm!(self, bus, direct_indirect_long_y, and, inc_addr),
            0x38 => self.sec(),
            0x39 => op_case_flagm!(self, bus, absolute_y, and, inc_addr),
            0x3A => self.dec(),
            0x3B => self.tsc(),
            0x3C => op_case_flagm!(self, bus, absolute_x, bit, inc_addr),
            0x3D => op_case_flagm!(self, bus, absolute_x, and, inc_addr),
            0x3E => op_case_flagm!(self, bus, absolute_x, rol_mem, inc_addr),
            0x3F => op_case_flagm!(self, bus, long_x, and, inc_addr),
            0x40 => self.rti(bus),
            0x41 => op_case_flagm!(self, bus, direct_x_indirect, eor, inc_addr),
            0x42 => op_case_imm!(self, bus, wdm),
            0x43 => op_case_flagm!(self, bus, stack_relative, eor, inc_addr16),
            0x44 => op_case_src_dst!(self, bus, mvp),
            0x45 => op_case_flagm!(self, bus, direct, eor, inc_addr16),
            0x46 => op_case_flagm!(self, bus, direct, lsr_mem, inc_addr16),
            0x47 => op_case_flagm!(self, bus, direct_indirect_long, eor, inc_addr),
            0x48 => self.pha(bus),
            0x49 => op_case_flagm_imm!(self, bus, eor),
            0x4A => self.lsr(),
            0x4B => self.phk(bus),
            0x4C => op_case_br!(self, bus, absolute, jmp),
            0x4D => op_case_flagm!(self, bus, absolute, eor, inc_addr),
            0x4E => op_case_flagm!(self, bus, absolute, lsr_mem, inc_addr),
            0x4F => op_case_flagm!(self, bus, long, eor, inc_addr),
            0x50 => op_case_br!(self, bus, relative, bvc),
            0x51 => op_case_flagm!(self, bus, direct_indirect_y, eor, inc_addr),
            0x52 => op_case_flagm!(self, bus, direct_indirect, eor, inc_addr),
            0x53 => op_case_flagm!(self, bus, stack_relative_indirect_y, eor, inc_addr),
            0x54 => op_case_src_dst!(self, bus, mvn),
            0x55 => op_case_flagm!(self, bus, direct_x, eor, inc_addr16),
            0x56 => op_case_flagm!(self, bus, direct_x, lsr_mem, inc_addr16),
            0x57 => op_case_flagm!(self, bus, direct_indirect_long_y, eor, inc_addr),
            0x58 => self.cli(),
            0x59 => op_case_flagm!(self, bus, absolute_y, eor, inc_addr),
            0x5A => self.phy(bus),
            0x5B => self.tcd(),
            0x5C => op_case_br!(self, bus, long, jmp),
            0x5D => op_case_flagm!(self, bus, absolute_x, eor, inc_addr),
            0x5E => op_case_flagm!(self, bus, absolute_x, lsr_mem, inc_addr),
            0x5F => op_case_flagm!(self, bus, long_x, eor, inc_addr),
            0x60 => self.rts(bus),
            0x61 => op_case_flagm!(self, bus, direct_x_indirect, adc, inc_addr),
            0x62 => op_case_long_imm!(self, bus, per),
            0x63 => op_case_flagm!(self, bus, stack_relative, adc, inc_addr16),
            0x64 => op_case_flagm!(self, bus, direct, stz, inc_addr16),
            0x65 => op_case_flagm!(self, bus, direct, adc, inc_addr16),
            0x66 => op_case_flagm!(self, bus, direct, ror_mem, inc_addr16),
            0x67 => op_case_flagm!(self, bus, direct_indirect_long, adc, inc_addr),
            0x68 => self.pla(bus),
            0x69 => op_case_flagm_imm!(self, bus, adc),
            0x6A => self.ror(),
            0x6B => self.rtl(bus),
            0x6C => op_case_br!(self, bus, absolute_indirect, jmp),
            0x6D => op_case_flagm!(self, bus, absolute, adc, inc_addr),
            0x6E => op_case_flagm!(self, bus, absolute, ror_mem, inc_addr),
            0x6F => op_case_flagm!(self, bus, long, adc, inc_addr),
            0x70 => op_case_br!(self, bus, relative, bvs),
            0x71 => op_case_flagm!(self, bus, direct_indirect_y, adc, inc_addr),
            0x72 => op_case_flagm!(self, bus, direct_indirect, adc, inc_addr),
            0x73 => op_case_flagm!(self, bus, stack_relative_indirect_y, adc, inc_addr),
            0x74 => op_case_flagm!(self, bus, direct_x, stz, inc_addr16),
            0x75 => op_case_flagm!(self, bus, direct_x, adc, inc_addr16),
            0x76 => op_case_flagm!(self, bus, direct_x, ror_mem, inc_addr16),
            0x77 => op_case_flagm!(self, bus, direct_indirect_long_y, adc, inc_addr),
            0x78 => self.sei(),
            0x79 => op_case_flagm!(self, bus, absolute_y, adc, inc_addr),
            0x7A => self.ply(bus),
            0x7B => self.tdc(),
            0x7C => op_case_br!(self, bus, absolute_x_indirect, jmp),
            0x7D => op_case_flagm!(self, bus, absolute_x, adc, inc_addr),
            0x7E => op_case_flagm!(self, bus, absolute_x, ror_mem, inc_addr),
            0x7F => op_case_flagm!(self, bus, long_x, adc, inc_addr),
            0x80 => op_case_br!(self, bus, relative, bra),
            0x81 => op_case_flagm!(self, bus, direct_x_indirect, sta, inc_addr),
            0x82 => op_case_br!(self, bus, relative_long, bra),
            0x83 => op_case_flagm!(self, bus, stack_relative, sta, inc_addr16),
            0x84 => op_case_flagx!(self, bus, direct, sty, inc_addr16),
            0x85 => op_case_flagm!(self, bus, direct, sta, inc_addr16),
            0x86 => op_case_flagx!(self, bus, direct, stx, inc_addr16),
            0x87 => op_case_flagm!(self, bus, direct_indirect_long, sta, inc_addr),
            0x88 => self.dey(),
            0x89 => op_case_flagm_imm!(self, bus, bit),
            0x8A => self.txa(),
            0x8B => self.phb(bus),
            0x8C => op_case_flagx!(self, bus, absolute, sty, inc_addr),
            0x8D => op_case_flagm!(self, bus, absolute, sta, inc_addr),
            0x8E => op_case_flagx!(self, bus, absolute, stx, inc_addr),
            0x8F => op_case_flagm!(self, bus, long, sta, inc_addr),
            0x90 => op_case_br!(self, bus, relative, bcc),
            0x91 => op_case_flagm!(self, bus, direct_indirect_y, sta, inc_addr),
            0x92 => op_case_flagm!(self, bus, direct_indirect, sta, inc_addr),
            0x93 => op_case_flagm!(self, bus, stack_relative_indirect_y, sta, inc_addr),
            0x94 => op_case_flagx!(self, bus, direct_x, sty, inc_addr16),
            0x95 => op_case_flagm!(self, bus, direct_x, sta, inc_addr16),
            0x96 => op_case_flagx!(self, bus, direct_y, stx, inc_addr16),
            0x97 => op_case_flagm!(self, bus, direct_indirect_long_y, sta, inc_addr),
            0x98 => self.tya(),
            0x99 => op_case_flagm!(self, bus, absolute_y, sta, inc_addr),
            0x9A => self.txs(),
            0x9B => self.txy(),
            0x9C => op_case_flagm!(self, bus, absolute, stz, inc_addr),
            0x9D => op_case_flagm!(self, bus, absolute_x, sta, inc_addr),
            0x9E => op_case_flagm!(self, bus, absolute_x, stz, inc_addr),
            0x9F => op_case_flagm!(self, bus, long_x, sta, inc_addr),
            0xA0 => op_case_flagx_imm!(self, bus, ldy),
            0xA1 => op_case_flagm!(self, bus, direct_x_indirect, lda, inc_addr),
            0xA2 => op_case_flagx_imm!(self, bus, ldx),
            0xA3 => op_case_flagm!(self, bus, stack_relative, lda, inc_addr16),
            0xA4 => op_case_flagx!(self, bus, direct, ldy, inc_addr16),
            0xA5 => op_case_flagm!(self, bus, direct, lda, inc_addr16),
            0xA6 => op_case_flagx!(self, bus, direct, ldx, inc_addr16),
            0xA7 => op_case_flagm!(self, bus, direct_indirect_long, lda, inc_addr),
            0xA8 => self.tay(),
            0xA9 => op_case_flagm_imm!(self, bus, lda),
            0xAA => self.tax(),
            0xAB => self.plb(bus),
            0xAC => op_case_flagx!(self, bus, absolute, ldy, inc_addr),
            0xAD => op_case_flagm!(self, bus, absolute, lda, inc_addr),
            0xAE => op_case_flagx!(self, bus, absolute, ldx, inc_addr),
            0xAF => op_case_flagm!(self, bus, long, lda, inc_addr),
            0xB0 => op_case_br!(self, bus, relative, bcs),
            0xB1 => op_case_flagm!(self, bus, direct_indirect_y, lda, inc_addr),
            0xB2 => op_case_flagm!(self, bus, direct_indirect, lda, inc_addr),
            0xB3 => op_case_flagm!(self, bus, stack_relative_indirect_y, lda, inc_addr),
            0xB4 => op_case_flagx!(self, bus, direct_x, ldy, inc_addr16),
            0xB5 => op_case_flagm!(self, bus, direct_x, lda, inc_addr16),
            0xB6 => op_case_flagx!(self, bus, direct_y, ldx, inc_addr16),
            0xB7 => op_case_flagm!(self, bus, direct_indirect_long_y, lda, inc_addr),
            0xB8 => self.clv(),
            0xB9 => op_case_flagm!(self, bus, absolute_y, lda, inc_addr),
            0xBA => self.tsx(),
            0xBB => self.tyx(),
            0xBC => op_case_flagx!(self, bus, absolute_x, ldy, inc_addr),
            0xBD => op_case_flagm!(self, bus, absolute_x, lda, inc_addr),
            0xBE => op_case_flagx!(self, bus, absolute_y, ldx, inc_addr),
            0xBF => op_case_flagm!(self, bus, long_x, lda, inc_addr),
            0xC0 => op_case_flagx_imm!(self, bus, cpy),
            0xC1 => op_case_flagm!(self, bus, direct_x_indirect, cmp, inc_addr),
            0xC2 => op_case_imm!(self, bus, rep),
            0xC3 => op_case_flagm!(self, bus, stack_relative, cmp, inc_addr16),
            0xC4 => op_case_flagx!(self, bus, direct, cpy, inc_addr16),
            0xC5 => op_case_flagm!(self, bus, direct, cmp, inc_addr16),
            0xC6 => op_case_flagm!(self, bus, direct, dec_mem, inc_addr16),
            0xC7 => op_case_flagm!(self, bus, direct_indirect_long, cmp, inc_addr),
            0xC8 => self.iny(),
            0xC9 => op_case_flagm_imm!(self, bus, cmp),
            0xCA => self.dex(),
            0xCB => self.wai(),
            0xCC => op_case_flagx!(self, bus, absolute, cpy, inc_addr),
            0xCD => op_case_flagm!(self, bus, absolute, cmp, inc_addr),
            0xCE => op_case_flagm!(self, bus, absolute, dec_mem, inc_addr),
            0xCF => op_case_flagm!(self, bus, long, cmp, inc_addr),
            0xD0 => op_case_br!(self, bus, relative, bne),
            0xD1 => op_case_flagm!(self, bus, direct_indirect_y, cmp, inc_addr),
            0xD2 => op_case_flagm!(self, bus, direct_indirect, cmp, inc_addr),
            0xD3 => op_case_flagm!(self, bus, stack_relative_indirect_y, cmp, inc_addr),
            0xD4 => op_case_long!(self, bus, direct, pex, inc_addr16),
            0xD5 => op_case_flagm!(self, bus, direct_x, cmp, inc_addr16),
            0xD6 => op_case_flagm!(self, bus, direct_x, dec_mem, inc_addr16),
            0xD7 => op_case_flagm!(self, bus, direct_indirect_long_y, cmp, inc_addr),
            0xD8 => self.cld(),
            0xD9 => op_case_flagm!(self, bus, absolute_y, cmp, inc_addr),
            0xDA => self.phx(bus),
            0xDB => self.stp(),
            0xDC => op_case_br!(self, bus, long_indirect, jmp),
            0xDD => op_case_flagm!(self, bus, absolute_x, cmp, inc_addr),
            0xDE => op_case_flagm!(self, bus, absolute_x, dec_mem, inc_addr),
            0xDF => op_case_flagm!(self, bus, long_x, cmp, inc_addr),
            0xE0 => op_case_flagx_imm!(self, bus, cpx),
            0xE1 => op_case_flagm!(self, bus, direct_x_indirect, sbc, inc_addr),
            0xE2 => op_case_imm!(self, bus, sep),
            0xE3 => op_case_flagm!(self, bus, stack_relative, sbc, inc_addr16),
            0xE4 => op_case_flagx!(self, bus, direct, cpx, inc_addr16),
            0xE5 => op_case_flagm!(self, bus, direct, sbc, inc_addr16),
            0xE6 => op_case_flagm!(self, bus, direct, inc_mem, inc_addr16),
            0xE7 => op_case_flagm!(self, bus, direct_indirect_long, sbc, inc_addr),
            0xE8 => self.inx(),
            0xE9 => op_case_flagm_imm!(self, bus, sbc),
            0xEA => self.nop(),
            0xEB => self.xba(),
            0xEC => op_case_flagx!(self, bus, absolute, cpx, inc_addr),
            0xED => op_case_flagm!(self, bus, absolute, sbc, inc_addr),
            0xEE => op_case_flagm!(self, bus, absolute, inc_mem, inc_addr),
            0xEF => op_case_flagm!(self, bus, long, sbc, inc_addr),
            0xF0 => op_case_br!(self, bus, relative, beq),
            0xF1 => op_case_flagm!(self, bus, direct_indirect_y, sbc, inc_addr),
            0xF2 => op_case_flagm!(self, bus, direct_indirect, sbc, inc_addr),
            0xF3 => op_case_flagm!(self, bus, stack_relative_indirect_y, sbc, inc_addr),
            0xF4 => op_case_long_imm!(self, bus, pex),
            0xF5 => op_case_flagm!(self, bus, direct_x, sbc, inc_addr16),
            0xF6 => op_case_flagm!(self, bus, direct_x, inc_mem, inc_addr16),
            0xF7 => op_case_flagm!(self, bus, direct_indirect_long_y, sbc, inc_addr),
            0xF8 => self.sed(),
            0xF9 => op_case_flagm!(self, bus, absolute_y, sbc, inc_addr),
            0xFA => self.plx(bus),
            0xFB => self.xce(),
            0xFC => op_case!(self, bus, absolute_x_indirect, jsr),
            0xFD => op_case_flagm!(self, bus, absolute_x, sbc, inc_addr),
            0xFE => op_case_flagm!(self, bus, absolute_x, inc_mem, inc_addr),
            0xFF => op_case_flagm!(self, bus, long_x, sbc, inc_addr),
        };
        
        if opcode == 0x20 {
            let ret_addr = self.pop_word(bus);
            self.sp -= 2;
            debug!("${:02X}{:04X}: JSR, Stk: {:04X}, depth: {}", self.pb, self.pc, ret_addr, self.debug_cnt);
            self.debug_cnt += 1;
        }
        
        if self.branch_taken {  
            self.clocks += Self::CYCLE_CLOCKS;

            if self.e {
                self.clocks += Self::CYCLE_CLOCKS;
            }
        }
        
        let extra_cycles = Self::EXTRA_CYCLES_LOOKUP[opcode as usize] as usize;
        
        self.clocks += Self::CYCLE_CLOCKS * extra_cycles;
    }
}

// Addressing Modes
impl Cpu65c816 {
    fn immediate(&mut self) -> Address {
        let offset = self.pc;
        self.pc += 1;        
        
        Address {
            bank: self.pb,
            offset,
        }
    }

    fn absolute(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let hi = self.read_prg(bus);
        
        Address {
            bank: self.db,
            offset: u16::from_le_bytes([lo, hi]),
        }
    }

    fn absolute_x(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let hi = self.read_prg(bus);
        
        Address::from_u32(
            u32::from_le_bytes([lo, hi, self.db, 0]) + self.x as u32
        )
    }

    fn absolute_y(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let hi = self.read_prg(bus);
        
        Address::from_u32(
            u32::from_le_bytes([lo, hi, self.db, 0]) + self.y as u32
        )
    }

    fn absolute_indirect(&mut self, bus: &mut CpuBus) -> Address {
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
    
    fn absolute_x_indirect(&mut self, bus: &mut CpuBus) -> Address {
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
        
    fn direct(&mut self, bus: &mut CpuBus) -> Address {
        Address {
            bank: 0,
            offset: self.dp + self.read_prg(bus) as u16,
        }
    }
        
    fn direct_x(&mut self, bus: &mut CpuBus) -> Address {
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
        
    fn direct_y(&mut self, bus: &mut CpuBus) -> Address {
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
        
    fn direct_indirect(&mut self, bus: &mut CpuBus) -> Address {
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
        
    fn direct_indirect_long(&mut self, bus: &mut CpuBus) -> Address {
        let data = self.read_prg(bus);

        let ptr_lo = self.dp + data as u16;
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
        
    fn direct_x_indirect(&mut self, bus: &mut CpuBus) -> Address {
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
        
    fn direct_indirect_y(&mut self, bus: &mut CpuBus) -> Address {
        Address::from_u32(
            self.direct_indirect(bus).to_u32() + self.y as u32
        )
    }
        
    fn direct_indirect_long_y(&mut self, bus: &mut CpuBus) -> Address {
        Address::from_u32(
            self.direct_indirect_long(bus).to_u32() + self.y as u32
        )
    }
        
    fn long(&mut self, bus: &mut CpuBus) -> Address {
        let lo = self.read_prg(bus);
        let mi = self.read_prg(bus);
        let hi = self.read_prg(bus);
        
        Address {
            bank: hi,
            offset: u16::from_le_bytes([lo, mi]),
        }
    }
        
    fn long_x(&mut self, bus: &mut CpuBus) -> Address {
        Address::from_u32(
            self.long(bus).to_u32() + self.x as u32
        )
    }
        
    fn long_indirect(&mut self, bus: &mut CpuBus) -> Address {
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
    
    fn asl(&mut self) {
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
    
    fn bcc(&mut self, addr: Address) {
        if !self.is_flag_set(Flag::FlagC) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bcs(&mut self, addr: Address) {
        if self.is_flag_set(Flag::FlagC) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn beq(&mut self, addr: Address) {
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

    fn bmi(&mut self, addr: Address) {
        if self.is_flag_set(Flag::FlagN) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bne(&mut self, addr: Address) {
        if !self.is_flag_set(Flag::FlagZ) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bpl(&mut self, addr: Address) {
        if !self.is_flag_set(Flag::FlagN) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bra(&mut self, addr: Address) {
        self.pc = addr.offset;
        self.branch_taken = true;
    }

    fn brk(&mut self, bus: &mut CpuBus) {   
        self.pc += 1; // Push address of next instruction

        self.handle_interrupt(bus, CpuInterrupt::BRK);
    }

    fn bvc(&mut self, addr: Address) {
        if !self.is_flag_set(Flag::FlagV) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }

    fn bvs(&mut self, addr: Address) {
        if self.is_flag_set(Flag::FlagV) {
            self.pc = addr.offset;
            self.branch_taken = true;
        }
    }
    
    fn clc(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, false);
    }

    fn cld(&mut self) {
        self.set_flag_to_bool(Flag::FlagD, false);
    }

    fn cli(&mut self) {
        self.set_flag_to_bool(Flag::FlagI, false);
    }

    fn clv(&mut self) {
        self.set_flag_to_bool(Flag::FlagV, false);
    }

    fn cmp_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        cmp_reg8!(self, self.a, bus, addr);
    }
    
    fn cmp_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        cmp_reg16!(self, self.a, bus, addr_lo, addr_hi);
    }

    fn cop(&mut self, bus: &mut CpuBus, addr: Address) {
        let _ = self.read(bus, addr); // read is discarded here

        self.handle_interrupt(bus, CpuInterrupt::COP);
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
    
    fn dec(&mut self) {
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

    fn dex(&mut self) {
        dec_idx!(self, self.x);
    }

    fn dey(&mut self) {
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
    
    fn inc(&mut self) {
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

    fn inx(&mut self) {
        inc_idx!(self, self.x);
    }

    fn iny(&mut self) {
        inc_idx!(self, self.y);
    }
    
    fn jmp(&mut self, addr: Address) {
        self.pc = addr.offset;
    }

    fn jmp_long(&mut self, addr: Address) {
        self.pb = addr.bank;
        self.pc = addr.offset;
    }

    fn jsr(&mut self, bus: &mut CpuBus, addr: Address) {
        self.push_word(bus, self.pc - 1);
        self.pc = addr.offset;
    }

    fn jsl(&mut self, bus: &mut CpuBus, addr: Address) {
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

    fn lsr(&mut self) {
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

    fn mvn(&mut self, bus: &mut CpuBus, src_addr: Address, dst_addr: Address) {
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

    fn mvp(&mut self, bus: &mut CpuBus, src_addr: Address, dst_addr: Address) {
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
    
    fn nop(&mut self) {
    }
    
    fn ora_m8(&mut self, bus: &mut CpuBus, address: Address) {
        self.a |= self.read(bus, address) as u16;
        set_nz8!(self, self.a);
    }
    
    fn ora_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.a |= self.read_word(bus, addr_lo, addr_hi) as u16;
        set_nz16!(self, self.a);
    }
    
    fn pex(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let data = self.read_word(bus, addr_lo, addr_hi);
        self.push_word(bus, data);
    }

    fn per(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        let offset = self.read_word(bus, addr_lo, addr_hi);
        self.push_word(bus, self.pc + offset);
    }

    fn pha(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagM) {
            self.push(bus, self.a as u8);
        } else {
            self.push_word(bus, self.a);
        }
    }

    fn phb(&mut self, bus: &mut CpuBus) {
        self.push(bus, self.db);
    }

    fn phd(&mut self, bus: &mut CpuBus) {
        self.push_word(bus, self.dp);
    }

    fn phk(&mut self, bus: &mut CpuBus) {
        self.push(bus, self.pb);
    }

    fn php(&mut self, bus: &mut CpuBus) {
        self.push(bus, self.p);
    }

    fn phx(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagX) {
            self.push(bus, self.x as u8);
        } else {
            self.push_word(bus, self.x);
        }
    }

    fn phy(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagX) {
            self.push(bus, self.y as u8);
        } else {
            self.push_word(bus, self.y);
        }
    }

    fn pla(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagM) {
            set_byte_n!(self.a, self.pop(bus) as u16, 0);
            set_nz8!(self, self.a);
        } else {
            self.a = self.pop_word(bus);
            set_nz16!(self, self.a);
        }
    }

    fn plb(&mut self, bus: &mut CpuBus) {
        self.db = self.pop(bus);
        set_nz8!(self, self.db);
    }

    fn pld(&mut self, bus: &mut CpuBus) {
        self.dp = self.pop_word(bus);
        set_nz16!(self, self.dp);
    }

    fn plp(&mut self, bus: &mut CpuBus) {
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

    fn plx(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagX) {
            self.x = self.pop(bus) as u16;
            set_nz8!(self, self.x);
        } else {
            self.x = self.pop_word(bus);
            set_nz16!(self, self.x);
        }
    }

    fn ply(&mut self, bus: &mut CpuBus) {
        if self.is_flag_set(Flag::FlagM) {
            self.y = self.pop(bus) as u16;
            set_nz8!(self, self.y);
        } else {
            self.y = self.pop_word(bus);
            set_nz16!(self, self.y);
        }
    }
    
    fn rep(&mut self, bus: &mut CpuBus, addr: Address) {
        self.p &= !self.read(bus, addr);
        
        if self.e {
            self.set_flag_to_bool(Flag::FlagM, true);
            self.set_flag_to_bool(Flag::FlagX, true);
        }
    }

    fn rol(&mut self) {
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

    fn ror(&mut self) {
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

    fn rti(&mut self, bus: &mut CpuBus) {
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

    fn rtl(&mut self, bus: &mut CpuBus) {
        self.pc = self.pop_word(bus) + 1;
        self.pb = self.pop(bus);
    }

    fn rts(&mut self, bus: &mut CpuBus) {
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

    fn sec(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, true);
    }

    fn sed(&mut self) {
        self.set_flag_to_bool(Flag::FlagD, true);
    }

    fn sei(&mut self) {
        self.set_flag_to_bool(Flag::FlagI, true);
    }

    fn sep(&mut self, bus: &mut CpuBus, addr: Address) {
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

    fn stp(&mut self) {
        self.stopped = true;
    }

    fn stx_x8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.write(bus, addr, self.x as u8);
    }
    
    fn stx_x16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.write_word(bus, addr_lo, addr_hi, self.x)
    }
    
    fn sty_x8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.write(bus, addr, self.y as u8);
    }
    
    fn sty_x16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.write_word(bus, addr_lo, addr_hi, self.y)
    }

    fn stz_m8(&mut self, bus: &mut CpuBus, addr: Address) {
        self.write(bus, addr, 0);
    }
    
    fn stz_m16(&mut self, bus: &mut CpuBus, addr_lo: Address, addr_hi: Address) {
        self.write_word(bus, addr_lo, addr_hi, 0)
    }
    
    fn tax(&mut self) {
        transfer_reg!(self, self.a, self.x, Flag::FlagX);
    }

    fn tay(&mut self) {
        transfer_reg!(self, self.a, self.y, Flag::FlagX);
    }

    fn tcd(&mut self) {
        self.dp = self.a;
        set_nz16!(self, self.dp);
    }

    fn tcs(&mut self) {
        if self.e {
            self.sp = 0x100 | (self.a & 0xFF);
        } else {
            self.sp = self.a;
        }
    }

    fn tdc(&mut self) {
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

    fn tsc(&mut self) {
        self.a = self.sp;
        set_nz16!(self, self.a);
    }

    fn tsx(&mut self) {
        transfer_reg!(self, self.sp, self.x, Flag::FlagX);
    }

    fn txa(&mut self) {
        transfer_reg!(self, self.x, self.a, Flag::FlagM);
    }

    fn txs(&mut self) {
        if self.e {
            self.sp = 0x100 | (self.x & 0xFF);
        } else {
            self.sp = self.x;
        }
    }

    fn txy(&mut self) {
        transfer_reg!(self, self.x, self.y, Flag::FlagX);
    }

    fn tya(&mut self) {
        transfer_reg!(self, self.y, self.a, Flag::FlagM);
    }

    fn tyx(&mut self) {
        transfer_reg!(self, self.y, self.x, Flag::FlagX);
    }
    
    fn wai(&mut self) {
        self.waiting_for_interrupt = true;
    }

    fn wdm(&mut self, bus: &mut CpuBus, addr: Address) {
        let _ = self.read(bus, addr);
    }

    fn xba(&mut self) {
        self.a = self.a.swap_bytes();
        set_nz8!(self, self.a); // Flags are always based on the low byte of A for this instr
    }

    fn xce(&mut self) {
        self.e = !self.e;
        
        if self.e {
            self.x &= 0xFF;
            self.y &= 0xFF;
            self.sp = 0x100 | (self.sp & 0xFF);
        }
    }
}