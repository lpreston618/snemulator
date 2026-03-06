pub const SCREEN_WIDTH: u32 = 160;  // Your emulator's native resolution
pub const SCREEN_HEIGHT: u32 = 144;

/// 128 KiB of WRAM
pub const WRAM_SIZE: usize = 128 * 1024;
/// 64 KiB of Video RAM
pub const VRAM_SIZE: usize = 32 * 1024;
/// 512 Bytes of Character-Graphics RAM (256 colors)
pub const CGRAM_SIZE: usize = 256;
/// 544 Bytes of Object Attribute Memory
pub const OAM_SIZE: usize = 544;