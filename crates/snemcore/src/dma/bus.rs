use crate::{cartridge::Cartridge, get_byte_n, scpu::Address, set_byte_n, sppu::{Color, MasterSlave, VideoType, VramIncMode, regs::PpuRegs}, ssmp::ioports::ApuIoPorts, sysinfo::{CGRAM_SIZE, OAM_SIZE, VRAM_SIZE, WRAM_SIZE}};

pub struct DmaBus<'a> {
    pub wram: &'a mut [u8; WRAM_SIZE],
    pub vram: &'a mut [u16; VRAM_SIZE],
    pub cgram: &'a mut [Color; CGRAM_SIZE],
    pub oam: &'a mut [u8; OAM_SIZE],
    pub ppu_regs: &'a mut PpuRegs,
    pub vblank_flag: bool,
    pub hblank_flag: bool,
    pub apu_ports: &'a mut ApuIoPorts,
    pub cart: &'a mut Cartridge,
}

impl<'a> DmaBus<'a> {
    pub fn read(&mut self, addr: Address) -> u8 {
        let value = match addr.bank {
            // Banks $00-$3F: LoROM mapping
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                // WRAM mirror (first 8KB)
                0x0000..=0x1FFF => self.wram[addr.offset as usize],

                // PPU registers
                0x2100..=0x213F => self.read_ppu_regs(addr.offset),

                // Cartridge (LoROM: $8000-$FFFF)
                0x8000..=0xFFFF => self.cart.read(addr),

                _ => 0, // Open bus
            },

            // Banks $40-$6F: LoROM cartridge
            0x40..=0x6F => self.cart.read(addr),

            // Banks $70-$7D: SRAM or ROM
            0x70..=0x7D => self.cart.read(addr),

            // Banks $7E-$7F: WRAM (full 128KB)
            0x7E..=0x7F => {
                let wram_addr = ((addr.bank as usize & 1) << 16) | (addr.offset as usize);
                self.wram[wram_addr]
            }

            // Banks $C0-$FF: HiROM cartridge / mirror
            0xC0..=0xFF => self.cart.read(addr),
        };
        
