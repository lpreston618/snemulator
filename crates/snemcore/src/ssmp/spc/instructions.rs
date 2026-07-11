use crate::debug::DebugHarness;
use crate::ssmp::spc::Flag;
use crate::ssmp::spc::Spc700;
use crate::ssmp::spc::bus::SpcBus;
use crate::{get_bit_n, get_byte_n};

macro_rules! opcode {
    // implied: no address. Some of these instructions never touch the bus
    // (asl_acc, dex, clrp, sei, div, mul, xcn, das, daa, sleep, stop, etc.) —
    // give those a dummy `_bus: &SpcBus<H>` parameter so every call site
    // here looks the same.
    ($cpu:ident, $bus:ident, $instr:ident, implied, $num_clocks:expr) => {{
        $cpu.$instr($bus);
        $num_clocks
    }};

    // immediate() doesn't touch the bus to compute its "address" (it's just PC).
    ($cpu:ident, $bus:ident, $instr:ident, immediate, $num_clocks:expr) => {{
        let addr = $cpu.immediate();
        $cpu.$instr($bus, addr);
        $num_clocks
    }};

    // jmp / conditional branches: addr mode reads the bus, but the
    // instruction itself only needs the resulting address, not the bus.
    ($cpu:ident, $bus:ident, addr_no_bus, $instr:ident, $addr_mode:ident, $num_clocks:expr) => {{
        let addr = $cpu.$addr_mode($bus);
        $cpu.$instr(addr);
        $num_clocks
    }};

    // Two-address (tuple) shapes.
    ($cpu:ident, $bus:ident, $instr:ident, direct_to_direct, $num_clocks:expr) => {{
        let (src_addr, dst_addr) = $cpu.direct_to_direct($bus);
        $cpu.$instr($bus, src_addr, dst_addr);
        $num_clocks
    }};
    ($cpu:ident, $bus:ident, $instr:ident, immediate_to_direct, $num_clocks:expr) => {{
        let (src_addr, dst_addr) = $cpu.immediate_to_direct($bus);
        $cpu.$instr($bus, src_addr, dst_addr);
        $num_clocks
    }};
    ($cpu:ident, $bus:ident, $instr:ident, indirect_to_indirect, $num_clocks:expr) => {{
        let (src_addr, dst_addr) = $cpu.indirect_to_indirect($bus);
        $cpu.$instr($bus, src_addr, dst_addr);
        $num_clocks
    }};
    ($cpu:ident, $bus:ident, $instr:ident, direct_relative, $num_clocks:expr) => {{
        let (data_addr, branch_addr) = $cpu.direct_relative($bus);
        $cpu.$instr($bus, data_addr, branch_addr);
        $num_clocks
    }};
    ($cpu:ident, $bus:ident, $instr:ident, x_direct_relative, $num_clocks:expr) => {{
        let (data_addr, branch_addr) = $cpu.x_direct_relative($bus);
        $cpu.$instr($bus, data_addr, branch_addr);
        $num_clocks
    }};

    // Address + bit shapes.
    ($cpu:ident, $bus:ident, $instr:ident, absolute_bit, $num_clocks:expr) => {{
        let (addr, bit) = $cpu.absolute_bit($bus);
        $cpu.$instr($bus, addr, bit);
        $num_clocks
    }};
    ($cpu:ident, $bus:ident, bit_op, $instr:ident, $addr_mode:ident, $bit:expr, $num_clocks:expr) => {{
        let addr = $cpu.$addr_mode($bus);
        $cpu.$instr($bus, addr, $bit);
        $num_clocks
    }};
    ($cpu:ident, $bus:ident, relative_bit_op, $instr:ident, $bit:expr, $num_clocks:expr) => {{
        let (data_addr, branch_addr) = $cpu.direct_relative($bus);
        $cpu.$instr($bus, data_addr, branch_addr, $bit);
        $num_clocks
    }};

    // Fixed-vector shape.
    ($cpu:ident, $bus:ident, tcall, $addr:expr, $num_clocks:expr) => {{
        $cpu.tcall($bus, $addr);
        $num_clocks
    }};

    // Fallback: addr mode reads the bus to produce an address,
    // instruction takes (bus, addr). Covers the majority of opcodes.
    ($cpu:ident, $bus:ident, $instr:ident, $addr_mode:ident, $num_clocks:expr) => {{
        let addr = $cpu.$addr_mode($bus);
        $cpu.$instr($bus, addr);
        $num_clocks
    }};
}

// Flag functions
impl Spc700 {
    pub fn exec_instr<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        if H::IS_DEBUGGING_HARNESS && H::TRACK_SPC_INSTRUCTIONS {
            self.prg_bytes.clear();
        }

        let opcode = self.read_prg(bus);
        self.branch_taken = false;

