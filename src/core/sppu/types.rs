use crate::core::sppu;

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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

/// Contains all the relavent information about a sprite to be rendered
#[derive(Debug)]
pub struct OAMSprite {
    pub x: u16,
    pub max_x: u16,
    pub y: u8,
    pub tile_idx: u8,
    pub use_second_obj_table: bool,
    pub palette: u8,
    pub priority: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorLayer {
    Bg1,
    Bg2,
    Bg3,
    Bg4,
    Obj,
    Back,
}

#[derive(Clone, Copy, Debug)]
pub struct ColorData {
    pub color: sppu::Color,
    pub priority: u8,
    pub transparent: bool,
}

#[derive(Debug, Clone)]
pub struct TileData {
    pub tile_addr: u16,
    pub tile_row: u8,
    pub tile_col: u8,
    pub tile_size: TileSize,
}

#[derive(Debug)]
pub struct ChrData {
    pub chr_idx: u16,
    pub chr_row: u8,
    pub chr_col: u8,
    pub chr_pal: u8,
    pub chr_priority: u8,
}

pub struct BgData {
    pub scroll_x: u16,
    pub scroll_y: u16,
    pub tilemap_cnt_x: TilemapCount,
    pub tilemap_cnt_y: TilemapCount,
    pub tile_size: TileSize,
    pub tilemap_base_addr: u16,
    pub mosaic_en: bool,
}

pub struct DotColorData {
    pub main_col: sppu::Color,
    pub sub_col: sppu::Color,
    pub cmath_en: bool,
}