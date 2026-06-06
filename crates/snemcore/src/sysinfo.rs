pub const SCREEN_WIDTH: u32 = 2 * 256;
pub const SCREEN_HEIGHT: u32 = 2 * 224;

/// 128 KiB of WRAM
pub const WRAM_SIZE: usize = 128 * 1024;
/// 64 KiB of Video RAM
pub const VRAM_SIZE: usize = 32 * 1024;
/// 64 KiB of Audio RAM
pub const ARAM_SIZE: usize = 64 * 1024;
/// 512 Bytes of Character-Graphics RAM (256 colors)
pub const CGRAM_SIZE: usize = 256;
/// 544 Bytes of Object Attribute Memory
pub const OAM_SIZE: usize = 512 + 32;
/// Frequency of the SNES master clock
pub const MASTER_CLOCK_HZ: usize = 21477300;
/// Frequency of the SPC700 internal clock
pub const SPC_CLOCK_HZ: usize = 1024000;

/*
TIMER2 runs at 64kHz, which translates to one tick per every 48 DSP clocks.
The Spc700 clocks every 3 DSP clocks, so TIMER2 is clocked once every 48/3, or 16, Spc700 clocks.
Timers 0 and 1 each run at 1/8th that speed (8kHz), so they are clocked every 16*8, or 128, SPC clocks.
*/
/// Clock period for the fast timer (16 SPC clocks)
pub const FAST_TIMER_CLOCK_PERIOD: usize = 16;
/// Clock period for the slow timer (128 SPC clocks)
pub const SLOW_TIMER_CLOCK_PERIOD: usize = 128;

pub const AUDIO_SAMPLE_HZ: usize = 32000;

/// Number of master clocks between joypad autoread steps. Autoread takes 4224
/// master clocks, four 16-bit regs need filling.
pub const CLOCKS_BETWEEN_AUTOREAD_STEPS: usize = 4224/16 * 4;

pub const FRAMES_PER_SECOND: f32 = 60.099;