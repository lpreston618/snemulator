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