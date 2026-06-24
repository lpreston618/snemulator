use serde::{Deserialize, Serialize};

use crate::{get_bit_n, sppu::Color};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ObjectSize {
    Size8x8,
    Size16x16,
    Size32x32,
    Size64x64,
    Size16x32,
    Size32x64,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum TileSize {
    #[default]
    Size8x8,
    Size16x16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum TilemapCount {
    #[default]
    One,
    Two,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum VramIncMode {
    #[default]
    HighByte,
    LowByte,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum AddressRemapping {
    #[default]
    None,
    ColDepth2,
    ColDepth4,
    ColDepth8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ColorDepth {
    #[default]
    Bpp2,
    Bpp4,
    Bpp8,
    // Direct,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum IncrSize {
    #[default]
    Bytes2,
    Bytes64,
    Bytes256,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum M7FillMode {
    #[default]
    Transparent,
    Character,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum WindowLogic {
    #[default]
    Or,
    And,
    Xor,
    Xnor,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum WindowColorRegion {
    #[default]
    Nowhere,
    Outside,
    Inside,
    Everywhere,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum CMathOperator {
    #[default]
    Add,
    Subtract,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum MasterSlave {
    #[default]
    Master,
    Slave,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum VideoType {
    #[default]
    Ntsc,
    Pal,
}

/// Contains all the relavent information about a sprite to be rendered
#[derive(Debug, Clone, Default)]
pub struct OAMSprite {
    pub x: i16,
    // pub max_x: u16,
    pub y: u8,
    // pub max_y: u16,
    pub tile_idx: u8,
    pub use_second_obj_table: bool,
    pub palette: u8,
    pub priority: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    // pub width: u8,
    // pub height: u8,
    pub size_select: bool,
}

impl OAMSprite {
    pub fn from_raw(bytes: &[u8], x_hi: bool, size_select: bool) -> OAMSprite {
        let mut sprite = Self::default();

        sprite.write_byte0(bytes[0]);
        sprite.write_byte1(bytes[1]);
        sprite.write_byte2(bytes[2]);
        sprite.write_byte3(bytes[3]);
        sprite.write_high_table_data(x_hi, size_select);

        sprite
    }

    pub fn write_byte0(&mut self, value: u8) {
        self.x = ((self.x >> 8) << 8) | value as i16;
    }

    pub fn write_byte1(&mut self, value: u8) {
        self.y = value;
    }

    pub fn write_byte2(&mut self, value: u8) {
        self.tile_idx = value;
    }

    pub fn write_byte3(&mut self, value: u8) {
        self.flip_y = get_bit_n!(value, 7);
        self.flip_x = get_bit_n!(value, 6);
        self.priority = (value >> 4) & 3;
        self.palette = (value >> 1) & 7;
        self.use_second_obj_table = get_bit_n!(value, 0);
    }

    pub fn write_high_table_data(&mut self, x_hi: bool, size_select: bool) {
        self.x &= 0xFF;
        self.x -= if x_hi { 0x100 } else { 0 };
        self.size_select = size_select;
    }

    pub fn sprite_size(&self, obj_sprite_size: ObjectSizeSelect) -> (usize, usize) {
        let spr_size = if self.size_select {
            match obj_sprite_size {
                ObjectSizeSelect::Size8x8_16x16 => ObjectSize::Size16x16,
                ObjectSizeSelect::Size8x8_32x32 => ObjectSize::Size32x32,
                ObjectSizeSelect::Size8x8_64x64 => ObjectSize::Size64x64,
                ObjectSizeSelect::Size16x16_32x32 => ObjectSize::Size32x32,
                ObjectSizeSelect::Size16x16_64x64 => ObjectSize::Size64x64,
                ObjectSizeSelect::Size32x32_64x64 => ObjectSize::Size64x64,
                ObjectSizeSelect::Size16x32_32x64 => ObjectSize::Size32x64,
                ObjectSizeSelect::Size16x32_32x32 => ObjectSize::Size32x32,
            }
        } else {
            match obj_sprite_size {
                ObjectSizeSelect::Size8x8_16x16 => ObjectSize::Size8x8,
                ObjectSizeSelect::Size8x8_32x32 => ObjectSize::Size8x8,
                ObjectSizeSelect::Size8x8_64x64 => ObjectSize::Size8x8,
                ObjectSizeSelect::Size16x16_32x32 => ObjectSize::Size16x16,
                ObjectSizeSelect::Size16x16_64x64 => ObjectSize::Size16x16,
                ObjectSizeSelect::Size32x32_64x64 => ObjectSize::Size32x32,
                ObjectSizeSelect::Size16x32_32x64 => ObjectSize::Size16x32,
                ObjectSizeSelect::Size16x32_32x32 => ObjectSize::Size16x32,
            }
        };
        
        match spr_size {
            ObjectSize::Size8x8 => (8, 8),
            ObjectSize::Size16x16 => (16, 16),
            ObjectSize::Size16x32 => (16, 32),
            ObjectSize::Size32x32 => (32, 32),
            ObjectSize::Size32x64 => (32, 64),
            ObjectSize::Size64x64 => (64, 64),
        }
    }
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
    pub color: Color,
    pub priority: u8,
    pub transparent: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ObjColorData {
    pub color: Color,
    pub palette: u8,
    pub priority: u8,
    pub transparent: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ObjPixelData {
    pub main: ObjColorData,
    pub sub: ObjColorData,
}

#[derive(Debug, Clone)]
pub struct TileData {
    pub tile_addr: u16,
    pub tile_row: u8,
    pub tile_col: u8,
    pub tile_size: TileSize,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ChrData {
    pub chr_idx: u16,
    pub chr_row: u8,
    pub chr_col: u8,   // dot-specific; recomputed from tile_col + flip_x at read time
    pub chr_pal: u8,
    pub chr_priority: u8,
    pub flip_x: bool,
    pub tile_width: u8, // 8 or 16; needed to invert chr_col when flip_x
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    pub logic: WindowLogic,
    pub main_en: bool,
    pub sub_en: bool,
    pub w1_en: bool,
    pub w1_inv: bool,
    pub w2_en: bool,
    pub w2_inv: bool,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct LayerSettings {
    pub main_en: bool,
    pub sub_en: bool,
    pub cmath_en: bool,
    pub window: WindowSettings,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BgSettings {
    pub main_en: bool,
    pub sub_en: bool,
    pub cmath_en: bool,
    pub scroll_x: u16,
    pub scroll_y: u16,
    pub tilemap_cnt_x: TilemapCount,
    pub tilemap_cnt_y: TilemapCount,
    pub chr_size: TileSize,
    pub chr_base_addr: u16,
    pub tilemap_base_addr: u16,
    pub mosaic_en: bool,
    pub window: WindowSettings,
}

pub struct DotColorData {
    pub main_col: Color,
    pub sub_col: Color,
    pub cmath_en: bool,
}

#[derive(Default, Clone)]
pub struct TileRowCacheEntry {
    pub valid: bool,
    pub tile_addr: u16,
    pub tile_row_key: u8,
    pub tile_col_block: u8, // tile_col / 8
    pub chr_data: ChrData,  // chr_col field is unused; see above
    pub pal_indices: u64,
}

impl TileRowCacheEntry {
    pub fn invalidate(&mut self) {
        self.valid = false;
    }
}

#[derive(Clone)]
pub struct TileRowCache<const SIZE: usize> {
    pub entries: [TileRowCacheEntry; SIZE],
    replace: usize,
}

impl<const SIZE: usize> TileRowCache<SIZE> {
    pub fn new() -> Self {
        Self {
            entries: std::array::from_fn(|_| TileRowCacheEntry::default()),
            replace: 0,
        }
    }
    
    /// Get the index of the entry containing the given tile, or None if tile is not in cache.
    pub fn get_entry_idx(&self, tile_data: &TileData) -> Option<usize> {
        let tile_col_block = tile_data.tile_col / 8;
        
        for (i, t) in self.entries.iter().enumerate() {
            if t.valid && t.tile_addr == tile_data.tile_addr
                && t.tile_row_key == tile_data.tile_row
                && t.tile_col_block == tile_col_block {
                    return Some(i);
            }
        }
        
        None
    }
    
    /// Replace a tile in the cache with a new entry. Returns the index of the replaced tile.
    pub fn cache_tile(&mut self, new_entry: TileRowCacheEntry) -> usize {
        let replaced_idx = self.replace;
        
        self.entries[replaced_idx] = new_entry;
        self.replace = (self.replace + 1) % SIZE; // Simple RR replacement strategy
        
        replaced_idx
    }
}