use crate::{coprocessor::{Coprocessor, superfx}, scpu::bus::Address};

// Positions of the start of the header for different memory mappings
const LOROM_POS: usize = 0x007FC0;
const HIROM_POS: usize = 0x00FFC0;
const EXHIROM_POS: usize = 0x40FFC0;
// Positions of key data in the ROM header
const CHECKSUM_OFFSET: usize = 0x1E;
const COMPLEMENT_OFFSET: usize = 0x1C;
const RESET_VEC_OFFSET: usize = 0x3C;
const MAPPING_MODE_OFFSET: usize = 0x15;
const LAST_TITLE_CHAR_OFFSET: usize = 0x14;

#[derive(Debug, Clone, Copy, Default)]
pub enum MappingMode {
    #[default]
    LoROM,
    HiROM,
    ExHiROM,
}

#[derive(Default)]
pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,

    pub ram_written: bool,

    pub title: [u8; 0x15],

    pub fast_rom: bool,
    pub mapping_mode: MappingMode,

    pub extra_ram: bool,
    pub battery: bool,
    pub coprocessor: Option<Coprocessor>,

    pub rom_size_shift: u8, // ROM size is (1 << rom_size) KiB
    pub ram_size_shift: u8, // RAM size is (1 << ram_size) KiB

    pub ram_size: usize,

    pub is_ntsc: bool,

    pub interrupt_vectors: [u8; 32],

    pub rom_hash: u32,
}

impl Cartridge {
    pub fn cycle(&mut self, clocks: usize) -> usize {
        if let Some(coprocessor) = self.coprocessor.as_mut() {
            match coprocessor {
                Coprocessor::SuperFx(sfx) => sfx.cycle(clocks, &self.rom, &mut self.ram),
                _ => 0,
            }
        } else {
            0
        }
    }

    pub fn mapping_mode(&self) -> MappingMode {
        self.mapping_mode
    }

    pub fn test_blank(title_str: &str, mapping_mode: MappingMode, reset_vec: u16) -> Self {
        const ROM_SIZE: usize = 0x10000;

        let title_str = title_str[..title_str.len().min(0x15)].to_string();

        let mut title = [0; 0x15];
        title[..title_str.len()].copy_from_slice(title_str.as_bytes());

        let mut cart = Self {
            rom: vec![0; ROM_SIZE],
            ram: Vec::new(),

            ram_written: false,

            title,

            fast_rom: false,
            mapping_mode,

            extra_ram: false,
            battery: false,
            coprocessor: None,

            rom_size_shift: (ROM_SIZE / 1024).trailing_zeros() as u8,
            ram_size_shift: 0,

            ram_size: 0,

            is_ntsc: true,

            interrupt_vectors: [0; 32],

            rom_hash: 0u32,
        };

        cart.force_write(Address::from_u32(0x00FFFC), reset_vec as u8);
        cart.force_write(Address::from_u32(0x00FFFD), (reset_vec >> 8) as u8);

        cart
    }

    /// Try to load a cartridges save RAM. Returns Err if the provided vec is of a different
    /// length than how much RAM the cartridge expects.
    pub fn try_load_sram(&mut self, sram: Vec<u8>) -> anyhow::Result<()> {
        if sram.len() != self.ram_size {
            return Err(anyhow::anyhow!(
                "cartridge expects {} bytes of s-ram, got {}",
                self.ram_size,
                sram.len()
            ));
        }

        self.ram = sram;

        Ok(())
    }

    /// Read in a cartridge from the given spc or sfc rom
    pub fn from_rom(mut cart_rom: Vec<u8>, rom_hash: u32) -> Result<Cartridge, String> {
        // Ignore optional 512 byte header
        if cart_rom.len() % 1024 == 512 {
            cart_rom.drain(0..512);
        }

        let cart_rom = pad_rom(&cart_rom)?;

        Self::from_padded_rom(cart_rom, rom_hash)
    }

