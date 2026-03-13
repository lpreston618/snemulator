use paste::paste;
use crate::core::scpu::CpuInterrupt;
use crate::core::scpu::ioregs::HVTimerIRQ;
use crate::core::sppu::bus::PpuBus;
use crate::core::sppu::color::Color;
use crate::core::sppu::regs::{
    BgMode, CMathOperator, ColorDepth, ObjectSize, ObjectSizeSelect, PpuRegs, TileSize, TilemapCount, WindowColorRegion, WindowLogic
};

pub mod bus;
pub mod color;
pub mod regs;

pub const VBLANK_START_SCANLINE: usize = 225;
const VBLANK_END_SCANLINE_NTSC: usize = 261;
const VISIBLE_SCANLINE_START_DOT: usize = 22;
pub const HBLANK_START_DOT: usize = 278;
const SCANLINE_END_DOT: usize = 340;

macro_rules! win_active_signals {
    ($regs:expr, $screen_x:expr, $bg:ident) => {
        paste!( {
            let win_en = if $regs.[<$bg _win_main_en>] || $regs.[<$bg _win_sub_en>] {
                Ppu5C7x::win_active_signal(
                    $regs,
                    $screen_x,
                    $regs.[<$bg _w1_en>],
                    $regs.[<$bg _w2_en>],
                    $regs.[<$bg _w1_inv>],
                    $regs.[<$bg _w2_inv>],
                    $regs.[<$bg _win_logic>],
                )
            } else {
                false
            };
    
            let [<$bg _win_main_en>] = win_en && $regs.[<$bg _win_main_en>];
            let [<$bg _win_sub_en>] = win_en && $regs.[<$bg _win_sub_en>];
    
            ([<$bg _win_main_en>], [<$bg _win_sub_en>])
        } )
    };
}

/// Contains all the relavent information about a sprite to be rendered
#[derive(Debug)]
struct OAMSprite {
    x: u16,
    max_x: u16,
    y: u8,
    tile_idx: u8,
    use_second_obj_table: bool,
    palette: u8,
    priority: u8,
    flip_x: bool,
    flip_y: bool,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ColorLayer {
    Bg1,
    Bg2,
    Bg3,
    Bg4,
    Obj,
    Back,
}

#[derive(Clone, Copy)]
struct ColorData {
    color: Color,
    priority: u8,
    transparent: bool,
}

#[derive(Debug)]
struct TileData {
    tile_addr: u16,
    tile_row: u8,
    tile_col: u8,
    tile_size: TileSize,
}

struct ChrData {
    chr_idx: u16,
    chr_row: u8,
    chr_col: u8,
    chr_pal: u8,
    chr_priority: u8,
}

struct BgData {
    scroll_x: u16,
    scroll_y: u16, 
    tilemap_cnt_x: TilemapCount,
    tilemap_cnt_y: TilemapCount, 
    tile_size: TileSize,
    tilemap_base_addr: u16,
    mosaic_en: bool,
}

struct DotColorData {
    main_col: Color,
    sub_col: Color,
    cmath_en: bool,
}

pub struct Ppu5C7x {
    pub dot: usize,
    pub scanline: usize,
    pub frame: usize,
    
    scanline_sprites: Vec<OAMSprite>,
    scanline_spr_cnt: usize,
    
    /// Number of master clocks until the next dot
    pub clocks: usize,
}

impl Ppu5C7x {
    pub fn new() -> Self {
        Self {
            dot: 0,
            scanline: 0,
            frame: 0,
            scanline_sprites: Vec::new(),
            scanline_spr_cnt: 0,
            clocks: 0,
        }
    }
    
    /// Cycles the PPU for a certain number of master clocks
    pub fn cycle(&mut self, bus: &mut PpuBus) {
        if (0 < self.scanline && self.scanline < VBLANK_START_SCANLINE)
            && (VISIBLE_SCANLINE_START_DOT <= self.dot && self.dot < HBLANK_START_DOT)
        {
            self.draw_dot(bus);
        }

        self.update_dot_and_scanline(bus);
        self.update_hv_timers(bus);

        self.clocks += 4;

        if self.dot >= SCANLINE_END_DOT-4 {
            self.clocks += 1;
        }
    }
    
    fn draw_dot(&mut self, bus: &mut PpuBus) {        
        match bus.ppu_regs.bg_mode {
            BgMode::Mode0
            | BgMode::Mode1 
            | BgMode::Mode2 
            | BgMode::Mode3 
            | BgMode::Mode4 => self.draw_dot_modes_0to4(bus),

            BgMode::Mode5
            | BgMode::Mode6 => self.draw_dot_modes_5to6(bus),

            // BgMode::Mode7 => self.bg_mode7_dot(screen_x, screen_y, spr_col),
            _ => {}
        };
    }
    
    fn set_pixel(frame_buffer: &mut [u8], pixel_idx: usize, col: Color) {
        frame_buffer[pixel_idx + 0] = col.r;
        frame_buffer[pixel_idx + 1] = col.g;
        frame_buffer[pixel_idx + 2] = col.b;
        frame_buffer[pixel_idx + 3] = 255;
    }

