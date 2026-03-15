use crate::core::sppu::color::Color;
use crate::core::sppu::types::*;

/// Contains all of the shared data (registers, memory, etc.) between the S-CPU and S-PPU.
#[derive(Default)]
pub struct PpuRegs {
    pub in_vblank: bool,
    pub h_counter: u16,
    pub v_counter: u16,

    // $2100    F... BBBB    Write only
    //       - Forced blanking (F)
    //       - Screen brightness (B)
    pub in_fblank: bool,
    pub screen_brightness: u8,

    // $2101    SSSN NbBB    Write only
    //       - OBJ sprite size (S)
    //       - Name secondary select (N)
    //       - Name base address (B)
    pub obj_sprite_size: ObjectSizeSelect,
    pub name_secondary_select: u8,
    pub name_base_addr: u8,

    // $2102    AAAA AAAA
    // $2103    P... ...B    Write x2 Only
    //       - OAM word address (A)
    //       - Priority rotation (P)
    //       - Address high bit / table select (B)
    pub oam_write_high_table: bool,
    pub internal_oam_addr: u16,
    pub priority_rotation: bool,
    pub priority_rotation_idx: u8,

    // $2104    DDDD DDDD    Write x2 Only
    //       - OAM data write byte (2x for word) (D), increments OAMADD byte
    pub oam_data_latch: u8,

    // $2105    4321 PMMM    Write Only
    //       - Tilemap tile size (#)
    //       - BG3 priority (P)
    //       - BG mode (M)
    pub bg4_char_size: TileSize,
    pub bg3_char_size: TileSize,
    pub bg2_char_size: TileSize,
    pub bg1_char_size: TileSize,
    pub bg3_mode1_priority: bool,
    pub bg_mode: BgMode,

    // $2106    SSSS 4321    Write Only
    //       - Mosaic size (S)
    //       - Mosaic BG enable (#)
    pub mosaic_size: u8,
    pub bg4_mosaic_en: bool,
    pub bg3_mosaic_en: bool,
    pub bg2_mosaic_en: bool,
    pub bg1_mosaic_en: bool,

    // $2107    AAAA AAYX    Write Only
    //       - BG1 Tilemap VRAM address (A)
    //       - BG1 Vertical tilemap count (Y)
    //       - BG1 Horizontal tilemap count (X)
    pub bg1_vram_addr: u8,
    pub bg1_tilemap_count_y: TilemapCount,
    pub bg1_tilemap_count_x: TilemapCount,

    // $2108    AAAA AAYX    Write Only
    //       - BG2 Tilemap VRAM address (A)
    //       - BG2 Vertical tilemap count (Y)
    //       - BG2 Horizontal tilemap count (X)
    pub bg2_vram_addr: u8,
    pub bg2_tilemap_count_y: TilemapCount,
    pub bg2_tilemap_count_x: TilemapCount,

    // $2109    AAAA AAYX    Write Only
    //       - BG3 Tilemap VRAM address (A)
    //       - BG3 Vertical tilemap count (Y)
    //       - BG3 Horizontal tilemap count (X)
    pub bg3_vram_addr: u8,
    pub bg3_tilemap_count_y: TilemapCount,
    pub bg3_tilemap_count_x: TilemapCount,

    // $210A    AAAA AAYX    Write Only
    //       - BG4 Tilemap VRAM address (A)
    //       - BG4 Vertical tilemap count (Y)
    //       - BG4 Horizontal tilemap count (X)
    pub bg4_vram_addr: u8,
    pub bg4_tilemap_count_y: TilemapCount,
    pub bg4_tilemap_count_x: TilemapCount,

    // $210B    BBBB AAAA    W8
    //       - BG2 CHR base address (B)
    //       - BG1 CHR base address (A)
    pub bg2_chr_base_addr: u8,
    pub bg1_chr_base_addr: u8,

    // $210C    DDDD CCCC    W8
    //       - BG4 CHR base address (D)
    //       - BG3 CHR base address (C)
    pub bg4_chr_base_addr: u8,
    pub bg3_chr_base_addr: u8,

    // Used for many registers affecting mode 7 behavior
    pub m7_latch: u8,

    // Used for all scroll offset registers
    pub bg_offset_latch: u8,
    pub bg_offset_x_latch: u8,

    // $210D    ...x xxXX LLLL LLLL    Write x2 Only
    //       - BG1 or Mode 7 horizontal scroll (X)
    //       - mode7_data_latch (L), writing sets new data latch to (...x xxXX)
    pub bg1_m7_x_offset: u16,

    // $210E    ...y yyYY LLLL LLLL    Write x2 Only
    //       - BG1 or Mode 7 vertical scroll (Y)
    //       - mode7_data_latch (L), writing sets new data latch to (.... ..YY)
    pub bg1_m7_y_offset: u16,

    // $210F    .... ..XX XXXX XXXX    Write x2 Only
    //       - BG2 horizontal scroll (X)
    pub bg2_x_offset: u16,

    // $2110    .... ..YY YYYY YYYY    Write x2 Only
    //       - BG2 vertical scroll (Y)
    pub bg2_y_offset: u16,

    // $2111    .... ..XX XXXX XXXX    Write x2 Only
    //       - BG3 horizontal scroll (X)
    pub bg3_x_offset: u16,

    // $2112    .... ..YY YYYY YYYY    Write x2 Only
    //       - BG3 vertical scroll (Y)
    pub bg3_y_offset: u16,

    // $2113    .... ..XX XXXX XXXX    Write x2 Only
    //       - BG4 horizontal scroll (X)
    pub bg4_x_offset: u16,

    // $2114    .... ..YY YYYY YYYY    Write x2 Only
    //       - BG4 vertical scroll (Y)
    pub bg4_y_offset: u16,

    // $2115    M... RRII    W8
    //       - VRAM address increment mode (M)
    //       - Remapping (R)
    //       - Increment size (I)
    pub vram_addr_inc_mode: VramIncMode,
    pub addr_remap_mode: AddressRemapping,
    pub addr_inc_size: IncrSize,

    // $2116    LLLL LLLL
    // $2117    hHHH HHHH    Write x2 Only
    //       - VRAM word address Low (L)
    //       - VRAM word address High (H)
    pub vram_addr: u16,

    // $211A    RF.. ..YX    W8
    //       - Mode 7 tilemap repeat (R)
    //       - Mode 7 non-repeat fill (F)
    //       - Mode 7 Flip vertical (Y)
    //       - Mode 7 Flip horizontal (X)
    pub m7_tilemap_repeat: bool,
    pub m7_fill_mode: M7FillMode,
    pub m7_flip_bg_y: bool,
    pub m7_flip_bg_x: bool,

    // $211B    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix A or signed 16-bit multiplication factor (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    pub m7_matrix_a: u16,
    pub mult_factor_16: u16,

    // $211C    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix B or signed 8-bit multiplication factor (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    pub m7_matrix_b: u16,
    pub mult_factor_8: u8,

    // $211D    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix C (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    pub m7_matrix_c: u16,

    // $211E    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix D (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    pub m7_matrix_d: u16,

    // $211F    ...X XXXX LLLL LLLL    Write Only
    //       - Mode 7 center X (X)
    //       - mode7_data_latch (L), writing sets new data latch to (...X XXXX)
    pub m7_center_x: u16,

    // $2120    ...Y YYYY LLLL LLLL    Write Only
    //       - Mode 7 center Y (Y)
    //       - mode7_data_latch (L), writing sets new data latch to (...Y YYYY)
    pub m7_center_y: u16,

    // Toggle used for $2121 and $2122 (CGRAM registers)
    pub cgram_toggle: bool,

    // $2121    AAAA AAAA    Write Only
    //       - CGRAM word address (A)
    pub cgram_addr: u8,
    pub cgram_latch: u8,

    // $2123    DdCc BbAa    Write Only
    //       - Enable (ABCD) and Invert (abcd) windows for BG1 (AB) and BG2 (CD)
    pub bg2_w2_en: bool,
    pub bg2_w2_inv: bool,
    pub bg2_w1_en: bool,
    pub bg2_w1_inv: bool,
    pub bg1_w2_en: bool,
    pub bg1_w2_inv: bool,
    pub bg1_w1_en: bool,
    pub bg1_w1_inv: bool,

    // $2124    DdCc BbAa    Write Only
    //       - Enable (EFGH) and Invert (efgh) windows for BG3 (EF) and BG2 (GH)
    pub bg4_w2_en: bool,
    pub bg4_w2_inv: bool,
    pub bg4_w1_en: bool,
    pub bg4_w1_inv: bool,
    pub bg3_w2_en: bool,
    pub bg3_w2_inv: bool,
    pub bg3_w1_en: bool,
    pub bg3_w1_inv: bool,

    // $2125    LlKk JjIi    Write Only
    //       - Enable (IJKL) and Invert (ijkl) windows for OBJ (IJ) and color (KL)
    pub col_w2_en: bool,
    pub col_w2_inv: bool,
    pub col_w1_en: bool,
    pub col_w1_inv: bool,
    pub obj_w2_en: bool,
    pub obj_w2_inv: bool,
    pub obj_w1_en: bool,
    pub obj_w1_inv: bool,

    // $2126    LLLL LLLL    Write Only
    //       - Window 1 left position (L)
    pub w1_left_pos: u8,

    // $2127    RRRR RRRR    Write Only
    //       - Window 1 right position (R)
    pub w1_right_pos: u8,

    // $2128    LLLL LLLL    Write Only
    //       - Window 2 left position (L)
    pub w2_left_pos: u8,

    // $2129    RRRR RRRR    Write Only
    //       - Window 2 right position (R)
    pub w2_right_pos: u8,

    // $212A    4433 2211    Write Only
    //       - Window mask logic for BG layers (00=OR, 01=AND, 10=XOR, 11=XNOR)
    pub bg4_win_logic: WindowLogic,
    pub bg3_win_logic: WindowLogic,
    pub bg2_win_logic: WindowLogic,
    pub bg1_win_logic: WindowLogic,

    // $212B    .... CCOO    Write Only
    //       - Window mask logic for OBJ (O) and color (C)
    pub obj_win_logic: WindowLogic,
    pub col_win_logic: WindowLogic,