    fn from_padded_rom(cart_rom: Vec<u8>, rom_hash: u32) -> Result<Self, String> {
        let mut cart = Cartridge {
            rom: cart_rom,
            ..Default::default()
        };

        cart.rom_hash = rom_hash;

        cart.mapping_mode = best_mapping_mode(&cart.rom);

        let header_start = match cart.mapping_mode {
            MappingMode::LoROM => LOROM_POS,
            MappingMode::HiROM => HIROM_POS,
            MappingMode::ExHiROM => EXHIROM_POS,
        };
        let header_end = header_start + 0x40 as usize;
        let header_bytes = &cart.rom[header_start..header_end];

        cart.title.copy_from_slice(&header_bytes[..0x15]);
        cart.fast_rom = (header_bytes[0x15] & 0x10) > 0;

        let declared_mapping_mode = header_bytes[0x15] & 0xF;
        let expected_header_mapping_mode = match cart.mapping_mode {
            MappingMode::LoROM => 0,
            MappingMode::HiROM => 1,
            MappingMode::ExHiROM => 5,
        };

        if declared_mapping_mode != expected_header_mapping_mode {
            log::warn!("Loading ROM with mapping mode {:?} ({expected_header_mapping_mode}), header says mapping mode {declared_mapping_mode}", cart.mapping_mode);
        }

        let has_coprocessor: bool;

        (cart.extra_ram, cart.battery, has_coprocessor) = match header_bytes[0x16] & 0x0F {
            0 => (false, false, false), // $00 - ROM only
            1 => (true, false, false),  // $01 - ROM + RAM
            2 => (true, true, false),   // $02 - ROM + RAM + battery
            3 => (false, false, true),  // $x3 - ROM + coprocessor
            4 => (true, false, true),   // $x4 - ROM + coprocessor + RAM
            5 => (true, true, true),    // $x5 - ROM + coprocessor + RAM + battery
            6 => (false, true, true),   // $x6 - ROM + coprocessor + battery
            _ => (false, false, false), // Should not happen?
        };

        cart.coprocessor = if has_coprocessor {
            match header_bytes[0x16] >> 4 {
                // 0 => CoprocessorKind::Dsp,
                1 => Some(Coprocessor::SuperFx(superfx::SuperFx::new())),
                // 2 => CoprocessorKind::ObC1,
                // 3 => CoprocessorKind::Sa1,
                // 4 => CoprocessorKind::SDd1,
                // 5 => CoprocessorKind::Rtc,
                // x => CoprocessorKind::Other(x),
                x => return Err(format!("Unimplemented coprocessor {x}")),
            }
        } else {
            None
        };
        cart.rom_size_shift = header_bytes[0x17];
        cart.ram_size_shift = header_bytes[0x18];
        if cart.extra_ram {
            cart.ram_size = 0x400 * (1 << cart.ram_size_shift);
            cart.ram = vec![0u8; cart.ram_size];
        } else {
            cart.ram_size = 0;
        }

        cart.is_ntsc = header_bytes[0x19] > 0;
        cart.interrupt_vectors
            .copy_from_slice(&header_bytes[0x20..0x40]);

        log::trace!("ROM Hash (CRC32) = 0x{:04X}", cart.rom_hash);
        log::trace!(
            "Title: '{}'",
            std::str::from_utf8(&cart.title).unwrap_or("<FAILED TO READ TITLE>")
        );
        log::trace!("  fast_rom: {}", cart.fast_rom);
        log::trace!("  mapping_mode: {}", declared_mapping_mode);
        log::trace!("  loaded_as: {:?}", cart.mapping_mode);
        log::trace!("  extra_ram: {}", cart.extra_ram);
        log::trace!("  battery: {}", cart.battery);
        log::trace!("  coprocessor: {}", cart.coprocessor.as_ref().map_or("None", |c| c.label()));
        log::trace!(
            "  rom_size: {} (= {} KiB)",
            cart.rom_size_shift,
            1 << cart.rom_size_shift
        );
        log::trace!(
            "  ram_size: {} (= {} KiB)",
            cart.ram_size_shift,
            1 << cart.ram_size_shift
        );
        log::trace!("  is_ntsc: {}", cart.is_ntsc);
        log::trace!("  padded rom size: 0x{:X}", cart.rom.len());
        log::trace!("  vectors:    NAT    EMU ");
        log::trace!(
            "    COP      ${:02X}{:02X}  ${:02X}{:02X}",
            cart.interrupt_vectors[0x05],
            cart.interrupt_vectors[0x04],
            cart.interrupt_vectors[0x15],
            cart.interrupt_vectors[0x14]
        );
        log::trace!(
            "    BRK      ${:02X}{:02X}  .....",
            cart.interrupt_vectors[0x07],
            cart.interrupt_vectors[0x06]
        );
        log::trace!(
            "    ABORT    ${:02X}{:02X}  ${:02X}{:02X}",
            cart.interrupt_vectors[0x09],
            cart.interrupt_vectors[0x08],
            cart.interrupt_vectors[0x19],
            cart.interrupt_vectors[0x18]
        );
        log::trace!(
            "    NMI      ${:02X}{:02X}  ${:02X}{:02X}",
            cart.interrupt_vectors[0x0B],
            cart.interrupt_vectors[0x0A],
            cart.interrupt_vectors[0x1B],
            cart.interrupt_vectors[0x1A]
        );
        log::trace!(
            "    RESET    .....  ${:02X}{:02X}",
            cart.interrupt_vectors[0x1D],
            cart.interrupt_vectors[0x1C]
        );
        log::trace!(
            "    IRQ      ${:02X}{:02X}  ${:02X}{:02X}",
            cart.interrupt_vectors[0x0F],
            cart.interrupt_vectors[0x0E],
            cart.interrupt_vectors[0x1F],
            cart.interrupt_vectors[0x1E]
        );

        Ok(cart)
    }

