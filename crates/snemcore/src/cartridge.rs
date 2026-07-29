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

#[derive(Clone, Copy, Debug)]
pub enum AddressMode {
    LoRom,
    HiRom,
    ExHiRom,
}

#[derive(Clone, Copy)]
pub enum SramWindow {
    Full64k,   // old boards: SRAM fills the whole 0000-FFFF of 70-7D/F0-FF
    Lower32k,  // new/BigLoROM boards: SRAM only in 0000-7FFF, upper 32k is more ROM
    HiRomBank, // HiROM-with-SRAM: 6000-7FFF window in banks 20-3F/A0-BF etc.
}

enum BusTarget {
    Rom(usize),
    Sram(usize),
    Chip(u16), // routed to coprocessor handler
}

pub struct CartridgeLayout {
    pub mode: AddressMode,
    pub rom_mask: usize,     // rom.len().next_power_of_two() - 1
    pub sram_mask: usize,    // sram size - 1 (0 if none)
    pub sram_window: Option<SramWindow>,
    pub coprocessor: Option<Coprocessor>,
}

impl Default for CartridgeLayout {
    fn default() -> Self {
        Self {
            mode: AddressMode::LoRom,
            rom_mask: 0,
            sram_mask: 0,
            sram_window: None,
            coprocessor: None,
        }
    }
}

impl CartridgeLayout {
    fn map_addr(&self, addr: Address) -> Option<BusTarget> {
        if let Some(coprocessor) = &self.coprocessor {
            match coprocessor {
                Coprocessor::Dsp1(_) => {
                    if is_dsp_register(self.mode, addr) {
                        return Some(BusTarget::Chip(addr.offset));
                    }
                },
                _ => panic!("unimplemented coprocessesor"),
            }
        }

        if let Some(sram_addr) = self.map_sram_addr(addr) {
            return Some(BusTarget::Sram(sram_addr));
        }

        match self.mode {
            AddressMode::LoRom => self.map_lorom(addr),
            AddressMode::HiRom => self.map_hirom(addr),
            AddressMode::ExHiRom => self.map_exhirom(addr),
        }
    }

    fn map_lorom(&self, addr: Address) -> Option<BusTarget> {
        let (bank, offset) = (addr.bank, addr.offset);

        // Plain ROM at 8000-FFFF (banks 00-7D/80-FF), mirrored into
        // 0000-7FFF for banks 40-7D/C0-FF (chip only decodes A0-A14+bank).
        let bank_lo7 = (bank & 0x7F) as usize;
        if bank_lo7 >= 0x7E {
            return None;
        }
        let in_rom_window = offset >= 0x8000;
        let in_mirror_window = offset < 0x8000 && (0x40..=0x7D).contains(&bank_lo7);
        if in_rom_window || in_mirror_window {
            let raw = bank_lo7 * 0x8000 + (offset as usize & 0x7FFF);
            return Some(BusTarget::Rom(raw & self.rom_mask));
        }

        None
    }

    fn map_hirom(&self, addr: Address) -> Option<BusTarget> {
        let (bank, offset) = (addr.bank, addr.offset);

        // 40-7D/C0-FF are the "real" 64K-per-bank ROM chip; 00-3F/80-BF only
        // see the upper half (8000-FFFF) as a mirror. Banks 00-3F/80-BF at
        // 6000-7FFF are *not* ROM even though bank & 0x3F would compute a
        // valid index — without an SRAM chip that range is just unwired.
        let is_main = matches!(bank, 0x40..=0x7D | 0xC0..=0xFF);
        let is_mirror = matches!(bank, 0x00..=0x3F | 0x80..=0xBF) && offset >= 0x8000;
        if !(is_main || is_mirror) {
            return None;
        }

        let bank_idx = (bank & 0x3F) as usize;
        let raw = bank_idx * 0x10000 + offset as usize;
        Some(BusTarget::Rom(raw & self.rom_mask))
    }

    fn map_exhirom(&self, addr: Address) -> Option<BusTarget> {        
        let (bank, offset) = (addr.bank, addr.offset);

        let is_main = matches!(bank, 0x40..=0x7D | 0xC0..=0xFF);
        let is_mirror = matches!(bank, 0x00..=0x3F | 0x80..=0xBF) && offset >= 0x8000;
        if !(is_main || is_mirror) {
            return None;
        }

        // Bit 7 of the bank picks which 4MB half of the ROM image this
        // access lands in; bank & 0x3F selects the page within that half,
        // same mirroring trick as plain HiROM.
        let half_base = if bank & 0x80 != 0 { 0x000000 } else { 0x400000 };
        let bank_idx = (bank & 0x3F) as usize;
        let raw = half_base + bank_idx * 0x10000 + offset as usize;
        Some(BusTarget::Rom(raw & self.rom_mask))
    }

