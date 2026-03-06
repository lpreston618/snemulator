use crate::core::sppu::color::Color;
use crate::core::sysinfo::{WRAM_SIZE, VRAM_SIZE, CGRAM_SIZE, OAM_SIZE};
use crate::core::sppu::regs::{AddressRemapping, BgMode, CMathOperator, IncrSize, M7FillMode, MasterSlave, ObjectSizeSelect, PpuRegs, TileSize, TilemapCount, VideoType, VramIncMode, WindowColorRegion, WindowLogic};
use crate::{get_bit_n, get_byte_n, set_byte_n};

#[derive(Debug, Clone, Copy)]
pub struct Address {
    pub bank: u8,
    pub offset: u16,
}

impl Address {
    pub fn to_u32(self) -> u32 { (self.bank as u32) << 16 | self.offset as u32 }
    pub fn from_u32(val: u32) -> Self { Address { bank: (val >> 16) as u8, offset: val as u16 } }
}

pub struct CpuBus<'a> {
    pub wram: &'a mut [u8; WRAM_SIZE],
    pub vram: &'a mut [u16; VRAM_SIZE],
    pub cgram: &'a mut [Color; CGRAM_SIZE],
    pub oam: &'a mut [u8; OAM_SIZE],
    pub ppu_regs: &'a mut PpuRegs,
    // pub apu_ports: &'a mut ApuPorts,
    // pub cpu_regs: &'a mut CpuIoRegs,
    // pub cart: &'a mut Cartridge, // or rom or what have you
}

impl<'a> CpuBus<'a> {
    pub fn read(&self, addr: Address) -> u8 {
        match addr.bank {
            // Banks $00-$3F: LoROM mapping
            0x00..=0x3F => match addr.offset {
                // WRAM mirror (first 8KB)
                0x0000..=0x1FFF => self.wram[addr.offset as usize],
                
                // PPU registers
                0x2100..=0x213F => self.read_ppu_regs(addr.offset),
                
                // APU ports
                0x2140..=0x217F => 0, // self.apu_ports.read_from_cpu(addr.offset & 0x3),
                
                // S-WRAM access registers
                0x2180..=0x2183 => self.read_wram_port(addr.offset),
                
                // CPU I/O registers (joypad, DMA, IRQ, etc.)
                0x4000..=0x43FF => self.cpu_regs.read(addr.offset),
                
                // Cartridge (LoROM: $8000-$FFFF)
                0x8000..=0xFFFF => self.cart.read(addr),
                
                _ => 0 // Open bus
            },
            
            // Banks $40-$6F: LoROM cartridge
            0x40..=0x6F => self.cart.read(addr),
            
            // Banks $70-$7D: SRAM (typically)
            0x70..=0x7D => self.cart.read(addr),
            
            // Banks $7E-$7F: WRAM (full 128KB)
            0x7E..=0x7F => {
                let wram_addr = ((addr.bank as usize & 1) << 16) | (addr.offset as usize);
                self.wram[wram_addr]
            },
            
            // Banks $80-$BF: Mirror of $00-$3F
            0x80..=0xBF => self.read(Address { bank: addr.bank & 0x7F, offset: addr.offset }),
            
            // Banks $C0-$FF: HiROM cartridge / mirror
            0xC0..=0xFF => self.cart.read(addr),
        }
    }
    
    pub fn write(&mut self, addr: Address, value: u8) {
        match addr.bank {
            // WRAM mirror
            0x00..=0x3F => match addr.offset {
                0x0000..=0x1FFF => self.wram[addr.offset as usize] = value,
                
                // PPU registers
                0x2100..=0x213F => self.write_ppu_regs(addr.offset, value),
                
                // APU ports
                0x2140..=0x217F => self.apu_ports.write_from_cpu(addr.offset & 0x3, value),
                
                // WRAM access port
                0x2180..=0x2183 => self.write_wram_port(addr.offset, value),
                
                // CPU I/O registers
                0x4000..=0x43FF => self.cpu_regs.write(addr.offset, value),
                
                // Cartridge (SRAM, mapper registers)
                0x8000..=0xFFFF => self.cart.write(addr, value),
                
                _ => {}
            },
            
            // WRAM direct access
            0x7E..=0x7F => {
                let wram_addr = ((addr.bank as usize & 1) << 16) | (addr.offset as usize);
                self.wram[wram_addr] = value;
            },
            
            // Mirror
            0x80..=0xBF => self.write(Address { bank: addr.bank & 0x7F, offset: addr.offset }, value),
            
            // Cartridge
            _ => self.cart.write(addr, value),
        }
    }
    
