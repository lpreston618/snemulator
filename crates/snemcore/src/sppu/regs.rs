use crate::sppu::color::Color;
use crate::sppu::types::*;
use crate::{get_bit_n, utils};

/// Contains all of the shared data (registers, memory, etc.) between the S-CPU and S-PPU.
#[derive(Default)]
pub struct PpuRegs {
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
    pub name_base_addr: u16,
    pub name_secondary_base_addr: u16,

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
    pub bg3_mode1_priority: bool,
    pub bg_mode: BgMode,

    // $2106    SSSS 4321    Write Only
    //       - Mosaic size (S)
    //       - Mosaic BG enable (#)
    pub mosaic_size: u8,

    // Contains several PPU registers for all layers. See snesdev for full list 
    // of registers for BG layers, OBJ layer, and Color layer.

    pub bg_settings: [BgSettings; 4],
    pub obj_settings: LayerSettings,
    pub col_settings: LayerSettings,

    // Used for many registers affecting mode 7 behavior
    pub m7_latch: u8,

    // Used for all scroll offset registers
    pub bg_offset_latch: u8,
    pub bg_offset_x_latch: u8,

    // $210D    ...x xxXX LLLL LLLL    Write x2 Only
    //       - BG1 or Mode 7 horizontal scroll (X)
    //       - mode7_data_latch (L), writing sets new data latch to (...x xxXX)
    pub m7_scroll_x: u16,

    // $210E    ...y yyYY LLLL LLLL    Write x2 Only
    //       - BG1 or Mode 7 vertical scroll (Y)
    //       - mode7_data_latch (L), writing sets new data latch to (.... ..YY)
    pub m7_scroll_y: u16,

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
    pub fn power_on(&mut self) {
        self.write_2100(0x80 | utils::rand_byte() & 0x0F);
        self.write_2101(utils::rand_byte());
        self.write_2102(utils::rand_byte());
        self.write_2103(utils::rand_byte());
        self.oam_data_latch = utils::rand_byte();
        self.write_2105(0xF0 | utils::rand_byte());
        self.write_2106(utils::rand_byte());
        self.write_2107(utils::rand_byte());
        self.write_2108(utils::rand_byte());
        self.write_2109(utils::rand_byte());
        self.write_210A(utils::rand_byte());
        self.write_210B(utils::rand_byte());
        self.write_210C(utils::rand_byte());
        self.write_210D(utils::rand_byte());
        self.write_210D(utils::rand_byte());
        self.write_210E(utils::rand_byte());
        self.write_210E(utils::rand_byte());
        self.write_210F(utils::rand_byte());
        self.write_210F(utils::rand_byte());
        self.write_2110(utils::rand_byte());
        self.write_2110(utils::rand_byte());
        self.write_2111(utils::rand_byte());
        self.write_2111(utils::rand_byte());
        self.write_2112(utils::rand_byte());
        self.write_2112(utils::rand_byte());
        self.write_2113(utils::rand_byte());
        self.write_2113(utils::rand_byte());
        self.write_2114(utils::rand_byte());
        self.write_2114(utils::rand_byte());
        self.write_2115(0x0F | utils::rand_byte());
        self.vram_addr = utils::rand_word();
        self.vram_latch = utils::rand_word();
        self.write_211A(utils::rand_byte());
        self.write_211B(0xFF);
        self.write_211B(0xFF);
        self.write_211C(0xFF);
        self.write_211C(0xFF);
        self.write_211D(utils::rand_byte());
        self.write_211D(utils::rand_byte());
        self.write_211E(utils::rand_byte());
        self.write_211E(utils::rand_byte());
        self.write_211F(utils::rand_byte());
        self.write_211F(utils::rand_byte());
        self.write_2120(utils::rand_byte());
        self.write_2120(utils::rand_byte());
        self.write_2121(utils::rand_byte());
        self.cgram_addr = utils::rand_byte();
        self.cgram_latch = utils::rand_byte();
        self.cgram_toggle = utils::rand_bool();
        self.write_2123(utils::rand_byte());
        self.write_2124(utils::rand_byte());
        self.write_2125(utils::rand_byte());
        self.write_2126(utils::rand_byte());
        self.write_2127(utils::rand_byte());
        self.write_2128(utils::rand_byte());
        self.write_2129(utils::rand_byte());
        self.write_212A(utils::rand_byte());
        self.write_212B(utils::rand_byte());
        self.write_212C(utils::rand_byte());
        self.write_212D(utils::rand_byte());
        self.write_212E(utils::rand_byte());
        self.write_212F(utils::rand_byte());
        self.write_2130(utils::rand_byte());
        self.write_2131(utils::rand_byte());
        self.write_2132(utils::rand_byte());
        self.write_2133(0);

        self.multiply_result = 0x000001;
        self.h_counter_latch = 0x01FF;
        self.v_counter_latch = 0x01FF;
    }

