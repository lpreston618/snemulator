use snemcore::ssmp::spc::Spc700;

#[derive(Clone, Copy, Debug)]
pub enum DisasmOperandKind {
    /// #$xx — 8-bit immediate
    Immediate8(u8),
    /// $xx — 8-bit direct-page address (also covers XIndirect/IndirectY dp byte)
    DirectPage(u8),
    /// $xxxx — 16-bit absolute address
    Absolute(u16),
    /// $xxxx.b — absolute address + bit index 0-7 (OR1/AND1/EOR1/MOV1/NOT1)
    AbsoluteBit(u16, u8),
    /// $xxxx — resolved branch target (PC + signed relative offset).
    BranchTarget(u16),
}

#[derive(Clone, Copy, Debug)]
pub struct DisasmOperand {
    pub kind: DisasmOperandKind,
}

pub struct DisasmLine {
    pub addr: u16,
    pub bytes: Vec<u8>,
    pub mnemonic: &'static str,
    pub operands: Vec<DisasmOperand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Indirect,          // (X), (Y) — no operand bytes, register is implied
    IndirectAutoInc,   // (X)+
    Direct,            // dp
    DirectX,           // dp+X
    DirectY,           // dp+Y
    XIndirect,         // [dp+X]
    IndirectY,         // [dp]+Y
    Immediate,         // #imm
    Relative,          // rel (branch alone, e.g. BRA/BEQ/BNE)
    DirectToDirect,    // dp, dp
    ImmediateToDirect, // dp, #imm
    DirectRelative,    // dp.bit, rel  (BBS/BBC)
    Absolute,          // !abs
    AbsoluteX,         // !abs+X
    AbsoluteY,         // !abs+Y
    AbsoluteBit,       // mem.bit (OR1/AND1/EOR1/...)
}

