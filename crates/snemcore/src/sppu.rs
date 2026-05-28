use std::marker::PhantomData;

use crate::probe::DebugProbe;
use crate::scpu::ioregs::HVTimerIRQ;
use crate::scpu::CpuInterrupt;
use crate::sppu::bus::PpuBus;
use crate::sppu::regs::PpuRegs;
use crate::sppu::utils::{interleave_2bpp, interleave_4bpp, interleave_8bpp};

pub use crate::sppu::color::Color;
pub use crate::sppu::types::*;

pub mod bus;
pub mod color;
pub mod regs;
pub mod types;

#[macro_use]
mod utils;

pub const VBLANK_START_SCANLINE: usize = 225;
const VBLANK_END_SCANLINE_NTSC: usize = 261;
const VISIBLE_SCANLINE_START_DOT: usize = 22;
pub const HBLANK_START_DOT: usize = 278;
const SCANLINE_END_DOT: usize = 340;

const TILE_CACHE_SIZE: usize = 1;

pub struct Ppu5C7x<P: DebugProbe> {
    pub dot: usize,
    pub scanline: usize,
    /// x position of the current dot on the screen
    pub x: usize,
    /// y position of the current scanline on the screen
    pub y: usize,
    pub frame: usize,

    in_w1: bool,
    in_w2: bool,

    scanline_sprites: Vec<OAMSprite>,
    scanline_spr_cnt: usize,

    bg_tile_cache: [TileRowCache<TILE_CACHE_SIZE>; 4],

    pub bg_tile_cache_hits: usize,
    pub bg_tile_cache_accesses: usize,

    /// Number of master clocks until the next dot
    pub clocks: usize,

    _phantom_probe: PhantomData<P>,
}

impl<P: DebugProbe> Ppu5C7x<P> {
    pub fn new() -> Self {
        let mut ppu = Self {
            dot: 0,
            scanline: 0,
            x: 0,
            y: 0,
            frame: 0,
            in_w1: false,
            in_w2: false,
            scanline_sprites: Vec::new(),
            scanline_spr_cnt: 0,
            bg_tile_cache: std::array::repeat(TileRowCache::new()),
            bg_tile_cache_hits: 0,
            bg_tile_cache_accesses: 0,
            clocks: 0,
            _phantom_probe: PhantomData {},
        };

        ppu.x = ppu.screen_x();
        ppu.y = ppu.screen_y();

        ppu
    }

    pub fn power_on(&mut self) {
        self.dot = 0;
        self.scanline = 0;
        self.x = self.screen_x();
        self.y = self.screen_y();
        self.frame = 0;
        self.scanline_sprites.clear();
        self.scanline_spr_cnt = 0;
        self.clocks = 0;
        self.in_w1 = false;
        self.in_w2 = false;

        for c in self.bg_tile_cache.iter_mut() {
            for t in c.entries.iter_mut() {
                t.invalidate();
            }
        }
    }

    pub fn reset(&mut self) {
        self.power_on();
    }

    /// Cycles the PPU for a certain number of master clocks
    pub fn cycle(&mut self, bus: &mut PpuBus) {
        if self.x < 256 && self.y < 224 {
            self.draw_dot(bus);
        }

        self.update_dot_and_scanline(bus);
        self.update_hv_timers(bus);

        self.clocks += 4;

        if self.dot >= SCANLINE_END_DOT - 4 {
            self.clocks += 1;
        }
    }

    pub fn cycle_no_output(&mut self, bus: &mut PpuBus) {
        self.update_dot_and_scanline(bus);
        self.update_hv_timers(bus);

        self.clocks += 4;

        if self.dot >= SCANLINE_END_DOT - 4 {
            self.clocks += 1;
        }
    }