    // $212C    ...O 4321    Write Only
    //       - Main screen layer enable (#)
    pub obj_main_en: bool,
    pub bg4_main_en: bool,
    pub bg3_main_en: bool,
    pub bg2_main_en: bool,
    pub bg1_main_en: bool,

    // $212D    ...O 4321    Write Only
    //       - Sub screen layer enable (#)
    pub obj_sub_en: bool,
    pub bg4_sub_en: bool,
    pub bg3_sub_en: bool,
    pub bg2_sub_en: bool,
    pub bg1_sub_en: bool,

    // $212E    ...O 4321    Write Only
    //       - Main screen layer window enable
    pub obj_win_main_en: bool,
    pub bg4_win_main_en: bool,
    pub bg3_win_main_en: bool,
    pub bg2_win_main_en: bool,
    pub bg1_win_main_en: bool,

    // $212F    ...O 4321    Write Only
    //       - Sub screen layer window enable
    pub obj_win_sub_en: bool,
    pub bg4_win_sub_en: bool,
    pub bg3_win_sub_en: bool,
    pub bg2_win_sub_en: bool,
    pub bg1_win_sub_en: bool,

    // $2130    MMSS ..AD    Write Only
    //       - main/sub screen color window black/transparent regions (MS)
    //       - fixed/subscreen (A)
    //       - direct color (D)
    pub col_win_main_region: WindowColorRegion,
    pub col_win_sub_region: WindowColorRegion,
    pub sub_color_fixed: bool,
    pub use_direct_col: bool,

    // $2131    MHBO 4321    Write Only
    //       - Color math add/subtract (M)
    //       - half (H)
    //       - backdrop (B)
    //       - layer enable (O4321)
    pub cmath_operator: CMathOperator,
    pub cmath_half: bool,
    pub back_cmath_en: bool,
    pub obj_cmath_en: bool,
    pub bg4_cmath_en: bool,
    pub bg3_cmath_en: bool,
    pub bg2_cmath_en: bool,
    pub bg1_cmath_en: bool,

    // $2132    BGRC CCCC    Write Only
    //       - Fixed color channel select (BGR) and value (C)
    pub fixed_color: Color,

    // $2133    EX.. HOiI    Write Only
    //       - External sync (E)
    //       - EXTBG (X)
    //       - Hi-res (H)
    //       - Overscan (O)
    //       - OBJ interlace (i)
    //       - Screen interlace (I)
    pub _external_sync: bool,
    pub ext_bg_en: bool,
    pub hi_res_en: bool,
    pub overscan_en: bool,
    pub obj_interlace_en: bool,
    pub screen_interlace_en: bool,

    // $2134    LLLL LLLL    Read Only
    // $2135    MMMM MMMM    Read Only
    // $2136    HHHH HHHH    Read Only
    //       - 24-bit signed multiplication result (read 8 bits per register)
    pub multiply_result: u32,

    // $2137    .... ....    Read Only
    //       - Software latch for H/V counters
    // READ CPU OPEN BUS

    // $2138    DDDD DDDD    Read Only
    //       - Read OAM data byte, increments OAMADD byte

    // $2139    LLLL LLLL
    // $213A    HHHH HHHH    Read x2 Only
    //       - VRAM data read. Increments VMADD after read according to VMAIN setting
    pub vram_latch: u16,

    // $213B    .BBB BBGG GGGR RRRR    Read Only
    //       - CGRAM data read, increments CGADD byte address after each write

    // $213C    ...H HHHH HHHH HHHH    Read Only
    //       - Output horizontal counter (latched)
    pub h_counter_toggle: bool,
    pub h_counter_latch: u16,

    // $213D    ...V VVVV VVVV VVVV    Read Only
    //       - Output vertical counter
    pub v_counter_toggle: bool,
    pub v_counter_latch: u16,

    // STAT77    $213E    TRM. VVVV    Read Only
    //       - Sprite overflow (T)
    //       - sprite tile overflow (R)
    //       - master/slave (M)
    //       - PPU1 version (V)
    pub sprite_overflow: bool,
    pub sprite_tile_overflow: bool,
    pub master_slave_state: MasterSlave,
    pub ppu1_version: u8,

    // STAT78    $213F    FL.M VVVV    Read Only
    //       - Interlace field (F)
    //       - counter latch value (L)
    //       - NTSC/PAL (M)
    //       - PPU2 version (V)
    pub interlace_field: bool,
    pub counter_toggle: bool,
    pub video_type: VideoType,
    pub ppu2_version: u8,
}

impl PpuRegs {
    pub fn update_multiply_result(&mut self) {
        let lhs = self.mult_factor_16 as i16;
        let rhs = self.mult_factor_8 as i8;
        let result = ((lhs as i32) * (rhs as i32)) as u32;

        self.multiply_result = result & 0xFFFFFF;
    }

    pub fn get_vram_addr(&self) -> u16 {
        match self.addr_remap_mode {
            AddressRemapping::None => self.vram_addr & 0x7FFF,
            AddressRemapping::ColDepth2 => {
                // rrrrrrrr YYYccccc -> rrrrrrrr cccccYYY
                let r = self.vram_addr & 0x7F00;
                let y = (self.vram_addr & 0x00E0) >> 5;
                let c = (self.vram_addr & 0x1F) << 3;

                r | c | y
            }
            AddressRemapping::ColDepth4 => {
                // rrrrrrrY YYcccccP -> rrrrrrrc ccccPYYY
                let r = self.vram_addr & 0x7E00;
                let y = (self.vram_addr & 0x01C0) >> 6;
                let cp = (self.vram_addr & 0x003F) << 3;

                r | cp | y
            }
            AddressRemapping::ColDepth8 => {
                // rrrrrrYY YcccccPP -> rrrrrrcc cccPPYYY
                let r = self.vram_addr & 0x7C00;
                let y = (self.vram_addr & 0x0380) >> 7;
                let cp = (self.vram_addr & 0x007F) << 3;

                r | cp | y
            }
        }
    }

    pub fn inc_vram_addr(&mut self) {
        let inc = match self.addr_inc_size {
            IncrSize::Bytes2 => 1,
            IncrSize::Bytes64 => 32,
            IncrSize::Bytes256 => 128,
        };

        self.vram_addr += inc;
    }
}

// Performs the window logic to determine whether a window is enabled/disabled
// for a particular layer given the window settings for that layer.
// fn window_enable(w1_en: bool, w1_inv: bool, w2_en: bool, w2_inv: bool, win_logic: WindowLogic) -> bool {
//     match win_logic {
//         WindowLogic::Or => (w1_en ^ w1_inv) | (w2_en ^ w2_inv),
//         WindowLogic::And => (w1_en ^ w1_inv) & (w2_en ^ w2_inv),
//         WindowLogic::Xor => (w1_en ^ w1_inv) ^ (w2_en ^ w2_inv),
//         WindowLogic::Xnor => !( (w1_en ^ w1_inv) ^ (w2_en ^ w2_inv) ),
//     }
// }

// impl Ppu5C7x {
//     fn screen_x(&self) -> usize { self.dot as usize - VISIBLE_SCANLINE_START_DOT }
//     fn screen_y(&self) -> usize { self.scanline as usize - 1 }
//     fn transparent_color(&self) -> u16 { self.registers.cgram[0].get() }
//     fn transparent_color_data(&self) -> ColorData {
//         ColorData {
//             raw_color: self.transparent_color(),
//             priority: 0,
//             transparent: true,
//         }
//     }

//     fn draw_dot(&mut self, frame_buffer: &mut [RGB565]) {
//         match self.bg_mode() {
//             BgMode::Mode0
//             | BgMode::Mode1
//             | BgMode::Mode2
//             | BgMode::Mode3
//             | BgMode::Mode4 => self.draw_dot_modes_0to4(frame_buffer),

//             BgMode::Mode5
//             | BgMode::Mode6 => self.draw_dot_modes_5to6(frame_buffer),

//             // BgMode::Mode7 => self.bg_mode7_dot(screen_x, screen_y, spr_col),
//             _ => {}
//         };
//     }

//     fn draw_dot_modes_0to4(&mut self, frame_buffer: &mut [RGB565]) {
//         let screen_x = self.screen_x();
//         let screen_y = self.screen_y();

//         let spr_col = self.sprite_col(screen_x, screen_y);

//         let (main_col, sub_col, cmath_en) = match self.bg_mode() {
//             BgMode::Mode0 => self.bg_mode0_dot(screen_x, screen_y, spr_col),
//             BgMode::Mode1 => self.bg_mode1_dot(screen_x, screen_y, spr_col),
//             BgMode::Mode2 => self.bg_mode2_dot(screen_x, screen_y, spr_col),
//             BgMode::Mode3 => self.bg_mode3_dot(screen_x, screen_y, spr_col),
//             BgMode::Mode4 => self.bg_mode4_dot(screen_x, screen_y, spr_col),
//             // BgMode::Mode5 => self.bg_mode5_dot(screen_x, screen_y, spr_col),
//             // BgMode::Mode6 => self.bg_mode6_dot(screen_x, screen_y, spr_col),
//             // BgMode::Mode7 => self.bg_mode7_dot(screen_x, screen_y, spr_col),
//             _ => (0, 0, false),
//         };

//         let (main_col, sub_col) = if self.in_fblank() {
//             (0, 0)
//         } else {
//             (main_col, sub_col)
//         };

//         let brightness = self.registers.screen_brightness.get();
//         let p = self.frame & 1;

//         if self.screen_interlace_enabled() && self.hi_res_enabled() {
//             let main_col = if cmath_en {
//                 self.apply_cmath(main_col, self.fixed_color(), screen_x)
//             } else {
//                 main_col
//             };
//             let sub_col = if cmath_en {
//                 self.apply_cmath(sub_col, self.fixed_color(), screen_x)
//             } else {
//                 sub_col
//             };
//             let main_col = self.apply_brightness(main_col, brightness);
//             let sub_col = self.apply_brightness(sub_col, brightness);

