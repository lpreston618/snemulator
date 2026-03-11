use crate::core::scpu::bus::Address;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DmaStatus {
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

#[derive(Clone, Copy, Default, Debug)]
pub enum Direction {
    #[default]
    AtoB,
    BtoA,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AddressIncMode {
    #[default]
    Inc,
    Fixed,
    Dec,
}

#[derive(Clone, Copy, Debug, Default)]
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
    
    // $43n1
    pub b_bus_addr: u8,
    
    // $43n2..=$43n4
    pub dma_src_addr: Address,
    
    // $43n5..=$43n7
    pub hdma_indirect_table_addr: Address,
    
    // $43n8..=$43n9
    pub hdma_table_offset: u16,
   
    // $43nA
    pub hdma_reload_flag: bool,
    pub scanline_counter: u8,
    
    // $43nB and $43nF
    pub unused: u8,
     
    // pub byte_count: u16,
    // pub hdma_indirect_table_bank: u8,
    // pub dma_src_addr: u32,
    // pub hdma_table_addr: u16,
    // pub scanlines_left: u8,
    // pub unused: u8,
    // pub hdma_repeat: bool,
    // pub bytes_written: usize, // needed to keep track of B bus increment patterns
}

// impl DmaRegs {
//     // Returns the actual B bus address we read/write based on the base B address
//     // and the transfer pattern. Since the B bus is made of various kinds of registers,
//     // each of which are written to/read from differently, these modes are designed
//     // to interface with those registers. Ex: writing to VRAM involves writing two
//     // bytes to two adjacent addresses over and over, so Pattern1 would be used.
//     pub fn get_b_with_offset(&self) -> u8 {
//         let truncated_bw = self.bytes_written as u8;

//         match self.transfer_pattern {
//             TransferPattern::Pattern0
//             | TransferPattern::Pattern2
//             | TransferPattern::Pattern6 => self.b_bus_addr,
            
//             TransferPattern::Pattern1 
//             | TransferPattern::Pattern5 => self.b_bus_addr + (truncated_bw & 1),
            
//             TransferPattern::Pattern3
//             | TransferPattern::Pattern7 => {
//                 if (truncated_bw & 3) < 2 {
//                     self.b_bus_addr
//                 } else {
//                     self.b_bus_addr + 1
//                 }
//             },

//             TransferPattern::Pattern4 => self.b_bus_addr + (truncated_bw & 3),
//         }
//     }

//     pub fn a_bus_addr(&self) -> u32 {
//         ((self.a_bus_bank as u32) << 16) | ((self.a_bus_hi as u32) << 8) | (self.a_bus_lo as u32)
//     }

//     pub fn inc_a_bus_addr(&mut self) {
//         let new_addr = match self.inc_mode {
//             AddressIncMode::Inc => self.a_bus_addr() + 1,
//             AddressIncMode::Dec => self.a_bus_addr() - 1,
//             AddressIncMode::Fixed => self.a_bus_addr(),
//         };

//         self.a_bus_lo = new_addr as u8;
//         self.a_bus_hi = (new_addr >> 8) as u8;
//     }
// }