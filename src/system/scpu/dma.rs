#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum DmaStatus {
    /// No enabled DMA or H-DMA channels
    Off,
    /// DMA in progress, no H-DMA channels enabled
    DMA,
    /// H-DMA in progress, no DMA channels enabled
    HDMA,
    /// H-DMA waiting for next hblank, no DMA channels enabled
    InactiveHDMA,
    /// H-DMA active, DMA waiting for H-DMA to finish
    ActiveLayeredHDMA,
    /// DMA active, H-DMA waiting for next hblank
    InactiveLayeredHDMA,
}

#[derive(Default, Clone, Debug)]
pub(super) enum Direction {
    #[default]
    AtoB,
    BtoA,
}

#[derive(Clone, Debug, Default)]
pub(super) enum AddressIncMode {
    #[default]
    Inc,
    Fixed,
    Dec,
}

#[derive(Clone, Debug, Default)]
pub(super) enum TransferPattern {
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
pub(super) struct DmaChannel {
    pub params_raw: u8,
    pub dma_enable: bool,
    pub hdma_enable: bool,
    pub table_started: bool,
    pub direction: Direction,
    pub indirect: bool,
    pub inc_mode: AddressIncMode,
    pub transfer_pattern: TransferPattern,
    pub a_bus_bank: u8,
    pub a_bus_hi: u8,
    pub a_bus_lo: u8,
    pub b_bus_addr: u8,
    pub byte_count: u16,
    pub hdma_indirect_table_bank: u8,
    pub hdma_table_start_addr: u32,
    pub hdma_table_addr: u16,
    pub hdma_reload: bool,
    pub scanlines_left: u8,
    pub unused: u8,
    pub repeat: bool,
    pub bytes_written: usize, // needed to keep track of B bus increment patterns
}

impl DmaChannel {
    // Returns the actual B bus address we read/write based on the base B address
    // and the transfer pattern. Since the B bus is made of various kinds of registers,
    // each of which are written to/read from differently, these modes are designed
    // to interface with those registers. Ex: writing to VRAM involves writing two
    // bytes to two adjacent addresses over and over, so Pattern1 would be used.
    pub(super) fn get_b_with_offset(&self) -> u8 {
        let truncated_bw = self.bytes_written as u8;

        match self.transfer_pattern {
            TransferPattern::Pattern0
            | TransferPattern::Pattern2
            | TransferPattern::Pattern6 => self.b_bus_addr,
            
            TransferPattern::Pattern1 
            | TransferPattern::Pattern5 => self.b_bus_addr + (truncated_bw & 1),
            
            TransferPattern::Pattern3
            | TransferPattern::Pattern7 => {
                if (truncated_bw & 3) < 2 {
                    self.b_bus_addr
                } else {
                    self.b_bus_addr + 1
                }
            },

            TransferPattern::Pattern4 => self.b_bus_addr + (truncated_bw & 3),
        }
    }

    pub(super) fn a_bus_addr(&self) -> u32 {
        ((self.a_bus_bank as u32) << 16) | ((self.a_bus_hi as u32) << 8) | (self.a_bus_lo as u32)
    }

    pub(super) fn inc_a_bus_addr(&mut self) {
        let new_addr = match self.inc_mode {
            AddressIncMode::Inc => self.a_bus_addr() + 1,
            AddressIncMode::Dec => self.a_bus_addr() - 1,
            AddressIncMode::Fixed => self.a_bus_addr(),
        };

        self.a_bus_lo = new_addr as u8;
        self.a_bus_hi = (new_addr >> 8) as u8;
    }
}