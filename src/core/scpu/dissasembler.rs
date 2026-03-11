use crate::core::scpu::{Cpu65c816, Flag, bus::{Address, CpuBus}};

#[derive(Clone, Copy)]
enum AddressingMode {
    Implied,
    Accumulator,
    Immediate8,
    Immediate16,
    ImmediateM,      // 8 or 16 bit depending on M flag
    ImmediateX,      // 8 or 16 bit depending on X flag
    Relative8,
    Relative16,
    Direct,
    DirectX,
    DirectY,
    DirectIndirect,
    DirectIndirectLong,
    DirectXIndirect,
    DirectIndirectY,
    DirectIndirectLongY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Long,
    LongX,
    AbsoluteIndirect,
    LongIndirect,
    AbsoluteXIndirect,
    StackRelative,
    StackRelativeIndirectY,
    SrcDst,
}

#[derive(Clone, Copy)]
struct DisassembleData {
    pub name: &'static str,
    pub addr_mode: AddressingMode,
}

// This table would be defined elsewhere with all 256 entries
static DISASSEMBLE_TABLE: [DisassembleData; 256] = [
    DisassembleData {name: "brk", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "ora", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {name: "cop", addr_mode: AddressingMode::Immediate8},
    DisassembleData {name: "ora", addr_mode: AddressingMode::StackRelative},
    DisassembleData {name: "tsb", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "ora", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "asl", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "ora", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {name: "php", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "ora", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {name: "asl", addr_mode: AddressingMode::Accumulator},
    DisassembleData {name: "phd", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "tsb", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "ora", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "asl", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "ora", addr_mode: AddressingMode::Long},
    DisassembleData {name: "bpl", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "ora", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {name: "ora", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {name: "ora", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {name: "trb", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "ora", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "asl", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "ora", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {name: "clc", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "ora", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "inc", addr_mode: AddressingMode::Accumulator},
    DisassembleData {name: "tcs", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "trb", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "ora", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "asl", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "ora", addr_mode: AddressingMode::LongX},
    DisassembleData {name: "jsr", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "and", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {name: "jsl", addr_mode: AddressingMode::Long},
    DisassembleData {name: "and", addr_mode: AddressingMode::StackRelative},
    DisassembleData {name: "bit", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "and", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "rol", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "and", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {name: "plp", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "and", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {name: "rol", addr_mode: AddressingMode::Accumulator},
    DisassembleData {name: "pld", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "bit", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "and", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "rol", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "and", addr_mode: AddressingMode::Long},
    DisassembleData {name: "bmi", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "and", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {name: "and", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {name: "and", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {name: "bit", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "and", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "rol", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "and", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {name: "sec", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "and", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "dec", addr_mode: AddressingMode::Accumulator},
    DisassembleData {name: "tsc", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "bit", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "and", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "rol", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "and", addr_mode: AddressingMode::LongX},
    DisassembleData {name: "rti", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "eor", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {name: "wdm", addr_mode: AddressingMode::Immediate8},
    DisassembleData {name: "eor", addr_mode: AddressingMode::StackRelative},
    DisassembleData {name: "mvp", addr_mode: AddressingMode::SrcDst},
    DisassembleData {name: "eor", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "lsr", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "eor", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {name: "pha", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "eor", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {name: "lsr", addr_mode: AddressingMode::Accumulator},
    DisassembleData {name: "phk", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "jmp", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "eor", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "lsr", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "eor", addr_mode: AddressingMode::Long},
    DisassembleData {name: "bvc", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "eor", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {name: "eor", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {name: "eor", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {name: "mvn", addr_mode: AddressingMode::SrcDst},
    DisassembleData {name: "eor", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "lsr", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "eor", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {name: "cli", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "eor", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "phy", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "tcd", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "jmp", addr_mode: AddressingMode::Long},
    DisassembleData {name: "eor", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "lsr", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "eor", addr_mode: AddressingMode::LongX},
    DisassembleData {name: "rts", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "adc", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {name: "per", addr_mode: AddressingMode::Immediate8},
    DisassembleData {name: "adc", addr_mode: AddressingMode::StackRelative},
    DisassembleData {name: "stz", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "adc", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "ror", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "adc", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {name: "pla", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "adc", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {name: "ror", addr_mode: AddressingMode::Accumulator},
    DisassembleData {name: "rtl", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "jmp", addr_mode: AddressingMode::AbsoluteIndirect},
    DisassembleData {name: "adc", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "ror", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "adc", addr_mode: AddressingMode::Long},
    DisassembleData {name: "bvs", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "adc", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {name: "adc", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {name: "adc", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {name: "stz", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "adc", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "ror", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "adc", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {name: "sei", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "adc", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "ply", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "tdc", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "jmp", addr_mode: AddressingMode::AbsoluteXIndirect},
    DisassembleData {name: "adc", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "ror", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "adc", addr_mode: AddressingMode::LongX},
    DisassembleData {name: "bra", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "sta", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {name: "brl", addr_mode: AddressingMode::Relative16},
    DisassembleData {name: "sta", addr_mode: AddressingMode::StackRelative},
    DisassembleData {name: "sty", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "sta", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "stx", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "sta", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {name: "dey", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "bit", addr_mode: AddressingMode::Immediate8},
    DisassembleData {name: "txa", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "phb", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "sty", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "sta", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "stx", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "sta", addr_mode: AddressingMode::Long},
    DisassembleData {name: "bcc", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "sta", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {name: "sta", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {name: "sta", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {name: "sty", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "sta", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "stx", addr_mode: AddressingMode::DirectY},
    DisassembleData {name: "sta", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {name: "tya", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "sta", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "txs", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "txy", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "stz", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "sta", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "stz", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "sta", addr_mode: AddressingMode::LongX},
    DisassembleData {name: "ldy", addr_mode: AddressingMode::ImmediateX},
    DisassembleData {name: "lda", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {name: "ldx", addr_mode: AddressingMode::ImmediateX},
    DisassembleData {name: "lda", addr_mode: AddressingMode::StackRelative},
    DisassembleData {name: "ldy", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "lda", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "ldx", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "lda", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {name: "tay", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "lda", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {name: "tax", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "plb", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "ldy", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "lda", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "ldx", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "lda", addr_mode: AddressingMode::Long},
    DisassembleData {name: "bcs", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "lda", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {name: "lda", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {name: "lda", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {name: "ldy", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "lda", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "ldx", addr_mode: AddressingMode::DirectY},
    DisassembleData {name: "lda", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {name: "clv", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "lda", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "tsx", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "tyx", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "ldy", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "lda", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "ldx", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "lda", addr_mode: AddressingMode::LongX},
    DisassembleData {name: "cpy", addr_mode: AddressingMode::ImmediateX},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {name: "rep", addr_mode: AddressingMode::Immediate8},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::StackRelative},
    DisassembleData {name: "cpy", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "dec", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {name: "iny", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {name: "dex", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "wai", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "cpy", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "dec", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::Long},
    DisassembleData {name: "bne", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {name: "pei", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "dec", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {name: "cld", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "phx", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "stp", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "jmp", addr_mode: AddressingMode::LongIndirect},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "dec", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "cmp", addr_mode: AddressingMode::LongX},
    DisassembleData {name: "cpx", addr_mode: AddressingMode::ImmediateX},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::DirectXIndirect},
    DisassembleData {name: "sep", addr_mode: AddressingMode::Immediate8},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::StackRelative},
    DisassembleData {name: "cpx", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "inc", addr_mode: AddressingMode::Direct},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::DirectIndirectLong},
    DisassembleData {name: "inx", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::ImmediateM},
    DisassembleData {name: "nop", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "xba", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "cpx", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "inc", addr_mode: AddressingMode::Absolute},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::Long},
    DisassembleData {name: "beq", addr_mode: AddressingMode::Relative8},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::DirectIndirectY},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::DirectIndirect},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::StackRelativeIndirectY},
    DisassembleData {name: "pea", addr_mode: AddressingMode::Immediate16},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "inc", addr_mode: AddressingMode::DirectX},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::DirectIndirectLongY},
    DisassembleData {name: "sed", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::AbsoluteY},
    DisassembleData {name: "plx", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "xcefc", addr_mode: AddressingMode::Implied},
    DisassembleData {name: "jsr", addr_mode: AddressingMode::AbsoluteXIndirect},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "inc", addr_mode: AddressingMode::AbsoluteX},
    DisassembleData {name: "sbc", addr_mode: AddressingMode::LongX},
];

macro_rules! read_byte {
    ($bus:expr, $bank:expr, $offset:expr) => {
        $bus.read(Address { bank: $bank, offset: $offset })
    };
}

macro_rules! read_word {
    ($bus:expr, $bank:expr, $offset:expr) => {
        {
            let lo = read_byte!($bus, $bank, $offset) as u16;
            let hi = read_byte!($bus, $bank, $offset + 1) as u16;
            (hi << 8) | lo
        }
    };
}

/// Returns the hardware register name for a given SNES MMIO address, if known
fn get_register_name(addr: u32) -> Option<&'static str> {
    // SNES MMIO is mirrored, so we only care about the lower 16 bits
    // and registers are in bank $00 (or mirrored banks)
    let addr = addr & 0xFFFF;
    
    match addr {
        // PPU Registers ($2100-$213F)
        0x2100 => Some("INIDISP"),
        0x2101 => Some("OBSEL"),
        0x2102 => Some("OAMADDL"),
        0x2103 => Some("OAMADDH"),
        0x2104 => Some("OAMDATA"),
        0x2105 => Some("BGMODE"),
        0x2106 => Some("MOSAIC"),
        0x2107 => Some("BG1SC"),
        0x2108 => Some("BG2SC"),
        0x2109 => Some("BG3SC"),
        0x210A => Some("BG4SC"),
        0x210B => Some("BG12NBA"),
        0x210C => Some("BG34NBA"),
        0x210D => Some("BG1HOFS"),
        0x210E => Some("BG1VOFS"),
        0x210F => Some("BG2HOFS"),
        0x2110 => Some("BG2VOFS"),
        0x2111 => Some("BG3HOFS"),
        0x2112 => Some("BG3VOFS"),
        0x2113 => Some("BG4HOFS"),
        0x2114 => Some("BG4VOFS"),
        0x2115 => Some("VMAIN"),
        0x2116 => Some("VMADDL"),
        0x2117 => Some("VMADDH"),
        0x2118 => Some("VMDATAL"),
        0x2119 => Some("VMDATAH"),
        0x211A => Some("M7SEL"),
        0x211B => Some("M7A"),
        0x211C => Some("M7B"),
        0x211D => Some("M7C"),
        0x211E => Some("M7D"),
        0x211F => Some("M7X"),
        0x2120 => Some("M7Y"),
        0x2121 => Some("CGADD"),
        0x2122 => Some("CGDATA"),
        0x2123 => Some("W12SEL"),
        0x2124 => Some("W34SEL"),
        0x2125 => Some("WOBJSEL"),
        0x2126 => Some("WH0"),
        0x2127 => Some("WH1"),
        0x2128 => Some("WH2"),
        0x2129 => Some("WH3"),
        0x212A => Some("WBGLOG"),
        0x212B => Some("WOBJLOG"),
        0x212C => Some("TM"),
        0x212D => Some("TS"),
        0x212E => Some("TMW"),
        0x212F => Some("TSW"),
        0x2130 => Some("CGWSEL"),
        0x2131 => Some("CGADSUB"),
        0x2132 => Some("COLDATA"),
        0x2133 => Some("SETINI"),
        0x2134 => Some("MPYL"),
        0x2135 => Some("MPYM"),
        0x2136 => Some("MPYH"),
        0x2137 => Some("SLHV"),
        0x2138 => Some("OAMDATAREAD"),
        0x2139 => Some("VMDATALREAD"),
        0x213A => Some("VMDATAHREAD"),
        0x213B => Some("CGDATAREAD"),
        0x213C => Some("OPHCT"),
        0x213D => Some("OPVCT"),
        0x213E => Some("STAT77"),
        0x213F => Some("STAT78"),
        
        // APU Registers ($2140-$2143)
        0x2140 => Some("APUIO0"),
        0x2141 => Some("APUIO1"),
        0x2142 => Some("APUIO2"),
        0x2143 => Some("APUIO3"),
        
        // WRAM Access ($2180-$2183)
        0x2180 => Some("WMDATA"),
        0x2181 => Some("WMADDL"),
        0x2182 => Some("WMADDM"),
        0x2183 => Some("WMADDH"),
        
        // CPU Registers ($4200-$421F)
        0x4200 => Some("NMITIMEN"),
        0x4201 => Some("WRIO"),
        0x4202 => Some("WRMPYA"),
        0x4203 => Some("WRMPYB"),
        0x4204 => Some("WRDIVL"),
        0x4205 => Some("WRDIVH"),
        0x4206 => Some("WRDIVB"),
        0x4207 => Some("HTIMEL"),
        0x4208 => Some("HTIMEH"),
        0x4209 => Some("VTIMEL"),
        0x420A => Some("VTIMEH"),
        0x420B => Some("MDMAEN"),
        0x420C => Some("HDMAEN"),
        0x420D => Some("MEMSEL"),
        0x4210 => Some("RDNMI"),
        0x4211 => Some("TIMEUP"),
        0x4212 => Some("HVBJOY"),
        0x4213 => Some("RDIO"),
        0x4214 => Some("RDDIVL"),
        0x4215 => Some("RDDIVH"),
        0x4216 => Some("RDMPYL"),
        0x4217 => Some("RDMPYH"),
        0x4218 => Some("JOY1L"),
        0x4219 => Some("JOY1H"),
        0x421A => Some("JOY2L"),
        0x421B => Some("JOY2H"),
        0x421C => Some("JOY3L"),
        0x421D => Some("JOY3H"),
        0x421E => Some("JOY4L"),
        0x421F => Some("JOY4H"),
        
        // DMA Registers ($4300-$43FF) - Channel 0-7
        addr if (0x4300..=0x437F).contains(&addr) => {
            let channel = (addr >> 4) & 0x7;
            let reg = addr & 0xF;
            match reg {
                0x0 => Some(match channel {
                    0 => "DMAP0", 1 => "DMAP1", 2 => "DMAP2", 3 => "DMAP3",
                    4 => "DMAP4", 5 => "DMAP5", 6 => "DMAP6", _ => "DMAP7",
                }),
                0x1 => Some(match channel {
                    0 => "BBAD0", 1 => "BBAD1", 2 => "BBAD2", 3 => "BBAD3",
                    4 => "BBAD4", 5 => "BBAD5", 6 => "BBAD6", _ => "BBAD7",
                }),
                0x2 => Some(match channel {
                    0 => "A1T0L", 1 => "A1T1L", 2 => "A1T2L", 3 => "A1T3L",
                    4 => "A1T4L", 5 => "A1T5L", 6 => "A1T6L", _ => "A1T7L",
                }),
                0x3 => Some(match channel {
                    0 => "A1T0H", 1 => "A1T1H", 2 => "A1T2H", 3 => "A1T3H",
                    4 => "A1T4H", 5 => "A1T5H", 6 => "A1T6H", _ => "A1T7H",
                }),
                0x4 => Some(match channel {
                    0 => "A1B0", 1 => "A1B1", 2 => "A1B2", 3 => "A1B3",
                    4 => "A1B4", 5 => "A1B5", 6 => "A1B6", _ => "A1B7",
                }),
                0x5 => Some(match channel {
                    0 => "DAS0L", 1 => "DAS1L", 2 => "DAS2L", 3 => "DAS3L",
                    4 => "DAS4L", 5 => "DAS5L", 6 => "DAS6L", _ => "DAS7L",
                }),
                0x6 => Some(match channel {
                    0 => "DAS0H", 1 => "DAS1H", 2 => "DAS2H", 3 => "DAS3H",
                    4 => "DAS4H", 5 => "DAS5H", 6 => "DAS6H", _ => "DAS7H",
                }),
                0x7 => Some(match channel {
                    0 => "DASB0", 1 => "DASB1", 2 => "DASB2", 3 => "DASB3",
                    4 => "DASB4", 5 => "DASB5", 6 => "DASB6", _ => "DASB7",
                }),
                0x8 => Some(match channel {
                    0 => "A2A0L", 1 => "A2A1L", 2 => "A2A2L", 3 => "A2A3L",
                    4 => "A2A4L", 5 => "A2A5L", 6 => "A2A6L", _ => "A2A7L",
                }),
                0x9 => Some(match channel {
                    0 => "A2A0H", 1 => "A2A1H", 2 => "A2A2H", 3 => "A2A3H",
                    4 => "A2A4H", 5 => "A2A5H", 6 => "A2A6H", _ => "A2A7H",
                }),
                0xA => Some(match channel {
                    0 => "NTRL0", 1 => "NTRL1", 2 => "NTRL2", 3 => "NTRL3",
                    4 => "NTRL4", 5 => "NTRL5", 6 => "NTRL6", _ => "NTRL7",
                }),
                0xB => Some(match channel {
                    0 => "UNUSED0", 1 => "UNUSED1", 2 => "UNUSED2", 3 => "UNUSED3",
                    4 => "UNUSED4", 5 => "UNUSED5", 6 => "UNUSED6", _ => "UNUSED7",
                }),
                _ => None,
            }
        }
        
        _ => None,
    }
}

/// Formats an absolute address, optionally replacing with register name
fn format_absolute(addr: u16) -> String {
    match get_register_name(addr as u32) {
        Some(name) => name.to_string(),
        None => format!("${:04X}", addr),
    }
}

/// Formats an absolute long address, optionally replacing with register name
fn format_absolute_long(addr: u32) -> String {
    // Only check for register names in bank $00 or $80 (mirror)
    let bank = (addr >> 16) & 0xFF;
    if bank != 0x00 && bank != 0x80 {
        return format!("${:06X}", addr);
    }
    
    match get_register_name(addr) {
        Some(name) => name.to_string(),
        None => format!("${:06X}", addr),
    }
}

/// Formats a direct page address, optionally replacing with register name
/// Note: This resolves the effective address using the direct page register
fn format_direct(dp: u16, dp_offset: u8) -> String {
    if dp != 0 {
        return format!("${:02X}", dp_offset);
    }
    
    match get_register_name(dp_offset as u32) {
        Some(name) => name.to_string(),
        None => format!("${:02X}", dp_offset),
    }
}

macro_rules! read_long {
    ($bus:expr, $bank:expr, $offset:expr) => {
        {
            let lo = read_byte!($bus, $bank, $offset) as u32;
            let mid = read_byte!($bus, $bank, $offset + 1) as u32;
            let hi = read_byte!($bus, $bank, $offset + 2) as u32;
            (hi << 16) | (mid << 8) | lo
        }
    };
}

fn _disassemble(bus: &mut CpuBus, bank: u8, pc: u16, dp: u16, flag_m: bool, flag_x: bool) -> (String, u16) {
    let op = read_byte!(bus, bank, pc);
    
    let data = &DISASSEMBLE_TABLE[op as usize];
    
    let (operand, bytes) = match data.addr_mode {
        AddressingMode::Implied => (String::new(), 0),
        
        AddressingMode::Accumulator => ("A".to_string(), 0),
        
        AddressingMode::Immediate8 => {
            (format!("#${:02X}", read_byte!(bus, bank, pc + 1)), 1)
        }
        
        AddressingMode::Immediate16 => {
            (format!("#${:04X}", read_word!(bus, bank, pc + 1)), 2)
        }
        
        AddressingMode::ImmediateM => {
            if flag_m {
                (format!("#${:02X}", read_byte!(bus, bank, pc + 1)), 1)
            } else {
                (format!("#${:04X}", read_word!(bus, bank, pc + 1)), 2)
            }
        }
        
        AddressingMode::ImmediateX => {
            if flag_x {
                (format!("#${:02X}", read_byte!(bus, bank, pc + 1)), 1)
            } else {
                (format!("#${:04X}", read_word!(bus, bank, pc + 1)), 2)
            }
        }
        
        AddressingMode::Relative8 => {
            let offset = read_byte!(bus, bank, pc + 1) as i8;
            let target = pc + 1 + offset as u16;
            (format!("${:04X}", target), 1)
        }
        
        AddressingMode::Relative16 => {
            let offset = read_word!(bus, bank, pc + 1) as i16;
            let target = pc + 2 + offset as u16;
            (format!("${:04X}", target), 2)
        }
        
        AddressingMode::Direct => {
            (format_direct(dp, read_byte!(bus, bank, pc + 1)), 1)
        }
        
        AddressingMode::DirectX => {
            (format!("{},X", format_direct(dp, read_byte!(bus, bank, pc + 1))), 1)
        }
        
        AddressingMode::DirectY => {
            (format!("{},Y", format_direct(dp, read_byte!(bus, bank, pc + 1))), 1)
        }
        
        AddressingMode::DirectIndirect => {
            (format!("({})", format_direct(dp, read_byte!(bus, bank, pc + 1))), 1)
        }
        
        AddressingMode::DirectIndirectLong => {
            (format!("[{}]", format_direct(dp, read_byte!(bus, bank, pc + 1))), 1)
        }
        
        AddressingMode::DirectXIndirect => {
            (format!("({},X)", format_direct(dp, read_byte!(bus, bank, pc + 1))), 1)
        }
        
        AddressingMode::DirectIndirectY => {
            (format!("({}),Y", format_direct(dp, read_byte!(bus, bank, pc + 1))), 1)
        }
        
        AddressingMode::DirectIndirectLongY => {
            (format!("[{}],Y", format_direct(dp, read_byte!(bus, bank, pc + 1))), 1)
        }
        
        AddressingMode::Absolute => {
            (format_absolute(read_word!(bus, bank, pc + 1)), 2)
        }
        
        AddressingMode::AbsoluteX => {
            (format!("{},X", format_absolute(read_word!(bus, bank, pc + 1))), 2)
        }
        
        AddressingMode::AbsoluteY => {
            (format!("{},Y", format_absolute(read_word!(bus, bank, pc + 1))), 2)
        }
        
        AddressingMode::Long => {
            (format_absolute_long(read_long!(bus, bank, pc + 1)), 3)
        }
        
        AddressingMode::LongX => {
            (format!("{},X", format_absolute_long(read_long!(bus, bank, pc + 1))), 3)
        }
        
        AddressingMode::AbsoluteIndirect => {
            (format!("({})", format_absolute(read_word!(bus, bank, pc + 1))), 2)
        }
        
        AddressingMode::LongIndirect => {
            (format!("[{}]", format_absolute(read_word!(bus, bank, pc + 1))), 2)
        }
        
        AddressingMode::AbsoluteXIndirect => {
            (format!("({},X)", format_absolute(read_word!(bus, bank, pc + 1))), 2)
        }
        
        AddressingMode::StackRelative => {
            (format!("${:02X},S", read_byte!(bus, bank, pc + 1)), 1)
        }
        
        AddressingMode::StackRelativeIndirectY => {
            (format!("(${:02X},S),Y", read_byte!(bus, bank, pc + 1)), 1)
        }
        
        AddressingMode::SrcDst => {
            let dst = read_byte!(bus, bank, pc + 1);
            let src = read_byte!(bus, bank, pc + 2);
            (format!("${:02X},${:02X}", src, dst), 2)
        }
    };
    
    let instr_str = if operand.is_empty() {
        data.name.to_string()
    } else {
        format!("{} {}", data.name, operand)
    };
    
    (instr_str, pc + bytes + 1)
}

pub fn disassemble(
    cpu: &Cpu65c816,
    bus: &mut CpuBus,
) -> String {
    let (instr_str, _) = _disassemble(
        bus, 
        cpu.pb, 
        cpu.pc, 
        cpu.dp, 
        cpu.is_flag_set(Flag::FlagM),
        cpu.is_flag_set(Flag::FlagX)
    );
    instr_str
}

/// Information about the state of the cpu that is assumed while disassembling the block.
/// This can affect things like how many bytes are read for an instruction and which
/// addresses are recognized as hardware registers.
pub struct ExecuteState {
    dp: u16,
    flag_m: bool,
    flag_x: bool,
}

pub fn disassemble_block(bus: &mut CpuBus, bank: u8, pc: u16, end_pc: u16, state: Option<ExecuteState>) -> Vec<String> {
    let mut disassembly = Vec::new();
    let mut pc = pc;
    
    let state = state.unwrap_or(ExecuteState { dp: 0, flag_m: false, flag_x: false });
    
    while pc < end_pc {
        let (instr_str, new_pc) = _disassemble(
            bus,
            bank,
            pc,
            state.dp,
            state.flag_m,
            state.flag_x
        );
        
        disassembly.push(instr_str);
        
        pc = new_pc;
    }
    
    disassembly
}