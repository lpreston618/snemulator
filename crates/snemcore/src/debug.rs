use crate::{Snemulator, dma::DmaController, scpu::{Address, Cpu65c816, CpuInterrupt}, ssmp::spc::Spc700};

#[allow(unused_variables)]
pub trait DebugHarness {
    /// Controls whether any callbacks will be called. By default, enabling this enables all
    /// callbacks. Individual callbacks can be disabled by setting other flags to false.
    /// Setting this to false will disable all callbacks regardless of other flags.
    const IS_DEBUGGING_HARNESS: bool;
    /// Controls whether the `on_power` and `on_reset` callbacks will be called for debugging harnesses.
    const TRACK_RESETS: bool = true;

    /// Controls whether the `on_instruction` callback will be called for debugging harnesses.
    const TRACK_CPU_INSTRUCTIONS: bool = true;
    /// Controls whether the `on_interrupt` callback will be called for debugging harnesses.
    const TRACK_CPU_INTERRUPTS: bool = true;
    /// Controls whether the `on_memory_write` and `on_memory_read` callbacks will be called for debugging harnesses.
    const TRACK_MEMORY: bool = true;
    /// Controls whether the `on_stack_push` and `on_stack_pop` callbacks will be called for debugging harnesses.
    const TRACK_STACK: bool = true;

    /// Controls whether the `on_dma_start`, `on_dma_tranfer`, and `on_dma_end` callbacks will be called for debugging harnesses.
    const TRACK_DMA: bool = true;
    /// Controls whether the `on_hdma_start`, `on_hdma_tranfer`, and `on_hdma_end` callbacks will be called for debugging harnesses.
    const TRACK_HDMA: bool = true;

    /// Controls whether the `on_vblank_start` and `on_vblank_end` callbacks will be called for debugging harnesses.
    const TRACK_VBLANK: bool = true;
    /// Controls whether the `on_hblank_start` and `on_hblank_end` callbacks will be called for debugging harnesses.
    const TRACK_HBLANK: bool = true;
    /// Controls whether the `on_fblank_start` and `on_fblank_end` callbacks will be called for debugging harnesses.
    const TRACK_FBLANK: bool = true;

    /// Controls whether the `on_ppu_step` callback will be called for debugging harnesses.
    const TRACK_PPU_STEP: bool = true;

    const TRACK_SPC_INSTRUCTIONS: bool = true;

    fn should_stop(&mut self, core: &mut Snemulator) -> bool { false }

    /// Called after each emulation step.
    fn on_emulation_step(&mut self, core: &mut Snemulator) {}

    /// Called after the core is powered on
    fn on_power(&mut self, core: &mut Snemulator) {}
    /// Called when the core is reset (also triggers on_interrupt)
    fn on_reset(&mut self, core: &mut Snemulator) {}

    /// Called after each instruction. prg_bytes contains the opcode and immediate data for the instruction.
    fn on_instruction(&mut self, cpu: &mut Cpu65c816, prg_bytes: &[u8]) {}
    /// Called after the CPU handles an interrupt.
    fn on_interrupt(&mut self, cpu: &mut Cpu65c816, kind: CpuInterrupt) {}
    /// Called after each CPU write.
    fn on_memory_write(&mut self, cpu: &mut Cpu65c816, addr: Address, value: u8) {}
    /// Called after each CPU read.
    fn on_memory_read(&mut self, cpu: &mut Cpu65c816, addr: Address, value: u8) {}
    /// Called after the CPU pushes a byte to the stack. SP is now the address of the value - 1.
    fn on_stack_push(&mut self, cpu: &mut Cpu65c816, value: u8) {}
    /// Called after the CPU pops a byte from the stack. SP is now the address of the value.
    fn on_stack_pop(&mut self, cpu: &mut Cpu65c816, value: u8) {}

    /// Called after DMA enable is set for the given channel.
    fn on_dma_start(&mut self, dma: &mut DmaController, channel: usize) {}
    /// Called after a single byte is transferred from `src_addr` to `dst_addr` over the given DMA channel.
    fn on_dma_transfer(&mut self, dma: &mut DmaController, channel: usize, src_addr: Address, dst_addr: Address, value: u8) {}
    /// Called after a DMA transfer completes on the given channel.
    fn on_dma_end(&mut self, dma: &mut DmaController, channel: usize) {}

    // /// Called after at the start of h-blank if H-DMA is enabled for the given channel.
    // fn on_hdma_start(&mut self, dma: &mut DmaController, channel: usize) {}
    // /// Called after a single byte is transferred from `src_addr` to `dst_addr` over the given H-DMA channel.
    // fn on_hdma_transfer(&mut self, dma: &mut DmaController, channel: usize, src_addr: Address, dst_addr: Address, value: u8) {}
    // /// Called after a single multi-byte transfer completes for the given channel. An H-DMA transfer may consist of 1 to 4 bytes each scanline.
    // fn on_hdma_scanline_end(&mut self, dma: &mut DmaController, channel: usize) {}
    // /// Called after an H-DMA transfer completes on the given channel (when all table entries have been processed)
    // fn on_hdma_end(&mut self, dma: &mut DmaController, channel: usize) {}

    /// Called on the first dot of v-blank. Can be used to perform operations once per frame.
    fn on_vblank_start(&mut self, core: &mut Snemulator) {}
    /// Called after the last dot of v-blank (when scanline & dot are both set to 0)
    fn on_vblank_end(&mut self, core: &mut Snemulator) {}
    /// Called on the first dot of h-blank each visible scanline (including scanline 0)
    fn on_hblank_start(&mut self, core: &mut Snemulator) {}
    /// Called after the last dot of h-blank each visible scanline.
    fn on_hblank_end(&mut self, core: &mut Snemulator) {}
    /// Called after the CPU writes 1 to f-blank enable when it was previously 0, or on reset.
    fn on_fblank_start(&mut self, core: &mut Snemulator) {}
    /// Called after the CPU writes 0 to f-blank enable when it was previously 1.
    fn on_fblank_end(&mut self, core: &mut Snemulator) {}

    /// Called after each time the PPU is advanced (once per dot).
    fn on_ppu_step(&mut self, core: &mut Snemulator) {}

    fn on_spc_instruction(&mut self, spc: &mut Spc700, prg_bytes: &[u8]) {}
}

/// Debug harness with no callbacks (compiles out for release builds)
pub struct NullHarness {}
impl DebugHarness for NullHarness {
    const IS_DEBUGGING_HARNESS: bool = false;
}