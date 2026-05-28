use crate::dma::{
    AddressIncMode,
    TransferPattern,
    Direction,
};
use crate::scpu::Address;

/// A single DMA/H-DMA channel
#[derive(Default, Clone, Copy)]
pub struct DmaRegs {
    // $420B
    pub dma_en: bool,

    // $420C
    pub hdma_en: bool,

    // $43n0
    pub params_raw: u8,
    pub direction: Direction,
    pub indirect_hdma: bool,
    pub inc_mode: AddressIncMode,
    pub transfer_pattern: TransferPattern,
    pub transfer_pattern_step: u8,

    // $43n1
    pub b_bus_addr: u8,

    // $43n2..=$43n4
    pub a_bus_addr: Address,

    // $43n5..=$43n7
    pub hdma_indirect_table_addr: Address,

    // $43n8..=$43n9
    pub hdma_table_offset: u16,

    // $43nA
    pub hdma_repeat_flag: bool,
    pub entry_scanline_count: u8, // Initial loaded scanline count for an HDMA entry
    pub scanlines_left: u8, // Current number of scanlines left until next HDMA entry

    // $43nB and $43nF
    pub unused: u8,

    pub hdma_entry_just_loaded: bool, // Whether an HDMA entry was just loaded this cycle, used to determine when to decrement scanlines_left
    pub hdma_initialized: bool,
    pub hdma_do_transfer: bool, // Set on entry load, cleared after first transfer for non-repeat entries
}

impl DmaRegs {
    pub fn power_on(&mut self) {
        self.dma_en = false;
        self.hdma_en = false;
        self.params_raw = 0xFF;
        self.direction = Direction::BtoA;
        self.indirect_hdma = true;
        self.inc_mode = AddressIncMode::Fixed;
        self.transfer_pattern = TransferPattern::Pattern7;
        self.transfer_pattern_step = 0;
        self.b_bus_addr = 0xFF;
        self.unused = 0xFF;
        self.a_bus_addr = Address { bank: 0xFF, offset: 0xFFFF };
        self.hdma_indirect_table_addr = Address { bank: 0xFF, offset: 0xFFFF };
        self.hdma_table_offset = 0xFFFF;
        self.entry_scanline_count = 0x7F;
        self.scanlines_left = 0x7F;
        self.hdma_repeat_flag = true;
    }
    
    pub fn reset(&mut self) {
        self.dma_en = false;
        self.hdma_en = false;
    }
    
    // Returns the actual B bus address we read/write based on the base B address
    // and the transfer pattern. Since the B bus is made of various kinds of registers,
    // each of which are written to/read from differently, these modes are designed
    // to interface with those registers. Ex: writing to VRAM involves writing two
    // bytes to two adjacent addresses over and over, so Pattern1 would be used.
    pub fn get_b_with_offset(&self) -> Address {
        let step = self.transfer_pattern_step;

        let low_byte = match self.transfer_pattern {
            TransferPattern::Pattern0 | TransferPattern::Pattern2 | TransferPattern::Pattern6 => {
                self.b_bus_addr
            }

            TransferPattern::Pattern1 | TransferPattern::Pattern5 => {
                self.b_bus_addr + (step & 1)
            }

            TransferPattern::Pattern3 | TransferPattern::Pattern7 => {
                if (step & 3) < 2 {
                    self.b_bus_addr
                } else {
                    self.b_bus_addr + (step & 1)
                }
            }

            TransferPattern::Pattern4 => {
                self.b_bus_addr + (step & 3)
            }
        };
        
        Address {
            bank: 0,
            offset: 0x2100 | low_byte as u16,
        }
    }

    pub fn inc_a_bus_addr(&mut self) {
        match self.inc_mode {
            AddressIncMode::Inc => {
                self.a_bus_addr.offset += 1;
            }
            AddressIncMode::Dec => {
                self.a_bus_addr.offset -= 1;
            }
            AddressIncMode::Fixed => {}
        };
    }

    // Fetch and auto-increment the A-bus address, taking into account direct/indirect mode
    pub fn hdma_get_a_bus_addr(&mut self) -> Address {
        if self.indirect_hdma {
            let a = self.hdma_indirect_table_addr;
            self.hdma_indirect_table_addr.offset += 1;
            a
        } else {
            let a = Address {
                bank: self.a_bus_addr.bank,
                offset: self.hdma_table_offset,
            };
            self.hdma_table_offset += 1;
            a
        }
    }
}