use std::{io::Read, path::Path};

use crate::header::Header;

pub struct Cartridge {
    cart_rom: Vec<u8>,
    header: Header,
}

impl Cartridge {
    pub fn from_path(path: &Path) -> Result<Self, String> {        
        let rom_file = std::fs::File::open(path).unwrap();

        let mut cart_rom: Vec<u8> = rom_file.bytes().map(|b| b.unwrap()).collect();

        // Ignore optional 512 byte header
        if cart_rom.len() % 1024 == 512 {
            cart_rom.drain(0..512);
        }
        let cart_rom = cart_rom;

        let header_start = Cartridge::find_header(&cart_rom)? as usize;
        let header_end = header_start + 0x40 as usize;

        let header = Header::from_bytes(&cart_rom[header_start..header_end]);

        header.print();

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

    fn compute_checksum(cart_rom: &Vec<u8>) -> Result<u16, String> {
        let size = cart_rom.len();
        let on_bits = size.count_ones();
        let bits = size.ilog2() as usize;

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