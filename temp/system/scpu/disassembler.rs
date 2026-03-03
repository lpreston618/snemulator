use crate::system::{cartridge::MappingMode, scpu::utils::{map_exhirom_addr, map_hirom_addr, map_lorom_addr}};

pub(super) fn instr_disassembly(
    prg_bank: u8, pc: u16, rom: &[u8], rom_mirror: usize, 
    m8: bool, x8: bool, e: bool, mapping_mode: MappingMode) -> String {
    // "${Bank}{PC}:  {Op} {B1?} {B2?} {B3?}  {instr} [addrmode_data]  (e={e}, x8={x8}, m8={m8})"
    let mut pc = pc;

    let prg_addr = ((prg_bank as usize) << 16) | pc as usize;
    let opcode_addr = pc;

    let mut read_prg = || -> u8 {
        let prg_addr = ((prg_bank as u32) << 16) | pc as u32;
        let mapped_addr = match mapping_mode {
            MappingMode::LoROM => map_lorom_addr(prg_addr),
            MappingMode::HiROM => map_hirom_addr(prg_addr),
            MappingMode::ExHiROM => map_exhirom_addr(prg_addr)
        } as usize;
        let data = rom[mapped_addr & rom_mirror];
        pc += 1;
        data
    };
    
    let opcode = read_prg();
    let b0 = read_prg();
    let b1 = read_prg();
    let b2 = read_prg();

    let instr_start: String = format!("${prg_addr:06X}:  {opcode:02X}", );
    let instr_name: String;
    let addr_mode_fmt: String;
    let instr_data: String;

    match (opcode, e, m8, x8) {
        (0x00, ..) => {
            instr_name = "brk".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x01, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},X)").to_string();
        }
        (0x02, ..) => {
            instr_name = "cop".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0x03, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},S").to_string();
        }
        (0x04, ..) => {
            instr_name = "tsb".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x05, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x06, ..) => {
            instr_name = "asl".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x07, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x08, ..) => {
            instr_name = "php".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x09, _, true, _) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0x09, _, false, _) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0x0A, ..) => {
            instr_name = "asl".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x0B, ..) => {
            instr_name = "phd".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x0C, ..) => {
            instr_name = "tsb".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x0D, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x0E, ..) => {
            instr_name = "asl".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x0F, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0x10, ..) => {
            instr_name = "bpl".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0x11, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X}),Y").to_string();
        }
        (0x12, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x13, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},S),Y").to_string();
        }
        (0x14, ..) => {
            instr_name = "trb".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x15, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x16, ..) => {
            instr_name = "asl".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x17, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("[${b0:02X}],Y").to_string();
        }
        (0x18, ..) => {
            instr_name = "clc".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x19, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0x1A, ..) => {
            instr_name = "inc".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x1B, ..) => {
            instr_name = "tcs".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x1C, ..) => {
            instr_name = "trb".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x1D, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x1E, ..) => {
            instr_name = "asl".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x1F, ..) => {
            instr_name = "ora".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X},X").to_string();
        }
        (0x20, ..) => {
            instr_name = "jsr".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x21, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},X)").to_string();
        }
        (0x22, ..) => {
            instr_name = "jsl".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0x23, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},S").to_string();
        }
        (0x24, ..) => {
            instr_name = "bit".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x25, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x26, ..) => {
            instr_name = "rol".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x27, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x28, ..) => {
            instr_name = "plp".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x29, _, true, _) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0x29, _, false, _) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0x2A, ..) => {
            instr_name = "rol".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x2B, ..) => {
            instr_name = "pld".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x2C, ..) => {
            instr_name = "bit".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x2D, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x2E, ..) => {
            instr_name = "rol".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x2F, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0x30, ..) => {
            instr_name = "bmi".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0x31, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X}),Y").to_string();
        }
        (0x32, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x33, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},S),Y").to_string();
        }
        (0x34, ..) => {
            instr_name = "bit".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x35, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x36, ..) => {
            instr_name = "rol".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x37, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("[${b0:02X}],Y").to_string();
        }
        (0x38, ..) => {
            instr_name = "sec".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x39, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0x3A, ..) => {
            instr_name = "dec".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x3B, ..) => {
            instr_name = "tsc".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x3C, ..) => {
            instr_name = "bit".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x3D, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x3E, ..) => {
            instr_name = "rol".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x3F, ..) => {
            instr_name = "and".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X},X").to_string();
        }
        (0x40, ..) => {
            instr_name = "rti".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x41, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},X)").to_string();
        }
        (0x42, ..) => {
            instr_name = "wdm".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x43, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},S").to_string();
        }
        (0x44, ..) => {
            instr_name = "mvp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X},#${b0:02X}").to_string();
        }
        (0x45, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x46, ..) => {
            instr_name = "lsr".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x47, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x48, ..) => {
            instr_name = "pha".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x49, _, true, _) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0x49, _, false, _) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0x4A, ..) => {
            instr_name = "lsr".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x4B, ..) => {
            instr_name = "phk".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x4C, ..) => {
            instr_name = "jmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x4D, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x4E, ..) => {
            instr_name = "lsr".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x4F, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0x50, ..) => {
            instr_name = "bvc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0x51, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X}),Y").to_string();
        }
        (0x52, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x53, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},S),Y").to_string();
        }
        (0x54, ..) => {
            instr_name = "mvn".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X},#${b0:02X}").to_string();
        }
        (0x55, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x56, ..) => {
            instr_name = "lsr".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x57, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("[${b0:02X}],Y").to_string();
        }
        (0x58, ..) => {
            instr_name = "cli".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x59, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0x5A, ..) => {
            instr_name = "phy".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x5B, ..) => {
            instr_name = "tcd".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x5C, ..) => {
            instr_name = "jmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0x5D, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x5E, ..) => {
            instr_name = "lsr".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x5F, ..) => {
            instr_name = "eor".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X},X").to_string();
        }
        (0x60, ..) => {
            instr_name = "rts".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x61, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},X)").to_string();
        }
        (0x62, ..) => {
            instr_name = "per".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x63, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},S").to_string();
        }
        (0x64, ..) => {
            instr_name = "stz".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x65, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x66, ..) => {
            instr_name = "ror".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x67, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x68, ..) => {
            instr_name = "pla".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x69, _, true, _) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0x69, _, false, _) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0x6A, ..) => {
            instr_name = "ror".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x6B, ..) => {
            instr_name = "rtl".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x6C, ..) => {
            instr_name = "jmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("(${b1:02X}{b0:02X})").to_string();
        }
        (0x6D, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x6E, ..) => {
            instr_name = "ror".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x6F, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0x70, ..) => {
            instr_name = "bvs".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0x71, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X}),Y").to_string();
        }
        (0x72, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x73, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},S),Y").to_string();
        }
        (0x74, ..) => {
            instr_name = "stz".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x75, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x76, ..) => {
            instr_name = "ror".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x77, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("[${b0:02X}],Y").to_string();
        }
        (0x78, ..) => {
            instr_name = "sei".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x79, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0x7A, ..) => {
            instr_name = "ply".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x7B, ..) => {
            instr_name = "tdc".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x7C, ..) => {
            instr_name = "jmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("(${b1:02X}{b0:02X},X)").to_string();
        }
        (0x7D, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x7E, ..) => {
            instr_name = "ror".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x7F, ..) => {
            instr_name = "adc".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X},X").to_string();
        }
        (0x80, ..) => {
            instr_name = "bra".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0x81, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},X)").to_string();
        }
        (0x82, ..) => {
            instr_name = "brl".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = "[REL16]".to_string();
        }
        (0x83, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},S").to_string();
        }
        (0x84, ..) => {
            instr_name = "sty".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x85, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x86, ..) => {
            instr_name = "stx".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0x87, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x88, ..) => {
            instr_name = "dey".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x89, _, true, _) => {
            instr_name = "bit".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0x89, _, false, _) => {
            instr_name = "bit".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0x8A, ..) => {
            instr_name = "txa".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x8B, ..) => {
            instr_name = "phb".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x8C, ..) => {
            instr_name = "sty".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x8D, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x8E, ..) => {
            instr_name = "stx".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x8F, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0x90, ..) => {
            instr_name = "bcc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0x91, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X}),Y").to_string();
        }
        (0x92, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0x93, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},S),Y").to_string();
        }
        (0x94, ..) => {
            instr_name = "sty".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x95, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0x96, ..) => {
            instr_name = "stx".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},Y").to_string();
        }
        (0x97, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("[${b0:02X}],Y").to_string();
        }
        (0x98, ..) => {
            instr_name = "tya".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x99, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0x9A, ..) => {
            instr_name = "txs".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x9B, ..) => {
            instr_name = "txy".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0x9C, ..) => {
            instr_name = "stz".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0x9D, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x9E, ..) => {
            instr_name = "stz".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0x9F, ..) => {
            instr_name = "sta".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X},X").to_string();
        }
        (0xA0, _, _, true) => {
            instr_name = "ldy".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xA0, _, _, false) => {
            instr_name = "ldy".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0xA1, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},X)").to_string();
        }
        (0xA2, _, _, true) => {
            instr_name = "ldx".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xA2, _, _, false) => {
            instr_name = "ldx".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0xA3, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},S").to_string();
        }
        (0xA4, ..) => {
            instr_name = "ldy".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xA5, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xA6, ..) => {
            instr_name = "ldx".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xA7, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0xA8, ..) => {
            instr_name = "tay".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xA9, _, true, _) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xA9, _, false, _) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0xAA, ..) => {
            instr_name = "tax".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xAB, ..) => {
            instr_name = "plb".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xAC, ..) => {
            instr_name = "ldy".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xAD, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xAE, ..) => {
            instr_name = "ldx".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xAF, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0xB0, ..) => {
            instr_name = "bcs".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0xB1, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X}),Y").to_string();
        }
        (0xB2, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0xB3, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},S),Y").to_string();
        }
        (0xB4, ..) => {
            instr_name = "ldy".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0xB5, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0xB6, ..) => {
            instr_name = "ldx".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},Y").to_string();
        }
        (0xB7, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("[${b0:02X}],Y").to_string();
        }
        (0xB8, ..) => {
            instr_name = "clv".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xB9, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0xBA, ..) => {
            instr_name = "tsx".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xBB, ..) => {
            instr_name = "tyx".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xBC, ..) => {
            instr_name = "ldy".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0xBD, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0xBE, ..) => {
            instr_name = "ldx".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0xBF, ..) => {
            instr_name = "lda".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X},X").to_string();
        }
        (0xC0, _, _, true) => {
            instr_name = "cpy".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xC0, _, _, false) => {
            instr_name = "cpy".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0xC1, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},X)").to_string();
        }
        (0xC2, ..) => {
            instr_name = "rep".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xC3, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},S").to_string();
        }
        (0xC4, ..) => {
            instr_name = "cpy".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xC5, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xC6, ..) => {
            instr_name = "dec".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xC7, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0xC8, ..) => {
            instr_name = "iny".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xC9, _, true, _) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xC9, _, false, _) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0xCA, ..) => {
            instr_name = "dex".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xCB, ..) => {
            instr_name = "wai".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xCC, ..) => {
            instr_name = "cpy".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xCD, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xCE, ..) => {
            instr_name = "dec".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xCF, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0xD0, ..) => {
            instr_name = "bne".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0xD1, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X}),Y").to_string();
        }
        (0xD2, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0xD3, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},S),Y").to_string();
        }
        (0xD4, ..) => {
            instr_name = "pei".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xD5, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0xD6, ..) => {
            instr_name = "dec".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0xD7, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("[${b0:02X}],Y").to_string();
        }
        (0xD8, ..) => {
            instr_name = "cld".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xD9, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0xDA, ..) => {
            instr_name = "phx".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xDB, ..) => {
            instr_name = "stp".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xDC, ..) => {
            instr_name = "jmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("[${b1:02X}{b0:02X}]").to_string();
        }
        (0xDD, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0xDE, ..) => {
            instr_name = "dec".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0xDF, ..) => {
            instr_name = "cmp".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X},X").to_string();
        }
        (0xE0, _, _, true) => {
            instr_name = "cpx".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xE0, _, _, false) => {
            instr_name = "cpx".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0xE1, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},X)").to_string();
        }
        (0xE2, ..) => {
            instr_name = "sep".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xE3, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},S").to_string();
        }
        (0xE4, ..) => {
            instr_name = "cpx".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xE5, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xE6, ..) => {
            instr_name = "inc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X}").to_string();
        }
        (0xE7, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0xE8, ..) => {
            instr_name = "inx".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xE9, _, true, _) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("#${b0:02X}").to_string();
        }
        (0xE9, _, false, _) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0xEA, ..) => {
            instr_name = "nop".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xEB, ..) => {
            instr_name = "xba".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xEC, ..) => {
            instr_name = "cpx".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xED, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xEE, ..) => {
            instr_name = "inc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X}").to_string();
        }
        (0xEF, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X}").to_string();
        }
        (0xF0, ..) => {
            instr_name = "beq".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = "[REL8]".to_string();
        }
        (0xF1, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X}),Y").to_string();
        }
        (0xF2, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X})").to_string();
        }
        (0xF3, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("(${b0:02X},S),Y").to_string();
        }
        (0xF4, ..) => {
            instr_name = "pea".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("#${b1:02X}{b0:02X}").to_string();
        }
        (0xF5, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0xF6, ..) => {
            instr_name = "inc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("${b0:02X},X").to_string();
        }
        (0xF7, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X}").to_string();
            addr_mode_fmt = format!("[${b0:02X}],Y").to_string();
        }
        (0xF8, ..) => {
            instr_name = "sed".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xF9, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},Y").to_string();
        }
        (0xFA, ..) => {
            instr_name = "plx".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xFB, ..) => {
            instr_name = "xce".to_string();
            instr_data = "".to_string();
            addr_mode_fmt = "".to_string();
        }
        (0xFC, ..) => {
            instr_name = "jsr".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("(${b1:02X}{b0:02X},X)").to_string();
        }
        (0xFD, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0xFE, ..) => {
            instr_name = "inc".to_string();
            instr_data = format!("{b0:02X} {b1:02X}").to_string();
            addr_mode_fmt = format!("${b1:02X}{b0:02X},X").to_string();
        }
        (0xFF, ..) => {
            instr_name = "sbc".to_string();
            instr_data = format!("{b0:02X} {b1:02X} {b2:02X}").to_string();
            addr_mode_fmt = format!("${b2:02X}{b1:02X}{b0:02X},X").to_string();
        }
    }

    let addr_mode_fmt = if addr_mode_fmt.contains("[REL8]") {
        let rel8_addr = opcode_addr + 2 + (b0 as i8) as u16;

        addr_mode_fmt.replace("[REL8]", &format!("${rel8_addr:04X}")).to_string()
    } else if addr_mode_fmt.contains("[REL16]") {
        let offset = ((b1 as u16) << 8) | (b0 as u16);
        let rel16_addr = opcode_addr + 3 + offset;

        addr_mode_fmt.replace("[REL8]", &format!("${rel16_addr:04X}")).to_string()
    } else {
        addr_mode_fmt
    };

    let instr_string = format!("{instr_start} {instr_data:<8} {instr_name} {addr_mode_fmt}");

    instr_string
}