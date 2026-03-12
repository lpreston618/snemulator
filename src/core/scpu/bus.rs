use log::debug;

use crate::core::cartridge::Cartridge;
use crate::core::controller::JoypadCmd;
use crate::core::scpu::dma::{AddressIncMode, Direction, DmaRegs, TransferPattern};
use crate::core::scpu::ioregs::{CpuIoRegs, HVTimerIRQ};
use crate::core::scpu::mult::Mult5A22;
use crate::core::sppu::color::Color;
use crate::core::sppu::regs::{
    AddressRemapping, BgMode, CMathOperator, IncrSize, M7FillMode, MasterSlave, ObjectSizeSelect,
    PpuRegs, TileSize, TilemapCount, VideoType, VramIncMode, WindowColorRegion, WindowLogic,
};
use crate::core::ssmp::ioports::ApuIoPorts;
use crate::core::sysinfo::{CGRAM_SIZE, OAM_SIZE, VRAM_SIZE, WRAM_SIZE};
use crate::{get_bit_n, get_byte_n, set_byte_n};

#[derive(Clone, Copy, Debug, Default)]
pub struct Address {
    pub bank: u8,
    pub offset: u16,
}

impl Address {
    pub fn to_u32(self) -> u32 {
        (self.bank as u32) << 16 | self.offset as u32
    }
    pub fn from_u32(val: u32) -> Self {
        Address {
            bank: (val >> 16) as u8,
            offset: val as u16,
        }
    }
}

pub struct CpuBus<'a> {
    pub wram: &'a mut [u8; WRAM_SIZE],
    pub vram: &'a mut [u16; VRAM_SIZE],
    pub cgram: &'a mut [Color; CGRAM_SIZE],
    pub oam: &'a mut [u8; OAM_SIZE],
    
    pub ppu_regs: &'a mut PpuRegs,
    pub cpu_regs: &'a mut CpuIoRegs,
    pub apu_ports: &'a mut ApuIoPorts,
    
    pub cart: &'a mut Cartridge, // or rom or what have you
    pub mult: &'a mut Mult5A22,
    pub dma_regs: &'a mut [DmaRegs; 8],
    pub dma_en: &'a mut bool,
    pub hdma_en: &'a mut bool,
    pub dma_active_ch: &'a mut usize,
    pub hdma_active_ch: &'a mut usize,
    
    pub joy1_in: u16,
    pub joy2_in: u16,
    pub joy1_data1_auto: u16,
    pub joy2_data1_auto: u16,
    pub joy1_data2_auto: u16,
    pub joy2_data2_auto: u16,
    pub joypad_cmd: &'a mut Option<JoypadCmd>
}

impl<'a> CpuBus<'a> {
    pub fn read(&mut self, addr: Address) -> u8 {
        match addr.bank {
            // Banks $00-$3F: LoROM mapping
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                // WRAM mirror (first 8KB)
                0x0000..=0x1FFF => self.wram[addr.offset as usize],

                // PPU registers
                0x2100..=0x213F => self.read_ppu_regs(addr.offset),

                // APU ports
                0x2140..=0x217F => {
                    match addr.offset & 0x3 {
                        0 => self.apu_ports.apuio0,
                        1 => self.apu_ports.apuio1,
                        2 => self.apu_ports.apuio2,
                        3 => self.apu_ports.apuio3,
                        _ => unreachable!(),
                    }
                },

                // S-WRAM access registers
                0x2180..=0x2183 => self.read_wram_port(addr.offset),

                // CPU I/O registers (joypad, DMA, IRQ, etc.)
                0x4000..=0x42FF => self.read_cpuio_regs(addr.offset),
                
                0x4300..=0x43FF => self.read_dma_regs(addr.offset),
                
                0x4400..=0x5FFF => 0, // Open bus

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
        }
    }