    pub fn reset(&mut self) {
        let byte_2100 = self.screen_brightness;

        self.write_2100(0x80 | byte_2100);
        self.write_2133(0);
    }

    pub fn write_2100(&mut self, value: u8) {
        self.in_fblank = get_bit_n!(value, 7);
        self.screen_brightness = value & 0x0F;
    }

    pub fn write_2101(&mut self, value: u8) {
        let new_obj_size = match value >> 5 {
            0 => ObjectSizeSelect::Size8x8_16x16,
            1 => ObjectSizeSelect::Size8x8_32x32,
            2 => ObjectSizeSelect::Size8x8_64x64,
            3 => ObjectSizeSelect::Size16x16_32x32,
            4 => ObjectSizeSelect::Size16x16_64x64,
            5 => ObjectSizeSelect::Size32x32_64x64,
            6 => ObjectSizeSelect::Size16x32_32x64,
            7 => ObjectSizeSelect::Size16x32_32x32,
            _ => unreachable!(),
        };

        // let new_obj_size = ObjectSizeSelect::Size16x16_32x32;

        self.obj_sprite_size = new_obj_size;
        self.name_base_addr = ((value as u16) & 3) << 13;

        let offset = ((((value as u16) >> 3) & 3) + 1) << 12;

        self.name_secondary_base_addr = self.name_base_addr + offset;
    }

    pub fn write_2102(&mut self, value: u8) {
        self.priority_rotation_idx = value & 0xFE;
        self.internal_oam_addr = (value as u16) << 1;
    }

    pub fn write_2103(&mut self, value: u8) {
        self.oam_write_high_table = get_bit_n!(value, 0);
        self.priority_rotation = get_bit_n!(value, 7);
    }

    pub fn write_2105(&mut self, value: u8) {
        self.bg_settings[3].chr_size = if get_bit_n!(value, 7) {
            TileSize::Size16x16
        } else {
            TileSize::Size8x8
        };
        self.bg_settings[2].chr_size = if get_bit_n!(value, 6) {
            TileSize::Size16x16
        } else {
            TileSize::Size8x8
        };
        self.bg_settings[1].chr_size = if get_bit_n!(value, 5) {
            TileSize::Size16x16
        } else {
            TileSize::Size8x8
        };
        self.bg_settings[0].chr_size = if get_bit_n!(value, 4) {
            TileSize::Size16x16
        } else {
            TileSize::Size8x8
        };
        self.bg3_mode1_priority = get_bit_n!(value, 3);
        self.bg_mode = match value & 7 {
            0 => BgMode::Mode0,
            1 => BgMode::Mode1,
            2 => BgMode::Mode2,
            3 => BgMode::Mode3,
            4 => BgMode::Mode4,
            5 => BgMode::Mode5,
            6 => BgMode::Mode6,
            7 => BgMode::Mode7,
            _ => unreachable!(),
        };

        match self.bg_mode {
            BgMode::Mode5 | BgMode::Mode6 => {
                self.hi_res_en = true;
            }
            _ => {}
        };
    }

    pub fn write_2106(&mut self, value: u8) {
        self.mosaic_size = value >> 4;
        self.bg_settings[3].mosaic_en = get_bit_n!(value, 3);
        self.bg_settings[2].mosaic_en = get_bit_n!(value, 2);
        self.bg_settings[1].mosaic_en = get_bit_n!(value, 1);
        self.bg_settings[0].mosaic_en = get_bit_n!(value, 0);
    }