    fn map_sram_addr(&self, addr: Address) -> Option<usize> {
        if self.sram_window.is_none() {
            return None;
        }

        let (bank, offset) = (addr.bank, addr.offset);

        let raw = match self.sram_window.unwrap() {
            // LoROM, old boards: SRAM fills the entire 64K of each bank
            // in 70-7D/F0-FF.
            SramWindow::Full64k => {
                if !matches!(bank, 0x70..=0x7D | 0xF0..=0xFF) {
                    return None;
                }
                let sram_bank = (bank & 0x0F) as usize;
                sram_bank * 0x10000 + offset as usize
            }

            // LoROM, BigLoROM-capable boards: SRAM only occupies the
            // lower 32K of 70-7D/F0-FF; the upper 32K is extra ROM
            // (handled separately in map_lorom, not here).
            SramWindow::Lower32k => {
                if !matches!(bank, 0x70..=0x7D | 0xF0..=0xFF) || offset > 0x7FFF {
                    return None;
                }
                let sram_bank = (bank & 0x0F) as usize;
                sram_bank * 0x8000 + offset as usize
            }

            // HiROM / ExHiROM: fixed 6000-7FFF window per bank, but which
            // banks carry it differs by mode.
            SramWindow::HiRomBank => {
                if !(0x6000..=0x7FFF).contains(&offset) {
                    return None;
                }
                let sram_bank = match self.mode {
                    AddressMode::HiRom if matches!(bank, 0x20..=0x3F | 0xA0..=0xBF) => {
                        (bank & 0x1F) as usize
                    }
                    AddressMode::ExHiRom if matches!(bank, 0x80..=0xBF) => {
                        (bank & 0x1F) as usize
                    }
                    _ => return None,
                };
                sram_bank * 0x2000 + (offset as usize - 0x6000)
            }
        };

        Some(raw & self.sram_mask)
    }
}

fn is_dsp_register(mapping_mode: AddressMode, addr: Address) -> bool {
    match mapping_mode {
        AddressMode::LoRom => match addr.bank {
            0x30..=0x3F | 0xB0..=0xBF => match addr.offset {
                0x8000..=0xFFFF => true,
                _ => false,
            },
            _ => false,
        },
        AddressMode::HiRom => match addr.bank {
            0x00..=0x0F | 0x80..=0x8F => match addr.offset {
                0x6000..=0x7FFF => true,
                _ => false,
            },
            _ => false,
        },
        // No known DSP-1 games use ExHiROM.
        AddressMode::ExHiRom => false,
    }
}