    fn draw_dot(&mut self, bus: &mut PpuBus) {
        match bus.ppu_regs.bg_mode {
            BgMode::Mode0 | BgMode::Mode1 | BgMode::Mode2 | BgMode::Mode3 | BgMode::Mode4 => {
                self.draw_dot_modes_0to4(bus)
            }

            // BgMode::Mode5 | BgMode::Mode6 => self.draw_dot_modes_5to6(bus),

            // BgMode::Mode7 => self.bg_mode7_dot(self.x, self.y, spr_col),
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

        let dot_col_data = if regs.in_fblank {
            DotColorData {
                main_col: Color::BLACK,
                sub_col: Color::BLACK,
                cmath_en: false,
            }
        } else {
            match regs.bg_mode {
                BgMode::Mode0 => self.bg_mode0_dot(bus),
                BgMode::Mode1 => self.bg_mode1_dot(bus),
                BgMode::Mode2 => self.bg_mode2_dot(bus),
                BgMode::Mode3 => self.bg_mode3_dot(bus),
                BgMode::Mode4 => self.bg_mode4_dot(bus),
                _ => DotColorData {
                    main_col: Color::BLACK,
                    sub_col: Color::BLACK,
                    cmath_en: false,
                },
            }
        };

        let main_col = dot_col_data.main_col;
        let sub_col = dot_col_data.sub_col;
        let cmath_en = dot_col_data.cmath_en;

        let brightness = regs.screen_brightness;
        let p = self.frame & 1;

        if regs.screen_interlace_en && regs.hi_res_en {
            let main_col = if cmath_en {
                self.apply_cmath(bus, main_col, regs.fixed_color)
            } else {
                main_col
            };
            let sub_col = if cmath_en {
                self.apply_cmath(bus, sub_col, regs.fixed_color)
            } else {
                sub_col
            };
            let main_col = self.apply_brightness(main_col, brightness);
            let sub_col = self.apply_brightness(sub_col, brightness);

            let main_pixel_idx = 4 * ((2 * self.y + p) * 512 + (2 * self.x + 0));
            let sub_pixel_idx = 4 * ((2 * self.y + p) * 512 + (2 * self.x + 1));

            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, main_pixel_idx, main_col);
            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, sub_pixel_idx, sub_col);
        } else if regs.screen_interlace_en {
            let dot_col = if cmath_en {
                self.apply_cmath(bus, main_col, sub_col)
            } else {
                main_col
            };
            let dot_col = self.apply_brightness(dot_col, brightness);

            let left_pixel_idx = 4 * ((2 * self.y + p) * 512 + (2 * self.x + 0));
            let right_pixel_idx = 4 * ((2 * self.y + p) * 512 + (2 * self.x + 1));

            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, left_pixel_idx, dot_col);
            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, right_pixel_idx, dot_col);
        } else if regs.hi_res_en {
            let main_col = if cmath_en {
                self.apply_cmath(bus, main_col, regs.fixed_color)
            } else {
                main_col
            };
            let sub_col = if cmath_en {
                self.apply_cmath(bus, sub_col, regs.fixed_color)
            } else {
                sub_col
            };
            let main_col = self.apply_brightness(main_col, brightness);
            let sub_col = self.apply_brightness(sub_col, brightness);

            let main_pixel_idx = 4 * ((2 * self.y + p) * 512 + (2 * self.x + 0));
            let sub_pixel_idx = 4 * ((2 * self.y + p) * 512 + (2 * self.x + 1));

            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, main_pixel_idx, main_col);
            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, sub_pixel_idx, sub_col);
        } else {
            let dot_col = if cmath_en {
                self.apply_cmath(bus, main_col, sub_col)
            } else {
                main_col
            };
            let dot_col = self.apply_brightness(dot_col, brightness);

            let top_left_pixel_idx = 4 * ((2 * self.y + 0) * 512 + (2 * self.x + 0));
            let top_right_pixel_idx = 4 * ((2 * self.y + 0) * 512 + (2 * self.x + 1));
            let bottom_left_pixel_idx = 4 * ((2 * self.y + 1) * 512 + (2 * self.x + 0));
            let bottom_right_pixel_idx = 4 * ((2 * self.y + 1) * 512 + (2 * self.x + 1));

            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, top_left_pixel_idx, dot_col);
            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, top_right_pixel_idx, dot_col);
            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, bottom_left_pixel_idx, dot_col);
            Ppu5C7x::<P>::set_pixel(bus.frame_buffer, bottom_right_pixel_idx, dot_col);
        }
    }

    fn draw_dot_modes_5to6(&mut self, bus: &mut PpuBus) {
        todo!();
    }

    fn apply_brightness(&self, col: Color, brightness: u8) -> Color {
        if brightness == 0 {
            return Color::BLACK;
        }
        if brightness == 15 {
            return col;
        }

        Color {
            r: (((col.r as u16) * (brightness as u16)) / 15) as u8,
            g: (((col.g as u16) * (brightness as u16)) / 15) as u8,
            b: (((col.b as u16) * (brightness as u16)) / 15) as u8,
        }
    }

    /// Gets the color of the first visible sprite on the screen.
    fn sprite_col(&mut self, bus: &PpuBus) -> ColorData {
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

            if sprite.x as usize <= self.x && self.x < sprite.max_x as usize {
                let sprite_col = self.x - sprite.x as usize;
                let sprite_row = self.y - sprite.y as usize;

                let sprite_row = if regs.screen_interlace_en && regs.obj_interlace_en {
                    2 * sprite_row + (self.frame & 1)
                } else {
                    sprite_row
                };

                let sprite_col = if sprite.flip_x {
                    sprite.width - sprite_col - 1
                } else {
                    sprite_col
                };
                let sprite_row = if sprite.flip_y {
                    sprite.height - sprite_row - 1
                } else {
                    sprite_row
                };

                let (tile_x, tile_col) = (sprite_col / 8, sprite_col % 8);
                let (tile_y, tile_row) = (sprite_row / 8, sprite_row % 8);

                let chr_idx = (tile_y << 4) + tile_x;

                let obj_table_base_addr = if sprite.use_second_obj_table {
                    regs.name_secondary_base_addr
                } else {
                    regs.name_base_addr
                };

                let obj_table_base_addr = obj_table_base_addr; // No longer mutable

                let spr_tile_base_addr =
                    (obj_table_base_addr as u16) + ((sprite.tile_idx as u16) << 4);
                let spr_tile_addr = spr_tile_base_addr + ((chr_idx as u16) << 4);
                let spr_tile_row_addr = spr_tile_addr + tile_row as u16;

                let bp01 = bus.vram[((spr_tile_row_addr as usize) + 0) & 0x7FFF];
                let bp23 = bus.vram[((spr_tile_row_addr as usize) + 8) & 0x7FFF];

                let b0 = ((bp01 >> (7 - tile_col)) as u8) & 1;
                let b1 = ((bp01 >> (15 - tile_col)) as u8) & 1;
                let b2 = ((bp23 >> (7 - tile_col)) as u8) & 1;
                let b3 = ((bp23 >> (15 - tile_col)) as u8) & 1;

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
            color: bus.cgram[0],
            priority: 0,
            transparent: true,
        }
    }

    fn bg_mode0_dot(&mut self, bus: &PpuBus) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x20;
        const BG3_CGRAM_BASE_ADDR: u8 = 0x40;
        const BG4_CGRAM_BASE_ADDR: u8 = 0x60;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;
        const BG3_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;
        const BG4_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;

        let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
        let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
            bus,
            BG1_COL_DEPTH,
            BG1_CGRAM_BASE_ADDR,
            ColorLayer::Bg1
        );
        let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
            bus,
            BG2_COL_DEPTH,
            BG2_CGRAM_BASE_ADDR,
            ColorLayer::Bg2
        );
        let (bg3_main_col, bg3_sub_col) = self.bg_layer_colors(
            bus,
            BG3_COL_DEPTH,
            BG3_CGRAM_BASE_ADDR,
            ColorLayer::Bg3
        );
        let (bg4_main_col, bg4_sub_col) = self.bg_layer_colors(
            bus,
            BG4_COL_DEPTH,
            BG4_CGRAM_BASE_ADDR,
            ColorLayer::Bg4
        );

        let (main_col, main_layer) = if obj_main_col.priority == 3 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else if bg4_main_col.priority != 0 && !bg4_main_col.transparent {
            (bg4_main_col.color, ColorLayer::Bg4)
        } else if !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else if !bg4_main_col.transparent {
            (bg4_main_col.color, ColorLayer::Bg4)
        } else {
            (bus.cgram[0], ColorLayer::Back)
        };

        let sub_col = if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else if bg4_sub_col.priority != 0 && !bg4_sub_col.transparent {
            bg4_sub_col.color
        } else if !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else if !bg4_sub_col.transparent {
            bg4_sub_col.color
        } else {
            bus.ppu_regs.fixed_color
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Bg3 => bus.ppu_regs.bg_settings[2].cmath_en,
            ColorLayer::Bg4 => bus.ppu_regs.bg_settings[3].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
        };

        DotColorData {
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode1_dot(&mut self, bus: &PpuBus) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG3_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
        const BG3_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;

        let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
        let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
            bus,
            BG1_COL_DEPTH,
            BG1_CGRAM_BASE_ADDR,
            ColorLayer::Bg1
        );
        let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
            bus,
            BG2_COL_DEPTH,
            BG2_CGRAM_BASE_ADDR,
            ColorLayer::Bg2
        );
        let (bg3_main_col, bg3_sub_col) = self.bg_layer_colors(
            bus,
            BG3_COL_DEPTH,
            BG3_CGRAM_BASE_ADDR,
            ColorLayer::Bg3
        );

        let (main_col, main_layer) = if bus.ppu_regs.bg3_mode1_priority
            && bg3_main_col.priority != 0
            && !bg3_main_col.transparent
        {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else if obj_main_col.priority == 3 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else if !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg3_main_col.transparent {
            (bg3_main_col.color, ColorLayer::Bg3)
        } else {
            (bus.cgram[0], ColorLayer::Back)
        };

        let sub_col = if bus.ppu_regs.sub_color_fixed {
            bus.ppu_regs.fixed_color
        } else if bus.ppu_regs.bg3_mode1_priority
            && bg3_sub_col.priority != 0
            && !bg3_sub_col.transparent
        {
            bg3_sub_col.color
        } else if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else if !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg3_sub_col.transparent {
            bg3_sub_col.color
        } else {
            bus.ppu_regs.fixed_color
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Bg3 => bus.ppu_regs.bg_settings[2].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 1
        };

        DotColorData {
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode2_dot(&mut self, bus: &PpuBus) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;

        let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
        let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
            bus,
            BG1_COL_DEPTH,
            BG1_CGRAM_BASE_ADDR,
            ColorLayer::Bg1
        );
        let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
            bus,
            BG2_COL_DEPTH,
            BG2_CGRAM_BASE_ADDR,
            ColorLayer::Bg2
        );

        let (main_col, main_layer) = if obj_main_col.priority == 3 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else {
            (bus.cgram[0], ColorLayer::Back)
        };

        let sub_col = if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else {
            bus.cgram[0]
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 2
        };

        DotColorData {
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode3_dot(&mut self, bus: &PpuBus) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp8;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;

        let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
        let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
            bus,
            BG1_COL_DEPTH,
            BG1_CGRAM_BASE_ADDR,
            ColorLayer::Bg1
        );
        let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
            bus,
            BG2_COL_DEPTH,
            BG2_CGRAM_BASE_ADDR,
            ColorLayer::Bg2
        );

        let (main_col, main_layer) = if obj_main_col.priority == 3 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else {
            (bus.cgram[0], ColorLayer::Back)
        };

        let sub_col = if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else {
            bus.cgram[0]
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 2
        };

        DotColorData {
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode4_dot(&mut self, bus: &PpuBus) -> DotColorData {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp8;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;

        let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
        let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
            bus,
            BG1_COL_DEPTH,
            BG1_CGRAM_BASE_ADDR,
            ColorLayer::Bg1
        );
        let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
            bus,
            BG2_COL_DEPTH,
            BG2_CGRAM_BASE_ADDR,
            ColorLayer::Bg2
        );

        let (main_col, main_layer) = if obj_main_col.priority == 3 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.color, ColorLayer::Bg1)
        } else if !obj_main_col.transparent {
            (obj_main_col.color, ColorLayer::Obj)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.color, ColorLayer::Bg2)
        } else {
            (bus.cgram[0], ColorLayer::Back)
        };

        let sub_col = if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.color
        } else if !obj_sub_col.transparent {
            obj_sub_col.color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.color
        } else {
            bus.cgram[0]
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
            _ => unreachable!(), // No other layers considered in Mode 2
        };

        DotColorData {
            main_col,
            sub_col,
            cmath_en,
        }
    }

    fn bg_mode5_dot(&mut self, bus: &PpuBus) -> DotColorData {
        todo!()
    }

    fn bg_mode6_dot(&mut self, bus: &PpuBus) -> DotColorData {
        todo!()
    }

    fn bg_mode7_dot(&mut self, bus: &PpuBus) -> DotColorData {
        todo!()
    }

    fn bg_col(
        &mut self,
        bus: &PpuBus,
        bg_layer: ColorLayer,
        color_depth: ColorDepth,
        bg_cgram_base_addr: u8,
    ) -> ColorData {
        let bg_data = Self::fetch_bg_data(bus.ppu_regs, bg_layer);

        let tile_data = match bus.ppu_regs.bg_mode {
            BgMode::Mode0 | BgMode::Mode1 | BgMode::Mode2 | BgMode::Mode3 | BgMode::Mode4 => {
                self.bg_tile_idx(bus.ppu_regs, bg_data, self.x, self.y)
            }
            BgMode::Mode5 | BgMode::Mode6 => todo!(),
            BgMode::Mode7 => todo!(),
        };

        let bg_idx = match bg_layer {
            ColorLayer::Bg1 => 0,
            ColorLayer::Bg2 => 1,
            ColorLayer::Bg3 => 2,
            ColorLayer::Bg4 => 3,
            _ => unreachable!(),
        };

        let cache_hit = self.bg_tile_cache[bg_idx].get_entry_idx(&tile_data);

        self.bg_tile_cache_accesses += 1;
        self.bg_tile_cache_hits += 1;

        let cache_idx = if let Some(idx) = cache_hit {
            idx
        } else {
            self.bg_tile_cache_hits -= 1;

            let chr_data = self.fetch_chr_data(bus.ppu_regs, bus.vram, tile_data.clone());
            let chr_base = bg_data.chr_base_addr;

            let pal_indices = match color_depth {
                ColorDepth::Bpp2 => {
                    let addr = chr_base + (chr_data.chr_idx << 3) + chr_data.chr_row as u16;
                    let bp10 = bus.vram[addr as usize];

                    interleave_2bpp(bp10) as u64
                }
                ColorDepth::Bpp4 => {
                    let addr = chr_base + (chr_data.chr_idx << 4) + chr_data.chr_row as u16;
                    let bp10 = bus.vram[addr as usize];
                    let bp32 = bus.vram[(addr + 8) as usize];

                    interleave_4bpp(bp10, bp32) as u64
                }
                ColorDepth::Bpp8 => {
                    let addr = chr_base + (chr_data.chr_idx << 5) + chr_data.chr_row as u16;
                    if bus.ppu_regs.use_direct_col {
                        // Store only the row base addr; vram must be read per dot
                        addr as u64
                    } else {
                        let bp10 = bus.vram[addr as usize];
                        let bp32 = bus.vram[(addr + 8) as usize];
                        let bp54 = bus.vram[(addr + 16) as usize];
                        let bp76 = bus.vram[(addr + 24) as usize];

                        interleave_8bpp(bp10, bp32, bp54, bp76)
                    }
                }
            };

            self.bg_tile_cache[bg_idx].cache_tile(
                TileRowCacheEntry {
                    valid: true,
                    tile_addr: tile_data.tile_addr,
                    tile_row_key: tile_data.tile_row,
                    tile_col_block: tile_data.tile_col / 8,
                    chr_data,
                    pal_indices,
                }
            )
        };

        // Recompute chr_col for this dot using cached flip_x and tile_width.
        // We cannot use c.chr_data.chr_col — it was computed for the tile_col at fill time.
        let t = &self.bg_tile_cache[bg_idx].entries[cache_idx];
        let raw_col = tile_data.tile_col; // 0..tile_width, pre-flip
        let flipped_col = if t.chr_data.flip_x {
            t.chr_data.tile_width - raw_col - 1
        } else {
            raw_col
        };
        let chr_col = flipped_col % 8; // sub-tile column within the 8px bitplane word

        match color_depth {
            ColorDepth::Bpp2 => Self::extract_2bpp(t, chr_col, bus.cgram, bg_cgram_base_addr),
            ColorDepth::Bpp4 => Self::extract_4bpp(t, chr_col, bus.cgram, bg_cgram_base_addr),
            ColorDepth::Bpp8 => Self::extract_8bpp(t, chr_col, bus.cgram, bus.ppu_regs, bus.vram),
        }
    }

    /// For modes 0-4
    pub fn bg_tile_idx(
        &self,
        regs: &PpuRegs,
        bg_data: &BgSettings,
        x: usize,
        y: usize,
    ) -> TileData {
        let (mosaic_x, mosaic_y) = Self::apply_mosaic(x, y, regs.mosaic_size as usize);

        let scroll_range = match bg_data.chr_size {
            TileSize::Size8x8 => 0x1FF,
            TileSize::Size16x16 => 0x3FF,
        };

        let shifted_x = ((mosaic_x as u16) + bg_data.scroll_x) & scroll_range;
        let shifted_y = ((mosaic_y as u16) + bg_data.scroll_y) & scroll_range;

        let tilemap_offset = Self::tm_offset(shifted_x, shifted_y, bg_data.tilemap_cnt_x, bg_data.tilemap_cnt_y);

        let x = shifted_x & 0xFF;
        let y = shifted_y & 0xFF;

        let tile_idx = match bg_data.chr_size {
            TileSize::Size8x8 => ((y >> 3) << 5) | (x >> 3),
            TileSize::Size16x16 => (y & 0xF0) | (x >> 4),
        };

        let (tile_col, tile_row) = match bg_data.chr_size {
            TileSize::Size8x8 => (x & 7, y & 7),
            TileSize::Size16x16 => (x & 0xF, y & 0xF),
        };

        TileData {
            tile_addr: bg_data.tilemap_base_addr + tilemap_offset + tile_idx,
            tile_row: tile_row as u8,
            tile_col: tile_col as u8,
            tile_size: bg_data.chr_size,
        }
    }

    fn apply_mosaic(x: usize, y: usize, m: usize) -> (usize, usize) {
        if m == 0 {
            return (x, y);
        }

        if (m + 1) & m == 0 {
            return (x & !m, y & !m);
        }

        (
            x - (x % (m + 1)),
            y - (y % (m + 1)),
        )
    }

    fn tm_offset(x: u16, y: u16, cnt_x: TilemapCount, cnt_y: TilemapCount) -> u16 {
        if cnt_x == cnt_y {
            if cnt_x == TilemapCount::One {
                return 0;
            }
            return ((x & 0x100) << 2) | ((y as u16 & 0x100) << 3);
        }

        if cnt_y == TilemapCount::One {
            return (x & 0x100) << 2;
        }

        (y as u16 & 0x100) << 2
    }

    /// For modes 5-6
    fn hi_res_bg_tile_idx(&self, bus: &PpuBus, bg_data: &BgSettings) -> TileData {
        todo!()
    }

    pub fn fetch_bg_data(regs: &PpuRegs, bg_layer: ColorLayer) -> &BgSettings {
        match bg_layer {
            ColorLayer::Bg1 => &regs.bg_settings[0],
            ColorLayer::Bg2 => &regs.bg_settings[1],
            ColorLayer::Bg3 => &regs.bg_settings[2],
            ColorLayer::Bg4 => &regs.bg_settings[3],

            _ => panic!(), // Only called for bg layers
        }
    }

    fn fetch_chr_data(&self, regs: &PpuRegs, vram: &[u16], tile_data: TileData) -> ChrData {
        let tile_word = vram[tile_data.tile_addr as usize];

        let in_true_hi_res_mode = matches!(regs.bg_mode, BgMode::Mode5 | BgMode::Mode6);

        let (tile_height, tile_width) = match tile_data.tile_size {
            TileSize::Size8x8 => (8u8, if in_true_hi_res_mode { 16u8 } else { 8u8 }),
            TileSize::Size16x16 => (16u8, 16u8),
        };

        let tile_chr_idx = tile_word & 0x3FF;
        let tile_pal      = ((tile_word >> 10) & 7) as u8;
        let tile_priority = ((tile_word >> 13) & 1) as u8;
        let flip_x = (tile_word & 0x4000) != 0;
        let flip_y = (tile_word & 0x8000) != 0;

        let tile_row = if flip_y {
            tile_height - tile_data.tile_row - 1
        } else {
            tile_data.tile_row
        };
        let tile_col = if flip_x {
            tile_width - tile_data.tile_col - 1
        } else {
            tile_data.tile_col
        };

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
            chr_row: tile_row,
            chr_col: tile_col, // valid only for this specific tile_data.tile_col; not used from cache
            chr_pal: tile_pal,
            chr_priority: tile_priority,
            flip_x,
            tile_width,
        }
    }

    fn extract_2bpp(t: &TileRowCacheEntry, chr_col: u8, cgram: &[Color], bg_cgram_base_addr: u8) -> ColorData {
        let pal_idx = ((t.pal_indices >> (2 * (7 - chr_col))) & 3) as u8;

        let cgram_addr = bg_cgram_base_addr | (t.chr_data.chr_pal << 2) | pal_idx;
        ColorData {
            color: if pal_idx == 0 { cgram[0] } else { cgram[cgram_addr as usize] },
            priority: t.chr_data.chr_priority,
            transparent: pal_idx == 0,
        }
    }

    fn extract_4bpp(t: &TileRowCacheEntry, chr_col: u8, cgram: &[Color], bg_cgram_base_addr: u8) -> ColorData {

        let pal_idx = ((t.pal_indices >> (4 * (7 - chr_col))) & 15) as u8;

        let cgram_addr = bg_cgram_base_addr | (t.chr_data.chr_pal << 4) | pal_idx;
        ColorData {
            color: if pal_idx == 0 { cgram[0] } else { cgram[cgram_addr as usize] },
            priority: t.chr_data.chr_priority,
            transparent: pal_idx == 0,
        }
    }

    fn extract_8bpp(t: &TileRowCacheEntry, chr_col: u8, cgram: &[Color], regs: &PpuRegs, vram: &[u16]) -> ColorData {
        if !regs.use_direct_col {
            let cgram_addr = (t.pal_indices >> (8 * (7 - chr_col))) as u8;

            ColorData {
                color: if cgram_addr == 0 { cgram[0] } else { cgram[cgram_addr as usize] },
                priority: t.chr_data.chr_priority,
                transparent: cgram_addr == 0,
            }
        } else {
            // Direct color: one vram word per dot column; bp_words[0] is the cached row base addr.
            // This path is inherently per-dot, so vram must be accessed here.
            let r_ext = (t.chr_data.chr_pal & 0x04) >> 1;
            let g_ext = (t.chr_data.chr_pal & 0x08) >> 2;
            let b_ext = (t.chr_data.chr_pal & 0x10) >> 2;

            // let row_base = t.bp_words[0];
            let row_base = t.pal_indices as u16;
            let rgb_data = vram[(row_base + chr_col as u16) as usize] as u8;

            let r = ((rgb_data & 0x07) << 2) | r_ext;
            let g = ((rgb_data & 0x38) >> 1) | g_ext;
            let b = ((rgb_data & 0xC0) >> 3) | b_ext;

            let color = Color::new(r, g, b);
            ColorData {
                color,
                priority: t.chr_data.chr_priority,
                transparent: (r == 0) && (g == 0) && (b == 0),
            }
        }
    }

    fn apply_cmath(&self, bus: &PpuBus, main_col: Color, sub_col: Color) -> Color {
        let col_win_en = Self::win_active_signal(self.in_w1, self.in_w1, &bus.ppu_regs.col_settings.window);

        let main_col = match bus.ppu_regs.col_win_main_region {
            WindowColorRegion::Nowhere => main_col,
            WindowColorRegion::Outside => {
                if col_win_en {
                    main_col
                } else {
                    Color::BLACK
                }
            }
            WindowColorRegion::Inside => {
                if col_win_en {
                    Color::BLACK
                } else {
                    main_col
                }
            }
            WindowColorRegion::Everywhere => Color::BLACK,
        };
        let sub_col = match bus.ppu_regs.col_win_sub_region {
            WindowColorRegion::Nowhere => sub_col,
            WindowColorRegion::Outside => {
                if col_win_en {
                    sub_col
                } else {
                    bus.cgram[0]
                }
            }
            WindowColorRegion::Inside => {
                if col_win_en {
                    bus.cgram[0]
                } else {
                    sub_col
                }
            }
            WindowColorRegion::Everywhere => bus.cgram[0],
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

    fn win_active_signal(
        in_w1: bool,
        in_w2: bool,
        win_settings: &WindowSettings,
    ) -> bool {
        let w1_en = (win_settings.w1_en && in_w1) ^ win_settings.w1_inv;
        let w2_en = (win_settings.w2_en && in_w2) ^ win_settings.w2_inv;

        let win_en = if win_settings.w1_en && win_settings.w2_en {
            match win_settings.logic {
                WindowLogic::Or => w1_en || w2_en,
                WindowLogic::And => w1_en && w2_en,
                WindowLogic::Xor => w1_en ^ w2_en,
                WindowLogic::Xnor => !(w1_en ^ w2_en),
            }
        } else if win_settings.w1_en {
            w1_en
        } else if win_settings.w2_en {
            w2_en
        } else {
            false
        };

        win_en
    }

    fn bg_layer_colors(&mut self, bus: &PpuBus, col_depth: ColorDepth, cgram_base_addr: u8, bg_layer: ColorLayer) -> (ColorData, ColorData) {
        let bg_data = Self::fetch_bg_data(bus.ppu_regs, bg_layer);

        let win_en = Self::win_active_signal(self.in_w1, self.in_w2, &bg_data.window);
        
        let win_main = bg_data.window.main_en && win_en;
        let win_sub = bg_data.window.sub_en && win_en;

        let mut bg_main_col = None;
        let mut bg_sub_col = None;

        if bg_data.main_en && !win_main {
            bg_main_col = Some(self.bg_col(
                bus,
                bg_layer, col_depth,
                cgram_base_addr
            ));
        }

        if bg_data.sub_en && !win_sub {
            bg_sub_col = Some(match bg_main_col {
                    Some(c) => c,
                    None => self.bg_col(
                        bus,
                        bg_layer, col_depth,
                        cgram_base_addr
                    ),
                });
        }

        let bg_main_col = match bg_main_col {
            Some(c) => c,
            None => self.transparent_color_data(bus),
        };
        let bg_sub_col = match bg_sub_col {
            Some(c) => c,
            None => self.transparent_color_data(bus),
        };

        (bg_main_col, bg_sub_col)
    }

    fn obj_layer_colors(&mut self, bus: &PpuBus) -> (ColorData, ColorData) {
        let win_en = Self::win_active_signal(self.in_w1, self.in_w2, &bus.ppu_regs.col_settings.window);

        let obj_win_main = bus.ppu_regs.obj_settings.window.main_en && win_en;
        let obj_win_sub = bus.ppu_regs.obj_settings.window.sub_en && win_en;

        let mut obj_main_col = None;
        let mut obj_sub_col = None;

        if bus.ppu_regs.obj_settings.main_en && !obj_win_main {
            obj_main_col = Some(self.sprite_col(bus));
        }

        if bus.ppu_regs.obj_settings.sub_en && !obj_win_sub {
            obj_sub_col = Some(obj_main_col.unwrap_or(self.sprite_col(bus)));
        }

        let obj_main_col = obj_main_col.unwrap_or(self.transparent_color_data(bus));
        let obj_sub_col = obj_sub_col.unwrap_or(self.transparent_color_data(bus));

        (obj_main_col, obj_sub_col)
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
        self.x = self.screen_x();

        self.in_w1 = bus.ppu_regs.w1_left_pos as usize <= self.x && self.x <= bus.ppu_regs.w1_right_pos as usize;
        self.in_w2 = bus.ppu_regs.w2_left_pos as usize <= self.x && self.x <= bus.ppu_regs.w2_right_pos as usize;

        if self.dot == SCANLINE_END_DOT {
            self.dot = 0;
            self.scanline += 1;
            self.y = self.screen_y();

            if self.scanline == VBLANK_END_SCANLINE_NTSC {
                self.scanline = 0;
            }

            if self.frame > 350 {
                log::debug!("scanline: {}, frame: {}, in_w1: {}, in_w2: {}", self.scanline, self.frame, self.in_w1, self.in_w2);
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

        let trigger_int = match cpu_regs.hv_timer_irq_mode {
            HVTimerIRQ::None => false,
            HVTimerIRQ::HTimer => ppu_regs.h_counter == cpu_regs.h_counter_target,
            HVTimerIRQ::VTimer => {
                ppu_regs.v_counter == cpu_regs.v_counter_target && ppu_regs.h_counter == 0
            }
            HVTimerIRQ::Both => {
                ppu_regs.v_counter == cpu_regs.v_counter_target
                    && ppu_regs.h_counter == cpu_regs.h_counter_target
            }
        };

        if trigger_int {
            cpu_regs.hv_timer_irq_flag = true;
            bus.trigger_interrupt(CpuInterrupt::IRQ);
        }
    }

    /// Finds all possible sprites that could be rendered on the current scanline
    /// based on the y-positions of the sprites
    fn find_scanline_sprites(&mut self, bus: &mut PpuBus) {
        let regs = &mut bus.ppu_regs;

        self.scanline_sprites.clear();

        let true_y = if regs.screen_interlace_en && regs.obj_interlace_en {
            2 * self.y + (self.frame & 1)
        } else {
            self.y
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
            if spr_y as usize <= true_y && true_y < spr_y_max as usize {
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

impl<P: DebugProbe> Ppu5C7x<P> {
    pub fn draw_debug_layers(
        &mut self,
        bus: &mut PpuBus,
        bg1_buffer: &mut [u8],
        bg2_buffer: &mut [u8],
        bg3_buffer: &mut [u8],
        bg4_buffer: &mut [u8],
        obj_buffer: &mut [u8],
    ) {
        let (bg1_col_depth, bg2_col_depth, bg3_col_depth, bg4_col_depth) =
            match bus.ppu_regs.bg_mode {
                BgMode::Mode0 => (Some(ColorDepth::Bpp2), Some(ColorDepth::Bpp2), Some(ColorDepth::Bpp2), Some(ColorDepth::Bpp2)),
                BgMode::Mode1 => (Some(ColorDepth::Bpp4), Some(ColorDepth::Bpp4), Some(ColorDepth::Bpp2), None),
                BgMode::Mode2 => (Some(ColorDepth::Bpp4), Some(ColorDepth::Bpp4), None, None),
                BgMode::Mode3 |
                BgMode::Mode4 => (Some(ColorDepth::Bpp8), Some(ColorDepth::Bpp4), None, None),
                _ => (None, None, None, None),
            };
        let (bg1_cgram_addr, bg2_cgram_addr, bg3_cgram_addr, bg4_cgram_addr) =
            match bus.ppu_regs.bg_mode {
                BgMode::Mode0 => (Some(0x00), Some(0x20), Some(0x40), Some(0x60)),
                BgMode::Mode1 => (Some(0x00), Some(0x00), Some(0x00), None),
                BgMode::Mode2 => (Some(0x00), Some(0x00), None, None),
                BgMode::Mode3 |
                BgMode::Mode4 => (Some(0x00), Some(0x00), None, None),
                _ => (None, None, None, None),
            };

        let bg1_col = if let (Some(color_depth), Some(bg_cgram_base_addr)) = (bg1_col_depth, bg1_cgram_addr) {
            self.bg_col(bus, ColorLayer::Bg1, color_depth, bg_cgram_base_addr)
        } else {
            self.transparent_color_data(bus)
        };
        let bg2_col = if let (Some(color_depth), Some(bg_cgram_base_addr)) = (bg2_col_depth, bg2_cgram_addr) {
            self.bg_col(bus, ColorLayer::Bg2, color_depth, bg_cgram_base_addr)
        } else {
            self.transparent_color_data(bus)
        };
        let bg3_col = if let (Some(color_depth), Some(bg_cgram_base_addr)) = (bg3_col_depth, bg3_cgram_addr) {
            self.bg_col(bus, ColorLayer::Bg3, color_depth, bg_cgram_base_addr)
        } else {
            self.transparent_color_data(bus)
        };
        let bg4_col = if let (Some(color_depth), Some(bg_cgram_base_addr)) = (bg4_col_depth, bg4_cgram_addr) {
            self.bg_col(bus, ColorLayer::Bg4, color_depth, bg_cgram_base_addr)
        } else {
            self.transparent_color_data(bus)
        };
        let obj_col = self.sprite_col(bus);

        let checker_col = if (self.x / 2 + self.y / 2) % 2 == 0 {
            [0x50, 0x50, 0x50, 255]
        } else {
            [0x30, 0x30, 0x30, 255]
        };

        let bg1_col = if bg1_col.transparent {
            checker_col
        } else {
            [bg1_col.color.r, bg1_col.color.g, bg1_col.color.b, 255]
        };
        let bg2_col = if bg2_col.transparent {
            checker_col
        } else {
            [bg2_col.color.r, bg2_col.color.g, bg2_col.color.b, 255]
        };
        let bg3_col = if bg3_col.transparent {
            checker_col
        } else {
            [bg3_col.color.r, bg3_col.color.g, bg3_col.color.b, 255]
        };
        let bg4_col = if bg4_col.transparent {
            checker_col
        } else {
            [bg4_col.color.r, bg4_col.color.g, bg4_col.color.b, 255]
        };
        let obj_col = if obj_col.transparent {
            checker_col
        } else {
            [obj_col.color.r, obj_col.color.g, obj_col.color.b, 255]
        };

        let pixel_idx = (self.y * 256 + self.x) * 4;

        bg1_buffer[pixel_idx..pixel_idx+4].copy_from_slice(&bg1_col);
        bg2_buffer[pixel_idx..pixel_idx+4].copy_from_slice(&bg2_col);
        bg3_buffer[pixel_idx..pixel_idx+4].copy_from_slice(&bg3_col);
        bg4_buffer[pixel_idx..pixel_idx+4].copy_from_slice(&bg4_col);
        obj_buffer[pixel_idx..pixel_idx+4].copy_from_slice(&obj_col);
    }
}