impl AddressingMode {
    /// Bytes consumed after the opcode byte.
    pub const fn operand_len(self) -> usize {
        use AddressingMode::*;
        match self {
            Implied | Indirect | IndirectAutoInc => 0,
            Direct | DirectX | DirectY | XIndirect
            | IndirectY | Immediate | Relative => 1,
            DirectToDirect | ImmediateToDirect | DirectRelative
            | Absolute | AbsoluteX | AbsoluteY | AbsoluteBit => 2,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DisassembleData {
    pub mnemonic: &'static str, // format template; `{}` marks a decoded operand
    pub mode: AddressingMode,
}

impl DisassembleData {
    pub const fn bytes(&self) -> usize {
        self.mode.operand_len() + 1
    }
}

macro_rules! op {
    ($mnemonic:expr, $mode:ident) => {
        DisassembleData { mnemonic: $mnemonic, mode: AddressingMode::$mode }
    };
}

pub const OPCODE_TABLE: [DisassembleData; 256] = [
    /* 0x00 */ op!("NOP", Implied),
    /* 0x01 */ op!("TCALL 0", Implied),
    /* 0x02 */ op!("SET1 {}.0", Direct),
    /* 0x03 */ op!("BBS {}.0, {}", DirectRelative),
    /* 0x04 */ op!("OR A, {}", Direct),
    /* 0x05 */ op!("OR A, {}", Absolute),
    /* 0x06 */ op!("OR A, (X)", Indirect),
    /* 0x07 */ op!("OR A, [{}+X]", XIndirect),
    /* 0x08 */ op!("OR A, {}", Immediate),
    /* 0x09 */ op!("OR {}, {}", DirectToDirect),
    /* 0x0A */ op!("OR1 C, {}", AbsoluteBit),
    /* 0x0B */ op!("ASL {}", Direct),
    /* 0x0C */ op!("ASL {}", Absolute),
    /* 0x0D */ op!("PUSH PSW", Implied),
    /* 0x0E */ op!("TSET1 {}", Absolute),
    /* 0x0F */ op!("BRK", Implied),
    /* 0x10 */ op!("BPL {}", Relative),
    /* 0x11 */ op!("TCALL 1", Implied),
    /* 0x12 */ op!("CLR1 {}.0", Direct),
    /* 0x13 */ op!("BBC {}.0, {}", DirectRelative),
    /* 0x14 */ op!("OR A, {}+X", DirectX),
    /* 0x15 */ op!("OR A, {}+X", AbsoluteX),
    /* 0x16 */ op!("OR A, {}+Y", AbsoluteY),
    /* 0x17 */ op!("OR A, [{}]+Y", IndirectY),
    /* 0x18 */ op!("OR {}, {}", ImmediateToDirect),
    /* 0x19 */ op!("OR (X), (Y)", Indirect),
    /* 0x1A */ op!("DECW {}", Direct),
    /* 0x1B */ op!("ASL {}+X", DirectX),
    /* 0x1C */ op!("ASL A", Implied),
    /* 0x1D */ op!("DEC X", Implied),
    /* 0x1E */ op!("CMP X, {}", Absolute),
    /* 0x1F */ op!("JMP [{}+X]", Absolute),
    /* 0x20 */ op!("CLRP", Implied),
    /* 0x21 */ op!("TCALL 2", Implied),
    /* 0x22 */ op!("SET1 {}.1", Direct),
    /* 0x23 */ op!("BBS {}.1, {}", DirectRelative),
    /* 0x24 */ op!("AND A, {}", Direct),
    /* 0x25 */ op!("AND A, {}", Absolute),
    /* 0x26 */ op!("AND A, (X)", Indirect),
    /* 0x27 */ op!("AND A, [{}+X]", XIndirect),
    /* 0x28 */ op!("AND A, {}", Immediate),
    /* 0x29 */ op!("AND {}, {}", DirectToDirect),
    /* 0x2A */ op!("OR1 C, /{}", AbsoluteBit),
    /* 0x2B */ op!("ROL {}", Direct),
    /* 0x2C */ op!("ROL {}", Absolute),
    /* 0x2D */ op!("PUSH A", Implied),
    /* 0x2E */ op!("CBNE {}, {}", DirectRelative),
    /* 0x2F */ op!("BRA {}", Relative),
    /* 0x30 */ op!("BMI {}", Relative),
    /* 0x31 */ op!("TCALL 3", Implied),
    /* 0x32 */ op!("CLR1 {}.1", Direct),
    /* 0x33 */ op!("BBC {}.1, {}", DirectRelative),
    /* 0x34 */ op!("AND A, {}+X", DirectX),
    /* 0x35 */ op!("AND A, {}+X", AbsoluteX),
    /* 0x36 */ op!("AND A, {}+Y", AbsoluteY),
    /* 0x37 */ op!("AND A, [{}]+Y", IndirectY),
    /* 0x38 */ op!("AND {}, {}", ImmediateToDirect),
    /* 0x39 */ op!("AND (X), (Y)", Indirect),
    /* 0x3A */ op!("INCW {}", Direct),
    /* 0x3B */ op!("ROL {}+X", DirectX),
    /* 0x3C */ op!("ROL A", Implied),
    /* 0x3D */ op!("INC X", Implied),
    /* 0x3E */ op!("CMP X, {}", Direct),
    /* 0x3F */ op!("CALL {}", Absolute),
    /* 0x40 */ op!("SETP", Implied),
    /* 0x41 */ op!("TCALL 4", Implied),
    /* 0x42 */ op!("SET1 {}.2", Direct),
    /* 0x43 */ op!("BBS {}.2, {}", DirectRelative),
    /* 0x44 */ op!("EOR A, {}", Direct),
    /* 0x45 */ op!("EOR A, {}", Absolute),
    /* 0x46 */ op!("EOR A, (X)", Indirect),
    /* 0x47 */ op!("EOR A, [{}+X]", XIndirect),
    /* 0x48 */ op!("EOR A, {}", Immediate),
    /* 0x49 */ op!("EOR {}, {}", DirectToDirect),
    /* 0x4A */ op!("AND1 C, {}", AbsoluteBit),
    /* 0x4B */ op!("LSR {}", Direct),
    /* 0x4C */ op!("LSR {}", Absolute),
    /* 0x4D */ op!("PUSH X", Implied),
    /* 0x4E */ op!("TCLR1 {}", Absolute),
    /* 0x4F */ op!("PCALL {}", Immediate),
    /* 0x50 */ op!("BVC {}", Relative),
    /* 0x51 */ op!("TCALL 5", Implied),
    /* 0x52 */ op!("CLR1 {}.2", Direct),
    /* 0x53 */ op!("BBC {}.2, {}", DirectRelative),
    /* 0x54 */ op!("EOR A, {}+X", DirectX),
    /* 0x55 */ op!("EOR A, {}+X", AbsoluteX),
    /* 0x56 */ op!("EOR A, {}+Y", AbsoluteY),
    /* 0x57 */ op!("EOR A, [{}]+Y", IndirectY),
    /* 0x58 */ op!("EOR {}, {}", ImmediateToDirect),
    /* 0x59 */ op!("EOR (X), (Y)", Indirect),
    /* 0x5A */ op!("CMPW YA, {}", Direct),
    /* 0x5B */ op!("LSR {}+X", DirectX),
    /* 0x5C */ op!("LSR A", Implied),
    /* 0x5D */ op!("MOV X, A", Implied),
    /* 0x5E */ op!("CMP Y, {}", Absolute),
    /* 0x5F */ op!("JMP {}", Absolute),
    /* 0x60 */ op!("CLRC", Implied),
    /* 0x61 */ op!("TCALL 6", Implied),
    /* 0x62 */ op!("SET1 {}.3", Direct),
    /* 0x63 */ op!("BBS {}.3, {}", DirectRelative),
    /* 0x64 */ op!("CMP A, {}", Direct),
    /* 0x65 */ op!("CMP A, {}", Absolute),
    /* 0x66 */ op!("CMP A, (X)", Indirect),
    /* 0x67 */ op!("CMP A, [{}+X]", XIndirect),
    /* 0x68 */ op!("CMP A, {}", Immediate),
    /* 0x69 */ op!("CMP {}, {}", DirectToDirect),
    /* 0x6A */ op!("AND1 C, /{}", AbsoluteBit),
    /* 0x6B */ op!("ROR {}", Direct),
    /* 0x6C */ op!("ROR {}", Absolute),
    /* 0x6D */ op!("PUSH Y", Implied),
    /* 0x6E */ op!("DBNZ {}, {}", DirectRelative),
    /* 0x6F */ op!("RET", Implied),
    /* 0x70 */ op!("BVS {}", Relative),
    /* 0x71 */ op!("TCALL 7", Implied),
    /* 0x72 */ op!("CLR1 {}.3", Direct),
    /* 0x73 */ op!("BBC {}.3, {}", DirectRelative),
    /* 0x74 */ op!("CMP A, {}+X", DirectX),
    /* 0x75 */ op!("CMP A, {}+X", AbsoluteX),
    /* 0x76 */ op!("CMP A, {}+Y", AbsoluteY),
    /* 0x77 */ op!("CMP A, [{}]+Y", IndirectY),
    /* 0x78 */ op!("CMP {}, {}", ImmediateToDirect),
    /* 0x79 */ op!("CMP (X), (Y)", Indirect),
    /* 0x7A */ op!("ADDW YA, {}", Direct),
    /* 0x7B */ op!("ROR {}+X", DirectX),
    /* 0x7C */ op!("ROR A", Implied),
    /* 0x7D */ op!("MOV A, X", Implied),
    /* 0x7E */ op!("CMP Y, {}", Direct),
    /* 0x7F */ op!("RET1", Implied),
    /* 0x80 */ op!("SETC", Implied),
    /* 0x81 */ op!("TCALL 8", Implied),
    /* 0x82 */ op!("SET1 {}.4", Direct),
    /* 0x83 */ op!("BBS {}.4, {}", DirectRelative),
    /* 0x84 */ op!("ADC A, {}", Direct),
    /* 0x85 */ op!("ADC A, {}", Absolute),
    /* 0x86 */ op!("ADC A, (X)", Indirect),
    /* 0x87 */ op!("ADC A, [{}+X]", XIndirect),
    /* 0x88 */ op!("ADC A, {}", Immediate),
    /* 0x89 */ op!("ADC {}, {}", DirectToDirect),
    /* 0x8A */ op!("EOR1 C, {}", AbsoluteBit),
    /* 0x8B */ op!("DEC {}", Direct),
    /* 0x8C */ op!("DEC {}", Absolute),
    /* 0x8D */ op!("MOV Y, {}", Immediate),
    /* 0x8E */ op!("POP PSW", Implied),
    /* 0x8F */ op!("MOV {}, {}", ImmediateToDirect),
    /* 0x90 */ op!("BCC {}", Relative),
    /* 0x91 */ op!("TCALL 9", Implied),
    /* 0x92 */ op!("CLR1 {}.4", Direct),
    /* 0x93 */ op!("BBC {}.4, {}", DirectRelative),
    /* 0x94 */ op!("ADC A, {}+X", DirectX),
    /* 0x95 */ op!("ADC A, {}+X", AbsoluteX),
    /* 0x96 */ op!("ADC A, {}+Y", AbsoluteY),
    /* 0x97 */ op!("ADC A, [{}]+Y", IndirectY),
    /* 0x98 */ op!("ADC {}, {}", ImmediateToDirect),
    /* 0x99 */ op!("ADC (X), (Y)", Indirect),
    /* 0x9A */ op!("SUBW YA, {}", Direct),
    /* 0x9B */ op!("DEC {}+X", DirectX),
    /* 0x9C */ op!("DEC A", Implied),
    /* 0x9D */ op!("MOV X, SP", Implied),
    /* 0x9E */ op!("DIV YA, X", Implied),
    /* 0x9F */ op!("XCN A", Implied),
    /* 0xA0 */ op!("EI", Implied),
    /* 0xA1 */ op!("TCALL 10", Implied),
    /* 0xA2 */ op!("SET1 {}.5", Direct),
    /* 0xA3 */ op!("BBS {}.5, {}", DirectRelative),
    /* 0xA4 */ op!("SBC A, {}", Direct),
    /* 0xA5 */ op!("SBC A, {}", Absolute),
    /* 0xA6 */ op!("SBC A, (X)", Indirect),
    /* 0xA7 */ op!("SBC A, [{}+X]", XIndirect),
    /* 0xA8 */ op!("SBC A, {}", Immediate),
    /* 0xA9 */ op!("SBC {}, {}", DirectToDirect),
    /* 0xAA */ op!("MOV1 C, {}", AbsoluteBit),
    /* 0xAB */ op!("INC {}", Direct),
    /* 0xAC */ op!("INC {}", Absolute),
    /* 0xAD */ op!("CMP Y, {}", Immediate),
    /* 0xAE */ op!("POP A", Implied),
    /* 0xAF */ op!("MOV (X)+, A", IndirectAutoInc),
    /* 0xB0 */ op!("BCS {}", Relative),
    /* 0xB1 */ op!("TCALL 11", Implied),
    /* 0xB2 */ op!("CLR1 {}.5", Direct),
    /* 0xB3 */ op!("BBC {}.5, {}", DirectRelative),
    /* 0xB4 */ op!("SBC A, {}+X", DirectX),
    /* 0xB5 */ op!("SBC A, {}+X", AbsoluteX),
    /* 0xB6 */ op!("SBC A, {}+Y", AbsoluteY),
    /* 0xB7 */ op!("SBC A, [{}]+Y", IndirectY),
    /* 0xB8 */ op!("SBC {}, {}", ImmediateToDirect),
    /* 0xB9 */ op!("SBC (X), (Y)", Indirect),
    /* 0xBA */ op!("MOVW YA, {}", Direct),
    /* 0xBB */ op!("INC {}+X", DirectX),
    /* 0xBC */ op!("INC A", Implied),
    /* 0xBD */ op!("MOV SP, X", Implied),
    /* 0xBE */ op!("DAS A", Implied),
    /* 0xBF */ op!("MOV A, (X)+", IndirectAutoInc),
    /* 0xC0 */ op!("DI", Implied),
    /* 0xC1 */ op!("TCALL 12", Implied),
    /* 0xC2 */ op!("SET1 {}.6", Direct),
    /* 0xC3 */ op!("BBS {}.6, {}", DirectRelative),
    /* 0xC4 */ op!("MOV {}, A", Direct),
    /* 0xC5 */ op!("MOV {}, A", Absolute),
    /* 0xC6 */ op!("MOV (X), A", Indirect),
    /* 0xC7 */ op!("MOV [{}+X], A", XIndirect),
    /* 0xC8 */ op!("CMP X, {}", Immediate),
    /* 0xC9 */ op!("MOV {}, X", Absolute),
    /* 0xCA */ op!("MOV1 {}, C", AbsoluteBit),
    /* 0xCB */ op!("MOV {}, Y", Direct),
    /* 0xCC */ op!("MOV {}, Y", Absolute),
    /* 0xCD */ op!("MOV X, {}", Immediate),
    /* 0xCE */ op!("POP X", Implied),
    /* 0xCF */ op!("MUL YA", Implied),
    /* 0xD0 */ op!("BNE {}", Relative),
    /* 0xD1 */ op!("TCALL 13", Implied),
    /* 0xD2 */ op!("CLR1 {}.6", Direct),
    /* 0xD3 */ op!("BBC {}.6, {}", DirectRelative),
    /* 0xD4 */ op!("MOV {}+X, A", DirectX),
    /* 0xD5 */ op!("MOV {}+X, A", AbsoluteX),
    /* 0xD6 */ op!("MOV {}+Y, A", AbsoluteY),
    /* 0xD7 */ op!("MOV [{}]+Y, A", IndirectY),
    /* 0xD8 */ op!("MOV {}, X", Direct),
    /* 0xD9 */ op!("MOV {}+Y, X", DirectY),
    /* 0xDA */ op!("MOVW {}, YA", Direct),
    /* 0xDB */ op!("MOV {}+X, Y", DirectX),
    /* 0xDC */ op!("DEC Y", Implied),
    /* 0xDD */ op!("MOV A, Y", Implied),
    /* 0xDE */ op!("CBNE {}+X, {}", DirectRelative),
    /* 0xDF */ op!("DAA A", Implied),
    /* 0xE0 */ op!("CLRV", Implied),
    /* 0xE1 */ op!("TCALL 14", Implied),
    /* 0xE2 */ op!("SET1 {}.7", Direct),
    /* 0xE3 */ op!("BBS {}.7, {}", DirectRelative),
    /* 0xE4 */ op!("MOV A, {}", Direct),
    /* 0xE5 */ op!("MOV A, {}", Absolute),
    /* 0xE6 */ op!("MOV A, (X)", Indirect),
    /* 0xE7 */ op!("MOV A, [{}+X]", XIndirect),
    /* 0xE8 */ op!("MOV A, {}", Immediate),
    /* 0xE9 */ op!("MOV X, {}", Absolute),
    /* 0xEA */ op!("NOT1 {}", AbsoluteBit),
    /* 0xEB */ op!("MOV Y, {}", Direct),
    /* 0xEC */ op!("MOV Y, {}", Absolute),
    /* 0xED */ op!("NOTC", Implied),
    /* 0xEE */ op!("POP Y", Implied),
    /* 0xEF */ op!("SLEEP", Implied),
    /* 0xF0 */ op!("BEQ {}", Relative),
    /* 0xF1 */ op!("TCALL 15", Implied),
    /* 0xF2 */ op!("CLR1 {}.7", Direct),
    /* 0xF3 */ op!("BBC {}.7, {}", DirectRelative),
    /* 0xF4 */ op!("MOV A, {}+X", DirectX),
    /* 0xF5 */ op!("MOV A, {}+X", AbsoluteX),
    /* 0xF6 */ op!("MOV A, {}+Y", AbsoluteY),
    /* 0xF7 */ op!("MOV A, [{}]+Y", IndirectY),
    /* 0xF8 */ op!("MOV X, {}", Direct),
    /* 0xF9 */ op!("MOV X, {}+Y", DirectY),
    /* 0xFA */ op!("MOV {}, {}", DirectToDirect),
    /* 0xFB */ op!("MOV Y, {}+X", DirectX),
    /* 0xFC */ op!("INC Y", Implied),
    /* 0xFD */ op!("MOV Y, A", Implied),
    /* 0xFE */ op!("DBNZ Y, {}", Relative),
    /* 0xFF */ op!("STOP", Implied),
];

pub fn disassemble_one(mem: &[u8], addr: u16) -> DisasmLine {
    let opcode = read_byte(mem, addr);
    let info = &OPCODE_TABLE[opcode as usize];
    let total_len = info.bytes() as usize;

    let bytes: Vec<u8> = (0..total_len).map(|i| read_byte(mem, addr + i as u16)).collect();
    let operand_bytes = &bytes[1..];

    let operands = decode_operands(info.mode, addr, operand_bytes);

    DisasmLine {
        addr,
        bytes,
        mnemonic: info.mnemonic,
        operands,
    }
}

// Wrapping read guards against falling off the end of `mem` near $FFFF,
// where a 2-byte operand could otherwise straddle the boundary.
fn read_byte(mem: &[u8], addr: u16) -> u8 {
    mem[(addr as usize) % mem.len()]
}

pub fn decode_operands(mode: AddressingMode, addr: u16, bytes: &[u8]) -> Vec<DisasmOperand> {
    use AddressingMode::*;
    use DisasmOperandKind::*;

    let next_pc = addr.wrapping_add(1 + mode.operand_len() as u16);

    match mode {
        Implied | Indirect | IndirectAutoInc => vec![],

        Direct | DirectX | DirectY | XIndirect | IndirectY => {
            vec![DisasmOperand { kind: DirectPage(bytes[0]) }]
        }

        Immediate => vec![DisasmOperand { kind: Immediate8(bytes[0]) }],

        Relative => {
            let offset = bytes[0] as i8 as i16;
            vec![DisasmOperand { kind: BranchTarget(next_pc.wrapping_add(offset as u16)) }]
        }

        AddressingMode::Absolute | AbsoluteX | AbsoluteY => {
            vec![DisasmOperand { kind: DisasmOperandKind::Absolute(u16::from_le_bytes([bytes[0], bytes[1]])) }]
        }

        AddressingMode::AbsoluteBit => {
            let raw = u16::from_le_bytes([bytes[0], bytes[1]]);
            vec![DisasmOperand { kind: DisasmOperandKind::AbsoluteBit(raw & 0x1FFF, (raw >> 13) as u8) }]
        }

        DirectToDirect => vec![
            DisasmOperand { kind: DirectPage(bytes[0]) },
            DisasmOperand { kind: DirectPage(bytes[1]) },
        ],

        ImmediateToDirect => vec![
            DisasmOperand { kind: DirectPage(bytes[0]) },
            DisasmOperand { kind: Immediate8(bytes[1]) },
        ],

        DirectRelative => {
            let offset = bytes[1] as i8 as i16;
            vec![
                DisasmOperand { kind: DirectPage(bytes[0]) },
                DisasmOperand { kind: BranchTarget(next_pc.wrapping_add(offset as u16)) },
            ]
        }
    }
}

fn relative_target(instr_addr: u16, instr_len: usize, offset_byte: u8) -> u16 {
    let offset = offset_byte as i8 as i32; // sign-extend
    let next_pc = instr_addr as i32 + instr_len as i32;
    (next_pc + offset) as u16
}

pub fn disassemble_range(mem: &[u8], ipl_rom: &[u8], ipl_read_en: bool, start_addr: u16, count: usize) -> Vec<DisasmLine> {
    let mut lines = Vec::with_capacity(count);
    let mut addr = start_addr;
    for _ in 0..count {
        let line = if addr >= 0xFFC0 && ipl_read_en {
            disassemble_one(ipl_rom, addr)
        } else {
            disassemble_one(mem, addr)
        };

        addr += line.bytes.len() as u16;
        lines.push(line);
    }
    lines
}