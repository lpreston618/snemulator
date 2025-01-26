use std::{io::Read, path::Path, str::FromStr};

use crate::system::cpu::{self, MappingMode};

struct Header {
    title: [u8; 0x15],

    fast_rom: bool,
    map_mode: cpu::MappingMode,

    extra_ram: bool,
    battery: bool,
    coprocessor: bool,
    coprocessor_id: u8,

    rom_size: u8, // ROM size is (1 << rom_size) kb

    ram_size: u8, // RAM size is (1 << ram_size) kb

    is_ntsc: bool,

    interrupt_vectors: [u8; 32],
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut title: [u8; 0x15] = [0; 0x15];
        title.clone_from_slice(&bytes[..0x15]);

        let fast_rom = (bytes[0x15] & 0x10) > 0;
        let map_mode = match bytes[0x15] & 0x0F {
            0 => cpu::MappingMode::LoROM,
            1 => cpu::MappingMode::HiROM,
            5 => cpu::MappingMode::ExHiROM,
            _ => {
                panic!("unimplemented mapping mode");
            }
        };

        let (extra_ram, battery, coprocessor) = match bytes[0x16] & 0x0F {
            0 => (false, false, false), // $00 - ROM only
            1 => (true, false, false),  // $01 - ROM + RAM
            2 => (true, true, false),   // $02 - ROM + RAM + battery
            3 => (false, false, true),  // $x3 - ROM + coprocessor
            4 => (true, false, true),   // $x4 - ROM + coprocessor + RAM
            5 => (true, true, true),    // $x5 - ROM + coprocessor + RAM + battery
            6 => (false, true, true),   // $x6 - ROM + coprocessor + battery

            _ => (false, false, false), // Should not happen
        };
        let coprocessor_id = bytes[0x16] >> 4;

        let rom_size = bytes[0x17];

        let ram_size = bytes[0x18];

        let is_ntsc = bytes[0x19] > 0;

        let mut interrupt_vectors: [u8; 0x20] = [0; 0x20];
        interrupt_vectors.clone_from_slice(&bytes[0x20..0x40]);

        Self {
            title,
            fast_rom,
            map_mode,
            extra_ram,
            battery,
            coprocessor,
            coprocessor_id,
            rom_size,
            ram_size,
            is_ntsc,
            interrupt_vectors,
        }
    }

    pub fn print(&self) {
        println!("title: {:?}", std::str::from_utf8(&self.title).unwrap());
        println!("fast_rom: {:?}", self.fast_rom);
        println!("map_mode: {:?}", self.map_mode);
        println!("extra_ram: {:?}", self.extra_ram);
        println!("battery: {:?}", self.battery);
        println!("coprocessor: {:?}", self.coprocessor);
        println!("coprocessor_id: {:?}", self.coprocessor_id);
        println!("rom_size: {:?}", self.rom_size);
        println!("ram_size: {:?}", self.ram_size);
        println!("is_ntsc: {:?}", self.is_ntsc);
        println!("interrupt_vectors: {:?}", self.interrupt_vectors);
    }
}

pub struct Cartridge {
    cart_rom: Vec<u8>,
    header: Header,
}

// Reading Cartridge
impl Cartridge {
    // Read in a cartridge from the given path to an spc or sfc file
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let rom_file = std::fs::File::open(path).unwrap();

        let mut cart_rom: Vec<u8> = rom_file.bytes().map(|b| b.unwrap()).collect();

        // Ignore optional 512 byte header
        if cart_rom.len() % 1024 == 512 {
            cart_rom.drain(0..512);
        }

        // println!("ROM LEN BEFORE PAD: 0x{:x}", cart_rom.len());

        let cart_rom = pad_rom(cart_rom)?;

        // println!("PADDED ROM LEN: 0x{:x}", cart_rom.len());

        // println!("CHECKSUM: 0x{:x}", Self::compute_checksum(&cart_rom));

        let header_start = Cartridge::find_header(&cart_rom)?;
        let header_end = header_start + 0x40 as usize;

        let header = Header::from_bytes(&cart_rom[header_start..header_end]);

        // header.print();

