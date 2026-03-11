use log::trace;

use crate::core::scpu::bus::Address;

#[derive(Debug, Clone, Copy, Default)]
enum MappingMode {
    #[default]
    LoROM,
    HiROM,
    ExHiROM,
}

#[derive(Default)]
pub struct Cartridge {
    rom: Vec<u8>,

    title: [u8; 0x15],

    fast_rom: bool,
    mapping_mode: MappingMode,

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
    /// Read in a cartridge from the given spc or sfc rom
    pub fn from_rom(mut cart_rom: Vec<u8>) -> Result<Cartridge, String> {
        // Ignore optional 512 byte header
        if cart_rom.len() % 1024 == 512 {
            cart_rom.drain(0..512);
        }

        let cart_rom = pad_rom(cart_rom)?;
        
        Self::from_padded_rom(cart_rom)
    }
    
    fn from_padded_rom(cart_rom: Vec<u8>) -> Result<Self, String> {
        let mut cart = Cartridge {
            rom: cart_rom,
            ..Default::default()
        };
        
        let header_start = find_header(&cart.rom)?;
        let header_end = header_start + 0x40 as usize;
        let header_bytes = &cart.rom[header_start..header_end];
        
        cart.title.copy_from_slice(&header_bytes[..0x15]);
        cart.fast_rom = (header_bytes[0x15] & 0x10) > 0;
        cart.mapping_mode = match header_bytes[0x15] & 0x0F {
            0 => MappingMode::LoROM,
            1 => MappingMode::HiROM,
            5 => MappingMode::ExHiROM,
            _ => {
                panic!("unimplemented mapping mode");
            }
        };
        (cart.extra_ram, cart.battery, cart.coprocessor) = match header_bytes[0x16] & 0x0F {
            0 => (false, false, false),  // $00 - ROM only
            1 => ( true, false, false),  // $01 - ROM + RAM
            2 => ( true,  true, false),  // $02 - ROM + RAM + battery
            3 => (false, false,  true),  // $x3 - ROM + coprocessor
            4 => ( true, false,  true),  // $x4 - ROM + coprocessor + RAM
            5 => ( true,  true,  true),  // $x5 - ROM + coprocessor + RAM + battery
            6 => (false,  true,  true),  // $x6 - ROM + coprocessor + battery
            _ => (false, false, false),  // Should not happen?
        };
        cart.coprocessor_id = header_bytes[0x16] >> 4;
        cart.rom_size = header_bytes[0x17];
        cart.ram_size = header_bytes[0x18];
        cart.is_ntsc = header_bytes[0x19] > 0;
        cart.interrupt_vectors.copy_from_slice(&header_bytes[0x20..0x40]);
        
        trace!("Title: '{}'", std::str::from_utf8(&cart.title).unwrap_or("<FAILED TO READ TITLE>"));
        trace!("  fast_rom: {}", cart.fast_rom);
        trace!("  mapping_mode: {:?}", cart.mapping_mode);
        trace!("  extra_ram: {}", cart.extra_ram);
        trace!("  battery: {}", cart.battery);
        trace!("  coprocessor: {}", cart.coprocessor);
        trace!("  coprocessor_id: {}", cart.coprocessor_id);
        trace!("  rom_size: {} (= {} KiB)", cart.rom_size, 1 << cart.rom_size);
        trace!("  ram_size: {} (= {} KiB)", cart.ram_size, 1 << cart.ram_size);
        trace!("  is_ntsc: {}", cart.is_ntsc);
        trace!("  padded rom size: 0x{:X}", cart.rom.len());
        trace!("  vectors:    NAT    EMU ");
        trace!("    COP      ${:02X}{:02X}  ${:02X}{:02X}", cart.interrupt_vectors[0x05], cart.interrupt_vectors[0x04], cart.interrupt_vectors[0x15], cart.interrupt_vectors[0x14]);
        trace!("    BRK      ${:02X}{:02X}  .....", cart.interrupt_vectors[0x07], cart.interrupt_vectors[0x06]);
        trace!("    ABORT    ${:02X}{:02X}  ${:02X}{:02X}", cart.interrupt_vectors[0x09], cart.interrupt_vectors[0x08], cart.interrupt_vectors[0x19], cart.interrupt_vectors[0x18]);
        trace!("    NMI      ${:02X}{:02X}  ${:02X}{:02X}", cart.interrupt_vectors[0x0B], cart.interrupt_vectors[0x0A], cart.interrupt_vectors[0x1B], cart.interrupt_vectors[0x1A]);
        trace!("    RESET    .....  ${:02X}{:02X}", cart.interrupt_vectors[0x1D], cart.interrupt_vectors[0x1C]);
        trace!("    IRQ      ${:02X}{:02X}  ${:02X}{:02X}", cart.interrupt_vectors[0x0F], cart.interrupt_vectors[0x0E], cart.interrupt_vectors[0x1F], cart.interrupt_vectors[0x1E]);
        
        Ok(cart)
    }
    