    pub fn read(&self, addr: Address) -> u8 {
        let mapped_addr = self.map_addr(addr);

        // Check for / perform SRAM reads
        match self.mapping_mode {
            MappingMode::LoROM => {
                if addr.bank >= 0x70 && addr.bank <= 0x7D && addr.offset < 0x8000 {
                    if self.ram_size != 0 {
                        let sram_addr = self.map_sram_addr(addr);
                        return self.ram[sram_addr];
                    }
                }
            }
            MappingMode::HiROM => {
                if addr.bank >= 0x30
                    && addr.bank <= 0x3F
                    && addr.offset >= 0x6000
                    && addr.offset <= 0x7FFF
                {
                    if self.ram_size != 0 {
                        let sram_addr = self.map_sram_addr(addr);
                        return self.ram[sram_addr];
                    }
                }
            }
            MappingMode::ExHiROM => {
                if addr.bank >= 0x80
                    && addr.bank <= 0xBF
                    && addr.offset >= 0x6000
                    && addr.offset <= 0x7FFF
                {
                    if self.ram_size != 0 {
                        let sram_addr = self.map_sram_addr(addr);
                        return self.ram[sram_addr];
                    }
                }
            }
        };

        // If not SRAM, read from ROM
        let mapped_addr = (mapped_addr as usize) & (self.rom.len() - 1);
        self.rom[mapped_addr]
    }

    pub fn write(&mut self, addr: Address, value: u8) {
        // Check for / perform SRAM write
        match self.mapping_mode {
            MappingMode::LoROM => {
                if addr.bank >= 0x70 && addr.bank <= 0x7D && addr.offset < 0x8000 {
                    if self.ram_size != 0 {
                        let sram_addr = self.map_sram_addr(addr);
                        self.ram[sram_addr] = value;
                        self.ram_written = true;
                    }
                }
            }
            MappingMode::HiROM => {
                if addr.bank >= 0x30
                    && addr.bank <= 0x3F
                    && addr.offset >= 0x6000
                    && addr.offset <= 0x7FFF
                {
                    if self.ram_size != 0 {
                        let sram_addr = self.map_sram_addr(addr);
                        self.ram[sram_addr] = value;
                        self.ram_written = true;
                    }
                }
            }
            MappingMode::ExHiROM => {
                if addr.bank >= 0x80
                    && addr.bank <= 0xBF
                    && addr.offset >= 0x6000
                    && addr.offset <= 0x7FFF
                {
                    if self.ram_size != 0 {
                        let sram_addr = self.map_sram_addr(addr);
                        self.ram[sram_addr] = value;
                        self.ram_written = true;
                    }
                }
            }
        }
    }