//             frame_buffer[(2*screen_y + p) * 512 + (2*screen_x + 0)] = RGB565::new_with_raw_value(main_col);
//             frame_buffer[(2*screen_y + p) * 512 + (2*screen_x + 1)] = RGB565::new_with_raw_value(sub_col);
//         } else if self.screen_interlace_enabled() {
//             let dot_col = if cmath_en {
//                 self.apply_cmath(main_col, sub_col, screen_x)
//             } else {
//                 main_col
//             };
//             let dot_col = self.apply_brightness(dot_col, brightness);

//             frame_buffer[(2*screen_y + p) * 512 + (2*screen_x + 0)] = RGB565::new_with_raw_value(dot_col);
//             frame_buffer[(2*screen_y + p) * 512 + (2*screen_x + 1)] = RGB565::new_with_raw_value(dot_col);
//         } else if self.hi_res_enabled() {
//             let main_col = if cmath_en {
//                 self.apply_cmath(main_col, self.fixed_color(), screen_x)
//             } else {
//                 main_col
//             };
//             let sub_col = if cmath_en {
//                 self.apply_cmath(sub_col, self.fixed_color(), screen_x)
//             } else {
//                 sub_col
//             };
//             let main_col = self.apply_brightness(main_col, brightness);
//             let sub_col = self.apply_brightness(sub_col, brightness);

//             frame_buffer[(2*screen_y + 0) * 512 + (2*screen_x + 0)] = RGB565::new_with_raw_value(main_col);
//             frame_buffer[(2*screen_y + 1) * 512 + (2*screen_x + 1)] = RGB565::new_with_raw_value(sub_col);
//         } else {
//             let dot_col = if cmath_en {
//                 self.apply_cmath(main_col, sub_col, screen_x)
//             } else {
//                 main_col
//             };
//             let dot_col = self.apply_brightness(dot_col, brightness);

//             frame_buffer[(2*screen_y + 0) * 512 + (2*screen_x + 0)] = RGB565::new_with_raw_value(dot_col);
//             frame_buffer[(2*screen_y + 0) * 512 + (2*screen_x + 1)] = RGB565::new_with_raw_value(dot_col);
//             frame_buffer[(2*screen_y + 1) * 512 + (2*screen_x + 0)] = RGB565::new_with_raw_value(dot_col);
//             frame_buffer[(2*screen_y + 1) * 512 + (2*screen_x + 1)] = RGB565::new_with_raw_value(dot_col);

//             // frame_buffer[screen_y * 256 + screen_x] = RGB565::new_with_raw_value(dot_col);
//         }
//     }

//     fn draw_dot_modes_5to6(&mut self, frame_buffer: &mut [RGB565]) {
//         let screen_x = self.screen_x();
//         let screen_y = self.screen_y();

//         let spr_col = self.sprite_col(screen_x, screen_y);

//         let (main_col1, sub_col1, cmath_en1) = match self.bg_mode() {
//             BgMode::Mode5 => self.bg_mode5_dot(screen_x + 0, screen_y, spr_col.clone()),
//             // BgMode::Mode6 => self.bg_mode6_dot(2*screen_x + 0, screen_y, spr_col),
//             _ => unreachable!() // Only called for modes 5 & 6
//         };
//         let (main_col2, sub_col2, cmath_en2) = match self.bg_mode() {
//             BgMode::Mode5 => self.bg_mode5_dot(screen_x + 256, screen_y, spr_col),
//             // BgMode::Mode6 => self.bg_mode6_dot(2*screen_x + 1, screen_y, spr_col),
//             _ => unreachable!() // Only called for modes 5 & 6
//         };

//         let brightness = self.registers.screen_brightness.get();
//         let p = self.frame & 1;

//         let dot_col1 = if cmath_en1 {
//             self.apply_cmath(main_col1, sub_col1, screen_x)
//         } else {
//             main_col1
//         };
//         let dot_col2 = if cmath_en2 {
//             self.apply_cmath(main_col2, sub_col2, screen_x)
//         } else {
//             main_col2
//         };

//         let dot_col1 = self.apply_brightness(dot_col1, brightness);
//         let dot_col2 = self.apply_brightness(dot_col2, brightness);

//         if self.screen_interlace_enabled() {
//             frame_buffer[(2*screen_y + p) * 512 + (screen_x + 0)] = RGB565::new_with_raw_value(dot_col1);
//             frame_buffer[(2*screen_y + p) * 512 + (screen_x + 256)] = RGB565::new_with_raw_value(dot_col2);
//         } else {
//             frame_buffer[(2*screen_y + 0) * 512 + (2*screen_x + 0)] = RGB565::new_with_raw_value(dot_col1);
//             frame_buffer[(2*screen_y + 1) * 512 + (2*screen_x + 0)] = RGB565::new_with_raw_value(dot_col1);
//             frame_buffer[(2*screen_y + 0) * 512 + (2*screen_x + 1)] = RGB565::new_with_raw_value(dot_col2);
//             frame_buffer[(2*screen_y + 1) * 512 + (2*screen_x + 1)] = RGB565::new_with_raw_value(dot_col2);
//         }
//     }

//     fn apply_brightness(&self, col: u16, brightness: u8) -> u16 {
//         if brightness == 0 { return 0; }
//         if brightness == 15 { return col; }

//         let (r, g, b) = rgb565_to_parts(col);

//         let r = (r * brightness as u16) / 15;
//         let g = (g * brightness as u16) / 15;
//         let b = (b * brightness as u16) / 15;

//         rgb565_from_parts(r, g, b)
//     }

//     /// Gets the color of the first visible sprite on the screen.
//     fn sprite_col(&mut self, screen_x: usize, screen_y: usize) -> ColorData {
//         let mut scanline_spr_cnt = self.scanline_spr_cnt;

//         if scanline_spr_cnt == 0 {
//             scanline_spr_cnt = 32;
//         }

//         for i in 0..self.scanline_sprites.len() {
//             scanline_spr_cnt -= 1;

//             let sprite = &self.scanline_sprites[scanline_spr_cnt];

//             if scanline_spr_cnt == 0 {
//                 scanline_spr_cnt = 32;
//             }

//             if sprite.x as usize <= screen_x && screen_x < sprite.max_x as usize {
//                 let sprite_col = screen_x - sprite.x as usize;
//                 let sprite_row = screen_y - sprite.y as usize;

//                 let sprite_row = if self.screen_interlace_enabled() && self.obj_interlace_enabled() {
//                     2*sprite_row + (self.frame & 1)
//                 } else {
//                     sprite_row
//                 };

//                 let sprite_col = if sprite.flip_x { sprite.width - sprite_col - 1 } else { sprite_col };
//                 let sprite_row = if sprite.flip_y { sprite.height - sprite_row - 1 } else { sprite_row };

//                 let (tile_x, tile_col) = (sprite_col / 8, sprite_col % 8);
//                 let (tile_y, tile_row) = (sprite_row / 8, sprite_row % 8);

//                 let chr_idx = (tile_y << 4) + tile_x;

//                 let obj_table_base_addr = if sprite.use_second_obj_table {
//                     self.name_base_addr() + self.name_secondary_select()
//                 } else {
//                     self.name_base_addr()
//                 };

//                 let spr_tile_base_addr = obj_table_base_addr + ((sprite.tile_idx as u16) << 4);
//                 let spr_tile_addr = spr_tile_base_addr + ((chr_idx as u16) << 4);
//                 let spr_tile_row_addr = spr_tile_addr + tile_row as u16;

//                 let bp01 = self.vram_read(spr_tile_row_addr + 0);
//                 let bp23 = self.vram_read(spr_tile_row_addr + 8);

//                 let b0 = ((bp01 >> (7-tile_col)) as u8) & 1;
//                 let b1 = ((bp01 >> (15-tile_col)) as u8) & 1;
//                 let b2 = ((bp23 >> (7-tile_col)) as u8) & 1;
//                 let b3 = ((bp23 >> (15-tile_col)) as u8) & 1;

//                 let pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;

//                 // Transparent sprite
//                 if pal_idx == 0 {
//                     // If it's the last sprite, all sprites were transparent
//                     if i == self.scanline_sprites.len() - 1 {
//                         return ColorData {
//                             raw_color: 0,
//                             priority: sprite.priority,
//                             transparent: true,
//                         };
//                     }

//                     continue;
//                 }

//                 let cgram_addr = 0x80 | (sprite.palette << 4) | pal_idx;

//                 let spr_col = self.registers.cgram[cgram_addr as usize].get();

//                 return ColorData {
//                     raw_color: spr_col,
//                     priority: sprite.priority,
//                     transparent: false,
//                 };
//             }
//         }

//         // No sprites on this dot, return a transparent color
//         ColorData {
//             raw_color: self.transparent_color(),
//             priority: 0,
//             transparent: true,
//         }
//     }

//     fn bg_mode0_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {
//         const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
//         const BG2_CGRAM_BASE_ADDR: u8 = 0x20;
//         const BG3_CGRAM_BASE_ADDR: u8 = 0x40;
//         const BG4_CGRAM_BASE_ADDR: u8 = 0x60;

//         let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x);
//         let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x);
//         let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x);
//         let (bg3_win_main, bg3_win_sub) = self.bg3_win_active_signals(screen_x);
//         let (bg4_win_main, bg4_win_sub) = self.bg4_win_active_signals(screen_x);

//         let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(spr_col); // Obj col should not be used past this point

//         let bg1_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg1, ColorDepth::Bpp2,
//             BG1_CGRAM_BASE_ADDR
//         );
//         let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg1_col); // Bg1 col should not be used past this point

//         let bg2_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg2, ColorDepth::Bpp2,
//             BG2_CGRAM_BASE_ADDR
//         );
//         let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg2_col); // Bg2 col should not be used past this point

//         let bg3_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg3, ColorDepth::Bpp2,
//             BG3_CGRAM_BASE_ADDR
//         );
//         let bg3_main_col = if self.bg3_main_enabled() && !bg3_win_main {
//             bg3_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg3_sub_col = if self.bg3_sub_enabled() && !bg3_win_sub {
//             bg3_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg3_col); // Bg3 col should not be used past this point