    fn draw_dot_modes_0to4(&mut self, bus: &mut PpuBus) {
        let regs = &bus.ppu_regs;
        
        let screen_x = self.screen_x();
        let screen_y = self.screen_y();

        let spr_col = self.sprite_col(bus, screen_x, screen_y);

        let dot_col_data = match regs.bg_mode {
            BgMode::Mode0 => self.bg_mode0_dot(bus, screen_x, screen_y, spr_col),
            BgMode::Mode1 => self.bg_mode1_dot(bus, screen_x, screen_y, spr_col),
            BgMode::Mode2 => self.bg_mode2_dot(bus, screen_x, screen_y, spr_col),
            BgMode::Mode3 => self.bg_mode3_dot(bus, screen_x, screen_y, spr_col),
            BgMode::Mode4 => self.bg_mode4_dot(bus, screen_x, screen_y, spr_col),
            // BgMode::Mode5 => self.bg_mode5_dot(screen_x, screen_y, spr_col),
            // BgMode::Mode6 => self.bg_mode6_dot(screen_x, screen_y, spr_col),
            // BgMode::Mode7 => self.bg_mode7_dot(screen_x, screen_y, spr_col),
            _ => DotColorData { main_col: Color::BLACK, sub_col: Color::BLACK, cmath_en: false },
        };

        let (main_col, sub_col) = if regs.in_fblank {
            (Color::BLACK, Color::BLACK)
        } else {
            (dot_col_data.main_col, dot_col_data.sub_col)
        };
        let cmath_en = dot_col_data.cmath_en;
        
        let brightness = regs.screen_brightness;
        let p = self.frame & 1;

        if regs.screen_interlace_en && regs.hi_res_en {
            let main_col = if cmath_en {
                self.apply_cmath(bus, main_col, regs.fixed_color, screen_x)
            } else {
                main_col
            };
            let sub_col = if cmath_en {
                self.apply_cmath(bus, sub_col, regs.fixed_color, screen_x)
            } else {
                sub_col
            };
            let main_col = self.apply_brightness(main_col, brightness);
            let sub_col = self.apply_brightness(sub_col, brightness);
            
            let main_pixel_idx = 4 * ( (2*screen_y + p) * 512 + (2*screen_x + 0) );
            let sub_pixel_idx = 4 * ( (2*screen_y + p) * 512 + (2*screen_x + 1) );

            Ppu5C7x::set_pixel(bus.frame_buffer, main_pixel_idx, main_col);
            Ppu5C7x::set_pixel(bus.frame_buffer, sub_pixel_idx, sub_col);
        } else if regs.screen_interlace_en {
            let dot_col = if cmath_en {
                self.apply_cmath(bus, main_col, sub_col, screen_x)
            } else {
                main_col
            };
            let dot_col = self.apply_brightness(dot_col, brightness);
            
            let left_pixel_idx = 4 * ( (2*screen_y + p) * 512 + (2*screen_x + 0) );
            let right_pixel_idx = 4 * ( (2*screen_y + p) * 512 + (2*screen_x + 1) );

            Ppu5C7x::set_pixel(bus.frame_buffer, left_pixel_idx, dot_col);
            Ppu5C7x::set_pixel(bus.frame_buffer, right_pixel_idx, dot_col);
        } else if regs.hi_res_en {
            let main_col = if cmath_en {
                self.apply_cmath(bus, main_col, regs.fixed_color, screen_x)
            } else {
                main_col
            };
            let sub_col = if cmath_en {
                self.apply_cmath(bus, sub_col, regs.fixed_color, screen_x)
            } else {
                sub_col
            };
            let main_col = self.apply_brightness(main_col, brightness);
            let sub_col = self.apply_brightness(sub_col, brightness);
            
            let main_pixel_idx = 4 * ( (2*screen_y + p) * 512 + (2*screen_x + 0) );
            let sub_pixel_idx = 4 * ( (2*screen_y + p) * 512 + (2*screen_x + 1) );

            Ppu5C7x::set_pixel(bus.frame_buffer, main_pixel_idx, main_col);
            Ppu5C7x::set_pixel(bus.frame_buffer, sub_pixel_idx, sub_col);
        } else {
            let dot_col = if cmath_en {
                self.apply_cmath(bus, main_col, sub_col, screen_x)
            } else {
                main_col
            };
            let dot_col = self.apply_brightness(dot_col, brightness);
            
            let top_left_pixel_idx = 4 * ( (2*screen_y + 0) * 512 + (2*screen_x + 0) );
            let top_right_pixel_idx = 4 * ( (2*screen_y + 0) * 512 + (2*screen_x + 1) );
            let bottom_left_pixel_idx = 4 * ( (2*screen_y + 1) * 512 + (2*screen_x + 0) );
            let bottom_right_pixel_idx = 4 * ( (2*screen_y + 1) * 512 + (2*screen_x + 1) );

            Ppu5C7x::set_pixel(bus.frame_buffer, top_left_pixel_idx, dot_col);
            Ppu5C7x::set_pixel(bus.frame_buffer, top_right_pixel_idx, dot_col);
            Ppu5C7x::set_pixel(bus.frame_buffer, bottom_left_pixel_idx, dot_col);
            Ppu5C7x::set_pixel(bus.frame_buffer, bottom_right_pixel_idx, dot_col);
        }
    }

    fn draw_dot_modes_5to6(&mut self, bus: &mut PpuBus) {
        let regs = &bus.ppu_regs;
        
        let screen_x = self.screen_x();
        let screen_y = self.screen_y();

        let spr_col = self.sprite_col(bus, screen_x, screen_y);

        let dot_col_data1 = match regs.bg_mode {
            BgMode::Mode5 => self.bg_mode5_dot(bus, screen_x + 0, screen_y, spr_col.clone()),
            // BgMode::Mode6 => self.bg_mode6_dot(bus, 2*screen_x + 0, screen_y, spr_col),
            _ => unreachable!() // Only called for modes 5 & 6
        };
        let dot_col_data2 = match regs.bg_mode {
            BgMode::Mode5 => self.bg_mode5_dot(bus, screen_x + 256, screen_y, spr_col),
            // BgMode::Mode6 => self.bg_mode6_dot(2*screen_x + 1, screen_y, spr_col),
            _ => unreachable!() // Only called for modes 5 & 6
        };

        let brightness = regs.screen_brightness;
        let p = self.frame & 1;

        let dot_col1 = if dot_col_data1.cmath_en {
            self.apply_cmath(bus, dot_col_data1.main_col, dot_col_data1.sub_col, screen_x)
        } else {
            dot_col_data1.main_col
        };
        let dot_col2 = if dot_col_data2.cmath_en {
            self.apply_cmath(bus, dot_col_data2.main_col, dot_col_data2.sub_col, screen_x)
        } else {
            dot_col_data2.main_col
        };

        let dot_col1 = self.apply_brightness(dot_col1, brightness);
        let dot_col2 = self.apply_brightness(dot_col2, brightness);

        if regs.screen_interlace_en {
            let dot1_pixel_idx = 4 * ( (2*screen_y + p) * 512 + (screen_x + 0) );
            let dot2_pixel_idx = 4 * ( (2*screen_y + p) * 512 + (screen_x + 256) );
            
            Ppu5C7x::set_pixel(bus.frame_buffer, dot1_pixel_idx, dot_col1);
            Ppu5C7x::set_pixel(bus.frame_buffer, dot2_pixel_idx, dot_col2);
        } else {
            let top_dot1_pixel_idx = 4 * ( (2*screen_y + 0) * 512 + (screen_x + 0) );
            let bottom_dot1_pixel_idx = 4 * ( (2*screen_y + 1) * 512 + (screen_x + 0) );
            let top_dot2_pixel_idx = 4 * ( (2*screen_y + 0) * 512 + (screen_x + 256) );
            let bottom_dot2_pixel_idx = 4 * ( (2*screen_y + 1) * 512 + (screen_x + 256) );
            
            Ppu5C7x::set_pixel(bus.frame_buffer, top_dot1_pixel_idx, dot_col1);
            Ppu5C7x::set_pixel(bus.frame_buffer, bottom_dot1_pixel_idx, dot_col1);
            Ppu5C7x::set_pixel(bus.frame_buffer, top_dot2_pixel_idx, dot_col2);
            Ppu5C7x::set_pixel(bus.frame_buffer, bottom_dot2_pixel_idx, dot_col2);
        }
    }

    fn apply_brightness(&self, col: Color, brightness: u8) -> Color {
        if brightness == 0 { return Color::BLACK; }
        if brightness == 15 { return col; }

        Color {
            r: (((col.r as u16) * (brightness as u16)) / 15) as u8,
            g: (((col.g as u16) * (brightness as u16)) / 15) as u8,
            b: (((col.b as u16) * (brightness as u16)) / 15) as u8,
        }
    }

