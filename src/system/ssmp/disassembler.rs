#![allow(dead_code)]

use std::io::Write;

const IPL_ROM: [u8; 64] = [
    0xCD, 0xEF, 0xBD, 0xE8, 0x00, 0xC6, 0x1D, 0xD0, 0xFC, 0x8F, 0xAA, 0xF4, 0x8F, 0xBB, 0xF5,
    0x78, 0xCC, 0xF4, 0xD0, 0xFB, 0x2F, 0x19, 0xEB, 0xF4, 0xD0, 0xFC, 0x7E, 0xF4, 0xD0, 0x0B,
    0xE4, 0xF5, 0xCB, 0xF4, 0xD7, 0x00, 0xFC, 0xD0, 0xF3, 0xAB, 0x01, 0x10, 0xEF, 0x7E, 0xF4,
    0x10, 0xEB, 0xBA, 0xF6, 0xDA, 0x00, 0xBA, 0xF4, 0xC4, 0xF4, 0xDD, 0x5D, 0xD0, 0xDB, 0x1F,
    0x00, 0x00, 0xC0, 0xFF,
];

struct DisassembledInstr {
    instr_pc: u16,
    instr_name: String,
    instr_op: u8,
    instr_b0: Option<u8>,
    instr_b1: Option<u8>,
    nbytes: usize,
    addr_mode_str: String,
}

impl DisassembledInstr {
    fn instr_as_string(&self) -> String {
        let instr_start = format!("${:04X}:  {:02X}", self.instr_pc, self.instr_op);

        let data_str = if let Some(b0) = self.instr_b0 {
            if let Some(b1) = self.instr_b1 {
                format!("{b0:02X} {b1:02X}")
            } else {
                format!("{b0:02X}")
            }
        } else {
            "".to_string()
        };

        format!("{} {:<5} {:<5} {}",
            instr_start,
            data_str,
            self.instr_name,
            self.addr_mode_str,
        ).to_string()
    }
}

