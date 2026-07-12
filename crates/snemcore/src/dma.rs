use crate::debug::DebugHarness;
use crate::{get_bit_n, savestate};
use crate::scpu::Address;
use crate::scpu::bus::CpuBus;

pub mod regs;
mod types;

pub use types::*;
pub use regs::DmaRegs;

pub struct DmaController {
    pub regs: [DmaRegs; 8],
    pub hdma_needs_init: bool,
    pub dma_active_ch: usize,
    pub hdma_active_ch: usize,
}

impl DmaController {
    pub fn new() -> Self {
        Self {
            regs: [DmaRegs::default(); 8],
            hdma_needs_init: false,
            dma_active_ch: 8,
            hdma_active_ch: 8,
        }
    }

    pub fn save_state(&self) -> savestate::DmaState {
        savestate::DmaState {
            hdma_needs_init: self.hdma_needs_init,
            dma_active_ch: self.dma_active_ch,
            hdma_active_ch: self.hdma_active_ch,
            channels: [
                self.regs[0].save_state(), self.regs[1].save_state(),
                self.regs[2].save_state(), self.regs[3].save_state(),
                self.regs[4].save_state(), self.regs[5].save_state(),
                self.regs[6].save_state(), self.regs[7].save_state(),
            ],
        }
    }

