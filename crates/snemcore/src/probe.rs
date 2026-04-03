use crate::scpu::Cpu65c816;

pub trait DebugProbe {
    fn on_instruction(&mut self, scpu: &Cpu65c816<impl DebugProbe>) {}
    fn on_memory_read(&mut self, addr: u32, value: u8) {}
    fn on_memory_write(&mut self, addr: u32, value: u8) {}
    fn on_dot(&mut self, dot: u16) {}
    fn on_scanline(&mut self, line: u16) {}
    fn on_frame(&mut self) {}
    fn should_stop(&mut self) -> bool { false }
}

pub struct NullProbe {}
impl DebugProbe for NullProbe {}