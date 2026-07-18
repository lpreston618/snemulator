use crate::{coprocessor::Coprocessor, scpu::{CpuInterrupt, bus::Address}};

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

/// Which DSP-1 register a given CPU address refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DspRegister {
    /// Command/Data register - RW, drives the chip's command protocol.
    Command,
    /// Status register - read-only, bit 7 is the Data Request (RQM) flag.
    Status,
}

/// Classifies a CPU address as a DSP-1 command or status register access, if it
/// is one, for the given mapping mode. Addresses per the SNESdev wiki:
///   Mode 20 (LoROM): Command $30-$3F/$B0-$BF:$8000-$BFFF, Status $30-$3F/$B0-$BF:$C000-$FFFF
///   Mode 21 (HiROM): Command $00-$0F/$80-$8F:$6000-$6FFF, Status $00-$0F/$80-$8F:$7000-$7FFF
/// This is the "Super Mario Kart"-style DSP-1 mapping used by the large majority of
/// DSP-1/1A/1B/2/3/4 games. A handful of titles (e.g. Pilotwings) use a different board
/// with different addresses - if you hit one of those, this will need a per-game override.
fn dsp_register(mapping_mode: MappingMode, addr: Address) -> Option<DspRegister> {
    match mapping_mode {
        MappingMode::LoROM => match addr.bank {
            0x30..=0x3F | 0xB0..=0xBF => match addr.offset {
                0x8000..=0xBFFF => Some(DspRegister::Command),
                0xC000..=0xFFFF => Some(DspRegister::Status),
                _ => None,
            },
            _ => None,
        },
        MappingMode::HiROM => match addr.bank {
            0x00..=0x0F | 0x80..=0x8F => match addr.offset {
                0x6000..=0x6FFF => Some(DspRegister::Command),
                0x7000..=0x7FFF => Some(DspRegister::Status),
                _ => None,
            },
            _ => None,
        },
        // No known DSP-1 games use ExHiROM; add a mapping here if you find one that does.
        MappingMode::ExHiROM => None,
    }
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

    pub cop_vec_e: u16,
    pub cop_vec_n: u16,
    pub brk_vec: u16,
    pub abort_vec_e: u16,
    pub abort_vec_n: u16,
    pub nmi_vec_e: u16,
    pub nmi_vec_n: u16,
    pub reset_vec: u16,
    pub irq_vec_e: u16,
    pub irq_vec_n: u16,

    pub rom_hash: u32,

    pub header_meta: RomHeaderMeta,
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

    pub fn interrupt_vector(&self, interrupt: CpuInterrupt, e: bool) -> u16 {
        match interrupt {
            CpuInterrupt::COP   => if e { self.cop_vec_e } else { self.cop_vec_n },
            CpuInterrupt::BRK   => self.brk_vec,
            CpuInterrupt::Abort => if e { self.abort_vec_e } else { self.abort_vec_n },
            CpuInterrupt::NMI   => if e { self.nmi_vec_e } else { self.nmi_vec_n },
            CpuInterrupt::Reset => self.reset_vec,
            CpuInterrupt::IRQ   => if e { self.irq_vec_e } else { self.irq_vec_n },
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

            cop_vec_e: 0u16,
            cop_vec_n: 0u16,
            brk_vec: 0u16,
            abort_vec_e: 0u16,
            abort_vec_n: 0u16,
            nmi_vec_e: 0u16,
            nmi_vec_n: 0u16,
            reset_vec: 0u16,
            irq_vec_e: 0u16,
            irq_vec_n: 0u16,

            rom_hash: 0u32,

            header_meta: RomHeaderMeta::default(),
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

        cart.header_meta = get_rom_meta(Some(&cart.rom));

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
            Coprocessor::from_id(header_bytes[0x16] >> 4)
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
        cart.cop_vec_n   = u16::from_le_bytes([header_bytes[0x24], header_bytes[0x25]]);
        cart.brk_vec     = u16::from_le_bytes([header_bytes[0x26], header_bytes[0x27]]);
        cart.abort_vec_n = u16::from_le_bytes([header_bytes[0x28], header_bytes[0x29]]);
        cart.nmi_vec_n   = u16::from_le_bytes([header_bytes[0x2A], header_bytes[0x2B]]);
        cart.irq_vec_n   = u16::from_le_bytes([header_bytes[0x2E], header_bytes[0x2F]]);
        cart.cop_vec_e   = u16::from_le_bytes([header_bytes[0x34], header_bytes[0x35]]);
        cart.abort_vec_e = u16::from_le_bytes([header_bytes[0x38], header_bytes[0x39]]);
        cart.nmi_vec_e   = u16::from_le_bytes([header_bytes[0x3A], header_bytes[0x3B]]);
        cart.reset_vec   = u16::from_le_bytes([header_bytes[0x3C], header_bytes[0x3D]]);
        cart.irq_vec_e   = u16::from_le_bytes([header_bytes[0x3E], header_bytes[0x3F]]);

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
        log::trace!("    COP      ${:04X}  ${:04X}", cart.cop_vec_n, cart.cop_vec_e);
        log::trace!("    BRK      ${:04X}  .....", cart.brk_vec);
        log::trace!("    ABORT    ${:04X}  ${:04X}", cart.abort_vec_n, cart.abort_vec_e);
        log::trace!("    NMI      ${:04X}  ${:04X}", cart.nmi_vec_n, cart.nmi_vec_e);
        log::trace!("    RESET    .....  ${:04X}", cart.reset_vec);
        log::trace!("    IRQ      ${:04X}  ${:04X}", cart.irq_vec_n, cart.irq_vec_e);

        Ok(cart)
    }

    pub fn read(&mut self, addr: Address) -> u8 {
        // Check for / perform coprocessor register reads
        if let Some(Coprocessor::Dsp1(dsp)) = self.coprocessor.as_mut() {
            if let Some(reg) = dsp_register(self.mapping_mode, addr) {
                return match reg {
                    DspRegister::Command => dsp.read(),
                    DspRegister::Status => dsp.status(),
                };
            }
        }

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
        // Check for / perform coprocessor register writes
        if let Some(Coprocessor::Dsp1(dsp)) = self.coprocessor.as_mut() {
            if let Some(reg) = dsp_register(self.mapping_mode, addr) {
                // Status is read-only; writes to it are ignored on real hardware.
                if reg == DspRegister::Command {
                    dsp.write(value);
                }
                return;
            }
        }

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

                        Some((mapped_addr as usize) & (self.rom.len() - 1))
                    } else {
                        None
                    }
                }
                _ => None,
            }.unwrap_or({
                let mapped_addr = match self.mapping_mode {
                    MappingMode::LoROM => ((addr & 0x7F0000) >> 1) | (addr & 0x7FFF),
                    MappingMode::HiROM => addr & 0x3FFFFF,
                    MappingMode::ExHiROM => (((addr & 0x800000) ^ 0x800000) >> 1) | (addr & 0x3FFFFF),
                };

                (mapped_addr as usize) & (self.rom.len() - 1)
            })
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

