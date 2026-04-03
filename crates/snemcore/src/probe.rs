use crate::Snemulator;

pub trait DebugProbe {
    /// Called before the emulator starts running again.
    fn resume_emulation(&mut self) {}
    /// Called before the instruction at full_pc is executed.
    fn on_instruction(&mut self, full_pc: u32) {}
    fn on_memory_read(&mut self, addr: u32, value: u8) {}
    fn on_memory_write(&mut self, addr: u32, value: u8) {}
    fn on_dot(&mut self, dot: u16) {}
    fn on_scanline(&mut self, line: u16) {}
    fn on_frame(&mut self) {}
    fn should_stop(&mut self) -> bool {
        false
    }
}

pub struct NullProbe {}
impl DebugProbe for NullProbe {}
