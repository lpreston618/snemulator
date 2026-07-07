use crate::debug::DebugHarness;
use crate::savestate;
use crate::scpu::ioregs::HVTimerIRQ;
use crate::sppu::bus::PpuBus;
use crate::sppu::regs::PpuRegs;
use crate::sppu::utils::{interleave_2bpp, interleave_4bpp, interleave_8bpp};

pub use color::Color;
pub use types::*;

pub mod bus;
pub mod color;
pub mod regs;
mod types;

#[macro_use]
pub mod utils;

pub const VBLANK_START_SCANLINE: usize = 225;
const VBLANK_END_SCANLINE_NTSC: usize = 262;
const VISIBLE_SCANLINE_START_DOT: usize = 22;
pub const HBLANK_START_DOT: usize = 278;
const SCANLINE_END_DOT: usize = 340;
const VISIBLE_DOTS_PER_SCANLINE: usize = HBLANK_START_DOT - VISIBLE_SCANLINE_START_DOT;

const TILE_CACHE_SIZE: usize = 1;

pub struct Ppu5C7x {
    pub dot: usize,
    pub scanline: usize,
    /// x position of the current dot on the screen
    pub x: usize,
    /// y position of the current scanline on the screen
    pub y: usize,
    pub frame: usize,

    scanline_sprites: Vec<usize>,

    bg_tile_cache: [TileRowCache<TILE_CACHE_SIZE>; 4],

    scanline_bg_counters: [usize; 4],
    // A background or object color of `None` corresponds to a transparent color
    scanline_bg_data: [[Option<BgColorData>; VISIBLE_DOTS_PER_SCANLINE]; 4],
    // Additional buffers for extra pixels in true hi-res mode (Bg modes 5 & 6)
    bg1_extra_data: [Option<BgColorData>; VISIBLE_DOTS_PER_SCANLINE],
    bg2_extra_data: [Option<BgColorData>; VISIBLE_DOTS_PER_SCANLINE],
    scanline_sprite_data: [Option<ObjColorData>; VISIBLE_DOTS_PER_SCANLINE],

    last_main_screen_color: Color,
    last_sub_screen_color: Option<Color>,
    last_main_screen_pixel_did_cmath: bool,

    /// Number of master clocks until the next dot
    pub clocks: usize,
}

impl Ppu5C7x {
    pub fn new() -> Self {
        let mut ppu = Self {
            dot: 0,
            scanline: 0,
            x: 0,
            y: 0,
            frame: 0,
            scanline_sprites: Vec::new(),
            bg_tile_cache: std::array::repeat(TileRowCache::new()),
            scanline_bg_counters: [0; 4],
            scanline_bg_data: [[None; VISIBLE_DOTS_PER_SCANLINE]; 4],
            bg1_extra_data: [None; VISIBLE_DOTS_PER_SCANLINE],
            bg2_extra_data: [None; VISIBLE_DOTS_PER_SCANLINE],
            scanline_sprite_data: [None; VISIBLE_DOTS_PER_SCANLINE],
            last_main_screen_color: Color::BLACK,
            last_sub_screen_color: None,
            last_main_screen_pixel_did_cmath: false,
            clocks: 0,
        };

        ppu.x = ppu.screen_x();
        ppu.y = ppu.screen_y();

        ppu
    }

    pub fn save_state(&self, regs: &PpuRegs) -> savestate::PpuState {
        savestate::PpuState {
            dot: self.dot,
            scanline: self.scanline,
            frame: self.frame,
            clocks: self.clocks,
            in_fblank: regs.in_fblank,
            screen_brightness: regs.screen_brightness,
            obj_sprite_size: regs.obj_sprite_size,
            name_base_addr: regs.name_base_addr,
            name_secondary_base_addr: regs.name_secondary_base_addr,
            oam_write_high_table: regs.oam_write_high_table,
            internal_oam_addr: regs.internal_oam_addr,
            priority_rotation: regs.priority_rotation,
            priority_rotation_idx: regs.priority_rotation_idx,
            oam_data_latch: regs.oam_data_latch,
            bg3_mode1_priority: regs.bg3_mode1_priority,
            bg_mode: regs.bg_mode,
            mosaic_size: regs.mosaic_size,
            bg_settings: regs.bg_settings.clone(),
            obj_settings: regs.obj_settings.clone(),
            col_window: regs.col_window.clone(),
            m7_latch: regs.m7_latch,
            bg_offset_latch: regs.bg_offset_latch,
            bg_offset_x_latch: regs.bg_offset_x_latch,
            m7_scroll_x: regs.m7_scroll_x,
            m7_scroll_y: regs.m7_scroll_y,
            vram_addr_inc_mode: regs.vram_addr_inc_mode,
            addr_remap_mode: regs.addr_remap_mode,
            addr_inc_size: regs.addr_inc_size,
            vram_addr: regs.vram_addr,
            m7_tilemap_repeat: regs.m7_tilemap_repeat,
            m7_fill_mode: regs.m7_fill_mode,
            m7_flip_bg_y: regs.m7_flip_bg_y,
            m7_flip_bg_x: regs.m7_flip_bg_x,
            m7_matrix_a: regs.m7_matrix_a,
            mult_factor_16: regs.mult_factor_16,
            m7_matrix_b: regs.m7_matrix_b,
            mult_factor_8: regs.mult_factor_8,
            m7_matrix_c: regs.m7_matrix_c,
            m7_matrix_d: regs.m7_matrix_d,
            m7_center_x: regs.m7_center_x,
            m7_center_y: regs.m7_center_y,
            cgram_toggle: regs.cgram_toggle,
            cgram_addr: regs.cgram_addr,
            cgram_latch: regs.cgram_latch,
            w1_left_pos: regs.w1_left_pos,
            w1_right_pos: regs.w1_right_pos,
            w2_left_pos: regs.w2_left_pos,
            w2_right_pos: regs.w2_right_pos,
            col_win_main_region: regs.col_win_main_region,
            col_win_sub_region: regs.col_win_sub_region,
            sub_color_fixed: regs.sub_color_fixed,
            use_direct_col: regs.use_direct_col,
            cmath_operator: regs.cmath_operator,
            cmath_half: regs.cmath_half,
            back_cmath_en: regs.back_cmath_en,
            fixed_color: regs.fixed_color,
            _external_sync: regs._external_sync,
            ext_bg_en: regs.ext_bg_en,
            hi_res_en: regs.hi_res_en,
            overscan_en: regs.overscan_en,
            obj_interlace_en: regs.obj_interlace_en,
            screen_interlace_en: regs.screen_interlace_en,
            multiply_result: regs.multiply_result,
            vram_latch: regs.vram_latch,
            h_counter_toggle: regs.h_counter_toggle,
            h_counter_latch: regs.h_counter_latch,
            v_counter_toggle: regs.v_counter_toggle,
            v_counter_latch: regs.v_counter_latch,
            sprite_overflow: regs.sprite_overflow,
            sprite_tile_overflow: regs.sprite_tile_overflow,
            master_slave_state: regs.master_slave_state,
            ppu1_version: regs.ppu1_version,
            interlace_field: regs.interlace_field,
            counter_toggle: regs.counter_toggle,
            video_type: regs.video_type,
            ppu2_version: regs.ppu2_version,
        }
    }

    pub fn load_state(&mut self, regs: &mut PpuRegs, state: &savestate::PpuState, _version: u32) {
        self.dot = state.dot;
        self.scanline = state.scanline;
        self.x = self.screen_x();
        self.y = self.screen_y();
        self.frame = state.frame;
        self.clocks = state.clocks;

        regs.in_fblank = state.in_fblank;
        regs.screen_brightness = state.screen_brightness;
        regs.obj_sprite_size = state.obj_sprite_size;
        regs.name_base_addr = state.name_base_addr;
        regs.name_secondary_base_addr = state.name_secondary_base_addr;
        regs.oam_write_high_table = state.oam_write_high_table;
        regs.internal_oam_addr = state.internal_oam_addr;
        regs.priority_rotation = state.priority_rotation;
        regs.priority_rotation_idx = state.priority_rotation_idx;
        regs.oam_data_latch = state.oam_data_latch;
        regs.bg3_mode1_priority = state.bg3_mode1_priority;
        regs.bg_mode = state.bg_mode;
        regs.mosaic_size = state.mosaic_size;
        regs.bg_settings = state.bg_settings.clone();
        regs.obj_settings = state.obj_settings.clone();
        regs.col_window = state.col_window.clone();
        regs.m7_latch = state.m7_latch;
        regs.bg_offset_latch = state.bg_offset_latch;
        regs.bg_offset_x_latch = state.bg_offset_x_latch;
        regs.m7_scroll_x = state.m7_scroll_x;
        regs.m7_scroll_y = state.m7_scroll_y;
        regs.vram_addr_inc_mode = state.vram_addr_inc_mode;
        regs.addr_remap_mode = state.addr_remap_mode;
        regs.addr_inc_size = state.addr_inc_size;
        regs.vram_addr = state.vram_addr;
        regs.m7_tilemap_repeat = state.m7_tilemap_repeat;
        regs.m7_fill_mode = state.m7_fill_mode;
        regs.m7_flip_bg_y = state.m7_flip_bg_y;
        regs.m7_flip_bg_x = state.m7_flip_bg_x;
        regs.m7_matrix_a = state.m7_matrix_a;
        regs.mult_factor_16 = state.mult_factor_16;
        regs.m7_matrix_b = state.m7_matrix_b;
        regs.mult_factor_8 = state.mult_factor_8;
        regs.m7_matrix_c = state.m7_matrix_c;
        regs.m7_matrix_d = state.m7_matrix_d;
        regs.m7_center_x = state.m7_center_x;
        regs.m7_center_y = state.m7_center_y;
        regs.cgram_toggle = state.cgram_toggle;
        regs.cgram_addr = state.cgram_addr;
        regs.cgram_latch = state.cgram_latch;
        regs.w1_left_pos = state.w1_left_pos;
        regs.w1_right_pos = state.w1_right_pos;
        regs.w2_left_pos = state.w2_left_pos;
        regs.w2_right_pos = state.w2_right_pos;
        regs.col_win_main_region = state.col_win_main_region;
        regs.col_win_sub_region = state.col_win_sub_region;
        regs.sub_color_fixed = state.sub_color_fixed;
        regs.use_direct_col = state.use_direct_col;
        regs.cmath_operator = state.cmath_operator;
        regs.cmath_half = state.cmath_half;
        regs.back_cmath_en = state.back_cmath_en;
        regs.fixed_color = state.fixed_color;
        regs._external_sync = state._external_sync;
        regs.ext_bg_en = state.ext_bg_en;
        regs.hi_res_en = state.hi_res_en;
        regs.overscan_en = state.overscan_en;
        regs.obj_interlace_en = state.obj_interlace_en;
        regs.screen_interlace_en = state.screen_interlace_en;
        regs.multiply_result = state.multiply_result;
        regs.vram_latch = state.vram_latch;
        regs.h_counter_toggle = state.h_counter_toggle;
        regs.h_counter_latch = state.h_counter_latch;
        regs.v_counter_toggle = state.v_counter_toggle;
        regs.v_counter_latch = state.v_counter_latch;
        regs.sprite_overflow = state.sprite_overflow;
        regs.sprite_tile_overflow = state.sprite_tile_overflow;
        regs.master_slave_state = state.master_slave_state;
        regs.ppu1_version = state.ppu1_version;
        regs.interlace_field = state.interlace_field;
        regs.counter_toggle = state.counter_toggle;
        regs.video_type = state.video_type;
        regs.ppu2_version = state.ppu2_version;

        regs.in_w1 = regs.w1_left_pos as usize <= self.x && self.x <= regs.w1_right_pos as usize;
        regs.in_w2 = regs.w2_left_pos as usize <= self.x && self.x <= regs.w2_right_pos as usize;
    }

