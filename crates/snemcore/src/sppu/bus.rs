use crate::probe::DebugProbe;
use crate::scpu::CpuInterrupt;
use crate::scpu::ioregs::CpuIoRegs;
use crate::sppu::color::Color;
use crate::sppu::regs::PpuRegs;
use crate::sysinfo::{CGRAM_SIZE, OAM_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, VRAM_SIZE};

pub struct PpuBus<'a> {
    pub vram: &'a mut [u16; VRAM_SIZE],
    pub cgram: &'a mut [Color; CGRAM_SIZE],
    pub oam: &'a mut [u8; OAM_SIZE],
    pub ppu_regs: &'a mut PpuRegs,
    pub cpu_regs: &'a mut CpuIoRegs,
    pub frame_buffer: &'a mut [u8],
    pub frame_ready: &'a mut bool,
    pub interrupt: &'a mut Option<CpuInterrupt>,
}

impl<'a> PpuBus<'a> {
    pub fn trigger_interrupt(&mut self, interrupt: CpuInterrupt) {
        *self.interrupt = Some(interrupt);
    }
    
    pub fn set_frame_finished(&mut self) {
        *self.frame_ready = true;
    }
}