    fn read_wram_port(&mut self, offset: u16) -> u8 {
        match offset {
            0x2180 => {
                let addr = self.cpu_regs.wram_addr as usize;
                let value = self.wram[addr & 0x1FFFF];
                self.cpu_regs.wram_addr = self.cpu_regs.wram_addr.wrapping_add(1) & 0x1FFFF;
                value
            }
            _ => 0
        }
    }
    
    fn write_wram_port(&mut self, offset: u16, value: u8) {
        match offset {
            0x2180 => {
                let addr = self.cpu_regs.wram_addr as usize;
                self.wram[addr & 0x1FFFF] = value;
                self.cpu_regs.wram_addr = self.cpu_regs.wram_addr.wrapping_add(1) & 0x1FFFF;
            }
            0x2181 => {
                self.cpu_regs.wram_addr = (self.cpu_regs.wram_addr & 0x1FF00) | (value as u32);
            }
            0x2182 => {
                self.cpu_regs.wram_addr = (self.cpu_regs.wram_addr & 0x100FF) | ((value as u32) << 8);
            }
            0x2183 => {
                self.cpu_regs.wram_addr = (self.cpu_regs.wram_addr & 0x0FFFF) | (((value & 1) as u32) << 16);
            }
            _ => {}
        }
    }
    
    fn read_ppu_regs(&mut self, offset: u16) -> u8 {
        let ppu_regs = &mut self.ppu_regs;
        
        match offset {
            0x2134 => { get_byte_n!(ppu_regs.multiply_result, 0) }
            0x2135 => { get_byte_n!(ppu_regs.multiply_result, 1) }
            0x2136 => { get_byte_n!(ppu_regs.multiply_result, 2) }
            
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
                let spr_tile_overflow_bit = if ppu_regs.sprite_tile_overflow { 0x40 } else { 0 };
                let master_slave_bit = match ppu_regs.master_slave_state {
                    MasterSlave::Master => 0x20,
                    MasterSlave::Slave => 0,
                };
                let ppu1_open_bus = 0;
                let ppu1_version_bits = ppu_regs.ppu1_version & 0x0F;

                spr_overflow_bit | spr_tile_overflow_bit | master_slave_bit | ppu1_open_bus | ppu1_version_bits
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
            
            _ => { 0 }
        }
    }
    