        let clocks = match opcode {
            0x00 => opcode!(self, bus, nop, implied, 2),
            0x01 => opcode!(self, bus, tcall, 0xFFDE, 8),
            0x02 => opcode!(self, bus, bit_op, set1, direct, 0, 4),
            0x03 => opcode!(self, bus, relative_bit_op, bbs, 0, 5),
            0x04 => opcode!(self, bus, or_acc, direct, 3),
            0x05 => opcode!(self, bus, or_acc, absolute, 4),
            0x06 => opcode!(self, bus, or_acc, indirect, 3),
            0x07 => opcode!(self, bus, or_acc, x_indirect, 6),
            0x08 => opcode!(self, bus, or_acc, immediate, 2),
            0x09 => opcode!(self, bus, or_mem, direct_to_direct, 6),
            0x0A => opcode!(self, bus, or1, absolute_bit, 5),
            0x0B => opcode!(self, bus, asl_mem, direct, 4),
            0x0C => opcode!(self, bus, asl_mem, absolute, 5),
            0x0D => opcode!(self, bus, push_psw, implied, 4),
            0x0E => opcode!(self, bus, tset1, absolute, 6),
            0x0F => opcode!(self, bus, brk, implied, 8),
            0x10 => opcode!(self, bus, addr_no_bus, bpl, relative, 2),
            0x11 => opcode!(self, bus, tcall, 0xFFDC, 8),
            0x12 => opcode!(self, bus, bit_op, clr1, direct, 0, 4),
            0x13 => opcode!(self, bus, relative_bit_op, bbc, 0, 5),
            0x14 => opcode!(self, bus, or_acc, x_direct, 4),
            0x15 => opcode!(self, bus, or_acc, x_absolute, 5),
            0x16 => opcode!(self, bus, or_acc, y_absolute, 5),
            0x17 => opcode!(self, bus, or_acc, indirect_y, 6),
            0x18 => opcode!(self, bus, or_mem, immediate_to_direct, 5),
            0x19 => opcode!(self, bus, or_mem, indirect_to_indirect, 5),
            0x1A => opcode!(self, bus, decw, direct, 6),
            0x1B => opcode!(self, bus, asl_mem, x_direct, 5),
            0x1C => opcode!(self, bus, asl_acc, implied, 2),
            0x1D => opcode!(self, bus, dex, implied, 2),
            0x1E => opcode!(self, bus, cmx, absolute, 4),
            0x1F => opcode!(self, bus, addr_no_bus, jmp, x_absolute_indirect, 6),
            0x20 => opcode!(self, bus, clrp, implied, 2),
            0x21 => opcode!(self, bus, tcall, 0xFFDA, 8),
            0x22 => opcode!(self, bus, bit_op, set1, direct, 1, 4),
            0x23 => opcode!(self, bus, relative_bit_op, bbs, 1, 5),
            0x24 => opcode!(self, bus, and_acc, direct, 3),
            0x25 => opcode!(self, bus, and_acc, absolute, 4),
            0x26 => opcode!(self, bus, and_acc, indirect, 3),
            0x27 => opcode!(self, bus, and_acc, x_indirect, 6),
            0x28 => opcode!(self, bus, and_acc, immediate, 2),
            0x29 => opcode!(self, bus, and_mem, direct_to_direct, 6),
            0x2A => opcode!(self, bus, or1_inv, absolute_bit, 5),
            0x2B => opcode!(self, bus, rol_mem, direct, 4),
            0x2C => opcode!(self, bus, rol_mem, absolute, 5),
            0x2D => opcode!(self, bus, push_acc, implied, 4),
            0x2E => opcode!(self, bus, cbne, direct_relative, 5),
            0x2F => opcode!(self, bus, addr_no_bus, bra, relative, 4),
            0x30 => opcode!(self, bus, addr_no_bus, bmi, relative, 2),
            0x31 => opcode!(self, bus, tcall, 0xFFD8, 8),
            0x32 => opcode!(self, bus, bit_op, clr1, direct, 1, 4),
            0x33 => opcode!(self, bus, relative_bit_op, bbc, 1, 5),
            0x34 => opcode!(self, bus, and_acc, x_direct, 4),
            0x35 => opcode!(self, bus, and_acc, x_absolute, 5),
            0x36 => opcode!(self, bus, and_acc, y_absolute, 5),
            0x37 => opcode!(self, bus, and_acc, indirect_y, 6),
            0x38 => opcode!(self, bus, and_mem, immediate_to_direct, 5),
            0x39 => opcode!(self, bus, and_mem, indirect_to_indirect, 5),
            0x3A => opcode!(self, bus, incw, direct, 6),
            0x3B => opcode!(self, bus, rol_mem, x_direct, 5),
            0x3C => opcode!(self, bus, rol_acc, implied, 2),
            0x3D => opcode!(self, bus, inx, implied, 2),
            0x3E => opcode!(self, bus, cmx, direct, 3),
            0x3F => opcode!(self, bus, call, absolute, 8),
            0x40 => opcode!(self, bus, setp, implied, 2),
            0x41 => opcode!(self, bus, tcall, 0xFFD6, 8),
            0x42 => opcode!(self, bus, bit_op, set1, direct, 2, 4),
            0x43 => opcode!(self, bus, relative_bit_op, bbs, 2, 5),
            0x44 => opcode!(self, bus, eor_acc, direct, 3),
            0x45 => opcode!(self, bus, eor_acc, absolute, 4),
            0x46 => opcode!(self, bus, eor_acc, indirect, 3),
            0x47 => opcode!(self, bus, eor_acc, x_indirect, 6),
            0x48 => opcode!(self, bus, eor_acc, immediate, 2),
            0x49 => opcode!(self, bus, eor_mem, direct_to_direct, 6),
            0x4A => opcode!(self, bus, and1, absolute_bit, 4),
            0x4B => opcode!(self, bus, lsr_mem, direct, 4),
            0x4C => opcode!(self, bus, lsr_mem, absolute, 5),
            0x4D => opcode!(self, bus, push_x, implied, 4),
            0x4E => opcode!(self, bus, tclr1, absolute, 6),
            0x4F => opcode!(self, bus, pcall, immediate, 6),
            0x50 => opcode!(self, bus, addr_no_bus, bvc, relative, 2),
            0x51 => opcode!(self, bus, tcall, 0xFFD4, 8),
            0x52 => opcode!(self, bus, bit_op, clr1, direct, 2, 4),
            0x53 => opcode!(self, bus, relative_bit_op, bbc, 2, 5),
            0x54 => opcode!(self, bus, eor_acc, x_direct, 4),
            0x55 => opcode!(self, bus, eor_acc, x_absolute, 5),
            0x56 => opcode!(self, bus, eor_acc, y_absolute, 5),
            0x57 => opcode!(self, bus, eor_acc, indirect_y, 6),
            0x58 => opcode!(self, bus, eor_mem, immediate_to_direct, 5),
            0x59 => opcode!(self, bus, eor_mem, indirect_to_indirect, 5),
            0x5A => opcode!(self, bus, cmpw, direct, 4),
            0x5B => opcode!(self, bus, lsr_mem, x_direct, 5),
            0x5C => opcode!(self, bus, lsr_acc, implied, 2),
            0x5D => opcode!(self, bus, tax, implied, 2),
            0x5E => opcode!(self, bus, cmy, absolute, 4),
            0x5F => opcode!(self, bus, addr_no_bus, jmp, absolute, 3),
            0x60 => opcode!(self, bus, clrc, implied, 2),
            0x61 => opcode!(self, bus, tcall, 0xFFD2, 8),
            0x62 => opcode!(self, bus, bit_op, set1, direct, 3, 4),
            0x63 => opcode!(self, bus, relative_bit_op, bbs, 3, 5),
            0x64 => opcode!(self, bus, cmp_acc, direct, 3),
            0x65 => opcode!(self, bus, cmp_acc, absolute, 4),
            0x66 => opcode!(self, bus, cmp_acc, indirect, 3),
            0x67 => opcode!(self, bus, cmp_acc, x_indirect, 6),
            0x68 => opcode!(self, bus, cmp_acc, immediate, 2),
            0x69 => opcode!(self, bus, cmp_mem, direct_to_direct, 6),
            0x6A => opcode!(self, bus, and1_inv, absolute_bit, 4),
            0x6B => opcode!(self, bus, ror_mem, direct, 4),
            0x6C => opcode!(self, bus, ror_mem, absolute, 5),
            0x6D => opcode!(self, bus, push_y, implied, 4),
            0x6E => opcode!(self, bus, dbnz_mem, direct_relative, 5),
            0x6F => opcode!(self, bus, ret, implied, 5),
            0x70 => opcode!(self, bus, addr_no_bus, bvs, relative, 2),
            0x71 => opcode!(self, bus, tcall, 0xFFD0, 8),
            0x72 => opcode!(self, bus, bit_op, clr1, direct, 3, 4),
            0x73 => opcode!(self, bus, relative_bit_op, bbc, 3, 5),
            0x74 => opcode!(self, bus, cmp_acc, x_direct, 4),
            0x75 => opcode!(self, bus, cmp_acc, x_absolute, 5),
            0x76 => opcode!(self, bus, cmp_acc, y_absolute, 5),
            0x77 => opcode!(self, bus, cmp_acc, indirect_y, 6),
            0x78 => opcode!(self, bus, cmp_mem, immediate_to_direct, 5),
            0x79 => opcode!(self, bus, cmp_mem, indirect_to_indirect, 5),
            0x7A => opcode!(self, bus, addw, direct, 5),
            0x7B => opcode!(self, bus, ror_mem, x_direct, 5),
            0x7C => opcode!(self, bus, ror_acc, implied, 2),
            0x7D => opcode!(self, bus, txa, implied, 2),
            0x7E => opcode!(self, bus, cmy, direct, 3),
            0x7F => opcode!(self, bus, ret1, implied, 6),
            0x80 => opcode!(self, bus, setc, implied, 2),
            0x81 => opcode!(self, bus, tcall, 0xFFCE, 8),
            0x82 => opcode!(self, bus, bit_op, set1, direct, 4, 4),
            0x83 => opcode!(self, bus, relative_bit_op, bbs, 4, 5),
            0x84 => opcode!(self, bus, adc_acc, direct, 3),
            0x85 => opcode!(self, bus, adc_acc, absolute, 4),
            0x86 => opcode!(self, bus, adc_acc, indirect, 3),
            0x87 => opcode!(self, bus, adc_acc, x_indirect, 6),
            0x88 => opcode!(self, bus, adc_acc, immediate, 2),
            0x89 => opcode!(self, bus, adc_mem, direct_to_direct, 6),
            0x8A => opcode!(self, bus, eor1, absolute_bit, 5),
            0x8B => opcode!(self, bus, dec_mem, direct, 4),
            0x8C => opcode!(self, bus, dec_mem, absolute, 5),
            0x8D => opcode!(self, bus, ldy, immediate, 2),
            0x8E => opcode!(self, bus, pop_psw, implied, 4),
            0x8F => opcode!(self, bus, mov, immediate_to_direct, 5),
            0x90 => opcode!(self, bus, addr_no_bus, bcc, relative, 2),
            0x91 => opcode!(self, bus, tcall, 0xFFCC, 8),
            0x92 => opcode!(self, bus, bit_op, clr1, direct, 4, 4),
            0x93 => opcode!(self, bus, relative_bit_op, bbc, 4, 5),
            0x94 => opcode!(self, bus, adc_acc, x_direct, 4),
            0x95 => opcode!(self, bus, adc_acc, x_absolute, 5),
            0x96 => opcode!(self, bus, adc_acc, y_absolute, 5),
            0x97 => opcode!(self, bus, adc_acc, indirect_y, 6),
            0x98 => opcode!(self, bus, adc_mem, immediate_to_direct, 5),
            0x99 => opcode!(self, bus, adc_mem, indirect_to_indirect, 5),
            0x9A => opcode!(self, bus, subw, direct, 5),
            0x9B => opcode!(self, bus, dec_mem, x_direct, 5),
            0x9C => opcode!(self, bus, dec_acc, implied, 2),
            0x9D => opcode!(self, bus, tsx, implied, 2),
            0x9E => opcode!(self, bus, div, implied, 12),
            0x9F => opcode!(self, bus, xcn, implied, 5),
            0xA0 => opcode!(self, bus, sei, implied, 3),
            0xA1 => opcode!(self, bus, tcall, 0xFFCA, 8),
            0xA2 => opcode!(self, bus, bit_op, set1, direct, 5, 4),
            0xA3 => opcode!(self, bus, relative_bit_op, bbs, 5, 5),
            0xA4 => opcode!(self, bus, sbc_acc, direct, 3),
            0xA5 => opcode!(self, bus, sbc_acc, absolute, 4),
            0xA6 => opcode!(self, bus, sbc_acc, indirect, 3),
            0xA7 => opcode!(self, bus, sbc_acc, x_indirect, 6),
            0xA8 => opcode!(self, bus, sbc_acc, immediate, 2),
            0xA9 => opcode!(self, bus, sbc_mem, direct_to_direct, 6),
            0xAA => opcode!(self, bus, ldc, absolute_bit, 4),
            0xAB => opcode!(self, bus, inc_mem, direct, 4),
            0xAC => opcode!(self, bus, inc_mem, absolute, 5),
            0xAD => opcode!(self, bus, cmy, immediate, 2),
            0xAE => opcode!(self, bus, pop_acc, implied, 4),
            0xAF => opcode!(self, bus, sta, indirect_inc, 4),
            0xB0 => opcode!(self, bus, addr_no_bus, bcs, relative, 2),
            0xB1 => opcode!(self, bus, tcall, 0xFFC8, 8),
            0xB2 => opcode!(self, bus, bit_op, clr1, direct, 5, 4),
            0xB3 => opcode!(self, bus, relative_bit_op, bbc, 5, 5),
            0xB4 => opcode!(self, bus, sbc_acc, x_direct, 4),
            0xB5 => opcode!(self, bus, sbc_acc, x_absolute, 5),
            0xB6 => opcode!(self, bus, sbc_acc, y_absolute, 5),
            0xB7 => opcode!(self, bus, sbc_acc, indirect_y, 6),
            0xB8 => opcode!(self, bus, sbc_mem, immediate_to_direct, 5),
            0xB9 => opcode!(self, bus, sbc_mem, indirect_to_indirect, 5),
            0xBA => opcode!(self, bus, ldya, direct, 5),
            0xBB => opcode!(self, bus, inc_mem, x_direct, 5),
            0xBC => opcode!(self, bus, inc_acc, implied, 2),
            0xBD => opcode!(self, bus, txs, implied, 2),
            0xBE => opcode!(self, bus, das, implied, 3),
            0xBF => opcode!(self, bus, lda, indirect_inc, 4),
            0xC0 => opcode!(self, bus, cli, implied, 3),
            0xC1 => opcode!(self, bus, tcall, 0xFFC6, 8),
            0xC2 => opcode!(self, bus, bit_op, set1, direct, 6, 4),
            0xC3 => opcode!(self, bus, relative_bit_op, bbs, 6, 5),
            0xC4 => opcode!(self, bus, sta, direct, 4),
            0xC5 => opcode!(self, bus, sta, absolute, 5),
            0xC6 => opcode!(self, bus, sta, indirect, 4),
            0xC7 => opcode!(self, bus, sta, x_indirect, 7),
            0xC8 => opcode!(self, bus, cmx, immediate, 2),
            0xC9 => opcode!(self, bus, stx, absolute, 5),
            0xCA => opcode!(self, bus, stc, absolute_bit, 6),
            0xCB => opcode!(self, bus, sty, direct, 4),
            0xCC => opcode!(self, bus, sty, absolute, 5),
            0xCD => opcode!(self, bus, ldx, immediate, 2),
            0xCE => opcode!(self, bus, pop_x, implied, 4),
            0xCF => opcode!(self, bus, mul, implied, 9),
            0xD0 => opcode!(self, bus, addr_no_bus, bne, relative, 2),
            0xD1 => opcode!(self, bus, tcall, 0xFFC4, 8),
            0xD2 => opcode!(self, bus, bit_op, clr1, direct, 6, 4),
            0xD3 => opcode!(self, bus, relative_bit_op, bbc, 6, 5),
            0xD4 => opcode!(self, bus, sta, x_direct, 5),
            0xD5 => opcode!(self, bus, sta, x_absolute, 6),
            0xD6 => opcode!(self, bus, sta, y_absolute, 6),
            0xD7 => opcode!(self, bus, sta, indirect_y, 7),
            0xD8 => opcode!(self, bus, stx, direct, 4),
            0xD9 => opcode!(self, bus, stx, y_direct, 5),
            0xDA => opcode!(self, bus, stya, direct, 5),
            0xDB => opcode!(self, bus, sty, x_direct, 5),
            0xDC => opcode!(self, bus, dey, implied, 2),
            0xDD => opcode!(self, bus, tya, implied, 2),
            0xDE => opcode!(self, bus, cbne, x_direct_relative, 6),
            0xDF => opcode!(self, bus, daa, implied, 3),
            0xE0 => opcode!(self, bus, clrv, implied, 2),
            0xE1 => opcode!(self, bus, tcall, 0xFFC2, 8),
            0xE2 => opcode!(self, bus, bit_op, set1, direct, 7, 4),
            0xE3 => opcode!(self, bus, relative_bit_op, bbs, 7, 5),
            0xE4 => opcode!(self, bus, lda, direct, 3),
            0xE5 => opcode!(self, bus, lda, absolute, 4),
            0xE6 => opcode!(self, bus, lda, indirect, 3),
            0xE7 => opcode!(self, bus, lda, x_indirect, 6),
            0xE8 => opcode!(self, bus, lda, immediate, 2),
            0xE9 => opcode!(self, bus, ldx, absolute, 4),
            0xEA => opcode!(self, bus, not1, absolute_bit, 5),
            0xEB => opcode!(self, bus, ldy, direct, 3),
            0xEC => opcode!(self, bus, ldy, absolute, 4),
            0xED => opcode!(self, bus, notc, implied, 3),
            0xEE => opcode!(self, bus, pop_y, implied, 4),
            0xEF => opcode!(self, bus, sleep, implied, 3),
            0xF0 => opcode!(self, bus, addr_no_bus, beq, relative, 2),
            0xF1 => opcode!(self, bus, tcall, 0xFFC0, 8),
            0xF2 => opcode!(self, bus, bit_op, clr1, direct, 7, 4),
            0xF3 => opcode!(self, bus, relative_bit_op, bbc, 7, 5),
            0xF4 => opcode!(self, bus, lda, x_direct, 4),
            0xF5 => opcode!(self, bus, lda, x_absolute, 5),
            0xF6 => opcode!(self, bus, lda, y_absolute, 5),
            0xF7 => opcode!(self, bus, lda, indirect_y, 6),
            0xF8 => opcode!(self, bus, ldx, direct, 3),
            0xF9 => opcode!(self, bus, ldx, y_direct, 4),
            0xFA => opcode!(self, bus, mov, direct_to_direct, 5),
            0xFB => opcode!(self, bus, ldy, x_direct, 4),
            0xFC => opcode!(self, bus, iny, implied, 2),
            0xFD => opcode!(self, bus, tay, implied, 2),
            0xFE => opcode!(self, bus, addr_no_bus, dbnz_y, relative, 4),
            0xFF => opcode!(self, bus, stop, implied, 3),
        };

