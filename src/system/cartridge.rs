use std::{io::Read, path::Path};

use crate::system::scpu::{self, MappingMode};

#[derive(Default)]
pub struct Cartridge {
    cart_rom: Option<Vec<u8>>,

    title: [u8; 0x15],

    fast_rom: bool,
    mapping_mode: scpu::MappingMode,

    extra_ram: bool,
    battery: bool,
    coprocessor: bool,
    coprocessor_id: u8,

    rom_size: u8, // ROM size is (1 << rom_size) kb
    ram_size: u8, // RAM size is (1 << ram_size) kb

    is_ntsc: bool,

    interrupt_vectors: [u8; 32],
}


impl Cartridge {
    /// Read in a cartridge from the given path to an spc or sfc file
    pub fn from_path(path: &Path) -> Result<Cartridge, String> {
        let mut rom_file = std::fs::File::open(path).unwrap();

        let mut cart_rom = Vec::new();

        let res = rom_file.read_to_end(&mut cart_rom);

        if let Err(_) = res {
            return Err("failed to read ROM bytes".to_string());
        }

        // Ignore optional 512 byte header
        if cart_rom.len() % 1024 == 512 {
            cart_rom.drain(0..512);
        }

        let cart_rom = pad_rom(cart_rom)?;

        let header_start = Cartridge::find_header(&cart_rom)?;
        let header_end = header_start + 0x40 as usize;

        let mut cart = Cartridge::default();

        cart.populate_header_data(&cart_rom[header_start..header_end]);
        cart.cart_rom = Some(cart_rom);

        Ok(cart)
    }

    fn populate_header_data(&mut self, bytes: &[u8]) {
        self.title.copy_from_slice(&bytes[..0x15]);
        self.fast_rom = (bytes[0x15] & 0x10) > 0;
        self.mapping_mode = match bytes[0x15] & 0x0F {
            0 => scpu::MappingMode::LoROM,
            1 => scpu::MappingMode::HiROM,
            5 => scpu::MappingMode::ExHiROM,
            _ => {
                panic!("unimplemented mapping mode");
            }
        };
        (self.extra_ram, self.battery, self.coprocessor) = match bytes[0x16] & 0x0F {
            0 => (false, false, false), // $00 - ROM only
            1 => (true, false, false),  // $01 - ROM + RAM
            2 => (true, true, false),   // $02 - ROM + RAM + battery
            3 => (false, false, true),  // $x3 - ROM + coprocessor
            4 => (true, false, true),   // $x4 - ROM + coprocessor + RAM
            5 => (true, true, true),    // $x5 - ROM + coprocessor + RAM + battery
            6 => (false, true, true),   // $x6 - ROM + coprocessor + battery
            _ => (false, false, false), // Should not happen
        };
        self.coprocessor_id = bytes[0x16] >> 4;
        self.rom_size = bytes[0x17];
        self.ram_size = bytes[0x18];
        self.is_ntsc = bytes[0x19] > 0;
        self.interrupt_vectors.copy_from_slice(&bytes[0x20..0x40]);
    }

