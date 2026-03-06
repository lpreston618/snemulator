#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    _a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            _a: 255,
        }
    }
    
    pub fn to_bgr555(self) -> u16 {
        let r = (self.r >> 3) as u16;
        let g = (self.g >> 3) as u16;
        let b = (self.b >> 3) as u16;
        (b << 11) | (g << 5) | r
    }
    
    pub fn from_bgr555(color: u16) -> Self {
        let r = ((color & 0x001F) << 3) as u8;
        let g = ((color & 0x03E0) >> 2) as u8;
        let b = ((color & 0x7C00) >> 7) as u8;
        Self { r, g, b, _a: 255 }
    }
}
