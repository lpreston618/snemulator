#[derive(Clone, Copy, Debug, Default)]
pub enum ObjectSizeSelect {
    #[default]
    Size8x8_16x16,
    Size8x8_32x32,
    Size8x8_64x64,
    Size16x16_32x32,
    Size16x16_64x64,
    Size32x32_64x64,
    Size16x32_32x64,
    Size16x32_32x32,
}

// impl From<u8> for ObjectSizeSelect {
//     fn from(value: u8) -> Self {
//         match
//     }
// }

#[derive(Clone, Copy, Debug)]
pub enum ObjectSize {
    Size8x8,
    Size16x16,
    Size32x32,
    Size64x64,
    Size16x32,
    Size32x64,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum TileSize {
    #[default]
    Size8x8,
    Size16x16,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum BgMode {
    #[default]
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Mode6,
    Mode7,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum TilemapCount {
    #[default]
    One,
    Two,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum VramIncMode {
    #[default]
    HighByte,
    LowByte,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AddressRemapping {
    #[default]
    None,
    ColDepth2,
    ColDepth4,
    ColDepth8,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ColorDepth {
    #[default]
    Bpp2,
    Bpp4,
    Bpp8,
    // Direct,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum IncrSize {
    #[default]
    Bytes2,
    Bytes64,
    Bytes256,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum M7FillMode {
    #[default]
    Transparent,
    Character,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum WindowLogic {
    #[default]
    Or,
    And,
    Xor,
    Xnor,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum WindowColorRegion {
    #[default]
    Nowhere,
    Outside,
    Inside,
    Everywhere,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum CMathOperator {
    #[default]
    Add,
    Subtract,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum MasterSlave {
    #[default]
    Master,
    Slave,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum VideoType {
    #[default]
    Ntsc,
    Pal,
}