        value
    }

    pub fn write(&mut self, addr: Address, value: u8) {
        match addr.bank {
            // WRAM mirror
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                0x0000..=0x1FFF => self.wram[addr.offset as usize] = value,

                // PPU registers
                0x2100..=0x213F => self.write_ppu_regs(addr.offset, value),

                // Cartridge (SRAM, mapper registers)
                0x8000..=0xFFFF => self.cart.write(addr, value),

                _ => {}
            },

            // WRAM direct access
            0x7E..=0x7F => {
                let wram_addr = ((addr.bank as usize & 1) << 16) | (addr.offset as usize);
                self.wram[wram_addr] = value;
            }

            // Cartridge
            _ => self.cart.write(addr, value),
        }
    }

    fn read_ppu_regs(&mut self, offset: u16) -> u8 {
        let ppu_regs = &mut self.ppu_regs;

        match offset {
            0x2134 => {
                get_byte_n!(ppu_regs.multiply_result, 0)
            }
            0x2135 => {
                get_byte_n!(ppu_regs.multiply_result, 1)
            }
            0x2136 => {
                get_byte_n!(ppu_regs.multiply_result, 2)
            }

            0x2137 => {
                // When counter_latch transitions from 0 to 1
                // https://snes.nesdev.org/wiki/PPU_registers#OPVCT
                if !ppu_regs.counter_toggle {
                    ppu_regs.h_counter_latch = ppu_regs.h_counter;
                    ppu_regs.v_counter_latch = ppu_regs.v_counter;
                }

                ppu_regs.counter_toggle = true;

                0 // CPU OPEN BUS
            }

            0x2138 => {
                ppu_regs.internal_oam_addr += 1;
                ppu_regs.internal_oam_addr %= OAM_SIZE as u16;

                self.oam[ppu_regs.internal_oam_addr as usize]
            }

            0x2139 => {
                let val = get_byte_n!(ppu_regs.vram_latch, 0);

                match ppu_regs.vram_addr_inc_mode {
                    VramIncMode::LowByte => {
                        ppu_regs.vram_latch = if ppu_regs.in_fblank || self.vblank_flag {
                            self.vram[ppu_regs.get_vram_addr() as usize]
                        } else {
                            0
                        };
                        ppu_regs.inc_vram_addr();
                    }

                    _ => {}
                }

                val
            }

            0x213A => {
                let val = get_byte_n!(ppu_regs.vram_latch, 1);

                match ppu_regs.vram_addr_inc_mode {
                    VramIncMode::HighByte => {
                        ppu_regs.vram_latch = if ppu_regs.in_fblank || self.vblank_flag {
                            self.vram[ppu_regs.get_vram_addr() as usize]
                        } else {
                            0
                        };
                        ppu_regs.inc_vram_addr();
                    }

                    _ => {}
                }

                val
            }

            0x213B => {
                let data = self.cgram[ppu_regs.cgram_addr as usize].to_bgr555();

                ppu_regs.cgram_toggle = !ppu_regs.cgram_toggle;

                if ppu_regs.cgram_toggle {
                    get_byte_n!(data, 0)
                } else {
                    get_byte_n!(data, 1)
                }
            }

            0x213C => {
                ppu_regs.h_counter_toggle = !ppu_regs.h_counter_toggle;

                if ppu_regs.h_counter_toggle {
                    get_byte_n!(ppu_regs.h_counter_latch, 0)
                } else {
                    get_byte_n!(ppu_regs.h_counter_latch, 1) // HIGH 7 BITS ARE PPU2 OPEN BUS
                }
            }

            0x213D => {
                ppu_regs.v_counter_toggle = !ppu_regs.v_counter_toggle;

                if ppu_regs.v_counter_toggle {
                    get_byte_n!(ppu_regs.v_counter_latch, 0)
                } else {
                    get_byte_n!(ppu_regs.v_counter_latch, 1) // HIGH 7 BITS ARE PPU2 OPEN BUS
                }
            }

            0x213E => {
                let spr_overflow_bit = if ppu_regs.sprite_overflow { 0x80 } else { 0 };
                let spr_tile_overflow_bit = if ppu_regs.sprite_tile_overflow {
                    0x40
                } else {
                    0
                };
                let master_slave_bit = match ppu_regs.master_slave_state {
                    MasterSlave::Master => 0x20,
                    MasterSlave::Slave => 0,
                };
                let ppu1_open_bus = 0;
                let ppu1_version_bits = ppu_regs.ppu1_version & 0x0F;

                spr_overflow_bit
                    | spr_tile_overflow_bit
                    | master_slave_bit
                    | ppu1_open_bus
                    | ppu1_version_bits
            }

            0x213F => {
                let interlace_bit = if ppu_regs.interlace_field { 0x80 } else { 0 };
                let counter_toggle_bit = if ppu_regs.counter_toggle { 0x40 } else { 0 };
                let ppu2_open_bus = 0;
                let ntsc_pal_bit = match ppu_regs.video_type {
                    VideoType::Ntsc => 0,
                    VideoType::Pal => 0x10,
                };
                let version_bits = ppu_regs.ppu2_version & 0x0F;

                ppu_regs.counter_toggle = false;
                ppu_regs.h_counter_toggle = false;
                ppu_regs.v_counter_toggle = false;

                interlace_bit | counter_toggle_bit | ppu2_open_bus | ntsc_pal_bit | version_bits
            }

            _ => 0,
        }
    }

    fn write_ppu_regs(&mut self, offset: u16, value: u8) {
        let ppu_regs = &mut self.ppu_regs;

        match offset {
            0x2100 => {
                ppu_regs.write_2100(value);
            }
            0x2101 => {
                ppu_regs.write_2101(value);
            }
            0x2102 => {
                ppu_regs.write_2102(value);
            }
            0x2103 => {
                ppu_regs.write_2103(value);
            }

            0x2104 => {
                let internal_oam_addr = ppu_regs.internal_oam_addr as usize;

                if internal_oam_addr & 1 == 0 {
                    ppu_regs.oam_data_latch = value;
                } else if !ppu_regs.oam_write_high_table {
                    self.oam[internal_oam_addr - 1] = ppu_regs.oam_data_latch;
                    self.oam[internal_oam_addr] = value;
                }

                if ppu_regs.oam_write_high_table {
                    self.oam[internal_oam_addr & 0x1F] = value;
                }

                ppu_regs.internal_oam_addr += 1;
                ppu_regs.internal_oam_addr &= 0x1FF;
            }

            0x2105 => {
                ppu_regs.write_2105(value);
            }
            0x2106 => {
                ppu_regs.write_2106(value);
            }
            0x2107 => {
                ppu_regs.write_2107(value);
            }
            0x2108 => {
                ppu_regs.write_2108(value);
            }
            0x2109 => {
                ppu_regs.write_2109(value);
            }
            0x210A => {
                ppu_regs.write_210A(value);
            }
            0x210B => {
                ppu_regs.write_210B(value);
            }
            0x210C => {
                ppu_regs.write_210C(value);
            }
            0x210D => {
                ppu_regs.write_210D(value);
            }
            0x210E => {
                ppu_regs.write_210E(value);
            }
            0x210F => {
                ppu_regs.write_210F(value);
            }
            0x2110 => {
                ppu_regs.write_2110(value);
            }
            0x2111 => {
                ppu_regs.write_2111(value);
            }
            0x2112 => {
                ppu_regs.write_2112(value);
            }
            0x2113 => {
                ppu_regs.write_2113(value);
            }
            0x2114 => {
                ppu_regs.write_2114(value);
            }
            0x2115 => {
                ppu_regs.write_2115(value);
            }

            0x2116 => {
                set_byte_n!(ppu_regs.vram_addr, value as u16, 0);
                ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];
            }

            0x2117 => {
                set_byte_n!(ppu_regs.vram_addr, value as u16, 1);
                ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];
            }

            0x2118 => {
                if ppu_regs.in_fblank || self.vblank_flag {
                    set_byte_n!(
                        self.vram[ppu_regs.get_vram_addr() as usize],
                        value as u16,
                        0
                    );
                }

                match ppu_regs.vram_addr_inc_mode {
                    VramIncMode::LowByte => ppu_regs.inc_vram_addr(),
                    _ => {}
                }
            }

            0x2119 => {
                if ppu_regs.in_fblank || self.vblank_flag {
                    set_byte_n!(
                        self.vram[ppu_regs.get_vram_addr() as usize],
                        value as u16,
                        1
                    );
                }

                match ppu_regs.vram_addr_inc_mode {
                    VramIncMode::HighByte => ppu_regs.inc_vram_addr(),
                    _ => {}
                }
            }

            0x211A => {
                ppu_regs.write_211A(value);
            }
            0x211B => {
                ppu_regs.write_211B(value);
            }
            0x211C => {
                ppu_regs.write_211C(value);
            }
            0x211D => {
                ppu_regs.write_211D(value);
            }
            0x211E => {
                ppu_regs.write_211E(value);
            }
            0x211F => {
                ppu_regs.write_211F(value);
            }
            0x2120 => {
                ppu_regs.write_2120(value);
            }
            0x2121 => {
                ppu_regs.write_2121(value);
            }

            0x2122 => {
                if !self.vblank_flag && !self.hblank_flag && !ppu_regs.in_fblank {
                    return;
                }
                
                if !ppu_regs.cgram_toggle {
                    ppu_regs.cgram_latch = value;
                } else {
                    let new_col = ((value as u16) << 8) | ppu_regs.cgram_latch as u16;

                    self.cgram[ppu_regs.cgram_addr as usize] = Color::from_bgr555(new_col);

                    ppu_regs.cgram_addr += 1;
                }

                ppu_regs.cgram_toggle = !ppu_regs.cgram_toggle;
            }

            0x2123 => {
                ppu_regs.write_2123(value);
            }
            0x2124 => {
                ppu_regs.write_2124(value);
            }
            0x2125 => {
                ppu_regs.write_2125(value);
            }
            0x2126 => {
                ppu_regs.write_2126(value);
            }
            0x2127 => {
                ppu_regs.write_2127(value);
            }
            0x2128 => {
                ppu_regs.write_2128(value);
            }
            0x2129 => {
                ppu_regs.write_2129(value);
            }
            0x212A => {
                ppu_regs.write_212A(value);
            }
            0x212B => {
                ppu_regs.write_212B(value);
            }
            0x212C => {
                ppu_regs.write_212C(value);
            }
            0x212D => {
                ppu_regs.write_212D(value);
            }
            0x212E => {
                ppu_regs.write_212E(value);
            }
            0x212F => {
                ppu_regs.write_212F(value);
            }
            0x2130 => {
                ppu_regs.write_2130(value);
            }
            0x2131 => {
                ppu_regs.write_2131(value);
            }
            0x2132 => {
                ppu_regs.write_2132(value);
            }
            0x2133 => {
                ppu_regs.write_2133(value);
            }

            _ => {}
        }
    }
}