    pub fn write(&mut self, addr: Address, value: u8) {
        match addr.bank {
            // WRAM mirror
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                0x0000..=0x1FFF => self.wram[addr.offset as usize] = value,

                // PPU registers
                0x2100..=0x213F => self.write_ppu_regs(addr.offset, value),

                // APU ports
                0x2140..=0x217F => {
                    match addr.offset & 3 {
                        0 => self.apu_ports.cpuio0 = value,
                        1 => self.apu_ports.cpuio1 = value,
                        2 => self.apu_ports.cpuio2 = value,
                        3 => self.apu_ports.cpuio3 = value,
                        _ => unreachable!(),
                    }
                },

                // WRAM access port
                0x2180..=0x2183 => self.write_wram_port(addr.offset, value),

                // CPU I/O registers
                0x4000..=0x42FF => self.write_cpuio_regs(addr.offset, value),
                
                0x4300..=0x43FF => self.write_dma_regs(addr.offset, value),
                
                0x4400..=0x5FFF => {} // Open bus

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
    
    /// Same as [`read`], but for DMA transfers. This version cannot access MMIO regs.
    pub fn dma_read(&mut self, addr: Address) -> u8 {
        match addr.bank {
            // Banks $00-$3F: LoROM mapping
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                // WRAM mirror (first 8KB)
                0x0000..=0x1FFF => self.wram[addr.offset as usize],

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
        }
    }
    
    /// Same as [`write`], but for DMA transfers. This version cannot access MMIO regs.
    pub fn dma_write(&mut self, addr: Address, value: u8) {
        match addr.bank {
            // WRAM mirror
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                0x0000..=0x1FFF => self.wram[addr.offset as usize] = value,

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

    fn read_wram_port(&mut self, offset: u16) -> u8 {
        // match offset {
        //     0x2180 => {
        //         let addr = self.cpu_regs.wram_addr as usize;
        //         let value = self.wram[addr & 0x1FFFF];
        //         self.cpu_regs.wram_addr = self.cpu_regs.wram_addr.wrapping_add(1) & 0x1FFFF;
        //         value
        //     }
        //     _ => 0,
        // }
        0
    }

    fn write_wram_port(&mut self, offset: u16, value: u8) {
        // match offset {
        //     0x2180 => {
        //         let addr = self.cpu_regs.wram_addr as usize;
        //         self.wram[addr & 0x1FFFF] = value;
        //         self.cpu_regs.wram_addr = self.cpu_regs.wram_addr.wrapping_add(1) & 0x1FFFF;
        //     }
        //     0x2181 => {
        //         self.cpu_regs.wram_addr = (self.cpu_regs.wram_addr & 0x1FF00) | (value as u32);
        //     }
        //     0x2182 => {
        //         self.cpu_regs.wram_addr =
        //             (self.cpu_regs.wram_addr & 0x100FF) | ((value as u32) << 8);
        //     }
        //     0x2183 => {
        //         self.cpu_regs.wram_addr =
        //             (self.cpu_regs.wram_addr & 0x0FFFF) | (((value & 1) as u32) << 16);
        //     }
        //     _ => {}
        // }
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
                        ppu_regs.vram_latch = if ppu_regs.in_fblank || ppu_regs.in_vblank {
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
                        ppu_regs.vram_latch = if ppu_regs.in_fblank || ppu_regs.in_vblank {
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
                ppu_regs.in_fblank = get_bit_n!(value, 7);
                ppu_regs.screen_brightness = value & 0x0F;

                // println!("Set fblank to {}, S: {} D: {}", self.in_fblank.get(), self.scanline.get(), self.dot.get());
            }

            0x2101 => {
                let new_obj_size = match value >> 5 {
                    0 => ObjectSizeSelect::Size8x8_16x16,
                    1 => ObjectSizeSelect::Size8x8_32x32,
                    2 => ObjectSizeSelect::Size8x8_64x64,
                    3 => ObjectSizeSelect::Size16x16_32x32,
                    4 => ObjectSizeSelect::Size16x16_64x64,
                    5 => ObjectSizeSelect::Size32x32_64x64,
                    6 => ObjectSizeSelect::Size16x32_32x64,
                    7 => ObjectSizeSelect::Size16x32_32x32,
                    _ => unreachable!(),
                };

                ppu_regs.obj_sprite_size = new_obj_size;
                ppu_regs.name_secondary_select = (value >> 3) & 0x03;
                ppu_regs.name_base_addr = value & 0x03;

                // println!("Set name base addr to ${:04X}", (ppu_regs.name_base_addr.get() as u16) << 13);
            }

            0x2102 => {
                ppu_regs.priority_rotation_idx = value & 0xFE;
                ppu_regs.internal_oam_addr = (value as u16) << 1;
            }

            0x2103 => {
                ppu_regs.oam_write_high_table = get_bit_n!(value, 0);
                ppu_regs.priority_rotation = get_bit_n!(value, 7);
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
                ppu_regs.bg4_char_size = if get_bit_n!(value, 7) {
                    TileSize::Size16x16
                } else {
                    TileSize::Size8x8
                };
                ppu_regs.bg3_char_size = if get_bit_n!(value, 6) {
                    TileSize::Size16x16
                } else {
                    TileSize::Size8x8
                };
                ppu_regs.bg2_char_size = if get_bit_n!(value, 5) {
                    TileSize::Size16x16
                } else {
                    TileSize::Size8x8
                };
                ppu_regs.bg1_char_size = if get_bit_n!(value, 4) {
                    TileSize::Size16x16
                } else {
                    TileSize::Size8x8
                };
                ppu_regs.bg3_mode1_priority = get_bit_n!(value, 3);
                ppu_regs.bg_mode = match value & 7 {
                    0 => BgMode::Mode0,
                    1 => BgMode::Mode1,
                    2 => BgMode::Mode2,
                    3 => BgMode::Mode3,
                    4 => BgMode::Mode4,
                    5 => BgMode::Mode5,
                    6 => BgMode::Mode6,
                    7 => BgMode::Mode7,
                    _ => unreachable!(),
                };

                match ppu_regs.bg_mode {
                    BgMode::Mode5 | BgMode::Mode6 => {
                        ppu_regs.hi_res_en = true;
                    }
                    _ => {}
                };

                // println!("Set Bg Mode to {:?} and bg3 priority to {}, bg tile sizes to bg1: {:?}, bg2: {:?}, bg3: {:?}, bg4: {:?}",
                //     ppu_regs.bg_mode.get(),
                //     ppu_regs.bg3_mode1_priority.get(),
                //     ppu_regs.bg1_char_size.get(),
                //     ppu_regs.bg2_char_size.get(),
                //     ppu_regs.bg3_char_size.get(),
                //     ppu_regs.bg4_char_size.get(),
                // );
            }

            0x2106 => {
                ppu_regs.mosaic_size = value >> 4;
                ppu_regs.bg4_mosaic_en = get_bit_n!(value, 3);
                ppu_regs.bg3_mosaic_en = get_bit_n!(value, 2);
                ppu_regs.bg2_mosaic_en = get_bit_n!(value, 1);
                ppu_regs.bg1_mosaic_en = get_bit_n!(value, 0);
            }

            0x2107 => {
                ppu_regs.bg1_vram_addr = (value >> 2) as u8;
                ppu_regs.bg1_tilemap_count_y = if get_bit_n!(value, 1) {
                    TilemapCount::Two
                } else {
                    TilemapCount::One
                };
                ppu_regs.bg1_tilemap_count_x = if get_bit_n!(value, 0) {
                    TilemapCount::Two
                } else {
                    TilemapCount::One
                };

                // println!("Set Bg1 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}",
                //     (ppu_regs.bg1_vram_addr as u16) << 10,
                //     ppu_regs.bg1_tilemap_count_x,
                //     ppu_regs.bg1_tilemap_count_y
                // );
            }

            0x2108 => {
                ppu_regs.bg2_vram_addr = value >> 2;
                ppu_regs.bg2_tilemap_count_y = if get_bit_n!(value, 1) {
                    TilemapCount::Two
                } else {
                    TilemapCount::One
                };
                ppu_regs.bg2_tilemap_count_x = if get_bit_n!(value, 0) {
                    TilemapCount::Two
                } else {
                    TilemapCount::One
                };

                // println!("Set Bg2 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}",
                //     (ppu_regs.bg2_vram_addr.get() as u16) << 10,
                //     ppu_regs.bg2_tilemap_count_x.get(),
                //     ppu_regs.bg2_tilemap_count_y.get()
                // );
            }

            0x2109 => {
                ppu_regs.bg3_vram_addr = value >> 2;
                ppu_regs.bg3_tilemap_count_y = if get_bit_n!(value, 1) {
                    TilemapCount::Two
                } else {
                    TilemapCount::One
                };
                ppu_regs.bg3_tilemap_count_x = if get_bit_n!(value, 0) {
                    TilemapCount::Two
                } else {
                    TilemapCount::One
                };

                // println!("Set Bg3 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}",
                //     (ppu_regs.bg3_vram_addr.get() as u16) << 10,
                //     ppu_regs.bg3_tilemap_count_x.get(),
                //     ppu_regs.bg3_tilemap_count_y.get()
                // );
            }

            0x210A => {
                ppu_regs.bg4_vram_addr = value >> 2;
                ppu_regs.bg4_tilemap_count_y = if get_bit_n!(value, 1) {
                    TilemapCount::Two
                } else {
                    TilemapCount::One
                };
                ppu_regs.bg4_tilemap_count_x = if get_bit_n!(value, 0) {
                    TilemapCount::Two
                } else {
                    TilemapCount::One
                };

                // println!("Set Bg4 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}",
                //     (ppu_regs.bg4_vram_addr.get() as u16) << 10,
                //     ppu_regs.bg4_tilemap_count_x.get(),
                //     ppu_regs.bg4_tilemap_count_y.get()
                // );
            }

            0x210B => {
                ppu_regs.bg2_chr_base_addr = value >> 4;
                ppu_regs.bg1_chr_base_addr = value & 0x0F;

                // println!("Set Bg1 chr base address to ${:04X}", (ppu_regs.bg1_chr_base_addr.get() as u16) << 12);
                // println!("Set Bg2 chr base address to ${:04X}", (ppu_regs.bg2_chr_base_addr.get() as u16) << 12);
            }

            0x210C => {
                ppu_regs.bg4_chr_base_addr = value >> 4;
                ppu_regs.bg3_chr_base_addr = value & 0x0F;

                // println!("Set Bg3 chr base address to ${:04X}", (ppu_regs.bg3_chr_base_addr.get() as u16) << 12);
                // println!("Set Bg4 chr base address to ${:04X}", (ppu_regs.bg4_chr_base_addr.get() as u16) << 12);
            }

            0x210D => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                let bghofs_latch = ppu_regs.bg_offset_x_latch as u16;
                ppu_regs.bg_offset_latch = value;
                ppu_regs.bg_offset_x_latch = value;

                ppu_regs.bg1_m7_x_offset =
                    (((value & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
            }

            0x210E => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                ppu_regs.bg_offset_latch = value;

                ppu_regs.bg1_m7_y_offset = (((value & 3) as u16) << 8) | bgofs_latch;
            }

            0x210F => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                let bghofs_latch = ppu_regs.bg_offset_x_latch as u16;
                ppu_regs.bg_offset_latch = value;
                ppu_regs.bg_offset_x_latch = value;

                ppu_regs.bg2_x_offset =
                    (((value & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
            }

            0x2110 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                ppu_regs.bg_offset_latch = value;

                ppu_regs.bg2_y_offset = (((value & 3) as u16) << 8) | bgofs_latch;
            }

            0x2111 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                let bghofs_latch = ppu_regs.bg_offset_x_latch as u16;
                ppu_regs.bg_offset_latch = value;
                ppu_regs.bg_offset_x_latch = value;

                ppu_regs.bg3_x_offset =
                    (((value & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
            }

            0x2112 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                ppu_regs.bg_offset_latch = value;

                ppu_regs.bg3_y_offset = (((value & 3) as u16) << 8) | bgofs_latch;
            }

            0x2113 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                let bghofs_latch = ppu_regs.bg_offset_x_latch as u16;
                ppu_regs.bg_offset_latch = value;
                ppu_regs.bg_offset_x_latch = value;

                ppu_regs.bg4_x_offset =
                    (((value & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
            }

            0x2114 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                ppu_regs.bg_offset_latch = value;

                ppu_regs.bg4_y_offset = (((value & 3) as u16) << 8) | bgofs_latch;
            }

            0x2115 => {
                ppu_regs.vram_addr_inc_mode = if get_bit_n!(value, 7) {
                    VramIncMode::HighByte
                } else {
                    VramIncMode::LowByte
                };
                ppu_regs.addr_remap_mode = match (value >> 2) & 3 {
                    0 => AddressRemapping::None,
                    1 => AddressRemapping::ColDepth2,
                    2 => AddressRemapping::ColDepth4,
                    3 => AddressRemapping::ColDepth8,
                    _ => unreachable!(),
                };
                ppu_regs.addr_inc_size = match value & 3 {
                    0 => IncrSize::Bytes2,
                    1 => IncrSize::Bytes64,
                    2 => IncrSize::Bytes256,
                    3 => IncrSize::Bytes256,
                    _ => unreachable!(),
                };
            }

            0x2116 => {
                set_byte_n!(ppu_regs.vram_addr, value as u16, 0);
                ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];

                // println!("Set vram addr (lo) to ${:04X}", ppu_regs.vram_addr.get());
            }

            0x2117 => {
                set_byte_n!(ppu_regs.vram_addr, value as u16, 1);
                ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];

                // println!("Set vram addr (hi) to ${:04X}", ppu_regs.vram_addr.get());
            }

            0x2118 => {
                if ppu_regs.in_fblank || ppu_regs.in_vblank {
                    set_byte_n!(self.vram[ppu_regs.get_vram_addr() as usize], value as u16, 0);
                }

                match ppu_regs.vram_addr_inc_mode {
                    VramIncMode::LowByte => ppu_regs.inc_vram_addr(),
                    _ => {}
                }
            }

            0x2119 => {
                if ppu_regs.in_fblank || ppu_regs.in_vblank {
                    set_byte_n!(self.vram[ppu_regs.get_vram_addr() as usize], value as u16, 1);
                }

                match ppu_regs.vram_addr_inc_mode {
                    VramIncMode::HighByte => ppu_regs.inc_vram_addr(),
                    _ => {}
                }
            }

            0x211A => {
                ppu_regs.m7_tilemap_repeat = get_bit_n!(value, 7);
                ppu_regs.m7_fill_mode = if get_bit_n!(value, 6) {
                    M7FillMode::Character
                } else {
                    M7FillMode::Transparent
                };
                ppu_regs.m7_flip_bg_y = get_bit_n!(value, 1);
                ppu_regs.m7_flip_bg_x = get_bit_n!(value, 0);
            }

            0x211B => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = value;

                ppu_regs.m7_matrix_a = ((value as u16) << 8) | latched_val;
                ppu_regs.mult_factor_16 = ((value as u16) << 8) | latched_val;

                ppu_regs.update_multiply_result();
            }

            0x211C => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = value;

                ppu_regs.m7_matrix_b = ((value as u16) << 8) | latched_val;
                ppu_regs.mult_factor_8 = latched_val as u8;

                ppu_regs.update_multiply_result();
            }

            0x211D => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = value;

                ppu_regs.m7_matrix_c = ((value as u16) << 8) | latched_val;
            }

            0x211E => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = value;

                ppu_regs.m7_matrix_d = ((value as u16) << 8) | latched_val;
            }

            0x211F => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = value;

                ppu_regs.m7_center_x = ((value as u16) << 8) | latched_val;
            }

            0x2120 => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = value;

                ppu_regs.m7_center_y = ((value as u16) << 8) | latched_val;
            }

            0x2121 => {
                ppu_regs.cgram_addr = value;
                ppu_regs.cgram_toggle = false;
            }

            0x2122 => {
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
                ppu_regs.bg2_w2_en = get_bit_n!(value, 7);
                ppu_regs.bg2_w2_inv = get_bit_n!(value, 6);
                ppu_regs.bg2_w1_en = get_bit_n!(value, 5);
                ppu_regs.bg2_w1_inv = get_bit_n!(value, 4);
                ppu_regs.bg1_w2_en = get_bit_n!(value, 3);
                ppu_regs.bg1_w2_inv = get_bit_n!(value, 2);
                ppu_regs.bg1_w1_en = get_bit_n!(value, 1);
                ppu_regs.bg1_w1_inv = get_bit_n!(value, 0);
            }

            0x2124 => {
                ppu_regs.bg4_w2_en = get_bit_n!(value, 7);
                ppu_regs.bg4_w2_inv = get_bit_n!(value, 6);
                ppu_regs.bg4_w1_en = get_bit_n!(value, 5);
                ppu_regs.bg4_w1_inv = get_bit_n!(value, 4);
                ppu_regs.bg3_w2_en = get_bit_n!(value, 3);
                ppu_regs.bg3_w2_inv = get_bit_n!(value, 2);
                ppu_regs.bg3_w1_en = get_bit_n!(value, 1);
                ppu_regs.bg3_w1_inv = get_bit_n!(value, 0);
            }

            0x2125 => {
                ppu_regs.col_w2_en = get_bit_n!(value, 7);
                ppu_regs.col_w2_inv = get_bit_n!(value, 6);
                ppu_regs.col_w1_en = get_bit_n!(value, 5);
                ppu_regs.col_w1_inv = get_bit_n!(value, 4);
                ppu_regs.obj_w2_en = get_bit_n!(value, 3);
                ppu_regs.obj_w2_inv = get_bit_n!(value, 2);
                ppu_regs.obj_w1_en = get_bit_n!(value, 1);
                ppu_regs.obj_w1_inv = get_bit_n!(value, 0);
            }

            0x2126 => {
                ppu_regs.w1_left_pos = value;
            }
            0x2127 => {
                ppu_regs.w1_right_pos = value;
            }
            0x2128 => {
                ppu_regs.w2_left_pos = value;
            }
            0x2129 => {
                ppu_regs.w2_right_pos = value;
            }

            0x212A => {
                ppu_regs.bg4_win_logic = match value >> 6 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
                ppu_regs.bg3_win_logic = match (value >> 4) & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
                ppu_regs.bg2_win_logic = match (value >> 2) & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
                ppu_regs.bg1_win_logic = match value & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
            }

            0x212B => {
                ppu_regs.col_win_logic = match (value >> 2) & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
                ppu_regs.obj_win_logic = match value & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
            }

            0x212C => {
                ppu_regs.obj_main_en = get_bit_n!(value, 4);
                ppu_regs.bg4_main_en = get_bit_n!(value, 3);
                ppu_regs.bg3_main_en = get_bit_n!(value, 2);
                ppu_regs.bg2_main_en = get_bit_n!(value, 1);
                ppu_regs.bg1_main_en = get_bit_n!(value, 0);

                // println!("Set main en flags to Bg1: {}, Bg2: {}, Bg3: {}, Bg4: {}, Obj: {}",
                //     ppu_regs.bg1_main_enabled.get(),
                //     ppu_regs.bg2_main_enabled.get(),
                //     ppu_regs.bg3_main_enabled.get(),
                //     ppu_regs.bg4_main_enabled.get(),
                //     ppu_regs.obj_main_enabled.get(),
                // );
            }

            0x212D => {
                ppu_regs.obj_sub_en = get_bit_n!(value, 4);
                ppu_regs.bg4_sub_en = get_bit_n!(value, 3);
                ppu_regs.bg3_sub_en = get_bit_n!(value, 2);
                ppu_regs.bg2_sub_en = get_bit_n!(value, 1);
                ppu_regs.bg1_sub_en = get_bit_n!(value, 0);
            }

            0x212E => {
                ppu_regs.obj_win_main_en = get_bit_n!(value, 4);
                ppu_regs.bg4_win_main_en = get_bit_n!(value, 3);
                ppu_regs.bg3_win_main_en = get_bit_n!(value, 2);
                ppu_regs.bg2_win_main_en = get_bit_n!(value, 1);
                ppu_regs.bg1_win_main_en = get_bit_n!(value, 0);
            }

            0x212F => {
                ppu_regs.obj_win_sub_en = get_bit_n!(value, 4);
                ppu_regs.bg4_win_sub_en = get_bit_n!(value, 3);
                ppu_regs.bg3_win_sub_en = get_bit_n!(value, 2);
                ppu_regs.bg2_win_sub_en = get_bit_n!(value, 1);
                ppu_regs.bg1_win_sub_en = get_bit_n!(value, 0);
            }

            0x2130 => {
                ppu_regs.col_win_main_region = match value >> 6 {
                    0 => WindowColorRegion::Nowhere,
                    1 => WindowColorRegion::Outside,
                    2 => WindowColorRegion::Inside,
                    3 => WindowColorRegion::Everywhere,
                    _ => unreachable!(),
                };
                ppu_regs.col_win_sub_region = match (value >> 4) & 3 {
                    0 => WindowColorRegion::Nowhere,
                    1 => WindowColorRegion::Outside,
                    2 => WindowColorRegion::Inside,
                    3 => WindowColorRegion::Everywhere,
                    _ => unreachable!(),
                };
                ppu_regs.sub_color_fixed = !get_bit_n!(value, 1);
                ppu_regs.use_direct_col = get_bit_n!(value, 0);
            }

            0x2131 => {
                ppu_regs.cmath_operator = if get_bit_n!(value, 7) {
                    CMathOperator::Subtract
                } else {
                    CMathOperator::Add
                };
                ppu_regs.cmath_half = get_bit_n!(value, 6);
                ppu_regs.back_cmath_en = get_bit_n!(value, 5);
                ppu_regs.obj_cmath_en = get_bit_n!(value, 4);
                ppu_regs.bg4_cmath_en = get_bit_n!(value, 3);
                ppu_regs.bg3_cmath_en = get_bit_n!(value, 2);
                ppu_regs.bg2_cmath_en = get_bit_n!(value, 1);
                ppu_regs.bg1_cmath_en = get_bit_n!(value, 0);
            }

            0x2132 => {
                let new_val = value & 0x1F;
                
                if get_bit_n!(value, 7) { ppu_regs.fixed_color.b = new_val; }
                if get_bit_n!(value, 6) { ppu_regs.fixed_color.g = new_val; }
                if get_bit_n!(value, 5) { ppu_regs.fixed_color.r = new_val; }
            }

            0x2133 => {
                ppu_regs._external_sync = get_bit_n!(value, 7);
                ppu_regs.ext_bg_en = get_bit_n!(value, 6);
                ppu_regs.hi_res_en = get_bit_n!(value, 3);
                ppu_regs.overscan_en = get_bit_n!(value, 2);
                ppu_regs.obj_interlace_en = get_bit_n!(value, 1);
                ppu_regs.screen_interlace_en = get_bit_n!(value, 0);
            }

            _ => {}
        }
    }
    