//         let bg4_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg4, ColorDepth::Bpp2,
//             BG4_CGRAM_BASE_ADDR
//         );
//         let bg4_main_col = if self.bg4_main_enabled() && !bg4_win_main {
//             bg4_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg4_sub_col = if self.bg4_sub_enabled() && !bg4_win_sub {
//             bg4_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg4_col); // Bg3 col should not be used past this point

//         let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
//             (bg3_main_col.raw_color, ColorLayer::Bg3)
//         } else if bg4_main_col.priority != 0 && !bg4_main_col.transparent {
//             (bg4_main_col.raw_color, ColorLayer::Bg4)
//         } else if !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg3_main_col.transparent {
//             (bg3_main_col.raw_color, ColorLayer::Bg3)
//         } else if !bg4_main_col.transparent {
//             (bg4_main_col.raw_color, ColorLayer::Bg4)
//         } else {
//             (self.transparent_color(), ColorLayer::Back)
//         };

//         let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
//             bg3_sub_col.raw_color
//         } else if bg4_sub_col.priority != 0 && !bg4_sub_col.transparent {
//             bg4_sub_col.raw_color
//         } else if !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg3_sub_col.transparent {
//             bg3_sub_col.raw_color
//         } else if !bg4_sub_col.transparent {
//             bg4_sub_col.raw_color
//         } else {
//             self.fixed_color()
//         };

//         let cmath_en = match main_layer {
//             ColorLayer::Bg1 => self.bg1_cmath_enabled(),
//             ColorLayer::Bg2 => self.bg2_cmath_enabled(),
//             ColorLayer::Bg3 => self.bg3_cmath_enabled(),
//             ColorLayer::Bg4 => self.bg4_cmath_enabled(),
//             ColorLayer::Obj => self.obj_cmath_enabled(),
//             ColorLayer::Back => self.back_cmath_enabled(),
//         };

//         (main_col, sub_col, cmath_en)
//     }

//     fn bg_mode1_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {
//         const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
//         const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
//         const BG3_CGRAM_BASE_ADDR: u8 = 0x00;

//         let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x);
//         let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x);
//         let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x);
//         let (bg3_win_main, bg3_win_sub) = self.bg3_win_active_signals(screen_x);

//         let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(spr_col); // Obj col should not be used past this point

//         let bg1_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg1, ColorDepth::Bpp4,
//             BG1_CGRAM_BASE_ADDR
//         );
//         let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg1_col); // Bg1 col should not be used past this point

//         let bg2_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg2, ColorDepth::Bpp4,
//             BG2_CGRAM_BASE_ADDR
//         );
//         let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg2_col); // Bg2 col should not be used past this point

//         let bg3_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg3, ColorDepth::Bpp2,
//             BG3_CGRAM_BASE_ADDR
//         );
//         let bg3_main_col = if self.bg3_main_enabled() && !bg3_win_main {
//             bg3_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg3_sub_col = if self.bg3_sub_enabled() && !bg3_win_sub {
//             bg3_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg3_col); // Bg3 col should not be used past this point

//         let (main_col, main_layer) = if self.bg3_mode1_priority() && bg3_main_col.priority != 0 && !bg3_main_col.transparent {
//             (bg3_main_col.raw_color, ColorLayer::Bg3)
//         } else if spr_main_col.priority == 3 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
//             (bg3_main_col.raw_color, ColorLayer::Bg3)
//         } else if !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg3_main_col.transparent {
//             (bg3_main_col.raw_color, ColorLayer::Bg3)
//         } else {
//             (self.transparent_color(), ColorLayer::Back)
//         };

//         let sub_col = if self.sub_color_fixed() {
//             self.fixed_color()
//         } else if self.bg3_mode1_priority() && bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
//             bg3_sub_col.raw_color
//         } else if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
//             bg3_sub_col.raw_color
//         } else if !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg3_sub_col.transparent {
//             bg3_sub_col.raw_color
//         } else {
//             self.fixed_color()
//         };

//         let cmath_en = match main_layer {
//             ColorLayer::Bg1 => self.bg1_cmath_enabled(),
//             ColorLayer::Bg2 => self.bg2_cmath_enabled(),
//             ColorLayer::Bg3 => self.bg3_cmath_enabled(),
//             ColorLayer::Obj => self.obj_cmath_enabled(),
//             ColorLayer::Back => self.back_cmath_enabled(),
//             _ => unreachable!(), // No other layers considered in Mode 1
//         };

//         (main_col, sub_col, cmath_en)
//     }

//     fn bg_mode2_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {
//         const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
//         const BG2_CGRAM_BASE_ADDR: u8 = 0x00;

//         let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x);
//         let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x);
//         let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x);

//         let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(spr_col); // Obj col should not be used past this point

//         let bg1_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg1, ColorDepth::Bpp4,
//             BG1_CGRAM_BASE_ADDR
//         );
//         let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg1_col); // Bg1 col should not be used past this point

//         let bg2_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg2, ColorDepth::Bpp4,
//             BG2_CGRAM_BASE_ADDR
//         );
//         let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg2_col); // Bg2 col should not be used past this point

//         let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else {
//             (self.transparent_color(), ColorLayer::Back)
//         };

//         let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else {
//             self.transparent_color()
//         };

//         let cmath_en = match main_layer {
//             ColorLayer::Bg1 => self.bg1_cmath_enabled(),
//             ColorLayer::Bg2 => self.bg2_cmath_enabled(),
//             ColorLayer::Obj => self.obj_cmath_enabled(),
//             ColorLayer::Back => self.back_cmath_enabled(),
//             _ => unreachable!(), // No other layers considered in Mode 2
//         };

//         (main_col, sub_col, cmath_en)
//     }

//     fn bg_mode3_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {
//         const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
//         const BG2_CGRAM_BASE_ADDR: u8 = 0x00;

//         let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x);
//         let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x);
//         let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x);

//         let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(spr_col); // Obj col should not be used past this point

//         let bg1_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg1, ColorDepth::Bpp8,
//             BG1_CGRAM_BASE_ADDR
//         );
//         let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg1_col); // Bg1 col should not be used past this point

//         let bg2_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg2, ColorDepth::Bpp4,
//             BG2_CGRAM_BASE_ADDR
//         );
//         let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg2_col); // Bg2 col should not be used past this point

//         let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else {
//             (self.transparent_color(), ColorLayer::Back)
//         };

//         let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else {
//             self.transparent_color()
//         };

//         let cmath_en = match main_layer {
//             ColorLayer::Bg1 => self.bg1_cmath_enabled(),
//             ColorLayer::Bg2 => self.bg2_cmath_enabled(),
//             ColorLayer::Obj => self.obj_cmath_enabled(),
//             ColorLayer::Back => self.back_cmath_enabled(),
//             _ => unreachable!(), // No other layers considered in Mode 2
//         };

//         (main_col, sub_col, cmath_en)
//     }

//     fn bg_mode4_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {
//         const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
//         const BG2_CGRAM_BASE_ADDR: u8 = 0x00;

//         let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x);
//         let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x);
//         let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x);

//         let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(spr_col); // Obj col should not be used past this point

//         let bg1_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg1, ColorDepth::Bpp8,
//             BG1_CGRAM_BASE_ADDR
//         );
//         let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg1_col); // Bg1 col should not be used past this point

//         let bg2_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg2, ColorDepth::Bpp4,
//             BG2_CGRAM_BASE_ADDR
//         );
//         let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg2_col); // Bg2 col should not be used past this point

//         let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else {
//             (self.transparent_color(), ColorLayer::Back)
//         };

//         let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else {
//             self.transparent_color()
//         };

//         let cmath_en = match main_layer {
//             ColorLayer::Bg1 => self.bg1_cmath_enabled(),
//             ColorLayer::Bg2 => self.bg2_cmath_enabled(),
//             ColorLayer::Obj => self.obj_cmath_enabled(),
//             ColorLayer::Back => self.back_cmath_enabled(),
//             _ => unreachable!(), // No other layers considered in Mode 2
//         };

//         (main_col, sub_col, cmath_en)
//     }

//     fn bg_mode5_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {
//         const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
//         const BG2_CGRAM_BASE_ADDR: u8 = 0x00;

//         let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x >> 1);
//         let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x >> 1);
//         let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x >> 1);

//         let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
//             spr_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(spr_col); // Obj col should not be used past this point

//         let bg1_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg1, ColorDepth::Bpp4,
//             BG1_CGRAM_BASE_ADDR
//         );
//         let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
//             bg1_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg1_col); // Bg1 col should not be used past this point

//         let bg2_col = self.bg_col(
//             screen_x, screen_y,
//             ColorLayer::Bg2, ColorDepth::Bpp2,
//             BG2_CGRAM_BASE_ADDR
//         );
//         let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
//             bg2_col.clone()
//         } else {
//             self.transparent_color_data()
//         };
//         drop(bg2_col); // Bg2 col should not be used past this point

//         let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg1_main_col.transparent {
//             (bg1_main_col.raw_color, ColorLayer::Bg1)
//         } else if !spr_main_col.transparent {
//             (spr_main_col.raw_color, ColorLayer::Obj)
//         } else if !bg2_main_col.transparent {
//             (bg2_main_col.raw_color, ColorLayer::Bg2)
//         } else {
//             (self.transparent_color(), ColorLayer::Back) // Main screen color is black if all layers are transparent
//         };

//         let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg1_sub_col.transparent {
//             bg1_sub_col.raw_color
//         } else if !spr_sub_col.transparent {
//             spr_sub_col.raw_color
//         } else if !bg2_sub_col.transparent {
//             bg2_sub_col.raw_color
//         } else {
//             self.fixed_color() // Sub screen color is fixed color if all layers are transparent
//         };

//         let cmath_en = match main_layer {
//             ColorLayer::Bg1 => self.bg1_cmath_enabled(),
//             ColorLayer::Bg2 => self.bg2_cmath_enabled(),
//             ColorLayer::Obj => self.obj_cmath_enabled(),
//             ColorLayer::Back => self.back_cmath_enabled(),
//             _ => unreachable!(), // No other layers considered in Mode 5
//         };

//         (main_col, sub_col, cmath_en)
//     }

//     // fn bg_mode6_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {

//     // }

