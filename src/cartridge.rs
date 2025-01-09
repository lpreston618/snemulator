use std::{io::Read, path::Path, str::FromStr};

use crate::scpu::{self, MappingMode};

struct Header {
    title: [u8; 0x15],
    
    fast_rom: bool,
    map_mode: scpu::MappingMode,

    extra_ram: bool,
    battery: bool,
    coprocessor: bool,
    coprocessor_id: u8,

    rom_size: u8, // ROM size is (1 << rom_size) kb

    ram_size: u8, // RAM size is (1 << ram_size) kb

    is_ntsc: bool,

    interrupt_vectors: [u8; 32]
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut title: [u8; 0x15] = [0; 0x15];
        title.clone_from_slice(&bytes[..0x15]);

        let fast_rom = (bytes[0x15] & 0x10) > 0;
        let map_mode = match bytes[0x15] & 0x0F {
            0 => scpu::MappingMode::LoROM,
            1 => scpu::MappingMode::HiROM,
            5 => scpu::MappingMode::ExHiROM,
            _ => { panic!("unimplemented mapping mode"); }
        };

        let (extra_ram, battery, coprocessor) = match bytes[0x16] & 0x0F {
            0 => (false, false, false), // $00 - ROM only
            1 => (true, false, false),  // $01 - ROM + RAM
            2 => (true, true, false),   // $02 - ROM + RAM + battery
            3 => (false, false, true),  // $x3 - ROM + coprocessor
            4 => (true, false, true),   // $x4 - ROM + coprocessor + RAM
            5 => (true, true, true),    // $x5 - ROM + coprocessor + RAM + battery
            6 => (false, true, true),   // $x6 - ROM + coprocessor + battery

            _ => (false, false, false) // Should not happen
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

        println!("ROM LEN: {}", cart_rom.len());

        // Ignore optional 512 byte header
        if cart_rom.len() % 1024 == 512 {
            cart_rom.drain(0..512);
        }
        let cart_rom = cart_rom;

        let header_start = Cartridge::find_header(&cart_rom)? as usize;
        let header_end = header_start + 0x40 as usize;

        let header = Header::from_bytes(&cart_rom[header_start..header_end]);

        // header.print();

        Ok(Self {
            cart_rom,
            header
        })
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
        let cart_rom = cart_rom;

        let header_start = match mode {
            MappingMode::LoROM => 0x007FC0,
            MappingMode::HiROM => 0x00FFC0,
            MappingMode::ExHiROM => 0x40FFC0,
        };
        let header_end = header_start + 0x40 as usize;

        let header = Header::from_bytes(&cart_rom[header_start..header_end]);

        // header.print();

        Ok(Self {
            cart_rom,
            header
        })
    }

    // Returns the address of the header in cartridge ROM
    fn find_header(cart_rom: &Vec<u8>) -> Result<usize, String> {
        // Positions of the start of the header for different memory mappings
        const LoROM_POS: usize = 0x007FC0;
        const HiROM_POS: usize = 0x00FFC0;
        const ExHiROM_POS: usize = 0x40FFC0;
        
        const CHECKSUM_OFFSET: usize = 0x1E;
        const COMPLEMENT_OFFSET: usize = 0x1C;

        let checksum = Cartridge::compute_checksum(cart_rom)?;
        let complement = !checksum;

        if cart_rom.len() < LoROM_POS + 2 {
            return Err(String::from("cart too small for LoROM check"));
        }
        let maybe_checksum = u16::from_le_bytes([
            cart_rom[LoROM_POS + CHECKSUM_OFFSET],
            cart_rom[LoROM_POS + CHECKSUM_OFFSET + 1]
        ]);
        let maybe_complement = u16::from_le_bytes([
            cart_rom[LoROM_POS + COMPLEMENT_OFFSET],
            cart_rom[LoROM_POS + COMPLEMENT_OFFSET + 1]
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) {
            return Ok(LoROM_POS);
        }

        if cart_rom.len() < HiROM_POS + 2 {
            return Err(String::from("cart too small for HiROM check"));
        }
        let maybe_checksum = u16::from_le_bytes([
            cart_rom[HiROM_POS + CHECKSUM_OFFSET],
            cart_rom[HiROM_POS + CHECKSUM_OFFSET + 1]
        ]);
        let maybe_complement = u16::from_le_bytes([
            cart_rom[HiROM_POS + COMPLEMENT_OFFSET],
            cart_rom[HiROM_POS + COMPLEMENT_OFFSET + 1]
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) {
            return Ok(HiROM_POS);
        }

        if cart_rom.len() < ExHiROM_POS + 2 {
            return Err(String::from("cart too small for ExHiROM check"));
        }
        let maybe_checksum = u16::from_le_bytes([
            cart_rom[ExHiROM_POS + CHECKSUM_OFFSET],
            cart_rom[ExHiROM_POS + CHECKSUM_OFFSET + 1]
        ]);
        let maybe_complement = u16::from_le_bytes([
            cart_rom[ExHiROM_POS + COMPLEMENT_OFFSET],
            cart_rom[ExHiROM_POS + COMPLEMENT_OFFSET + 1]
        ]);
        if (checksum == maybe_checksum) && (complement == maybe_complement) {
            return Ok(ExHiROM_POS);
        }
  
        Err(String::from("ROM header not found"))
    }

    // Compute the checksum of the cartridge using the proper mirroring
    fn compute_checksum(cart_rom: &Vec<u8>) -> Result<u16, String> {
        let size = cart_rom.len();
        let on_bits = size.count_ones();
        let bits = size.ilog2() as usize + 1;

        println!("Size: 0x{size:02X}, on_bits: {on_bits}, bits: {bits}");

        let checksum: u16;
        
        if size == 0 {
            return Err(String::from("ROM file empty"));
        } else if on_bits == 1 {
            checksum = cart_rom.iter().take(size).map(|&byte| byte as u16).sum();
        } else if on_bits == 2 {
            let larger_size = 1usize << (bits - 1);
            let smaller_size = size - larger_size;

            let mut larger_sum = 0;
            for i in 0..larger_size {
                larger_sum += cart_rom[i] as u16;
            }

            let mut smaller_sum = 0;
            for i in 0..smaller_size {
                smaller_sum += cart_rom[larger_size + i] as u16;
            }

            checksum = larger_sum + smaller_sum * (larger_size / smaller_size) as u16;
        } else {
            let larger_size = 1usize << (bits - 1);
            let smaller_size = size - larger_size;
            let next_pow_2 = smaller_size.next_power_of_two();

            let mut larger_sum = 0;
            for i in 0..larger_size {
                larger_sum += cart_rom[i] as u16;
            }

            let mut smaller_sum = 0;
            for i in 0..smaller_size {
                smaller_sum += cart_rom[larger_size + i] as u16;
            }

            checksum = larger_sum + smaller_sum * (larger_size / next_pow_2) as u16;
        }

        Ok(checksum)
    }
}

// Public Access
impl Cartridge {
    // The mapping mode of the cartridge as determined by the location of the header in the ROM
    pub fn mapping_mode(&self) -> scpu::MappingMode {
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