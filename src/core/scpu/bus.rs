use crate::core::sysinfo::{WRAM_SIZE, VRAM_SIZE, CGRAM_SIZE, OAM_SIZE};

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
    // pub vram: &'a mut [u8; VRAM_SIZE], // for writing to vram thru ppuregs
    // pub cgram: &'a mut [u8; CGRAM_SIZE],
    // pub oam: &'a mut [u8; OAM_SIZE],
    // pub ppu_regs: &'a mut PpuRegs,
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
                0x2100..=0x213F => self.ppu_regs.read(addr.offset),
                
                // APU ports
                0x2140..=0x217F => self.apu_ports.read_from_cpu(addr.offset & 0x3),
                
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
                0x2100..=0x213F => self.ppu_regs.write(addr.offset, value),
                
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
}