//     // fn bg_mode7_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> (u16, u16, bool) {

//     // }

//     fn bg_col(&self, screen_x: usize, screen_y: usize,
//         bg_layer: ColorLayer, color_depth: ColorDepth,
//         bg_cgram_base_addr: u8) -> ColorData {

//         let bg_chr_base_addr = match bg_layer {
//             ColorLayer::Bg1 => self.bg1_chr_base_addr(),
//             ColorLayer::Bg2 => self.bg2_chr_base_addr(),
//             ColorLayer::Bg3 => self.bg3_chr_base_addr(),
//             ColorLayer::Bg4 => self.bg4_chr_base_addr(),

//             _ => unreachable!("Should only be called for bg layers")
//         };

//         let tile_data = match self.bg_mode() {
//             BgMode::Mode0
//             | BgMode::Mode1
//             | BgMode::Mode2
//             | BgMode::Mode3
//             | BgMode::Mode4 => self.bg_tile_idx(screen_x, screen_y, bg_layer),

//             BgMode::Mode5
//             | BgMode::Mode6 => self.hi_res_bg_tile_idx(screen_x, screen_y, bg_layer),

//             BgMode::Mode7 => todo!(),
//         };

//         // if screen_y == 111 && screen_x == 240 && bg_layer == ColorLayer::Bg1 {
//             // println!("({screen_x}, {screen_y}): addr: ${:04X}, row: {}, col: {}, size: {:?}",
//             //     tile_data.tile_addr,
//             //     tile_data.tile_row,
//             //     tile_data.tile_col,
//             //     tile_data.tile_size,
//             // );

//             // if tile_data.tile_addr == 0x2CB5 {
//             //     let mut vram_clone = Vec::new();

//             //     for w in self.registers.vram.iter() {
//             //         vram_clone.push(w.get());
//             //     }

//             //     crate::tools::hexdump::hexdump16_to_file(&vram_clone, 0, "vram_dump.txt");

//             //     std::process::exit(0);
//             // }
//         // }

//         let col = match color_depth {
//             ColorDepth::Bpp2 => self.bg_col_2bpp(tile_data, bg_chr_base_addr, bg_cgram_base_addr),
//             ColorDepth::Bpp4 =>  self.bg_col_4bpp(tile_data, bg_chr_base_addr, bg_cgram_base_addr),
//             ColorDepth::Bpp8 => self.bg_col_8bpp(tile_data, bg_chr_base_addr),
//         };

//         // let col = match bg_layer {
//         //     ColorLayer::Bg1 => {
//         //         let (x, col) = (screen_x / 8, screen_x % 8);
//         //         let (y, row) = (screen_y / 8, screen_y % 8);

//         //         let chr_data = ChrData {
//         //             chr_idx: (y*16 + x) as u16,
//         //             chr_col: col as u8,
//         //             chr_row: row as u8,
//         //             chr_pal: 0,
//         //             chr_priority: 0,
//         //         };

//         //         let base_chr_addr = (((self.frame / 15) * 0x100) & (VRAM_SIZE-1)) as u16;

//         //         let tile_chr_addr = base_chr_addr + (chr_data.chr_idx << 4) + chr_data.chr_row as u16;

//         //         let bp01 = self.vram_read(tile_chr_addr + 0);
//         //         let bp23 = self.vram_read(tile_chr_addr + 8);

//         //         let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
//         //         let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;
//         //         let b2 = ((bp23 >> (7-chr_data.chr_col)) & 1) as u8;
//         //         let b3 = ((bp23 >> (15-chr_data.chr_col)) & 1) as u8;

//         //         let pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;

//         //         let cgram_addr = bg_cgram_base_addr | (chr_data.chr_pal << 4) | pal_idx;

//         //         let raw_color = if pal_idx == 0 {
//         //             self.transparent_color()
//         //         } else {
//         //             self.registers.cgram[cgram_addr as usize].get()
//         //         };

//         //         ColorData {
//         //             raw_color,
//         //             priority: chr_data.chr_priority,
//         //             transparent: pal_idx == 0,
//         //         }
//         //     }
//         //     _ => self.transparent_color_data()
//         // };

//         col
//     }

//     /// For modes 0-4
//     fn bg_tile_idx(&self, screen_x: usize, screen_y: usize, bg_layer: ColorLayer) -> TileData {
//         let bg_data = self.fetch_bg_data(bg_layer);

//         let (mosaic_x, mosaic_y) = if bg_data.mosaic_en {
//             let mosaic_mod = (self.mosaic_size() + 1) as usize;

//             (screen_x - (screen_x % mosaic_mod), screen_y - (screen_y % mosaic_mod))
//         } else {
//             (screen_x, screen_y)
//         };

//         let scroll_range = match bg_data.tile_size {
//             TileSize::Size8x8 => 0x1FF,
//             TileSize::Size16x16 => 0x3FF,
//         };

//         let shifted_x = ((mosaic_x as u16) + bg_data.scroll_x) & scroll_range;
//         let shifted_y = ((mosaic_y as u16) + bg_data.scroll_y) & scroll_range;

//         let tilemap_offset = match (bg_data.tilemap_cnt_x, bg_data.tilemap_cnt_y) {
//             (TilemapCount::One, TilemapCount::One) => 0x000,
//             (TilemapCount::One, TilemapCount::Two) => {
//                 if shifted_y >= 256 {
//                     0x400
//                 } else {
//                     0x000
//                 }
//             }
//             (TilemapCount::Two, TilemapCount::One) => {
//                 if shifted_x >= 256 {
//                     0x400
//                 } else {
//                     0x000
//                 }
//             }
//             (TilemapCount::Two, TilemapCount::Two) => {
//                 if shifted_x >= 256 && shifted_y >= 256 {
//                     0xC00
//                 } else if shifted_y >= 256 {
//                     0x800
//                 } else if shifted_x >= 256 {
//                     0x400
//                 } else {
//                     0x000
//                 }
//             }
//         };

//         let x = shifted_x & 0xFF;
//         let y = shifted_y & 0xFF;

//         let tile_idx = match bg_data.tile_size {
//             TileSize::Size8x8 => ((y >> 3) << 5) | (x >> 3),
//             TileSize::Size16x16 => (y & 0xF0) | (x >> 4),
//         };

//         let (tile_col, tile_row) = match bg_data.tile_size {
//             TileSize::Size8x8 => (x & 7, y & 7),
//             TileSize::Size16x16 => (x & 0xF, y & 0xF),
//         };

//         // if screen_x == 4 && screen_y == 4 {
//         //     match bg_layer {
//         //         ColorLayer::Bg1 => {
//         //             println!("tile_addr: ${:04X}, tile_row: {}, tile_col: {}",
//         //                 bg_data.tilemap_base_addr + tilemap_offset + tile_idx,
//         //                 tile_row,
//         //                 tile_col,
//         //             )
//         //         },
//         //         _ => {}
//         //     }
//         // }

//         TileData {
//             tile_addr: bg_data.tilemap_base_addr + tilemap_offset + tile_idx,
//             tile_row: tile_row as u8,
//             tile_col: tile_col as u8,
//             tile_size: bg_data.tile_size
//         }
//     }

//     /// For modes 5-6
//     fn hi_res_bg_tile_idx(&self, screen_x: usize, screen_y: usize, bg_layer: ColorLayer) -> TileData {
//         let bg_data = self.fetch_bg_data(bg_layer);

//         let (mosaic_x, mosaic_y) = if bg_data.mosaic_en {
//             let mosaic_mod = (self.mosaic_size() + 1) as usize;

//             (screen_x - (screen_x % mosaic_mod), screen_y - (screen_y % mosaic_mod))
//         } else {
//             (screen_x, screen_y)
//         };

//         let scroll_range_x = 0x1FF;
//         let scroll_range_y = match bg_data.tile_size {
//             TileSize::Size8x8 => 0x1FF,
//             TileSize::Size16x16 => 0x3FF,
//         };

//         let shifted_x = ((mosaic_x as u16) + bg_data.scroll_x) & scroll_range_x;
//         let shifted_y = ((mosaic_y as u16) + bg_data.scroll_y) & scroll_range_y;

//         let tilemap_offset = match (bg_data.tilemap_cnt_x, bg_data.tilemap_cnt_y) {
//             (TilemapCount::One, TilemapCount::One) => 0x000,
//             (TilemapCount::One, TilemapCount::Two) => {
//                 if shifted_y >= 256 {
//                     0x400
//                 } else {
//                     0x000
//                 }
//             }
//             (TilemapCount::Two, TilemapCount::One) => {
//                 if shifted_x >= 256 {
//                     0x400
//                 } else {
//                     0x000
//                 }
//             }
//             (TilemapCount::Two, TilemapCount::Two) => {
//                 if shifted_x >= 256 && shifted_y >= 256 {
//                     0xC00
//                 } else if shifted_y >= 256 {
//                     0x800
//                 } else if shifted_x >= 256 {
//                     0x400
//                 } else {
//                     0x000
//                 }
//             }
//         };

//         let x = shifted_x & 0x1FF;
//         let y = shifted_y & 0xFF;

//         let tile_idx = match bg_data.tile_size {
//             TileSize::Size8x8 => ((y >> 3) << 5) | (x >> 4),
//             TileSize::Size16x16 => (y & 0xF0) | (x >> 4),
//         };

//         let (tile_col, tile_row) = match bg_data.tile_size {
//             TileSize::Size8x8 => (x & 0xF, y & 7),
//             TileSize::Size16x16 => (x & 0xF, y & 0xF),
//         };

//         TileData {
//             tile_addr: bg_data.tilemap_base_addr + tilemap_offset + tile_idx,
//             tile_row: tile_row as u8,
//             tile_col: tile_col as u8,
//             tile_size: bg_data.tile_size
//         }
//     }

//     fn fetch_bg_data(&self, bg_layer: ColorLayer) -> BgData {
//         match bg_layer {
//             ColorLayer::Bg1 => BgData {
//                 scroll_x: self.bg1_m7_x_offset(),
//                 scroll_y: self.bg1_m7_y_offset(),
//                 tilemap_cnt_x: self.bg1_tilemap_count_x(),
//                 tilemap_cnt_y: self.bg1_tilemap_count_y(),
//                 tile_size: self.bg1_tile_size(),
//                 tilemap_base_addr: self.bg1_vram_base_addr(),
//                 mosaic_en: self.bg1_mosaic_en()
//             },