    /// Returns the address of the header in cartridge ROM
    fn find_header(cart_rom: &Vec<u8>) -> Result<usize, String> {
        // Positions of the start of the header for different memory mappings
        const LoROM_POS: usize = 0x007FC0;
        const HiROM_POS: usize = 0x00FFC0;
        const ExHiROM_POS: usize = 0x40FFC0;

        const CHECKSUM_OFFSET: usize = 0x1E;
        const COMPLEMENT_OFFSET: usize = 0x1C;

        let mut rom_mapping_mode: Option<MappingMode> = None;

        let checksum = Cartridge::compute_checksum(cart_rom);
        let complement = !checksum;

        let rom_mirror = cart_rom.len() - 1;

        let read_rom = |addr: usize| { cart_rom[addr & rom_mirror] };

        let maybe_checksum = u16::from_le_bytes([
            read_rom(LoROM_POS + CHECKSUM_OFFSET + 0),
            read_rom(LoROM_POS + CHECKSUM_OFFSET + 1),
        ]);
        let maybe_complement = u16::from_le_bytes([
            read_rom(LoROM_POS + COMPLEMENT_OFFSET + 0),
            read_rom(LoROM_POS + COMPLEMENT_OFFSET + 1),
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) {
            rom_mapping_mode = Some(MappingMode::LoROM);
        }

        let maybe_checksum = u16::from_le_bytes([
            read_rom(HiROM_POS + CHECKSUM_OFFSET + 0),
            read_rom(HiROM_POS + CHECKSUM_OFFSET + 1),
        ]);
        let maybe_complement = u16::from_le_bytes([
            read_rom(HiROM_POS + COMPLEMENT_OFFSET + 0),
            read_rom(HiROM_POS + COMPLEMENT_OFFSET + 1),
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) && rom_mapping_mode.is_none() {
            rom_mapping_mode = Some(MappingMode::HiROM);
        }

        let maybe_checksum = u16::from_le_bytes([
            read_rom(ExHiROM_POS + CHECKSUM_OFFSET + 0),
            read_rom(ExHiROM_POS + CHECKSUM_OFFSET + 1),
        ]);
        let maybe_complement = u16::from_le_bytes([
            read_rom(ExHiROM_POS + COMPLEMENT_OFFSET + 0),
            read_rom(ExHiROM_POS + COMPLEMENT_OFFSET + 1),
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) && rom_mapping_mode.is_none() {
            rom_mapping_mode = Some(MappingMode::ExHiROM);
        }

        if rom_mapping_mode.is_none() {
            return Err(String::from("ROM header not found"));
        }

        let rom_mapping_mode = rom_mapping_mode.unwrap();

        let header_pos = match rom_mapping_mode {
            MappingMode::LoROM => LoROM_POS,
            MappingMode::HiROM => HiROM_POS,
            MappingMode::ExHiROM => ExHiROM_POS,
        };
        let expected_self_ident = match rom_mapping_mode {
            MappingMode::LoROM => 0,
            MappingMode::HiROM => 1,
            MappingMode::ExHiROM => 5,
        };

        let rom_mapping_mode_self_ident = read_rom(header_pos + 0x15) & 0xF;

        if rom_mapping_mode_self_ident != expected_self_ident {
            let map_mode_str = match rom_mapping_mode {
                MappingMode::LoROM => "LoROM",
                MappingMode::HiROM => "HiROM",
                MappingMode::ExHiROM => "ExHiROM",
            };

            let expected_map_mode_str = match rom_mapping_mode_self_ident {
                0 => "LoROM",
                1 => "HiROM",
                5 => "ExHiROM",
                _ => unreachable!(),
            };

            let err_msg = format!("found header in {} pos, but header wants {}", map_mode_str, expected_map_mode_str);

            return Err(err_msg);
        }

        Ok(header_pos)
    }

    // Compute the checksum of the cartridge using the proper mirroring
    fn compute_checksum(cart_rom: &Vec<u8>) -> u16 {
        cart_rom.iter().fold(0u16, |acc, &x| acc + x as u16)
    }
}

// Public Access
impl Cartridge {
    pub fn mapping_mode(&self) -> scpu::MappingMode {
        self.mapping_mode
    }

    pub fn rom_data(self) -> Vec<u8> {
        self.cart_rom.unwrap()
    }
}

/// Pad the ROM data to a power of two size, correctly mirroring the smaller
/// portion of ROM according to https://snes.nesdev.org/wiki/ROM_file_formats.
fn pad_rom(rom: Vec<u8>) -> Result<Vec<u8>, String> {
    match usize::count_ones(rom.len()) {
        0 => return Err(String::from("Empty ROM data")),
        1 => return Ok(rom),
        2 => {
            // Get the width of the binary representation of ROM size.
            // Ex: if rom size is 1024 bytes, bitwidth = 10 (2^10 = 1024).
            let bitwidth = rom.len().ilog2() as usize;
            let larger_size = 1 << bitwidth;
            let smaller_size = rom.len() & (larger_size - 1);
            let repeat_count = larger_size / smaller_size;

            let mut padded_rom = rom[..larger_size].to_vec();
            padded_rom.extend(
                rom[larger_size..]
                    .iter()
                    .cycle()
                    .take(smaller_size * repeat_count),
            );

            return Ok(padded_rom);
        }
        _ => {
            let bitwidth = rom.len().ilog2() as usize;
            let larger_size = 1 << bitwidth;
            let smaller_size = rom.len() & (larger_size - 1);
            let smaller_pow2_size = smaller_size.next_power_of_two();
            let repeat_count = larger_size / smaller_pow2_size;

            let mut padded_rom = rom[..larger_size].to_vec();
            let mut smaller_part: Vec<u8> = rom[larger_size..].to_vec();
            smaller_part.resize(smaller_pow2_size, 0);

            padded_rom.extend(
                smaller_part
                    .iter()
                    .cycle()
                    .take(smaller_pow2_size * repeat_count),
            );

            return Ok(padded_rom);
        }
    }
}