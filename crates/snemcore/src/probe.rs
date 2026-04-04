use crate::{Snemulator, sppu::Ppu5C7x};

pub trait DebugProbe: Sized {
    /// Called before the emulator starts running again.
    fn resume_emulation(&mut self) {}
    /// Called before the instruction at full_pc is executed.
    fn on_instruction(&mut self, core: &mut Snemulator<Self>) {}
    fn on_memory_read(&mut self, addr: u32, value: u8) {}
    fn on_memory_write(&mut self, addr: u32, value: u8) {}
    fn on_dot(&mut self, core: &mut Snemulator<Self>) {}
    fn on_scanline(&mut self, core: &mut Snemulator<Self>) {}
    /// Called when the PPU finishes rendering all dots for this frame (the start of vblank)
    fn on_frame_end(&mut self, core: &mut Snemulator<Self>) {}
    fn should_stop(&mut self) -> bool {
        false
    }
}

pub struct NullProbe {}
impl DebugProbe for NullProbe {}
