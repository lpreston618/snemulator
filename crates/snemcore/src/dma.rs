use crate::get_bit_n;
use crate::scpu::Address;
use crate::scpu::bus::CpuBus;

mod types;
mod regs;

pub use types::*;
use regs::DmaRegs;

pub struct DmaController {
    pub regs: [DmaRegs; 8],
    pub dma_en: bool,
    pub hdma_en: bool,
    pub hdma_pending: bool,
    pub hdma_needs_init: bool,
    pub dma_active_ch: usize,
    pub hdma_active_ch: usize,
}

impl DmaController {
    pub fn new() -> Self {
        Self {
            regs: [DmaRegs::default(); 8],
            dma_en: false,
            hdma_en: false,
            hdma_pending: false,
            hdma_needs_init: false,
            dma_active_ch: 8,
            hdma_active_ch: 8,
        }
    }

    pub fn power_on(&mut self) {
        for regs in self.regs.iter_mut() {
            regs.power_on();
        }
        self.reset_state();
    }

    pub fn reset(&mut self) {
        for regs in self.regs.iter_mut() {
            regs.reset();
        }
        self.reset_state();
    }

    fn reset_state(&mut self) {
        self.dma_en = false;
        self.hdma_en = false;
        self.hdma_pending = false;
        self.hdma_needs_init = true;
        self.dma_active_ch = 8;
        self.hdma_active_ch = 8;
    }