    pub fn write_2107(&mut self, value: u8) {
        self.bg_settings[0].tilemap_base_addr = ((value as u16) & 0x7C) << 8;
        self.bg_settings[0].tilemap_cnt_y = if get_bit_n!(value, 1) {
            TilemapCount::Two
        } else {
            TilemapCount::One
        };
        self.bg_settings[0].tilemap_cnt_x = if get_bit_n!(value, 0) {
            TilemapCount::Two
        } else {
            TilemapCount::One
        };
    }

    pub fn write_2108(&mut self, value: u8) {
        self.bg_settings[1].tilemap_base_addr = ((value as u16) & 0x7C) << 8;
        self.bg_settings[1].tilemap_cnt_y = if get_bit_n!(value, 1) {
            TilemapCount::Two
        } else {
            TilemapCount::One
        };
        self.bg_settings[1].tilemap_cnt_x = if get_bit_n!(value, 0) {
            TilemapCount::Two
        } else {
            TilemapCount::One
        };
    }

    pub fn write_2109(&mut self, value: u8) {
        self.bg_settings[2].tilemap_base_addr = ((value as u16) & 0x7C) << 8;
        self.bg_settings[2].tilemap_cnt_y = if get_bit_n!(value, 1) {
            TilemapCount::Two
        } else {
            TilemapCount::One
        };
        self.bg_settings[2].tilemap_cnt_x = if get_bit_n!(value, 0) {
            TilemapCount::Two
        } else {
            TilemapCount::One
        };
    }

    #[allow(non_snake_case)]
    pub fn write_210A(&mut self, value: u8) {
        self.bg_settings[3].tilemap_base_addr = ((value as u16) & 0x7C) << 8;
        self.bg_settings[3].tilemap_cnt_y = if get_bit_n!(value, 1) {
            TilemapCount::Two
        } else {
            TilemapCount::One
        };
        self.bg_settings[3].tilemap_cnt_x = if get_bit_n!(value, 0) {
            TilemapCount::Two
        } else {
            TilemapCount::One
        };
    }

    #[allow(non_snake_case)]
    pub fn write_210B(&mut self, value: u8) {
        self.bg_settings[0].chr_base_addr = ((value as u16) & 0x07) << 12;
        self.bg_settings[1].chr_base_addr = ((value as u16) & 0x70) << 8;
    }

    #[allow(non_snake_case)]
    pub fn write_210C(&mut self, value: u8) {
        self.bg_settings[2].chr_base_addr = ((value as u16) & 0x07) << 12;
        self.bg_settings[3].chr_base_addr = ((value as u16) & 0x70) << 8;
    }