    fn read_cpuio_regs(&mut self, offset: u16) -> u8 {
        match offset {
            0x4000..=0x4015 => 0, // Open bus
            
            0x4016 => {
                *self.joypad_cmd = Some(JoypadCmd::ClockJoy1);
                
                let joy1_data1 = (self.joy1_in & 1) as u8;
                let joy1_data2 = 0x00; // unused for joypads
                
                joy1_data2 | joy1_data1
            },
            
            0x4017 => {
                const ALWAYS_ON: u8 = 0x1C;
                
                *self.joypad_cmd = Some(JoypadCmd::ClockJoy2);
                
                let joy2_data1 = (self.joy2_in & 1) as u8;
                let joy2_data2 = 0x00; // unused for joypads
                
                ALWAYS_ON | joy2_data2 | joy2_data1
            },
            
            0x4018..=0x41FF => 0, // Open bus
            
            0x4200..=0x420D => 0, // Write-only registers
            
            0x420E..=0x420F => 0, // Open bus
            
            0x4210 => {
                let vblank_nmi = if self.cpu_regs.vblank_nmi_flag { 0x80 } else { 0 };
                let cpu_version = 0x02;
                
                self.cpu_regs.vblank_nmi_flag = false;
                
                vblank_nmi | cpu_version
            }
            
            0x4211 => {
                let timer_irq = if self.cpu_regs.hv_timer_irq_flag { 0x80 } else { 0 };
                
                self.cpu_regs.hv_timer_irq_flag = false;
                
                timer_irq
            }
            
            0x4212 => {
                let in_vblank = if self.cpu_regs.vblank_flag { 0x80 } else { 0 };
                let in_hblank = if self.cpu_regs.hblank_flag { 0x40 } else { 0 };
                let in_joypad_autoread = if self.cpu_regs.joypad_autoread_flag { 1 } else { 0 };
                
                in_vblank | in_hblank | in_joypad_autoread
            }
            
            0x4213 => self.cpu_regs.rdio,
            
            0x4214 => get_byte_n!(self.mult.div_quotient, 0),
            0x4215 => get_byte_n!(self.mult.div_quotient, 1),
            
            0x4216 => get_byte_n!(self.mult.result, 0),
            0x4217 => get_byte_n!(self.mult.result, 1),
            
            0x4218 => get_byte_n!(self.joy1_data1_auto, 0),
            0x4219 => get_byte_n!(self.joy1_data1_auto, 1),
            0x421A => get_byte_n!(self.joy2_data1_auto, 0),
            0x421B => get_byte_n!(self.joy2_data1_auto, 1),
            0x421C => get_byte_n!(self.joy1_data2_auto, 0),
            0x421D => get_byte_n!(self.joy1_data2_auto, 1),
            0x421E => get_byte_n!(self.joy2_data2_auto, 0),
            0x421F => get_byte_n!(self.joy2_data2_auto, 1),
            
            _ => 0,
        }
    }
    