/// Returns a disassembled version of the instruction
fn disassemble_instr(pc: u16, pc_disp_offset: u16, aram: &[u8]) -> DisassembledInstr {
    let opcode_addr = pc;

    let mut pc = pc;
    let mut read_prg = || -> u8 {
        let data = aram[pc as usize];
        pc += 1;
        data
    };
    
    let opcode = if (opcode_addr+0) as usize >= aram.len() { 0 } else { read_prg() };
    let b0 = if (opcode_addr+1) as usize >= aram.len() { 0 } else { read_prg() };
    let b1 = if (opcode_addr+2) as usize >= aram.len() { 0 } else { read_prg() };

    let instr_name: String;
    let addr_mode_str: String;
    let nbytes: usize;

    let opcode_addr = opcode_addr + pc_disp_offset;

    match opcode {
        0x00 => {
            instr_name = "nop".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x01 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "0".to_string();
        }
        0x02 => {
            instr_name = "set1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}").to_string();
        }
        0x03 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbs0".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x04 => {
            instr_name = "or".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x05 => {
            instr_name = "or".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x06 => {
            instr_name = "or".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)").to_string();
        }
        0x07 => {
            instr_name = "or".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x08 => {
            instr_name = "or".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0x09 => {
            instr_name = "or".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},${b0:02X}").to_string();
        }
        0x0A => {
            let addr = (((b1 as u16) << 8) | (b0 as u16)) & 0x1FFF;
            let bit = b1 >> 5;

            instr_name = "or1".to_string();
            nbytes = 3;
            addr_mode_str = format!("${addr:04X},{bit}").to_string();
        }
        0x0B => {
            instr_name = "asl".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x0C => {
            instr_name = "asl".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x0D => {
            instr_name = "push".to_string();
            nbytes = 1;
            addr_mode_str = "psw".to_string();
        }
        0x0E => {
            instr_name = "tset1".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x0F => {
            instr_name = "brk".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x10 => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "bpl".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0x11 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "1".to_string();
        }
        0x12 => {
            instr_name = "clr1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},0").to_string();
        }
        0x13 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbc0".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x14 => {
            instr_name = "or".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x15 => {
            instr_name = "or".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+X").to_string();
        }
        0x16 => {
            instr_name = "or".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+Y").to_string();
        }
        0x17 => {
            instr_name = "or".to_string();
            nbytes = 2;
            addr_mode_str = format!("(${b0:02X})+Y").to_string();
        }
        0x18 => {
            instr_name = "or".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},#${b0:02X}").to_string();
        }
        0x19 => {
            instr_name = "or".to_string();
            nbytes = 1;
            addr_mode_str = "(X),(Y)".to_string();
        }
        0x1A => {
            instr_name = "decw".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x1B => {
            instr_name = "asl".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x1C => {
            instr_name = "asl".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x1D => {
            instr_name = "dex".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x1E => {
            instr_name = "cmx".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x1F => {
            instr_name = "jmp".to_string();
            nbytes = 3;
            addr_mode_str = format!("(${b1:02X}{b0:02X}+X)");
        }
        0x20 => {
            instr_name = "clrp".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x21 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "2".to_string();
        }
        0x22 => {
            instr_name = "set1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},1").to_string();
        }
        0x23 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbs1".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x24 => {
            instr_name = "and".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x25 => {
            instr_name = "and".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x26 => {
            instr_name = "and".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)").to_string();
        }
        0x27 => {
            instr_name = "and".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x28 => {
            instr_name = "and".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0x29 => {
            instr_name = "and".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},${b0:02X}").to_string();
        }
        0x2A => {
            let addr = (((b1 as u16) << 8) | (b0 as u16)) & 0x1FFF;
            let bit = b1 >> 5;

            instr_name = "or1n".to_string();
            nbytes = 3;
            addr_mode_str = format!("${addr:04X},{bit}").to_string();
        }
        0x2B => {
            instr_name = "rol".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x2C => {
            instr_name = "rol".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x2D => {
            instr_name = "push".to_string();
            nbytes = 1;
            addr_mode_str = "acc".to_string();
        }
        0x2E => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "cbne".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x2F => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "bra".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0x30 => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "bmi".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0x31 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "3".to_string();
        }
        0x32 => {
            instr_name = "clr1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},1").to_string();
        }
        0x33 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbc1".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x34 => {
            instr_name = "and".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x35 => {
            instr_name = "and".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+X").to_string();
        }
        0x36 => {
            instr_name = "and".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+Y").to_string();
        }
        0x37 => {
            instr_name = "and".to_string();
            nbytes = 2;
            addr_mode_str = format!("(${b0:02X})+Y").to_string();
        }
        0x38 => {
            instr_name = "and".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},#${b0:02X}").to_string();
        }
        0x39 => {
            instr_name = "and".to_string();
            nbytes = 1;
            addr_mode_str = "(X),(Y)".to_string();
        }
        0x3A => {
            instr_name = "incw".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x3B => {
            instr_name = "rol".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x3C => {
            instr_name = "rol".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x3D => {
            instr_name = "inx".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x3E => {
            instr_name = "cmx".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x3F => {
            instr_name = "call".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x40 => {
            instr_name = "setp".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x41 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "4".to_string();
        }
        0x42 => {
            instr_name = "set1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},2").to_string();
        }
        0x43 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbs2".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x44 => {
            instr_name = "eor".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x45 => {
            instr_name = "eor".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x46 => {
            instr_name = "eor".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)").to_string();
        }
        0x47 => {
            instr_name = "eor".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x48 => {
            instr_name = "eor".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0x49 => {
            instr_name = "eor".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},${b0:02X}").to_string();
        }
        0x4A => {
            let addr = (((b1 as u16) << 8) | (b0 as u16)) & 0x1FFF;
            let bit = b1 >> 5;

            instr_name = "and1".to_string();
            nbytes = 3;
            addr_mode_str = format!("${addr:04X},{bit}").to_string();
        }
        0x4B => {
            instr_name = "lsr".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x4C => {
            instr_name = "lsr".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x4D => {
            instr_name = "push".to_string();
            nbytes = 1;
            addr_mode_str = "x".to_string();
        }
        0x4E => {
            instr_name = "tclr1".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x4F => {
            instr_name = "pcall".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0x50 => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "bvc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0x51 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "5".to_string();
        }
        0x52 => {
            instr_name = "clr1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},2").to_string();
        }
        0x53 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbc2".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x54 => {
            instr_name = "eor".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x55 => {
            instr_name = "eor".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+X").to_string();
        }
        0x56 => {
            instr_name = "eor".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+Y").to_string();
        }
        0x57 => {
            instr_name = "eor".to_string();
            nbytes = 2;
            addr_mode_str = format!("(${b0:02X})+Y").to_string();
        }
        0x58 => {
            instr_name = "eor".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},#${b0:02X}").to_string();
        }
        0x59 => {
            instr_name = "eor".to_string();
            nbytes = 1;
            addr_mode_str = "(X),(Y)".to_string();
        }
        0x5A => {
            instr_name = "cmpw".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x5B => {
            instr_name = "lsr".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x5C => {
            instr_name = "lsr".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x5D => {
            instr_name = "tax".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x5E => {
            instr_name = "cmy".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x5F => {
            instr_name = "jmp".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x60 => {
            instr_name = "clrc".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x61 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "6".to_string();
        }
        0x62 => {
            instr_name = "set1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},3").to_string();
        }
        0x63 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbs3".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x64 => {
            instr_name = "cmp".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x65 => {
            instr_name = "cmp".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x66 => {
            instr_name = "cmp".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)").to_string();
        }
        0x67 => {
            instr_name = "cmp".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x68 => {
            instr_name = "cmp".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0x69 => {
            instr_name = "cmp".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},${b0:02X}").to_string();
        }
        0x6A => {
            let addr = (((b1 as u16) << 8) | (b0 as u16)) & 0x1FFF;
            let bit = b1 >> 5;

            instr_name = "and1n".to_string();
            nbytes = 3;
            addr_mode_str = format!("${addr:04X},{bit}").to_string();
        }
        0x6B => {
            instr_name = "ror".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x6C => {
            instr_name = "ror".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x6D => {
            instr_name = "push".to_string();
            nbytes = 1;
            addr_mode_str = "y".to_string();
        }
        0x6E => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "dbnz".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x6F => {
            instr_name = "ret".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x70 => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "bvs".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0x71 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "7".to_string();
        }
        0x72 => {
            instr_name = "clr1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},3").to_string();
        }
        0x73 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbc3".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x74 => {
            instr_name = "cmp".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x75 => {
            instr_name = "cmp".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+X").to_string();
        }
        0x76 => {
            instr_name = "cmp".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+Y").to_string();
        }
        0x77 => {
            instr_name = "cmp".to_string();
            nbytes = 2;
            addr_mode_str = format!("(${b0:02X})+Y").to_string();
        }
        0x78 => {
            instr_name = "cmp".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},#${b0:02X}").to_string();
        }
        0x79 => {
            instr_name = "cmp".to_string();
            nbytes = 1;
            addr_mode_str = "(X),(Y)".to_string();
        }
        0x7A => {
            instr_name = "addw".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x7B => {
            instr_name = "ror".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x7C => {
            instr_name = "ror".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x7D => {
            instr_name = "txa".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x7E => {
            instr_name = "cmy".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x7F => {
            instr_name = "ret1".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x80 => {
            instr_name = "setc".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x81 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "8".to_string();
        }
        0x82 => {
            instr_name = "set1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},4").to_string();
        }
        0x83 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbs4".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x84 => {
            instr_name = "adc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x85 => {
            instr_name = "adc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x86 => {
            instr_name = "adc".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)").to_string();
        }
        0x87 => {
            instr_name = "adc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x88 => {
            instr_name = "adc".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0x89 => {
            instr_name = "adc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},${b0:02X}").to_string();
        }
        0x8A => {
            let addr = (((b1 as u16) << 8) | (b0 as u16)) & 0x1FFF;
            let bit = b1 >> 5;

            instr_name = "eor1".to_string();
            nbytes = 3;
            addr_mode_str = format!("${addr:04X},{bit}").to_string();
        }
        0x8B => {
            instr_name = "dec".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x8C => {
            instr_name = "dec".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0x8D => {
            instr_name = "ldy".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0x8E => {
            instr_name = "pop".to_string();
            nbytes = 1;
            addr_mode_str = "psw".to_string();
        }
        0x8F => {
            instr_name = "mov".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},#${b0:02X}").to_string();
        }
        0x90 => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "bcc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0x91 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "9".to_string();
        }
        0x92 => {
            instr_name = "clr1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},4").to_string();
        }
        0x93 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbc4".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0x94 => {
            instr_name = "adc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x95 => {
            instr_name = "adc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+X").to_string();
        }
        0x96 => {
            instr_name = "adc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+Y").to_string();
        }
        0x97 => {
            instr_name = "adc".to_string();
            nbytes = 2;
            addr_mode_str = format!("(${b0:02X})+Y").to_string();
        }
        0x98 => {
            instr_name = "adc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},#${b0:02X}").to_string();
        }
        0x99 => {
            instr_name = "adc".to_string();
            nbytes = 1;
            addr_mode_str = "(X),(Y)".to_string();
        }
        0x9A => {
            instr_name = "subw".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0x9B => {
            instr_name = "dec".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0x9C => {
            instr_name = "dec".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x9D => {
            instr_name = "tsx".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x9E => {
            instr_name = "div".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0x9F => {
            instr_name = "xcn".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xA0 => {
            instr_name = "sei".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xA1 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "A".to_string();
        }
        0xA2 => {
            instr_name = "set1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},5").to_string();
        }
        0xA3 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbs5".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0xA4 => {
            instr_name = "sbc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xA5 => {
            instr_name = "sbc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0xA6 => {
            instr_name = "sbc".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)").to_string();
        }
        0xA7 => {
            instr_name = "sbc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xA8 => {
            instr_name = "sbc".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0xA9 => {
            instr_name = "sbc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},${b0:02X}").to_string();
        }
        0xAA => {
            let addr = (((b1 as u16) << 8) | (b0 as u16)) & 0x1FFF;
            let bit = b1 >> 5;

            instr_name = "ldc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${addr:04X},{bit}").to_string();
        }
        0xAB => {
            instr_name = "inc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xAC => {
            instr_name = "inc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0xAD => {
            instr_name = "cmy".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0xAE => {
            instr_name = "pop".to_string();
            nbytes = 1;
            addr_mode_str = "acc".to_string();
        }
        0xAF => {
            instr_name = "sta".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)+").to_string();
        }
        0xB0 => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "bcs".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0xB1 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "B".to_string();
        }
        0xB2 => {
            instr_name = "clr1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},5").to_string();
        }
        0xB3 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbc5".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0xB4 => {
            instr_name = "sbc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xB5 => {
            instr_name = "sbc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+X").to_string();
        }
        0xB6 => {
            instr_name = "sbc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+Y").to_string();
        }
        0xB7 => {
            instr_name = "sbc".to_string();
            nbytes = 2;
            addr_mode_str = format!("(${b0:02X})+Y").to_string();
        }
        0xB8 => {
            instr_name = "sbc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},#${b0:02X}").to_string();
        }
        0xB9 => {
            instr_name = "sbc".to_string();
            nbytes = 1;
            addr_mode_str = "(X),(Y)".to_string();
        }
        0xBA => {
            instr_name = "ldya".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xBB => {
            instr_name = "inc".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xBC => {
            instr_name = "inc".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xBD => {
            instr_name = "txs".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xBE => {
            instr_name = "das".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xBF => {
            instr_name = "lda".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)+").to_string();
        }
        0xC0 => {
            instr_name = "cli".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xC1 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "C".to_string();
        }
        0xC2 => {
            instr_name = "set1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},6").to_string();
        }
        0xC3 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbs6".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0xC4 => {
            instr_name = "sta".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xC5 => {
            instr_name = "sta".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0xC6 => {
            instr_name = "sta".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)").to_string();
        }
        0xC7 => {
            instr_name = "sta".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xC8 => {
            instr_name = "cmx".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0xC9 => {
            instr_name = "stx".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0xCA => {
            let addr = (((b1 as u16) << 8) | (b0 as u16)) & 0x1FFF;
            let bit = b1 >> 5;

            instr_name = "stc".to_string();
            nbytes = 3;
            addr_mode_str = format!("${addr:04X},{bit}").to_string();
        }
        0xCB => {
            instr_name = "sty".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xCC => {
            instr_name = "sty".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0xCD => {
            instr_name = "ldx".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0xCE => {
            instr_name = "pop".to_string();
            nbytes = 1;
            addr_mode_str = "x".to_string();
        }
        0xCF => {
            instr_name = "mul".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xD0 => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "bne".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0xD1 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "D".to_string();
        }
        0xD2 => {
            instr_name = "clr1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},6").to_string();
        }
        0xD3 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbc6".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0xD4 => {
            instr_name = "sta".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xD5 => {
            instr_name = "sta".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+X").to_string();
        }
        0xD6 => {
            instr_name = "sta".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+Y").to_string();
        }
        0xD7 => {
            instr_name = "sta".to_string();
            nbytes = 2;
            addr_mode_str = format!("(${b0:02X})+Y").to_string();
        }
        0xD8 => {
            instr_name = "stx".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xD9 => {
            instr_name = "stx".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+Y").to_string();
        }
        0xDA => {
            instr_name = "stya".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xDB => {
            instr_name = "sty".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xDC => {
            instr_name = "dey".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xDD => {
            instr_name = "tya".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xDE => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "cbne".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X}+X,${rel_addr:04X}").to_string();
        }
        0xDF => {
            instr_name = "daa".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xE0 => {
            instr_name = "clrv".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xE1 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "E".to_string();
        }
        0xE2 => {
            instr_name = "set1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},7").to_string();
        }
        0xE3 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbs7".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0xE4 => {
            instr_name = "lda".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xE5 => {
            instr_name = "lda".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0xE6 => {
            instr_name = "lda".to_string();
            nbytes = 1;
            addr_mode_str = format!("(X)").to_string();
        }
        0xE7 => {
            instr_name = "lda".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xE8 => {
            instr_name = "lda".to_string();
            nbytes = 2;
            addr_mode_str = format!("#${b0:02X}").to_string();
        }
        0xE9 => {
            instr_name = "ldx".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0xEA => {
            let addr = (((b1 as u16) << 8) | (b0 as u16)) & 0x1FFF;
            let bit = b1 >> 5;

            instr_name = "not1".to_string();
            nbytes = 3;
            addr_mode_str = format!("${addr:04X},{bit}").to_string();
        }
        0xEB => {
            instr_name = "ldy".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xEC => {
            instr_name = "ldy".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}");
        }
        0xED => {
            instr_name = "notc".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xEE => {
            instr_name = "pop".to_string();
            nbytes = 1;
            addr_mode_str = "y".to_string();
        }
        0xEF => {
            instr_name = "sleep".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xF0 => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "beq".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0xF1 => {
            instr_name = "tcall".to_string();
            nbytes = 1;
            addr_mode_str = "F".to_string();
        }
        0xF2 => {
            instr_name = "clr1".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X},7").to_string();
        }
        0xF3 => {
            let rel_addr = opcode_addr + 2 + (b1 as i8) as u16;

            instr_name = "bbc7".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b0:02X},${rel_addr:04X}").to_string();
        }
        0xF4 => {
            instr_name = "lda".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xF5 => {
            instr_name = "lda".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+X").to_string();
        }
        0xF6 => {
            instr_name = "lda".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X}{b0:02X}+Y").to_string();
        }
        0xF7 => {
            instr_name = "lda".to_string();
            nbytes = 2;
            addr_mode_str = format!("(${b0:02X})+Y").to_string();
        }
        0xF8 => {
            instr_name = "ldx".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}");
        }
        0xF9 => {
            instr_name = "ldx".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+Y").to_string();
        }
        0xFA => {
            instr_name = "mov".to_string();
            nbytes = 3;
            addr_mode_str = format!("${b1:02X},${b0:02X}").to_string();
        }
        0xFB => {
            instr_name = "ldy".to_string();
            nbytes = 2;
            addr_mode_str = format!("${b0:02X}+X").to_string();
        }
        0xFC => {
            instr_name = "iny".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xFD => {
            instr_name = "tay".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
        0xFE => {
            let rel_addr = opcode_addr + 2 + (b0 as i8) as u16;

            instr_name = "dbnz_y".to_string();
            nbytes = 2;
            addr_mode_str = format!("${rel_addr:04X}").to_string();
        }
        0xFF => {
            instr_name = "stop".to_string();
            nbytes = 1;
            addr_mode_str = "".to_string();
        }
    }

    DisassembledInstr {
        instr_pc: opcode_addr,
        instr_op: opcode,
        instr_b0: if nbytes > 1 { Some(b0) } else { None },
        instr_b1: if nbytes > 2 { Some(b1) } else { None },
        instr_name,
        nbytes,
        addr_mode_str,
    }
}

pub(super) fn disassembly_string(pc: u16, aram: &[u8]) -> String {
    disassemble_instr(pc, 0, aram).instr_as_string()    
}

pub(super) fn disassemble_block(prg_bytes: &[u8], pc_disp_offset: u16, outfile_str: &str) {
    let mut outfile = std::fs::File::create(std::path::Path::new(outfile_str)).unwrap();

    let mut pc = 0;
    let mut instr_cnt = 0;

    while pc < prg_bytes.len() as u16 {
        let instr = disassemble_instr(pc, pc_disp_offset, prg_bytes);

        let instr_str = format!("{}\n", instr.instr_as_string());

        let _ = outfile.write(instr_str.as_bytes());

        instr_cnt += 1;

        if (pc as usize) + instr.nbytes > 0xFFFF {
            println!("Program exceeded maximum size. Stopping.");
            break;
        }

        pc += instr.nbytes as u16;
    }

    println!("Disassembly of {} bytes ({} instrs) finished.", prg_bytes.len(), instr_cnt);
}

#[cfg(test)]
mod test {
    use crate::system::ssmp::disassembler::disassemble_block;

    #[test]
    fn disassemble_ipl_rom() {
        disassemble_block(&super::IPL_ROM, 0xFFC0, "src/system/ssmp/ipl_disassembly.s");
    }
}