        Ok(Self { cart_rom, header })
    }

    // Used for testing purposes. Forcibly loads a cart using the given mapping
    // mode, ignoring the checksum.
    pub fn from_path_with_mode(path: &Path, mode: MappingMode) -> Result<Cartridge, String> {
        let rom_file = std::fs::File::open(path).unwrap();

        let mut cart_rom: Vec<u8> = rom_file.bytes().map(|b| b.unwrap()).collect();

        // Ignore optional 512 byte header
        if cart_rom.len() % 1024 == 512 {
            cart_rom.drain(0..512);
        }
        let cart_rom = pad_rom(cart_rom)?;

        let header_start = match mode {
            MappingMode::LoROM => 0x007FC0,
            MappingMode::HiROM => 0x00FFC0,
            MappingMode::ExHiROM => 0x40FFC0,
        };
        let header_end = header_start + 0x40 as usize;

        let header = Header::from_bytes(&cart_rom[header_start..header_end]);

        // header.print();

        Ok(Self { cart_rom, header })
    }

    // Returns the address of the header in cartridge ROM
    fn find_header(cart_rom: &Vec<u8>) -> Result<usize, String> {
        // Positions of the start of the header for different memory mappings
        const LoROM_POS: usize = 0x007FC0;
        const HiROM_POS: usize = 0x00FFC0;
        const ExHiROM_POS: usize = 0x40FFC0;

        const CHECKSUM_OFFSET: usize = 0x1E;
        const COMPLEMENT_OFFSET: usize = 0x1C;

        let checksum = Cartridge::compute_checksum(cart_rom);
        let complement = !checksum;

        if cart_rom.len() < LoROM_POS + 2 {
            return Err(String::from("cart too small for LoROM check"));
        }
        let maybe_checksum = u16::from_le_bytes([
            cart_rom[LoROM_POS + CHECKSUM_OFFSET],
            cart_rom[LoROM_POS + CHECKSUM_OFFSET + 1],
        ]);
        let maybe_complement = u16::from_le_bytes([
            cart_rom[LoROM_POS + COMPLEMENT_OFFSET],
            cart_rom[LoROM_POS + COMPLEMENT_OFFSET + 1],
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) {
            return Ok(LoROM_POS);
        }

        if cart_rom.len() < HiROM_POS + 2 {
            return Err(String::from("cart too small for HiROM check"));
        }
        let maybe_checksum = u16::from_le_bytes([
            cart_rom[HiROM_POS + CHECKSUM_OFFSET],
            cart_rom[HiROM_POS + CHECKSUM_OFFSET + 1],
        ]);
        let maybe_complement = u16::from_le_bytes([
            cart_rom[HiROM_POS + COMPLEMENT_OFFSET],
            cart_rom[HiROM_POS + COMPLEMENT_OFFSET + 1],
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) {
            return Ok(HiROM_POS);
        }

        if cart_rom.len() < ExHiROM_POS + 2 {
            return Err(String::from("cart too small for ExHiROM check"));
        }
        let maybe_checksum = u16::from_le_bytes([
            cart_rom[ExHiROM_POS + CHECKSUM_OFFSET],
            cart_rom[ExHiROM_POS + CHECKSUM_OFFSET + 1],
        ]);
        let maybe_complement = u16::from_le_bytes([
            cart_rom[ExHiROM_POS + COMPLEMENT_OFFSET],
            cart_rom[ExHiROM_POS + COMPLEMENT_OFFSET + 1],
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) {
            return Ok(ExHiROM_POS);
        }

        Err(String::from("ROM header not found"))
    }

    // Compute the checksum of the cartridge using the proper mirroring
    fn compute_checksum(cart_rom: &Vec<u8>) -> u16 {
        cart_rom.iter().fold(0u16, |acc, &x| acc + x as u16)
    }
}

// Public Access
impl Cartridge {
    // The mapping mode of the cartridge as determined by the location of the header in the ROM
    pub fn mapping_mode(&self) -> cpu::MappingMode {
        self.header.map_mode
    }

    // The entire cartridge rom
    pub fn rom_data(&self) -> Vec<u8> {
        self.cart_rom.clone()
    }

    // The size of the cartridge ROM (in KiB). Always a power of 2.
    pub fn rom_size(&self) -> usize {
        (1 << self.header.rom_size) * 1024
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
            let larger_size = (1usize << bitwidth);
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
            let larger_size = 1usize << bitwidth;
            let smaller_size = rom.len() & (larger_size - 1);
            let smaller_pow2_size = smaller_size.next_power_of_two(); // WTH rust just has this?
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

/// Checks if a number is a power of 2 using bitwise operations.
fn is_pow_two(num: usize) -> bool {
    num & (num - 1) == 0
}