//             ColorLayer::Bg2 => BgData {
//                 scroll_x: self.bg2_x_offset(),
//                 scroll_y: self.bg2_y_offset(),
//                 tilemap_cnt_x: self.bg2_tilemap_count_x(),
//                 tilemap_cnt_y: self.bg2_tilemap_count_y(),
//                 tile_size: self.bg2_tile_size(),
//                 tilemap_base_addr: self.bg2_vram_base_addr(),
//                 mosaic_en: self.bg2_mosaic_en()
//             },

//             ColorLayer::Bg3 => BgData {
//                 scroll_x: self.bg3_x_offset(),
//                 scroll_y: self.bg3_y_offset(),
//                 tilemap_cnt_x: self.bg3_tilemap_count_x(),
//                 tilemap_cnt_y: self.bg3_tilemap_count_y(),
//                 tile_size: self.bg3_tile_size(),
//                 tilemap_base_addr: self.bg3_vram_base_addr(),
//                 mosaic_en: self.bg3_mosaic_en()
//             },

//             ColorLayer::Bg4 => BgData {
//                 scroll_x: self.bg4_x_offset(),
//                 scroll_y: self.bg4_y_offset(),
//                 tilemap_cnt_x: self.bg4_tilemap_count_x(),
//                 tilemap_cnt_y: self.bg4_tilemap_count_y(),
//                 tile_size: self.bg4_tile_size(),
//                 tilemap_base_addr: self.bg4_vram_base_addr(),
//                 mosaic_en: self.bg4_mosaic_en()
//             },

//             _ => unreachable!() // Only called for bg layers
//         }
//     }

//     fn fetch_chr_data(&self, tile_data: TileData) -> ChrData {
//         let tile_word = self.vram_read(tile_data.tile_addr);

//         let (tile_height, tile_width) = match tile_data.tile_size {
//             TileSize::Size8x8 => (8, if self.in_true_hi_res_mode() { 16 } else { 8 }),
//             TileSize::Size16x16 => (16,16),
//         };

//         let tile_chr_idx = tile_word & 0x3FF;
//         let tile_pal = ((tile_word >> 10) & 7) as u8;
//         let tile_priority = ((tile_word >> 13) & 1) as u8;
//         let flip_x = (tile_word & 0x4000) != 0;
//         let flip_y = (tile_word & 0x8000) != 0;

//         let tile_row = if flip_y { tile_height - tile_data.tile_row - 1 } else { tile_data.tile_row };
//         let tile_col = if flip_x { tile_width - tile_data.tile_col - 1 } else { tile_data.tile_col };

//         let (tile_chr_idx, tile_row) = if tile_row > 7 {
//             (tile_chr_idx + 32, tile_row - 8)
//         } else {
//             (tile_chr_idx, tile_row)
//         };

//         let (tile_chr_idx, tile_col) = if tile_col > 7 {
//             (tile_chr_idx + 1, tile_col - 8)
//         } else {
//             (tile_chr_idx, tile_col)
//         };

//         ChrData {
//             chr_idx: tile_chr_idx,
//             chr_col: tile_col,
//             chr_row: tile_row,
//             chr_pal: tile_pal,
//             chr_priority: tile_priority,
//         }
//     }

//     pub fn dump_vram(&self) {
//         let mut outf = std::fs::File::create("vram_dump.bin").unwrap();

//         let mut vram_clone = Vec::new();
//         for b in self.registers.vram.iter() {
//             vram_clone.push(b.get().get_lo());
//             vram_clone.push(b.get().get_hi());
//         }

//         outf.write(&vram_clone).unwrap();

//         println!("Dumped vram.");

//         let mut vram_clone = Vec::new();
//         for b in self.registers.vram.iter() {
//             vram_clone.push(b.get());
//         }

//         crate::tools::hexdump::hexdump16_to_file(&vram_clone, 0, "vram_dump.txt");
//     }

//     fn bg_col_2bpp(&self, tile_data: TileData, bg_chr_base_addr: u16, bg_cgram_base_addr: u8) -> ColorData {
//         let chr_data = self.fetch_chr_data(tile_data);

//         let tile_chr_addr = bg_chr_base_addr + (chr_data.chr_idx << 3) + chr_data.chr_row as u16;

//         let bp01 = self.vram_read(tile_chr_addr);

//         let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
//         let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;

//         let pal_idx = (b1 << 1) | b0;

//         let cgram_addr = bg_cgram_base_addr | (chr_data.chr_pal << 2) | pal_idx;

//         let raw_color = if pal_idx == 0 {
//             self.transparent_color()
//         } else {
//             self.registers.cgram[cgram_addr as usize].get()
//         };

//         ColorData {
//             raw_color,
//             priority: chr_data.chr_priority,
//             transparent: pal_idx == 0,
//         }
//     }

//     fn bg_col_4bpp(&self, tile_data: TileData, bg_chr_base_addr: u16, bg_cgram_base_addr: u8) -> ColorData {
//         let chr_data = self.fetch_chr_data(tile_data);

//         let tile_chr_addr = bg_chr_base_addr + (chr_data.chr_idx << 4) + chr_data.chr_row as u16;

//         let bp01 = self.vram_read(tile_chr_addr + 0);
//         let bp23 = self.vram_read(tile_chr_addr + 8);

//         let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
//         let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;
//         let b2 = ((bp23 >> (7-chr_data.chr_col)) & 1) as u8;
//         let b3 = ((bp23 >> (15-chr_data.chr_col)) & 1) as u8;

//         let pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;

//         let cgram_addr = bg_cgram_base_addr | (chr_data.chr_pal << 4) | pal_idx;

//         let raw_color = if pal_idx == 0 {
//             self.transparent_color()
//         } else {
//             self.registers.cgram[cgram_addr as usize].get()
//         };

//         ColorData {
//             raw_color,
//             priority: chr_data.chr_priority,
//             transparent: pal_idx == 0,
//         }
//     }

//     fn bg_col_8bpp(&self, tile_data: TileData, bg_chr_base_addr: u16) -> ColorData {
//         let chr_data = self.fetch_chr_data(tile_data);

//         let tile_chr_addr = bg_chr_base_addr + (chr_data.chr_idx << 5) + chr_data.chr_row as u16;

//         if !self.use_direct_col() {
//             let bp01 = self.vram_read(tile_chr_addr + 0);
//             let bp23 = self.vram_read(tile_chr_addr + 8);
//             let bp45 = self.vram_read(tile_chr_addr + 16);
//             let bp67 = self.vram_read(tile_chr_addr + 24);

//             let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
//             let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;
//             let b2 = ((bp23 >> (7-chr_data.chr_col)) & 1) as u8;
//             let b3 = ((bp23 >> (15-chr_data.chr_col)) & 1) as u8;
//             let b4 = ((bp45 >> (7-chr_data.chr_col)) & 1) as u8;
//             let b5 = ((bp45 >> (15-chr_data.chr_col)) & 1) as u8;
//             let b6 = ((bp67 >> (7-chr_data.chr_col)) & 1) as u8;
//             let b7 = ((bp67 >> (15-chr_data.chr_col)) & 1) as u8;

//             let cgram_addr = (b7 << 7) | (b6 << 6) | (b5 << 5) | (b4 << 4) | (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;

//             let raw_color = if cgram_addr == 0 {
//                 self.transparent_color()
//             } else {
//                 self.registers.cgram[cgram_addr as usize].get()
//             };

//             ColorData {
//                 raw_color,
//                 priority: chr_data.chr_priority,
//                 transparent: cgram_addr == 0,
//             }
//         } else {
//             let r_ext = ((chr_data.chr_pal & 0x4) >> 1) as u16;
//             let g_ext = ((chr_data.chr_pal & 0x8) >> 2) as u16;
//             let b_ext = ((chr_data.chr_pal & 0x10) >> 2) as u16;

//             let rgb_data = self.vram_read(tile_chr_addr + chr_data.chr_col as u16);

//             let r = ((rgb_data & 0x7) << 2) | r_ext;
//             let g = ((rgb_data & 0x38) >> 1) | g_ext;
//             let b = ((rgb_data & 0xC0) >> 3) | b_ext;

//             let raw_color = rgb565_from_parts(r, g, b);

//             ColorData {
//                 raw_color,
//                 priority: chr_data.chr_priority,
//                 transparent: raw_color == 0,
//             }
//         }
//     }

//     fn apply_cmath(&self, main_col: u16, sub_col: u16, screen_x: usize) -> u16 {
//         let col_win_en = self.col_win_active_signal(screen_x);

//         let main_col = match self.col_win_main_region() {
//             WindowColorRegion::Nowhere => main_col,
//             WindowColorRegion::Outside => if col_win_en { main_col } else { 0 },
//             WindowColorRegion::Inside => if col_win_en { 0 } else { main_col },
//             WindowColorRegion::Everywhere => { 0 }
//         };
//         let sub_col = match self.col_win_sub_region() {
//             WindowColorRegion::Nowhere => sub_col,
//             WindowColorRegion::Outside => if col_win_en { sub_col } else { self.transparent_color() },
//             WindowColorRegion::Inside => if col_win_en { self.transparent_color() } else { sub_col },
//             WindowColorRegion::Everywhere => { self.transparent_color() }
//         };

//         let sub_col = if self.sub_color_fixed() {
//             self.fixed_color()
//         } else {
//             sub_col
//         };

//         let (main_r, main_g, main_b) = rgb565_to_parts(main_col);
//         let (sub_r, sub_g, sub_b) = rgb565_to_parts(sub_col);

//         let (r,g,b) = match self.cmath_operator() {
//             CMathOperator::Add => (main_r + sub_r, main_g + sub_g, main_b + sub_b),
//             CMathOperator::Subtract => (main_r - sub_r, main_g - sub_g, main_b - sub_b),
//         };

//         let (r,g,b) = if self.cmath_half() {
//             (r >> 1, g >> 1, b >> 1)
//         } else {
//             (r, g, b)
//         };