#[derive(Default)]
pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,

    pub ram_written: bool,

    pub title: [u8; 0x15],

    pub fast_rom: bool,
    pub layout: CartridgeLayout,

    pub extra_ram: bool,
    pub battery: bool,

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
    // pub fn cycle(&mut self, clocks: usize) -> usize {
    //     if let Some(coprocessor) = self.layout.coprocessor.as_mut() {
    //         match coprocessor {
    //             _ => 0,
    //         }
    //     } else {
    //         0
    //     }
    // }

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

    pub fn mapping_mode(&self) -> AddressMode {
        self.layout.mode
    }

    pub fn test_blank(title_str: &str, mapping_mode: AddressMode, reset_vec: u16) -> Self {
        const ROM_SIZE: usize = 0x10000;

        let title_str = title_str[..title_str.len().min(0x15)].to_string();

        let mut title = [0; 0x15];
        title[..title_str.len()].copy_from_slice(title_str.as_bytes());

        Self {
            rom: vec![0; ROM_SIZE],
            ram: Vec::new(),

            ram_written: false,

            title,

            fast_rom: false,
            layout: CartridgeLayout {
                mode: mapping_mode,
                rom_mask: ROM_SIZE - 1,
                sram_mask: 0,
                sram_window: None,
                coprocessor: None,
            },

            extra_ram: false,
            battery: false,

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
            reset_vec,
            irq_vec_e: 0u16,
            irq_vec_n: 0u16,

            rom_hash: 0u32,

            header_meta: RomHeaderMeta::default(),
        }
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
        
        cart.layout.mode = best_mapping_mode(&cart.rom);
        cart.layout.rom_mask = cart.rom.len() - 1;

        let header_start = match cart.layout.mode {
            AddressMode::LoRom => LOROM_POS,
            AddressMode::HiRom => HIROM_POS,
            AddressMode::ExHiRom => EXHIROM_POS,
        };
        let header_end = header_start + 0x40 as usize;
        let header_bytes = &cart.rom[header_start..header_end];

        cart.title.copy_from_slice(&header_bytes[..0x15]);
        cart.fast_rom = (header_bytes[0x15] & 0x10) > 0;

        let declared_mapping_mode = header_bytes[0x15] & 0xF;
        let expected_header_mapping_mode = match cart.layout.mode {
            AddressMode::LoRom => 0,
            AddressMode::HiRom => 1,
            AddressMode::ExHiRom => 5,
        };

        if declared_mapping_mode != expected_header_mapping_mode {
            log::warn!("Loading ROM with mapping mode {:?} ({expected_header_mapping_mode}), header says mapping mode {declared_mapping_mode}", cart.layout.mode);
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

        cart.layout.coprocessor = if has_coprocessor {
            Some(Coprocessor::from_id(header_bytes[0x16] >> 4))
        } else {
            None
        };

        if let Some(c) = &cart.layout.coprocessor {
            if !c.is_implemented() {
                return Err(format!("unimplemented coprocessor {}", c.label()));
            }
        }

        cart.rom_size_shift = header_bytes[0x17];
        cart.ram_size_shift = header_bytes[0x18];
        if cart.extra_ram {
            cart.ram_size = 0x400 * (1 << cart.ram_size_shift);
            cart.ram = vec![0u8; cart.ram_size];
        } else {
            cart.ram_size = 0;
        }

        cart.layout.sram_mask = if cart.ram.len() > 0 { cart.ram.len() - 1 } else { 0 };

        cart.layout.sram_window = if !cart.extra_ram {
            None
        } else {
            match cart.layout.mode {
                AddressMode::LoRom => {
                    if cart.rom.len() > 0x20_0000 {
                        Some(SramWindow::Lower32k)
                    } else {
                        Some(SramWindow::Full64k)
                    }
                }
                // HiROM and ExHiROM never have the Full64k/Lower32k distinction —
                // their SRAM lives in a fixed 6000-7FFF window per bank instead.
                AddressMode::HiRom | AddressMode::ExHiRom => Some(SramWindow::HiRomBank),
            }
        };

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
        log::trace!("  loaded_as: {:?}", cart.layout.mode);
        log::trace!("  extra_ram: {}", cart.extra_ram);
        log::trace!("  battery: {}", cart.battery);
        log::trace!("  coprocessor: {}", cart.layout.coprocessor.as_ref().map_or("None", |c| c.label()));
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
        if let Some(target) = self.layout.map_addr(addr) {
            match target {
                BusTarget::Rom(addr)  => self.rom[addr],
                BusTarget::Sram(addr) => self.ram[addr],
                BusTarget::Chip(addr) => match self.layout.coprocessor.as_mut().unwrap() {
                    Coprocessor::Dsp1(dsp1) => {
                        match self.layout.mode {
                            AddressMode::LoRom =>  match addr {
                                0x8000..=0xBFFF => dsp1.get_dr(),
                                0xC000..=0xFFFF => dsp1.get_sr(),
                                _ => unreachable!(),
                            },
                            AddressMode::HiRom => match addr {
                                0x6000..=0x6FFF => dsp1.get_dr(),
                                0x7000..=0x7FFF => dsp1.get_sr(),
                                _ => unreachable!(),
                            },
                            // No known DSP-1 games use ExHiROM
                            AddressMode::ExHiRom => unreachable!(),
                        }
                    },
                    _ => panic!("unimplemented coprocessesor"),
                }
            }
        } else {
            0
        }
    }

    pub fn write(&mut self, addr: Address, value: u8) {
        if let Some(target) = self.layout.map_addr(addr) {
            match target {
                BusTarget::Rom(addr)  => { self.rom[addr] = value; },
                BusTarget::Sram(addr) => { self.ram[addr] = value; },
                BusTarget::Chip(addr) => match self.layout.coprocessor.as_mut().unwrap() {
                    Coprocessor::Dsp1(dsp1) => {
                        match self.layout.mode {
                            AddressMode::LoRom =>  match addr {
                                0x8000..=0xBFFF => dsp1.set_dr(value),
                                0xC000..=0xFFFF => {},
                                _ => unreachable!(),
                            },
                            AddressMode::HiRom => match addr {
                                0x6000..=0x6FFF => dsp1.set_dr(value),
                                0x7000..=0x7FFF => {},
                                _ => unreachable!(),
                            },
                            // No known DSP-1 games use ExHiROM
                            AddressMode::ExHiRom => unreachable!(),
                        }
                    },
                    _ => panic!("unimplemented coprocessesor"),
                }
            }
        }
    }

    /// Overwrite ROM data. Only writes to ROM go thru.
    pub fn write_rom(&mut self, addr: Address, value: u8) {
        if let Some(BusTarget::Rom(mapped_addr)) = self.layout.map_addr(addr) {
            self.rom[mapped_addr] = value;
        }
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

    // pub fn map_addr(&self, addr: Address) -> usize {
    //     let addr = addr.to_u32();

    //     if let Some(coprocessor) = &self.coprocessor {
    //         match coprocessor {
    //             Coprocessor::SuperFx(_) => {
    //                 // Super FX exposes the whole ROM linearly (no 32KB fold) at banks
    //                 // $40-$5F and their mirror $C0-$DF, for the GSU's own direct access.
    //                 let bank = (addr >> 16) & 0xFF;
    //                 if (0x40..=0x5F).contains(&bank) || (0xC0..=0xDF).contains(&bank) {
    //                     let mapped_addr = addr & 0x1FFFFF; // linear, unfolded

    //                     Some((mapped_addr as usize) & (self.rom.len() - 1))
    //                 } else {
    //                     None
    //                 }
    //             }
    //             _ => None,
    //         }.unwrap_or({
    //             let mapped_addr = match self.mapping_mode {
    //                 AddressMode::LoRom => ((addr & 0x7F0000) >> 1) | (addr & 0x7FFF),
    //                 AddressMode::HiRom => addr & 0x3FFFFF,
    //                 AddressMode::ExHiRom => (((addr & 0x800000) ^ 0x800000) >> 1) | (addr & 0x3FFFFF),
    //             };

    //             (mapped_addr as usize) & (self.rom.len() - 1)
    //         })
    //     } else {
    //         let mapped_addr = match self.mapping_mode {
    //             AddressMode::LoRom => ((addr & 0x7F0000) >> 1) | (addr & 0x7FFF),
    //             AddressMode::HiRom => addr & 0x3FFFFF,
    //             AddressMode::ExHiRom => (((addr & 0x800000) ^ 0x800000) >> 1) | (addr & 0x3FFFFF),
    //         };

    //         (mapped_addr as usize) & (self.rom.len() - 1)
    //     }
    // }

    // // Take an address and map it into an SRAM / extra cart ram vector address.
    // // Assumes that the given address is actually a valid SRAM address - up to cart.read()/write() to validate this.
    // fn map_sram_addr(&self, addr: Address) -> usize {
    //     let mapped_addr = match self.mapping_mode {
    //         AddressMode::LoRom => addr.offset as usize + 0x8000 * (addr.bank as usize - 0x7F),
    //         AddressMode::HiRom => {
    //             0x2000 * (addr.bank as usize - 0x30) + addr.offset as usize - 0x6000
    //         }
    //         AddressMode::ExHiRom => {
    //             0x2000 * (addr.bank as usize - 0x80) + addr.offset as usize - 0x6000
    //         }
    //     };
    //     mapped_addr & (self.ram_size - 1)
    // }
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
fn score_header(cart_rom: &[u8], map: AddressMode, checksum: u16, complement: u16) -> i32 {
    let mut score = 0;

    let addr = match map {
        AddressMode::LoRom => LOROM_POS,
        AddressMode::HiRom => HIROM_POS,
        AddressMode::ExHiRom => EXHIROM_POS,
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

    if maybe_map == 0 && matches!(map, AddressMode::LoRom)
        || maybe_map == 1 && matches!(map, AddressMode::HiRom)
        || maybe_map == 5 && matches!(map, AddressMode::ExHiRom)
    {
        score += 8;
    }

    score
}

// Returns the most likely mapping mode for the rom based on various heuristics
fn best_mapping_mode(cart_rom: &[u8]) -> AddressMode {
    let checksum = compute_checksum(cart_rom);
    let complement = !checksum;

    let lorom_score = score_header(cart_rom, AddressMode::LoRom, checksum, complement);
    let hirom_score = score_header(cart_rom, AddressMode::HiRom, checksum, complement);
    let exhirom_score = score_header(cart_rom, AddressMode::ExHiRom, checksum, complement);

    if lorom_score >= hirom_score && lorom_score >= exhirom_score {
        return AddressMode::LoRom;
    }

    if hirom_score > lorom_score && hirom_score >= exhirom_score {
        return AddressMode::HiRom;
    }

    if exhirom_score > lorom_score && exhirom_score > hirom_score {
        return AddressMode::ExHiRom;
    }

    AddressMode::LoRom
}

/// Returns the address of the header in cartridge ROM
fn find_header(cart_rom: &[u8]) -> Result<usize, String> {
    let checksum = compute_checksum(cart_rom);
    let complement = !checksum;

    let lorom_score = score_header(cart_rom, AddressMode::LoRom, checksum, complement);
    let hirom_score = score_header(cart_rom, AddressMode::HiRom, checksum, complement);
    let exhirom_score = score_header(cart_rom, AddressMode::ExHiRom, checksum, complement);

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
        AddressMode::LoRom => "LoRom",
        AddressMode::HiRom => "HiRom",
        AddressMode::ExHiRom => "ExHiRom",
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
        Coprocessor::from_id(padded_rom[header_pos + 0x16] >> 4).label()
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