    pub fn write_ppu_regs(&mut self, offset: u16, data: u8) {
        let ppu_regs = &mut self.ppu_regs;
        
        match offset {
            0x2100 => {
                ppu_regs.in_fblank = get_bit_n!(data, 7);
                ppu_regs.screen_brightness = data & 0x0F;

                // println!("Set fblank to {}, S: {} D: {}", self.in_fblank.get(), self.scanline.get(), self.dot.get());
            }

            0x2101 => {
                let new_obj_size = match data >> 5 {
                    0 => ObjectSizeSelect::Size8x8_16x16,
                    1 => ObjectSizeSelect::Size8x8_32x32,
                    2 => ObjectSizeSelect::Size8x8_64x64,
                    3 => ObjectSizeSelect::Size16x16_32x32,
                    4 => ObjectSizeSelect::Size16x16_64x64,
                    5 => ObjectSizeSelect::Size32x32_64x64,
                    6 => ObjectSizeSelect::Size16x32_32x64,
                    7 => ObjectSizeSelect::Size16x32_32x32,
                    _ => unreachable!()
                };

                ppu_regs.obj_sprite_size = new_obj_size;
                ppu_regs.name_secondary_select = (data >> 3) & 0x03;
                ppu_regs.name_base_addr = data & 0x03;

                // println!("Set name base addr to ${:04X}", (ppu_regs.name_base_addr.get() as u16) << 13);
            }

            0x2102 => {
                set_byte_n!(ppu_regs.oam_addr, data as u16, 0);
                ppu_regs.priority_rotation_idx = data & 0xFE;
                ppu_regs.internal_oam_addr = (ppu_regs.oam_addr & 0x1FF) << 1;
            }

            0x2103 => {
                set_byte_n!(ppu_regs.oam_addr, data as u16, 1);
                ppu_regs.priority_rotation = get_bit_n!(data, 7);
                ppu_regs.internal_oam_addr = (ppu_regs.oam_addr & 0x1FF) << 1;
            }

            0x2104 => {
                let internal_oam_addr = ppu_regs.internal_oam_addr as usize;

                if internal_oam_addr & 1 == 0 {
                    ppu_regs.oam_data_latch = data;
                } else if internal_oam_addr < 0x200 {
                    self.oam[internal_oam_addr - 1] = ppu_regs.oam_data_latch;
                    self.oam[internal_oam_addr] = data;
                }
                
                if internal_oam_addr >= 0x200 {
                    self.oam[internal_oam_addr] = data;
                }

                ppu_regs.internal_oam_addr += 1;
                ppu_regs.internal_oam_addr %= OAM_SIZE as u16;
            }

            0x2105 => {
                ppu_regs.bg4_char_size = if get_bit_n!(data, 7) { TileSize::Size16x16 } else { TileSize::Size8x8 };
                ppu_regs.bg3_char_size = if get_bit_n!(data, 6) { TileSize::Size16x16 } else { TileSize::Size8x8 };
                ppu_regs.bg2_char_size = if get_bit_n!(data, 5) { TileSize::Size16x16 } else { TileSize::Size8x8 };
                ppu_regs.bg1_char_size = if get_bit_n!(data, 4) { TileSize::Size16x16 } else { TileSize::Size8x8 };
                ppu_regs.bg3_mode1_priority = get_bit_n!(data, 3);
                ppu_regs.bg_mode = match data & 7 {
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
                    BgMode::Mode5 | BgMode::Mode6 => { ppu_regs.hi_res_enabled = true; },
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
                ppu_regs.mosaic_size = data >> 4;
                ppu_regs.bg4_mosaic = get_bit_n!(data, 3);
                ppu_regs.bg3_mosaic = get_bit_n!(data, 2);
                ppu_regs.bg2_mosaic = get_bit_n!(data, 1);
                ppu_regs.bg1_mosaic = get_bit_n!(data, 0);
            }

            0x2107 => {
                ppu_regs.bg1_vram_addr = (data >> 2) as u8;
                ppu_regs.bg1_tilemap_count_y = if get_bit_n!(data, 1) { TilemapCount::Two } else { TilemapCount::One };
                ppu_regs.bg1_tilemap_count_x = if get_bit_n!(data, 0) { TilemapCount::Two } else { TilemapCount::One };

                // println!("Set Bg1 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}", 
                //     (ppu_regs.bg1_vram_addr as u16) << 10, 
                //     ppu_regs.bg1_tilemap_count_x, 
                //     ppu_regs.bg1_tilemap_count_y
                // );
            }

            0x2108 => {
                ppu_regs.bg2_vram_addr = data >> 2;
                ppu_regs.bg2_tilemap_count_y = if get_bit_n!(data, 1) { TilemapCount::Two } else { TilemapCount::One };
                ppu_regs.bg2_tilemap_count_x = if get_bit_n!(data, 0) { TilemapCount::Two } else { TilemapCount::One };

                // println!("Set Bg2 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}", 
                //     (ppu_regs.bg2_vram_addr.get() as u16) << 10, 
                //     ppu_regs.bg2_tilemap_count_x.get(), 
                //     ppu_regs.bg2_tilemap_count_y.get()
                // );
            }

            0x2109 => {
                ppu_regs.bg3_vram_addr = data >> 2;
                ppu_regs.bg3_tilemap_count_y = if get_bit_n!(data, 1) { TilemapCount::Two } else { TilemapCount::One };
                ppu_regs.bg3_tilemap_count_x = if get_bit_n!(data, 0) { TilemapCount::Two } else { TilemapCount::One };

                // println!("Set Bg3 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}", 
                //     (ppu_regs.bg3_vram_addr.get() as u16) << 10, 
                //     ppu_regs.bg3_tilemap_count_x.get(), 
                //     ppu_regs.bg3_tilemap_count_y.get()
                // );
            }

            0x210A => {
                ppu_regs.bg4_vram_addr = data >> 2;
                ppu_regs.bg4_tilemap_count_y = if get_bit_n!(data, 1) { TilemapCount::Two } else { TilemapCount::One };
                ppu_regs.bg4_tilemap_count_x = if get_bit_n!(data, 0) { TilemapCount::Two } else { TilemapCount::One };

                // println!("Set Bg4 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}", 
                //     (ppu_regs.bg4_vram_addr.get() as u16) << 10, 
                //     ppu_regs.bg4_tilemap_count_x.get(), 
                //     ppu_regs.bg4_tilemap_count_y.get()
                // );
            }

            0x210B => {
                ppu_regs.bg2_chr_base_addr = data >> 4;
                ppu_regs.bg1_chr_base_addr = data & 0x0F;

                // println!("Set Bg1 chr base address to ${:04X}", (ppu_regs.bg1_chr_base_addr.get() as u16) << 12);
                // println!("Set Bg2 chr base address to ${:04X}", (ppu_regs.bg2_chr_base_addr.get() as u16) << 12);
            }

            0x210C => {
                ppu_regs.bg4_chr_base_addr = data >> 4;
                ppu_regs.bg3_chr_base_addr = data & 0x0F;

                // println!("Set Bg3 chr base address to ${:04X}", (ppu_regs.bg3_chr_base_addr.get() as u16) << 12);
                // println!("Set Bg4 chr base address to ${:04X}", (ppu_regs.bg4_chr_base_addr.get() as u16) << 12);
            }

            0x210D => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                let bghofs_latch = ppu_regs.bg_offset_x_latch as u16;
                ppu_regs.bg_offset_latch = data;
                ppu_regs.bg_offset_x_latch = data;

                ppu_regs.bg1_m7_x_offset = (((data & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
            }

            0x210E => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                ppu_regs.bg_offset_latch = data;

                ppu_regs.bg1_m7_y_offset = (((data & 3) as u16) << 8) | bgofs_latch;
            }

            0x210F => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                let bghofs_latch = ppu_regs.bg_offset_x_latch as u16;
                ppu_regs.bg_offset_latch = data;
                ppu_regs.bg_offset_x_latch = data;

                ppu_regs.bg2_x_offset = (((data & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
            }

            0x2110 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                ppu_regs.bg_offset_latch = data;

                ppu_regs.bg2_y_offset = (((data & 3) as u16) << 8) | bgofs_latch;
            }

            0x2111 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                let bghofs_latch = ppu_regs.bg_offset_x_latch as u16;
                ppu_regs.bg_offset_latch = data;
                ppu_regs.bg_offset_x_latch = data;

                ppu_regs.bg3_x_offset = (((data & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
            }

            0x2112 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                ppu_regs.bg_offset_latch = data;

                ppu_regs.bg3_y_offset = (((data & 3) as u16) << 8) | bgofs_latch;
            }

            0x2113 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                let bghofs_latch = ppu_regs.bg_offset_x_latch as u16;
                ppu_regs.bg_offset_latch = data;
                ppu_regs.bg_offset_x_latch = data;

                ppu_regs.bg4_x_offset = (((data & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
            }

            0x2114 => {
                let bgofs_latch = ppu_regs.bg_offset_latch as u16;
                ppu_regs.bg_offset_latch = data;

                ppu_regs.bg4_y_offset = (((data & 3) as u16) << 8) | bgofs_latch;
            }

            0x2115 => {
                ppu_regs.vram_addr_inc_mode = if get_bit_n!(data, 7) { VramIncMode::HighByte } else { VramIncMode::LowByte };
                ppu_regs.addr_remap_mode = match (data >> 2) & 3 {
                    0 => AddressRemapping::None,
                    1 => AddressRemapping::ColDepth2,
                    2 => AddressRemapping::ColDepth4,
                    3 => AddressRemapping::ColDepth8,
                    _ => unreachable!(),
                };
                ppu_regs.addr_inc_size = match data & 3 {
                    0 => IncrSize::Bytes2,
                    1 => IncrSize::Bytes64,
                    2 => IncrSize::Bytes256,
                    3 => IncrSize::Bytes256,
                    _ => unreachable!(),
                };
            }

            0x2116 => {
                set_byte_n!(ppu_regs.vram_addr, data as u16, 0);
                ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];

                // println!("Set vram addr (lo) to ${:04X}", ppu_regs.vram_addr.get());
            }

            0x2117 => {
                set_byte_n!(ppu_regs.vram_addr, data as u16, 1);
                ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];

                // println!("Set vram addr (hi) to ${:04X}", ppu_regs.vram_addr.get());
            }

            0x2118 => {
                if ppu_regs.in_fblank || ppu_regs.in_vblank {
                    set_byte_n!(self.vram[ppu_regs.get_vram_addr() as usize], data as u16, 0);
                }

                match ppu_regs.vram_addr_inc_mode {
                    VramIncMode::LowByte => ppu_regs.inc_vram_addr(),
                    _ => {}
                }
            }

            0x2119 => {
                if ppu_regs.in_fblank || ppu_regs.in_vblank {
                    set_byte_n!(self.vram[ppu_regs.get_vram_addr() as usize], data as u16, 1);
                }

                match ppu_regs.vram_addr_inc_mode {
                    VramIncMode::HighByte => ppu_regs.inc_vram_addr(),
                    _ => {}
                }
            }

            0x211A => {
                ppu_regs.m7_tilemap_repeat = get_bit_n!(data, 7);
                ppu_regs.m7_fill_mode = if get_bit_n!(data, 6) { M7FillMode::Character } else { M7FillMode::Transparent };
                ppu_regs.m7_flip_bg_y = get_bit_n!(data, 1);
                ppu_regs.m7_flip_bg_x = get_bit_n!(data, 0);
            }

            0x211B => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = data;

                ppu_regs.m7_matrix_a = ((data as u16) << 8) | latched_val;
                ppu_regs.mult_factor_16 = ((data as u16) << 8) | latched_val;

                ppu_regs.update_multiply_result();
            }

            0x211C => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = data;

                ppu_regs.m7_matrix_b = ((data as u16) << 8) | latched_val;
                ppu_regs.mult_factor_8 = latched_val as u8;

                ppu_regs.update_multiply_result();
            }

            0x211D => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = data;

                ppu_regs.m7_matrix_c = ((data as u16) << 8) | latched_val;
            }

            0x211E => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = data;

                ppu_regs.m7_matrix_d = ((data as u16) << 8) | latched_val;
            }

            0x211F => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = data;

                ppu_regs.m7_center_x = ((data as u16) << 8) | latched_val;
            }

            0x2120 => {
                let latched_val = ppu_regs.m7_latch as u16;
                ppu_regs.m7_latch = data;

                ppu_regs.m7_center_y = ((data as u16) << 8) | latched_val;
            }

            0x2121 => {
                ppu_regs.cgram_addr = data;
                ppu_regs.cgram_toggle = false;
            }

            0x2122 => {
                ppu_regs.cgram_toggle = !ppu_regs.cgram_toggle;
                
                if ppu_regs.cgram_toggle {
                    ppu_regs.cgram_latch = data;
                } else {
                    let new_col = u16::from_le_bytes([ppu_regs.cgram_latch, data]);

                    self.cgram[ppu_regs.cgram_addr as usize] = Color::from_bgr555(new_col);

                    ppu_regs.cgram_addr += 1;
                }
            }

            0x2123 => {
                ppu_regs.bg2_w2_enabled = get_bit_n!(data, 7);
                ppu_regs.bg2_w2_inverted = get_bit_n!(data, 6);
                ppu_regs.bg2_w1_enabled = get_bit_n!(data, 5);
                ppu_regs.bg2_w1_inverted = get_bit_n!(data, 4);
                ppu_regs.bg1_w2_enabled = get_bit_n!(data, 3);
                ppu_regs.bg1_w2_inverted = get_bit_n!(data, 2);
                ppu_regs.bg1_w1_enabled = get_bit_n!(data, 1);
                ppu_regs.bg1_w1_inverted = get_bit_n!(data, 0);
            }

            0x2124 => {
                ppu_regs.bg4_w2_enabled = get_bit_n!(data, 7);
                ppu_regs.bg4_w2_inverted = get_bit_n!(data, 6);
                ppu_regs.bg4_w1_enabled = get_bit_n!(data, 5);
                ppu_regs.bg4_w1_inverted = get_bit_n!(data, 4);
                ppu_regs.bg3_w2_enabled = get_bit_n!(data, 3);
                ppu_regs.bg3_w2_inverted = get_bit_n!(data, 2);
                ppu_regs.bg3_w1_enabled = get_bit_n!(data, 1);
                ppu_regs.bg3_w1_inverted = get_bit_n!(data, 0);
            }

            0x2125 => {
                ppu_regs.col_w2_enabled = get_bit_n!(data, 7);
                ppu_regs.col_w2_inverted = get_bit_n!(data, 6);
                ppu_regs.col_w1_enabled = get_bit_n!(data, 5);
                ppu_regs.col_w1_inverted = get_bit_n!(data, 4);
                ppu_regs.obj_w2_enabled = get_bit_n!(data, 3);
                ppu_regs.obj_w2_inverted = get_bit_n!(data, 2);
                ppu_regs.obj_w1_enabled = get_bit_n!(data, 1);
                ppu_regs.obj_w1_inverted = get_bit_n!(data, 0);
            }

            0x2126 => { ppu_regs.w1_left_pos = data; }
            0x2127 => { ppu_regs.w1_right_pos = data; }
            0x2128 => { ppu_regs.w2_left_pos = data; }
            0x2129 => { ppu_regs.w2_right_pos = data; }

            0x212A => {
                ppu_regs.bg4_win_logic = match data >> 6 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
                ppu_regs.bg3_win_logic = match (data >> 4) & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
                ppu_regs.bg2_win_logic = match (data >> 2) & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
                ppu_regs.bg1_win_logic = match data & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
            }

            0x212B => {
                ppu_regs.col_win_logic = match (data >> 2) & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
                ppu_regs.obj_win_logic = match data & 3 {
                    0 => WindowLogic::Or,
                    1 => WindowLogic::And,
                    2 => WindowLogic::Xor,
                    3 => WindowLogic::Xnor,
                    _ => unreachable!(),
                };
            }

            0x212C => {
                ppu_regs.obj_main_enabled = get_bit_n!(data, 4);
                ppu_regs.bg4_main_enabled = get_bit_n!(data, 3);
                ppu_regs.bg3_main_enabled = get_bit_n!(data, 2);
                ppu_regs.bg2_main_enabled = get_bit_n!(data, 1);
                ppu_regs.bg1_main_enabled = get_bit_n!(data, 0);

                // println!("Set main en flags to Bg1: {}, Bg2: {}, Bg3: {}, Bg4: {}, Obj: {}",
                //     ppu_regs.bg1_main_enabled.get(),
                //     ppu_regs.bg2_main_enabled.get(),
                //     ppu_regs.bg3_main_enabled.get(),
                //     ppu_regs.bg4_main_enabled.get(),
                //     ppu_regs.obj_main_enabled.get(),
                // );
            }

            0x212D => {
                ppu_regs.obj_sub_enabled = get_bit_n!(data, 4);
                ppu_regs.bg4_sub_enabled = get_bit_n!(data, 3);
                ppu_regs.bg3_sub_enabled = get_bit_n!(data, 2);
                ppu_regs.bg2_sub_enabled = get_bit_n!(data, 1);
                ppu_regs.bg1_sub_enabled = get_bit_n!(data, 0);
            }

            0x212E => {
                ppu_regs.obj_win_main_enabled = get_bit_n!(data, 4);
                ppu_regs.bg4_win_main_enabled = get_bit_n!(data, 3);
                ppu_regs.bg3_win_main_enabled = get_bit_n!(data, 2);
                ppu_regs.bg2_win_main_enabled = get_bit_n!(data, 1);
                ppu_regs.bg1_win_main_enabled = get_bit_n!(data, 0);
            }

            0x212F => {
                ppu_regs.obj_win_sub_enabled = get_bit_n!(data, 4);
                ppu_regs.bg4_win_sub_enabled = get_bit_n!(data, 3);
                ppu_regs.bg3_win_sub_enabled = get_bit_n!(data, 2);
                ppu_regs.bg2_win_sub_enabled = get_bit_n!(data, 1);
                ppu_regs.bg1_win_sub_enabled = get_bit_n!(data, 0);
            }

            0x2130 => {
                ppu_regs.col_win_main_region = match data >> 6 {
                    0 => WindowColorRegion::Nowhere,
                    1 => WindowColorRegion::Outside,
                    2 => WindowColorRegion::Inside,
                    3 => WindowColorRegion::Everywhere,
                    _ => unreachable!(),
                };
                ppu_regs.col_win_sub_region = match (data >> 4) & 3 {
                    0 => WindowColorRegion::Nowhere,
                    1 => WindowColorRegion::Outside,
                    2 => WindowColorRegion::Inside,
                    3 => WindowColorRegion::Everywhere,
                    _ => unreachable!(),
                };
                ppu_regs.sub_color_fixed = !get_bit_n!(data, 1);
                ppu_regs.use_direct_col = get_bit_n!(data, 0);
            }

            0x2131 => {
                ppu_regs.cmath_operator = if get_bit_n!(data, 7) { CMathOperator::Subtract } else { CMathOperator::Add };
                ppu_regs.cmath_half = get_bit_n!(data, 6);
                ppu_regs.back_cmath_enabled = get_bit_n!(data, 5);
                ppu_regs.obj_cmath_enabled = get_bit_n!(data, 4);
                ppu_regs.bg4_cmath_enabled = get_bit_n!(data, 3);
                ppu_regs.bg3_cmath_enabled = get_bit_n!(data, 2);
                ppu_regs.bg2_cmath_enabled = get_bit_n!(data, 1);
                ppu_regs.bg1_cmath_enabled = get_bit_n!(data, 0);
            }

            0x2132 => {
                let new_val = (data & 0x1F) as u16;
                
                if get_bit_n!(data, 7) {
                    ppu_regs.fixed_color &= 0x03FF;
                    ppu_regs.fixed_color |= new_val << 10;
                };
                if get_bit_n!(data, 6) {
                    ppu_regs.fixed_color &= 0x7C1F;
                    ppu_regs.fixed_color |= new_val << 5;
                };
                if get_bit_n!(data, 5) {
                    ppu_regs.fixed_color &= 0x7FE0;
                    ppu_regs.fixed_color |= new_val;
                };
            }

            0x2133 => {
                ppu_regs._external_sync = get_bit_n!(data, 7);
                ppu_regs.ext_bg_enabled = get_bit_n!(data, 6);
                ppu_regs.hi_res_enabled = get_bit_n!(data, 3);
                ppu_regs.overscan_enabled = get_bit_n!(data, 2);
                ppu_regs.obj_interlace_enabled = get_bit_n!(data, 1);
                ppu_regs.screen_interlace_enabled = get_bit_n!(data, 0);
            }

            _ => {}
        }
    }
}