    pub fn power_on(&mut self) {
        self.dot = 0;
        self.scanline = 0;
        self.x = self.screen_x();
        self.y = self.screen_y();
        self.frame = 0;
        self.scanline_sprites.clear();
        self.clocks = 0;

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
    pub fn cycle<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
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

    fn draw_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
        if bus.ppu_regs.in_fblank{
            self.set_pixel(bus, Color::BLACK, Some(Color::BLACK), false);
            return;        
        }

        match bus.ppu_regs.bg_mode {
            BgMode::Mode0 => self.draw_mode0_dot(bus),
            BgMode::Mode1 => self.draw_mode1_dot(bus),
            BgMode::Mode2 => self.draw_mode2_dot(bus),
            BgMode::Mode3 => self.draw_mode3_dot(bus),
            BgMode::Mode4 => self.draw_mode4_dot(bus),
            BgMode::Mode5 => self.draw_mode5_dot(bus),
            BgMode::Mode6 => self.draw_mode6_dot(bus),
            BgMode::Mode7 => self.draw_mode7_dot(bus),
        };
    }

    #[inline(always)]
    fn set_pixel<H: DebugHarness>(
        &self, 
        bus: &mut PpuBus<H>, 
        main_color: Color,
        sub_color: Option<Color>,
        cmath_en: bool,
    ) {
        const BYTES_PER_PIXEL: usize = 4;
        const PIXELS_PER_ROW: usize = 512;
        
        let true_hi_res = matches!(bus.ppu_regs.bg_mode, BgMode::Mode5 | BgMode::Mode6);
        let hi_res = bus.ppu_regs.hi_res_en || true_hi_res;
        let interlace = bus.ppu_regs.screen_interlace_en;
        let x = self.x;
        let y = self.y;
        let field = self.frame & 1;

        let main_color = if cmath_en {
            self.apply_cmath(bus, main_color, sub_color)
        } else {
            main_color
        };

        let sub_color = sub_color.unwrap_or(bus.cgram[0]);
        let sub_color = if hi_res {
            if self.last_main_screen_pixel_did_cmath {
                if self.last_sub_screen_color == Some(bus.ppu_regs.fixed_color) {
                    self.apply_cmath(bus, sub_color, Some(bus.ppu_regs.fixed_color))
                } else {
                    self.apply_cmath(bus, sub_color, Some(self.last_main_screen_color))
                }
            } else {
                sub_color
            }
        } else {
            sub_color
        };

        let main_color = Self::apply_brightness(main_color, bus.ppu_regs.screen_brightness);
        let sub_color  = Self::apply_brightness(sub_color,  bus.ppu_regs.screen_brightness);

        if interlace && hi_res {
            let row = 2 * y + field;
            let col = 2 * x;
            let idx1 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * row) + (col + 0));
            let idx2 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * row) + (col + 1));

            bus.frame_buffer[idx1..idx1+4].copy_from_slice(&sub_color.to_rgba_bytes());
            bus.frame_buffer[idx2..idx2+4].copy_from_slice(&main_color.to_rgba_bytes());
        } else if interlace {
            let row = 2 * y + field;
            let col = 2 * x;
            let idx1 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * row) + (col + 0));
            let idx2 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * row) + (col + 1));

            bus.frame_buffer[idx1..idx1+4].copy_from_slice(&main_color.to_rgba_bytes());
            bus.frame_buffer[idx2..idx2+4].copy_from_slice(&main_color.to_rgba_bytes());
        } else if hi_res {
            let row = 2 * y;
            let col = 2 * x;
            let sub1  = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * (row + 0)) + (col + 0));
            let sub2  = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * (row + 1)) + (col + 0));
            let main1 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * (row + 0)) + (col + 1));
            let main2 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * (row + 1)) + (col + 1));

            bus.frame_buffer[sub1..sub1+4].copy_from_slice(&sub_color.to_rgba_bytes());
            bus.frame_buffer[sub2..sub2+4].copy_from_slice(&sub_color.to_rgba_bytes());
            bus.frame_buffer[main1..main1+4].copy_from_slice(&main_color.to_rgba_bytes());
            bus.frame_buffer[main2..main2+4].copy_from_slice(&main_color.to_rgba_bytes());
        } else {
            let row = 2 * y;
            let col = 2 * x;
            let idx1 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * (row + 0)) + (col + 0));
            let idx2 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * (row + 0)) + (col + 1));
            let idx3 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * (row + 1)) + (col + 0));
            let idx4 = BYTES_PER_PIXEL * ((PIXELS_PER_ROW * (row + 1)) + (col + 1));

            bus.frame_buffer[idx1..idx1+4].copy_from_slice(&main_color.to_rgba_bytes());
            bus.frame_buffer[idx2..idx2+4].copy_from_slice(&main_color.to_rgba_bytes());
            bus.frame_buffer[idx3..idx3+4].copy_from_slice(&main_color.to_rgba_bytes());
            bus.frame_buffer[idx4..idx4+4].copy_from_slice(&main_color.to_rgba_bytes());
        }
    }

    #[inline(always)]
    fn render_bg_tiles<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>, mode_bg_settings: &[BgRenderSettings]) {
        for (bg, &bg_render_settings) in mode_bg_settings.into_iter().enumerate() {
            if self.x == self.scanline_bg_counters[bg] {
                let dots_rendered = self.render_tile(bus, bg, bg_render_settings);

                debug_assert!(dots_rendered > 0, "{} {:?} {} {}", bg, bus.ppu_regs.bg_mode, self.x, self.y);

                self.scanline_bg_counters[bg] += dots_rendered;
            }
        }
    }

    #[inline(always)]
    fn render_hires_bg_tiles<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>, mode_bg_settings: &[BgRenderSettings]) {
        for (bg, &bg_render_settings) in mode_bg_settings.into_iter().enumerate() {
            if self.x == self.scanline_bg_counters[bg] {
                // self.render_hires_tile(bus, bg, bg_render_settings, true);
                // let dots_rendered = self.render_hires_tile(bus, bg, bg_render_settings, false);

                let dots_rendered = self.render_hires_tile(bus, bg, bg_render_settings);

                debug_assert!(dots_rendered > 0, "{} {:?} {} {}", bg, bus.ppu_regs.bg_mode, self.x, self.y);

                self.scanline_bg_counters[bg] += dots_rendered;
            }
        }
    }

    fn layer_window_signals(regs: &PpuRegs) -> WindowSignals {
        let bg_main: [bool; 4] = std::array::from_fn(|bg| {
            regs.bg_settings[bg].main_en && (!regs.bg_settings[bg].window.main_en || !regs.bg_apply_window_signals[bg])
        });
        let bg_sub: [bool; 4] = std::array::from_fn(|bg| {
            regs.bg_settings[bg].sub_en && (!regs.bg_settings[bg].window.sub_en || !regs.bg_apply_window_signals[bg])
        });

        let obj_main = regs.obj_settings.main_en && (!regs.obj_settings.window.main_en || !regs.obj_apply_window_signal);
        let obj_sub = regs.obj_settings.sub_en && (!regs.obj_settings.window.sub_en || !regs.obj_apply_window_signal);

        let (col_main, col_sub) = Self::color_window_signals(regs);

        WindowSignals {
            bg_main,
            bg_sub,
            obj_main,
            obj_sub,
            color_main: col_main,
            color_sub: col_sub,
        }
    }

    fn draw_mode0_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x20;
        const BG3_CGRAM_BASE_ADDR: u8 = 0x40;
        const BG4_CGRAM_BASE_ADDR: u8 = 0x60;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;
        const BG3_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;
        const BG4_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;

        const MODE0_BG_SETTINGS: &[BgRenderSettings] = &[
            BgRenderSettings { cgram_base: BG1_CGRAM_BASE_ADDR, color_depth: BG1_COL_DEPTH, use_offset_per_tile: false },
            BgRenderSettings { cgram_base: BG2_CGRAM_BASE_ADDR, color_depth: BG2_COL_DEPTH, use_offset_per_tile: false },
            BgRenderSettings { cgram_base: BG3_CGRAM_BASE_ADDR, color_depth: BG3_COL_DEPTH, use_offset_per_tile: false },
            BgRenderSettings { cgram_base: BG4_CGRAM_BASE_ADDR, color_depth: BG4_COL_DEPTH, use_offset_per_tile: false },
        ];

        self.render_bg_tiles(bus, MODE0_BG_SETTINGS);

        let regs = &bus.ppu_regs;

        let win_signals = Self::layer_window_signals(regs);

        let obj_main_col = if win_signals.obj_main { self.scanline_sprite_data[self.x] } else { None };
        let bg1_main_col = if win_signals.bg_main[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_main_col = if win_signals.bg_main[1] { self.scanline_bg_data[1][self.x] } else { None };
        let bg3_main_col = if win_signals.bg_main[2] { self.scanline_bg_data[2][self.x] } else { None };
        let bg4_main_col = if win_signals.bg_main[3] { self.scanline_bg_data[3][self.x] } else { None };

        let (main_col, main_col_layer) = Self::bg_mode0_choose_priority_color(
            obj_main_col,
            bg1_main_col,
            bg2_main_col,
            bg3_main_col,
            bg4_main_col,
        ).unwrap_or((bus.cgram[0], ColorLayer::Back));

        let main_col = if win_signals.color_main { Color::BLACK } else { main_col };

        let obj_sub_col = if win_signals.obj_sub { self.scanline_sprite_data[self.x] } else { None };
        let bg1_sub_col = if win_signals.bg_sub[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_sub_col = if win_signals.bg_sub[1] { self.scanline_bg_data[1][self.x] } else { None };
        let bg3_sub_col = if win_signals.bg_sub[2] { self.scanline_bg_data[2][self.x] } else { None };
        let bg4_sub_col = if win_signals.bg_sub[3] { self.scanline_bg_data[3][self.x] } else { None };

        let sub_col = Self::bg_mode0_choose_priority_color(
            obj_sub_col,  bg1_sub_col,  bg2_sub_col, 
            bg3_sub_col, bg4_sub_col,
        ).map(|pair| pair.0);

        let cmath_en = match main_col_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Bg3 => bus.ppu_regs.bg_settings[2].cmath_en,
            ColorLayer::Bg4 => bus.ppu_regs.bg_settings[3].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.unwrap().palette >= 4,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
        } && !win_signals.color_sub;

        self.set_pixel(
            bus,
            main_col,
            sub_col,
            cmath_en,
        );

        self.last_main_screen_color = main_col;
        self.last_sub_screen_color = sub_col;
        self.last_main_screen_pixel_did_cmath = cmath_en;
    }

    fn draw_mode1_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG3_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
        const BG3_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;

        const MODE1_BG_SETTINGS: &[BgRenderSettings] = &[
            BgRenderSettings { cgram_base: BG1_CGRAM_BASE_ADDR, color_depth: BG1_COL_DEPTH, use_offset_per_tile: false },
            BgRenderSettings { cgram_base: BG2_CGRAM_BASE_ADDR, color_depth: BG2_COL_DEPTH, use_offset_per_tile: false },
            BgRenderSettings { cgram_base: BG3_CGRAM_BASE_ADDR, color_depth: BG3_COL_DEPTH, use_offset_per_tile: false },
        ];

        self.render_bg_tiles(bus, MODE1_BG_SETTINGS);

        let regs = &bus.ppu_regs;

        let win_signals = Self::layer_window_signals(regs);

        let obj_main_col = if win_signals.obj_main { self.scanline_sprite_data[self.x] } else { None };
        let bg1_main_col = if win_signals.bg_main[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_main_col = if win_signals.bg_main[1] { self.scanline_bg_data[1][self.x] } else { None };
        let bg3_main_col = if win_signals.bg_main[2] { self.scanline_bg_data[2][self.x] } else { None };

        let (main_col, main_col_layer) = Self::bg_mode1_choose_priority_color(
            obj_main_col,
            bg1_main_col,
            bg2_main_col,
            bg3_main_col,
            bus.ppu_regs.bg3_mode1_priority,
        ).unwrap_or((bus.cgram[0], ColorLayer::Back));

        let main_col = if win_signals.color_main { Color::BLACK } else { main_col };

        let obj_sub_col = if win_signals.obj_sub { self.scanline_sprite_data[self.x] } else { None };
        let bg1_sub_col = if win_signals.bg_sub[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_sub_col = if win_signals.bg_sub[1] { self.scanline_bg_data[1][self.x] } else { None };
        let bg3_sub_col = if win_signals.bg_sub[2] { self.scanline_bg_data[2][self.x] } else { None };

        // let sub_col = if bus.ppu_regs.sub_color_fixed {
        //     Some(bus.ppu_regs.fixed_color)
        // } else {
        //     Self::bg_mode1_choose_priority_color(
        //         obj_sub_col, bg1_sub_col, 
        //         bg2_sub_col, bg3_sub_col,
        //         bus.ppu_regs.bg3_mode1_priority,
        //     ).map(|pair| pair.0)
        // };

        let sub_col = Self::bg_mode1_choose_priority_color(
            obj_sub_col, bg1_sub_col, 
            bg2_sub_col, bg3_sub_col,
            bus.ppu_regs.bg3_mode1_priority,
        ).map_or(Some(bus.ppu_regs.fixed_color), |pair| Some(pair.0));

        // let sub_col = if win_signals.color_sub { None } else { sub_col };

        let cmath_en = match main_col_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Bg3 => bus.ppu_regs.bg_settings[2].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.unwrap().palette >= 4,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
            _ => unreachable!(),
        } && !win_signals.color_sub;

        self.set_pixel(
            bus,
            main_col,
            sub_col,
            cmath_en,
        );

        self.last_main_screen_color = main_col;
        self.last_sub_screen_color = sub_col;
        self.last_main_screen_pixel_did_cmath = cmath_en;
    }
    
    fn draw_mode2_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;

        const MODE2_BG_SETTINGS: &[BgRenderSettings] = &[
            BgRenderSettings { cgram_base: BG1_CGRAM_BASE_ADDR, color_depth: BG1_COL_DEPTH, use_offset_per_tile: true },
            BgRenderSettings { cgram_base: BG2_CGRAM_BASE_ADDR, color_depth: BG2_COL_DEPTH, use_offset_per_tile: true },
        ];

        self.render_bg_tiles(bus, MODE2_BG_SETTINGS);

        let regs = &bus.ppu_regs;

        let win_signals = Self::layer_window_signals(regs);

        let obj_main_col = if win_signals.obj_main { self.scanline_sprite_data[self.x] } else { None };
        let bg1_main_col = if win_signals.bg_main[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_main_col = if win_signals.bg_main[1] { self.scanline_bg_data[1][self.x] } else { None };

        let (main_col, main_col_layer) = Self::bg_modes2thru5_choose_priority_color(
            obj_main_col,
            bg1_main_col,
            bg2_main_col,
        ).unwrap_or((bus.cgram[0], ColorLayer::Back));

        let main_col = if win_signals.color_main { Color::BLACK } else { main_col };

        let obj_sub_col = if win_signals.obj_sub { self.scanline_sprite_data[self.x] } else { None };
        let bg1_sub_col = if win_signals.bg_sub[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_sub_col = if win_signals.bg_sub[1] { self.scanline_bg_data[1][self.x] } else { None };

        let sub_col = Self::bg_modes2thru5_choose_priority_color(
            obj_sub_col, bg1_sub_col, bg2_sub_col,
        ).map(|pair| pair.0);

        let cmath_en = match main_col_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.unwrap().palette >= 4,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
            _ => unreachable!(),
        } && !win_signals.color_sub;

        self.set_pixel(
            bus,
            main_col,
            sub_col,
            cmath_en,
        );

        self.last_main_screen_color = main_col;
        self.last_sub_screen_color = sub_col;
        self.last_main_screen_pixel_did_cmath = cmath_en;
    }

    fn draw_mode3_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp8;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;

        const MODE3_BG_SETTINGS: &[BgRenderSettings] = &[
            BgRenderSettings { cgram_base: BG1_CGRAM_BASE_ADDR, color_depth: BG1_COL_DEPTH, use_offset_per_tile: false },
            BgRenderSettings { cgram_base: BG2_CGRAM_BASE_ADDR, color_depth: BG2_COL_DEPTH, use_offset_per_tile: false },
        ];

        self.render_bg_tiles(bus, MODE3_BG_SETTINGS);

        let regs = &bus.ppu_regs;

        let win_signals = Self::layer_window_signals(regs);

        let obj_main_col = if win_signals.obj_main { self.scanline_sprite_data[self.x] } else { None };
        let bg1_main_col = if win_signals.bg_main[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_main_col = if win_signals.bg_main[1] { self.scanline_bg_data[1][self.x] } else { None };

        let (main_col, main_col_layer) = Self::bg_modes2thru5_choose_priority_color(
            obj_main_col,
            bg1_main_col,
            bg2_main_col,
        ).unwrap_or((bus.cgram[0], ColorLayer::Back));

        let main_col = if win_signals.color_main { Color::BLACK } else { main_col };

        let obj_sub_col = if win_signals.obj_sub { self.scanline_sprite_data[self.x] } else { None };
        let bg1_sub_col = if win_signals.bg_sub[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_sub_col = if win_signals.bg_sub[1] { self.scanline_bg_data[1][self.x] } else { None };

        let sub_col = Self::bg_modes2thru5_choose_priority_color(
            obj_sub_col, bg1_sub_col, bg2_sub_col,
        ).map(|pair| pair.0);

        let cmath_en = match main_col_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.unwrap().palette >= 4,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
            _ => unreachable!(),
        } && !win_signals.color_sub;

        self.set_pixel(
            bus,
            main_col,
            sub_col,
            cmath_en,
        );

        self.last_main_screen_color = main_col;
        self.last_sub_screen_color = sub_col;
        self.last_main_screen_pixel_did_cmath = cmath_en;
    }

    fn draw_mode4_dot<H: DebugHarness>(&self, bus: &mut PpuBus<H>) {
        
    }

    fn draw_mode5_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
        const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;

        const MODE5_BG_SETTINGS: &[BgRenderSettings] = &[
            BgRenderSettings { cgram_base: BG1_CGRAM_BASE_ADDR, color_depth: BG1_COL_DEPTH, use_offset_per_tile: false },
            BgRenderSettings { cgram_base: BG2_CGRAM_BASE_ADDR, color_depth: BG2_COL_DEPTH, use_offset_per_tile: false },
        ];

        self.render_hires_bg_tiles(bus, MODE5_BG_SETTINGS);

        let regs = &bus.ppu_regs;

        let win_signals = Self::layer_window_signals(regs);

        let obj_main_col = if win_signals.obj_main { self.scanline_sprite_data[self.x] } else { None };
        let bg1_main_col = if win_signals.bg_main[0] { self.scanline_bg_data[0][self.x] } else { None };
        let bg2_main_col = if win_signals.bg_main[1] { self.scanline_bg_data[1][self.x] } else { None };

        let (main_col, main_col_layer) = Self::bg_modes2thru5_choose_priority_color(
            obj_main_col,
            bg1_main_col,
            bg2_main_col,
        ).unwrap_or((bus.cgram[0], ColorLayer::Back));

        let main_col = if win_signals.color_main { Color::BLACK } else { main_col };

        let obj_sub_col = if win_signals.obj_sub { self.scanline_sprite_data[self.x] } else { None };
        let bg1_sub_col = if win_signals.bg_sub[0] { self.bg1_extra_data[self.x] } else { None };
        let bg2_sub_col = if win_signals.bg_sub[1] { self.bg2_extra_data[self.x] } else { None };

        // let sub_col = if bus.ppu_regs.sub_color_fixed {
        //     bus.ppu_regs.fixed_color
        // } else {
        //     Self::bg_modes2thru5_choose_priority_color(
        //         obj_sub_col, 
        //         bg1_sub_col, 
        //         bg2_sub_col,
        //     ).map_or(
        //         bus.ppu_regs.fixed_color, 
        //         |pair| pair.0
        //     )
        // };

        let sub_col = Self::bg_modes2thru5_choose_priority_color(
            obj_sub_col, bg1_sub_col, bg2_sub_col,
        ).map(|pair| pair.0);

        let cmath_en = match main_col_layer {
            ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
            ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
            ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.unwrap().palette >= 4,
            ColorLayer::Back => bus.ppu_regs.back_cmath_en,
            _ => unreachable!("BGs 3 & 4 not used in mode 5"),
        } && !win_signals.color_sub;

        self.set_pixel(
            bus,
            main_col,
            sub_col,
            cmath_en,
        );

        self.last_main_screen_color = main_col;
        self.last_sub_screen_color = sub_col;
        self.last_main_screen_pixel_did_cmath = cmath_en;
    }

    fn draw_mode6_dot<H: DebugHarness>(&self, bus: &mut PpuBus<H>) {
        
    }
    fn draw_mode7_dot<H: DebugHarness>(&self, bus: &mut PpuBus<H>) {
        
    }

    #[inline(always)]
    fn apply_scroll<H: DebugHarness>(&self, bus: &PpuBus<H>, bg: usize, use_offset_per_tile: bool) -> (u16, u16) {
        let bg_settings = &bus.ppu_regs.bg_settings[bg];

        let scroll_range = match bg_settings.chr_size {
            ChrSize::Size8x8 => 0x1FF,
            ChrSize::Size16x16 => 0x3FF,
        };
        let scroll_x = bg_settings.scroll_x;
        let scroll_y = bg_settings.scroll_y;

        let mut shifted_x = (self.x as u16 + scroll_x) & scroll_range;
        let mut shifted_y = (self.y as u16 + scroll_y) & scroll_range;

        // If using offset per tile and not on the first 8-pixel column
        if use_offset_per_tile && self.x >= 8 {
            let bg3 = &bus.ppu_regs.bg_settings[2];

            let bg3_x = (shifted_x & 7) | (((self.x as u16 - 8) & (!7)) + (bg3.scroll_x & (!7)));
            let bg3_y = bg3.scroll_y;

            let bg3_tile_x = bg3_x / 8;
            let bg3_tile_y = bg3_y / 8;

            let tilemap_offset_x = match (bg3.tilemap_cnt_x, bg3.tilemap_cnt_y) {
                (TilemapCount::One, TilemapCount::One) => 0,
                (TilemapCount::One, TilemapCount::Two) => {
                    (bg3_tile_y & 0x20) << 5
                },
                (TilemapCount::Two, TilemapCount::One) => {
                    (bg3_tile_x & 0x20) << 5
                },
                (TilemapCount::Two, TilemapCount::Two) => {
                    ((bg3_tile_y & 0x20) << 6) + ((bg3_tile_x & 0x20) << 5)
                }
            };

            let tilemap_offset_y = match (bg3.tilemap_cnt_x, bg3.tilemap_cnt_y) {
                (TilemapCount::One, TilemapCount::One) => 0,
                (TilemapCount::One, TilemapCount::Two) => {
                    ((bg3_tile_y + 1) & 0x20) << 5
                },
                (TilemapCount::Two, TilemapCount::One) => {
                    (bg3_tile_x & 0x20) << 5
                },
                (TilemapCount::Two, TilemapCount::Two) => {
                    (((bg3_tile_y + 1) & 0x20) << 6) + ((bg3_tile_x & 0x20) << 5)
                }
            };

            let bg3_entry_addr_x = (bg3.tilemap_base_addr + ((bg3_tile_y & 0x1F) << 5) + (bg3_tile_x & 0x1F) + tilemap_offset_x) & 0x7FFF;
            let bg3_entry_addr_y = (bg3.tilemap_base_addr + (((bg3_tile_y + 1) & 0x1F) << 5) + (bg3_tile_x & 0x1F) + tilemap_offset_y) & 0x7FFF;
        
            let scroll_x_entry = TilemapScrollEntry::from_word(bus.vram[bg3_entry_addr_x as usize]);
            let scroll_y_entry = TilemapScrollEntry::from_word(bus.vram[bg3_entry_addr_y as usize]);

            let do_x_scroll = if bg == 0 { scroll_x_entry.bg1_offset_en } else { scroll_x_entry.bg2_offset_en };
            let do_y_scroll = if bg == 0 { scroll_y_entry.bg1_offset_en } else { scroll_y_entry.bg2_offset_en };

            if do_x_scroll {
                shifted_x = (shifted_x & 7) | ((self.x as u16 & (!7)) + (scroll_x_entry.scroll & (!7)));
            }

            if do_y_scroll {
                shifted_y = self.y as u16 + scroll_y_entry.scroll;
            }

            shifted_x &= scroll_range;
            shifted_y &= scroll_range;
        }

        (shifted_x, shifted_y)
    }

    /// Renders a tile to a BG scanline buffer. Returns the number of dots rendered.
    #[inline(always)]
    fn render_tile<H: DebugHarness>(
        &mut self,
        bus: &mut PpuBus<H>,
        bg: usize,
        bg_render_settings: BgRenderSettings,
    ) -> usize {
        let bpp = bg_render_settings.color_depth.bits_per_pixel();

        let bg_settings = &bus.ppu_regs.bg_settings[bg];

        let (shifted_x, shifted_y) = self.apply_scroll(bus, bg, bg_render_settings.use_offset_per_tile);
    
        let m = bus.ppu_regs.mosaic_size as u16;

        let (playfield_x, playfield_y) = if bg_settings.mosaic_en {
            (Self::apply_mosaic(shifted_x, m), Self::apply_mosaic(shifted_y, m))
        } else {
            (shifted_x, shifted_y)
        };
        
        let (size_x, size_y) = bg_settings.chr_size.raw_size();
        let tilemap_x = playfield_x / size_x;
        let tilemap_y = playfield_y / size_y;
        let tile_col = playfield_x % size_x;
        let tile_row = playfield_y % size_y;

        // Calculate offset into VRAM to find the tilemap given playfield position
        // and background settings.
        let tilemap_offset = match (bg_settings.tilemap_cnt_x, bg_settings.tilemap_cnt_y) {
            (TilemapCount::One, TilemapCount::One) => 0,
            (TilemapCount::One, TilemapCount::Two) => {
                (tilemap_y & 0x20) << 5
            },
            (TilemapCount::Two, TilemapCount::One) => {
                (tilemap_x & 0x20) << 5
            },
            (TilemapCount::Two, TilemapCount::Two) => {
                ((tilemap_y & 0x20) << 6) + ((tilemap_x & 0x20) << 5)
            }
        };

        let tilemap_entry_addr = (bg_settings.tilemap_base_addr + ((tilemap_y & 0x1F) << 5) + (tilemap_x & 0x1F) + tilemap_offset) & 0x7FFF;
        let tilemap_entry = TilemapEntry::from_word(bus.vram[tilemap_entry_addr as usize]);

        let tile_col = if tilemap_entry.flip_x { size_x - tile_col - 1 } else { tile_col };
        let tile_row = if tilemap_entry.flip_y { size_y - tile_row - 1 } else { tile_row };

        // tile_number = tile_number + 1 if tile_col >= 8, + 32 if tile_row >= 8
        let chr_x = tilemap_entry.chr_num & 0x1F;
        let chr_y = tilemap_entry.chr_num >> 5;
        let tile_x = (chr_x << (size_x >> 4)) + (tile_col >> 3);
        let tile_y = (chr_y << (size_y >> 4)) + (tile_row >> 3);
        let tile_number = (tile_y << (5 + (size_y >> 4))) + tile_x;
        // chr_addr will never be out of range when reading a chr
        let tile_addr = ((bg_settings.chr_base_addr + tile_number * 4 * bpp) & 0x7FFF) as usize;
        let tile_row_addr = tile_addr + (tile_row as usize % 8);

        let pal_indices: [u8; 8] = match bg_render_settings.color_depth {
            ColorDepth::Bpp2 => {
                let bp10 = bus.vram[tile_row_addr];

                std::array::from_fn(|i| {
                    let bp1 = ((bp10 >> (15 - i)) & 1) as u8;
                    let bp0 = ((bp10 >> (7 - i)) & 1) as u8;
                    (bp1 << 1) | bp0
                })
            }
            ColorDepth::Bpp4 => {
                let bp10 = bus.vram[tile_row_addr];
                let bp32 = bus.vram[tile_row_addr + 8];
                
                let indices = std::array::from_fn(|i| {
                    let bp0 = ((bp10 >> ( 7 - i)) & 1) as u8;
                    let bp1 = ((bp10 >> (15 - i)) & 1) as u8;
                    let bp2 = ((bp32 >> ( 7 - i)) & 1) as u8;
                    let bp3 = ((bp32 >> (15 - i)) & 1) as u8;
                    (bp3 << 3) | (bp2 << 2) | (bp1 << 1) | bp0
                });

                indices
            }
            ColorDepth::Bpp8 => {
                if bus.ppu_regs.use_direct_col {
                    let [col0, col1] = bus.vram[tile_row_addr].to_le_bytes();
                    let [col2, col3] = bus.vram[tile_row_addr + 8].to_le_bytes();
                    let [col4, col5] = bus.vram[tile_row_addr + 16].to_le_bytes();
                    let [col6, col7] = bus.vram[tile_row_addr + 24].to_le_bytes();

                    [col0, col1, col2, col3, col4, col5, col6, col7]
                } else {
                    let bp10 = bus.vram[tile_row_addr];
                    let bp32 = bus.vram[tile_row_addr + 8];
                    let bp54 = bus.vram[tile_row_addr + 16];
                    let bp76 = bus.vram[tile_row_addr + 24];
    
                    std::array::from_fn(|i| {
                        let bp0 = ((bp10 >> (7 - i)) & 1) as u8;
                        let bp1 = ((bp10 >> (15 - i)) & 1) as u8;
                        let bp2 = ((bp32 >> (7 - i)) & 1) as u8;
                        let bp3 = ((bp32 >> (15 - i)) & 1) as u8;
                        let bp4 = ((bp54 >> (7 - i)) & 1) as u8;
                        let bp5 = ((bp54 >> (15 - i)) & 1) as u8;
                        let bp6 = ((bp76 >> (7 - i)) & 1) as u8;
                        let bp7 = ((bp76 >> (15 - i)) & 1) as u8;
                        (bp7 << 7) | (bp6 << 6) | (bp5 << 5) | (bp4 << 4) | (bp3 << 3) | (bp2 << 2) | (bp1 << 1) | bp0
                    })
                }
            }
        };

        let mut dots_rendered = 0;

        // Cut off the ends of the tile if we are close to the edge of the screen
        let col_start = if self.x == 0 { bg_settings.scroll_x % 8 } else { 0 };
        let col_end = if self.x > 256 - 8 { 256 - self.x as u16 } else { 8 };

        let mut i = 0;

        for col in col_start..col_end {
            let mosaiced_x = Self::apply_mosaic(shifted_x + col, m);

            let idx = self.x + i as usize;
            i += 1;

            // TODO: Move this check before the tile recalculation somehow to avoid all of the math.
            // 
            // When doing mosaic, if the mosaiced x < scrolled x, then we are rendering a part of a mosaic
            // tile that is not the first dot in the mosaic tile, so we can grab the color from the prev dot
            // and repeat it for the whole mosaic tile. We can only do this trick per-scanline, however, as 
            // mosaic may have changed between scanlines. If we are on the first pixel of a mosaic tile, then
            // mosaiced_x will be equal to scrolled x, so we render the color as normal. We cannot use this 
            // trick on the first pixel of the background, obviously, as there is no previous color to extend.
            if idx > 0 && mosaiced_x < shifted_x + col {
                self.scanline_bg_data[bg][idx] = self.scanline_bg_data[bg][idx - 1];

                dots_rendered += 1;

                continue;
            }

            let pal_idx = if tilemap_entry.flip_x {
                pal_indices[7 - col as usize]
            } else {
                pal_indices[col as usize]
            };

            let color = if pal_idx == 0 {
                None
            } else {
                let color = if bg_render_settings.color_depth == ColorDepth::Bpp8 && bus.ppu_regs.use_direct_col {
                    let raw_color = pal_idx; // Pal index is the raw color

                    // https://snes.nesdev.org/wiki/Tiles#8bpp_Direct_Color
                    let r_ext = (tilemap_entry.palette >> 0) & 1;
                    let g_ext = (tilemap_entry.palette >> 1) & 1;
                    let b_ext = (tilemap_entry.palette >> 2) & 1;
                    let r = (((raw_color >> 0) & 7) << 2) | (r_ext << 1);
                    let g = (((raw_color >> 3) & 7) << 2) | (g_ext << 1);
                    let b = (((raw_color >> 6) & 3) << 3) | (b_ext << 2);
                    
                    Color::new(r, g, b)
                } else {
                    let cgram_addr = bg_render_settings.cgram_base + (tilemap_entry.palette << bpp) + pal_idx;

                    bus.cgram[cgram_addr as usize]
                };

                Some(BgColorData {
                    color,
                    palette: tilemap_entry.palette,
                    priority: tilemap_entry.priority,
                })
            };

            self.scanline_bg_data[bg][idx] = color;

            dots_rendered += 1;
        }

        dots_rendered
    }

    fn render_hires_tile<H: DebugHarness>(
        &mut self,
        bus: &mut PpuBus<H>,
        bg: usize,
        bg_render_settings: BgRenderSettings,
    ) -> usize {
        let interlace = bus.ppu_regs.screen_interlace_en;
        let field = (self.frame & 1) as u16;
        let bpp = bg_render_settings.color_depth.bits_per_pixel();
        let bg_settings = &bus.ppu_regs.bg_settings[bg];

        // ---- Step 1: Coordinates in HI-RES space ----------------------------
        //
        // bsnes works in hi-res pixel space throughout: hpixel is doubled, and
        // scroll registers are shifted left by 1 so all math is consistent.
        //
        //   hpixel  = self.x * 2                     [in 0..512]
        //   hscroll = scroll_x * 2                   [in 0..1024]
        //   hoffset = (hpixel + hscroll) & (hsize-1)
        let hpixel = (self.x as u16) << 1;
        let hscroll = bg_settings.scroll_x << 1;

        // ---- Step 2: Vertical coordinate ------------------------------------
        //
        // In interlace, the vertical is doubled and the field bit selects a
        // half-line. Per bsnes: `vpixel = vpixel << 1 | (field && !mosaic.enable)`.
        let mosaic_en = bg_settings.mosaic_en;
        let vpixel_base = self.y as u16;
        let vpixel = if interlace {
            (vpixel_base << 1) | if !mosaic_en { field } else { 0 }
        } else {
            vpixel_base
        };
        let vscroll = bg_settings.scroll_y;

        // Note: I'm skipping mosaic vertical offset adjustment for brevity; add
        // `vpixel -= mosaic.voffset() << (hires && interlace)` if you support it.

        // ---- Step 3: Playfield offsets and sizes ----------------------------
        //
        // hsize/vsize are in hi-res pixel units.
        //   width = 256 << 1 = 512
        //   hsize = width << tileSize << screenSize.bit(0)
        //   vsize = width << tileSize << screenSize.bit(1)
        let (_, tile_h_native) = bg_settings.chr_size.raw_size(); // 8 or 16
        let tile_size_bit = if tile_h_native == 16 { 1u16 } else { 0u16 };

        let width_hires: u16 = 512;
        let screen_size_x_bit = matches!(bg_settings.tilemap_cnt_x, TilemapCount::Two) as u16;
        let screen_size_y_bit = matches!(bg_settings.tilemap_cnt_y, TilemapCount::Two) as u16;

        let hsize = width_hires << tile_size_bit << screen_size_x_bit;
        let vsize = width_hires << tile_size_bit << screen_size_y_bit;

        let hoffset = hpixel.wrapping_add(hscroll) & (hsize - 1);
        let voffset = vpixel.wrapping_add(vscroll) & (vsize - 1);

        // ---- Step 4: Tile indices -------------------------------------------
        //
        // htiles = 4 in hi-res  (each tilemap entry covers 16 hi-res pixels wide)
        // vtiles = 3 + tileSize (8 or 16 tall)
        let htiles = 4u16;
        let vtiles = 3u16 + tile_size_bit;

        let htile = hoffset >> htiles;   // tilemap x index (in tile grid)
        let vtile = voffset >> vtiles;   // tilemap y index

        // ---- Step 5: Tilemap address ----------------------------------------
        let hscreen: u16 = if screen_size_x_bit == 1 { 32 << 5 } else { 0 };
        let vscreen: u16 = if screen_size_y_bit == 1 {
            32u16 << (5 + screen_size_x_bit)
        } else {
            0
        };

        let mut tilemap_offset = ((htile & 0x1F) << 0) | ((vtile & 0x1F) << 5);
        if htile & 0x20 != 0 { tilemap_offset = tilemap_offset.wrapping_add(hscreen); }
        if vtile & 0x20 != 0 { tilemap_offset = tilemap_offset.wrapping_add(vscreen); }

        let tilemap_addr = bg_settings.tilemap_base_addr.wrapping_add(tilemap_offset);
        let tilemap_entry = TilemapEntry::from_word(bus.vram[tilemap_addr as usize]);

        // ---- Step 6: Compute CHR numbers for the two halves ----------------
        //
        // bsnes performs two nametable fetches per 16-hi-res-pixel span, with
        // hpixel differing by 8 between them. For EACH fetch, it applies:
        //   if (htiles == 4 && bool(hoffset & 8) != hmirror) character += 1;
        //   if (vtiles == 4 && bool(voffset & 8) != vmirror) character += 16;
        //
        // We replicate this per-half so that the left tile and right tile each
        // get the correct character number. This matters even at scanline start
        // because scroll can put us at any hoffset value.
        //
        // Note: `hoffset` and `voffset` here are the SAME values computed in
        // Step 3 (masked to hsize/vsize).

        let left_hoffset = hoffset;
        let right_hoffset = (hoffset.wrapping_add(8)) & (hsize - 1);

        let char_base = tilemap_entry.chr_num;

        let mut char_left = char_base;
        if (left_hoffset & 8 != 0) != tilemap_entry.flip_x {
            char_left = char_left.wrapping_add(1);
        }
        if vtiles == 4 && (voffset & 8 != 0) != tilemap_entry.flip_y {
            char_left = char_left.wrapping_add(16);
        }
        char_left &= 0x3FF;

        let mut char_right = char_base;
        if (right_hoffset & 8 != 0) != tilemap_entry.flip_x {
            char_right = char_right.wrapping_add(1);
        }
        if vtiles == 4 && (voffset & 8 != 0) != tilemap_entry.flip_y {
            char_right = char_right.wrapping_add(16);
        }
        char_right &= 0x3FF;

        // ---- Step 7 (revised): Row within the 8x8 tile ---------------------
        //
        // The +16 for the lower half of a 16-tall tile is already applied in
        // char_left/char_right. Here we just extract which row within the
        // selected 8x8 tile to use, applying vertical flip.
        // let row_in_tile = (voffset as u16) & ((1 << vtiles) - 1); // 0..tile_h
        let row_in_8x8_pre_flip = (voffset & 7) as u16;
        let row_in_8x8 = if tilemap_entry.flip_y {
            7 - row_in_8x8_pre_flip
        } else {
            row_in_8x8_pre_flip
        };

        // ---- Step 8 (revised): Fetch both tile row addresses ---------------
        //
        // A 2bpp tile is 8 words, 4bpp is 16 words. This matches bsnes'
        // `origin << (3 + mode)` where mode is the bpp INDEX (0/1/2), giving
        // shift of 3/4/5 = tile sizes of 8/16/32 words.
        let words_per_tile = (bpp << 2) as u16;
        let addr_left = bg_settings.chr_base_addr
            .wrapping_add(char_left.wrapping_mul(words_per_tile))
            .wrapping_add(row_in_8x8)
            & 0x7FFF;

        let addr_right = bg_settings.chr_base_addr
            .wrapping_add(char_right.wrapping_mul(words_per_tile))
            .wrapping_add(row_in_8x8)
            & 0x7FFF;

        // ---- Step 9: Decode 16 palette indices (8 from each CHR tile) -------
        //
        // Left tile provides hi-res columns 0..8 of the span,
        // Right tile provides hi-res columns 8..16.
        // Within each tile, apply hmirror to column order.
        let mut pal_indices = [0u8; 16];
        self.decode_tile_row_into(bus, addr_left as usize,  bpp, tilemap_entry.flip_x, &mut pal_indices[0..8]);
        self.decode_tile_row_into(bus, addr_right as usize, bpp, tilemap_entry.flip_x, &mut pal_indices[8..16]);

            // ---- Step 10: Emit pixels into both buffers -------------------------
        //
        // The 16 hi-res columns of this span map to native slots as follows:
        //   hi-res col 2k    -> native slot (self.x + k), sub-screen  (even col)
        //   hi-res col 2k+1  -> native slot (self.x + k), main-screen (odd col)
        //
        // We just decoded 16 palette indices in playfield hi-res col order
        // (pal_indices[0] = leftmost). Since we started at native x = self.x,
        // and the tilemap fetch aligns to a 16-hi-res-pixel boundary in
        // playfield space, we may need to skip some columns at the start if
        // hscroll causes a mid-span start.
        //
        // The number of hi-res columns to skip at the start:
        //   skip = hoffset & 15    (playfield offset within the 16-px span)
        //
        // But note: `hoffset` here is the playfield offset for `hpixel = self.x*2`.
        // Since scroll is coarse (2-hires-pixel granularity per the docs), the
        // low bit is always 0, so skip is always even. That means we always start
        // on a sub-screen column, which is what we want.
        let skip = (hoffset & 15) as usize;

        // How many hi-res cols we can still emit before running off the screen:
        let hires_available = 16usize - skip;
        let hires_remaining_scanline = (512 - hpixel as usize).min(hires_available);

        let extra_buffer = if bg == 0 {
            &mut self.bg1_extra_data
        } else {
            &mut self.bg2_extra_data
        };

        let mut dots_rendered = 0usize;
        let mut last_native_x: Option<usize> = None;

        for k in 0..hires_remaining_scanline {
            let pal_idx = pal_indices[skip + k];
            let hires_col_relative = k;  // 0 = sub, 1 = main, 2 = sub, ...
            let native_slot = self.x + (hires_col_relative >> 1);
            let is_main_col = (hires_col_relative & 1) == 1;

            let color = if pal_idx == 0 {
                None
            } else {
                let cgram_addr = bg_render_settings.cgram_base
                    + (tilemap_entry.palette << bpp)
                    + pal_idx;
                Some(BgColorData {
                    color: bus.cgram[cgram_addr as usize],
                    palette: tilemap_entry.palette,
                    priority: tilemap_entry.priority,
                })
            };

            if is_main_col {
                self.scanline_bg_data[bg][native_slot] = color;
            } else {
                extra_buffer[native_slot] = color;
            }

            if last_native_x != Some(native_slot) {
                dots_rendered += 1;
                last_native_x = Some(native_slot);
            }
        }

        dots_rendered
    }

    // Helper: decode 8 palette indices from one CHR tile row into `out`.
    fn decode_tile_row_into<H: DebugHarness>(
        &self,
        bus: &PpuBus<H>,
        row_addr: usize,
        bpp: u16,
        hmirror: bool,
        out: &mut [u8],
    ) {
        debug_assert_eq!(out.len(), 8);
        match bpp {
            2 => {
                let bp10 = bus.vram[row_addr];
                for i in 0..8 {
                    let src_bit = if hmirror { i } else { 7 - i };
                    let bp0 = ((bp10 >> src_bit)       & 1) as u8;
                    let bp1 = ((bp10 >> (8 + src_bit)) & 1) as u8;
                    out[i] = (bp1 << 1) | bp0;
                }
            }
            4 => {
                let bp10 = bus.vram[row_addr];
                let bp32 = bus.vram[row_addr + 8];
                for i in 0..8 {
                    let src_bit = if hmirror { i } else { 7 - i };
                    let bp0 = ((bp10 >> src_bit)       & 1) as u8;
                    let bp1 = ((bp10 >> (8 + src_bit)) & 1) as u8;
                    let bp2 = ((bp32 >> src_bit)       & 1) as u8;
                    let bp3 = ((bp32 >> (8 + src_bit)) & 1) as u8;
                    out[i] = (bp3 << 3) | (bp2 << 2) | (bp1 << 1) | bp0;
                }
            }
            _ => unreachable!("Mode 5/6 use 2bpp or 4bpp only"),
        }
    }

    #[inline]
    fn bg_mode0_choose_priority_color(
        obj_col: Option<ObjColorData>,
        bg1_col: Option<BgColorData>,
        bg2_col: Option<BgColorData>,
        bg3_col: Option<BgColorData>,
        bg4_col: Option<BgColorData>,
    ) -> Option<(Color, ColorLayer)> {
        if obj_col.is_some() && obj_col.unwrap().priority == 3 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg1_col.is_some() && bg1_col.unwrap().priority {
            Some((bg1_col.unwrap().color, ColorLayer::Bg1))
        } else if bg2_col.is_some() && bg2_col.unwrap().priority {
            Some((bg2_col.unwrap().color, ColorLayer::Bg2))
        } else if obj_col.is_some() && obj_col.unwrap().priority == 2 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg1_col.is_some() {
            Some((bg1_col.unwrap().color, ColorLayer::Bg1))
        } else if bg2_col.is_some() {
            Some((bg2_col.unwrap().color, ColorLayer::Bg2))
        } else if obj_col.is_some() && obj_col.unwrap().priority == 1 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg3_col.is_some() && bg3_col.unwrap().priority {
            Some((bg3_col.unwrap().color, ColorLayer::Bg3))
        } else if bg4_col.is_some() && bg4_col.unwrap().priority {
            Some((bg4_col.unwrap().color, ColorLayer::Bg4))
        } else if obj_col.is_some() {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg3_col.is_some() {
            Some((bg3_col.unwrap().color, ColorLayer::Bg3))
        } else if bg4_col.is_some() {
            Some((bg4_col.unwrap().color, ColorLayer::Bg4))
        } else {
            None
        }
    }

    #[inline]
    fn bg_mode1_choose_priority_color(
        obj_col: Option<ObjColorData>,
        bg1_col: Option<BgColorData>,
        bg2_col: Option<BgColorData>,
        bg3_col: Option<BgColorData>,
        bg3_priority: bool,
    ) -> Option<(Color, ColorLayer)> {
        if bg3_col.is_some() && bg3_priority && bg3_col.unwrap().priority {
            Some((bg3_col.unwrap().color, ColorLayer::Bg3))
        } else if obj_col.is_some() && obj_col.unwrap().priority == 3 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg1_col.is_some() && bg1_col.unwrap().priority {
            Some((bg1_col.unwrap().color, ColorLayer::Bg1))
        } else if bg2_col.is_some() && bg2_col.unwrap().priority {
            Some((bg2_col.unwrap().color, ColorLayer::Bg2))
        } else if obj_col.is_some() && obj_col.unwrap().priority == 2 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg1_col.is_some() {
            Some((bg1_col.unwrap().color, ColorLayer::Bg1))
        } else if bg2_col.is_some() {
            Some((bg2_col.unwrap().color, ColorLayer::Bg2))
        } else if obj_col.is_some() && obj_col.unwrap().priority == 1 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg3_col.is_some() && bg3_col.unwrap().priority {
            Some((bg3_col.unwrap().color, ColorLayer::Bg3))
        } else if obj_col.is_some() {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg3_col.is_some() {
            Some((bg3_col.unwrap().color, ColorLayer::Bg3))
        } else {
            None
        }
    }

    #[inline]
    fn bg_modes2thru5_choose_priority_color(
        obj_col: Option<ObjColorData>,
        bg1_col: Option<BgColorData>,
        bg2_col: Option<BgColorData>,
    ) -> Option<(Color, ColorLayer)> {
        if obj_col.is_some() && obj_col.unwrap().priority == 3 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg1_col.is_some() && bg1_col.unwrap().priority {
            Some((bg1_col.unwrap().color, ColorLayer::Bg1))
        } else if obj_col.is_some() && obj_col.unwrap().priority == 2 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg2_col.is_some() && bg2_col.unwrap().priority {
            Some((bg2_col.unwrap().color, ColorLayer::Bg2))
        } else if obj_col.is_some() && obj_col.unwrap().priority == 1 {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg1_col.is_some() {
            Some((bg1_col.unwrap().color, ColorLayer::Bg1))
        } else if obj_col.is_some() {
            Some((obj_col.unwrap().color, ColorLayer::Obj))
        } else if bg2_col.is_some() {
            Some((bg2_col.unwrap().color, ColorLayer::Bg2))
        } else {
            None
        }
    }

    fn apply_brightness(col: Color, brightness: u8) -> Color {
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

    // fn bg_mode1_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) -> DotColorData {
    //     const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG3_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
    //     const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
    //     const BG3_COL_DEPTH: ColorDepth = ColorDepth::Bpp2;

    //     let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
    //     let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG1_COL_DEPTH,
    //         BG1_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg1
    //     );
    //     let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG2_COL_DEPTH,
    //         BG2_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg2
    //     );
    //     let (bg3_main_col, bg3_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG3_COL_DEPTH,
    //         BG3_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg3
    //     );

    //     let (main_col, main_layer) = if bus.ppu_regs.bg3_mode1_priority
    //         && bg3_main_col.priority != 0
    //         && !bg3_main_col.transparent
    //     {
    //         (bg3_main_col.color, ColorLayer::Bg3)
    //     } else if obj_main_col.priority == 3 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
    //         (bg1_main_col.color, ColorLayer::Bg1)
    //     } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
    //         (bg2_main_col.color, ColorLayer::Bg2)
    //     } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if !bg1_main_col.transparent {
    //         (bg1_main_col.color, ColorLayer::Bg1)
    //     } else if !bg2_main_col.transparent {
    //         (bg2_main_col.color, ColorLayer::Bg2)
    //     } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
    //         (bg3_main_col.color, ColorLayer::Bg3)
    //     } else if !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if !bg3_main_col.transparent {
    //         (bg3_main_col.color, ColorLayer::Bg3)
    //     } else {
    //         (bus.cgram[0], ColorLayer::Back)
    //     };

    //     let sub_col = if bus.ppu_regs.sub_color_fixed {
    //         bus.ppu_regs.fixed_color
    //     } else if bus.ppu_regs.bg3_mode1_priority
    //         && bg3_sub_col.priority != 0
    //         && !bg3_sub_col.transparent
    //     {
    //         bg3_sub_col.color
    //     } else if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
    //         bg1_sub_col.color
    //     } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
    //         bg2_sub_col.color
    //     } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if !bg1_sub_col.transparent {
    //         bg1_sub_col.color
    //     } else if !bg2_sub_col.transparent {
    //         bg2_sub_col.color
    //     } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
    //         bg3_sub_col.color
    //     } else if !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if !bg3_sub_col.transparent {
    //         bg3_sub_col.color
    //     } else {
    //         bus.ppu_regs.fixed_color
    //     };

    //     let cmath_en = match main_layer {
    //         ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
    //         ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
    //         ColorLayer::Bg3 => bus.ppu_regs.bg_settings[2].cmath_en,
    //         ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.palette >= 4,
    //         ColorLayer::Back => bus.ppu_regs.back_cmath_en,
    //         _ => unreachable!(), // No other layers considered in Mode 1
    //     };

    //     DotColorData {
    //         main_col,
    //         sub_col,
    //         cmath_en,
    //     }
    // }

    // fn bg_mode2_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) -> DotColorData {
    //     const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;
    //     const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;

    //     let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
    //     let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG1_COL_DEPTH,
    //         BG1_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg1
    //     );
    //     let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG2_COL_DEPTH,
    //         BG2_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg2
    //     );

    //     let (main_col, main_layer) = if obj_main_col.priority == 3 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
    //         (bg1_main_col.color, ColorLayer::Bg1)
    //     } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
    //         (bg2_main_col.color, ColorLayer::Bg2)
    //     } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if !bg1_main_col.transparent {
    //         (bg1_main_col.color, ColorLayer::Bg1)
    //     } else if !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if !bg2_main_col.transparent {
    //         (bg2_main_col.color, ColorLayer::Bg2)
    //     } else {
    //         (bus.cgram[0], ColorLayer::Back)
    //     };

    //     let sub_col = if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
    //         bg1_sub_col.color
    //     } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
    //         bg2_sub_col.color
    //     } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if !bg1_sub_col.transparent {
    //         bg1_sub_col.color
    //     } else if !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if !bg2_sub_col.transparent {
    //         bg2_sub_col.color
    //     } else {
    //         bus.cgram[0]
    //     };

    //     let cmath_en = match main_layer {
    //         ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
    //         ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
    //         ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.palette >= 4,
    //         ColorLayer::Back => bus.ppu_regs.back_cmath_en,
    //         _ => unreachable!(), // No other layers considered in Mode 2
    //     };

    //     DotColorData {
    //         main_col,
    //         sub_col,
    //         cmath_en,
    //     }
    // }

    // fn bg_mode3_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) -> DotColorData {
    //     const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp8;
    //     const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;

    //     let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
    //     let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG1_COL_DEPTH,
    //         BG1_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg1
    //     );
    //     let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG2_COL_DEPTH,
    //         BG2_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg2
    //     );

    //     let (main_col, main_layer) = if obj_main_col.priority == 3 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
    //         (bg1_main_col.color, ColorLayer::Bg1)
    //     } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
    //         (bg2_main_col.color, ColorLayer::Bg2)
    //     } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if !bg1_main_col.transparent {
    //         (bg1_main_col.color, ColorLayer::Bg1)
    //     } else if !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if !bg2_main_col.transparent {
    //         (bg2_main_col.color, ColorLayer::Bg2)
    //     } else {
    //         (bus.cgram[0], ColorLayer::Back)
    //     };

    //     let sub_col = if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
    //         bg1_sub_col.color
    //     } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
    //         bg2_sub_col.color
    //     } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if !bg1_sub_col.transparent {
    //         bg1_sub_col.color
    //     } else if !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if !bg2_sub_col.transparent {
    //         bg2_sub_col.color
    //     } else {
    //         bus.cgram[0]
    //     };

    //     let cmath_en = match main_layer {
    //         ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
    //         ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
    //         ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.palette >= 4,
    //         ColorLayer::Back => bus.ppu_regs.back_cmath_en,
    //         _ => unreachable!(), // No other layers considered in Mode 2
    //     };

    //     DotColorData {
    //         main_col,
    //         sub_col,
    //         cmath_en,
    //     }
    // }

    // fn bg_mode4_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) -> DotColorData {
    //     const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
    //     const BG1_COL_DEPTH: ColorDepth = ColorDepth::Bpp8;
    //     const BG2_COL_DEPTH: ColorDepth = ColorDepth::Bpp4;

    //     let (obj_main_col, obj_sub_col) = self.obj_layer_colors(bus);
    //     let (bg1_main_col, bg1_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG1_COL_DEPTH,
    //         BG1_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg1
    //     );
    //     let (bg2_main_col, bg2_sub_col) = self.bg_layer_colors(
    //         bus,
    //         BG2_COL_DEPTH,
    //         BG2_CGRAM_BASE_ADDR,
    //         ColorLayer::Bg2
    //     );

    //     let (main_col, main_layer) = if obj_main_col.priority == 3 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
    //         (bg1_main_col.color, ColorLayer::Bg1)
    //     } else if obj_main_col.priority == 2 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
    //         (bg2_main_col.color, ColorLayer::Bg2)
    //     } else if obj_main_col.priority == 1 && !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if !bg1_main_col.transparent {
    //         (bg1_main_col.color, ColorLayer::Bg1)
    //     } else if !obj_main_col.transparent {
    //         (obj_main_col.color, ColorLayer::Obj)
    //     } else if !bg2_main_col.transparent {
    //         (bg2_main_col.color, ColorLayer::Bg2)
    //     } else {
    //         (bus.cgram[0], ColorLayer::Back)
    //     };

    //     let sub_col = if obj_sub_col.priority == 3 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
    //         bg1_sub_col.color
    //     } else if obj_sub_col.priority == 2 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
    //         bg2_sub_col.color
    //     } else if obj_sub_col.priority == 1 && !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if !bg1_sub_col.transparent {
    //         bg1_sub_col.color
    //     } else if !obj_sub_col.transparent {
    //         obj_sub_col.color
    //     } else if !bg2_sub_col.transparent {
    //         bg2_sub_col.color
    //     } else {
    //         bus.cgram[0]
    //     };

    //     let cmath_en = match main_layer {
    //         ColorLayer::Bg1 => bus.ppu_regs.bg_settings[0].cmath_en,
    //         ColorLayer::Bg2 => bus.ppu_regs.bg_settings[1].cmath_en,
    //         ColorLayer::Obj => bus.ppu_regs.obj_settings.cmath_en && obj_main_col.palette >= 4,
    //         ColorLayer::Back => bus.ppu_regs.back_cmath_en,
    //         _ => unreachable!(), // No other layers considered in Mode 2
    //     };

    //     DotColorData {
    //         main_col,
    //         sub_col,
    //         cmath_en,
    //     }
    // }

    // fn bg_mode5_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) -> DotColorData {
    //     todo!()
    // }

    // fn bg_mode6_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) -> DotColorData {
    //     todo!()
    // }

    // fn bg_mode7_dot<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) -> DotColorData {
    //     todo!()
    // }

    // fn bg_col<H: DebugHarness>(
    //     &mut self,
    //     bus: &mut PpuBus<H>,
    //     bg_layer: ColorLayer,
    //     color_depth: ColorDepth,
    //     bg_cgram_base_addr: u8,
    // ) -> ColorData {
    //     let bg_data = Self::fetch_bg_data(bus.ppu_regs, bg_layer);

    //     let tile_data = match bus.ppu_regs.bg_mode {
    //         BgMode::Mode0 | BgMode::Mode1 | BgMode::Mode2 | BgMode::Mode3 | BgMode::Mode4 => {
    //             self.bg_tile_idx(bus.ppu_regs, bg_data, self.x, self.y)
    //         }
    //         BgMode::Mode5 | BgMode::Mode6 => todo!(),
    //         BgMode::Mode7 => todo!(),
    //     };

    //     let bg_idx = match bg_layer {
    //         ColorLayer::Bg1 => 0,
    //         ColorLayer::Bg2 => 1,
    //         ColorLayer::Bg3 => 2,
    //         ColorLayer::Bg4 => 3,
    //         _ => unreachable!(),
    //     };

    //     let cache_hit = self.bg_tile_cache[bg_idx].get_entry_idx(&tile_data);

    //     let cache_idx = if let Some(idx) = cache_hit {
    //         idx
    //     } else {
    //         let chr_data = self.fetch_chr_data(bus.ppu_regs, bus.vram, tile_data.clone());

    //         let chr_base = bg_data.chr_base_addr;

    //         let pal_indices = match color_depth {
    //             ColorDepth::Bpp2 => {
    //                 let addr = chr_base + (chr_data.chr_idx << 3) + chr_data.chr_row as u16;
    //                 let bp10 = bus.vram[(addr & 0x7FFF) as usize];

    //                 interleave_2bpp(bp10) as u64
    //             }
    //             ColorDepth::Bpp4 => {
    //                 let addr = chr_base + (chr_data.chr_idx << 4) + chr_data.chr_row as u16;
    //                 let bp10 = bus.vram[(addr & 0x7FFF) as usize];
    //                 let bp32 = bus.vram[((addr + 8) & 0x7FFF) as usize];

    //                 interleave_4bpp(bp10, bp32) as u64
    //             }
    //             ColorDepth::Bpp8 => {
    //                 let addr = chr_base + (chr_data.chr_idx << 5) + chr_data.chr_row as u16;
    //                 if bus.ppu_regs.use_direct_col {
    //                     // Store only the row base addr; vram must be read per dot
    //                     addr as u64
    //                 } else {
    //                     let bp10 = bus.vram[(addr & 0x7FFF) as usize];
    //                     let bp32 = bus.vram[((addr + 8) & 0x7FFF) as usize];
    //                     let bp54 = bus.vram[((addr + 16) & 0x7FFF) as usize];
    //                     let bp76 = bus.vram[((addr + 24) & 0x7FFF) as usize];

    //                     interleave_8bpp(bp10, bp32, bp54, bp76)
    //                 }
    //             }
    //         };

    //         self.bg_tile_cache[bg_idx].cache_tile(
    //             TileRowCacheEntry {
    //                 valid: true,
    //                 tile_addr: tile_data.tile_addr,
    //                 tile_row_key: tile_data.tile_row,
    //                 tile_col_block: tile_data.tile_col / 8,
    //                 chr_data,
    //                 pal_indices,
    //             }
    //         )
    //     };

    //     // Recompute chr_col for this dot using cached flip_x and tile_width.
    //     // We cannot use c.chr_data.chr_col — it was computed for the tile_col at fill time.
    //     let t = &self.bg_tile_cache[bg_idx].entries[cache_idx];
    //     let raw_col = tile_data.tile_col; // 0..tile_width, pre-flip
    //     let flipped_col = if t.chr_data.flip_x {
    //         t.chr_data.tile_width - raw_col - 1
    //     } else {
    //         raw_col
    //     };
    //     let chr_col = flipped_col % 8; // sub-tile column within the 8px bitplane word

    //     match color_depth {
    //         ColorDepth::Bpp2 => Self::extract_2bpp(t, chr_col, bus.cgram, bg_cgram_base_addr),
    //         ColorDepth::Bpp4 => Self::extract_4bpp(t, chr_col, bus.cgram, bg_cgram_base_addr),
    //         ColorDepth::Bpp8 => Self::extract_8bpp(t, chr_col, bus.cgram, bus.ppu_regs, bus.vram),
    //     }
    // }

    /// For modes 0-4
    // pub fn bg_tile_idx(
    //     &self,
    //     regs: &PpuRegs,
    //     bg_data: &BgSettings,
    //     x: usize,
    //     y: usize,
    // ) -> TileData {
    //     let (mosaic_x, mosaic_y) = Self::apply_mosaic(x, y, regs.mosaic_size as usize);

    //     let scroll_range = match bg_data.chr_size {
    //         TileSize::Size8x8 => 0x1FF,
    //         TileSize::Size16x16 => 0x3FF,
    //     };

    //     let shifted_x = ((mosaic_x as u16) + bg_data.scroll_x) & scroll_range;
    //     let shifted_y = ((mosaic_y as u16) + bg_data.scroll_y) & scroll_range;

    //     let tilemap_offset = Self::tm_offset(shifted_x, shifted_y, bg_data.tilemap_cnt_x, bg_data.tilemap_cnt_y);

    //     let x = shifted_x & 0xFF;
    //     let y = shifted_y & 0xFF;

    //     let tile_idx = match bg_data.chr_size {
    //         TileSize::Size8x8 => ((y >> 3) << 5) | (x >> 3),
    //         TileSize::Size16x16 => ((y >> 4) << 5) | (x >> 4),
    //     };

    //     let (tile_col, tile_row) = match bg_data.chr_size {
    //         TileSize::Size8x8 => (x & 7, y & 7),
    //         TileSize::Size16x16 => (x & 0xF, y & 0xF),
    //     };

    //     TileData {
    //         tile_addr: bg_data.tilemap_base_addr + tilemap_offset + tile_idx,
    //         tile_row: tile_row as u8,
    //         tile_col: tile_col as u8,
    //         tile_size: bg_data.chr_size,
    //     }
    // }

    fn apply_mosaic(value: u16, mosaic: u16) -> u16 {
        if mosaic == 0 {
            return value;
        }

        // If m+1 is power of 2
        if (mosaic + 1) & mosaic == 0 {
            return value & !mosaic; // Same as x - x & m, which is same as x - (x % (m+1)) for powers of 2
        }

        value - (value % (mosaic + 1))
    }

    fn tm_offset(x: u16, y: u16, cnt_x: TilemapCount, cnt_y: TilemapCount) -> u16 {
        if cnt_x == cnt_y {
            if cnt_x == TilemapCount::One {
                return 0;
            }
            return ((x & 0x100) << 2) | ((y & 0x100) << 3);
        }

        if cnt_y == TilemapCount::One {
            return (x & 0x100) << 2;
        }

        (y & 0x100) << 2
    }

    /// For modes 5-6
    fn hi_res_bg_tile_idx<H: DebugHarness>(&self, bus: &mut PpuBus<H>, bg_data: &BgSettings) -> TileData {
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
        let tile_word = vram[(tile_data.tile_addr & 0x7FFF) as usize];

        let in_true_hi_res_mode = matches!(regs.bg_mode, BgMode::Mode5 | BgMode::Mode6);

        let (tile_height, tile_width) = match tile_data.tile_size {
            ChrSize::Size8x8 => (8u8, if in_true_hi_res_mode { 16u8 } else { 8u8 }),
            ChrSize::Size16x16 => (16u8, 16u8),
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
            (tile_chr_idx + 16, tile_row - 8) // TODO: Verify +16 or +32?
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

    fn color_window_signals(regs: &PpuRegs) -> (bool, bool) {        
        let apply_col_window = regs.col_apply_window_signal;
        let main_region = regs.col_win_main_region;
        let sub_region = regs.col_win_sub_region;

        let apply_col_win_main = match main_region {
            WindowColorRegion::Nowhere => false,
            WindowColorRegion::Inside => apply_col_window,
            WindowColorRegion::Outside => !apply_col_window,
            WindowColorRegion::Everywhere => true,
        };

        let apply_col_win_sub = match sub_region {
            WindowColorRegion::Nowhere => false,
            WindowColorRegion::Inside => apply_col_window,
            WindowColorRegion::Outside => !apply_col_window,
            WindowColorRegion::Everywhere => true,
        };

        (apply_col_win_main, apply_col_win_sub)
    }

    #[inline(always)]
    fn apply_cmath<H: DebugHarness>(&self, bus: &PpuBus<H>, main_col: Color, sub_col: Option<Color>) -> Color {
        // addend bit: sub_color_fixed==true means addend=0 (always fixed color).
        // addend=1 means "use subscreen", falling back to fixed color only when
        // the subscreen pixel is transparent -- and in that fallback case Div2
        // is forced off regardless of the cmath_half setting.
        let (operand, force_no_div2) = if bus.ppu_regs.sub_color_fixed {
            (bus.ppu_regs.fixed_color, false)
        } else {
            match sub_col {
                Some(c) => (c, false),
                None => (bus.cgram[0], true),
            }
        };
        
        let mut color = match bus.ppu_regs.cmath_operator {
            CMathOperator::Add => Color {
                r: main_col.r.saturating_add(operand.r) & 0xF8, // 5-bit color max
                g: main_col.g.saturating_add(operand.g) & 0xF8,
                b: main_col.b.saturating_add(operand.b) & 0xF8,
            },
            CMathOperator::Subtract => Color {
                r: main_col.r.saturating_sub(operand.r),
                g: main_col.g.saturating_sub(operand.g),
                b: main_col.b.saturating_sub(operand.b),
            },
        };

        if bus.ppu_regs.cmath_half && !force_no_div2 {
            color.r = (color.r >> 1) & 0xF8;
            color.g = (color.g >> 1) & 0xF8;
            color.b = (color.b >> 1) & 0xF8;
        }

        color
    }

    // fn win_active_signal(
    //     in_w1: bool,
    //     in_w2: bool,
    //     win_settings: &WindowSettings,
    // ) -> bool {
    //     let w1_en = win_settings.w1_en && (in_w1 ^ win_settings.w1_inv);
    //     let w2_en = win_settings.w2_en && (in_w2 ^ win_settings.w2_inv);

    //     let win_en = if win_settings.w1_en && win_settings.w2_en {
    //         match win_settings.logic {
    //             WindowLogic::Or => w1_en || w2_en,
    //             WindowLogic::And => w1_en && w2_en,
    //             WindowLogic::Xor => w1_en ^ w2_en,
    //             WindowLogic::Xnor => !(w1_en ^ w2_en),
    //         }
    //     } else if win_settings.w1_en {
    //         w1_en
    //     } else if win_settings.w2_en {
    //         w2_en
    //     } else {
    //         false
    //     };

    //     win_en
    // }

    // fn bg_layer_colors<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>, col_depth: ColorDepth, cgram_base_addr: u8, bg_layer: ColorLayer) -> (ColorData, ColorData) {
    //     let bg_data = Self::fetch_bg_data(bus.ppu_regs, bg_layer);

    //     let win_en = Self::win_active_signal(self.in_w1, self.in_w2, &bg_data.window);
        
    //     let win_main = bg_data.window.main_en && win_en;
    //     let win_sub = bg_data.window.sub_en && win_en;

    //     let mut bg_main_col = None;
    //     let mut bg_sub_col = None;

    //     if bg_data.main_en && !win_main {
    //         bg_main_col = Some(self.bg_col(
    //             bus,
    //             bg_layer, col_depth,
    //             cgram_base_addr
    //         ));
    //     }

    //     let bg_data = Self::fetch_bg_data(bus.ppu_regs, bg_layer);

    //     if bg_data.sub_en && !win_sub {
    //         bg_sub_col = Some(match bg_main_col {
    //                 Some(c) => c,
    //                 None => self.bg_col(
    //                     bus,
    //                     bg_layer, col_depth,
    //                     cgram_base_addr
    //                 ),
    //             });
    //     }

    //     let bg_main_col = match bg_main_col {
    //         Some(c) => c,
    //         None => self.transparent_color_data(bus),
    //     };
    //     let bg_sub_col = match bg_sub_col {
    //         Some(c) => c,
    //         None => self.transparent_color_data(bus),
    //     };

    //     (bg_main_col, bg_sub_col)
    // }

    // fn obj_layer_colors<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) -> (ObjColorData, ObjColorData) {
    //     let win_en = Self::win_active_signal(self.in_w1, self.in_w2, &bus.ppu_regs.obj_settings.window);

    //     let obj_win_main = bus.ppu_regs.obj_settings.window.main_en && win_en;
    //     let obj_win_sub = bus.ppu_regs.obj_settings.window.sub_en && win_en;

    //     let mut obj_main_col = self.transparent_obj_color_data(bus);
    //     let mut obj_sub_col = self.transparent_obj_color_data(bus);
        
    //     if let Some(obj_col_data) = self.scanline_sprite_data[self.x] {
    //         if bus.ppu_regs.obj_settings.main_en && !obj_win_main {
    //             obj_main_col = obj_col_data;
    //         }
            
    //         if bus.ppu_regs.obj_settings.sub_en && !obj_win_sub {
    //             obj_sub_col = obj_col_data;
    //         }
    //     }

    //     (obj_main_col, obj_sub_col)
    // }

    fn transparent_color_data<H: DebugHarness>(&self, bus: &mut PpuBus<H>) -> ColorData {
        ColorData {
            color: bus.cgram[0],
            priority: 0,
            transparent: true,
        }
    }

    fn transparent_obj_color_data<H: DebugHarness>(&self, bus: &mut PpuBus<H>) -> ObjColorData {
        ObjColorData {
            color: bus.cgram[0],
            palette: 0,
            priority: 0,
        }
    }

    fn update_dot_and_scanline<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
        let cpu_regs = &mut bus.cpu_regs;

        self.dot += 1;
        self.x = self.screen_x();

        // Reset the number of dots rendered for each bg layer on each scanline
        if self.x == 0 {
            self.scanline_bg_counters.fill(0);
        }

        let in_w1_old = bus.ppu_regs.in_w1;
        let in_w2_old = bus.ppu_regs.in_w2;

        bus.ppu_regs.in_w1 = bus.ppu_regs.w1_left_pos as usize <= self.x && self.x <= bus.ppu_regs.w1_right_pos as usize;
        bus.ppu_regs.in_w2 = bus.ppu_regs.w2_left_pos as usize <= self.x && self.x <= bus.ppu_regs.w2_right_pos as usize;

        if (in_w1_old != bus.ppu_regs.in_w1) || (in_w2_old != bus.ppu_regs.in_w2) {
            bus.ppu_regs.update_all_in_window_signals();
        }

        if self.dot == SCANLINE_END_DOT {
            self.dot = 0;
            self.scanline += 1;
            self.y = self.screen_y();

            if self.scanline == VBLANK_END_SCANLINE_NTSC {
                self.scanline = 0;
            }
        }

        // End of v-blank, scanline 0 is not visible
        if self.dot == 0 && self.scanline == 0 {
            cpu_regs.vblank_flag = false;
            cpu_regs.vblank_nmi_flag = false;
            *bus.vblank_end = true;
            bus.ppu_regs.sprite_overflow = false;
            bus.ppu_regs.sprite_tile_overflow = false;
        }

        // End of h-blank
        if self.dot == VISIBLE_SCANLINE_START_DOT {
            cpu_regs.hblank_flag = false;
            *bus.hblank_end = true;

            // Start of visible scanline
            if 0 < self.scanline && self.scanline < VBLANK_START_SCANLINE {
                self.render_scanline_sprites(bus);
            }
        }

        let cpu_regs = &mut bus.cpu_regs; // repeat to appease the borrow checker

        // Start of h-blank
        if self.dot == HBLANK_START_DOT && self.scanline < VBLANK_START_SCANLINE {
            cpu_regs.hblank_flag = true;
            *bus.hblank_start = true;
        }

        // Start of v-blank
        if self.dot == 0 && self.scanline == VBLANK_START_SCANLINE {
            cpu_regs.vblank_flag = true;
            cpu_regs.vblank_nmi_flag = true;
            *bus.vblank_start = true;

            if cpu_regs.vblank_nmi_en {
                bus.cpu_regs.nmi_pending = true;
            }

            self.frame += 1;
            *bus.frame_ready = true;
        }
    }

    fn update_hv_timers<H: DebugHarness>(&self, bus: &mut PpuBus<H>) {
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
        }
    }

    /// Finds all possible sprites that could be rendered on the current scanline
    /// based on the y-positions of the sprites
    fn render_scanline_sprites<H: DebugHarness>(&mut self, bus: &mut PpuBus<H>) {
        let regs = &mut bus.ppu_regs;

        self.scanline_sprite_data.fill(None);
        self.scanline_sprites.clear();

        let true_y = if regs.screen_interlace_en && regs.obj_interlace_en {
            2 * self.y + (self.frame & 1)
        } else {
            self.y
        };

        'find_scanline_sprites: for (sprite_idx, sprite) in bus.oam.iter().enumerate() {
            let (spr_w, spr_h) = sprite.sprite_size(regs.obj_sprite_size);

            let max_x = sprite.x + spr_w as i16;
            let max_y = sprite.y as usize + spr_h;

            let in_y_range = if max_y >= 256 {
                // If the sprite wraps around to top of screen, then we hit the sprite if it starts
                // above this scanline OR it wraps to below this scanline.
                sprite.y as usize <= true_y || true_y < (max_y & 0xFF)
            } else {
                sprite.y as usize <= true_y && true_y < max_y
            };

            let in_x_range = 0 < max_x;

            if self.y == 0 && !in_x_range {
                log::debug!("Sprite {sprite_idx} out of bounds at x={}, w={}, max_x={}", sprite.x, spr_w, max_x);
            }

            // Sprite should be on scanline
            if in_y_range && in_x_range {
                if self.scanline_sprites.len() == 32 {
                    regs.sprite_overflow = true;
                    break 'find_scanline_sprites;
                }

                self.scanline_sprites.push(sprite_idx);
            }
        }

        let mut num_slivers = 0;

        for &sprite_idx in self.scanline_sprites.iter().rev() {
            let sprite = &bus.oam[sprite_idx];

            let (spr_w, spr_h) = sprite.sprite_size(regs.obj_sprite_size);

            let sprite_slivers = spr_w / 8;
            num_slivers += sprite_slivers;

            let sprite_row = self.y - sprite.y as usize;

            let sprite_row = if regs.obj_interlace_en {
                2 * sprite_row + (self.frame & 1)
            } else {
                sprite_row
            };
            
            let sprite_row = if sprite.flip_y {
                spr_h as u8 - sprite_row as u8 - 1
            } else {
                sprite_row as u8
            };

            let (tile_y, tile_row) = (sprite_row / 8, sprite_row % 8);

            let obj_table_base_addr = if sprite.use_second_obj_table {
                regs.name_secondary_base_addr
            } else {
                regs.name_base_addr
            };
            
            let spr_tile_base_addr = (obj_table_base_addr as u16) + ((sprite.tile_idx as u16) << 4);
            
            'draw_sprite_slivers: for sliver in 0..sprite_slivers {
                let first_pixel_of_sliver = sprite.x + sliver as i16 * 8;
                let last_pixel_of_sliver = first_pixel_of_sliver + 7;

                if last_pixel_of_sliver < 0 || first_pixel_of_sliver >= 256 {
                    // No part of this sliver will be drawn, skip
                    continue 'draw_sprite_slivers;
                } 

                let tile_x = if sprite.flip_x {
                    sprite_slivers - sliver - 1
                } else {
                    sliver
                };

                let chr_idx = (tile_y << 4) + tile_x as u8;
                
                let spr_tile_addr = (spr_tile_base_addr + ((chr_idx as u16) << 4)) & 0x7FFF;
                let spr_tile_row_addr = spr_tile_addr + tile_row as u16;

                let bp10 = bus.vram[(spr_tile_row_addr as usize) + 0];
                let bp32 = bus.vram[(spr_tile_row_addr as usize) + 8];

                let bitplanes = interleave_4bpp(bp10, bp32);

                'draw_sprite_pixel: for tile_col in 0..8 {
                    let x = sprite.x + sliver as i16 * 8 + tile_col;
    
                    if x < 0 || x >= 256 {
                        // Pixel not drawn, skip
                        continue 'draw_sprite_pixel;
                    }

                    let tile_col = if sprite.flip_x {
                        7 - tile_col
                    } else {
                        tile_col
                    };

                    let pal_idx = (bitplanes >> ((7 - tile_col) * 4)) & 0xF;

                    // Transparent sprite
                    if pal_idx == 0 {
                        continue;
                    }

                    let cgram_addr = 0x80 | (sprite.palette << 4) | pal_idx as u8;

                    let spr_col = bus.cgram[cgram_addr as usize];

                    self.scanline_sprite_data[x as usize] = Some(ObjColorData {
                        color: spr_col,
                        palette: sprite.palette,
                        priority: sprite.priority,
                    });
                }
            }
        }

        if num_slivers > 34 {
            bus.ppu_regs.sprite_tile_overflow = true;
        }
    }

    fn screen_x(&self) -> usize {
        self.dot - VISIBLE_SCANLINE_START_DOT
    }

    fn screen_y(&self) -> usize {
        self.scanline - 1
    }
}