    #[allow(non_snake_case)]
    pub fn write_420B(&mut self, value: u8) {
        self.dma_en = value != 0;
        self.dma_active_ch = value.trailing_zeros() as usize;
        
        for i in 0..8 {
            self.regs[i].dma_en = get_bit_n!(value, i);

            if self.regs[i].dma_en {
                self.regs[i].transfer_pattern_step = 0;
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn write_420C(&mut self, value: u8) {
        for i in 0..8 {
            self.regs[i].hdma_en = get_bit_n!(value, i);
            self.regs[i].hdma_initialized = false; // Mark all as needing init
        }
        self.hdma_needs_init = true;
    }

    pub fn do_dma(&mut self, bus: &mut CpuBus, cpu_stopped: &mut bool) {
        let mut dma_ch_regs = &mut self.regs[self.dma_active_ch];

        // HDMA indirect table register is same as DMA byte count register
        let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;

        // Channel's DMA transfer complete
        if byte_count == 0 {
            dma_ch_regs.dma_en = false;
            self.dma_active_ch += 1;

            'seek_active_channel: while self.dma_active_ch < 8 {
                dma_ch_regs = &mut self.regs[self.dma_active_ch];

                let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;

                if dma_ch_regs.dma_en {
                    // Active channel found
                    if byte_count != 0 {
                        break 'seek_active_channel;
                    }

                    // Enabled channel has no bytes to transfer, disable it
                    dma_ch_regs.dma_en = false;
                }

                self.dma_active_ch += 1;
            }
        }

        // No DMA channels are enabled, disable DMA
        if self.dma_active_ch == 8 {
            self.dma_en = false;
            *cpu_stopped = false;
            return;
        }

        let dma_ch_regs = &mut self.regs[self.dma_active_ch]; // No longer mutable

        let a_bus_addr = dma_ch_regs.a_bus_addr;
        let b_bus_addr = dma_ch_regs.get_b_with_offset();

        let (src_addr, dst_addr) = match dma_ch_regs.direction {
            Direction::AtoB => (a_bus_addr, b_bus_addr),
            Direction::BtoA => (b_bus_addr, a_bus_addr),
        };

        dma_ch_regs.hdma_indirect_table_addr.offset -= 1; // byte_count -= 1
        dma_ch_regs.transfer_pattern_step += 1;
        dma_ch_regs.inc_a_bus_addr();

        let value = bus.read(src_addr);
        bus.write(dst_addr, value);
    }

    pub fn do_hdma(&mut self, bus: &mut CpuBus, cpu_stopped: &mut bool) {
        // Table entry finished
        if self.regs[self.hdma_active_ch].scanlines_left == 0 {
            if !self.hdma_load_entry(self.hdma_active_ch, bus) {
                // Channel exhausted, find next active channel
                self.hdma_active_ch = (self.hdma_active_ch + 1..8)
                    .find(|&ch| self.regs[ch].hdma_en)
                    .unwrap_or(8);
            }
        }

        // No active HDMA channel found
        if self.hdma_active_ch == 8 {
            self.hdma_en = false;
            self.hdma_pending = false;
            *cpu_stopped = false;
            return;
        }

        let hdma_ch_regs = &mut self.regs[self.hdma_active_ch];

        let a_bus_addr: Address;
        let b_bus_addr: Address;

        if hdma_ch_regs.indirect_hdma {
            a_bus_addr = hdma_ch_regs.hdma_indirect_table_addr;
            b_bus_addr = hdma_ch_regs.get_b_with_offset();

            if hdma_ch_regs.hdma_repeat_flag {
                hdma_ch_regs.hdma_indirect_table_addr.offset += 1;
            }
        } else {
            a_bus_addr = Address {
                bank: hdma_ch_regs.a_bus_addr.bank,
                offset: hdma_ch_regs.hdma_table_offset,
            };
            b_bus_addr = hdma_ch_regs.get_b_with_offset();

            if hdma_ch_regs.hdma_repeat_flag {
                hdma_ch_regs.hdma_table_offset += 1;
            }
        }



        let (src_addr, dst_addr) = match hdma_ch_regs.direction {
            Direction::AtoB => (a_bus_addr, b_bus_addr),
            Direction::BtoA => (b_bus_addr, a_bus_addr),
        };

        let transfer_pattern_length = match hdma_ch_regs.transfer_pattern {
            // Stop after first byte
            TransferPattern::Pattern0 => 1,
            // Stop after two bytes
            TransferPattern::Pattern1
            | TransferPattern::Pattern2
            | TransferPattern::Pattern6 => 2,
            // Stop after four bytes
            TransferPattern::Pattern3
            | TransferPattern::Pattern4
            | TransferPattern::Pattern5
            | TransferPattern::Pattern7 => 4,
        };

        hdma_ch_regs.transfer_pattern_step += 1;
        hdma_ch_regs.transfer_pattern_step %= transfer_pattern_length;

        if hdma_ch_regs.transfer_pattern_step == 0 {
            // Full pattern transferred for this scanline; stop until next hblank
            self.hdma_en = false;
            *cpu_stopped = false;

            // Non-repeat: only transfer once per entry. Repeat: transfer every scanline.
            if !hdma_ch_regs.hdma_repeat_flag {
                hdma_ch_regs.hdma_do_transfer = false;
            }

            if hdma_ch_regs.hdma_entry_just_loaded {
                hdma_ch_regs.hdma_entry_just_loaded = false;
            } else {
                hdma_ch_regs.scanlines_left -= 1;
            }
        }

        if hdma_ch_regs.hdma_do_transfer {
            let value = bus.read(src_addr);
            bus.write(dst_addr, value);
        }
    }

    /// Called once per frame before the first hblank of active display.
    /// Resets table pointers and loads the first entry for every HDMA-enabled channel.
    /// Channels whose first entry has scanline_count == 0 are disabled immediately.
    pub fn hdma_init_channels(&mut self, bus: &mut CpuBus) {
        for ch in 0..8 {
            if !self.regs[ch].hdma_en {
                continue;
            }

            // Reset table pointer to the base A-bus address for this frame
            self.regs[ch].hdma_table_offset = self.regs[ch].a_bus_addr.offset;
            self.regs[ch].hdma_initialized = true;

            if !self.hdma_load_entry(ch, bus) {
                // Channel had an empty table; already disabled inside hdma_load_entry
                continue;
            }
        }

        // Set hdma_active_ch to first still-enabled channel
        self.hdma_active_ch = (0..8)
            .find(|&ch| self.regs[ch].hdma_en)
            .unwrap_or(8);

        self.hdma_pending = self.hdma_active_ch < 8;
    }

    /// Reads the next HDMA table entry for `ch` into runtime state.
    /// Advances hdma_table_offset past the consumed bytes.
    /// For indirect mode, also reads and stores hdma_indirect_table_addr.
    /// Returns false if scanline_count == 0 (end of table), disabling the channel.
    pub fn hdma_load_entry(&mut self, ch: usize, bus: &mut CpuBus) -> bool {
        let table_addr = Address {
            bank: self.regs[ch].a_bus_addr.bank,
            offset: self.regs[ch].hdma_table_offset,
        };

        let scanline_count = bus.read(table_addr);
        self.regs[ch].hdma_table_offset += 1;

        if scanline_count == 0 {
            self.regs[ch].hdma_en = false;
            return false;
        }

        self.regs[ch].entry_scanline_count = scanline_count & 0x7F;
        self.regs[ch].scanlines_left = scanline_count & 0x7F;
        self.regs[ch].hdma_repeat_flag = get_bit_n!(scanline_count, 7);
        self.regs[ch].hdma_entry_just_loaded = true;
        self.regs[ch].transfer_pattern_step = 0;
        self.regs[ch].hdma_do_transfer = true;

        if self.regs[ch].indirect_hdma {
            let lo_addr = Address {
                bank: self.regs[ch].a_bus_addr.bank,
                offset: self.regs[ch].hdma_table_offset,
            };
            let lo = bus.read(lo_addr);

            let hi_addr = Address { offset: lo_addr.offset + 1, ..lo_addr };
            let hi = bus.read(hi_addr);

            self.regs[ch].hdma_table_offset += 2;

            // Bank byte comes from $43n7, already stored in hdma_indirect_table_addr.bank
            let indirect_bank = self.regs[ch].hdma_indirect_table_addr.bank;
            self.regs[ch].hdma_indirect_table_addr = Address {
                bank: indirect_bank,
                offset: u16::from_le_bytes([lo, hi]),
            };
        }
        
        true
    }
}