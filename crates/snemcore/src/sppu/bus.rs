use crate::debug::DebugHarness;
use crate::scpu::ioregs::CpuIoRegs;
use crate::sppu::OAMSprite;
use crate::sppu::color::Color;
use crate::sppu::regs::PpuRegs;
use crate::sysinfo::{CGRAM_SIZE, OAM_SPRITE_COUNT, VRAM_SIZE};

pub struct PpuBus<'a, H: DebugHarness> {
    pub vram: &'a mut [u16; VRAM_SIZE],
    pub cgram: &'a mut [Color; CGRAM_SIZE],
    pub oam: &'a mut [OAMSprite; OAM_SPRITE_COUNT],
    pub ppu_regs: &'a mut PpuRegs,
    pub cpu_regs: &'a mut CpuIoRegs,
    pub frame_buffer: &'a mut [u8],
    pub frame_ready: &'a mut bool,

    pub harness: &'a mut H,
    pub vblank_start: &'a mut bool,
    pub vblank_end: &'a mut bool,
    pub hblank_start: &'a mut bool,
    pub hblank_end: &'a mut bool,
}