    #[allow(non_snake_case)]
    pub fn write_210D(&mut self, value: u8) {
        let bgofs_latch = self.bg_offset_latch as u16;
        let bghofs_latch = self.bg_offset_x_latch as u16;
        let m7_latch = self.m7_latch as u16;
        self.bg_offset_latch = value;
        self.bg_offset_x_latch = value;
        self.m7_latch = value;

        self.bg_settings[0].scroll_x = (((value & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
        self.m7_scroll_x = ((value & 0x1F) as u16) << 8 | m7_latch;
    }

    #[allow(non_snake_case)]
    pub fn write_210E(&mut self, value: u8) {
        let bgofs_latch = self.bg_offset_latch as u16;
        let m7_latch = self.m7_latch as u16;
        self.bg_offset_latch = value;
        self.m7_latch = value;

        self.bg_settings[0].scroll_y = (((value & 3) as u16) << 8) | bgofs_latch;
        self.m7_scroll_y = ((value & 0x1F) as u16) << 8 | m7_latch;
    }

    #[allow(non_snake_case)]
    pub fn write_210F(&mut self, value: u8) {
        let bgofs_latch = self.bg_offset_latch as u16;
        let bghofs_latch = self.bg_offset_x_latch as u16;
        self.bg_offset_latch = value;
        self.bg_offset_x_latch = value;

        self.bg_settings[1].scroll_x =
            (((value & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
    }

    pub fn write_2110(&mut self, value: u8) {
        let bgofs_latch = self.bg_offset_latch as u16;
        self.bg_offset_latch = value;

        self.bg_settings[1].scroll_y = (((value & 3) as u16) << 8) | bgofs_latch;
    }

    pub fn write_2111(&mut self, value: u8) {
        let bgofs_latch = self.bg_offset_latch as u16;
        let bghofs_latch = self.bg_offset_x_latch as u16;
        self.bg_offset_latch = value;
        self.bg_offset_x_latch = value;

        self.bg_settings[2].scroll_x =
            (((value & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
    }

    pub fn write_2112(&mut self, value: u8) {
        let bgofs_latch = self.bg_offset_latch as u16;
        self.bg_offset_latch = value;

        self.bg_settings[2].scroll_y = (((value & 3) as u16) << 8) | bgofs_latch;
    }

    pub fn write_2113(&mut self, value: u8) {
        let bgofs_latch = self.bg_offset_latch as u16;
        let bghofs_latch = self.bg_offset_x_latch as u16;
        self.bg_offset_latch = value;
        self.bg_offset_x_latch = value;

        self.bg_settings[3].scroll_x =
            (((value & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07);
    }

    pub fn write_2114(&mut self, value: u8) {
        let bgofs_latch = self.bg_offset_latch as u16;
        self.bg_offset_latch = value;

        self.bg_settings[3].scroll_y = (((value & 3) as u16) << 8) | bgofs_latch;
    }

    pub fn write_2115(&mut self, value: u8) {
        self.vram_addr_inc_mode = if get_bit_n!(value, 7) {
            VramIncMode::HighByte
        } else {
            VramIncMode::LowByte
        };
        self.addr_remap_mode = match (value >> 2) & 3 {
            0 => AddressRemapping::None,
            1 => AddressRemapping::ColDepth2,
            2 => AddressRemapping::ColDepth4,
            3 => AddressRemapping::ColDepth8,
            _ => unreachable!(),
        };
        self.addr_inc_size = match value & 3 {
            0 => IncrSize::Bytes2,
            1 => IncrSize::Bytes64,
            2 => IncrSize::Bytes256,
            3 => IncrSize::Bytes256,
            _ => unreachable!(),
        };
    }

    #[allow(non_snake_case)]
    pub fn write_211A(&mut self, value: u8) {
        self.m7_tilemap_repeat = get_bit_n!(value, 7);
        self.m7_fill_mode = if get_bit_n!(value, 6) {
            M7FillMode::Character
        } else {
            M7FillMode::Transparent
        };
        self.m7_flip_bg_y = get_bit_n!(value, 1);
        self.m7_flip_bg_x = get_bit_n!(value, 0);
    }

    #[allow(non_snake_case)]
    pub fn write_211B(&mut self, value: u8) {
        let latched_val = self.m7_latch as u16;
        self.m7_latch = value;

        self.m7_matrix_a = ((value as u16) << 8) | latched_val;
        self.mult_factor_16 = ((value as u16) << 8) | latched_val;

        self.update_multiply_result();
    }

    #[allow(non_snake_case)]
    pub fn write_211C(&mut self, value: u8) {
        let latched_val = self.m7_latch as u16;
        self.m7_latch = value;

        self.m7_matrix_b = ((value as u16) << 8) | latched_val;
        self.mult_factor_8 = latched_val as u8;

        self.update_multiply_result();
    }

    #[allow(non_snake_case)]
    pub fn write_211D(&mut self, value: u8) {
        let latched_val = self.m7_latch as u16;
        self.m7_latch = value;

        self.m7_matrix_c = ((value as u16) << 8) | latched_val;
    }

    #[allow(non_snake_case)]
    pub fn write_211E(&mut self, value: u8) {
        let latched_val = self.m7_latch as u16;
        self.m7_latch = value;

        self.m7_matrix_d = ((value as u16) << 8) | latched_val;
    }

    #[allow(non_snake_case)]
    pub fn write_211F(&mut self, value: u8) {
        let latched_val = self.m7_latch as u16;
        self.m7_latch = value;

        self.m7_center_x = ((value as u16) << 8) | latched_val;
    }

    pub fn write_2120(&mut self, value: u8) {
        let latched_val = self.m7_latch as u16;
        self.m7_latch = value;

        self.m7_center_y = ((value as u16) << 8) | latched_val;
    }

    pub fn write_2121(&mut self, value: u8) {
        self.cgram_addr = value;
        self.cgram_toggle = false;
    }

    pub fn write_2123(&mut self, value: u8) {
        self.bg_settings[1].window.w2_en = get_bit_n!(value, 7);
        self.bg_settings[1].window.w2_inv = get_bit_n!(value, 6);
        self.bg_settings[1].window.w1_en = get_bit_n!(value, 5);
        self.bg_settings[1].window.w1_inv = get_bit_n!(value, 4);
        self.bg_settings[0].window.w2_en = get_bit_n!(value, 3);
        self.bg_settings[0].window.w2_inv = get_bit_n!(value, 2);
        self.bg_settings[0].window.w1_en = get_bit_n!(value, 1);
        self.bg_settings[0].window.w1_inv = get_bit_n!(value, 0);
    }

    pub fn write_2124(&mut self, value: u8) {
        self.bg_settings[3].window.w2_en = get_bit_n!(value, 7);
        self.bg_settings[3].window.w2_inv = get_bit_n!(value, 6);
        self.bg_settings[3].window.w1_en = get_bit_n!(value, 5);
        self.bg_settings[3].window.w1_inv = get_bit_n!(value, 4);
        self.bg_settings[2].window.w2_en = get_bit_n!(value, 3);
        self.bg_settings[2].window.w2_inv = get_bit_n!(value, 2);
        self.bg_settings[2].window.w1_en = get_bit_n!(value, 1);
        self.bg_settings[2].window.w1_inv = get_bit_n!(value, 0);
    }

    pub fn write_2125(&mut self, value: u8) {
        self.col_settings.window.w2_en = get_bit_n!(value, 7);
        self.col_settings.window.w2_inv = get_bit_n!(value, 6);
        self.col_settings.window.w1_en = get_bit_n!(value, 5);
        self.col_settings.window.w1_inv = get_bit_n!(value, 4);
        self.obj_settings.window.w2_en = get_bit_n!(value, 3);
        self.obj_settings.window.w2_inv = get_bit_n!(value, 2);
        self.obj_settings.window.w1_en = get_bit_n!(value, 1);
        self.obj_settings.window.w1_inv = get_bit_n!(value, 0);
    }

    pub fn write_2126(&mut self, value: u8) {
        // log::debug!("W1 left pos set to {value}");

        self.w1_left_pos = value;
    }

    pub fn write_2127(&mut self, value: u8) {
        // log::debug!("W1 right pos set to {value}");

        self.w1_right_pos = value;
    }

    pub fn write_2128(&mut self, value: u8) {
        // log::debug!("W2 left pos set to {value}");

        self.w2_left_pos = value;
    }

    pub fn write_2129(&mut self, value: u8) {
        // log::debug!("W2 right pos set to {value}");

        self.w2_right_pos = value;
    }

    #[allow(non_snake_case)]
    pub fn write_212A(&mut self, value: u8) {
        self.bg_settings[3].window.logic = match value >> 6 {
            0 => WindowLogic::Or,
            1 => WindowLogic::And,
            2 => WindowLogic::Xor,
            3 => WindowLogic::Xnor,
            _ => unreachable!(),
        };
        self.bg_settings[2].window.logic = match (value >> 4) & 3 {
            0 => WindowLogic::Or,
            1 => WindowLogic::And,
            2 => WindowLogic::Xor,
            3 => WindowLogic::Xnor,
            _ => unreachable!(),
        };
        self.bg_settings[1].window.logic = match (value >> 2) & 3 {
            0 => WindowLogic::Or,
            1 => WindowLogic::And,
            2 => WindowLogic::Xor,
            3 => WindowLogic::Xnor,
            _ => unreachable!(),
        };
        self.bg_settings[0].window.logic = match value & 3 {
            0 => WindowLogic::Or,
            1 => WindowLogic::And,
            2 => WindowLogic::Xor,
            3 => WindowLogic::Xnor,
            _ => unreachable!(),
        };
    }

    #[allow(non_snake_case)]
    pub fn write_212B(&mut self, value: u8) {
        self.col_settings.window.logic = match (value >> 2) & 3 {
            0 => WindowLogic::Or,
            1 => WindowLogic::And,
            2 => WindowLogic::Xor,
            3 => WindowLogic::Xnor,
            _ => unreachable!(),
        };
        self.obj_settings.window.logic = match value & 3 {
            0 => WindowLogic::Or,
            1 => WindowLogic::And,
            2 => WindowLogic::Xor,
            3 => WindowLogic::Xnor,
            _ => unreachable!(),
        };
    }

    #[allow(non_snake_case)]
    pub fn write_212C(&mut self, value: u8) {
        self.obj_settings.main_en = get_bit_n!(value, 4);
        self.bg_settings[3].main_en = get_bit_n!(value, 3);
        self.bg_settings[2].main_en = get_bit_n!(value, 2);
        self.bg_settings[1].main_en = get_bit_n!(value, 1);
        self.bg_settings[0].main_en = get_bit_n!(value, 0);
    }

    #[allow(non_snake_case)]
    pub fn write_212D(&mut self, value: u8) {
        self.obj_settings.sub_en = get_bit_n!(value, 4);
        self.bg_settings[3].sub_en = get_bit_n!(value, 3);
        self.bg_settings[2].sub_en = get_bit_n!(value, 2);
        self.bg_settings[1].sub_en = get_bit_n!(value, 1);
        self.bg_settings[0].sub_en = get_bit_n!(value, 0);
    }

    #[allow(non_snake_case)]
    pub fn write_212E(&mut self, value: u8) {
        self.obj_settings.window.main_en = get_bit_n!(value, 4);
        self.bg_settings[3].window.main_en = get_bit_n!(value, 3);
        self.bg_settings[2].window.main_en = get_bit_n!(value, 2);
        self.bg_settings[1].window.main_en = get_bit_n!(value, 1);
        self.bg_settings[0].window.main_en = get_bit_n!(value, 0);
    }

    #[allow(non_snake_case)]
    pub fn write_212F(&mut self, value: u8) {
        self.obj_settings.window.sub_en = get_bit_n!(value, 4);
        self.bg_settings[3].window.sub_en = get_bit_n!(value, 3);
        self.bg_settings[2].window.sub_en = get_bit_n!(value, 2);
        self.bg_settings[1].window.sub_en = get_bit_n!(value, 1);
        self.bg_settings[0].window.sub_en = get_bit_n!(value, 0);
    }

    pub fn write_2130(&mut self, value: u8) {
        self.col_win_main_region = match value >> 6 {
            0 => WindowColorRegion::Nowhere,
            1 => WindowColorRegion::Outside,
            2 => WindowColorRegion::Inside,
            3 => WindowColorRegion::Everywhere,
            _ => unreachable!(),
        };
        self.col_win_sub_region = match (value >> 4) & 3 {
            0 => WindowColorRegion::Nowhere,
            1 => WindowColorRegion::Outside,
            2 => WindowColorRegion::Inside,
            3 => WindowColorRegion::Everywhere,
            _ => unreachable!(),
        };
        self.sub_color_fixed = !get_bit_n!(value, 1);
        self.use_direct_col = get_bit_n!(value, 0);
    }

    pub fn write_2131(&mut self, value: u8) {
        self.cmath_operator = if get_bit_n!(value, 7) {
            CMathOperator::Subtract
        } else {
            CMathOperator::Add
        };
        self.cmath_half = get_bit_n!(value, 6);
        self.back_cmath_en = get_bit_n!(value, 5);
        self.obj_settings.cmath_en = get_bit_n!(value, 4);
        self.bg_settings[3].cmath_en = get_bit_n!(value, 3);
        self.bg_settings[2].cmath_en = get_bit_n!(value, 2);
        self.bg_settings[1].cmath_en = get_bit_n!(value, 1);
        self.bg_settings[0].cmath_en = get_bit_n!(value, 0);
    }

    pub fn write_2132(&mut self, value: u8) {
        let new_val = value & 0x1F;

        if get_bit_n!(value, 7) {
            self.fixed_color.b = new_val;
        }
        if get_bit_n!(value, 6) {
            self.fixed_color.g = new_val;
        }
        if get_bit_n!(value, 5) {
            self.fixed_color.r = new_val;
        }
    }

    pub fn write_2133(&mut self, value: u8) {
        self._external_sync = get_bit_n!(value, 7);
        self.ext_bg_en = get_bit_n!(value, 6);
        self.hi_res_en = get_bit_n!(value, 3);
        self.overscan_en = get_bit_n!(value, 2);
        self.obj_interlace_en = get_bit_n!(value, 1);
        self.screen_interlace_en = get_bit_n!(value, 0);
    }

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