//         // Negative values clamped to 0, positive values clamped to 31
//         let r = if r.bit_en(15) { 0 } else { r & 0x1F };
//         let g = if g.bit_en(15) { 0 } else { g & 0x1F };
//         let b = if b.bit_en(15) { 0 } else { b & 0x1F };

//         rgb565_from_parts(r, g, b)
//     }
// }

// // Getters & Setters for registers
// impl Ppu5C7x {
//     fn win_active_signal(&self, screen_x: usize, layer_w1_en: bool, layer_w2_en: bool,
//         layer_w1_inv: bool, layer_w2_inv: bool, win_logic: WindowLogic) -> bool {

//         let w1_left = self.w1_left_pos() as usize;
//         let w1_right = self.w1_right_pos() as usize;
//         let w2_left = self.w2_left_pos() as usize;
//         let w2_right = self.w2_right_pos() as usize;

//         let in_w1 = w1_left <= screen_x && screen_x <= w1_right;
//         let in_w2 = w2_left <= screen_x && screen_x <= w2_right;

//         let w1_en = (layer_w1_en && in_w1) ^ layer_w1_inv;
//         let w2_en = (layer_w2_en && in_w2) ^ layer_w2_inv;

//         let win_en = if layer_w1_en && layer_w2_en {
//             match win_logic {
//                 WindowLogic::Or => w1_en || w2_en,
//                 WindowLogic::And => w1_en && w2_en,
//                 WindowLogic::Xor => w1_en ^ w2_en,
//                 WindowLogic::Xnor => !(w1_en ^ w2_en),
//             }
//         } else if layer_w1_en {
//             w1_en
//         } else if layer_w2_en {
//             w2_en
//         } else {
//             false
//         };

//         win_en
//     }

//     fn bg1_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
//         let win_en = if self.bg1_win_main_enabled() || self.bg1_win_sub_enabled() {
//             self.win_active_signal(screen_x,
//                 self.bg1_w1_enabled(),
//                 self.bg1_w2_enabled(),
//                 self.bg1_w1_inverted(),
//                 self.bg1_w2_inverted(),
//                 self.bg1_win_logic()
//             )
//         } else {
//             false
//         };

//         let bg1_win_main_en = win_en && self.bg1_win_main_enabled();
//         let bg1_win_sub_en = win_en && self.bg1_win_sub_enabled();

//         (bg1_win_main_en, bg1_win_sub_en)
//     }

//     fn bg2_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
//         let win_en = if self.bg2_win_main_enabled() || self.bg2_win_sub_enabled() {
//             self.win_active_signal(screen_x,
//                 self.bg2_w1_enabled(),
//                 self.bg2_w2_enabled(),
//                 self.bg2_w1_inverted(),
//                 self.bg2_w2_inverted(),
//                 self.bg2_win_logic()
//             )
//         } else {
//             false
//         };

//         let bg2_win_main_en = win_en && self.bg2_win_main_enabled();
//         let bg2_win_sub_en = win_en && self.bg2_win_sub_enabled();

//         (bg2_win_main_en, bg2_win_sub_en)
//     }

//     fn bg3_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
//         let win_en = if self.bg3_win_main_enabled() || self.bg3_win_sub_enabled() {
//             self.win_active_signal(screen_x,
//                 self.bg3_w1_enabled(),
//                 self.bg3_w2_enabled(),
//                 self.bg3_w1_inverted(),
//                 self.bg3_w2_inverted(),
//                 self.bg3_win_logic()
//             )
//         } else {
//             false
//         };

//         let bg3_win_main_en = win_en && self.bg3_win_main_enabled();
//         let bg3_win_sub_en = win_en && self.bg3_win_sub_enabled();

//         (bg3_win_main_en, bg3_win_sub_en)
//     }

//     fn bg4_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
//         let win_en = if self.bg4_win_main_enabled() || self.bg4_win_sub_enabled() {
//             self.win_active_signal(screen_x,
//                 self.bg4_w1_enabled(),
//                 self.bg4_w2_enabled(),
//                 self.bg4_w1_inverted(),
//                 self.bg4_w2_inverted(),
//                 self.bg4_win_logic()
//             )
//         } else {
//             false
//         };

//         let bg4_win_main_en = win_en && self.bg4_win_main_enabled();
//         let bg4_win_sub_en = win_en && self.bg4_win_sub_enabled();

//         (bg4_win_main_en, bg4_win_sub_en)
//     }

//     fn obj_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
//         let win_en = if self.obj_win_main_enabled() || self.obj_win_sub_enabled() {
//             self.win_active_signal(screen_x,
//                 self.obj_w1_enabled(),
//                 self.obj_w2_enabled(),
//                 self.obj_w1_inverted(),
//                 self.obj_w2_inverted(),
//                 self.obj_win_logic()
//             )
//         } else {
//             false
//         };

//         let obj_win_main_en = win_en && self.obj_win_main_enabled();
//         let obj_win_sub_en = win_en && self.obj_win_sub_enabled();

//         (obj_win_main_en, obj_win_sub_en)
//     }

//     fn col_win_active_signal(&self, screen_x: usize) -> bool {
//         let win_en = self.win_active_signal(screen_x,
//             self.col_w1_enabled(),
//             self.col_w2_enabled(),
//             self.col_w1_inverted(),
//             self.col_w2_inverted(),
//             self.col_win_logic()
//         );

//         win_en
//     }

//     fn bg1_vram_base_addr(&self) -> u16 { (self.registers.bg1_vram_addr.get() as u16) << 10 }
//     fn bg2_vram_base_addr(&self) -> u16 { (self.registers.bg2_vram_addr.get() as u16) << 10 }
//     fn bg3_vram_base_addr(&self) -> u16 { (self.registers.bg3_vram_addr.get() as u16) << 10 }
//     fn bg4_vram_base_addr(&self) -> u16 { (self.registers.bg4_vram_addr.get() as u16) << 10 }

//     fn bg1_chr_base_addr(&self) -> u16 { (self.registers.bg1_chr_base_addr.get() as u16) << 12 }
//     fn bg2_chr_base_addr(&self) -> u16 { (self.registers.bg2_chr_base_addr.get() as u16) << 12 }
//     fn bg3_chr_base_addr(&self) -> u16 { (self.registers.bg3_chr_base_addr.get() as u16) << 12 }
//     fn bg4_chr_base_addr(&self) -> u16 { (self.registers.bg4_chr_base_addr.get() as u16) << 12 }

//     fn in_fblank(&self) -> bool { self.registers.in_fblank.get() }
//     fn in_hblank(&self) -> bool { self.registers.in_hblank.get() }
//     fn in_vblank(&self) -> bool { self.registers.in_vblank.get() }

//     fn bg4_tile_size(&self) -> TileSize { self.registers.bg4_char_size.get() }
//     fn bg3_tile_size(&self) -> TileSize { self.registers.bg3_char_size.get() }
//     fn bg2_tile_size(&self) -> TileSize { self.registers.bg2_char_size.get() }
//     fn bg1_tile_size(&self) -> TileSize { self.registers.bg1_char_size.get() }
//     fn obj_sprite_size(&self) -> ObjectSizeSelect { self.registers.obj_sprite_size.get() }

//     fn bg_mode(&self) -> BgMode { self.registers.bg_mode.get() }
//     fn fixed_color(&self) -> u16 { self.registers.fixed_color.get() }
//     fn bg3_mode1_priority(&self) -> bool { self.registers.bg3_mode1_priority.get() }

//     fn name_base_addr(&self) -> u16 { (self.registers.name_base_addr.get() as u16) << 13 }
//     fn name_secondary_select(&self) -> u16 { (self.registers.name_secondary_select.get() as u16 + 1) << 12 }

//     fn bg1_m7_x_offset(&self) -> u16 { self.registers.bg1_m7_x_offset.get() }
//     fn bg1_m7_y_offset(&self) -> u16 { self.registers.bg1_m7_y_offset.get() }
//     fn bg2_x_offset(&self) -> u16 { self.registers.bg2_x_offset.get() }
//     fn bg2_y_offset(&self) -> u16 { self.registers.bg2_y_offset.get() }
//     fn bg3_x_offset(&self) -> u16 { self.registers.bg3_x_offset.get() }
//     fn bg3_y_offset(&self) -> u16 { self.registers.bg3_y_offset.get() }
//     fn bg4_x_offset(&self) -> u16 { self.registers.bg4_x_offset.get() }
//     fn bg4_y_offset(&self) -> u16 { self.registers.bg4_y_offset.get() }

//     fn mosaic_size(&self) -> u8 { self.registers.mosaic_size.get() }
//     fn bg4_mosaic_en(&self) -> bool { self.registers.bg4_mosaic.get() }
//     fn bg3_mosaic_en(&self) -> bool { self.registers.bg3_mosaic.get() }
//     fn bg2_mosaic_en(&self) -> bool { self.registers.bg2_mosaic.get() }
//     fn bg1_mosaic_en(&self) -> bool { self.registers.bg1_mosaic.get() }

