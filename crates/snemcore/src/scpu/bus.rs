use crate::cartridge::Cartridge;
use crate::controller::{ControllerData, JoypadCmd};
use crate::dma::DmaController;
use crate::dma::{AddressIncMode, Direction, TransferPattern};
use crate::scpu::ioregs::CpuIoRegs;
use crate::sppu::color::Color;
use crate::sppu::regs::PpuRegs;
use crate::sppu::{MasterSlave, VideoType, VramIncMode};
use crate::ssmp::ioports::ApuIoPorts;
use crate::sysinfo::{CGRAM_SIZE, OAM_SIZE, VRAM_SIZE, WRAM_SIZE};
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

    pub cart: &'a mut Cartridge,
    pub dma: Option<&'a mut DmaController>,

    pub controller_data: &'a mut ControllerData,
}

impl<'a> CpuBus<'a> {
    pub fn read(&mut self, addr: Address) -> u8 {
        let value = match addr.bank {
            // Banks $00-$3F: LoROM mapping
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                // WRAM mirror (first 8KB)
                0x0000..=0x1FFF => self.wram[addr.offset as usize],

                // PPU registers
                0x2100..=0x213F => self.read_ppu_regs(addr.offset),

                // APU ports
                0x2140..=0x217F => match addr.offset & 0x3 {
                    0 => self.apu_ports.apuio0,
                    1 => self.apu_ports.apuio1,
                    2 => self.apu_ports.apuio2,
                    3 => self.apu_ports.apuio3,
                    _ => unreachable!(),
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
        };

        value
    }

    pub fn write(&mut self, addr: Address, value: u8) {
        match addr.bank {
            0x00..=0x3F | 0x80..=0xBF => match addr.offset {
                // WRAM mirror
                0x0000..=0x1FFF => self.wram[addr.offset as usize] = value,

                // PPU registers
                0x2100..=0x213F => self.write_ppu_regs(addr.offset, value),

                // APU ports
                0x2140..=0x217F => match addr.offset & 3 {
                    0 => self.apu_ports.cpuio0 = value,
                    1 => self.apu_ports.cpuio1 = value,
                    2 => self.apu_ports.cpuio2 = value,
                    3 => self.apu_ports.cpuio3 = value,
                    _ => unreachable!(),
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

    fn read_wram_port(&mut self, _offset: u16) -> u8 {
        log::warn!("Attempting to read from WRAM port");
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
        log::warn!(
            "Attempting to write to WRAM ports: {:02X} to ${:04X}",
            value,
            offset
        );
        // uncomment and pray it works
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
                ppu_regs.h_counter_latch = ppu_regs.h_counter;
                ppu_regs.v_counter_latch = ppu_regs.v_counter;

                ppu_regs.counter_toggle = true;

                0 // CPU OPEN BUS
            }

            0x2138 => {
                let data = self.oam[ppu_regs.internal_oam_addr as usize];
                
                ppu_regs.internal_oam_addr += 1;
                ppu_regs.internal_oam_addr %= OAM_SIZE as u16;

                data
            }

            0x2139 => {
                let val = get_byte_n!(ppu_regs.vram_latch, 0);

                if matches!(ppu_regs.vram_addr_inc_mode, VramIncMode::LowByte) {
                    ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];
                    ppu_regs.inc_vram_addr();
                }

                val
            }

            0x213A => {
                let val = get_byte_n!(ppu_regs.vram_latch, 1);

                if matches!(ppu_regs.vram_addr_inc_mode, VramIncMode::HighByte) {
                    ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];
                    ppu_regs.inc_vram_addr();
                }

                val
            }

            0x213B => {
                let data_word = self.cgram[ppu_regs.cgram_addr as usize].to_bgr555();

                let data = if ppu_regs.cgram_toggle {
                    get_byte_n!(data_word, 0)
                } else {
                    ppu_regs.cgram_addr += 1;
                    get_byte_n!(data_word, 1)
                };
                
                ppu_regs.cgram_toggle = !ppu_regs.cgram_toggle;
            
                data
            }

            0x213C => {
                let data = if ppu_regs.h_counter_toggle {
                    get_byte_n!(ppu_regs.h_counter_latch, 0)
                } else {
                    get_byte_n!(ppu_regs.h_counter_latch, 1) // HIGH 7 BITS ARE PPU2 OPEN BUS
                };
                
                ppu_regs.h_counter_toggle = !ppu_regs.h_counter_toggle;
            
                data
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
                    self.oam[0x200 | internal_oam_addr & 0x1F] = value;
                }

                ppu_regs.internal_oam_addr += 1;
                ppu_regs.internal_oam_addr &= 0x1FF;

                if ppu_regs.internal_oam_addr == 0 {
                    ppu_regs.oam_write_high_table = !ppu_regs.oam_write_high_table;
                }
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
                if ppu_regs.in_fblank || self.cpu_regs.vblank_flag {
                    set_byte_n!(self.vram[ppu_regs.get_vram_addr() as usize], value as u16, 0);
                    ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];
                }
                
                if matches!(ppu_regs.vram_addr_inc_mode, VramIncMode::LowByte) {
                    ppu_regs.inc_vram_addr();
                }
            }

            0x2119 => {
                if ppu_regs.in_fblank || self.cpu_regs.vblank_flag {
                    set_byte_n!(self.vram[ppu_regs.get_vram_addr() as usize], value as u16, 1);
                    ppu_regs.vram_latch = self.vram[ppu_regs.get_vram_addr() as usize];
                }
                
                if matches!(ppu_regs.vram_addr_inc_mode, VramIncMode::HighByte) {
                    ppu_regs.inc_vram_addr();
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

    fn read_cpuio_regs(&mut self, offset: u16) -> u8 {
        let cpu_regs = &mut self.cpu_regs;

        match offset {
            0x4000..=0x4015 => 0, // Open bus

            0x4016 => {
                self.controller_data.joypad_cmd = Some(JoypadCmd::ClockJoy1);

                let joy1_data1 = (self.controller_data.joy1_latch & 1) as u8;
                let joy1_data2 = 0x00; // unused for joypads

                joy1_data2 | joy1_data1
            }

            0x4017 => {
                const ALWAYS_ON: u8 = 0x1C;

                self.controller_data.joypad_cmd = Some(JoypadCmd::ClockJoy2);

                let joy2_data1 = (self.controller_data.joy2_latch & 1) as u8;
                let joy2_data2 = 0x00; // unused for joypads

                ALWAYS_ON | joy2_data2 | joy2_data1
            }

            0x4018..=0x41FF => 0, // Open bus

            0x4200..=0x420D => 0, // Write-only registers

            0x420E..=0x420F => 0, // Open bus

            0x4210 => {
                let vblank_nmi = if cpu_regs.vblank_flag { 0x80 } else { 0 };
                let cpu_version = 0x02;

                cpu_regs.vblank_nmi_flag = false;

                vblank_nmi | cpu_version
            }

            0x4211 => {
                let timer_irq = if cpu_regs.hv_timer_irq_flag { 0x80 } else { 0 };

                cpu_regs.hv_timer_irq_flag = false;

                timer_irq
            }

            0x4212 => {
                let in_vblank = if cpu_regs.vblank_flag { 0x80 } else { 0 };
                let in_hblank = if cpu_regs.hblank_flag { 0x40 } else { 0 };
                let in_joypad_autoread = if cpu_regs.joypad_autoread_flag { 1 } else { 0 };

                in_vblank | in_hblank | in_joypad_autoread
            }

            0x4213 => cpu_regs.raw_rdwrio,

            0x4214 => get_byte_n!(cpu_regs.div_quotient, 0),
            0x4215 => get_byte_n!(cpu_regs.div_quotient, 1),

            0x4216 => get_byte_n!(cpu_regs.mult_result, 0),
            0x4217 => get_byte_n!(cpu_regs.mult_result, 1),

            0x4218 => get_byte_n!(self.controller_data.joy1_data1_auto, 0),
            0x4219 => get_byte_n!(self.controller_data.joy1_data1_auto, 1),
            0x421A => get_byte_n!(self.controller_data.joy2_data1_auto, 0),
            0x421B => get_byte_n!(self.controller_data.joy2_data1_auto, 1),
            0x421C => get_byte_n!(self.controller_data.joy1_data2_auto, 0),
            0x421D => get_byte_n!(self.controller_data.joy1_data2_auto, 1),
            0x421E => get_byte_n!(self.controller_data.joy2_data2_auto, 0),
            0x421F => get_byte_n!(self.controller_data.joy2_data2_auto, 1),

            _ => 0,
        }
    }

    fn write_cpuio_regs(&mut self, offset: u16, value: u8) {
        let cpu_regs = &mut self.cpu_regs;

        match offset {
            0x4000..=0x4015 => {} // Open bus

            0x4016 => {
                self.cpu_regs.latch_controllers = get_bit_n!(value, 0);
            }

            0x4017 => {} // Write-only register

            0x4018..=0x41FF => {} // Open bus

            0x4200 => {
                cpu_regs.write_4200(value);
            }

            0x4201 => {
                cpu_regs.write_4201(value);
            }

            0x4202 => {
                cpu_regs.write_4202(value);
            }
            0x4203 => {
                cpu_regs.write_4203(value);
            }
            0x4204 => {
                cpu_regs.write_4204(value);
            }
            0x4205 => {
                cpu_regs.write_4205(value);
            }
            0x4206 => {
                cpu_regs.write_4206(value);
            }
            0x4207 => {
                cpu_regs.write_4207(value);
            }
            0x4208 => {
                cpu_regs.write_4208(value);
            }
            0x4209 => {
                cpu_regs.write_4209(value);
            }
            0x420A => {
                cpu_regs.write_420A(value);
            }

            0x420B => {
                if let Some(ref mut dma) = self.dma {
                    dma.write_420B(value);
                }
            }

            0x420C => {
                if let Some(ref mut dma) = self.dma {
                    dma.write_420C(value);
                }
            }

            0x4210..=0x42FF => {} // Read-only regs

            _ => {}
        }
    }

    fn read_dma_regs(&mut self, offset: u16) -> u8 {
        if self.dma.is_none() {
            return 0;
        }

        let channel_idx = ((offset >> 4) & 0xF) as usize;

        if channel_idx >= 7 {
            return 0; // TODO: Maybe mirror channel_idx & 7?
        }

        let channel = &mut self.dma.as_mut().unwrap().regs[channel_idx];

        match offset & 0xF {
            0x0 => channel.params_raw,
            0x1 => channel.b_bus_addr,
            0x2 => get_byte_n!(channel.a_bus_addr.offset, 0),
            0x3 => get_byte_n!(channel.a_bus_addr.offset, 1),
            0x4 => channel.a_bus_addr.bank,
            0x5 => get_byte_n!(channel.hdma_indirect_table_addr.offset, 0),
            0x6 => get_byte_n!(channel.hdma_indirect_table_addr.offset, 1),
            0x7 => channel.hdma_indirect_table_addr.bank,
            0x8 => get_byte_n!(channel.hdma_table_offset, 0),
            0x9 => get_byte_n!(channel.hdma_table_offset, 1),
            0xA => {
                let hdma_reload = if channel.hdma_repeat_flag { 0x80 } else { 0 };
                hdma_reload | channel.entry_scanline_count
            }
            0xB => channel.unused,
            0xC..=0xE => 0, // Open bus
            0xF => channel.unused,
            _ => unreachable!(),
        }
    }

    fn write_dma_regs(&mut self, offset: u16, value: u8) {
        if self.dma.is_none() {
            return;
        }

        let channel_idx = ((offset >> 4) & 0xF) as usize;

        if channel_idx > 7 {
            return; // TODO: Maybe mirror channel_idx & 7?
        }

        let channel = &mut self.dma.as_mut().unwrap().regs[channel_idx];

        match offset & 0xF {
            0x0 => {
                channel.params_raw = value;

                channel.direction = match get_bit_n!(value, 7) {
                    false => Direction::AtoB,
                    true => Direction::BtoA,
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
            }

            0x1 => {
                channel.b_bus_addr = value;
            }
            0x2 => {
                set_byte_n!(channel.a_bus_addr.offset, value as u16, 0);
            }
            0x3 => {
                set_byte_n!(channel.a_bus_addr.offset, value as u16, 1);
            }
            0x4 => {
                channel.a_bus_addr.bank = value;
            }
            0x5 => {
                set_byte_n!(channel.hdma_indirect_table_addr.offset, value as u16, 0);
            }
            0x6 => {
                set_byte_n!(channel.hdma_indirect_table_addr.offset, value as u16, 1);
            }
            0x7 => {
                channel.hdma_indirect_table_addr.bank = value;
            }
            0x8 => {
                set_byte_n!(channel.hdma_table_offset, value as u16, 0);
            }
            0x9 => {
                set_byte_n!(channel.hdma_table_offset, value as u16, 1);
            }
            0xA => {
                channel.hdma_repeat_flag = get_bit_n!(value, 7);
                channel.entry_scanline_count = value & 0x7F;
                channel.scanlines_left = value & 0x7F;
            }
            0xB => {
                channel.unused = value;
            }
            0xC..=0xE => {} // Open bus
            0xF => {
                channel.unused = value;
            }
            _ => unreachable!(),
        }
    }
}
