use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Self = Color { r: 0, g: 0, b: 0 };
    pub const DARKEST_GRAY: Self = Color { r: 48, g: 48, b: 48 };
    pub const DARK_GRAY: Self = Color { r: 80, g: 80, b: 80 };
    pub const RED: Self = Color { r: 255, g: 0, b: 0 };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color {
            r,
            g,
            b,
        }
    }
    
    pub const fn to_bgr555(self) -> u16 {
        let r = (self.r >> 3) as u16;
        let g = (self.g >> 3) as u16;
        let b = (self.b >> 3) as u16;
        (b << 11) | (g << 5) | r
    }
    
    pub const fn from_bgr555(color: u16) -> Self {
        let r = ((color & 0x001F) << 3) as u8;
        let g = ((color & 0x03E0) >> 2) as u8;
        let b = ((color & 0x7C00) >> 7) as u8;
        Color { r, g, b }
    }

    pub const fn to_rgba8888(self) -> u32 {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;
        (r << 24) | (g << 16) | (b << 8) | 0xFF
    }

    pub const fn to_rgba_bytes(self) -> [u8; 4] {
        [self.r, self.g, self.b, 0xFFu8]
    }
}