    /// Can overwrite ROM data.
    pub fn force_write(&mut self, addr: Address, value: u8) {
        let mapped_addr = self.map_addr(addr);
        let mapped_addr = (mapped_addr as usize) & (self.rom.len() - 1);
        self.rom[mapped_addr] = value;
    }

    pub fn sram_slice(&self) -> &[u8] {
        &self.ram[..]
    }

    pub fn rom_slice(&self) -> &[u8] {
        &self.rom[..]
    }

    pub fn rom_slice_mut(&mut self) -> &mut [u8] {
        &mut self.rom[..]
    }

    pub fn map_addr(&self, addr: Address) -> usize {
        let addr = addr.to_u32();

        if let Some(coprocessor) = &self.coprocessor {
            match coprocessor {
                Coprocessor::SuperFx(_) => {
                    // Super FX exposes the whole ROM linearly (no 32KB fold) at banks
                    // $40-$5F and their mirror $C0-$DF, for the GSU's own direct access.
                    let bank = (addr >> 16) & 0xFF;
                    if (0x40..=0x5F).contains(&bank) || (0xC0..=0xDF).contains(&bank) {
                        let mapped_addr = addr & 0x1FFFFF; // linear, unfolded

                        (mapped_addr as usize) & (self.rom.len() - 1)
                    } else {
                        let mapped_addr = match self.mapping_mode {
                            MappingMode::LoROM => ((addr & 0x7F0000) >> 1) | (addr & 0x7FFF),
                            MappingMode::HiROM => addr & 0x3FFFFF,
                            MappingMode::ExHiROM => (((addr & 0x800000) ^ 0x800000) >> 1) | (addr & 0x3FFFFF),
                        };

                        (mapped_addr as usize) & (self.rom.len() - 1)
                    }
                }
                _ => unreachable!("Not implemented yet"),
            }
        } else {
            let mapped_addr = match self.mapping_mode {
                MappingMode::LoROM => ((addr & 0x7F0000) >> 1) | (addr & 0x7FFF),
                MappingMode::HiROM => addr & 0x3FFFFF,
                MappingMode::ExHiROM => (((addr & 0x800000) ^ 0x800000) >> 1) | (addr & 0x3FFFFF),
            };

            (mapped_addr as usize) & (self.rom.len() - 1)
        }
    }

    // Take an address and map it into an SRAM / extra cart ram vector address.
    // Assumes that the given address is actually a valid SRAM address - up to cart.read()/write() to validate this.
    fn map_sram_addr(&self, addr: Address) -> usize {
        let mapped_addr = match self.mapping_mode {
            MappingMode::LoROM => addr.offset as usize + 0x8000 * (addr.bank as usize - 0x7F),
            MappingMode::HiROM => {
                0x2000 * (addr.bank as usize - 0x30) + addr.offset as usize - 0x6000
            }
            MappingMode::ExHiROM => {
                0x2000 * (addr.bank as usize - 0x80) + addr.offset as usize - 0x6000
            }
        };
        mapped_addr & (self.ram_size - 1)
    }
}