    /// Gets the color of the first visible sprite on the screen.
    fn sprite_col(&mut self, bus: &PpuBus, screen_x: usize, screen_y: usize) -> ColorData {
        let regs = &bus.ppu_regs;
        
        let mut scanline_spr_cnt = self.scanline_spr_cnt;

        if scanline_spr_cnt == 0 {
            scanline_spr_cnt = 32;
        }

        for i in 0..self.scanline_sprites.len() {
            scanline_spr_cnt -= 1;

            let sprite = &self.scanline_sprites[scanline_spr_cnt];

            if scanline_spr_cnt == 0 {
                scanline_spr_cnt = 32;
            }

            if sprite.x as usize <= screen_x && screen_x < sprite.max_x as usize {
                let sprite_col = screen_x - sprite.x as usize;
                let sprite_row = screen_y - sprite.y as usize;

                let sprite_row = if regs.screen_interlace_en && regs.obj_interlace_en {
                    2*sprite_row + (self.frame & 1)
                } else {
                    sprite_row
                };

                let sprite_col = if sprite.flip_x { sprite.width - sprite_col - 1 } else { sprite_col };
                let sprite_row = if sprite.flip_y { sprite.height - sprite_row - 1 } else { sprite_row };

                let (tile_x, tile_col) = (sprite_col / 8, sprite_col % 8);
                let (tile_y, tile_row) = (sprite_row / 8, sprite_row % 8);

                let chr_idx = (tile_y << 4) + tile_x;

                let obj_table_base_addr = if sprite.use_second_obj_table {
                    regs.name_base_addr + regs.name_secondary_select
                } else {
                    regs.name_base_addr
                };

                let spr_tile_base_addr = (obj_table_base_addr as u16) + ((sprite.tile_idx as u16) << 4);
                let spr_tile_addr = spr_tile_base_addr + ((chr_idx as u16) << 4);
                let spr_tile_row_addr = spr_tile_addr + tile_row as u16;

                let bp01 = bus.vram[((spr_tile_row_addr as usize) + 0) & 0x7FFF];
                let bp23 = bus.vram[((spr_tile_row_addr as usize) + 8) & 0x7FFF];

                let b0 = ((bp01 >> (7-tile_col)) as u8) & 1;
                let b1 = ((bp01 >> (15-tile_col)) as u8) & 1;
                let b2 = ((bp23 >> (7-tile_col)) as u8) & 1;
                let b3 = ((bp23 >> (15-tile_col)) as u8) & 1;

                let pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;

                // Transparent sprite
                if pal_idx == 0 {
                    // If it's the last sprite, all sprites were transparent
                    if i == self.scanline_sprites.len() - 1 {
                        return ColorData {
                            color: Color::BLACK,
                            priority: sprite.priority,
                            transparent: true,
                        };
                    }

                    continue;
                }

                let cgram_addr = 0x80 | (sprite.palette << 4) | pal_idx;

                let spr_col = bus.cgram[cgram_addr as usize];

                return ColorData {
                    color: spr_col,
                    priority: sprite.priority,
                    transparent: false,
                };
            }
        }

        // No sprites on this dot, return a transparent color
        ColorData {
            color: self.transparent_color(bus),
            priority: 0,
            transparent: true,
        }
    }

    fn bg_mode0_dot(&mut self, bus: &PpuBus, screen_x: usize, screen_y: usize, spr_col: ColorData) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x20;
        const BG3_CGRAM_BASE_ADDR: u8 = 0x40;
        const BG4_CGRAM_BASE_ADDR: u8 = 0x60;
        
        let regs = &bus.ppu_regs;

        let (obj_win_main, obj_win_sub) = win_active_signals!(regs, screen_x, obj);
        let (bg1_win_main, bg1_win_sub) = win_active_signals!(regs, screen_x, bg1);
        let (bg2_win_main, bg2_win_sub) = win_active_signals!(regs, screen_x, bg2);
        let (bg3_win_main, bg3_win_sub) = win_active_signals!(regs, screen_x, bg3);
        let (bg4_win_main, bg4_win_sub) = win_active_signals!(regs, screen_x, bg4);