    pub fn load_state(&mut self, state: &savestate::DmaState, version: u32) {
        self.hdma_needs_init = state.hdma_needs_init;
        self.dma_active_ch = state.dma_active_ch;
        self.hdma_active_ch = state.hdma_active_ch;

        for ch in 0..8usize {
            self.regs[ch].load_state(&state.channels[ch], version);
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
        self.hdma_needs_init = true;
        self.dma_active_ch = 8;
        self.hdma_active_ch = 8;
    }

    #[allow(non_snake_case)]
    pub fn write_420B<H: DebugHarness>(&mut self, value: u8, harness: &mut H) {        
        self.dma_active_ch = value.trailing_zeros() as usize;
        
        for i in 0..8 {
            self.regs[i].dma_en = get_bit_n!(value, i);

            if self.regs[i].dma_en {
                self.regs[i].transfer_pattern_step = 0;
                self.regs[i].dma_bytes_transferred = 0;
            }
        }

        if H::IS_DEBUGGING_HARNESS && H::TRACK_DMA && self.dma_active_ch < 8 {
            harness.on_dma_start(self, self.dma_active_ch);
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

    /// Returns whether a dma transfer occured
    pub fn do_dma<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) -> bool {
        // HDMA indirect table register is same as DMA byte count register
        let byte_count = self.regs[self.dma_active_ch].hdma_indirect_table_addr.offset;

        // Channel's DMA transfer complete
        if byte_count == 0 {
            if H::IS_DEBUGGING_HARNESS && H::TRACK_DMA {
                bus.harness.on_dma_end(self, self.dma_active_ch);
            }

            self.regs[self.dma_active_ch].dma_en = false;
            self.dma_active_ch += 1;

            'seek_active_channel: while self.dma_active_ch < 8 {
                let dma_ch_regs = &mut self.regs[self.dma_active_ch];

                let byte_count = dma_ch_regs.hdma_indirect_table_addr.offset;

                if dma_ch_regs.dma_en {
                    // Active channel found
                    if byte_count != 0 {
                        if H::IS_DEBUGGING_HARNESS && H::TRACK_DMA {
                            bus.harness.on_dma_start(self, self.dma_active_ch);
                        }

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
            return false;
        }

        let dma_ch_regs = &mut self.regs[self.dma_active_ch]; // No longer mutable

        let a_bus_addr = dma_ch_regs.a_bus_addr;
        let b_bus_addr = dma_ch_regs.get_b_with_offset();

        let (src_addr, dst_addr) = match dma_ch_regs.direction {
            Direction::AtoB => (a_bus_addr, b_bus_addr),
            Direction::BtoA => (b_bus_addr, a_bus_addr),
        };

        dma_ch_regs.hdma_indirect_table_addr.offset -= 1; // byte_count -= 1
        dma_ch_regs.dma_bytes_transferred += 1;
        dma_ch_regs.transfer_pattern_step += 1;
        dma_ch_regs.transfer_pattern_step %= dma_ch_regs.transfer_pattern_length();
        dma_ch_regs.inc_a_bus_addr();

        let value = bus.read(src_addr);
        bus.write(dst_addr, value);

        if H::IS_DEBUGGING_HARNESS && H::TRACK_DMA {
            bus.harness.on_dma_transfer(self, self.dma_active_ch, src_addr, dst_addr, value);
        }

        true
    }

    fn do_hdma_transfer<H: DebugHarness>(&mut self, ch: usize, bus: &mut CpuBus<H>) -> usize {
        let hdma_ch_regs = &mut self.regs[ch];

        let num_bytes = hdma_ch_regs.transfer_pattern_length();
        
        if hdma_ch_regs.hdma_entry_just_loaded || hdma_ch_regs.hdma_repeat_flag {
            hdma_ch_regs.transfer_pattern_step = 0;

            for _ in 0..num_bytes {
                let a_bus_addr: Address;
                let b_bus_addr: Address;

                if hdma_ch_regs.indirect_hdma {
                    a_bus_addr = hdma_ch_regs.hdma_indirect_table_addr;
                    b_bus_addr = hdma_ch_regs.get_b_with_offset();

                    hdma_ch_regs.hdma_indirect_table_addr.offset += 1;
                } else {
                    a_bus_addr = Address {
                        bank: hdma_ch_regs.a_bus_addr.bank,
                        offset: hdma_ch_regs.hdma_table_offset,
                    };
                    b_bus_addr = hdma_ch_regs.get_b_with_offset();

                    hdma_ch_regs.hdma_table_offset += 1;
                }
                
                let (src_addr, dst_addr) = match hdma_ch_regs.direction {
                    Direction::AtoB => (a_bus_addr, b_bus_addr),
                    Direction::BtoA => (b_bus_addr, a_bus_addr),
                };

                let value = bus.read(src_addr);
                bus.write(dst_addr, value);
                
                hdma_ch_regs.transfer_pattern_step += 1;
            }
        }
                
        hdma_ch_regs.hdma_entry_just_loaded = false;
        hdma_ch_regs.scanlines_left -= 1;

        num_bytes as usize
    }

    /// Returns the number of clock cycles taken to do all H-DMA channel transfers
    pub fn do_hdma<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) -> usize {
        let mut num_bytes = 0;
        
        for ch in 0..8 {
            // Channel innactive, skip
            if !self.regs[ch].hdma_en { continue; }

            num_bytes += self.do_hdma_transfer(ch, bus);

            // Table entry finished
            if self.regs[ch].scanlines_left == 0 {
                // No more table entries, disable channel and go to next
                if !self.hdma_load_entry(ch, bus) {
                    continue;
                }
            }
        }
        
        num_bytes
    }

    /// Called once per frame before the first hblank of active display.
    /// Resets table pointers and loads the first entry for every HDMA-enabled channel.
    /// Channels whose first entry has scanline_count == 0 are disabled immediately.
    pub fn hdma_init_channels<H: DebugHarness>(&mut self, bus: &mut CpuBus<H>) {
        for ch in 0..8 {
            if !self.regs[ch].hdma_en {
                continue;
            }

            // Reset table pointer to the base A-bus address for this frame
            self.regs[ch].hdma_table_offset = self.regs[ch].a_bus_addr.offset;
            
            self.hdma_load_entry(ch, bus);

            self.regs[ch].hdma_initialized = true;
            
            if H::IS_DEBUGGING_HARNESS && H::TRACK_HDMA {
                bus.harness.on_hdma_init(self, ch);
            }
        }
    }

    /// Reads the next HDMA table entry for `ch` into runtime state.
    /// Advances hdma_table_offset past the consumed bytes.
    /// For indirect mode, also reads and stores hdma_indirect_table_addr.
    /// Returns false if scanline_count == 0 (end of table), disabling the channel.
    pub fn hdma_load_entry<H: DebugHarness>(&mut self, ch: usize, bus: &mut CpuBus<H>) -> bool {
        let regs = &mut self.regs[ch];
        
        let table_addr = Address {
            bank: regs.a_bus_addr.bank,
            offset: regs.hdma_table_offset,
        };

        let scanline_count = bus.read(table_addr);
        regs.hdma_table_offset += 1;

        if scanline_count == 0 {
            regs.hdma_en = false;
            return false;
        }

        regs.entry_scanline_count = scanline_count & 0x7F;
        regs.scanlines_left = scanline_count & 0x7F;
        regs.hdma_repeat_flag = get_bit_n!(scanline_count, 7);
        regs.hdma_entry_just_loaded = true;
        regs.transfer_pattern_step = 0;
        regs.hdma_do_transfer = true;

        if regs.indirect_hdma {
            let lo_addr = Address {
                bank: regs.a_bus_addr.bank,
                offset: regs.hdma_table_offset,
            };
            let lo = bus.read(lo_addr);

            let hi_addr = Address { offset: lo_addr.offset + 1, ..lo_addr };
            let hi = bus.read(hi_addr);

            regs.hdma_table_offset += 2;

            // Bank byte comes from $43n7, already stored in hdma_indirect_table_addr.bank
            let indirect_bank = regs.hdma_indirect_table_addr.bank;
            regs.hdma_indirect_table_addr = Address {
                bank: indirect_bank,
                offset: u16::from_le_bytes([lo, hi]),
            };
        }

        if H::IS_DEBUGGING_HARNESS && H::TRACK_HDMA {
            bus.harness.on_hdma_load_entry(self, ch);
        }
        
        true
    }
}