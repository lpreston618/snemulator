
use crate::core::ssmp::spc::Flag;
use crate::core::ssmp::spc::Spc700;
use crate::core::ssmp::spc::bus::SpcBus;
use crate::{get_bit_n, get_byte_n};

// Flag functions
impl Spc700 {
    pub fn exec_instr(&mut self, bus: &mut SpcBus) {
        let clocks: usize;
        let opcode = self.read_prg(bus);
        self.branch_taken = false;
    
        match opcode {
            0x00 => {
                self.nop();
                clocks = 2;
            }
            0x01 => {
                self.tcall(bus, 0xFFDE);
                clocks = 8;
            }
            0x02 => {
                let addr = self.direct(bus);
                self.set1(bus, addr, 0);
                clocks = 4;
            }
            0x03 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbs(bus, data_addr, branch_addr, 0);
                clocks = 5;
            }
            0x04 => {
                let addr = self.direct(bus);
                self.or_acc(bus, addr);
                clocks = 3;
            }
            0x05 => {
                let addr = self.absolute(bus);
                self.or_acc(bus, addr);
                clocks = 4;
            }
            0x06 => {
                let addr = self.indirect(bus);
                self.or_acc(bus, addr);
                clocks = 3;
            }
            0x07 => {
                let addr = self.x_indirect(bus);
                self.or_acc(bus, addr);
                clocks = 6;
            }
            0x08 => {
                let addr = self.immediate();
                self.or_acc(bus, addr);
                clocks = 2;
            }
            0x09 => {
                let (src_addr, dst_addr) = self.direct_to_direct(bus);
                self.or_mem(bus, src_addr, dst_addr);
                clocks = 6;
            }
            0x0A => {
                let (addr, bit) = self.absolute_bit(bus);
                self.or1(bus, addr, bit);
                clocks = 5;
            }
            0x0B => {
                let addr = self.direct(bus);
                self.asl_mem(bus, addr);
                clocks = 4;
            }
            0x0C => {
                let addr = self.absolute(bus);
                self.asl_mem(bus, addr);
                clocks = 5;
            }
            0x0D => {
                self.push_psw(bus);
                clocks = 4;
            }
            0x0E => {
                let addr = self.absolute(bus);
                self.tset1(bus, addr);
                clocks = 6;
            }
            0x0F => {
                self.brk(bus);
                clocks = 8;
            }
            0x10 => {
                let addr = self.relative(bus);
                self.bpl(addr);
                clocks = 2;
            }
            0x11 => {
                self.tcall(bus, 0xFFDC);
                clocks = 8;
            }
            0x12 => {
                let addr = self.direct(bus);
                self.clr1(bus, addr, 0);
                clocks = 4;
            }
            0x13 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbc(bus, data_addr, branch_addr, 0);
                clocks = 5;
            }
            0x14 => {
                let addr = self.x_direct(bus);
                self.or_acc(bus, addr);
                clocks = 4;
            }
            0x15 => {
                let addr = self.x_absolute(bus);
                self.or_acc(bus, addr);
                clocks = 5;
            }
            0x16 => {
                let addr = self.y_absolute(bus);
                self.or_acc(bus, addr);
                clocks = 5;
            }
            0x17 => {
                let addr = self.indirect_y(bus);
                self.or_acc(bus, addr);
                clocks = 6;
            }
            0x18 => {
                let (src_addr, dst_addr) = self.immediate_to_direct(bus);
                self.or_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x19 => {
                let (src_addr, dst_addr) = self.indirect_to_indirect(bus);
                self.or_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x1A => {
                let addr = self.direct(bus);
                self.decw(bus, addr);
                clocks = 6;
            }
            0x1B => {
                let addr = self.x_direct(bus);
                self.asl_mem(bus, addr);
                clocks = 5;
            }
            0x1C => {
                self.asl_acc();
                clocks = 2;
            }
            0x1D => {
                self.dex();
                clocks = 2;
            }
            0x1E => {
                let addr = self.absolute(bus);
                self.cmx(bus, addr);
                clocks = 4;
            }
            0x1F => {
                let addr = self.x_absolute_indirect(bus);
                self.jmp(addr);
                clocks = 6;
            }
            0x20 => {
                self.clrp();
                clocks = 2;
            }
            0x21 => {
                self.tcall(bus, 0xFFDA);
                clocks = 8;
            }
            0x22 => {
                let addr = self.direct(bus);
                self.set1(bus, addr, 1);
                clocks = 4;
            }
            0x23 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbs(bus, data_addr, branch_addr, 1);
                clocks = 5;
            }
            0x24 => {
                let addr = self.direct(bus);
                self.and_acc(bus, addr);
                clocks = 3;
            }
            0x25 => {
                let addr = self.absolute(bus);
                self.and_acc(bus, addr);
                clocks = 4;
            }
            0x26 => {
                let addr = self.indirect(bus);
                self.and_acc(bus, addr);
                clocks = 3;
            }
            0x27 => {
                let addr = self.x_indirect(bus);
                self.and_acc(bus, addr);
                clocks = 6;
            }
            0x28 => {
                let addr = self.immediate();
                self.and_acc(bus, addr);
                clocks = 2;
            }
            0x29 => {
                let (src_addr, dst_addr) = self.direct_to_direct(bus);
                self.and_mem(bus, src_addr, dst_addr);
                clocks = 6;
            }
            0x2A => {
                let (addr, bit) = self.absolute_bit(bus);
                self.or1_inv(bus, addr, bit);
                clocks = 5;
            }
            0x2B => {
                let addr = self.direct(bus);
                self.rol_mem(bus, addr);
                clocks = 4;
            }
            0x2C => {
                let addr = self.absolute(bus);
                self.rol_mem(bus, addr);
                clocks = 5;
            }
            0x2D => {
                self.push_acc(bus);
                clocks = 4;
            }
            0x2E => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.cbne(bus, data_addr, branch_addr);
                clocks = 5;
            }
            0x2F => {
                let addr = self.relative(bus);
                self.bra(addr);
                clocks = 4;
            }
            0x30 => {
                let addr = self.relative(bus);
                self.bmi(addr);
                clocks = 2;
            }
            0x31 => {
                self.tcall(bus, 0xFFD8);
                clocks = 8;
            }
            0x32 => {
                let addr = self.direct(bus);
                self.clr1(bus, addr, 1);
                clocks = 4;
            }
            0x33 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbc(bus, data_addr, branch_addr, 1);
                clocks = 5;
            }
            0x34 => {
                let addr = self.x_direct(bus);
                self.and_acc(bus, addr);
                clocks = 4;
            }
            0x35 => {
                let addr = self.x_absolute(bus);
                self.and_acc(bus, addr);
                clocks = 5;
            }
            0x36 => {
                let addr = self.y_absolute(bus);
                self.and_acc(bus, addr);
                clocks = 5;
            }
            0x37 => {
                let addr = self.indirect_y(bus);
                self.and_acc(bus, addr);
                clocks = 6;
            }
            0x38 => {
                let (src_addr, dst_addr) = self.immediate_to_direct(bus);
                self.and_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x39 => {
                let (src_addr, dst_addr) = self.indirect_to_indirect(bus);
                self.and_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x3A => {
                let addr = self.direct(bus);
                self.incw(bus, addr);
                clocks = 6;
            }
            0x3B => {
                let addr = self.x_direct(bus);
                self.rol_mem(bus, addr);
                clocks = 5;
            }
            0x3C => {
                self.rol_acc();
                clocks = 2;
            }
            0x3D => {
                self.inx();
                clocks = 2;
            }
            0x3E => {
                let addr = self.direct(bus);
                self.cmx(bus, addr);
                clocks = 3;
            }
            0x3F => {
                let addr = self.absolute(bus);
                self.call(bus, addr);
                clocks = 8;
            }
            0x40 => {
                self.setp();
                clocks = 2;
            }
            0x41 => {
                self.tcall(bus, 0xFFD6);
                clocks = 8;
            }
            0x42 => {
                let addr = self.direct(bus);
                self.set1(bus, addr, 2);
                clocks = 4;
            }
            0x43 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbs(bus, data_addr, branch_addr, 2);
                clocks = 5;
            }
            0x44 => {
                let addr = self.direct(bus);
                self.eor_acc(bus, addr);
                clocks = 3;
            }
            0x45 => {
                let addr = self.absolute(bus);
                self.eor_acc(bus, addr);
                clocks = 4;
            }
            0x46 => {
                let addr = self.indirect(bus);
                self.eor_acc(bus, addr);
                clocks = 3;
            }
            0x47 => {
                let addr = self.x_indirect(bus);
                self.eor_acc(bus, addr);
                clocks = 6;
            }
            0x48 => {
                let addr = self.immediate();
                self.eor_acc(bus, addr);
                clocks = 2;
            }
            0x49 => {
                let (src_addr, dst_addr) = self.direct_to_direct(bus);
                self.eor_mem(bus, src_addr, dst_addr);
                clocks = 6;
            }
            0x4A => {
                let (addr, bit) = self.absolute_bit(bus);
                self.and1(bus, addr, bit);
                clocks = 4;
            }
            0x4B => {
                let addr = self.direct(bus);
                self.lsr_mem(bus, addr);
                clocks = 4;
            }
            0x4C => {
                let addr = self.absolute(bus);
                self.lsr_mem(bus, addr);
                clocks = 5;
            }
            0x4D => {
                self.push_x(bus);
                clocks = 4;
            }
            0x4E => {
                let addr = self.absolute(bus);
                self.tclr1(bus, addr);
                clocks = 6;
            }
            0x4F => {
                let addr = self.immediate();
                self.pcall(bus, addr);
                clocks = 6;
            }
            0x50 => {
                let addr = self.relative(bus);
                self.bvc(addr);
                clocks = 2;
            }
            0x51 => {
                self.tcall(bus, 0xFFD4);
                clocks = 8;
            }
            0x52 => {
                let addr = self.direct(bus);
                self.clr1(bus, addr, 2);
                clocks = 4;
            }
            0x53 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbc(bus, data_addr, branch_addr, 2);
                clocks = 5;
            }
            0x54 => {
                let addr = self.x_direct(bus);
                self.eor_acc(bus, addr);
                clocks = 4;
            }
            0x55 => {
                let addr = self.x_absolute(bus);
                self.eor_acc(bus, addr);
                clocks = 5;
            }
            0x56 => {
                let addr = self.y_absolute(bus);
                self.eor_acc(bus, addr);
                clocks = 5;
            }
            0x57 => {
                let addr = self.indirect_y(bus);
                self.eor_acc(bus, addr);
                clocks = 6;
            }
            0x58 => {
                let (src_addr, dst_addr) = self.immediate_to_direct(bus);
                self.eor_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x59 => {
                let (src_addr, dst_addr) = self.indirect_to_indirect(bus);
                self.eor_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x5A => {
                let addr = self.direct(bus);
                self.cmpw(bus, addr);
                clocks = 4;
            }
            0x5B => {
                let addr = self.x_direct(bus);
                self.lsr_mem(bus, addr);
                clocks = 5;
            }
            0x5C => {
                self.lsr_acc();
                clocks = 2;
            }
            0x5D => {
                self.tax();
                clocks = 2;
            }
            0x5E => {
                let addr = self.absolute(bus);
                self.cmy(bus, addr);
                clocks = 4;
            }
            0x5F => {
                let addr = self.absolute(bus);
                self.jmp(addr);
                clocks = 3;
            }
            0x60 => {
                self.clrc();
                clocks = 2;
            }
            0x61 => {
                self.tcall(bus, 0xFFD2);
                clocks = 8;
            }
            0x62 => {
                let addr = self.direct(bus);
                self.set1(bus, addr, 3);
                clocks = 4;
            }
            0x63 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbs(bus, data_addr, branch_addr, 3);
                clocks = 5;
            }
            0x64 => {
                let addr = self.direct(bus);
                self.cmp_acc(bus, addr);
                clocks = 3;
            }
            0x65 => {
                let addr = self.absolute(bus);
                self.cmp_acc(bus, addr);
                clocks = 4;
            }
            0x66 => {
                let addr = self.indirect(bus);
                self.cmp_acc(bus, addr);
                clocks = 3;
            }
            0x67 => {
                let addr = self.x_indirect(bus);
                self.cmp_acc(bus, addr);
                clocks = 6;
            }
            0x68 => {
                let addr = self.immediate();
                self.cmp_acc(bus, addr);
                clocks = 2;
            }
            0x69 => {
                let (src_addr, dst_addr) = self.direct_to_direct(bus);
                self.cmp_mem(bus, src_addr, dst_addr);
                clocks = 6;
            }
            0x6A => {
                let (addr, bit) = self.absolute_bit(bus);
                self.and1_inv(bus, addr, bit);
                clocks = 4;
            }
            0x6B => {
                let addr = self.direct(bus);
                self.ror_mem(bus, addr);
                clocks = 4;
            }
            0x6C => {
                let addr = self.absolute(bus);
                self.ror_mem(bus, addr);
                clocks = 5;
            }
            0x6D => {
                self.push_y(bus);
                clocks = 4;
            }
            0x6E => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.dbnz_mem(bus, data_addr, branch_addr);
                clocks = 5;
            }
            0x6F => {
                self.ret(bus);
                clocks = 5;
            }
            0x70 => {
                let addr = self.relative(bus);
                self.bvs(addr);
                clocks = 2;
            }
            0x71 => {
                self.tcall(bus, 0xFFD0);
                clocks = 8;
            }
            0x72 => {
                let addr = self.direct(bus);
                self.clr1(bus, addr, 3);
                clocks = 4;
            }
            0x73 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbc(bus, data_addr, branch_addr, 3);
                clocks = 5;
            }
            0x74 => {
                let addr = self.x_direct(bus);
                self.cmp_acc(bus, addr);
                clocks = 4;
            }
            0x75 => {
                let addr = self.x_absolute(bus);
                self.cmp_acc(bus, addr);
                clocks = 5;
            }
            0x76 => {
                let addr = self.y_absolute(bus);
                self.cmp_acc(bus, addr);
                clocks = 5;
            }
            0x77 => {
                let addr = self.indirect_y(bus);
                self.cmp_acc(bus, addr);
                clocks = 6;
            }
            0x78 => {
                let (src_addr, dst_addr) = self.immediate_to_direct(bus);
                self.cmp_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x79 => {
                let (src_addr, dst_addr) = self.indirect_to_indirect(bus);
                self.cmp_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x7A => {
                let addr = self.direct(bus);
                self.addw(bus, addr);
                clocks = 5;
            }
            0x7B => {
                let addr = self.x_direct(bus);
                self.ror_mem(bus, addr);
                clocks = 5;
            }
            0x7C => {
                self.ror_acc();
                clocks = 2;
            }
            0x7D => {
                self.txa();
                clocks = 2;
            }
            0x7E => {
                let addr = self.direct(bus);
                self.cmy(bus, addr);
                clocks = 3;
            }
            0x7F => {
                self.ret1(bus);
                clocks = 6;
            }
            0x80 => {
                self.setc();
                clocks = 2;
            }
            0x81 => {
                self.tcall(bus, 0xFFCE);
                clocks = 8;
            }
            0x82 => {
                let addr = self.direct(bus);
                self.set1(bus, addr, 4);
                clocks = 4;
            }
            0x83 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbs(bus, data_addr, branch_addr, 4);
                clocks = 5;
            }
            0x84 => {
                let addr = self.direct(bus);
                self.adc_acc(bus, addr);
                clocks = 3;
            }
            0x85 => {
                let addr = self.absolute(bus);
                self.adc_acc(bus, addr);
                clocks = 4;
            }
            0x86 => {
                let addr = self.indirect(bus);
                self.adc_acc(bus, addr);
                clocks = 3;
            }
            0x87 => {
                let addr = self.x_indirect(bus);
                self.adc_acc(bus, addr);
                clocks = 6;
            }
            0x88 => {
                let addr = self.immediate();
                self.adc_acc(bus, addr);
                clocks = 2;
            }
            0x89 => {
                let (src_addr, dst_addr) = self.direct_to_direct(bus);
                self.adc_mem(bus, src_addr, dst_addr);
                clocks = 6;
            }
            0x8A => {
                let (addr, bit) = self.absolute_bit(bus);
                self.eor1(bus, addr, bit);
                clocks = 5;
            }
            0x8B => {
                let addr = self.direct(bus);
                self.dec_mem(bus, addr);
                clocks = 4;
            }
            0x8C => {
                let addr = self.absolute(bus);
                self.dec_mem(bus, addr);
                clocks = 5;
            }
            0x8D => {
                let addr = self.immediate();
                self.ldy(bus, addr);
                clocks = 2;
            }
            0x8E => {
                self.pop_psw(bus);
                clocks = 4;
            }
            0x8F => {
                let (src_addr, dst_addr) = self.immediate_to_direct(bus);
                self.mov(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x90 => {
                let addr = self.relative(bus);
                self.bcc(addr);
                clocks = 2;
            }
            0x91 => {
                self.tcall(bus, 0xFFCC);
                clocks = 8;
            }
            0x92 => {
                let addr = self.direct(bus);
                self.clr1(bus, addr, 4);
                clocks = 4;
            }
            0x93 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbc(bus, data_addr, branch_addr, 4);
                clocks = 5;
            }
            0x94 => {
                let addr = self.x_direct(bus);
                self.adc_acc(bus, addr);
                clocks = 4;
            }
            0x95 => {
                let addr = self.x_absolute(bus);
                self.adc_acc(bus, addr);
                clocks = 5;
            }
            0x96 => {
                let addr = self.y_absolute(bus);
                self.adc_acc(bus, addr);
                clocks = 5;
            }
            0x97 => {
                let addr = self.indirect_y(bus);
                self.adc_acc(bus, addr);
                clocks = 6;
            }
            0x98 => {
                let (src_addr, dst_addr) = self.immediate_to_direct(bus);
                self.adc_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x99 => {
                let (src_addr, dst_addr) = self.indirect_to_indirect(bus);
                self.adc_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0x9A => {
                let addr = self.direct(bus);
                self.subw(bus, addr);
                clocks = 5;
            }
            0x9B => {
                let addr = self.x_direct(bus);
                self.dec_mem(bus, addr);
                clocks = 5;
            }
            0x9C => {
                self.dec_acc();
                clocks = 2;
            }
            0x9D => {
                self.tsx();
                clocks = 2;
            }
            0x9E => {
                self.div();
                clocks = 12;
            }
            0x9F => {
                self.xcn();
                clocks = 5;
            }
            0xA0 => {
                self.sei();
                clocks = 3;
            }
            0xA1 => {
                self.tcall(bus, 0xFFCA);
                clocks = 8;
            }
            0xA2 => {
                let addr = self.direct(bus);
                self.set1(bus, addr, 5);
                clocks = 4;
            }
            0xA3 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbs(bus, data_addr, branch_addr, 5);
                clocks = 5;
            }
            0xA4 => {
                let addr = self.direct(bus);
                self.sbc_acc(bus, addr);
                clocks = 3;
            }
            0xA5 => {
                let addr = self.absolute(bus);
                self.sbc_acc(bus, addr);
                clocks = 4;
            }
            0xA6 => {
                let addr = self.indirect(bus);
                self.sbc_acc(bus, addr);
                clocks = 3;
            }
            0xA7 => {
                let addr = self.x_indirect(bus);
                self.sbc_acc(bus, addr);
                clocks = 6;
            }
            0xA8 => {
                let addr = self.immediate();
                self.sbc_acc(bus, addr);
                clocks = 2;
            }
            0xA9 => {
                let (src_addr, dst_addr) = self.direct_to_direct(bus);
                self.sbc_mem(bus, src_addr, dst_addr);
                clocks = 6;
            }
            0xAA => {
                let (addr, bit) = self.absolute_bit(bus);
                self.ldc(bus, addr, bit);
                clocks = 4;
            }
            0xAB => {
                let addr = self.direct(bus);
                self.inc_mem(bus, addr);
                clocks = 4;
            }
            0xAC => {
                let addr = self.absolute(bus);
                self.inc_mem(bus, addr);
                clocks = 5;
            }
            0xAD => {
                let addr = self.immediate();
                self.cmy(bus, addr);
                clocks = 2;
            }
            0xAE => {
                self.pop_acc(bus);
                clocks = 4;
            }
            0xAF => {
                let addr = self.indirect_inc(bus);
                self.sta(bus, addr);
                clocks = 4;
            }
            0xB0 => {
                let addr = self.relative(bus);
                self.bcs(addr);
                clocks = 2;
            }
            0xB1 => {
                self.tcall(bus, 0xFFC8);
                clocks = 8;
            }
            0xB2 => {
                let addr = self.direct(bus);
                self.clr1(bus, addr, 5);
                clocks = 4;
            }
            0xB3 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbc(bus, data_addr, branch_addr, 5);
                clocks = 5;
            }
            0xB4 => {
                let addr = self.x_direct(bus);
                self.sbc_acc(bus, addr);
                clocks = 4;
            }
            0xB5 => {
                let addr = self.x_absolute(bus);
                self.sbc_acc(bus, addr);
                clocks = 5;
            }
            0xB6 => {
                let addr = self.y_absolute(bus);
                self.sbc_acc(bus, addr);
                clocks = 5;
            }
            0xB7 => {
                let addr = self.indirect_y(bus);
                self.sbc_acc(bus, addr);
                clocks = 6;
            }
            0xB8 => {
                let (src_addr, dst_addr) = self.immediate_to_direct(bus);
                self.sbc_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0xB9 => {
                let (src_addr, dst_addr) = self.indirect_to_indirect(bus);
                self.sbc_mem(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0xBA => {
                let addr = self.direct(bus);
                self.ldya(bus, addr);
                clocks = 5;
            }
            0xBB => {
                let addr = self.x_direct(bus);
                self.inc_mem(bus, addr);
                clocks = 5;
            }
            0xBC => {
                self.inc_acc();
                clocks = 2;
            }
            0xBD => {
                self.txs();
                clocks = 2;
            }
            0xBE => {
                self.das();
                clocks = 3;
            }
            0xBF => {
                let addr = self.indirect_inc(bus);
                self.lda(bus, addr);
                clocks = 4;
            }
            0xC0 => {
                self.cli();
                clocks = 3;
            }
            0xC1 => {
                self.tcall(bus, 0xFFC6);
                clocks = 8;
            }
            0xC2 => {
                let addr = self.direct(bus);
                self.set1(bus, addr, 6);
                clocks = 4;
            }
            0xC3 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbs(bus, data_addr, branch_addr, 6);
                clocks = 5;
            }
            0xC4 => {
                let addr = self.direct(bus);
                self.sta(bus, addr);
                clocks = 4;
            }
            0xC5 => {
                let addr = self.absolute(bus);
                self.sta(bus, addr);
                clocks = 5;
            }
            0xC6 => {
                let addr = self.indirect(bus);
                self.sta(bus, addr);
                clocks = 4;
            }
            0xC7 => {
                let addr = self.x_indirect(bus);
                self.sta(bus, addr);
                clocks = 7;
            }
            0xC8 => {
                let addr = self.immediate();
                self.cmx(bus, addr);
                clocks = 2;
            }
            0xC9 => {
                let addr = self.absolute(bus);
                self.stx(bus, addr);
                clocks = 5;
            }
            0xCA => {
                let (addr, bit) = self.absolute_bit(bus);
                self.stc(bus, addr, bit);
                clocks = 6;
            }
            0xCB => {
                let addr = self.direct(bus);
                self.sty(bus, addr);
                clocks = 4;
            }
            0xCC => {
                let addr = self.absolute(bus);
                self.sty(bus, addr);
                clocks = 5;
            }
            0xCD => {
                let addr = self.immediate();
                self.ldx(bus, addr);
                clocks = 2;
            }
            0xCE => {
                self.pop_x(bus);
                clocks = 4;
            }
            0xCF => {
                self.mul();
                clocks = 9;
            }
            0xD0 => {
                let addr = self.relative(bus);
                self.bne(addr);
                clocks = 2;
            }
            0xD1 => {
                self.tcall(bus, 0xFFC4);
                clocks = 8;
            }
            0xD2 => {
                let addr = self.direct(bus);
                self.clr1(bus, addr, 6);
                clocks = 4;
            }
            0xD3 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbc(bus, data_addr, branch_addr, 6);
                clocks = 5;
            }
            0xD4 => {
                let addr = self.x_direct(bus);
                self.sta(bus, addr);
                clocks = 5;
            }
            0xD5 => {
                let addr = self.x_absolute(bus);
                self.sta(bus, addr);
                clocks = 6;
            }
            0xD6 => {
                let addr = self.y_absolute(bus);
                self.sta(bus, addr);
                clocks = 6;
            }
            0xD7 => {
                let addr = self.indirect_y(bus);
                self.sta(bus, addr);
                clocks = 7;
            }
            0xD8 => {
                let addr = self.direct(bus);
                self.stx(bus, addr);
                clocks = 4;
            }
            0xD9 => {
                let addr = self.y_direct(bus);
                self.stx(bus, addr);
                clocks = 5;
            }
            0xDA => {
                let addr = self.direct(bus);
                self.stya(bus, addr);
                clocks = 5;
            }
            0xDB => {
                let addr = self.x_direct(bus);
                self.sty(bus, addr);
                clocks = 5;
            }
            0xDC => {
                self.dey();
                clocks = 2;
            }
            0xDD => {
                self.tya();
                clocks = 2;
            }
            0xDE => {
                let (data_addr, branch_addr) = self.x_direct_relative(bus);
                self.cbne(bus, data_addr, branch_addr);
                clocks = 6;
            }
            0xDF => {
                self.daa();
                clocks = 3;
            }
            0xE0 => {
                self.clrv();
                clocks = 2;
            }
            0xE1 => {
                self.tcall(bus, 0xFFC2);
                clocks = 8;
            }
            0xE2 => {
                let addr = self.direct(bus);
                self.set1(bus, addr, 7);
                clocks = 4;
            }
            0xE3 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbs(bus, data_addr, branch_addr, 7);
                clocks = 5;
            }
            0xE4 => {
                let addr = self.direct(bus);
                self.lda(bus, addr);
                clocks = 3;
            }
            0xE5 => {
                let addr = self.absolute(bus);
                self.lda(bus, addr);
                clocks = 4;
            }
            0xE6 => {
                let addr = self.indirect(bus);
                self.lda(bus, addr);
                clocks = 3;
            }
            0xE7 => {
                let addr = self.x_indirect(bus);
                self.lda(bus, addr);
                clocks = 6;
            }
            0xE8 => {
                let addr = self.immediate();
                self.lda(bus, addr);
                clocks = 2;
            }
            0xE9 => {
                let addr = self.absolute(bus);
                self.ldx(bus, addr);
                clocks = 4;
            }
            0xEA => {
                let (addr, bit) = self.absolute_bit(bus);
                self.not1(bus, addr, bit);
                clocks = 5;
            }
            0xEB => {
                let addr = self.direct(bus);
                self.ldy(bus, addr);
                clocks = 3;
            }
            0xEC => {
                let addr = self.absolute(bus);
                self.ldy(bus, addr);
                clocks = 4;
            }
            0xED => {
                self.notc();
                clocks = 3;
            }
            0xEE => {
                self.pop_y(bus);
                clocks = 4;
            }
            0xEF => {
                self.sleep();
                clocks = 3;
            }
            0xF0 => {
                let addr = self.relative(bus);
                self.beq(addr);
                clocks = 2;
            }
            0xF1 => {
                self.tcall(bus, 0xFFC0);
                clocks = 8;
            }
            0xF2 => {
                let addr = self.direct(bus);
                self.clr1(bus, addr, 7);
                clocks = 4;
            }
            0xF3 => {
                let (data_addr, branch_addr) = self.direct_relative(bus);
                self.bbc(bus, data_addr, branch_addr, 7);
                clocks = 5;
            }
            0xF4 => {
                let addr = self.x_direct(bus);
                self.lda(bus, addr);
                clocks = 4;
            }
            0xF5 => {
                let addr = self.x_absolute(bus);
                self.lda(bus, addr);
                clocks = 5;
            }
            0xF6 => {
                let addr = self.y_absolute(bus);
                self.lda(bus, addr);
                clocks = 5;
            }
            0xF7 => {
                let addr = self.indirect_y(bus);
                self.lda(bus, addr);
                clocks = 6;
            }
            0xF8 => {
                let addr = self.direct(bus);
                self.ldx(bus, addr);
                clocks = 3;
            }
            0xF9 => {
                let addr = self.y_direct(bus);
                self.ldx(bus, addr);
                clocks = 4;
            }
            0xFA => {
                let (src_addr, dst_addr) = self.direct_to_direct(bus);
                self.mov(bus, src_addr, dst_addr);
                clocks = 5;
            }
            0xFB => {
                let addr = self.x_direct(bus);
                self.ldy(bus, addr);
                clocks = 4;
            }
            0xFC => {
                self.iny();
                clocks = 2;
            }
            0xFD => {
                self.tay();
                clocks = 2;
            }
            0xFE => {
                let addr = self.relative(bus);
                self.dbnz_y(addr);
                clocks = 4;
            }
            0xFF => {
                self.stop();
                clocks = 3;
            }
        }

        self.clocks += clocks;
    
        if self.branch_taken {
            self.clocks += 2;
        }
    }
    
    /// Reads the next byte of the program and increments PC
    fn read_prg(&mut self, bus: &mut SpcBus) -> u8 {
        let value = self.read(bus, self.pc);
        self.pc += 1;
        value
    }

    fn read(&mut self, bus: &mut SpcBus, addr: u16) -> u8 {
        bus.read(addr)
    }

    fn write(&mut self, bus: &mut SpcBus, addr: u16, value: u8) {
        bus.write(addr, value);
    }

    fn read_word_dp(&mut self, bus: &mut SpcBus, addr: u16) -> u16 {
        let addr2 = (addr & 0xFF00) | ((addr + 1) & 0xFF);
        
        u16::from_le_bytes([
            self.read(bus, addr),
            self.read(bus, addr2),
        ])
    }

    fn read_word(&mut self, bus: &mut SpcBus, addr: u16) -> u16 {
        u16::from_le_bytes([
            self.read(bus, addr),
            self.read(bus, addr + 1),
        ])
    }

    fn write_word(&mut self, bus: &mut SpcBus, addr: u16, value: u16) {
        let addr2 = (addr & 0xFF00) | ((addr + 1) & 0xFF);
        
        self.write(bus, addr, get_byte_n!(value, 0));
        self.write(bus, addr2, get_byte_n!(value, 1));
    }

    fn pop(&mut self, bus: &mut SpcBus) -> u8 {
        self.sp += 1;
        self.read(bus, 0x100 | self.sp as u16)
    }

    fn push(&mut self, bus: &mut SpcBus, value: u8) {
        self.write(bus, 0x100 | self.sp as u16, value);
        self.sp -= 1;
    }

    fn pop_word(&mut self, bus: &mut SpcBus) -> u16 {
        u16::from_le_bytes([
            self.pop(bus),
            self.pop(bus)
        ])
    }

    fn push_word(&mut self, bus: &mut SpcBus, value: u16) {
        self.push(bus, get_byte_n!(value, 1));
        self.push(bus, get_byte_n!(value, 0));
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
}