        let spr_main_col = if regs.obj_main_en && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        let spr_sub_col = if regs.obj_sub_en && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg1_col = self.bg_col(
            bus,
            screen_x, screen_y, 
            ColorLayer::Bg1, ColorDepth::Bpp2,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if regs.bg1_main_en && !bg1_win_main {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg1_sub_col = if regs.bg1_sub_en && !bg1_win_sub {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg2_col = self.bg_col(
            bus,
            screen_x, screen_y, 
            ColorLayer::Bg2, ColorDepth::Bpp2,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if regs.bg2_main_en && !bg2_win_main {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg2_sub_col = if regs.bg2_sub_en && !bg2_win_sub {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg3_col = self.bg_col(
            bus,
            screen_x, screen_y, 
            ColorLayer::Bg3, ColorDepth::Bpp2,
            BG3_CGRAM_BASE_ADDR
        );
        let bg3_main_col = if regs.bg3_main_en && !bg3_win_main {
            bg3_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg3_sub_col = if regs.bg3_sub_en && !bg3_win_sub {
            bg3_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg4_col = self.bg_col(
            bus,
            screen_x, screen_y, 
            ColorLayer::Bg4, ColorDepth::Bpp2,
            BG4_CGRAM_BASE_ADDR
        );
        let bg4_main_col = if regs.bg4_main_en && !bg4_win_main {
            bg4_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg4_sub_col = if regs.bg4_sub_en && !bg4_win_sub {
            bg4_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else if bg4_main_col.priority != 0 && !bg4_main_col.transparent {
            (bg4_main_col.color, ColorLayer::Bg4)
        } else if !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else if !bg4_main_col.transparent {
            (bg4_main_col.color, ColorLayer::Bg4)
        } else {
            (self.transparent_color(bus), ColorLayer::Back)
        };

        let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else if bg4_sub_col.priority != 0 && !bg4_sub_col.transparent {
            bg4_sub_col.color
        } else if !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else if !bg4_sub_col.transparent {
            bg4_sub_col.color
        } else {
            regs.fixed_color
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => regs.bg1_cmath_en,
            ColorLayer::Bg2 => regs.bg2_cmath_en,
            ColorLayer::Bg3 => regs.bg3_cmath_en,
            ColorLayer::Bg4 => regs.bg4_cmath_en,
            ColorLayer::Obj => regs.obj_cmath_en,
            ColorLayer::Back => regs.back_cmath_en,
        };

        DotColorData { 
            main_col,
            sub_col,
            cmath_en
        }
    }
    
    fn bg_mode1_dot(&mut self, bus: &PpuBus, screen_x: usize, screen_y: usize, spr_col: ColorData) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG3_CGRAM_BASE_ADDR: u8 = 0x00;
        
        let regs = &bus.ppu_regs;

        let (obj_win_main, obj_win_sub) = win_active_signals!(regs, screen_x, obj);
        let (bg1_win_main, bg1_win_sub) = win_active_signals!(regs, screen_x, bg1);
        let (bg2_win_main, bg2_win_sub) = win_active_signals!(regs, screen_x, bg2);
        let (bg3_win_main, bg3_win_sub) = win_active_signals!(regs, screen_x, bg3);

        let spr_main_col = if regs.obj_main_en && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        let spr_sub_col = if regs.obj_sub_en && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg1_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg1, ColorDepth::Bpp4,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if regs.bg1_main_en && !bg1_win_main {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg1_sub_col = if regs.bg1_sub_en && !bg1_win_sub {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg2_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg2, ColorDepth::Bpp4,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if regs.bg2_main_en && !bg2_win_main {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg2_sub_col = if regs.bg2_sub_en && !bg2_win_sub {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg3_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg3, ColorDepth::Bpp2,
            BG3_CGRAM_BASE_ADDR
        );
        let bg3_main_col = if regs.bg3_main_en && !bg3_win_main {
            bg3_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg3_sub_col = if regs.bg3_sub_en && !bg3_win_sub {
            bg3_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let (main_col, main_layer) = if regs.bg3_mode1_priority && bg3_main_col.priority != 0 && !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else if !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else {
            (self.transparent_color(bus), ColorLayer::Back)
        };

        let sub_col = if regs.sub_color_fixed {
            regs.fixed_color
        } else if regs.bg3_mode1_priority && bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else if !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else {
            regs.fixed_color
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => regs.bg1_cmath_en,
            ColorLayer::Bg2 => regs.bg2_cmath_en,
            ColorLayer::Bg3 => regs.bg3_cmath_en,
            ColorLayer::Obj => regs.obj_cmath_en,
            ColorLayer::Back => regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 1
        };

        DotColorData{
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode2_dot(&mut self, bus: &PpuBus, screen_x: usize, screen_y: usize, spr_col: ColorData) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        
        let regs = &bus.ppu_regs;

        let (obj_win_main, obj_win_sub) = win_active_signals!(regs, screen_x, obj);
        let (bg1_win_main, bg1_win_sub) = win_active_signals!(regs, screen_x, bg1);
        let (bg2_win_main, bg2_win_sub) = win_active_signals!(regs, screen_x, bg2);

        let spr_main_col = if regs.obj_main_en && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        let spr_sub_col = if regs.obj_sub_en && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg1_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg1, ColorDepth::Bpp4,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if regs.bg1_main_en && !bg1_win_main {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg1_sub_col = if regs.bg1_sub_en && !bg1_win_sub {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg2_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg2, ColorDepth::Bpp4,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if regs.bg2_main_en && !bg2_win_main {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg2_sub_col = if regs.bg2_sub_en && !bg2_win_sub {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else {
            (self.transparent_color(bus), ColorLayer::Back)
        };

        let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else {
            self.transparent_color(bus)
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => regs.bg1_cmath_en,
            ColorLayer::Bg2 => regs.bg2_cmath_en,
            ColorLayer::Obj => regs.obj_cmath_en,
            ColorLayer::Back => regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 2
        };

        DotColorData{
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode3_dot(&mut self, bus: &PpuBus, screen_x: usize, screen_y: usize, spr_col: ColorData) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        
        let regs = &bus.ppu_regs;

        let (obj_win_main, obj_win_sub) = win_active_signals!(regs, screen_x, obj);
        let (bg1_win_main, bg1_win_sub) = win_active_signals!(regs, screen_x, bg1);
        let (bg2_win_main, bg2_win_sub) = win_active_signals!(regs, screen_x, bg2);

        let spr_main_col = if regs.obj_main_en && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        let spr_sub_col = if regs.obj_sub_en && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg1_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg1, ColorDepth::Bpp8,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if regs.bg1_main_en && !bg1_win_main {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg1_sub_col = if regs.bg1_sub_en && !bg1_win_sub {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg2_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg2, ColorDepth::Bpp4,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if regs.bg2_main_en && !bg2_win_main {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg2_sub_col = if regs.bg2_sub_en && !bg2_win_sub {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else {
            (self.transparent_color(bus), ColorLayer::Back)
        };

        let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else {
            self.transparent_color(bus)
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => regs.bg1_cmath_en,
            ColorLayer::Bg2 => regs.bg2_cmath_en,
            ColorLayer::Obj => regs.obj_cmath_en,
            ColorLayer::Back => regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 2
        };

        DotColorData{
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode4_dot(&mut self, bus: &PpuBus, screen_x: usize, screen_y: usize, spr_col: ColorData) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        
        let regs = &bus.ppu_regs;

        let (obj_win_main, obj_win_sub) = win_active_signals!(regs, screen_x, obj);
        let (bg1_win_main, bg1_win_sub) = win_active_signals!(regs, screen_x, bg1);
        let (bg2_win_main, bg2_win_sub) = win_active_signals!(regs, screen_x, bg2);

        let spr_main_col = if regs.obj_main_en && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        let spr_sub_col = if regs.obj_sub_en && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg1_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg1, ColorDepth::Bpp8,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if regs.bg1_main_en && !bg1_win_main {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg1_sub_col = if regs.bg1_sub_en && !bg1_win_sub {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg2_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg2, ColorDepth::Bpp4,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if regs.bg2_main_en && !bg2_win_main {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg2_sub_col = if regs.bg2_sub_en && !bg2_win_sub {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else {
            (self.transparent_color(bus), ColorLayer::Back)
        };

        let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else {
            self.transparent_color(bus)
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => regs.bg1_cmath_en,
            ColorLayer::Bg2 => regs.bg2_cmath_en,
            ColorLayer::Obj => regs.obj_cmath_en,
            ColorLayer::Back => regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 2
        };

        DotColorData{
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode5_dot(&mut self, bus: &PpuBus, screen_x: usize, screen_y: usize, spr_col: ColorData) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        
        let regs = &bus.ppu_regs;

        let (obj_win_main, obj_win_sub) = win_active_signals!(regs, screen_x >> 1, obj);
        let (bg1_win_main, bg1_win_sub) = win_active_signals!(regs, screen_x >> 1, bg1);
        let (bg2_win_main, bg2_win_sub) = win_active_signals!(regs, screen_x >> 1, bg2);

        let spr_main_col = if regs.obj_main_en && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        let spr_sub_col = if regs.obj_sub_en && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg1_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg1, ColorDepth::Bpp4,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if regs.bg1_main_en && !bg1_win_main {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg1_sub_col = if regs.bg1_sub_en && !bg1_win_sub {
            bg1_col
        } else {
            self.transparent_color_data(bus)
        };
        
        let bg2_col = self.bg_col(
            bus,
            screen_x, screen_y,
            ColorLayer::Bg2, ColorDepth::Bpp2,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if regs.bg2_main_en && !bg2_win_main {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };
        let bg2_sub_col = if regs.bg2_sub_en && !bg2_win_sub {
            bg2_col
        } else {
            self.transparent_color_data(bus)
        };

        let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !spr_main_col.transparent {
            (spr_main_col.color, ColorLayer::Obj)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else {
            (self.transparent_color(bus), ColorLayer::Back) // Main screen color is black if all layers are transparent
        };

        let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !spr_sub_col.transparent {
            spr_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else {
            regs.fixed_color // Sub screen color is fixed color if all layers are transparent
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => regs.bg1_cmath_en,
            ColorLayer::Bg2 => regs.bg2_cmath_en,
            ColorLayer::Obj => regs.obj_cmath_en,
            ColorLayer::Back => regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 5
        };

        DotColorData {
            main_col,
            sub_col,
            cmath_en,
        }
    }

    // fn bg_mode6_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {

    // }

    // fn bg_mode7_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {

    // }

    fn bg_col(&self, bus: &PpuBus, screen_x: usize, screen_y: usize, 
        bg_layer: ColorLayer, color_depth: ColorDepth, 
        bg_cgram_base_addr: u8) -> ColorData {

        let bg_chr_base_addr = match bg_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg1_chr_base_addr as u16,
            ColorLayer::Bg2 => bus.ppu_regs.bg2_chr_base_addr as u16,
            ColorLayer::Bg3 => bus.ppu_regs.bg3_chr_base_addr as u16,
            ColorLayer::Bg4 => bus.ppu_regs.bg4_chr_base_addr as u16,

            _ => unreachable!("Should only be called for bg layers")
        };
        
        let tile_data = match bus.ppu_regs.bg_mode {
            BgMode::Mode0
            | BgMode::Mode1
            | BgMode::Mode2
            | BgMode::Mode3
            | BgMode::Mode4 => self.bg_tile_idx(bus, screen_x, screen_y, bg_layer),

            BgMode::Mode5
            | BgMode::Mode6 => self.hi_res_bg_tile_idx(bus, screen_x, screen_y, bg_layer),

            BgMode::Mode7 => todo!(),
        };

        let col = match color_depth {
            ColorDepth::Bpp2 => self.bg_col_2bpp(bus, tile_data, bg_chr_base_addr, bg_cgram_base_addr),
            ColorDepth::Bpp4 =>  self.bg_col_4bpp(bus, tile_data, bg_chr_base_addr, bg_cgram_base_addr),
            ColorDepth::Bpp8 => self.bg_col_8bpp(bus, tile_data, bg_chr_base_addr),
        };

        // let col = match bg_layer {
        //     ColorLayer::Bg1 => {
        //         let (x, col) = (screen_x / 8, screen_x % 8);
        //         let (y, row) = (screen_y / 8, screen_y % 8);

        //         let chr_data = ChrData {
        //             chr_idx: (y*16 + x) as u16,
        //             chr_col: col as u8,
        //             chr_row: row as u8,
        //             chr_pal: 0,
        //             chr_priority: 0,
        //         };

        //         let base_chr_addr = (((self.frame / 15) * 0x100) & (VRAM_SIZE-1)) as u16;

        //         let tile_chr_addr = base_chr_addr + (chr_data.chr_idx << 4) + chr_data.chr_row as u16;

        //         let bp01 = bus.vram[(tile_chr_addr + 0) as usize];
        //         let bp23 = bus.vram[(tile_chr_addr + 8) as usize];

        //         let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
        //         let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;
        //         let b2 = ((bp23 >> (7-chr_data.chr_col)) & 1) as u8;
        //         let b3 = ((bp23 >> (15-chr_data.chr_col)) & 1) as u8;

        //         let pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;
                
        //         let cgram_addr = bg_cgram_base_addr | (chr_data.chr_pal << 4) | pal_idx;

        //         let raw_color = if pal_idx == 0 {
        //             self.transparent_color(bus)
        //         } else {
        //             self.registers.cgram[cgram_addr as usize].get()
        //         };

        //         ColorData {
        //             raw_color,
        //             priority: chr_data.chr_priority,
        //             transparent: pal_idx == 0,
        //         }
        //     }
        //     _ => self.transparent_color_data(bus)
        // };

        col
    }

    /// For modes 0-4
    fn bg_tile_idx(&self, bus: &PpuBus, screen_x: usize, screen_y: usize, bg_layer: ColorLayer) -> TileData {
        let bg_data = self.fetch_bg_data(bus.ppu_regs, bg_layer);

        let (mosaic_x, mosaic_y) = if bg_data.mosaic_en {
            let mosaic_mod = (bus.ppu_regs.mosaic_size + 1) as usize;

            (screen_x - (screen_x % mosaic_mod), screen_y - (screen_y % mosaic_mod))
        } else {
            (screen_x, screen_y)
        };

        let scroll_range = match bg_data.tile_size {
            TileSize::Size8x8 => 0x1FF,
            TileSize::Size16x16 => 0x3FF,
        };

        let shifted_x = ((mosaic_x as u16) + bg_data.scroll_x) & scroll_range;
        let shifted_y = ((mosaic_y as u16) + bg_data.scroll_y) & scroll_range;

        let tilemap_offset = match (bg_data.tilemap_cnt_x, bg_data.tilemap_cnt_y) {
            (TilemapCount::One, TilemapCount::One) => 0x000,
            (TilemapCount::One, TilemapCount::Two) => {
                if shifted_y >= 256 {
                    0x400
                } else {
                    0x000
                }
            }
            (TilemapCount::Two, TilemapCount::One) => {
                if shifted_x >= 256 {
                    0x400
                } else {
                    0x000
                }
            }
            (TilemapCount::Two, TilemapCount::Two) => {
                if shifted_x >= 256 && shifted_y >= 256 {
                    0xC00
                } else if shifted_y >= 256 {
                    0x800
                } else if shifted_x >= 256 {
                    0x400
                } else {
                    0x000
                }
            }
        };

        let x = shifted_x & 0xFF;
        let y = shifted_y & 0xFF;

        let tile_idx = match bg_data.tile_size {
            TileSize::Size8x8 => ((y >> 3) << 5) | (x >> 3),
            TileSize::Size16x16 => (y & 0xF0) | (x >> 4),
        };

        let (tile_col, tile_row) = match bg_data.tile_size {
            TileSize::Size8x8 => (x & 7, y & 7),
            TileSize::Size16x16 => (x & 0xF, y & 0xF),
        };

        // if screen_x == 4 && screen_y == 4 {
        //     match bg_layer {
        //         ColorLayer::Bg1 => {
        //             println!("tile_addr: ${:04X}, tile_row: {}, tile_col: {}",
        //                 bg_data.tilemap_base_addr + tilemap_offset + tile_idx,
        //                 tile_row,
        //                 tile_col,
        //             )
        //         },
        //         _ => {}
        //     }
        // }

        TileData {
            tile_addr: bg_data.tilemap_base_addr + tilemap_offset + tile_idx,
            tile_row: tile_row as u8,
            tile_col: tile_col as u8,
            tile_size: bg_data.tile_size
        }
    }

    /// For modes 5-6
    fn hi_res_bg_tile_idx(&self, bus: &PpuBus, screen_x: usize, screen_y: usize, bg_layer: ColorLayer) -> TileData {
        let bg_data = self.fetch_bg_data(bus.ppu_regs, bg_layer);

        let (mosaic_x, mosaic_y) = if bg_data.mosaic_en {
            let mosaic_mod = (bus.ppu_regs.mosaic_size + 1) as usize;

            (screen_x - (screen_x % mosaic_mod), screen_y - (screen_y % mosaic_mod))
        } else {
            (screen_x, screen_y)
        };

        let scroll_range_x = 0x1FF;
        let scroll_range_y = match bg_data.tile_size {
            TileSize::Size8x8 => 0x1FF,
            TileSize::Size16x16 => 0x3FF,
        };

        let shifted_x = ((mosaic_x as u16) + bg_data.scroll_x) & scroll_range_x;
        let shifted_y = ((mosaic_y as u16) + bg_data.scroll_y) & scroll_range_y;

        let tilemap_offset = match (bg_data.tilemap_cnt_x, bg_data.tilemap_cnt_y) {
            (TilemapCount::One, TilemapCount::One) => 0x000,
            (TilemapCount::One, TilemapCount::Two) => {
                if shifted_y >= 256 {
                    0x400
                } else {
                    0x000
                }
            }
            (TilemapCount::Two, TilemapCount::One) => {
                if shifted_x >= 256 {
                    0x400
                } else {
                    0x000
                }
            }
            (TilemapCount::Two, TilemapCount::Two) => {
                if shifted_x >= 256 && shifted_y >= 256 {
                    0xC00
                } else if shifted_y >= 256 {
                    0x800
                } else if shifted_x >= 256 {
                    0x400
                } else {
                    0x000
                }
            }
        };

        let x = shifted_x & 0x1FF;
        let y = shifted_y & 0xFF;

        let tile_idx = match bg_data.tile_size {
            TileSize::Size8x8 => ((y >> 3) << 5) | (x >> 4),
            TileSize::Size16x16 => (y & 0xF0) | (x >> 4),
        };

        let (tile_col, tile_row) = match bg_data.tile_size {
            TileSize::Size8x8 => (x & 0xF, y & 7),
            TileSize::Size16x16 => (x & 0xF, y & 0xF),
        };

        TileData {
            tile_addr: bg_data.tilemap_base_addr + tilemap_offset + tile_idx,
            tile_row: tile_row as u8,
            tile_col: tile_col as u8,
            tile_size: bg_data.tile_size
        }
    }

    fn fetch_bg_data(&self, regs: &PpuRegs, bg_layer: ColorLayer) -> BgData {
        match bg_layer {
            ColorLayer::Bg1 => BgData {
                scroll_x: regs.bg1_m7_x_offset,
                scroll_y: regs.bg1_m7_y_offset,
                tilemap_cnt_x: regs.bg1_tilemap_count_x,
                tilemap_cnt_y: regs.bg1_tilemap_count_y,
                tile_size: regs.bg1_char_size,
                tilemap_base_addr: (regs.bg1_vram_addr as u16) << 10,
                mosaic_en: regs.bg1_mosaic_en
            },

            ColorLayer::Bg2 => BgData {
                scroll_x: regs.bg2_x_offset,
                scroll_y: regs.bg2_y_offset,
                tilemap_cnt_x: regs.bg2_tilemap_count_x,
                tilemap_cnt_y: regs.bg2_tilemap_count_y,
                tile_size: regs.bg2_char_size,
                tilemap_base_addr: (regs.bg2_vram_addr as u16) << 10,
                mosaic_en: regs.bg2_mosaic_en
            },

            ColorLayer::Bg3 => BgData {
                scroll_x: regs.bg3_x_offset,
                scroll_y: regs.bg3_y_offset,
                tilemap_cnt_x: regs.bg3_tilemap_count_x,
                tilemap_cnt_y: regs.bg3_tilemap_count_y,
                tile_size: regs.bg3_char_size,
                tilemap_base_addr: (regs.bg3_vram_addr as u16) << 10,
                mosaic_en: regs.bg3_mosaic_en
            },

            ColorLayer::Bg4 => BgData {
                scroll_x: regs.bg4_x_offset,
                scroll_y: regs.bg4_y_offset,
                tilemap_cnt_x: regs.bg4_tilemap_count_x,
                tilemap_cnt_y: regs.bg4_tilemap_count_y,
                tile_size: regs.bg4_char_size,
                tilemap_base_addr: (regs.bg4_vram_addr as u16) << 10,
                mosaic_en: regs.bg4_mosaic_en
            },

            _ => unreachable!() // Only called for bg layers
        }
    }

    fn fetch_chr_data(&self, bus: &PpuBus, tile_data: TileData) -> ChrData {
        let tile_word = bus.vram[(tile_data.tile_addr) as usize];

        let in_true_hi_res_mode = match bus.ppu_regs.bg_mode {
            BgMode::Mode5 | BgMode::Mode6 => true,
            _ => false,
        };
        
        let (tile_height, tile_width) = match tile_data.tile_size {
            TileSize::Size8x8 => (8, if in_true_hi_res_mode { 16 } else { 8 }),
            TileSize::Size16x16 => (16,16),
        };

        let tile_chr_idx = tile_word & 0x3FF;
        let tile_pal = ((tile_word >> 10) & 7) as u8;
        let tile_priority = ((tile_word >> 13) & 1) as u8;
        let flip_x = (tile_word & 0x4000) != 0;
        let flip_y = (tile_word & 0x8000) != 0;

        let tile_row = if flip_y { tile_height - tile_data.tile_row - 1 } else { tile_data.tile_row };
        let tile_col = if flip_x { tile_width - tile_data.tile_col - 1 } else { tile_data.tile_col };

        let (tile_chr_idx, tile_row) = if tile_row > 7 {
            (tile_chr_idx + 32, tile_row - 8)
        } else {
            (tile_chr_idx, tile_row)
        };

        let (tile_chr_idx, tile_col) = if tile_col > 7 {
            (tile_chr_idx + 1, tile_col - 8)
        } else {
            (tile_chr_idx, tile_col)
        };

        ChrData {
            chr_idx: tile_chr_idx,
            chr_col: tile_col,
            chr_row: tile_row,
            chr_pal: tile_pal,
            chr_priority: tile_priority,
        }
    }

    fn bg_col_2bpp(&self, bus: &PpuBus, tile_data: TileData, bg_chr_base_addr: u16, bg_cgram_base_addr: u8) -> ColorData {        
        let chr_data = self.fetch_chr_data(bus, tile_data);

        let tile_chr_addr = bg_chr_base_addr + (chr_data.chr_idx << 3) + chr_data.chr_row as u16;

        let bp01 = bus.vram[(tile_chr_addr) as usize];

        let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
        let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;

        let pal_idx = (b1 << 1) | b0;
        
        let cgram_addr = bg_cgram_base_addr | (chr_data.chr_pal << 2) | pal_idx;

        let color = if pal_idx == 0 {
            self.transparent_color(bus)
        } else {
            bus.cgram[cgram_addr as usize]
        };

        ColorData {
            color,
            priority: chr_data.chr_priority,
            transparent: pal_idx == 0,
        }
    }

    fn bg_col_4bpp(&self, bus: &PpuBus, tile_data: TileData, bg_chr_base_addr: u16, bg_cgram_base_addr: u8) -> ColorData {
        let chr_data = self.fetch_chr_data(bus, tile_data);

        let tile_chr_addr = bg_chr_base_addr + (chr_data.chr_idx << 4) + chr_data.chr_row as u16;

        let bp01 = bus.vram[(tile_chr_addr + 0) as usize];
        let bp23 = bus.vram[(tile_chr_addr + 8) as usize];

        let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
        let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;
        let b2 = ((bp23 >> (7-chr_data.chr_col)) & 1) as u8;
        let b3 = ((bp23 >> (15-chr_data.chr_col)) & 1) as u8;

        let pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;
        
        let cgram_addr = bg_cgram_base_addr | (chr_data.chr_pal << 4) | pal_idx;

        let color = if pal_idx == 0 {
            self.transparent_color(bus)
        } else {
            bus.cgram[cgram_addr as usize]
        };

        ColorData {
            color,
            priority: chr_data.chr_priority,
            transparent: pal_idx == 0,
        }
    }

    fn bg_col_8bpp(&self, bus: &PpuBus, tile_data: TileData, bg_chr_base_addr: u16) -> ColorData {
        let chr_data = self.fetch_chr_data(bus, tile_data);

        let tile_chr_addr = bg_chr_base_addr + (chr_data.chr_idx << 5) + chr_data.chr_row as u16;

        if !bus.ppu_regs.use_direct_col {
            let bp01 = bus.vram[(tile_chr_addr + 0) as usize];
            let bp23 = bus.vram[(tile_chr_addr + 8) as usize];
            let bp45 = bus.vram[(tile_chr_addr + 16) as usize];
            let bp67 = bus.vram[(tile_chr_addr + 24) as usize];
    
            let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
            let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;
            let b2 = ((bp23 >> (7-chr_data.chr_col)) & 1) as u8;
            let b3 = ((bp23 >> (15-chr_data.chr_col)) & 1) as u8;
            let b4 = ((bp45 >> (7-chr_data.chr_col)) & 1) as u8;
            let b5 = ((bp45 >> (15-chr_data.chr_col)) & 1) as u8;
            let b6 = ((bp67 >> (7-chr_data.chr_col)) & 1) as u8;
            let b7 = ((bp67 >> (15-chr_data.chr_col)) & 1) as u8;
    
            let cgram_addr = (b7 << 7) | (b6 << 6) | (b5 << 5) | (b4 << 4) | (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;
    
            let color = if cgram_addr == 0 {
                self.transparent_color(bus)
            } else {
                bus.cgram[cgram_addr as usize]
            };
    
            ColorData {
                color,
                priority: chr_data.chr_priority,
                transparent: cgram_addr == 0,
            }
        } else {
            let r_ext = (chr_data.chr_pal & 0x4) >> 1;
            let g_ext = (chr_data.chr_pal & 0x8) >> 2;
            let b_ext = (chr_data.chr_pal & 0x10) >> 2;

            let rgb_data = bus.vram[(tile_chr_addr + chr_data.chr_col as u16) as usize] as u8;

            let r = ((rgb_data & 0x7) << 2) | r_ext;
            let g = ((rgb_data & 0x38) >> 1) | g_ext;
            let b = ((rgb_data & 0xC0) >> 3) | b_ext;

            let color = Color::new(r, g, b);

            ColorData {
                color,
                priority: chr_data.chr_priority,
                transparent: (r == 0) && (g == 0) && (b == 0),
            }
        }
    }

    fn apply_cmath(&self, bus: &PpuBus, main_col: Color, sub_col: Color, screen_x: usize) -> Color {
        let col_win_en = Ppu5C7x::col_win_active_signal(bus.ppu_regs, screen_x);

        let main_col = match bus.ppu_regs.col_win_main_region {
            WindowColorRegion::Nowhere => main_col,
            WindowColorRegion::Outside => if col_win_en { main_col } else { Color::BLACK },
            WindowColorRegion::Inside => if col_win_en { Color::BLACK } else { main_col },
            WindowColorRegion::Everywhere => { Color::BLACK }
        };
        let sub_col = match bus.ppu_regs.col_win_sub_region {
            WindowColorRegion::Nowhere => sub_col,
            WindowColorRegion::Outside => if col_win_en { sub_col } else { self.transparent_color(bus) },
            WindowColorRegion::Inside => if col_win_en { self.transparent_color(bus) } else { sub_col },
            WindowColorRegion::Everywhere => { self.transparent_color(bus) }
        };

        let sub_col = if bus.ppu_regs.sub_color_fixed {
            bus.ppu_regs.fixed_color
        } else {
            sub_col
        };
        
        let mut color = match bus.ppu_regs.cmath_operator {
            CMathOperator::Add => Color {
                r: main_col.r.saturating_add(sub_col.r) & 0xF8, // 5-bit color max
                g: main_col.g.saturating_add(sub_col.g) & 0xF8,
                b: main_col.b.saturating_add(sub_col.b) & 0xF8,
            },
            CMathOperator::Subtract => Color {
                r: main_col.r.saturating_sub(sub_col.r),
                g: main_col.g.saturating_sub(sub_col.g),
                b: main_col.b.saturating_sub(sub_col.b),
            },
        };
        
        if bus.ppu_regs.cmath_half {
            color.r = (color.r >> 1) & 0xF8;
            color.g = (color.g >> 1) & 0xF8;
            color.b = (color.b >> 1) & 0xF8;
        }

        color
    }
    
    fn win_active_signal(regs: &PpuRegs, screen_x: usize, layer_w1_en: bool, layer_w2_en: bool,
        layer_w1_inv: bool, layer_w2_inv: bool, win_logic: WindowLogic) -> bool {

        let w1_left = regs.w1_left_pos as usize;
        let w1_right = regs.w1_right_pos as usize;
        let w2_left = regs.w2_left_pos as usize;
        let w2_right = regs.w2_right_pos as usize;

        let in_w1 = w1_left <= screen_x && screen_x <= w1_right;
        let in_w2 = w2_left <= screen_x && screen_x <= w2_right;

        let w1_en = (layer_w1_en && in_w1) ^ layer_w1_inv;
        let w2_en = (layer_w2_en && in_w2) ^ layer_w2_inv;

        let win_en = if layer_w1_en && layer_w2_en {
            match win_logic {
                WindowLogic::Or => w1_en || w2_en,
                WindowLogic::And => w1_en && w2_en,
                WindowLogic::Xor => w1_en ^ w2_en,
                WindowLogic::Xnor => !(w1_en ^ w2_en),
            }
        } else if layer_w1_en {
            w1_en
        } else if layer_w2_en {
            w2_en
        } else {
            false
        };

        win_en
    }

    fn col_win_active_signal(regs: &PpuRegs, screen_x: usize) -> bool {
        let win_en = Ppu5C7x::win_active_signal(
            regs,
            screen_x,
            regs.col_w1_en,
            regs.col_w2_en,
            regs.col_w1_inv,
            regs.col_w2_inv,
            regs.col_win_logic
        );

        win_en
    }
    
    fn transparent_color(&self, bus: &PpuBus) -> Color {
        bus.cgram[0]
    }
    
    fn transparent_color_data(&self, bus: &PpuBus) -> ColorData { 
        ColorData {
            color: bus.cgram[0],
            priority: 0,
            transparent: true,
        }
    }
    
    fn update_dot_and_scanline(&mut self, bus: &mut PpuBus) {
        let cpu_regs = &mut bus.cpu_regs;
        
        self.dot += 1;

        if self.dot == SCANLINE_END_DOT {
            self.dot = 0;
            self.scanline += 1;

            if self.scanline == VBLANK_END_SCANLINE_NTSC {
                self.scanline = 0;
            }
        }

        // End of v-blank, scanline 0 is not visible
        if self.dot == 0 && self.scanline == 0 {
            cpu_regs.vblank_flag = false;
            cpu_regs.vblank_nmi_flag = false;
        }

        // End of h-blank
        if self.dot == VISIBLE_SCANLINE_START_DOT {
            cpu_regs.hblank_flag = false;

            // Start of visible scanline
            if 0 < self.scanline && self.scanline < VBLANK_START_SCANLINE {
                self.find_scanline_sprites(bus);
            }
        }
        
        let cpu_regs = &mut bus.cpu_regs; // repeat to appease the borrow checker

        // Start of h-blank
        if self.dot == HBLANK_START_DOT && self.scanline < VBLANK_START_SCANLINE {
            cpu_regs.hblank_flag = true;
        }

        // Start of v-blank
        if self.dot == 0 && self.scanline == VBLANK_START_SCANLINE {
            cpu_regs.vblank_flag = true;
            
            if cpu_regs.vblank_nmi_en {
                cpu_regs.vblank_nmi_flag = true;
                bus.trigger_interrupt(CpuInterrupt::NMI);
            }
            
            self.frame += 1;
            bus.set_frame_finished();
        }
    }
    
    fn update_hv_timers(&self, bus: &mut PpuBus) {
        let ppu_regs = &mut bus.ppu_regs;
        let cpu_regs = &mut bus.cpu_regs;
        
        ppu_regs.h_counter = self.dot as u16;
        ppu_regs.v_counter = self.scanline as u16;

        match cpu_regs.hv_timer_irq_mode {
            HVTimerIRQ::None => {}
            HVTimerIRQ::HTimer => {
                if ppu_regs.h_counter == cpu_regs.h_counter_target {
                    cpu_regs.hv_timer_irq_flag = true;
                    bus.trigger_interrupt(CpuInterrupt::IRQ);
                }
            }
            HVTimerIRQ::VTimer => {
                if ppu_regs.v_counter == cpu_regs.v_counter_target && ppu_regs.h_counter == 0 {
                    cpu_regs.hv_timer_irq_flag = true;
                    bus.trigger_interrupt(CpuInterrupt::IRQ);
                }
            }
            HVTimerIRQ::Both => {
                if ppu_regs.v_counter == cpu_regs.v_counter_target && ppu_regs.h_counter == cpu_regs.h_counter_target {
                    cpu_regs.hv_timer_irq_flag = true;
                    bus.trigger_interrupt(CpuInterrupt::IRQ);
                }
            }
        }
    }
    
    /// Finds all possible sprites that could be rendered on the current scanline
    /// based on the y-positions of the sprites
    fn find_scanline_sprites(&mut self, bus: &mut PpuBus) {
        let regs = &mut bus.ppu_regs;
        
        self.scanline_sprites.clear();

        let screen_y = if regs.screen_interlace_en && regs.obj_interlace_en {
            2*self.screen_y() + (self.frame & 1)
        } else {
            self.screen_y()
        };

        self.scanline_spr_cnt = 0;
        for (spr_idx, spr_data) in bus.oam[..0x200].chunks(4).enumerate().rev() {
            // This bit munging is absolutely horrifying but works. We need to 1) get the packed byte containing
            // our data, 2) create a mask to get the bits within the packed byte, and 3) or the byte with the
            // mask to get the relevant bits. Each byte looks like DdCcBbAa, with each letter pair corresponding
            // to a single sprite (32 bytes * 4 pairs = 128, matching # of sprites in OAM).
            let spr_extra_data = (bus.oam[0x200 | (spr_idx >> 2)] >> ((spr_idx & 3) << 1)) & 3;
            let spr_size_sel = (spr_extra_data & 2) != 0;
            let spr_size = if spr_size_sel {
                match regs.obj_sprite_size {
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
                match regs.obj_sprite_size {
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
            let (spr_w, spr_h) = match spr_size {
                ObjectSize::Size8x8 => (8, 8),
                ObjectSize::Size16x16 => (16, 16),
                ObjectSize::Size16x32 => (16, 32),
                ObjectSize::Size32x32 => (32, 32),
                ObjectSize::Size32x64 => (32, 64),
                ObjectSize::Size64x64 => (64, 64),
            };
            let spr_y = spr_data[1];
            let spr_x = (((spr_extra_data as u16) & 1) << 8) | (spr_data[0] as u16);
            let (spr_x_max, spr_y_max) = match spr_size {
                ObjectSize::Size8x8 => (spr_x + 8, spr_y + 8),
                ObjectSize::Size16x16 => (spr_x + 16, spr_y + 16),
                ObjectSize::Size16x32 | ObjectSize::Size32x32 => (spr_x + 32, spr_y + 32),
                ObjectSize::Size32x64 | ObjectSize::Size64x64 => (spr_x + 64, spr_y + 64),
            };

            // Sprite should be on scanline
            if spr_y as usize <= screen_y && screen_y < spr_y_max as usize  {
                let sprite = OAMSprite {
                    x: spr_x,
                    max_x: spr_x_max,
                    y: spr_y,
                    tile_idx: spr_data[2],
                    use_second_obj_table: (spr_data[3] & 1) != 0,
                    palette: (spr_data[3] >> 1) & 7,
                    priority: (spr_data[3] >> 4) & 3,
                    flip_x: (spr_data[3] & 0x40) != 0,
                    flip_y: (spr_data[3] & 0x80) != 0,
                    width: spr_w,
                    height: spr_h,
                };

                if self.scanline_sprites.len() < 32 {
                    self.scanline_sprites.push(sprite);
                } else {
                    self.scanline_sprites[self.scanline_spr_cnt] = sprite;
                }

                self.scanline_spr_cnt = (self.scanline_spr_cnt + 1) & 0x1F;
            }
        }
    }
    
    fn screen_x(&self) -> usize {
        self.dot - VISIBLE_SCANLINE_START_DOT
    }
    
    fn screen_y(&self) -> usize {
        self.scanline - 1
    }
}