//     // fn screen_brightness(&self) -> u8 { self.registers.screen_brightness.get() }
//     // fn oam_addr(&self) -> u16 { self.registers.oam_addr.get() }
//     // fn priority_rotation(&self) -> bool { self.registers.priority_rotation.get() }
//     // fn oam_data_latch(&self) -> u8 { self.registers.oam_data_latch.get() }
//     // fn bg_mode(&self) -> BgMode { self.registers.bg_mode.get() }
//     // fn m7_latch(&self) -> u8 { self.registers.m7_latch.get() }
//     // fn bg_offset_latch(&self) -> u8 { self.registers.bg_offset_latch.get() }
//     // fn bg_offset_x_latch(&self) -> u8 { self.registers.bg_offset_x_latch.get() }
//     // fn vram_addr_inc_mode(&self) -> VramIncMode { self.registers.vram_addr_inc_mode.get() }
//     // fn addr_remap_mode(&self) -> AddressRemapping { self.registers.addr_remap_mode.get() }
//     // fn addr_inc_size(&self) -> IncrSize { self.registers.addr_inc_size.get() }
//     // fn vram_addr(&self) -> u16 { self.registers.vram_addr.get() }
//     // fn vram_data(&self) -> u16 { self.registers.vram_data.get() }
//     // fn m7_tilemap_repeat(&self) -> bool { self.registers.m7_tilemap_repeat.get() }
//     // fn m7_fill_mode(&self) -> M7FillMode { self.registers.m7_fill_mode.get() }
//     // fn m7_flip_bg_y(&self) -> bool { self.registers.m7_flip_bg_y.get() }
//     // fn m7_flip_bg_x(&self) -> bool { self.registers.m7_flip_bg_x.get() }
//     // fn m7_matrix_a(&self) -> u16 { self.registers.m7_matrix_a.get() }
//     // fn m7_matrix_b(&self) -> u16 { self.registers.m7_matrix_b.get() }
//     // fn m7_matrix_c(&self) -> u16 { self.registers.m7_matrix_c.get() }
//     // fn m7_matrix_d(&self) -> u16 { self.registers.m7_matrix_d.get() }
//     // fn m7_center_x(&self) -> u16 { self.registers.m7_center_x.get() }
//     // fn m7_center_y(&self) -> u16 { self.registers.m7_center_y.get() }
//     // fn cgram_toggle(&self) -> ToggleState { self.registers.cgram_toggle.get() }
//     // fn cgram_addr(&self) -> u8 { self.registers.cgram_addr.get() }

//     fn in_true_hi_res_mode(&self) -> bool {
//         match self.bg_mode() {
//             BgMode::Mode5 | BgMode::Mode6 => true,
//             _ => false
//         }
//     }

//     fn bg2_w2_enabled(&self) -> bool { self.registers.bg2_w2_enabled.get() }
//     fn bg2_w2_inverted(&self) -> bool { self.registers.bg2_w2_inverted.get() }
//     fn bg2_w1_enabled(&self) -> bool { self.registers.bg2_w1_enabled.get() }
//     fn bg2_w1_inverted(&self) -> bool { self.registers.bg2_w1_inverted.get() }
//     fn bg1_w2_enabled(&self) -> bool { self.registers.bg1_w2_enabled.get() }
//     fn bg1_w2_inverted(&self) -> bool { self.registers.bg1_w2_inverted.get() }
//     fn bg1_w1_enabled(&self) -> bool { self.registers.bg1_w1_enabled.get() }
//     fn bg1_w1_inverted(&self) -> bool { self.registers.bg1_w1_inverted.get() }
//     fn bg4_w2_enabled(&self) -> bool { self.registers.bg4_w2_enabled.get() }
//     fn bg4_w2_inverted(&self) -> bool { self.registers.bg4_w2_inverted.get() }
//     fn bg4_w1_enabled(&self) -> bool { self.registers.bg4_w1_enabled.get() }
//     fn bg4_w1_inverted(&self) -> bool { self.registers.bg4_w1_inverted.get() }
//     fn bg3_w2_enabled(&self) -> bool { self.registers.bg3_w2_enabled.get() }
//     fn bg3_w2_inverted(&self) -> bool { self.registers.bg3_w2_inverted.get() }
//     fn bg3_w1_enabled(&self) -> bool { self.registers.bg3_w1_enabled.get() }
//     fn bg3_w1_inverted(&self) -> bool { self.registers.bg3_w1_inverted.get() }
//     fn col_w2_enabled(&self) -> bool { self.registers.col_w2_enabled.get() }
//     fn col_w2_inverted(&self) -> bool { self.registers.col_w2_inverted.get() }
//     fn col_w1_enabled(&self) -> bool { self.registers.col_w1_enabled.get() }
//     fn col_w1_inverted(&self) -> bool { self.registers.col_w1_inverted.get() }
//     fn obj_w2_enabled(&self) -> bool { self.registers.obj_w2_enabled.get() }
//     fn obj_w2_inverted(&self) -> bool { self.registers.obj_w2_inverted.get() }
//     fn obj_w1_enabled(&self) -> bool { self.registers.obj_w1_enabled.get() }
//     fn obj_w1_inverted(&self) -> bool { self.registers.obj_w1_inverted.get() }
//     fn w1_left_pos(&self) -> u8 { self.registers.w1_left_pos.get() }
//     fn w1_right_pos(&self) -> u8 { self.registers.w1_right_pos.get() }
//     fn w2_left_pos(&self) -> u8 { self.registers.w2_left_pos.get() }
//     fn w2_right_pos(&self) -> u8 { self.registers.w2_right_pos.get() }
//     fn bg4_win_logic(&self) -> WindowLogic { self.registers.bg4_win_logic.get() }
//     fn bg3_win_logic(&self) -> WindowLogic { self.registers.bg3_win_logic.get() }
//     fn bg2_win_logic(&self) -> WindowLogic { self.registers.bg2_win_logic.get() }
//     fn bg1_win_logic(&self) -> WindowLogic { self.registers.bg1_win_logic.get() }
//     fn obj_win_logic(&self) -> WindowLogic { self.registers.obj_win_logic.get() }
//     fn col_win_logic(&self) -> WindowLogic { self.registers.col_win_logic.get() }
//     fn obj_main_enabled(&self) -> bool { self.registers.obj_main_enabled.get() }
//     fn bg4_main_enabled(&self) -> bool { self.registers.bg4_main_enabled.get() }
//     fn bg3_main_enabled(&self) -> bool { self.registers.bg3_main_enabled.get() }
//     fn bg2_main_enabled(&self) -> bool { self.registers.bg2_main_enabled.get() }
//     fn bg1_main_enabled(&self) -> bool { self.registers.bg1_main_enabled.get() }
//     fn obj_sub_enabled(&self) -> bool { self.registers.obj_sub_enabled.get() }
//     fn bg4_sub_enabled(&self) -> bool { self.registers.bg4_sub_enabled.get() }
//     fn bg3_sub_enabled(&self) -> bool { self.registers.bg3_sub_enabled.get() }
//     fn bg2_sub_enabled(&self) -> bool { self.registers.bg2_sub_enabled.get() }
//     fn bg1_sub_enabled(&self) -> bool { self.registers.bg1_sub_enabled.get() }
//     fn obj_win_main_enabled(&self) -> bool { self.registers.obj_win_main_enabled.get() }
//     fn bg4_win_main_enabled(&self) -> bool { self.registers.bg4_win_main_enabled.get() }
//     fn bg3_win_main_enabled(&self) -> bool { self.registers.bg3_win_main_enabled.get() }
//     fn bg2_win_main_enabled(&self) -> bool { self.registers.bg2_win_main_enabled.get() }
//     fn bg1_win_main_enabled(&self) -> bool { self.registers.bg1_win_main_enabled.get() }
//     fn obj_win_sub_enabled(&self) -> bool { self.registers.obj_win_sub_enabled.get() }
//     fn bg4_win_sub_enabled(&self) -> bool { self.registers.bg4_win_sub_enabled.get() }
//     fn bg3_win_sub_enabled(&self) -> bool { self.registers.bg3_win_sub_enabled.get() }
//     fn bg2_win_sub_enabled(&self) -> bool { self.registers.bg2_win_sub_enabled.get() }
//     fn bg1_win_sub_enabled(&self) -> bool { self.registers.bg1_win_sub_enabled.get() }
//     fn col_win_main_region(&self) -> WindowColorRegion { self.registers.col_win_main_region.get() }
//     fn col_win_sub_region(&self) -> WindowColorRegion { self.registers.col_win_sub_region.get() }
//     fn sub_color_fixed(&self) -> bool { self.registers.sub_color_fixed.get() }
//     fn cmath_operator(&self) -> CMathOperator { self.registers.cmath_operator.get() }
//     fn cmath_half(&self) -> bool { self.registers.cmath_half.get() }
//     fn back_cmath_enabled(&self) -> bool { self.registers.back_cmath_enabled.get() }
//     fn obj_cmath_enabled(&self) -> bool { self.registers.obj_cmath_enabled.get() }
//     fn bg4_cmath_enabled(&self) -> bool { self.registers.bg4_cmath_enabled.get() }
//     fn bg3_cmath_enabled(&self) -> bool { self.registers.bg3_cmath_enabled.get() }
//     fn bg2_cmath_enabled(&self) -> bool { self.registers.bg2_cmath_enabled.get() }
//     fn bg1_cmath_enabled(&self) -> bool { self.registers.bg1_cmath_enabled.get() }
//     fn bg1_tilemap_count_y(&self) -> TilemapCount { self.registers.bg1_tilemap_count_y.get() }
//     fn bg1_tilemap_count_x(&self) -> TilemapCount { self.registers.bg1_tilemap_count_x.get() }
//     fn bg2_tilemap_count_y(&self) -> TilemapCount { self.registers.bg2_tilemap_count_y.get() }
//     fn bg2_tilemap_count_x(&self) -> TilemapCount { self.registers.bg2_tilemap_count_x.get() }
//     fn bg3_tilemap_count_y(&self) -> TilemapCount { self.registers.bg3_tilemap_count_y.get() }
//     fn bg3_tilemap_count_x(&self) -> TilemapCount { self.registers.bg3_tilemap_count_x.get() }
//     fn bg4_tilemap_count_y(&self) -> TilemapCount { self.registers.bg4_tilemap_count_y.get() }
//     fn bg4_tilemap_count_x(&self) -> TilemapCount { self.registers.bg4_tilemap_count_x.get() }

//     fn hi_res_enabled(&self) -> bool { self.registers.hi_res_enabled.get() }
//     fn obj_interlace_enabled(&self) -> bool { self.registers.obj_interlace_enabled.get() }
//     fn screen_interlace_enabled(&self) -> bool { self.registers.screen_interlace_enabled.get() }
//     fn use_direct_col(&self) -> bool { self.registers.use_direct_col.get() }

//     // fn ext_bg_enabled(&self) -> bool { self.registers.ext_bg_enabled.get() }
//     // fn overscan_enabled(&self) -> bool { self.registers.overscan_enabled.get() }

//     fn vram_read(&self, address: u16) -> u16 { self.registers.vram[(address & 0x7FFF) as usize].get() }
// }