    fn write_cpuio_regs(&mut self, offset: u16, value: u8) {
        match offset {
            0x4000..=0x4015 => {}, // Open bus
            
            0x4016 => {
                self.cpu_regs.latch_controllers = get_bit_n!(value, 0);
            },
            
            0x4017 => {}, // Write-only register
            
            0x4018..=0x41FF => {}, // Open bus
            
            0x4200 => {
                self.cpu_regs.vblank_nmi_en = get_bit_n!(value, 7);
                self.cpu_regs.joypad_autoread_en = get_bit_n!(value, 0);
                
                self.cpu_regs.hv_timer_irq_mode = match (value >> 4) & 3 {
                    0 => HVTimerIRQ::None,
                    1 => HVTimerIRQ::HTimer,
                    2 => HVTimerIRQ::VTimer,
                    3 => HVTimerIRQ::Both,
                    _ => unreachable!(),
                };
            }
            
            0x4201 => {
                // TODO: Implement joypad IO interface
            }
            
            0x4202 => {
                self.mult.mult_factor1 = value;
            }
            
            0x4203 => {
                self.mult.mult_factor2 = value;
                // TODO: Make mult circuit take cycles to compute result
                self.mult.result = self.mult.mult_factor1 as u16 * self.mult.mult_factor2 as u16;
            }
            
            0x4204 => { set_byte_n!(self.mult.div_numer, value as u16, 0); }
            0x4205 => { set_byte_n!(self.mult.div_numer, value as u16, 1); }
            
            0x4206 => {
                self.mult.div_denom = value;
                if self.mult.div_denom == 0 {
                    self.mult.div_quotient = 0xFFFF;
                    self.mult.result = self.mult.div_numer;
                } else {
                    self.mult.div_quotient = self.mult.div_numer / (self.mult.div_denom as u16);
                    self.mult.result = self.mult.div_numer % (self.mult.div_denom as u16);
                }
            }

            0x4207 => { set_byte_n!(self.cpu_regs.h_counter_target, value as u16, 0); }
            0x4208 => { set_byte_n!(self.cpu_regs.h_counter_target, (value & 1) as u16, 1); }
            0x4209 => { set_byte_n!(self.cpu_regs.v_counter_target, value as u16, 0); }
            0x420A => { set_byte_n!(self.cpu_regs.v_counter_target, (value & 1) as u16, 1); }
            
            0x420B => {
                *self.dma_en = value != 0;
                *self.dma_active_ch = value.trailing_zeros() as usize;
                for i in 0..8 {                    
                    self.dma_regs[i].dma_en = get_bit_n!(value, i);
                    
                    if self.dma_regs[i].dma_en {
                        self.dma_regs[i].transfer_pattern_step = 0;
                    }
                }
            }
            
            0x420C => {
                *self.hdma_en = value != 0;
                *self.hdma_active_ch = value.trailing_zeros() as usize;
                for i in 0..8 {
                    self.dma_regs[i].hdma_en = get_bit_n!(value, i);
                }
            }
            
            0x4210..=0x42FF => {}, // Read-only regs
            
            _ => {},
        }
    }
    
