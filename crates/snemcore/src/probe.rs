use crate::{Snemulator, scpu::CpuInterrupt};

pub trait DebugProbe: Sized {
    fn init(&mut self, core: &mut Snemulator<Self>) {}
    /// Called before the emulator starts running again.
    fn resume_emulation(&mut self) {}
    
    fn on_emulation_cycle(&mut self, core: &mut Snemulator<Self>) {}
    
    fn on_dot(&mut self, core: &mut Snemulator<Self>) {}
    fn on_scanline(&mut self, core: &mut Snemulator<Self>) {}
    /// Called when the PPU finishes rendering all dots for this frame (the start of vblank)
    fn on_frame(&mut self, core: &mut Snemulator<Self>) {}
    /// Called before the instruction at full_pc is executed.
    
    fn on_instruction(&mut self, core: &mut Snemulator<Self>) {}
    fn on_interrupt(&mut self, kind: CpuInterrupt) {}
    
    fn on_memory_read(&mut self, addr: u32, value: u8) {}
    fn on_memory_write(&mut self, addr: u32, value: u8) {}
    
    fn on_dma_start(&mut self, channel: usize) {}
    fn on_dma_transfer(&mut self, channel: usize, src_addr: u32, dst_addr: u32, value: u8) {}
    fn on_dma_end(&mut self, channel: usize) {}
    
    fn on_hdma_start(&mut self, channel: usize) {}
    fn on_hdma_transfer(&mut self, channel: usize, src_addr: u32, dst_addr: u32, value: u8) {}
    fn on_hdma_end(&mut self, channel: usize) {}
    
    fn should_stop(&mut self) -> bool {
        false
    }
}

pub struct NullProbe {}
impl DebugProbe for NullProbe {}