/// Pad the ROM data to a power of two size, correctly mirroring the smaller
/// portion of ROM according to https://snes.nesdev.org/wiki/ROM_file_formats.
fn pad_rom(rom: &[u8]) -> Result<Vec<u8>, String> {
    match usize::count_ones(rom.len()) {
        0 => return Err(String::from("Empty ROM data")),
        1 => return Ok(rom.to_vec()),
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

/// Evaluate the likelihood of a ROM header being located at the given position
fn score_header(cart_rom: &[u8], map: MappingMode, checksum: u16, complement: u16) -> i32 {
    let mut score = 0;

    let addr = match map {
        MappingMode::LoROM => LOROM_POS,
        MappingMode::HiROM => HIROM_POS,
        MappingMode::ExHiROM => EXHIROM_POS,
    };

    let rom_mirror = cart_rom.len() - 1;
    let read_rom = |addr: usize| cart_rom[addr & rom_mirror];

    let maybe_checksum = u16::from_le_bytes([
        read_rom(addr + CHECKSUM_OFFSET + 0),
        read_rom(addr + CHECKSUM_OFFSET + 1),
    ]);
    let maybe_complement = u16::from_le_bytes([
        read_rom(addr + COMPLEMENT_OFFSET + 0),
        read_rom(addr + COMPLEMENT_OFFSET + 1),
    ]);
    let maybe_reset_vec = u16::from_le_bytes([
        read_rom(addr + RESET_VEC_OFFSET + 0),
        read_rom(addr + RESET_VEC_OFFSET + 1),
    ]);

    if maybe_reset_vec < 0x8000 {
        return 0; // Reset should always be in ROM, so if the vector points outside of ROM, this ain't it.
    }

    if (checksum == maybe_checksum) && (complement == maybe_complement) {
        score += 4;
    }

    if (maybe_checksum + maybe_complement) == 0xFFFF {
        score += 4;
    }

    if read_rom(addr + LAST_TITLE_CHAR_OFFSET) == 0x20 {
        score += 2; // The last character of the title is often a space because the title is space-padded.
    }

    let opcode = read_rom(maybe_reset_vec as usize);

    match opcode {
        0x78 | 0x18 | 0x38 | 0x9C | 0x4C | 0x5C => {
            // Matches instructions likely to begin an interrupt vector:
            // sei, clc, sec, stz, jmp, jml
            score += 8;
        }
        _ => {}
    }

    let maybe_map = read_rom(addr + MAPPING_MODE_OFFSET) & 0x0F;

    if maybe_map == 0 && matches!(map, MappingMode::LoROM)
        || maybe_map == 1 && matches!(map, MappingMode::HiROM)
        || maybe_map == 5 && matches!(map, MappingMode::ExHiROM)
    {
        score += 8;
    }

    score
}

// Returns the most likely mapping mode for the rom based on various heuristics
fn best_mapping_mode(cart_rom: &[u8]) -> MappingMode {
    let checksum = compute_checksum(cart_rom);
    let complement = !checksum;

    let lorom_score = score_header(cart_rom, MappingMode::LoROM, checksum, complement);
    let hirom_score = score_header(cart_rom, MappingMode::HiROM, checksum, complement);
    let exhirom_score = score_header(cart_rom, MappingMode::ExHiROM, checksum, complement);

    if lorom_score >= hirom_score && lorom_score >= exhirom_score {
        return MappingMode::LoROM;
    }

    if hirom_score > lorom_score && hirom_score >= exhirom_score {
        return MappingMode::HiROM;
    }

    if exhirom_score > lorom_score && exhirom_score > hirom_score {
        return MappingMode::ExHiROM;
    }

    MappingMode::LoROM
}

/// Returns the address of the header in cartridge ROM
fn find_header(cart_rom: &[u8]) -> Result<usize, String> {
    let checksum = compute_checksum(cart_rom);
    let complement = !checksum;

    let lorom_score = score_header(cart_rom, MappingMode::LoROM, checksum, complement);
    let hirom_score = score_header(cart_rom, MappingMode::HiROM, checksum, complement);
    let exhirom_score = score_header(cart_rom, MappingMode::ExHiROM, checksum, complement);

    log::trace!(
        "Header search: lo {}, hi {}, exhi {}",
        lorom_score,
        hirom_score,
        exhirom_score
    );

    if lorom_score == 0 && hirom_score == 0 && exhirom_score == 0 {
        return Err(String::from("No valid header found"));
    }

    if lorom_score >= hirom_score {
        if lorom_score >= exhirom_score {
            Ok(LOROM_POS)
        } else {
            Ok(EXHIROM_POS)
        }
    } else if hirom_score >= exhirom_score {
        Ok(HIROM_POS)
    } else {
        Ok(EXHIROM_POS)
    }
}

pub fn get_rom_title(rom: &[u8]) -> Option<String> {
    let padded_rom = pad_rom(rom).ok()?;
    let header_pos = find_header(&padded_rom).ok()?;
    if header_pos + 0x15 >= padded_rom.len() {
        return None;
    }
    let title_bytes = &padded_rom[header_pos..header_pos + 0x15];
    Some(String::from_utf8_lossy(title_bytes).to_string())
}

// Compute the checksum of the cartridge using the proper mirroring
fn compute_checksum(cart_rom: &[u8]) -> u16 {
    cart_rom.iter().fold(0u16, |acc, &x| acc + x as u16)
}