// Addressing Modes
impl Spc700 {
    fn immediate(&mut self) -> u16 {
        let addr = self.pc;
        self.pc += 1;
        addr
    }

    fn direct(&mut self, bus: &mut SpcBus) -> u16 {
        (self.read_prg(bus) as u16) | self.dir_page
    }

    fn x_direct(&mut self, bus: &mut SpcBus) -> u16 {
        ((self.read_prg(bus) + self.x) as u16) | self.dir_page
    }

    fn y_direct(&mut self, bus: &mut SpcBus) -> u16 {
        ((self.read_prg(bus) + self.y) as u16) | self.dir_page
    }

    fn indirect(&mut self, _bus: &mut SpcBus) -> u16 {
        (self.x as u16) | self.dir_page
    }

    fn indirect_inc(&mut self, bus: &mut SpcBus) -> u16 {
        let addr = self.indirect(bus);
        self.x += 1;
        addr
    }

    fn direct_to_direct(&mut self, bus: &mut SpcBus) -> (u16, u16) {
        let src_addr = self.direct(bus);
        let dst_addr = self.direct(bus);

        (src_addr, dst_addr)
    }

    fn indirect_to_indirect(&mut self, _bus: &mut SpcBus) -> (u16, u16) {
        let arg1_addr = (self.x as u16) | self.dir_page;
        let arg2_addr = (self.y as u16) | self.dir_page;

        (arg2_addr, arg1_addr)
    }