    pub fn read(&mut self, addr: Address) -> u8 {
        let addr = addr.to_u32();
        
        let mapped_addr = match self.mapping_mode {
            MappingMode::LoROM => {
                ((addr & 0x7F0000) >> 1) | (addr & 0x7FFF)
            }
            MappingMode::HiROM => {
                addr & 0x3FFFFF
            }
            MappingMode::ExHiROM => {
                (((addr & 0x800000) ^ 0x800000) >> 1) | (addr & 0x3FFFFF)
            }
        };
        
        let mapped_addr = (mapped_addr as usize) & (self.rom.len() - 1);
        
        self.rom[mapped_addr]
    }
    
    pub fn write(&mut self, addr: Address, value: u8) {
        
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

/// Returns the address of the header in cartridge ROM
fn find_header(cart_rom: &Vec<u8>) -> Result<usize, String> {
    // Positions of the start of the header for different memory mappings
    const LOROM_POS: usize = 0x007FC0;
    const HIROM_POS: usize = 0x00FFC0;
    const EXHIROM_POS: usize = 0x40FFC0;

    const CHECKSUM_OFFSET: usize = 0x1E;
    const COMPLEMENT_OFFSET: usize = 0x1C;

    let mut rom_mapping_mode: Option<MappingMode> = None;

    let checksum = compute_checksum(cart_rom);
    let complement = !checksum;

    let rom_mirror = cart_rom.len() - 1;

    let read_rom = |addr: usize| { cart_rom[addr & rom_mirror] };

    let maybe_checksum = u16::from_le_bytes([
        read_rom(LOROM_POS + CHECKSUM_OFFSET + 0),
        read_rom(LOROM_POS + CHECKSUM_OFFSET + 1),
    ]);
    let maybe_complement = u16::from_le_bytes([
        read_rom(LOROM_POS + COMPLEMENT_OFFSET + 0),
        read_rom(LOROM_POS + COMPLEMENT_OFFSET + 1),
    ]);
    if (checksum == maybe_checksum) && (complement == maybe_complement) {
        rom_mapping_mode = Some(MappingMode::LoROM);
    }

    let maybe_checksum = u16::from_le_bytes([
        read_rom(HIROM_POS + CHECKSUM_OFFSET + 0),
        read_rom(HIROM_POS + CHECKSUM_OFFSET + 1),
    ]);
    let maybe_complement = u16::from_le_bytes([
        read_rom(HIROM_POS + COMPLEMENT_OFFSET + 0),
        read_rom(HIROM_POS + COMPLEMENT_OFFSET + 1),
    ]);
    if (checksum == maybe_checksum) && (complement == maybe_complement) && rom_mapping_mode.is_none() {
        rom_mapping_mode = Some(MappingMode::HiROM);
    }

    let maybe_checksum = u16::from_le_bytes([
        read_rom(EXHIROM_POS + CHECKSUM_OFFSET + 0),
        read_rom(EXHIROM_POS + CHECKSUM_OFFSET + 1),
    ]);
    let maybe_complement = u16::from_le_bytes([
        read_rom(EXHIROM_POS + COMPLEMENT_OFFSET + 0),
        read_rom(EXHIROM_POS + COMPLEMENT_OFFSET + 1),
    ]);
    if (checksum == maybe_checksum) && (complement == maybe_complement) && rom_mapping_mode.is_none() {
        rom_mapping_mode = Some(MappingMode::ExHiROM);
    }

    if rom_mapping_mode.is_none() {
        return Err(String::from("ROM header not found"));
    }

    let rom_mapping_mode = rom_mapping_mode.unwrap();

    let header_pos = match rom_mapping_mode {
        MappingMode::LoROM => LOROM_POS,
        MappingMode::HiROM => HIROM_POS,
        MappingMode::ExHiROM => EXHIROM_POS,
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