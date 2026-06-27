pub struct CpuSnapshot {
    pub a: u16,  // Accumulator
    pub x: u16,  // X index
    pub y: u16,  // Y index
    pub sp: u16, // Stack pointer
    pub pc: u16, // Program counter
    pub pb: u8,  // Program bank
    pub db: u8,  // Data bank
    pub dp: u16, // Direct page
    pub p: u8,   // Processor status
    pub e: bool, // Emulation mode

    pub halted: bool,
    pub stopped: bool,
    pub waiting_for_interrupt: bool,
    pub irq_pending: bool,
    pub nmi_pending: bool,
}

pub struct DmaStartSnapshot {
    
}

pub struct DmaTransferSnapshot {
    
}

pub struct DmaChannelSnapshot {
    pub dma_en: bool,
    pub hdma_en: bool,
    pub direction: Direction,
    pub indirect_hdma: bool,
    pub inc_mode: AddressIncMode,
    pub transfer_pattern: TransferPattern,
    pub b_bus_addr: u8,
    pub a_bus_addr: Address,
    pub hdma_indirect_table_addr: Address, // Also DMA byte count
    pub hdma_table_offset: u16,
    pub hdma_repeat_flag: bool,
    pub entry_scanline_count: u8, // Initial loaded scanline count for an HDMA entry
    pub scanlines_left: u8, // Current number of scanlines left until next HDMA entry
    pub unused: u8,
}

pub enum CoreEvent {
    CpuInstruction { cpu: CpuSnapshot, prg_bytes: Vec<u8>, },
    CpuMemoryRead { cpu: CpuSnapshot, addr: Address, value: u8 },
    CpuMemoryWrite { cpu: CpuSnapshot, addr: Address, value: u8 },
    CpuInterrupt { cpu: CpuSnapshot, kind: CpuInterrupt },

    DmaStart {
        channel: u8,
        is_hdma: bool,
        direction: Direction,
        inc_mode: AddressIncMode,
        transfer_pattern: TransferPattern,
        byte_count: usize,
    },
    DmaTransfer {
        channel: u8,
        src_addr: Address,
        dst_addr: Address,
        value: u8,
        byte_num: usize,
    },
    DmaEnd {
        channel: u8,
        is_hdma: bool,
        byte_count: usize,
    }
}