    fn immediate_to_direct(&mut self, bus: &mut SpcBus) -> (u16, u16) {
        let src_addr = self.immediate();
        let dst_addr = self.direct(bus);

        (src_addr, dst_addr)
    }

    fn direct_relative(&mut self, bus: &mut SpcBus) -> (u16, u16) {
        let data_addr = self.direct(bus);
        let branch_addr = self.relative(bus);

        (data_addr, branch_addr)
    }

    fn absolute(&mut self, bus: &mut SpcBus) -> u16 {
        u16::from_le_bytes([
            self.read_prg(bus),
            self.read_prg(bus),
        ])
    }

    fn absolute_bit(&mut self, bus: &mut SpcBus) -> (u16, u8) {
        let address = self.absolute(bus);

        (address & 0x1FFF, (address >> 13) as u8)
    }

    fn x_absolute(&mut self, bus: &mut SpcBus) -> u16 {
        self.absolute(bus) + (self.x as u16)
    }

    fn x_absolute_indirect(&mut self, bus: &mut SpcBus) -> u16 {
        let ptr_addr = self.x_absolute(bus);

        self.read_word(bus, ptr_addr)
    }

    fn y_absolute(&mut self, bus: &mut SpcBus) -> u16 {
        self.absolute(bus) + (self.y as u16)
    }