    fn read_dma_regs(&mut self, offset: u16) -> u8 {
        let channel_idx = ((offset >> 4) & 0xF) as usize;
        
        if channel_idx >= 7 {
            return 0; // TODO: Maybe mirror channel_idx & 7?
        }
        
        let channel = &mut self.dma_regs[channel_idx];
        
        match offset & 0xF {
            0x0 => channel.params_raw,
            0x1 => channel.b_bus_addr.offset as u8,
            0x2 => get_byte_n!(channel.a_bus_addr.offset, 0),
            0x3 => get_byte_n!(channel.a_bus_addr.offset, 1),
            0x4 => channel.a_bus_addr.bank,
            0x5 => get_byte_n!(channel.hdma_indirect_table_addr.offset, 0),
            0x6 => get_byte_n!(channel.hdma_indirect_table_addr.offset, 1),
            0x7 => channel.hdma_indirect_table_addr.bank,
            0x8 => get_byte_n!(channel.hdma_table_offset, 0),
            0x9 => get_byte_n!(channel.hdma_table_offset, 1),
            0xA => {
                let hdma_reload = if channel.hdma_reload_flag { 0x80 } else { 0 };
                hdma_reload | channel.scanline_counter
            },
            0xB => channel.unused,
            0xC..=0xE => 0, // Open bus
            0xF => channel.unused,
            _ => unreachable!(),
        }
    }
    
