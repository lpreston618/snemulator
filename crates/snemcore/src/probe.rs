use crate::{Snemulator, scpu::CpuInterrupt};

pub trait DebugProbe: Sized {
    fn init(&mut self, _core: &mut Snemulator<Self>) {}
    /// Called before the emulator starts running again.
    fn resume_emulation(&mut self) {}
    
    fn on_emulation_cycle(&mut self, _core: &mut Snemulator<Self>) {}
    
    fn on_dot(&mut self, _core: &mut Snemulator<Self>) {}
    fn on_scanline(&mut self, _core: &mut Snemulator<Self>) {}
    /// Called when the PPU finishes rendering all dots for this frame (the start of vblank)
    fn on_frame(&mut self, _core: &mut Snemulator<Self>) {}
    /// Called before the instruction at full_pc is executed.
    
    fn on_instruction(&mut self, _core: &mut Snemulator<Self>) {}
    fn on_interrupt(&mut self, _kind: CpuInterrupt) {}
    
    fn on_memory_read(&mut self, _addr: u32, _value: u8) {}
    fn on_memory_write(&mut self, _addr: u32, _value: u8) {}
    
    fn on_dma_start(&mut self, _channel: usize) {}
    fn on_dma_transfer(&mut self, _channel: usize, _src_addr: u32, _dst_addr: u32, _value: u8) {}
    fn on_dma_end(&mut self, _channel: usize) {}
    
    fn on_hdma_start(&mut self, _channel: usize) {}
    fn on_hdma_transfer(&mut self, _channel: usize, _src_addr: u32, _dst_addr: u32, _value: u8) {}
    fn on_hdma_end(&mut self, _channel: usize) {}
    
    fn should_stop(&mut self) -> bool {
        false
    }
}

pub struct NullProbe {}
impl DebugProbe for NullProbe {}