    fn x_direct_relative(&mut self, bus: &mut SpcBus) -> (u16, u16) {
        let data_addr = self.x_direct(bus);
        let branch_addr = self.relative(bus);

        (data_addr, branch_addr)
    }

    fn x_indirect(&mut self, bus: &mut SpcBus) -> u16 {
        let ptr_addr = self.x_direct(bus);

        self.read_word(bus, ptr_addr)
    }

    fn indirect_y(&mut self, bus: &mut SpcBus) -> u16 {
        let ptr_addr = self.direct(bus);

        self.read_word(bus, ptr_addr) + self.y as u16
    }

    fn relative(&mut self, bus: &mut SpcBus) -> u16 {
        let offset = ((self.read_prg(bus) as i8) as i16) as u16;

        self.pc + offset
    }
}

// CPU Instructions
impl Spc700 {
    fn add_16_base(&mut self, arg1: u16, arg2: u16) -> u16 {
        let result = (arg1 as u32) + (arg2 as u32);
        let half_result = (arg1 & 0x7FF) + (arg2 & 0x7FF);

        self.set_flag_to_bool(Flag::FlagC, result > 0xFFFF);
        self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
        self.set_flag_to_bool(Flag::FlagH, half_result > 0x7FF);
        self.set_flag_to_bool(Flag::FlagZ, result & 0xFFFF == 0);

        // Set V flag if acc and data are same sign, but result is different sign
        let a = get_bit_n!(arg1, 15);
        let d = get_bit_n!(arg2, 15);
        let r = get_bit_n!(result, 15);
        self.set_flag_to_bool(Flag::FlagV, !(a ^ d) & (a ^ r));

        result as u16
    }

