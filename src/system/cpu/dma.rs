#[derive(Default, Clone)]
pub enum Direction {
    #[default]
    AtoB,
    BtoA,
}

#[derive(Default, Clone)]
pub enum AddressIncMode {
    #[default]
    Inc,
    Fixed,
    Dec,
}

#[derive(Default, Clone)]
pub enum TransferPattern {
    #[default]
    Pattern0,
    Pattern1,
    Pattern2,
    Pattern3,
    Pattern4,
    Pattern5,
    Pattern6,
    Pattern7,
}

#[derive(Default, Clone)]
pub struct DmaChannel {
    pub active: bool,
    pub direction: Direction,
    pub indirect: bool,
    pub inc_mode: AddressIncMode,
    pub transfer_pattern: TransferPattern,
    pub b_bus_addr: u8,
    pub src_addr: u32,
    pub dma_byte_count: u32,
    pub hdma_table_addr: u16,
    pub hdma_reload: bool,
    pub scanline: u8,
    pub unused: u8, // don't know if we actually need this, but it's one byte sooo...
    pub bytes_written: usize, // needed to keep track of B bus increment patterns
}

impl DmaChannel {
    // Returns the actual B bus address we read/write based on the base B address
    // and the transfer pattern. Since the B bus is made of various kinds of registers,
    // each of which are written to/read from differently, these modes are designed
    // to interface with those registers. Ex: writing to VRAM involves writing two
    // bytes to two adjacent addresses over and over, so Pattern1 would be used.
    fn get_b_with_offset(&self) -> u8 {
        let truncated_bw = (self.bytes_written % 256) as u8; // type systems suck
        match self.transfer_pattern {
            TransferPattern::Pattern0 => self.b_bus_addr,
            TransferPattern::Pattern1 => self.b_bus_addr + truncated_bw % 2,
            TransferPattern::Pattern2 => self.b_bus_addr,
            TransferPattern::Pattern3 => {
                if truncated_bw % 4 < 2 {
                    self.b_bus_addr
                } else {
                    self.b_bus_addr + 1
                }
            }
            TransferPattern::Pattern4 => self.b_bus_addr + truncated_bw % 4,
            TransferPattern::Pattern5 => {
                self.b_bus_addr + truncated_bw % 2 // Undocumented mode
            }
            TransferPattern::Pattern6 => {
                self.b_bus_addr // Same as 2, undocumented
            }
            TransferPattern::Pattern7 => {
                // Same as 3, undocumented
                if truncated_bw % 4 < 2 {
                    self.b_bus_addr
                } else {
                    self.b_bus_addr + 1
                }
            }
        }
    }

    fn update_src_addr(&mut self) {
        match self.inc_mode {
            AddressIncMode::Fixed => _,
            AddressIncMode::Dec => {}
        }
    }
}