#[derive(Default, Clone)]
pub struct RomHeaderMeta {
    pub title: String,
    pub saves_game: bool,
    pub rom_size_bytes: usize,
    pub coprocessor_name: String,
    pub mapping_name: String,
}

pub fn get_rom_meta(rom: Option<&[u8]>) -> RomHeaderMeta {
    let mut rom_meta = RomHeaderMeta {
        title: "???".to_owned(),
        saves_game: false,
        rom_size_bytes: rom.map_or(0, |r| r.len()),
        coprocessor_name: "Unknown".to_owned(),
        mapping_name: "Unknown".to_owned()
    };

    if rom.is_none() {
        return rom_meta;
    }

    let rom = rom.unwrap();

    let Some(padded_rom) = pad_rom(rom).ok() else {
        return rom_meta;
    };

    let Some(header_pos) = find_header(&padded_rom).ok() else {
        return rom_meta;
    };
    
    if header_pos + 0x16 >= padded_rom.len() {
        return rom_meta
    }

    let best_mapping = best_mapping_mode(rom);

    let mapping_name = match best_mapping {
        MappingMode::LoROM => "LoRom",
        MappingMode::HiROM => "HiRom",
        MappingMode::ExHiROM => "ExHiRom",
    }.to_owned();



    //     00h     ROM             ;if gamecode="042J" --> ROM+SGB2
    //     01h     ROM+RAM (if any such produced?)
    //     02h     ROM+RAM+Battery ;if gamecode="XBND" --> ROM+RAM+Batt+XBandModem
    //                             ;if gamecode="MENU" --> ROM+RAM+Batt+Nintendo Power
    //     03h     ROM+DSP
    //     04h     ROM+DSP+RAM (no such produced)
    //     05h     ROM+DSP+RAM+Battery
    //     13h     ROM+MarioChip1/ExpansionRAM (and "hacked version of OBC1")
    //     14h     ROM+GSU+RAM                    ;\ROM size up to 1MByte -> GSU1
    //     15h     ROM+GSU+RAM+Battery            ;/ROM size above 1MByte -> GSU2
    //     1Ah     ROM+GSU1+RAM+Battery+Fast Mode? (Stunt Race)
    //     25h     ROM+OBC1+RAM+Battery
    //     32h     ROM+SA1+RAM+Battery (?) "F1 Grand Prix Sample (J)"
    //     34h     ROM+SA1+RAM (?) "Dragon Ball Z - Hyper Dimension"
    //     35h     ROM+SA1+RAM+Battery
    //     43h     ROM+S-DD1
    //     45h     ROM+S-DD1+RAM+Battery
    //     55h     ROM+S-RTC+RAM+Battery
    //     E3h     ROM+Super Gameboy      (SGB)
    //     E5h     ROM+Satellaview BIOS   (BS-X)
    //     F5h.00h ROM+Custom+RAM+Battery     (SPC7110)
    //     F9h.00h ROM+Custom+RAM+Battery+RTC (SPC7110+RTC)
    //     F6h.01h ROM+Custom+Battery         (ST010/ST011)
    //     F5h.02h ROM+Custom+RAM+Battery     (ST018)
    //     F3h.10h ROM+Custom                 (CX4)

    let (saves_game, has_coprocessor) = match padded_rom[header_pos + 0x16] & 0x0F {
        0x00 => (false, false),
        0x01 => (false, false),
        0x02 => (true, false),
        0x03 => (false, true),
        0x04 => (false, true),
        0x05 => (true, true),
        0x06 => (false, true),
        0x15 => (true, true),
        0x1A => (true, true),
        0x25 => (true, true),
        0x32 => (true, true),
        0x35 => (true, true),
        0x45 => (true, true),
        0x55 => (true, true),
        0xF5 => (true, true),
        0xF6 => (true, true),
        0xF9 => (true, true),
        _ => (false, false), // Should not happen?
    };

    let coprocessor_name = if !has_coprocessor {
        "None"
    } else {
        Coprocessor::from_id(padded_rom[header_pos + 0x16] >> 4)
            .map_or("Unimplemented", |c| c.label())
    }.to_owned();

    let title_bytes = &padded_rom[header_pos..header_pos + 0x15];
    let title = String::from_utf8_lossy(title_bytes).to_string();

    rom_meta.title = title;
    rom_meta.saves_game = saves_game;
    rom_meta.coprocessor_name = coprocessor_name;
    rom_meta.mapping_name = mapping_name;

    rom_meta
}

// Compute the checksum of the cartridge using the proper mirroring
fn compute_checksum(cart_rom: &[u8]) -> u16 {
    cart_rom.iter().fold(0u16, |acc, &x| acc + x as u16)
}