    fn adc_base(&mut self, arg1: u8, arg2: u8, carry_in: bool) -> u8 {
        let result = (arg1 as u16) + (arg2 as u16) + if carry_in { 1 } else { 0 };
        let half_result = (arg1 & 0xF) + (arg2 & 0xF) + if carry_in { 1 } else { 0 };

        self.set_flag_to_bool(Flag::FlagC, result > 0xFF);
        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagH, half_result > 0xF);
        self.set_flag_to_bool(Flag::FlagZ, result & 0xFF == 0);

        // Set V flag if acc and data are same sign, but result is different sign
        let a = get_bit_n!(arg1, 7);
        let d = get_bit_n!(arg2, 7);
        let r = get_bit_n!(result, 7);
        self.set_flag_to_bool(Flag::FlagV, !(a ^ d) & (a ^ r));

        result as u8
    }

    fn adc_acc(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        self.a = self.adc_base(self.a, data, self.is_flag_set(Flag::FlagC));
    }

    fn adc_mem(&mut self, bus: &mut SpcBus, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);

        let result = self.adc_base(arg1, arg2, self.is_flag_set(Flag::FlagC));

        self.write(bus, addr2, result);
    }

    fn addw(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read_word_dp(bus, address);
        let ya = ((self.y as u16) << 8) | (self.a as u16);
        let result = self.add_16_base(ya, data);

        self.y = (result >> 8) as u8;
        self.a = result as u8;
    }

    // AND - AND Memory with Accumulator
    fn and_acc(&mut self, bus: &mut SpcBus, address: u16) {
        self.a &= self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn and_mem(&mut self, bus: &mut SpcBus, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let result = arg1 & arg2;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write(bus, addr2, result);
    }

    fn and1(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagC, self.is_flag_set(Flag::FlagC) && get_bit_n!(data, bit));
    }

    fn and1_inv(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagC, self.is_flag_set(Flag::FlagC) && get_bit_n!(!data, bit));
    }

    // ASL - Shift Left One Bit (Accumulator version)
    fn asl_acc(&mut self) {
        let result = self.a << 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 7));

        self.a = result;
    }

    // ASL - Shift Left One Bit (Memory version)
    fn asl_mem(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        let result = data << 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 7));

        self.write(bus, address, result);
    }

    // BBC - Branch if Bit Clear
    fn bbc(&mut self, bus: &mut SpcBus, data_addr: u16, branch_addr: u16, bit: u8) {
        let data = self.read(bus, data_addr);

        if get_bit_n!(!data, bit) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BBS - Branch if Bit Set
    fn bbs(&mut self, bus: &mut SpcBus, data_addr: u16, branch_addr: u16, bit: u8) {
        let data = self.read(bus, data_addr);

        if get_bit_n!(data, bit) {
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
    fn brk(&mut self, bus: &mut SpcBus) {
        const BRK_VECTOR: u16 = 0xFFDE;

        self.push_word(bus, self.pc);
        self.push(bus, self.status);

        self.pc = self.read_word(bus, BRK_VECTOR);

        self.clear_flag(Flag::FlagI);
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
    fn call(&mut self, bus: &mut SpcBus, new_addr: u16) {
        self.push_word(bus, self.pc);
        self.pc = new_addr;
    }

    // CBNE - Compare and Branch if Not Equal
    fn cbne(&mut self, bus: &mut SpcBus, address: u16, branch_addr: u16) {
        let data = self.read(bus, address);

        if self.a != data {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // CMP - Compare Memory with Accumulator
    fn cmp_acc(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        let result = self.a - data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, self.a >= data);
    }

    fn cmp_mem(&mut self, bus: &mut SpcBus, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let result = arg2 - arg1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, arg2 >= arg1);
    }

    // CLI - CLear Interrupt flag (called DI in SPC700 documentation)
    fn cli(&mut self) {
        self.clear_flag(Flag::FlagI);
    }

    // CLR1 - clears a single bit in the direct page
    fn clr1(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let b = 1 << bit;

        self.write(bus, address, data & !b);
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

    // CLRV - clear overflow flag (and half carry)
    fn clrv(&mut self) {
        self.clear_flag(Flag::FlagV);
        self.clear_flag(Flag::FlagH);
    }

    // CMPW - Compare Word with YA
    fn cmpw(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read_word_dp(bus, address);
        let ya = ((self.y as u16) << 8) | (self.a as u16);
        let result = ya - data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 15));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, ya >= data);
    }

    // CMX - Compare Memory with X
    fn cmx(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        let result = self.x - data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, self.x >= data);
    }

    // CMY - Compare Memory with Y
    fn cmy(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        let result = self.y - data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, self.y >= data);
    }

    // DAA - Decimal Adjust Addition
    fn daa(&mut self) {
        if self.is_flag_set(Flag::FlagC) || self.a >= 0x9A {
            self.a += 0x60;
            self.set_flag(Flag::FlagC);
        }
        if self.is_flag_set(Flag::FlagH) || (self.a & 0xF) >= 0xA {
            self.a += 0x6;
        }

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    // DAS - Decimal Adjust Subtraction
    fn das(&mut self) {
        if !self.is_flag_set(Flag::FlagC) || self.a >= 0x9A {
            self.a -= 0x60;
            self.clear_flag(Flag::FlagC);
        }
        if !self.is_flag_set(Flag::FlagH) || (self.a & 0xF) >= 0xA {
            self.a -= 0x6;
        }
        
        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    // DBNZ - Decrement and Branch if Not Zero (Y register)
    fn dbnz_y(&mut self, branch_addr: u16) {
        self.y -= 1;

        if self.y != 0 {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // DBNZ - Decrement and Branch if Not Zero (memory)
    fn dbnz_mem(&mut self, bus: &mut SpcBus, address: u16, branch_addr: u16) {
        let result = self.read(bus, address) - 1;
        self.write(bus, address, result);

        if result != 0 {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // DEC - decrement (accumulator)
    fn dec_acc(&mut self) {
        self.a -= 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    // DEC - decrement (memory)
    fn dec_mem(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address) - 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(data, 7));
        self.set_flag_to_bool(Flag::FlagZ, data == 0);

        self.write(bus, address, data);
    }
    
    fn decw(&mut self, bus: &mut SpcBus, address: u16) {
        let result = self.read_word_dp(bus, address) - 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 15));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write_word(bus, address, result);
    }

    fn dex(&mut self) {
        self.x -= 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn dey(&mut self) {
        self.y -= 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn div(&mut self) {
        let ya = ((self.y as u16) << 8) | (self.a as u16);

        self.set_flag_to_bool(Flag::FlagH, (self.y & 0xF) >= (self.x & 0xF));
        self.set_flag_to_bool(Flag::FlagV, self.y >= self.x);

        if (self.y as u16) < ((self.x as u16) << 1) {
            let div_result = ya / self.x as u16;
            let mod_result = ya % self.x as u16;

            self.a = div_result as u8;
            self.y = mod_result as u8;
        } else {
            self.a = (255 - (ya - ((self.x as u16) << 9)) / (256 - (self.x as u16))) as u8;
            self.y = ((self.x as u16) + (ya - ((self.x as u16) << 9)) % (256 - (self.x as u16))) as u8;
        }

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn eor_acc(&mut self, bus: &mut SpcBus, address: u16) {
        self.a ^= self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn eor_mem(&mut self, bus: &mut SpcBus, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let result = arg1 ^ arg2;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write(bus, addr2, result);
    }

    fn eor1(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let result = self.is_flag_set(Flag::FlagC) ^ get_bit_n!(data, bit);

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn inc_acc(&mut self) {
        self.a += 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn inc_mem(&mut self, bus: &mut SpcBus, address: u16) {
        let result = self.read(bus, address) + 1;

        self.write(bus, address, result);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn incw(&mut self, bus: &mut SpcBus, address: u16) {
        let result = self.read_word_dp(bus, address) + 1;

        self.write_word(bus, address, result);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 15));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn inx(&mut self) {
        self.x += 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn iny(&mut self) {
        self.y += 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    fn lda(&mut self, bus: &mut SpcBus, address: u16) {
        self.a = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn ldc(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, bit));
    }

    fn ldx(&mut self, bus: &mut SpcBus, address: u16) {
        self.x = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn ldy(&mut self, bus: &mut SpcBus, address: u16) {
        self.y = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn ldya(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read_word_dp(bus, address);

        self.y = (data >> 8) as u8;
        self.a = data as u8;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0 && self.a == 0);
    }

    fn lsr_acc(&mut self) {
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 0));

        self.a >>= 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn lsr_mem(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        let result = data >> 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 0));

        self.write(bus, address, result);
    }

    fn mov(&mut self, bus: &mut SpcBus, src_addr: u16, dst_addr: u16) {
        let data = self.read(bus, src_addr);

        self.write(bus, dst_addr, data);
    }

    fn mul(&mut self) {
        let result = (self.y as u16) * (self.a as u16);

        self.y = (result >> 8) as u8;
        self.a = result as u8;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn nop(&self) {}

    fn not1(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let b = 1 << bit;
        let result = data ^ b;

        self.write(bus, address, result);
    }

    fn notc(&mut self) {
        self.status ^= Flag::FlagC as u8;
    }

    fn or1(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let result = self.is_flag_set(Flag::FlagC) || get_bit_n!(data, bit);

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn or1_inv(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let result = self.is_flag_set(Flag::FlagC) || get_bit_n!(!data, bit);

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn or_acc(&mut self, bus: &mut SpcBus, address: u16) {
        self.a |= self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn or_mem(&mut self, bus: &mut SpcBus, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let result = arg1 | arg2;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write(bus, addr2, result);
    }

    fn pcall(&mut self, bus: &mut SpcBus, address: u16) {
        let call_addr = 0xFF00 | self.read(bus, address) as u16;

        self.call(bus, call_addr);
    }

    fn pop_acc(&mut self, bus: &mut SpcBus) {
        self.a = self.pop(bus);
    }

    fn pop_x(&mut self, bus: &mut SpcBus) {
        self.x = self.pop(bus);
    }

    fn pop_y(&mut self, bus: &mut SpcBus) {
        self.y = self.pop(bus);
    }

    fn pop_psw(&mut self, bus: &mut SpcBus) {
        self.status = self.pop(bus);

        if self.is_flag_set(Flag::FlagP) {
            self.dir_page = 0x100;
        } else {
            self.dir_page = 0;
        }
    }

    fn push_acc(&mut self, bus: &mut SpcBus) {
        self.push(bus, self.a);
    }

    fn push_x(&mut self, bus: &mut SpcBus) {
        self.push(bus, self.x);
    }

    fn push_y(&mut self, bus: &mut SpcBus) {
        self.push(bus, self.y);
    }

    fn push_psw(&mut self, bus: &mut SpcBus) {
        self.push(bus, self.status);
    }

    fn ret(&mut self, bus: &mut SpcBus) {
        self.pc = self.pop_word(bus);
    }

    fn ret1(&mut self, bus: &mut SpcBus) {
        self.status = self.pop(bus);
        self.pc = self.pop_word(bus);

        if self.is_flag_set(Flag::FlagP) {
            self.dir_page = 0x100;
        } else {
            self.dir_page = 0;
        }
    }

    fn rol_acc(&mut self) {
        let new_c = get_bit_n!(self.a, 7);
        
        self.a <<= 1;
        self.a |= if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        
        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
        self.set_flag_to_bool(Flag::FlagC, new_c);
    }

    fn rol_mem(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        let result = (data << 1) | if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 7));

        self.write(bus, address, result);
    }

    fn ror_acc(&mut self) {
        let new_c = get_bit_n!(self.a, 0);

        self.a >>= 1;
        self.a |= if self.is_flag_set(Flag::FlagC) {
            0x80
        } else {
            0
        };

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
        self.set_flag_to_bool(Flag::FlagC, new_c);
    }

    fn ror_mem(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        let result = (if self.is_flag_set(Flag::FlagC) {
            0x80
        } else {
            0
        }) | (data >> 1);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 0));

        self.write(bus, address, result);
    }

    fn sbc_acc(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);
        let comp = !data;

        self.a = self.adc_base(self.a, comp, self.is_flag_set(Flag::FlagC));
    }

    fn sbc_mem(&mut self, bus: &mut SpcBus, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let comp1 = !arg1;

        let result = self.adc_base(arg2, comp1, self.is_flag_set(Flag::FlagC));

        self.write(bus, addr2, result);
    }

    fn sei(&mut self) {
        self.set_flag(Flag::FlagI)
    }

    fn set1(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let b = 1 << bit;

        self.write(bus, address, data | b);
    }

    fn setc(&mut self) {
        self.set_flag(Flag::FlagC);
    }

    fn setp(&mut self) {
        self.set_flag(Flag::FlagP);
        self.dir_page = 0x100;
    }

    fn sleep(&self) {}

    fn sta(&mut self, bus: &mut SpcBus, address: u16) {
        self.write(bus, address, self.a);
    }

    // MOV1 alias
    fn stc(&mut self, bus: &mut SpcBus, address: u16, bit: u8) {
        if self.is_flag_set(Flag::FlagC) {
            self.set1(bus, address, bit);
        } else {
            self.clr1(bus, address, bit);
        }
    }

    fn stop(&mut self) { self.stopped = true; }

    fn stx(&mut self, bus: &mut SpcBus, address: u16) {
        self.write(bus, address, self.x);
    }

    fn sty(&mut self, bus: &mut SpcBus, address: u16) {
        self.write(bus, address, self.y);
    }

    fn stya(&mut self, bus: &mut SpcBus, address: u16) {
        let addr2 = (address & 0xFF00) | ((address + 1) & 0xFF);
        self.write(bus, address, self.a);
        self.write(bus, addr2, self.y);
    }

    fn subw(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read_word_dp(bus, address);
        let comp = !data + 1;
        let ya = ((self.y as u16) << 8) | (self.a as u16);
        let result = self.add_16_base(ya, comp);

        self.y = (result >> 8) as u8;
        self.a = result as u8;
    }

    fn tax(&mut self) {
        self.x = self.a;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn tay(&mut self) {
        self.y = self.a;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn tcall(&mut self, bus: &mut SpcBus, address: u16) {
        self.push_word(bus, self.pc);
        self.pc = self.read_word(bus, address);
    }

    fn tclr1(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!((self.a - data), 7));
        self.set_flag_to_bool(Flag::FlagZ, (self.a - data) == 0);
        
        self.write(bus, address, data & !self.a);
    }

    fn tset1(&mut self, bus: &mut SpcBus, address: u16) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!((self.a - data), 7));
        self.set_flag_to_bool(Flag::FlagZ, (self.a - data) == 0);
        
        self.write(bus, address, data | self.a);
    }

    fn tsx(&mut self) {
        self.x = self.sp;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn txa(&mut self) {
        self.a = self.x;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn txs(&mut self) {
        self.sp = self.x;
    }

    fn tya(&mut self) {
        self.a = self.y;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn xcn(&mut self) {
        self.a = (self.a >> 4) | (self.a << 4);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }
}