        self.clocks += clocks;

        if self.branch_taken {
            self.clocks += 2;
        }
    }

    /// Reads the next byte of the program and increments PC
    fn read_prg<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u8 {
        let value = self.read(bus, self.pc);
        self.pc += 1;

        if H::IS_DEBUGGING_HARNESS && H::TRACK_SPC_INSTRUCTIONS {
            self.prg_bytes.push(value);
        }

        value
    }

    fn read<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr: u16) -> u8 {
        bus.read(addr)
    }

    fn write<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr: u16, value: u8) {
        bus.write(addr, value);
    }

    fn read_word_dp<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr: u16) -> u16 {
        let addr2 = (addr & 0xFF00) | ((addr + 1) & 0xFF);
        
        u16::from_le_bytes([
            self.read(bus, addr),
            self.read(bus, addr2),
        ])
    }

    fn read_word<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr: u16) -> u16 {
        u16::from_le_bytes([
            self.read(bus, addr),
            self.read(bus, addr + 1),
        ])
    }

    fn write_word<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr: u16, value: u16) {
        let addr2 = (addr & 0xFF00) | ((addr + 1) & 0xFF);
        
        self.write(bus, addr, get_byte_n!(value, 0));
        self.write(bus, addr2, get_byte_n!(value, 1));
    }

    fn pop<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u8 {
        self.sp += 1;
        self.read(bus, 0x100 | self.sp as u16)
    }

    fn push<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, value: u8) {
        self.write(bus, 0x100 | self.sp as u16, value);
        self.sp -= 1;
    }

    fn pop_word<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        u16::from_le_bytes([
            self.pop(bus),
            self.pop(bus)
        ])
    }

    fn push_word<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, value: u16) {
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

    fn direct<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        (self.read_prg(bus) as u16) | self.dir_page
    }

    fn x_direct<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        ((self.read_prg(bus) + self.x) as u16) | self.dir_page
    }

    fn y_direct<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        ((self.read_prg(bus) + self.y) as u16) | self.dir_page
    }

    fn indirect<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) -> u16 {
        (self.x as u16) | self.dir_page
    }

    fn indirect_inc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        let addr = self.indirect(bus);
        self.x += 1;
        addr
    }

    fn direct_to_direct<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> (u16, u16) {
        let src_addr = self.direct(bus);
        let dst_addr = self.direct(bus);

        (src_addr, dst_addr)
    }

    fn indirect_to_indirect<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) -> (u16, u16) {
        let arg1_addr = (self.x as u16) | self.dir_page;
        let arg2_addr = (self.y as u16) | self.dir_page;

        (arg2_addr, arg1_addr)
    }

    fn immediate_to_direct<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> (u16, u16) {
        let src_addr = self.immediate();
        let dst_addr = self.direct(bus);

        (src_addr, dst_addr)
    }

    fn direct_relative<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> (u16, u16) {
        let data_addr = self.direct(bus);
        let branch_addr = self.relative(bus);

        (data_addr, branch_addr)
    }

    fn absolute<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        u16::from_le_bytes([
            self.read_prg(bus),
            self.read_prg(bus),
        ])
    }

    fn absolute_bit<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> (u16, u8) {
        let address = self.absolute(bus);

        (address & 0x1FFF, (address >> 13) as u8)
    }

    fn x_absolute<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        self.absolute(bus) + (self.x as u16)
    }

    fn x_absolute_indirect<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        let ptr_addr = self.x_absolute(bus);

        self.read_word(bus, ptr_addr)
    }

    fn y_absolute<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        self.absolute(bus) + (self.y as u16)
    }

    fn x_direct_relative<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> (u16, u16) {
        let data_addr = self.x_direct(bus);
        let branch_addr = self.relative(bus);

        (data_addr, branch_addr)
    }

    fn x_indirect<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        let ptr_addr = self.x_direct(bus);

        self.read_word(bus, ptr_addr)
    }

    fn indirect_y<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
        let ptr_addr = self.direct(bus);

        self.read_word(bus, ptr_addr) + self.y as u16
    }

    fn relative<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) -> u16 {
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

    fn adc_acc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);
        self.a = self.adc_base(self.a, data, self.is_flag_set(Flag::FlagC));
    }

    fn adc_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);

        let result = self.adc_base(arg1, arg2, self.is_flag_set(Flag::FlagC));

        self.write(bus, addr2, result);
    }

    fn addw<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read_word_dp(bus, address);
        let ya = ((self.y as u16) << 8) | (self.a as u16);
        let result = self.add_16_base(ya, data);

        self.y = (result >> 8) as u8;
        self.a = result as u8;
    }

    // AND - AND Memory with Accumulator
    fn and_acc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.a &= self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn and_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let result = arg1 & arg2;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write(bus, addr2, result);
    }

    fn and1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagC, self.is_flag_set(Flag::FlagC) && get_bit_n!(data, bit));
    }

    fn and1_inv<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagC, self.is_flag_set(Flag::FlagC) && get_bit_n!(!data, bit));
    }

    // ASL - Shift Left One Bit (Accumulator version)
    fn asl_acc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        let result = self.a << 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 7));

        self.a = result;
    }

    // ASL - Shift Left One Bit (Memory version)
    fn asl_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);
        let result = data << 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 7));

        self.write(bus, address, result);
    }

    // BBC - Branch if Bit Clear
    fn bbc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, data_addr: u16, branch_addr: u16, bit: u8) {
        let data = self.read(bus, data_addr);

        if get_bit_n!(!data, bit) {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // BBS - Branch if Bit Set
    fn bbs<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, data_addr: u16, branch_addr: u16, bit: u8) {
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
    fn brk<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
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
    fn call<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, new_addr: u16) {
        self.push_word(bus, self.pc);
        self.pc = new_addr;
    }

    // CBNE - Compare and Branch if Not Equal
    fn cbne<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, branch_addr: u16) {
        let data = self.read(bus, address);

        if self.a != data {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // CMP - Compare Memory with Accumulator
    fn cmp_acc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);
        let result = self.a - data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, self.a >= data);
    }

    fn cmp_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let result = arg2 - arg1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, arg2 >= arg1);
    }

    // CLI - CLear Interrupt flag (called DI in SPC700 documentation)
    fn cli<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.clear_flag(Flag::FlagI);
    }

    // CLR1 - clears a single bit in the direct page
    fn clr1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let b = 1 << bit;

        self.write(bus, address, data & !b);
    }

    // CLRC - clear carry flag
    fn clrc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.clear_flag(Flag::FlagC);
    }

    // CLRP - clear direct page flag
    fn clrp<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.clear_flag(Flag::FlagP);
        self.dir_page = 0;
    }

    // CLRV - clear overflow flag (and half carry)
    fn clrv<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.clear_flag(Flag::FlagV);
        self.clear_flag(Flag::FlagH);
    }

    // CMPW - Compare Word with YA
    fn cmpw<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read_word_dp(bus, address);
        let ya = ((self.y as u16) << 8) | (self.a as u16);
        let result = ya - data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 15));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, ya >= data);
    }

    // CMX - Compare Memory with X
    fn cmx<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);
        let result = self.x - data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, self.x >= data);
    }

    // CMY - Compare Memory with Y
    fn cmy<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);
        let result = self.y - data;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, self.y >= data);
    }

    // DAA - Decimal Adjust Addition
    fn daa<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
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
    fn das<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
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
    fn dbnz_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, branch_addr: u16) {
        let result = self.read(bus, address) - 1;
        self.write(bus, address, result);

        if result != 0 {
            self.pc = branch_addr;
            self.branch_taken = true;
        }
    }

    // DEC - decrement (accumulator)
    fn dec_acc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.a -= 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    // DEC - decrement (memory)
    fn dec_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address) - 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(data, 7));
        self.set_flag_to_bool(Flag::FlagZ, data == 0);

        self.write(bus, address, data);
    }
    
    fn decw<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let result = self.read_word_dp(bus, address) - 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 15));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write_word(bus, address, result);
    }

    fn dex<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.x -= 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn dey<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.y -= 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn div<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
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

    fn eor_acc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.a ^= self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn eor_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let result = arg1 ^ arg2;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write(bus, addr2, result);
    }

    fn eor1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let result = self.is_flag_set(Flag::FlagC) ^ get_bit_n!(data, bit);

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn inc_acc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.a += 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn inc_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let result = self.read(bus, address) + 1;

        self.write(bus, address, result);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn incw<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let result = self.read_word_dp(bus, address) + 1;

        self.write_word(bus, address, result);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 15));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
    }

    fn inx<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.x += 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn iny<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.y += 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    fn lda<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.a = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn ldc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, bit));
    }

    fn ldx<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.x = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn ldy<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.y = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn ldya<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read_word_dp(bus, address);

        self.y = (data >> 8) as u8;
        self.a = data as u8;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0 && self.a == 0);
    }

    fn lsr_acc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(self.a, 0));

        self.a >>= 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn lsr_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);
        let result = data >> 1;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 0));

        self.write(bus, address, result);
    }

    fn mov<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, src_addr: u16, dst_addr: u16) {
        let data = self.read(bus, src_addr);

        self.write(bus, dst_addr, data);
    }

    fn mul<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        let result = (self.y as u16) * (self.a as u16);

        self.y = (result >> 8) as u8;
        self.a = result as u8;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn nop<H: DebugHarness>(&self, _bus: &SpcBus<H>) {}

    fn not1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let b = 1 << bit;
        let result = data ^ b;

        self.write(bus, address, result);
    }

    fn notc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.status ^= Flag::FlagC as u8;
    }

    fn or1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let result = self.is_flag_set(Flag::FlagC) || get_bit_n!(data, bit);

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn or1_inv<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let result = self.is_flag_set(Flag::FlagC) || get_bit_n!(!data, bit);

        self.set_flag_to_bool(Flag::FlagC, result);
    }

    fn or_acc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.a |= self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn or_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let result = arg1 | arg2;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);

        self.write(bus, addr2, result);
    }

    fn pcall<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let call_addr = 0xFF00 | self.read(bus, address) as u16;

        self.call(bus, call_addr);
    }

    fn pop_acc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.a = self.pop(bus);
    }

    fn pop_x<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.x = self.pop(bus);
    }

    fn pop_y<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.y = self.pop(bus);
    }

    fn pop_psw<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.status = self.pop(bus);

        if self.is_flag_set(Flag::FlagP) {
            self.dir_page = 0x100;
        } else {
            self.dir_page = 0;
        }
    }

    fn push_acc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.push(bus, self.a);
    }

    fn push_x<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.push(bus, self.x);
    }

    fn push_y<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.push(bus, self.y);
    }

    fn push_psw<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.push(bus, self.status);
    }

    fn ret<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.pc = self.pop_word(bus);
    }

    fn ret1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>) {
        self.status = self.pop(bus);
        self.pc = self.pop_word(bus);

        if self.is_flag_set(Flag::FlagP) {
            self.dir_page = 0x100;
        } else {
            self.dir_page = 0;
        }
    }

    fn rol_acc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        let new_c = get_bit_n!(self.a, 7);
        
        self.a <<= 1;
        self.a |= if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };
        
        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
        self.set_flag_to_bool(Flag::FlagC, new_c);
    }

    fn rol_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);
        let result = (data << 1) | if self.is_flag_set(Flag::FlagC) { 1 } else { 0 };

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(result, 7));
        self.set_flag_to_bool(Flag::FlagZ, result == 0);
        self.set_flag_to_bool(Flag::FlagC, get_bit_n!(data, 7));

        self.write(bus, address, result);
    }

    fn ror_acc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
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

    fn ror_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
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

    fn sbc_acc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);
        let comp = !data;

        self.a = self.adc_base(self.a, comp, self.is_flag_set(Flag::FlagC));
    }

    fn sbc_mem<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, addr1: u16, addr2: u16) {
        let arg1 = self.read(bus, addr1);
        let arg2 = self.read(bus, addr2);
        let comp1 = !arg1;

        let result = self.adc_base(arg2, comp1, self.is_flag_set(Flag::FlagC));

        self.write(bus, addr2, result);
    }

    fn sei<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.set_flag(Flag::FlagI)
    }

    fn set1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        let data = self.read(bus, address);
        let b = 1 << bit;

        self.write(bus, address, data | b);
    }

    fn setc<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.set_flag(Flag::FlagC);
    }

    fn setp<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.set_flag(Flag::FlagP);
        self.dir_page = 0x100;
    }

    fn sleep<H: DebugHarness>(&self, _bus: &SpcBus<H>) {}

    fn sta<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.write(bus, address, self.a);
    }

    // MOV1 alias
    fn stc<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16, bit: u8) {
        if self.is_flag_set(Flag::FlagC) {
            self.set1(bus, address, bit);
        } else {
            self.clr1(bus, address, bit);
        }
    }

    fn stop<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) { self.stopped = true; }

    fn stx<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.write(bus, address, self.x);
    }

    fn sty<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.write(bus, address, self.y);
    }

    fn stya<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let addr2 = (address & 0xFF00) | ((address + 1) & 0xFF);
        self.write(bus, address, self.a);
        self.write(bus, addr2, self.y);
    }

    fn subw<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read_word_dp(bus, address);
        let comp = !data + 1;
        let ya = ((self.y as u16) << 8) | (self.a as u16);
        let result = self.add_16_base(ya, comp);

        self.y = (result >> 8) as u8;
        self.a = result as u8;
    }

    fn tax<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.x = self.a;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn tay<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.y = self.a;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.y, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.y == 0);
    }

    fn tcall<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        self.push_word(bus, self.pc);
        self.pc = self.read_word(bus, address);
    }

    fn tclr1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!((self.a - data), 7));
        self.set_flag_to_bool(Flag::FlagZ, (self.a - data) == 0);
        
        self.write(bus, address, data & !self.a);
    }

    fn tset1<H: DebugHarness>(&mut self, bus: &mut SpcBus<H>, address: u16) {
        let data = self.read(bus, address);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!((self.a - data), 7));
        self.set_flag_to_bool(Flag::FlagZ, (self.a - data) == 0);
        
        self.write(bus, address, data | self.a);
    }

    fn tsx<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.x = self.sp;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.x, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.x == 0);
    }

    fn txa<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.a = self.x;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn txs<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.sp = self.x;
    }

    fn tya<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.a = self.y;

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }

    fn xcn<H: DebugHarness>(&mut self, _bus: &SpcBus<H>) {
        self.a = (self.a >> 4) | (self.a << 4);

        self.set_flag_to_bool(Flag::FlagN, get_bit_n!(self.a, 7));
        self.set_flag_to_bool(Flag::FlagZ, self.a == 0);
    }
}