    fn write_dma_regs(&mut self, offset: u16, value: u8) {
        let channel_idx = ((offset >> 4) & 0xF) as usize;
        
        if channel_idx >= 7 {
            return; // TODO: Maybe mirror channel_idx & 7?
        }
        
        let channel = &mut self.dma_regs[channel_idx];
        
        match offset & 0xF {
            0x0 => {
                channel.params_raw = value;
                
                channel.direction = match get_bit_n!(value, 7) {
                    true => Direction::AtoB,
                    false => Direction::BtoA,
                };
                channel.indirect_hdma = get_bit_n!(value, 6);
                channel.inc_mode = match (value >> 3) & 3 {
                    0 => AddressIncMode::Inc,
                    1 => AddressIncMode::Fixed,
                    2 => AddressIncMode::Dec,
                    3 => AddressIncMode::Fixed,
                    _ => unreachable!(),
                };
                channel.transfer_pattern = match value & 7 {
                    0 => TransferPattern::Pattern0,
                    1 => TransferPattern::Pattern1,
                    2 => TransferPattern::Pattern2,
                    3 => TransferPattern::Pattern3,
                    4 => TransferPattern::Pattern4,
                    5 => TransferPattern::Pattern5,
                    6 => TransferPattern::Pattern6,
                    7 => TransferPattern::Pattern7,
                    _ => unreachable!(),
                };
            },
            
            0x1 => { channel.b_bus_addr = Address { bank: 0, offset: value as u16 }; },
            0x2 => { set_byte_n!(channel.a_bus_addr.offset, value as u16, 0); },
            0x3 => { set_byte_n!(channel.a_bus_addr.offset, value as u16, 1); },
            0x4 => { channel.a_bus_addr.bank = value; },
            0x5 => { set_byte_n!(channel.hdma_indirect_table_addr.offset, value as u16, 0); },
            0x6 => { set_byte_n!(channel.hdma_indirect_table_addr.offset, value as u16, 1); },
            0x7 => { channel.hdma_indirect_table_addr.bank = value; },
            0x8 => { set_byte_n!(channel.hdma_table_offset, value as u16, 0); },
            0x9 => { set_byte_n!(channel.hdma_table_offset, value as u16, 1); },
            0xA => {
                channel.hdma_reload_flag = get_bit_n!(value, 7);
                channel.scanline_counter = value & 0x7F;
            },
            0xB => { channel.unused = value; },
            0xC..=0xE => {}, // Open bus
            0xF => { channel.unused = value; },
            _ => unreachable!(),
        }
    }
}
