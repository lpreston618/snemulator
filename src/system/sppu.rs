mod utils;

use utils::{xbgr0555_to_rgb565, rgb565_to_xbgr0555, Togglable, ToggleState};

use crate::system::sppu::utils::{rgb565_from_parts, rgb565_to_parts};
use crate::utils::{GetBits, SetCellBytes};
use crate::log::{SnemLogger, LogLevel};

use libretro_rs::retro::pixel::format::RGB565;

use std::{cell::Cell, rc::Rc};

const VBLANK_START_SCANLINE: usize = 225;
const VBLANK_END_SCANLINE_NTSC: usize = 261;
// const VBLANK_END_SCANLINE_PAL: u16 = 311;
// const VBLANK_INTERLACE_START_SCANLINE: u16 = 239;
const VISIBLE_SCANLINE_START_DOT: usize = 22;
const HBLANK_END_DOT: usize = VISIBLE_SCANLINE_START_DOT;
const HBLANK_START_DOT: usize = 278;
const HBLANK_DISABLE_SCANLINE: usize = VBLANK_START_SCANLINE-1;
const SCANLINE_END_DOT: usize = 340;

/// 64 KiB (32 Ki-Word) or Video RAM
const VRAM_SIZE: usize = 32 * 1024;
/// 512 Bytes (256 Words) of Character-Graphics RAM
const CGRAM_SIZE: usize = 256;
/// 544 Bytes of Object Attribute Memory
const OAM_SIZE: usize = 544;

#[derive(Clone, Copy, Debug)]
pub enum HVTimerIRQ {
    None,   // Ignore H/V Timers
    HTimer, // IRQ when H counter == HTIME
    VTimer, // IRQ when V counter == VTIME and H counter == 0
    Both,   // IRQ when V counter == VTIME and H counter == HTIME
}

#[derive(Clone, Copy, Debug)]
enum ObjectSizeSelect {
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
enum ObjectSize {
    Size8x8,
    Size16x16,
    Size32x32,
    Size64x64,
    Size16x32,
    Size32x64,
}

#[derive(Clone, Copy, Debug)]
enum TileSize {
    Size8x8,
    Size16x16,
}

#[derive(Clone, Copy, Debug)]
enum BgMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Mode6,
    Mode7
}

#[derive(Clone, Copy, Debug)]
enum TilemapCount {
    One,
    Two,
}

#[derive(Clone, Copy, Debug)]
enum VramIncMode {
    LowByte,
    HighByte
}

#[derive(Clone, Copy, Debug)]
enum AddressRemapping {
    None,
    ColDepth2,
    ColDepth4,
    ColDepth8,
}

#[derive(Clone, Copy, Debug)]
enum ColorDepth {
    Direct,
    Bpp2,
    Bpp4,
    Bpp8,
}

#[derive(Clone, Copy, Debug)]
enum IncrSize {
    Bytes2,
    Bytes64,
    Bytes256,
}

#[derive(Clone, Copy, Debug)]
enum M7FillMode {
    Transparent,
    Character,
}

#[derive(Clone, Copy, Debug)]
enum WindowLogic {
    Or,
    And,
    Xor,
    Xnor,
}

#[derive(Clone, Copy, Debug)]
enum WindowColorRegion {
    Nowhere,
    Outside,
    Inside,
    Everywhere,
}

#[derive(Clone, Copy, Debug)]
enum CMathOperator {
    Add,
    Subtract,
}

#[derive(Clone, Copy, Default, Debug)]
enum MasterSlave {
    #[default]
    Master,
    Slave,
}

#[derive(Clone, Copy, Default, Debug)]
enum VideoType {
    #[default]
    Ntsc,
    Pal,
}

/// Contains all of the shared data (registers, memory, etc.) between the S-CPU
/// and S-PPU.
pub struct PpuData {
    scanline: Cell<usize>,
    dot: Cell<usize>,

    // $2100    F... BBBB    Write only
    //       - Forced blanking (F)
    //       - Screen brightness (B)
    in_fblank: Cell<bool>,
    screen_brightness: Cell<u8>,

    // $2101    SSSN NbBB    Write only
    //       - OBJ sprite size (S)
    //       - Name secondary select (N)
    //       - Name base address (B)
    obj_sprite_size: Cell<ObjectSizeSelect>,
    name_secondary_select: Cell<u8>,
    name_base_addr: Cell<u8>,

    // $2102    AAAA AAAA
    // $2103    P... ...B    Write x2 Only
    //       - OAM word address (A)
    //       - Priority rotation (P)
    //       - Address high bit / table select (B)
    oam_addr: Cell<u16>,
    internal_oam_addr: Cell<u16>,
    priority_rotation: Cell<bool>,
    priority_rotation_idx: Cell<u8>,

    // $2104    DDDD DDDD    Write x2 Only
    //       - OAM data write byte (2x for word) (D), increments OAMADD byte
    oam_data_latch: Cell<u8>,

    // $2105    4321 PMMM    Write Only
    //       - Tilemap tile size (#)
    //       - BG3 priority (P)
    //       - BG mode (M)
    bg4_char_size: Cell<TileSize>,
    bg3_char_size: Cell<TileSize>,
    bg2_char_size: Cell<TileSize>,
    bg1_char_size: Cell<TileSize>,
    bg3_mode1_priority: Cell<bool>,
    bg_mode: Cell<BgMode>,

    // $2106    SSSS 4321    Write Only
    //       - Mosaic size (S)
    //       - Mosaic BG enable (#)
    mosaic_size: Cell<u8>,
    bg4_mosaic: Cell<bool>,
    bg3_mosaic: Cell<bool>,
    bg2_mosaic: Cell<bool>,
    bg1_mosaic: Cell<bool>,

    // $2107    AAAA AAYX    Write Only
    //       - BG1 Tilemap VRAM address (A)
    //       - BG1 Vertical tilemap count (Y)
    //       - BG1 Horizontal tilemap count (X)
    bg1_vram_addr: Cell<u8>,
    bg1_tilemap_count_y: Cell<TilemapCount>,
    bg1_tilemap_count_x: Cell<TilemapCount>,

    // $2108    AAAA AAYX    Write Only
    //       - BG2 Tilemap VRAM address (A)
    //       - BG2 Vertical tilemap count (Y)
    //       - BG2 Horizontal tilemap count (X)
    bg2_vram_addr: Cell<u8>,
    bg2_tilemap_count_y: Cell<TilemapCount>,
    bg2_tilemap_count_x: Cell<TilemapCount>,

    // $2109    AAAA AAYX    Write Only
    //       - BG3 Tilemap VRAM address (A)
    //       - BG3 Vertical tilemap count (Y)
    //       - BG3 Horizontal tilemap count (X)
    bg3_vram_addr: Cell<u8>,
    bg3_tilemap_count_y: Cell<TilemapCount>,
    bg3_tilemap_count_x: Cell<TilemapCount>,

    // $210A    AAAA AAYX    Write Only
    //       - BG4 Tilemap VRAM address (A)
    //       - BG4 Vertical tilemap count (Y)
    //       - BG4 Horizontal tilemap count (X)
    bg4_vram_addr: Cell<u8>,
    bg4_tilemap_count_y: Cell<TilemapCount>,
    bg4_tilemap_count_x: Cell<TilemapCount>,

    // $210B    BBBB AAAA    W8
    //       - BG2 CHR base address (B)
    //       - BG1 CHR base address (A)
    bg2_chr_base_addr: Cell<u8>,
    bg1_chr_base_addr: Cell<u8>,

    // $210C    DDDD CCCC    W8
    //       - BG4 CHR base address (D)
    //       - BG3 CHR base address (C)
    bg4_chr_base_addr: Cell<u8>,
    bg3_chr_base_addr: Cell<u8>,

    // Used for many registers affecting mode 7 behavior
    m7_latch: Cell<u8>,

    // Used for all scroll offset registers
    bg_offset_latch: Cell<u8>,
    bg_offset_x_latch: Cell<u8>,

    // $210D    ...x xxXX LLLL LLLL    Write x2 Only
    //       - BG1 or Mode 7 horizontal scroll (X)
    //       - mode7_data_latch (L), writing sets new data latch to (...x xxXX)
    bg1_m7_x_offset: Cell<u16>,

    // $210E    ...y yyYY LLLL LLLL    Write x2 Only
    //       - BG1 or Mode 7 vertical scroll (Y)
    //       - mode7_data_latch (L), writing sets new data latch to (.... ..YY)
    bg1_m7_y_offset: Cell<u16>,

    // $210F    .... ..XX XXXX XXXX    Write x2 Only
    //       - BG2 horizontal scroll (X)
    bg2_x_offset: Cell<u16>,

    // $2110    .... ..YY YYYY YYYY    Write x2 Only
    //       - BG2 vertical scroll (Y)
    bg2_y_offset: Cell<u16>,

    // $2111    .... ..XX XXXX XXXX    Write x2 Only
    //       - BG3 horizontal scroll (X)
    bg3_x_offset: Cell<u16>,

    // $2112    .... ..YY YYYY YYYY    Write x2 Only
    //       - BG3 vertical scroll (Y)
    bg3_y_offset: Cell<u16>,

    // $2113    .... ..XX XXXX XXXX    Write x2 Only
    //       - BG4 horizontal scroll (X)
    bg4_x_offset: Cell<u16>,

    // $2114    .... ..YY YYYY YYYY    Write x2 Only
    //       - BG4 vertical scroll (Y)
    bg4_y_offset: Cell<u16>,

    // $2115    M... RRII    W8
    //       - VRAM address increment mode (M)
    //       - Remapping (R)
    //       - Increment size (I)
    vram_addr_inc_mode: Cell<VramIncMode>,
    addr_remap_mode: Cell<AddressRemapping>,
    addr_inc_size: Cell<IncrSize>,

    // $2116    LLLL LLLL
    // $2117    hHHH HHHH    Write x2 Only
    //       - VRAM word address Low (L)
    //       - VRAM word address High (H)
    pub(crate) vram_addr: Cell<u16>,

    // $211A    RF.. ..YX    W8
    //       - Mode 7 tilemap repeat (R)
    //       - Mode 7 non-repeat fill (F)
    //       - Mode 7 Flip vertical (Y)
    //       - Mode 7 Flip horizontal (X)
    m7_tilemap_repeat: Cell<bool>,
    m7_fill_mode: Cell<M7FillMode>,
    m7_flip_bg_y: Cell<bool>,
    m7_flip_bg_x: Cell<bool>,

    // $211B    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix A or signed 16-bit multiplication factor (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    m7_matrix_a: Cell<u16>,
    mult_factor_16: Cell<u16>,

    // $211C    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix B or signed 8-bit multiplication factor (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    m7_matrix_b: Cell<u16>,
    mult_factor_8: Cell<u8>,

    // $211D    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix C (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    m7_matrix_c: Cell<u16>,

    // $211E    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix D (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    m7_matrix_d: Cell<u16>,

    // $211F    ...X XXXX LLLL LLLL    Write Only
    //       - Mode 7 center X (X)
    //       - mode7_data_latch (L), writing sets new data latch to (...X XXXX)
    m7_center_x: Cell<u16>,

    // $2120    ...Y YYYY LLLL LLLL    Write Only
    //       - Mode 7 center Y (Y)
    //       - mode7_data_latch (L), writing sets new data latch to (...Y YYYY)
    m7_center_y: Cell<u16>,

    // Toggle used for $2121 and $2122 (CGRAM registers)
    cgram_toggle: Cell<ToggleState>,

    // $2121    AAAA AAAA    Write Only
    //       - CGRAM word address (A)
    cgram_addr: Cell<u8>,
    cgram_latch: Cell<u8>,

    // $2123    DdCc BbAa    Write Only
    //       - Enable (ABCD) and Invert (abcd) windows for BG1 (AB) and BG2 (CD)
    bg2_w2_enabled: Cell<bool>,
    bg2_w2_inverted: Cell<bool>,
    bg2_w1_enabled: Cell<bool>,
    bg2_w1_inverted: Cell<bool>,
    bg1_w2_enabled: Cell<bool>,
    bg1_w2_inverted: Cell<bool>,
    bg1_w1_enabled: Cell<bool>,
    bg1_w1_inverted: Cell<bool>,

    // $2124    DdCc BbAa    Write Only
    //       - Enable (EFGH) and Invert (efgh) windows for BG3 (EF) and BG2 (GH)
    bg4_w2_enabled: Cell<bool>,
    bg4_w2_inverted: Cell<bool>,
    bg4_w1_enabled: Cell<bool>,
    bg4_w1_inverted: Cell<bool>,
    bg3_w2_enabled: Cell<bool>,
    bg3_w2_inverted: Cell<bool>,
    bg3_w1_enabled: Cell<bool>,
    bg3_w1_inverted: Cell<bool>,

    // $2125    LlKk JjIi    Write Only
    //       - Enable (IJKL) and Invert (ijkl) windows for OBJ (IJ) and color (KL)
    col_w2_enabled: Cell<bool>,
    col_w2_inverted: Cell<bool>,
    col_w1_enabled: Cell<bool>,
    col_w1_inverted: Cell<bool>,
    obj_w2_enabled: Cell<bool>,
    obj_w2_inverted: Cell<bool>,
    obj_w1_enabled: Cell<bool>,
    obj_w1_inverted: Cell<bool>,

    // $2126    LLLL LLLL    Write Only
    //       - Window 1 left position (L)
    w1_left_pos: Cell<u8>,

    // $2127    RRRR RRRR    Write Only
    //       - Window 1 right position (R)
    w1_right_pos: Cell<u8>,

    // $2128    LLLL LLLL    Write Only
    //       - Window 2 left position (L)
    w2_left_pos: Cell<u8>,

    // $2129    RRRR RRRR    Write Only
    //       - Window 2 right position (R)
    w2_right_pos: Cell<u8>,

    // $212A    4433 2211    Write Only
    //       - Window mask logic for BG layers (00=OR, 01=AND, 10=XOR, 11=XNOR)
    bg4_win_logic: Cell<WindowLogic>,
    bg3_win_logic: Cell<WindowLogic>,
    bg2_win_logic: Cell<WindowLogic>,
    bg1_win_logic: Cell<WindowLogic>,

    // $212B    .... CCOO    Write Only
    //       - Window mask logic for OBJ (O) and color (C)
    obj_win_logic: Cell<WindowLogic>,
    col_win_logic: Cell<WindowLogic>,

    // $212C    ...O 4321    Write Only
    //       - Main screen layer enable (#)
    obj_main_enabled: Cell<bool>,
    bg4_main_enabled: Cell<bool>,
    bg3_main_enabled: Cell<bool>,
    bg2_main_enabled: Cell<bool>,
    bg1_main_enabled: Cell<bool>,

    // $212D    ...O 4321    Write Only
    //       - Sub screen layer enable (#)
    obj_sub_enabled: Cell<bool>,
    bg4_sub_enabled: Cell<bool>,
    bg3_sub_enabled: Cell<bool>,
    bg2_sub_enabled: Cell<bool>,
    bg1_sub_enabled: Cell<bool>,

    // $212E    ...O 4321    Write Only
    //       - Main screen layer window enable
    obj_win_main_enabled: Cell<bool>,
    bg4_win_main_enabled: Cell<bool>,
    bg3_win_main_enabled: Cell<bool>,
    bg2_win_main_enabled: Cell<bool>,
    bg1_win_main_enabled: Cell<bool>,

    // $212F    ...O 4321    Write Only
    //       - Sub screen layer window enable
    obj_win_sub_enabled: Cell<bool>,
    bg4_win_sub_enabled: Cell<bool>,
    bg3_win_sub_enabled: Cell<bool>,
    bg2_win_sub_enabled: Cell<bool>,
    bg1_win_sub_enabled: Cell<bool>,

    // $2130    MMSS ..AD    Write Only
    //       - main/sub screen color window black/transparent regions (MS)
    //       - fixed/subscreen (A)
    //       - direct color (D)
    col_win_main_region: Cell<WindowColorRegion>,
    col_win_sub_region: Cell<WindowColorRegion>,
    sub_color_fixed: Cell<bool>,
    use_direct_col: Cell<bool>,

    // $2131    MHBO 4321    Write Only
    //       - Color math add/subtract (M)
    //       - half (H)
    //       - backdrop (B)
    //       - layer enable (O4321)
    cmath_operator: Cell<CMathOperator>,
    cmath_half: Cell<bool>,
    back_cmath_enabled: Cell<bool>,
    obj_cmath_enabled: Cell<bool>,
    bg4_cmath_enabled: Cell<bool>,
    bg3_cmath_enabled: Cell<bool>,
    bg2_cmath_enabled: Cell<bool>,
    bg1_cmath_enabled: Cell<bool>,

    // $2132    BGRC CCCC    Write Only
    //       - Fixed color channel select (BGR) and value (C)
    fixed_color: Cell<u16>,

    // $2133    EX.. HOiI    Write Only
    //       - External sync (E)
    //       - EXTBG (X)
    //       - Hi-res (H)
    //       - Overscan (O)
    //       - OBJ interlace (i)
    //       - Screen interlace (I)
    _external_sync: Cell<bool>,
    ext_bg_enabled: Cell<bool>,
    hi_res_enabled: Cell<bool>,
    overscan_enabled: Cell<bool>,
    obj_interlace_enabled: Cell<bool>,
    screen_interlace_enabled: Cell<bool>,

    // $2134    LLLL LLLL    Read Only
    // $2135    MMMM MMMM    Read Only
    // $2136    HHHH HHHH    Read Only
    //       - 24-bit signed multiplication result (read 8 bits per register)
    multiply_result: Cell<u32>,

    // $2137    .... ....    Read Only
    //       - Software latch for H/V counters
    // READ CPU OPEN BUS

    // $2138    DDDD DDDD    Read Only
    //       - Read OAM data byte, increments OAMADD byte

    // $2139    LLLL LLLL
    // $213A    HHHH HHHH    Read x2 Only
    //       - VRAM data read. Increments VMADD after read according to VMAIN setting
    vram_latch: Cell<u16>,

    // $213B    .BBB BBGG GGGR RRRR    Read Only
    //       - CGRAM data read, increments CGADD byte address after each write

    // $213C    ...H HHHH HHHH HHHH    Read Only
    //       - Output horizontal counter (latched)
    h_counter_toggle: Cell<ToggleState>,
    h_counter_latch: Cell<u16>,

    // $213D    ...V VVVV VVVV VVVV    Read Only
    //       - Output vertical counter
    v_counter_toggle: Cell<ToggleState>,
    v_counter_latch: Cell<u16>,

    // STAT77    $213E    TRM. VVVV    Read Only
    //       - Sprite overflow (T)
    //       - sprite tile overflow (R)
    //       - master/slave (M)
    //       - PPU1 version (V)
    sprite_overflow: Cell<bool>,
    sprite_tile_overflow: Cell<bool>,
    master_slave_state: Cell<MasterSlave>,
    ppu1_version: Cell<u8>,

    // STAT78    $213F    FL.M VVVV    Read Only
    //       - Interlace field (F)
    //       - counter latch value (L)
    //       - NTSC/PAL (M)
    //       - PPU2 version (V)
    interlace_field: Cell<bool>,
    counter_toggle: Cell<ToggleState>,
    video_type: Cell<VideoType>,
    ppu2_version: Cell<u8>,

    oam: Vec<Cell<u8>>, // 544 Bytes of OAM
    vram: Vec<Cell<u16>>, // 64 KiB of VRAM
    cgram: Vec<Cell<u16>>,

    in_vblank: Cell<bool>,
    in_hblank: Cell<bool>,

    h_counter: Cell<u16>,
    v_counter: Cell<u16>,

    pub h_counter_target: Cell<u16>,
    pub v_counter_target: Cell<u16>,
    
    pub hv_timer_irq_mode: Cell<HVTimerIRQ>,
    pub hv_timer_irq: Cell<bool>,
    pub cpu_vblank_nmi: Cell<bool>,
    pub hblank_start: Cell<bool>,
    pub vblank_start: Cell<bool>,

    logger: Rc<SnemLogger>
}

impl PpuData {
    pub fn new(logger: Rc<SnemLogger>) -> PpuData {
        PpuData {
            scanline: Cell::new(0),
            dot: Cell::new(0),

            in_fblank: Cell::new(false),
            screen_brightness: Cell::new(0),

            obj_sprite_size: Cell::new(ObjectSizeSelect::Size8x8_16x16),
            name_secondary_select: Cell::new(0),
            name_base_addr: Cell::new(0),

            oam_addr: Cell::new(0),
            internal_oam_addr: Cell::new(0),
            priority_rotation: Cell::new(false),
            priority_rotation_idx: Cell::new(0),

            oam_data_latch: Cell::new(0),

            bg4_char_size: Cell::new(TileSize::Size8x8),
            bg3_char_size: Cell::new(TileSize::Size8x8),
            bg2_char_size: Cell::new(TileSize::Size8x8),
            bg1_char_size: Cell::new(TileSize::Size8x8),
            bg3_mode1_priority: Cell::new(false),
            bg_mode: Cell::new(BgMode::Mode0),

            mosaic_size: Cell::new(0),
            bg4_mosaic: Cell::new(false),
            bg3_mosaic: Cell::new(false),
            bg2_mosaic: Cell::new(false),
            bg1_mosaic: Cell::new(false),

            bg1_vram_addr: Cell::new(0),
            bg1_tilemap_count_y: Cell::new(TilemapCount::One),
            bg1_tilemap_count_x: Cell::new(TilemapCount::One),

            bg2_vram_addr: Cell::new(0),
            bg2_tilemap_count_y: Cell::new(TilemapCount::One),
            bg2_tilemap_count_x: Cell::new(TilemapCount::One),

            bg3_vram_addr: Cell::new(0),
            bg3_tilemap_count_y: Cell::new(TilemapCount::One),
            bg3_tilemap_count_x: Cell::new(TilemapCount::One),

            bg4_vram_addr: Cell::new(0),
            bg4_tilemap_count_y: Cell::new(TilemapCount::One),
            bg4_tilemap_count_x: Cell::new(TilemapCount::One),

            bg2_chr_base_addr: Cell::new(0),
            bg1_chr_base_addr: Cell::new(0),

            bg4_chr_base_addr: Cell::new(0),
            bg3_chr_base_addr: Cell::new(0),

            m7_latch: Cell::new(0),

            bg_offset_latch: Cell::new(0),
            bg_offset_x_latch: Cell::new(0),

            bg1_m7_x_offset: Cell::new(0),

            bg1_m7_y_offset: Cell::new(0),

            bg2_x_offset: Cell::new(0),

            bg2_y_offset: Cell::new(0),

            bg3_x_offset: Cell::new(0),

            bg3_y_offset: Cell::new(0),

            bg4_x_offset: Cell::new(0),

            bg4_y_offset: Cell::new(0),

            vram_addr_inc_mode: Cell::new(VramIncMode::HighByte),
            addr_remap_mode: Cell::new(AddressRemapping::None),
            addr_inc_size: Cell::new(IncrSize::Bytes2),

            vram_addr: Cell::new(0),

            m7_tilemap_repeat: Cell::new(false),
            m7_fill_mode: Cell::new(M7FillMode::Transparent),
            m7_flip_bg_y: Cell::new(false),
            m7_flip_bg_x: Cell::new(false),

            m7_matrix_a: Cell::new(0),
            mult_factor_16: Cell::new(0),

            m7_matrix_b: Cell::new(0),
            mult_factor_8: Cell::new(0),

            m7_matrix_c: Cell::new(0),

            m7_matrix_d: Cell::new(0),

            m7_center_x: Cell::new(0),

            m7_center_y: Cell::new(0),

            cgram_toggle: Cell::new(ToggleState::default()),

            cgram_addr: Cell::new(0),
            cgram_latch: Cell::new(0),

            bg2_w2_enabled: Cell::new(false),
            bg2_w2_inverted: Cell::new(false),
            bg2_w1_enabled: Cell::new(false),
            bg2_w1_inverted: Cell::new(false),
            bg1_w2_enabled: Cell::new(false),
            bg1_w2_inverted: Cell::new(false),
            bg1_w1_enabled: Cell::new(false),
            bg1_w1_inverted: Cell::new(false),

            bg4_w2_enabled: Cell::new(false),
            bg4_w2_inverted: Cell::new(false),
            bg4_w1_enabled: Cell::new(false),
            bg4_w1_inverted: Cell::new(false),
            bg3_w2_enabled: Cell::new(false),
            bg3_w2_inverted: Cell::new(false),
            bg3_w1_enabled: Cell::new(false),
            bg3_w1_inverted: Cell::new(false),

            col_w2_enabled: Cell::new(false),
            col_w2_inverted: Cell::new(false),
            col_w1_enabled: Cell::new(false),
            col_w1_inverted: Cell::new(false),
            obj_w2_enabled: Cell::new(false),
            obj_w2_inverted: Cell::new(false),
            obj_w1_enabled: Cell::new(false),
            obj_w1_inverted: Cell::new(false),

            w1_left_pos: Cell::new(0),

            w1_right_pos: Cell::new(0),

            w2_left_pos: Cell::new(0),

            w2_right_pos: Cell::new(0),

            bg4_win_logic: Cell::new(WindowLogic::Or),
            bg3_win_logic: Cell::new(WindowLogic::Or),
            bg2_win_logic: Cell::new(WindowLogic::Or),
            bg1_win_logic: Cell::new(WindowLogic::Or),

            obj_win_logic: Cell::new(WindowLogic::Or),
            col_win_logic: Cell::new(WindowLogic::Or),

            obj_main_enabled: Cell::new(false),
            bg4_main_enabled: Cell::new(false),
            bg3_main_enabled: Cell::new(false),
            bg2_main_enabled: Cell::new(false),
            bg1_main_enabled: Cell::new(false),

            obj_sub_enabled: Cell::new(false),
            bg4_sub_enabled: Cell::new(false),
            bg3_sub_enabled: Cell::new(false),
            bg2_sub_enabled: Cell::new(false),
            bg1_sub_enabled: Cell::new(false),

            obj_win_main_enabled: Cell::new(false),
            bg4_win_main_enabled: Cell::new(false),
            bg3_win_main_enabled: Cell::new(false),
            bg2_win_main_enabled: Cell::new(false),
            bg1_win_main_enabled: Cell::new(false),

            obj_win_sub_enabled: Cell::new(false),
            bg4_win_sub_enabled: Cell::new(false),
            bg3_win_sub_enabled: Cell::new(false),
            bg2_win_sub_enabled: Cell::new(false),
            bg1_win_sub_enabled: Cell::new(false),

            col_win_main_region: Cell::new(WindowColorRegion::Nowhere),
            col_win_sub_region: Cell::new(WindowColorRegion::Nowhere),
            sub_color_fixed: Cell::new(false),
            use_direct_col: Cell::new(false),

            cmath_operator: Cell::new(CMathOperator::Add),
            cmath_half: Cell::new(false),
            back_cmath_enabled: Cell::new(false),
            obj_cmath_enabled: Cell::new(false),
            bg4_cmath_enabled: Cell::new(false),
            bg3_cmath_enabled: Cell::new(false),
            bg2_cmath_enabled: Cell::new(false),
            bg1_cmath_enabled: Cell::new(false),

            fixed_color: Cell::new(0),

            _external_sync: Cell::new(false),
            ext_bg_enabled: Cell::new(false),
            hi_res_enabled: Cell::new(false),
            overscan_enabled: Cell::new(false),
            obj_interlace_enabled: Cell::new(false),
            screen_interlace_enabled: Cell::new(false),

            multiply_result: Cell::new(u32::default()),

            vram_latch: Cell::new(0),

            h_counter_toggle: Cell::new(ToggleState::default()),
            h_counter_latch: Cell::new(0),

            v_counter_toggle: Cell::new(ToggleState::default()),
            v_counter_latch: Cell::new(0),

            sprite_overflow: Cell::new(false),
            sprite_tile_overflow: Cell::new(false),
            master_slave_state: Cell::new(MasterSlave::default()),
            ppu1_version: Cell::new(0),

            interlace_field: Cell::new(false),
            counter_toggle: Cell::new(ToggleState::default()),
            video_type: Cell::new(VideoType::default()),
            ppu2_version: Cell::new(0),
            
            oam: vec![Cell::new(0); OAM_SIZE],
            vram: vec![Cell::new(0); VRAM_SIZE],
            cgram: vec![Cell::new(0); CGRAM_SIZE],

            in_vblank: Cell::new(false),
            in_hblank: Cell::new(false),
            
            h_counter: Cell::new(0),
            v_counter: Cell::new(0),            
            h_counter_target: Cell::new(0),
            v_counter_target: Cell::new(0),
            hv_timer_irq_mode: Cell::new(HVTimerIRQ::None),

            cpu_vblank_nmi: Cell::new(false),
            hv_timer_irq: Cell::new(false),
            hblank_start: Cell::new(false),
            vblank_start: Cell::new(false),

            logger,
        }
    }
}

// CPU Access
impl PpuData {
    pub fn write(&self, address: u8, data: u8) {
        match address {
            0x00 => {
                self.in_fblank.set(data.bit_en(7));
                self.screen_brightness.set(data & 0x0F);

                // println!("Set fblank to {}, S: {} D: {}", self.in_fblank.get(), self.scanline.get(), self.dot.get());
            }

            0x01 => {
                let new_obj_size = match data >> 5 {
                    0 => ObjectSizeSelect::Size8x8_16x16,
                    1 => ObjectSizeSelect::Size8x8_32x32,
                    2 => ObjectSizeSelect::Size8x8_64x64,
                    3 => ObjectSizeSelect::Size16x16_32x32,
                    4 => ObjectSizeSelect::Size16x16_64x64,
                    5 => ObjectSizeSelect::Size32x32_64x64,
                    6 => ObjectSizeSelect::Size16x32_32x64,
                    7 => ObjectSizeSelect::Size16x32_32x32,
                    _ => unreachable!()
                };

                self.obj_sprite_size.set(new_obj_size);
                self.name_secondary_select.set((data >> 3) & 0x03);
                self.name_base_addr.set(data & 0x03);

                // println!("Set name base addr to ${:04X}", (self.name_base_addr.get() as u16) << 13);
            }

            0x02 => {
                let new_addr = (self.oam_addr.get() & 0xFF00) | (data as u16);

                self.oam_addr.set(new_addr);
                self.priority_rotation_idx.set(data & 0xFE);
                self.internal_oam_addr.set((self.oam_addr.get() & 0x1FF) << 1);
            }

            0x03 => {
                let new_addr = self.oam_addr.get() & 0x00FF | ((data as u16) << 8);

                self.oam_addr.set(new_addr);
                self.priority_rotation.set(data.bit_en(7));
                self.internal_oam_addr.set((self.oam_addr.get() & 0x1FF) << 1);
            }

            0x04 => {
                let internal_oam_addr = self.internal_oam_addr.get() as usize;

                if internal_oam_addr & 1 == 0 {
                    self.oam_data_latch.set(data);
                } else if internal_oam_addr < 0x200 {
                    self.oam[internal_oam_addr - 1].set(self.oam_data_latch.get());
                    self.oam[internal_oam_addr].set(data);
                }
                
                if internal_oam_addr >= 0x200 {
                    self.oam[internal_oam_addr % OAM_SIZE].set(data); // this is lazy, doesn't actually work with mod
                }

                self.internal_oam_addr.set((internal_oam_addr as u16 + 1) % OAM_SIZE as u16);
            }

            0x05 => {
                self.bg4_char_size.set(
                    if data.bit_en(7) { TileSize::Size16x16 } else { TileSize::Size8x8 }
                );
                self.bg3_char_size.set(
                    if data.bit_en(6) { TileSize::Size16x16 } else { TileSize::Size8x8 }
                );
                self.bg2_char_size.set(
                    if data.bit_en(5) { TileSize::Size16x16 } else { TileSize::Size8x8 }
                );
                self.bg1_char_size.set(
                    if data.bit_en(4) { TileSize::Size16x16 } else { TileSize::Size8x8 }
                );
                self.bg3_mode1_priority.set(data.bit_en(3));
                self.bg_mode.set(
                    match data & 7 {
                        0 => BgMode::Mode0,
                        1 => BgMode::Mode1,
                        2 => BgMode::Mode2,
                        3 => BgMode::Mode3,
                        4 => BgMode::Mode4,
                        5 => BgMode::Mode5,
                        6 => BgMode::Mode6,
                        7 => BgMode::Mode7,
                        _ => unreachable!(),
                    }
                );

                // println!("Set Bg Mode to {:?} and bg3 priority to {}, bg tile sizes to bg1: {:?}, bg2: {:?}, bg3: {:?}, bg4: {:?}",
                //     self.bg_mode.get(),
                //     self.bg3_mode1_priority.get(),
                //     self.bg1_char_size.get(),
                //     self.bg2_char_size.get(),
                //     self.bg3_char_size.get(),
                //     self.bg4_char_size.get(),
                // );
            }

            0x06 => {
                self.mosaic_size.set(data >> 4);
                self.bg4_mosaic.set(data.bit_en(3));
                self.bg3_mosaic.set(data.bit_en(2));
                self.bg2_mosaic.set(data.bit_en(1));
                self.bg1_mosaic.set(data.bit_en(0));
            }

            0x07 => {
                self.bg1_vram_addr.set(data >> 2);
                self.bg1_tilemap_count_y.set(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg1_tilemap_count_x.set(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
                );

                println!("Set Bg1 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}", 
                    (self.bg1_vram_addr.get() as u16) << 10, 
                    self.bg1_tilemap_count_x.get(), 
                    self.bg1_tilemap_count_y.get()
                );
            }

            0x08 => {
                self.bg2_vram_addr.set(data >> 2);
                self.bg2_tilemap_count_y.set(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg2_tilemap_count_x.set(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
                );

                println!("Set Bg2 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}", 
                    (self.bg2_vram_addr.get() as u16) << 10, 
                    self.bg2_tilemap_count_x.get(), 
                    self.bg2_tilemap_count_y.get()
                );
            }

            0x09 => {
                self.bg3_vram_addr.set(data >> 2);
                self.bg3_tilemap_count_y.set(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg3_tilemap_count_x.set(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
                );

                println!("Set Bg3 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}", 
                    (self.bg3_vram_addr.get() as u16) << 10, 
                    self.bg3_tilemap_count_x.get(), 
                    self.bg3_tilemap_count_y.get()
                );
            }

            0x0A => {
                self.bg4_vram_addr.set(data >> 2);
                self.bg4_tilemap_count_y.set(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg4_tilemap_count_x.set(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
                );

                println!("Set Bg4 vram base addr to ${:04X}, count_x: {:?}, count_y: {:?}", 
                    (self.bg4_vram_addr.get() as u16) << 10, 
                    self.bg4_tilemap_count_x.get(), 
                    self.bg4_tilemap_count_y.get()
                );
            }

            0x0B => {
                self.bg2_chr_base_addr.set(data >> 4);
                self.bg1_chr_base_addr.set(data & 0x0F);

                println!("Set Bg1 chr base address to ${:04X}", (self.bg1_chr_base_addr.get() as u16) << 12);
                println!("Set Bg2 chr base address to ${:04X}", (self.bg2_chr_base_addr.get() as u16) << 12);
            }

            0x0C => {
                self.bg4_chr_base_addr.set(data >> 4);
                self.bg3_chr_base_addr.set(data & 0x0F);

                println!("Set Bg3 chr base address to ${:04X}", (self.bg3_chr_base_addr.get() as u16) << 12);
                println!("Set Bg4 chr base address to ${:04X}", (self.bg4_chr_base_addr.get() as u16) << 12);
            }

            0x0D => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg1_m7_x_offset.set(
                    (((data & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );
            }

            0x0E => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg1_m7_y_offset.set((((data & 3) as u16) << 8) | bgofs_latch);
            }

            0x0F => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg2_x_offset.set(
                    (((data & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );
            }

            0x10 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg2_y_offset.set((((data & 3) as u16) << 8) | bgofs_latch);
            }

            0x11 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg3_x_offset.set(
                    (((data & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );
            }

            0x12 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg3_y_offset.set((((data & 3) as u16) << 8) | bgofs_latch);
            }

            0x13 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg4_x_offset.set(
                    (((data & 3) as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );
            }

            0x14 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg4_y_offset.set((((data & 3) as u16) << 8) | bgofs_latch);
            }

            0x15 => {
                self.vram_addr_inc_mode.set(
                    if data.bit_en(7) { VramIncMode::HighByte } else { VramIncMode::LowByte }
                );
                self.addr_remap_mode.set(
                    match (data >> 2) & 3 {
                        0 => AddressRemapping::None,
                        1 => AddressRemapping::ColDepth2,
                        2 => AddressRemapping::ColDepth4,
                        3 => AddressRemapping::ColDepth8,
                        _ => unreachable!(),
                    }
                );
                self.addr_inc_size.set(
                    match data & 3 {
                        0 => IncrSize::Bytes2,
                        1 => IncrSize::Bytes64,
                        2 => IncrSize::Bytes256,
                        3 => IncrSize::Bytes256,
                        _ => unreachable!(),
                    }
                );
            }

            0x16 => {
                self.vram_addr.set_lo(data);
                self.vram_latch.set(
                    self.vram[self.get_vram_addr() as usize].get()
                );

                // println!("Set vram addr (lo) to ${:04X}", self.vram_addr.get());
            }

            0x17 => {
                self.vram_addr.set_hi(data);
                self.vram_latch.set(
                    self.vram[self.get_vram_addr() as usize].get()
                );

                // println!("Set vram addr (hi) to ${:04X}", self.vram_addr.get());
            }

            0x18 => {
                if self.in_fblank.get() || self.in_vblank.get() {
                    self.vram[self.get_vram_addr() as usize].set_lo(data);
                }

                match self.vram_addr_inc_mode.get() {
                    VramIncMode::LowByte => self.inc_vram_addr(),
                    _ => {}
                }
            }

            0x19 => {
                if self.in_fblank.get() || self.in_vblank.get() {
                    self.vram[self.get_vram_addr() as usize].set_hi(data);
                }

                match self.vram_addr_inc_mode.get() {
                    VramIncMode::HighByte => self.inc_vram_addr(),
                    _ => {}
                }
            }

            0x1A => {
                self.m7_tilemap_repeat.set(data.bit_en(7));
                self.m7_fill_mode.set(
                    if data.bit_en(6) { M7FillMode::Character } else { M7FillMode::Transparent }
                );
                self.m7_flip_bg_y.set(data.bit_en(1));
                self.m7_flip_bg_x.set(data.bit_en(0));
            }

            0x1B => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_a.set(
                    ((data as u16) << 8) | latched_val
                );
                self.mult_factor_16.set(
                    ((data as u16) << 8) | latched_val
                );

                self.update_multiply_result();
            }

            0x1C => {
                let latched_val = self.m7_latch.replace(data);

                self.m7_matrix_b.set(
                    ((data as u16) << 8) | (latched_val as u16)
                );
                self.mult_factor_8.set(latched_val);

                self.update_multiply_result();
            }

            0x1D => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_c.set(((data as u16) << 8) | latched_val);
            }

            0x1E => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_d.set(((data as u16) << 8) | latched_val);
            }

            0x1F => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_center_x.set(((data as u16) << 8) | latched_val);
            }

            0x20 => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_center_y.set(((data as u16) << 8) | latched_val);
            }

            0x21 => {
                self.cgram_addr.set(data);
                self.cgram_toggle.set_lo();
            }

            0x22 => {
                if self.cgram_toggle.toggle() {
                    let addr = self.cgram_addr.get();
                    let new_col = ((data as u16) << 8) | self.cgram_latch.get() as u16;

                    let rgb565 = xbgr0555_to_rgb565(new_col);

                    self.cgram[addr as usize].set(rgb565);

                    self.cgram_addr.set(addr + 1);
                } else {
                    self.cgram_latch.set(data);
                }
            }

            0x23 => {
                self.bg2_w2_enabled.set(data.bit_en(7));
                self.bg2_w2_inverted.set(data.bit_en(6));
                self.bg2_w1_enabled.set(data.bit_en(5));
                self.bg2_w1_inverted.set(data.bit_en(4));
                self.bg1_w2_enabled.set(data.bit_en(3));
                self.bg1_w2_inverted.set(data.bit_en(2));
                self.bg1_w1_enabled.set(data.bit_en(1));
                self.bg1_w1_inverted.set(data.bit_en(0));
            }

            0x24 => {
                self.bg4_w2_enabled.set(data.bit_en(7));
                self.bg4_w2_inverted.set(data.bit_en(6));
                self.bg4_w1_enabled.set(data.bit_en(5));
                self.bg4_w1_inverted.set(data.bit_en(4));
                self.bg3_w2_enabled.set(data.bit_en(3));
                self.bg3_w2_inverted.set(data.bit_en(2));
                self.bg3_w1_enabled.set(data.bit_en(1));
                self.bg3_w1_inverted.set(data.bit_en(0));
            }

            0x25 => {
                self.col_w2_enabled.set(data.bit_en(7));
                self.col_w2_inverted.set(data.bit_en(6));
                self.col_w1_enabled.set(data.bit_en(5));
                self.col_w1_inverted.set(data.bit_en(4));
                self.obj_w2_enabled.set(data.bit_en(3));
                self.obj_w2_inverted.set(data.bit_en(2));
                self.obj_w1_enabled.set(data.bit_en(1));
                self.obj_w1_inverted.set(data.bit_en(0));
            }

            0x26 => { self.w1_left_pos.set(data); }
            0x27 => { self.w1_right_pos.set(data); }
            0x28 => { self.w2_left_pos.set(data); }
            0x29 => { self.w2_right_pos.set(data); }

            0x2A => {
                self.bg4_win_logic.set(
                    match data >> 6 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.bg3_win_logic.set(
                    match (data >> 4) & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.bg2_win_logic.set(
                    match (data >> 2) & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.bg1_win_logic.set(
                    match data & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
            }

            0x2B => {
                self.col_win_logic.set(
                    match (data >> 2) & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.obj_win_logic.set(
                    match data & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
            }

            0x2C => {
                self.obj_main_enabled.set(data.bit_en(4));
                self.bg4_main_enabled.set(data.bit_en(3));
                self.bg3_main_enabled.set(data.bit_en(2));
                self.bg2_main_enabled.set(data.bit_en(1));
                self.bg1_main_enabled.set(data.bit_en(0));

                println!("Set main en flags to Bg1: {}, Bg2: {}, Bg3: {}, Bg4: {}, Obj: {}",
                    self.bg1_main_enabled.get(),
                    self.bg2_main_enabled.get(),
                    self.bg3_main_enabled.get(),
                    self.bg4_main_enabled.get(),
                    self.obj_main_enabled.get(),
                );
            }

            0x2D => {
                self.obj_sub_enabled.set(data.bit_en(4));
                self.bg4_sub_enabled.set(data.bit_en(3));
                self.bg3_sub_enabled.set(data.bit_en(2));
                self.bg2_sub_enabled.set(data.bit_en(1));
                self.bg1_sub_enabled.set(data.bit_en(0));
            }

            0x2E => {
                self.obj_win_main_enabled.set(data.bit_en(4));
                self.bg4_win_main_enabled.set(data.bit_en(3));
                self.bg3_win_main_enabled.set(data.bit_en(2));
                self.bg2_win_main_enabled.set(data.bit_en(1));
                self.bg1_win_main_enabled.set(data.bit_en(0));
            }

            0x2F => {
                self.obj_win_sub_enabled.set(data.bit_en(4));
                self.bg4_win_sub_enabled.set(data.bit_en(3));
                self.bg3_win_sub_enabled.set(data.bit_en(2));
                self.bg2_win_sub_enabled.set(data.bit_en(1));
                self.bg1_win_sub_enabled.set(data.bit_en(0));
            }

            0x30 => {
                self.col_win_main_region.set(
                    match data >> 6 {
                        0 => WindowColorRegion::Nowhere,
                        1 => WindowColorRegion::Outside,
                        2 => WindowColorRegion::Inside,
                        3 => WindowColorRegion::Everywhere,
                        _ => unreachable!(),
                    }
                );
                self.col_win_sub_region.set(
                    match (data >> 4) & 3 {
                        0 => WindowColorRegion::Nowhere,
                        1 => WindowColorRegion::Outside,
                        2 => WindowColorRegion::Inside,
                        3 => WindowColorRegion::Everywhere,
                        _ => unreachable!(),
                    }
                );
                self.sub_color_fixed.set(!data.bit_en(1));
                self.use_direct_col.set(data.bit_en(0));
            }

            0x31 => {
                self.cmath_operator.set(
                    match data >> 7 {
                        0 => CMathOperator::Add,
                        1 => CMathOperator::Subtract,
                        _ => unreachable!(),
                    }
                );
                self.cmath_half.set(data.bit_en(6));
                self.back_cmath_enabled.set(data.bit_en(5));
                self.obj_cmath_enabled.set(data.bit_en(4));
                self.bg4_cmath_enabled.set(data.bit_en(3));
                self.bg3_cmath_enabled.set(data.bit_en(2));
                self.bg2_cmath_enabled.set(data.bit_en(1));
                self.bg1_cmath_enabled.set(data.bit_en(0));
            }

            0x32 => {
                let prev_col = self.fixed_color.get();

                let (prev_r, prev_g, prev_b) = rgb565_to_parts(prev_col);

                let new_val = (data & 0x1F) as u16;

                let new_r = if data.bit_en(5) { new_val } else { prev_r };
                let new_g = if data.bit_en(6) { new_val } else { prev_g };
                let new_b = if data.bit_en(7) { new_val } else { prev_b };

                let new_col = rgb565_from_parts(new_r, new_g, new_b);

                self.fixed_color.set(new_col);
            }

            0x33 => {
                self._external_sync.set(data.bit_en(7));
                self.ext_bg_enabled.set(data.bit_en(6));
                self.hi_res_enabled.set(data.bit_en(3));
                self.overscan_enabled.set(data.bit_en(2));
                self.obj_interlace_enabled.set(data.bit_en(1));
                self.screen_interlace_enabled.set(data.bit_en(0));
            }

            _ => {}
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        let data = match address {
            0x34 => { self.multiply_result.get() as u8 }
            0x35 => { (self.multiply_result.get() >> 8) as u8 }
            0x36 => { (self.multiply_result.get() >> 16) as u8 }

            0x37 => {
                // When counter_latch transitions from 0 to 1
                // https://snes.nesdev.org/wiki/PPU_registers#OPVCT
                if !self.counter_toggle.is_high() {
                    self.h_counter_latch.set(self.h_counter.get());
                    self.v_counter_latch.set(self.v_counter.get());
                }

                self.counter_toggle.set_hi();

                0 // CPU OPEN BUS
            }

            0x38 => {
                let addr = self.internal_oam_addr.replace(self.internal_oam_addr.get() + 1);

                self.oam[addr as usize].get()
            }

            0x39 => {
                let val = self.vram_latch.get() as u8;

                match self.vram_addr_inc_mode.get() {
                    VramIncMode::LowByte => {
                        self.vram_latch.set(
                            if self.in_fblank.get() || self.in_vblank.get() {
                                self.vram[self.get_vram_addr() as usize].get()
                            } else {
                                0
                            }
                        );
                        self.inc_vram_addr();
                    }

                    _ => {}
                }

                val
            }

            0x3A => {
                let val = (self.vram_latch.get() >> 8) as u8;

                match self.vram_addr_inc_mode.get() {
                    VramIncMode::HighByte => {
                        self.vram_latch.set(
                            if self.in_fblank.get() || self.in_vblank.get() {
                                self.vram[self.get_vram_addr() as usize].get()
                            } else {
                                0
                            }
                        );
                        self.inc_vram_addr();
                    }

                    _ => {}
                }

                val
            }

            0x3B => {
                let rgb565 = self.cgram[self.cgram_addr.get() as usize].get();
                
                let data = rgb565_to_xbgr0555(rgb565);

                if self.cgram_toggle.toggle() {
                    data as u8
                } else {
                    (data >> 8) as u8
                }
            }

            0x3C => {
                if self.h_counter_toggle.toggle() {
                    (self.h_counter_latch.get() >> 8) as u8 // HIGH 7 BITS ARE PPU2 OPEN BUS
                } else {
                    self.h_counter_latch.get() as u8
                }
            }

            0x3D => {
                if self.v_counter_toggle.toggle() {
                    (self.v_counter_latch.get() >> 8) as u8 // HIGH 7 BITS ARE PPU2 OPEN BUS
                } else {
                    self.v_counter_latch.get() as u8
                }
            }

            0x3E => {
                let spr_overflow_bit = if self.sprite_overflow.get() { 0x80 } else { 0 };
                let spr_tile_overflow_bit = if self.sprite_tile_overflow.get() { 0x40 } else { 0 };
                let master_slave_bit = match self.master_slave_state.get() {
                    MasterSlave::Master => 0x20,
                    MasterSlave::Slave => 0,
                };
                let ppu1_open_bus = 0;
                let ppu1_version_bits = self.ppu1_version.get() & 0x0F;

                spr_overflow_bit | spr_tile_overflow_bit | master_slave_bit | ppu1_open_bus | ppu1_version_bits
            }

            0x3F => {
                let interlace_bit = if self.interlace_field.get() { 0x80 } else { 0 };
                let counter_toggle_bit = if self.counter_toggle.is_high() { 0x40 } else { 0 };
                let ppu2_open_bus = 0;
                let ntsc_pal_bit = match self.video_type.get() {
                    VideoType::Ntsc => 0,
                    VideoType::Pal => 0x10,
                };
                let version_bits = self.ppu2_version.get() & 0x0F;

                self.counter_toggle.set_lo();
                self.h_counter_toggle.set_lo();
                self.v_counter_toggle.set_lo();

                interlace_bit | counter_toggle_bit | ppu2_open_bus | ntsc_pal_bit | version_bits
            }

            _ => { 0 }
        };

        data
    }

    fn update_multiply_result(&self) {
        let lhs = self.mult_factor_16.get() as i16;
        let rhs = self.mult_factor_8.get() as i8;
        let result = ((lhs as i32) * (rhs as i32)) as u32;

        self.multiply_result.set(result & 0x00FFFFFF);
    }

    fn get_vram_addr(&self) -> u16 {
        match self.addr_remap_mode.get() {
            AddressRemapping::None => { self.vram_addr.get() & 0x7FFF },
            AddressRemapping::ColDepth2 => {
                // rrrrrrrr YYYccccc -> rrrrrrrr cccccYYY
                let addr = self.vram_addr.get();

                let r = addr & 0x7F00;
                let y = (addr & 0x00E0) >> 5;
                let c = (addr & 0x1F) << 3;

                r | c | y
            }
            AddressRemapping::ColDepth4 => {
                // rrrrrrrY YYcccccP -> rrrrrrrc ccccPYYY
                let addr = self.vram_addr.get();

                let r = addr & 0x7E00;
                let y = (addr & 0x01C0) >> 6;
                let cp = (addr & 0x003F) << 3;

                r | cp | y
            }
            AddressRemapping::ColDepth8 => {
                // rrrrrrYY YcccccPP -> rrrrrrcc cccPPYYY
                let addr = self.vram_addr.get();

                let r = addr & 0x7C00;
                let y = (addr & 0x0380) >> 7;
                let cp = (addr & 0x007F) << 3;

                r | cp | y
            }
        }
    }

    fn inc_vram_addr(&self) {
        let inc = match self.addr_inc_size.get() {
            IncrSize::Bytes2 => 1,
            IncrSize::Bytes64 => 32,
            IncrSize::Bytes256 => 128,
        };

        self.vram_addr.set(self.vram_addr.get() + inc);
    }

    pub fn in_vblank(&self) -> bool { self.in_vblank.get() }
    pub fn in_hblank(&self) -> bool { self.in_hblank.get() }
    pub fn cpu_vblank_nmi(&self) -> bool { self.cpu_vblank_nmi.get() }
    pub fn clear_cpu_vblank_nmi(&self) { self.cpu_vblank_nmi.set(false); }
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

pub enum FrameBufferSize {
    Size256x240,
    Size512x480,
}

pub struct Ppu5C7x {
    registers: Rc<PpuData>,

    dot: usize,
    scanline: usize,
    sys_clocks_until_clock: usize,
    frame: usize,

    scanline_sprites: Vec<OAMSprite>,
    scanline_spr_cnt: usize,

    pub frame_finished: bool,
    pub new_frame_buf_size: Option<FrameBufferSize>,

    logger: Rc<SnemLogger>,
}

impl Ppu5C7x {
    pub fn new(ppu_data: Rc<PpuData>, logger: Rc<SnemLogger>) -> Self {
        Ppu5C7x {
            registers: ppu_data,
            dot: 0,
            scanline: 0,
            sys_clocks_until_clock: 1,
            frame: 0,
            scanline_sprites: Vec::with_capacity(32),
            scanline_spr_cnt: 0,
            frame_finished: false,
            new_frame_buf_size: None,
            logger,
        }
    }

    pub fn remove_clocks(&mut self, clocks: usize) { self.sys_clocks_until_clock -= clocks; }
    pub fn sys_clocks_left(&self) -> usize { self.sys_clocks_until_clock }

    /// Clocks the PPU until the next dot is complete
    pub fn clock(&mut self, frame_buffer: &mut [RGB565]) {
        self.sys_clocks_until_clock = 0;

        if !self.in_vblank() && !self.in_hblank() && self.scanline != 0 {
            self.dot(frame_buffer);
        }

        self.update_dot_and_scanline();

        self.registers.scanline.set(self.scanline);
        self.registers.dot.set(self.dot);

        self.sys_clocks_until_clock += 4;

        if self.dot >= SCANLINE_END_DOT-4 {
            self.sys_clocks_until_clock += 1;
        }
    }

    fn update_dot_and_scanline(&mut self) {
        self.dot += 1;

        if self.dot == SCANLINE_END_DOT {
            self.dot = 0;
            self.scanline += 1;

            if self.scanline == VBLANK_END_SCANLINE_NTSC {
                self.scanline = 0;
            }
        }

       match (self.dot, self.scanline) {
            // End of v-blank, scanline 0 is not visible
            (0, 0) => {
                self.registers.in_vblank.set(false);
                self.registers.vblank_start.set(false);
                self.registers.cpu_vblank_nmi.set(false);
            }
            // Start of visible scanline, end of h-blank
            (HBLANK_END_DOT, _) => {
                self.registers.in_hblank.set(false);

                if self.screen_y() < VBLANK_START_SCANLINE {
                    self.find_scanline_sprites();
                }
            }
            // End of scanline, start of h-blank
            (HBLANK_START_DOT, 0..VBLANK_START_SCANLINE) => {
                self.registers.in_hblank.set(true);
                self.registers.hblank_start.set(true);
            }
            // Dot after hblank start
            (286, 0..VBLANK_START_SCANLINE) => {
                self.registers.hblank_start.set(false);
            }
            // Start of v-blank
            (0, VBLANK_START_SCANLINE) => {
                self.registers.in_vblank.set(true);
                self.registers.vblank_start.set(true);
                self.registers.cpu_vblank_nmi.set(true);
                self.frame_finished = true;
                self.frame += 1;
            }
            _ => {}
        }

        self.update_hv_timers();
    }

    fn update_hv_timers(&self) {
        self.registers.h_counter.set(self.dot as u16);
        self.registers.v_counter.set(self.scanline as u16);

        match self.registers.hv_timer_irq_mode.get() {
            HVTimerIRQ::None => {}
            HVTimerIRQ::HTimer => {
                let h_timer = self.registers.h_counter.get();
                let h_target = self.registers.h_counter_target.get();

                if h_timer == h_target {
                    self.registers.hv_timer_irq.set(true);
                }
            }
            HVTimerIRQ::VTimer => {
                let v_timer = self.registers.v_counter.get();
                let v_target = self.registers.v_counter_target.get();
                let h_timer = self.registers.h_counter.get();

                if v_timer == v_target && h_timer == 0 {
                    self.registers.hv_timer_irq.set(true);
                }
            }
            HVTimerIRQ::Both => {
                let v_timer = self.registers.v_counter.get();
                let v_target = self.registers.v_counter_target.get();
                let h_timer = self.registers.h_counter.get();
                let h_target = self.registers.h_counter_target.get();

                if v_timer == v_target && h_timer == h_target {
                    self.registers.hv_timer_irq.set(true);
                }
            }
        }
    }

    /// Finds all possible sprites that could be rendered on the current scanline
    /// based on the y-positions of the sprites
    fn find_scanline_sprites(&mut self) {
        self.scanline_sprites.clear();

        let screen_y = self.screen_y();

        self.scanline_spr_cnt = 0;
        for (spr_idx, spr_data) in self.registers.oam[..0x200].chunks(4).enumerate().rev() {
            // This bit munging is absolutely horrifying but works. We need to 1) get the packed byte containing
            // our data, 2) create a mask to get the bits within the packed byte, and 3) or the byte with the
            // mask to get the relevant bits. Each byte looks like DdCcBbAa, with each letter pair corresponding
            // to a single sprite (32 bytes * 4 pairs = 128, matching # of sprites in OAM).
            let spr_extra_data = (self.registers.oam[0x200 | (spr_idx >> 2)].get() >> ((spr_idx & 3) << 1)) & 3;
            let spr_size_sel = (spr_extra_data & 2) != 0;
            let spr_size = if spr_size_sel {
                match self.obj_sprite_size() {
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
                match self.obj_sprite_size() {
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
            let spr_y = spr_data[1].get();
            let spr_x = (((spr_extra_data as u16) & 1) << 8) | (spr_data[0].get() as u16);
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
                    tile_idx: spr_data[2].get(),
                    use_second_obj_table: (spr_data[3].get() & 1) != 0,
                    palette: (spr_data[3].get() >> 1) & 7,
                    priority: (spr_data[3].get() >> 4) & 3,
                    flip_x: (spr_data[3].get() & 0x40) != 0,
                    flip_y: (spr_data[3].get() & 0x80) != 0,
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
}

/// Performs the window logic to determine whether a window is enabled/disabled 
/// for a particular layer given the window settings for that layer.
// fn window_enable(w1_en: bool, w1_inv: bool, w2_en: bool, w2_inv: bool, win_logic: WindowLogic) -> bool {
//     match win_logic {
//         WindowLogic::Or => (w1_en ^ w1_inv) | (w2_en ^ w2_inv),
//         WindowLogic::And => (w1_en ^ w1_inv) & (w2_en ^ w2_inv),
//         WindowLogic::Xor => (w1_en ^ w1_inv) ^ (w2_en ^ w2_inv),
//         WindowLogic::Xnor => !( (w1_en ^ w1_inv) ^ (w2_en ^ w2_inv) ),
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq)]
enum ColorLayer {
    Bg1,
    Bg2,
    Bg3,
    Bg4,
    Obj,
    Back,
}

#[derive(Clone)]
struct ColorData {
    raw_color: u16,
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

impl Ppu5C7x {
    fn screen_x(&self) -> usize { self.dot as usize - VISIBLE_SCANLINE_START_DOT }
    fn screen_y(&self) -> usize { self.scanline as usize - 1 }
    fn transparent_color(&self) -> u16 { self.registers.cgram[0].get() }
    fn transparent_color_data(&self) -> ColorData { 
        ColorData {
            raw_color: self.transparent_color(),
            priority: 0,
            transparent: true,
        }
    }

    fn dot(&mut self, frame_buffer: &mut [RGB565]) {
        let screen_x = self.screen_x();
        let screen_y = self.screen_y();

        let brightness = self.registers.screen_brightness.get();

        if self.in_fblank() || brightness == 0 {
            frame_buffer[screen_y * 256 + screen_x] = RGB565::new_with_raw_value(0);
            return;
        }

        // All bg modes need spr_col
        let spr_col = self.sprite_col(screen_x, screen_y);

        let dot_col = match self.bg_mode() {
            BgMode::Mode0 => self.bg_mode0_dot(screen_x, screen_y, spr_col),
            BgMode::Mode1 => self.bg_mode1_dot(screen_x, screen_y, spr_col),
            // BgMode::Mode2 => self.bg_mode2_dot(frame_buffer, spr_col),
            // BgMode::Mode3 => self.bg_mode3_dot(frame_buffer, spr_col),
            // BgMode::Mode4 => self.bg_mode4_dot(frame_buffer, spr_col),
            BgMode::Mode5 => self.bg_mode5_dot(screen_x, screen_y, spr_col),
            // BgMode::Mode6 => self.bg_mode6_dot(frame_buffer, spr_col),
            // BgMode::Mode7 => self.bg_mode7_dot(frame_buffer, spr_col),
            _ => 0,
        };

        if brightness == 15 {
            frame_buffer[screen_y * 256 + screen_x] = RGB565::new_with_raw_value(dot_col);
            return;
        }

        let (r, g, b) = rgb565_to_parts(dot_col);

        let brightness = (self.registers.screen_brightness.get() as f32) / 15.0;

        let r = ((r as f32) * brightness) as u16;
        let g = ((g as f32) * brightness) as u16;
        let b = ((b as f32) * brightness) as u16;

        let dot_col = rgb565_from_parts(r, g, b);

        frame_buffer[screen_y * 256 + screen_x] = RGB565::new_with_raw_value(dot_col);
    }

    /// Gets the color of the first visible sprite on the screen.
    fn sprite_col(&mut self, screen_x: usize, screen_y: usize) -> ColorData {
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

                let sprite_col = if sprite.flip_x { sprite.width - sprite_col - 1 } else { sprite_col };
                let sprite_row = if sprite.flip_y { sprite.height - sprite_row - 1 } else { sprite_row };

                let (tile_x, tile_col) = (sprite_col / 8, sprite_col % 8);
                let (tile_y, tile_row) = (sprite_row / 8, sprite_row % 8);

                let chr_idx = (tile_y << 4) + tile_x;

                let obj_table_base_addr = if sprite.use_second_obj_table {
                    self.name_base_addr() + ((self.name_secondary_select() as u16) << 12)
                } else {
                    self.name_base_addr()
                };

                let spr_tile_base_addr = obj_table_base_addr + ((sprite.tile_idx as u16) << 4);
                let spr_tile_addr = spr_tile_base_addr + ((chr_idx as u16) << 4);
                let spr_tile_row_addr = spr_tile_addr + tile_row as u16;

                let bp01 = self.vram_read(spr_tile_row_addr + 0);
                let bp23 = self.vram_read(spr_tile_row_addr + 8);

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
                            raw_color: 0,
                            priority: sprite.priority,
                            transparent: true,
                        };
                    }

                    continue;
                }

                let cgram_addr = 0x80 | (sprite.palette << 4) | pal_idx;

                let spr_col = self.registers.cgram[cgram_addr as usize].get();

                return ColorData {
                    raw_color: spr_col,
                    priority: sprite.priority,
                    transparent: false,
                };
            }
        }

        // No sprites on this dot, return a transparent color
        ColorData {
            raw_color: self.transparent_color(),
            priority: 0,
            transparent: true,
        }
    }

    /// Compute the color of this dot, combining all bg layers and object color
    /// data. Computes only as many layers as it needs to before returning the
    /// color of the dot.
    fn bg_mode0_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> u16 {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x20;
        const BG3_CGRAM_BASE_ADDR: u8 = 0x40;
        const BG4_CGRAM_BASE_ADDR: u8 = 0x60;

        let col_win_en = self.col_win_active_signal(screen_x);
        let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x);
        let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x);
        let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x);
        let (bg3_win_main, bg3_win_sub) = self.bg3_win_active_signals(screen_x);
        let (bg4_win_main, bg4_win_sub) = self.bg4_win_active_signals(screen_x);

        let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data()
        };
        let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(spr_col); // Obj col should not be used past this point

        let bg1_col = self.bg_col(
            screen_x, screen_y, 
            ColorLayer::Bg1, ColorDepth::Bpp2,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
            bg1_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
            bg1_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg1_col); // Bg1 col should not be used past this point

        let bg2_col = self.bg_col(
            screen_x, screen_y, 
            ColorLayer::Bg2, ColorDepth::Bpp2,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
            bg2_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
            bg2_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg2_col); // Bg2 col should not be used past this point

        let bg3_col = self.bg_col(
            screen_x, screen_y, 
            ColorLayer::Bg3, ColorDepth::Bpp2,
            BG3_CGRAM_BASE_ADDR
        );
        let bg3_main_col = if self.bg3_main_enabled() && !bg3_win_main {
            bg3_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg3_sub_col = if self.bg3_sub_enabled() && !bg3_win_sub {
            bg3_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg3_col); // Bg3 col should not be used past this point

        let bg4_col = self.bg_col(
            screen_x, screen_y, 
            ColorLayer::Bg4, ColorDepth::Bpp2,
            BG4_CGRAM_BASE_ADDR
        );
        let bg4_main_col = if self.bg4_main_enabled() && !bg4_win_main {
            bg4_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg4_sub_col = if self.bg4_sub_enabled() && !bg4_win_sub {
            bg4_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg4_col); // Bg3 col should not be used past this point

        let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.raw_color, ColorLayer::Bg1)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.raw_color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.raw_color, ColorLayer::Bg1)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.raw_color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
            (bg3_main_col.raw_color, ColorLayer::Bg3)
        } else if bg4_main_col.priority != 0 && !bg4_main_col.transparent {
            (bg4_main_col.raw_color, ColorLayer::Bg4)
        } else if !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if !bg3_main_col.transparent {
            (bg3_main_col.raw_color, ColorLayer::Bg3)
        } else if !bg4_main_col.transparent {
            (bg4_main_col.raw_color, ColorLayer::Bg4)
        } else {
            (self.transparent_color(), ColorLayer::Back) // Main screen color is black if all layers are transparent
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => self.bg1_cmath_enabled(),
            ColorLayer::Bg2 => self.bg2_cmath_enabled(),
            ColorLayer::Bg3 => self.bg3_cmath_enabled(),
            ColorLayer::Bg4 => self.bg4_cmath_enabled(),
            ColorLayer::Obj => self.obj_cmath_enabled(),
            ColorLayer::Back => self.back_cmath_enabled(),
            _ => unreachable!(), // No other layers considered in Mode 0
        };

        if !cmath_en {
            return main_col;
        }

        let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.raw_color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.raw_color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.raw_color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.raw_color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
            bg3_sub_col.raw_color
        } else if bg4_sub_col.priority != 0 && !bg4_sub_col.transparent {
            bg4_sub_col.raw_color
        } else if !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if !bg3_sub_col.transparent {
            bg3_sub_col.raw_color
        } else if !bg4_sub_col.transparent {
            bg4_sub_col.raw_color
        } else {
            self.fixed_color() // sub screen color is black if all layers are transparent
        };

        let main_col = match self.col_win_main_region() {
            WindowColorRegion::Nowhere => main_col,
            WindowColorRegion::Outside => if col_win_en { main_col } else { 0 },
            WindowColorRegion::Inside => if col_win_en { 0 } else { main_col },
            WindowColorRegion::Everywhere => { 0 }
        };
        let sub_col = match self.col_win_sub_region() {
            WindowColorRegion::Nowhere => sub_col,
            WindowColorRegion::Outside => if col_win_en { sub_col } else { self.fixed_color() },
            WindowColorRegion::Inside => if col_win_en { self.fixed_color() } else { sub_col },
            WindowColorRegion::Everywhere => { self.fixed_color() }
        };

        self.apply_cmath(main_col, sub_col)
    }
    /// Computes the color of the dot on the screen using the Mode 1 process.
    /// Mode 1 used layers Bg1 at 4bpp, Bg2 at 4bpp, Bg3 at 2bpp, and Obj.
    /// It is able to use the features: Mosaic, Scroll, Interlace,
    /// 8x8 or 16x16 Tiles, Windowing, and Color Math.
    fn bg_mode1_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> u16 {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG3_CGRAM_BASE_ADDR: u8 = 0x00;

        let col_win_en = self.col_win_active_signal(screen_x);
        let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x);
        let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x);
        let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x);
        let (bg3_win_main, bg3_win_sub) = self.bg3_win_active_signals(screen_x);

        let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data()
        };
        let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(spr_col); // Obj col should not be used past this point

        let bg1_col = self.bg_col(
            screen_x, screen_y,
            ColorLayer::Bg1, ColorDepth::Bpp4,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
            bg1_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
            bg1_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg1_col); // Bg1 col should not be used past this point

        let bg2_col = self.bg_col(
            screen_x, screen_y,
            ColorLayer::Bg2, ColorDepth::Bpp4,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
            bg2_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
            bg2_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg2_col); // Bg2 col should not be used past this point

        let bg3_col = self.bg_col(
            screen_x, screen_y,
            ColorLayer::Bg3, ColorDepth::Bpp2,
            BG3_CGRAM_BASE_ADDR
        );
        let bg3_main_col = if self.bg3_main_enabled() && !bg3_win_main {
            bg3_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg3_sub_col = if self.bg3_sub_enabled() && !bg3_win_sub {
            bg3_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg3_col); // Bg3 col should not be used past this point

        let (main_col, main_layer) = if self.bg3_mode1_priority() && bg3_main_col.priority != 0 && !bg3_main_col.transparent {
            (bg3_main_col.raw_color, ColorLayer::Bg3)
        } else if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.raw_color, ColorLayer::Bg1)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.raw_color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.raw_color, ColorLayer::Bg1)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.raw_color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if bg3_main_col.priority != 0 && !bg3_main_col.transparent {
            (bg3_main_col.raw_color, ColorLayer::Bg3)
        } else if !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if !bg3_main_col.transparent {
            (bg3_main_col.raw_color, ColorLayer::Bg3)
        } else {
            (self.transparent_color(), ColorLayer::Back) // Main screen color is black if all layers are transparent
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => self.bg1_cmath_enabled(),
            ColorLayer::Bg2 => self.bg2_cmath_enabled(),
            ColorLayer::Bg3 => self.bg3_cmath_enabled(),
            ColorLayer::Obj => self.obj_cmath_enabled(),
            ColorLayer::Back => self.back_cmath_enabled(),
            _ => unreachable!(), // No other layers considered in Mode 1
        };

        if !cmath_en {
            return main_col;
        }

        let sub_col = if self.sub_color_fixed() {
            self.fixed_color()
        } else if self.bg3_mode1_priority() && bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
            bg3_sub_col.raw_color
        } else if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.raw_color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.raw_color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.raw_color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.raw_color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if bg3_sub_col.priority != 0 && !bg3_sub_col.transparent {
            bg3_sub_col.raw_color
        } else if !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if !bg3_sub_col.transparent {
            bg3_sub_col.raw_color
        } else {
            self.fixed_color() // Sub screen is fixed color if all layers are transparent
        };

        let main_col = match self.col_win_main_region() {
            WindowColorRegion::Nowhere => main_col,
            WindowColorRegion::Outside => if col_win_en { main_col } else { 0 },
            WindowColorRegion::Inside => if col_win_en { 0 } else { main_col },
            WindowColorRegion::Everywhere => { 0 }
        };
        let sub_col = match self.col_win_sub_region() {
            WindowColorRegion::Nowhere => sub_col,
            WindowColorRegion::Outside => if col_win_en { sub_col } else { self.transparent_color() },
            WindowColorRegion::Inside => if col_win_en { self.transparent_color() } else { sub_col },
            WindowColorRegion::Everywhere => { self.transparent_color() }
        };

        let sub_col = if self.sub_color_fixed() {
            self.fixed_color()
        } else {
            sub_col
        };

        self.apply_cmath(main_col, sub_col)
    }

    // fn bg_mode2_dot(&mut self, frame_buffer: &mut [ORGB1555]) {

    // }

    // fn bg_mode3_dot(&mut self, frame_buffer: &mut [ORGB1555]) {

    // }

    // fn bg_mode4_dot(&mut self, frame_buffer: &mut [ORGB1555]) {

    // }

    fn bg_mode5_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: ColorData) -> u16 {
        const BG1_CGRAM_BASE_ADDR: u8 = 0x00;
        const BG2_CGRAM_BASE_ADDR: u8 = 0x00;

        let col_win_en = self.col_win_active_signal(screen_x);
        let (obj_win_main, obj_win_sub) = self.obj_win_active_signals(screen_x);
        let (bg1_win_main, bg1_win_sub) = self.bg1_win_active_signals(screen_x);
        let (bg2_win_main, bg2_win_sub) = self.bg2_win_active_signals(screen_x);

        let spr_main_col = if self.obj_main_enabled() && !obj_win_main {
            spr_col.clone()
        } else {
            self.transparent_color_data()
        };
        let spr_sub_col = if self.obj_sub_enabled() && !obj_win_sub {
            spr_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(spr_col); // Obj col should not be used past this point

        let bg1_col = self.bg_col(
            screen_x, screen_y,
            ColorLayer::Bg1, ColorDepth::Bpp4,
            BG1_CGRAM_BASE_ADDR
        );
        let bg1_main_col = if self.bg1_main_enabled() && !bg1_win_main {
            bg1_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg1_sub_col = if self.bg1_sub_enabled() && !bg1_win_sub {
            bg1_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg1_col); // Bg1 col should not be used past this point

        let bg2_col = self.bg_col(
            screen_x, screen_y,
            ColorLayer::Bg2, ColorDepth::Bpp4,
            BG2_CGRAM_BASE_ADDR
        );
        let bg2_main_col = if self.bg2_main_enabled() && !bg2_win_main {
            bg2_col.clone()
        } else {
            self.transparent_color_data()
        };
        let bg2_sub_col = if self.bg2_sub_enabled() && !bg2_win_sub {
            bg2_col.clone()
        } else {
            self.transparent_color_data()
        };
        drop(bg2_col); // Bg2 col should not be used past this point

        let (main_col, main_layer) = if spr_main_col.priority == 3 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if bg1_main_col.priority != 0 && !bg1_main_col.transparent {
            (bg1_main_col.raw_color, ColorLayer::Bg1)
        } else if spr_main_col.priority == 2 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if bg2_main_col.priority != 0 && !bg2_main_col.transparent {
            (bg2_main_col.raw_color, ColorLayer::Bg2)
        } else if spr_main_col.priority == 1 && !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if !bg1_main_col.transparent {
            (bg1_main_col.raw_color, ColorLayer::Bg1)
        } else if !spr_main_col.transparent {
            (spr_main_col.raw_color, ColorLayer::Obj)
        } else if !bg2_main_col.transparent {
            (bg2_main_col.raw_color, ColorLayer::Bg2)
        } else {
            (self.transparent_color(), ColorLayer::Back) // Main screen color is black if all layers are transparent
        };

        let cmath_en = match main_layer {
            ColorLayer::Bg1 => self.bg1_cmath_enabled(),
            ColorLayer::Bg2 => self.bg2_cmath_enabled(),
            ColorLayer::Obj => self.obj_cmath_enabled(),
            ColorLayer::Back => self.back_cmath_enabled(),
            _ => unreachable!(), // No other layers considered in Mode 5
        };

        if !cmath_en {
            return main_col;
        }

        let sub_col = if spr_sub_col.priority == 3 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if bg1_sub_col.priority != 0 && !bg1_sub_col.transparent {
            bg1_sub_col.raw_color
        } else if spr_sub_col.priority == 2 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if bg2_sub_col.priority != 0 && !bg2_sub_col.transparent {
            bg2_sub_col.raw_color
        } else if spr_sub_col.priority == 1 && !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if !bg1_sub_col.transparent {
            bg1_sub_col.raw_color
        } else if !spr_sub_col.transparent {
            spr_sub_col.raw_color
        } else if !bg2_sub_col.transparent {
            bg2_sub_col.raw_color
        } else {
            self.fixed_color() // Sub screen color is fixed color if all layers are transparent
        };

        let main_col = match self.col_win_main_region() {
            WindowColorRegion::Nowhere => main_col,
            WindowColorRegion::Outside => if col_win_en { main_col } else { 0 },
            WindowColorRegion::Inside => if col_win_en { 0 } else { main_col },
            WindowColorRegion::Everywhere => { 0 }
        };
        let sub_col = match self.col_win_sub_region() {
            WindowColorRegion::Nowhere => sub_col,
            WindowColorRegion::Outside => if col_win_en { sub_col } else { self.fixed_color() },
            WindowColorRegion::Inside => if col_win_en { self.fixed_color() } else { sub_col },
            WindowColorRegion::Everywhere => { self.fixed_color() }
        };

        self.apply_cmath(main_col, sub_col)
    }

    // fn bg_mode6_dot(&mut self, frame_buffer: &mut [ORGB1555]) {

    // }

    // fn bg_mode7_dot(&mut self, frame_buffer: &mut [ORGB1555]) {

    // }

    fn bg_col(&self, screen_x: usize, screen_y: usize, 
        bg_layer: ColorLayer, color_depth: ColorDepth, 
        bg_cgram_base_addr: u8) -> ColorData {

        let bg_chr_base_addr = match bg_layer {
            ColorLayer::Bg1 => self.bg1_chr_base_addr(),
            ColorLayer::Bg2 => self.bg2_chr_base_addr(),
            ColorLayer::Bg3 => self.bg3_chr_base_addr(),
            ColorLayer::Bg4 => self.bg4_chr_base_addr(),

            _ => unreachable!("Should only be called for bg layers")
        };
        
        let tile_data = self.bg_tile_idx(screen_x, screen_y, bg_layer);

        // if screen_y == 111 && screen_x == 240 && bg_layer == ColorLayer::Bg1 {
            // println!("({screen_x}, {screen_y}): addr: ${:04X}, row: {}, col: {}, size: {:?}",
            //     tile_data.tile_addr,
            //     tile_data.tile_row,
            //     tile_data.tile_col,
            //     tile_data.tile_size,
            // );

            // if tile_data.tile_addr == 0x2CB5 {
            //     let mut vram_clone = Vec::new();

            //     for w in self.registers.vram.iter() {
            //         vram_clone.push(w.get());
            //     }

            //     crate::tools::hexdump::hexdump16_to_file(&vram_clone, 0, "vram_dump.txt");

            //     std::process::exit(0);
            // }
        // }

        let col = match color_depth {
            ColorDepth::Bpp2 => self.bg_col_2bpp(tile_data, bg_chr_base_addr, bg_cgram_base_addr),
            ColorDepth::Bpp4 =>  self.bg_col_4bpp(tile_data, bg_chr_base_addr, bg_cgram_base_addr),
            _ => todo!("8bpp and direct color not implemented yet"),
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
        //     _ => self.transparent_color_data()
        // };

        col
    }

    fn bg_tile_idx(&self, screen_x: usize, screen_y: usize, bg_layer: ColorLayer) -> TileData {
        let (scroll_x, scroll_y, 
            tilemap_cnt_x, tilemap_cnt_y, 
            tile_size, tilemap_base_addr) = match bg_layer {

            ColorLayer::Bg1 => (
                self.bg1_m7_x_offset(), self.bg1_m7_y_offset(),
                self.bg1_tilemap_count_x(), self.bg1_tilemap_count_y(),
                self.bg1_tile_size(), self.bg1_vram_base_addr(),
            ),

            ColorLayer::Bg2 => (
                self.bg2_x_offset(), self.bg2_y_offset(),
                self.bg2_tilemap_count_x(), self.bg2_tilemap_count_y(),
                self.bg2_tile_size(), self.bg2_vram_base_addr(),
            ),

            ColorLayer::Bg3 => (
                self.bg3_x_offset(), self.bg3_y_offset(),
                self.bg3_tilemap_count_x(), self.bg3_tilemap_count_y(),
                self.bg3_tile_size(), self.bg3_vram_base_addr(),
            ),

            ColorLayer::Bg4 => (
                self.bg4_x_offset(), self.bg4_y_offset(),
                self.bg4_tilemap_count_x(), self.bg4_tilemap_count_y(),
                self.bg4_tile_size(), self.bg4_vram_base_addr(),
            ),

            _ => unreachable!("Should only be called for bg layers.")
        };

        let shifted_x = (screen_x as u16) + (scroll_x as u16);
        let shifted_y = (screen_y as u16) + (scroll_y as u16);

        let tilemap_offset = match (tilemap_cnt_x, tilemap_cnt_y) {
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

        let tile_idx = match tile_size {
            TileSize::Size8x8 => ((y >> 3) << 5) | (x >> 3),
            TileSize::Size16x16 => (y & 0xF0) | (x >> 4),
        };

        let (tile_col, tile_row) = match tile_size {
            TileSize::Size8x8 => (x & 7, y & 7),
            TileSize::Size16x16 => (x & 0xF, y & 0xF),
        };

        TileData {
            tile_addr: tilemap_base_addr + tilemap_offset + tile_idx,
            tile_row: tile_row as u8,
            tile_col: tile_col as u8,
            tile_size
        }
    }

    fn fetch_chr_data(&self, tile_data: TileData) -> ChrData {
        let tile_word = self.vram_read(tile_data.tile_addr);

        let (tile_height, tile_width) = match tile_data.tile_size {
            TileSize::Size8x8 => (8,8),
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

    fn bg_col_2bpp(&self, tile_data: TileData, bg_chr_base_addr: u16, bg_cgram_base_addr: u8) -> ColorData {        
        let chr_data = self.fetch_chr_data(tile_data);

        let tile_chr_addr = bg_chr_base_addr + (chr_data.chr_idx << 3) + chr_data.chr_row as u16;

        let bp01 = self.vram_read(tile_chr_addr);

        let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
        let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;

        let pal_idx = (b1 << 1) | b0;
        
        let cgram_addr = bg_cgram_base_addr | (chr_data.chr_pal << 2) | pal_idx;

        let raw_color = if pal_idx == 0 {
            self.transparent_color()
        } else {
            self.registers.cgram[cgram_addr as usize].get()
        };

        ColorData {
            raw_color,
            priority: chr_data.chr_priority,
            transparent: pal_idx == 0,
        }
    }

    fn bg_col_4bpp(&self, tile_data: TileData, bg_chr_base_addr: u16, bg_cgram_base_addr: u8) -> ColorData {
        let chr_data = self.fetch_chr_data(tile_data);

        let tile_chr_addr = bg_chr_base_addr + (chr_data.chr_idx << 4) + chr_data.chr_row as u16;

        let bp01 = self.vram_read(tile_chr_addr + 0);
        let bp23 = self.vram_read(tile_chr_addr + 8);

        let b0 = ((bp01 >> (7-chr_data.chr_col)) & 1) as u8;
        let b1 = ((bp01 >> (15-chr_data.chr_col)) & 1) as u8;
        let b2 = ((bp23 >> (7-chr_data.chr_col)) & 1) as u8;
        let b3 = ((bp23 >> (15-chr_data.chr_col)) & 1) as u8;

        let pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;
        
        let cgram_addr = bg_cgram_base_addr | (chr_data.chr_pal << 4) | pal_idx;

        let raw_color = if pal_idx == 0 {
            self.transparent_color()
        } else {
            self.registers.cgram[cgram_addr as usize].get()
        };

        ColorData {
            raw_color,
            priority: chr_data.chr_priority,
            transparent: pal_idx == 0,
        }
    }

    fn apply_cmath(&self, main_col: u16, sub_col: u16) -> u16 {
        let (main_r, main_g, main_b) = rgb565_to_parts(main_col);
        let (sub_r, sub_g, sub_b) = rgb565_to_parts(sub_col);

        let (r,g,b) = match self.cmath_operator() {
            CMathOperator::Add => (main_r + sub_r, main_g + sub_g, main_b + sub_b),
            CMathOperator::Subtract => (main_r - sub_r, main_g - sub_g, main_b - sub_b),
        };

        let (r,g,b) = if self.cmath_half() {
            (r >> 1, g >> 1, b >> 1)
        } else {
            (r, g, b)
        };

        // Negative values clamped to 0, positive values clamped to 31
        let r = if r.bit_en(15) { 0 } else { r & 0x1F };
        let g = if g.bit_en(15) { 0 } else { g & 0x1F };
        let b = if b.bit_en(15) { 0 } else { b & 0x1F };

        rgb565_from_parts(r, g, b)
    }
}

// Getters & Setters for registers
impl Ppu5C7x {
    fn win_active_signal(&self, screen_x: usize, layer_w1_en: bool, layer_w2_en: bool,
        layer_w1_inv: bool, layer_w2_inv: bool, win_logic: WindowLogic) -> bool {

        let w1_left = self.w1_left_pos() as usize;
        let w1_right = self.w1_right_pos() as usize;
        let w2_left = self.w2_left_pos() as usize;
        let w2_right = self.w2_right_pos() as usize;

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

    fn bg1_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
        let win_en = if self.bg1_win_main_enabled() || self.bg1_win_sub_enabled() {
            self.win_active_signal(screen_x,
                self.bg1_w1_enabled(),
                self.bg1_w2_enabled(),
                self.bg1_w1_inverted(),
                self.bg1_w2_inverted(),
                self.bg1_win_logic()
            )
        } else {
            false
        };

        let bg1_win_main_en = win_en && self.bg1_win_main_enabled();
        let bg1_win_sub_en = win_en && self.bg1_win_sub_enabled();

        (bg1_win_main_en, bg1_win_sub_en)
    }

    fn bg2_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
        let win_en = if self.bg2_win_main_enabled() || self.bg2_win_sub_enabled() {
            self.win_active_signal(screen_x,
                self.bg2_w1_enabled(),
                self.bg2_w2_enabled(),
                self.bg2_w1_inverted(),
                self.bg2_w2_inverted(),
                self.bg2_win_logic()
            )
        } else {
            false
        };

        let bg2_win_main_en = win_en && self.bg2_win_main_enabled();
        let bg2_win_sub_en = win_en && self.bg2_win_sub_enabled();

        (bg2_win_main_en, bg2_win_sub_en)
    }

    fn bg3_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
        let win_en = if self.bg3_win_main_enabled() || self.bg3_win_sub_enabled() {
            self.win_active_signal(screen_x,
                self.bg3_w1_enabled(),
                self.bg3_w2_enabled(),
                self.bg3_w1_inverted(),
                self.bg3_w2_inverted(),
                self.bg3_win_logic()
            )
        } else {
            false
        };

        let bg3_win_main_en = win_en && self.bg3_win_main_enabled();
        let bg3_win_sub_en = win_en && self.bg3_win_sub_enabled();

        (bg3_win_main_en, bg3_win_sub_en)
    }

    fn bg4_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
        let win_en = if self.bg4_win_main_enabled() || self.bg4_win_sub_enabled() {
            self.win_active_signal(screen_x,
                self.bg4_w1_enabled(),
                self.bg4_w2_enabled(),
                self.bg4_w1_inverted(),
                self.bg4_w2_inverted(),
                self.bg4_win_logic()
            )
        } else {
            false
        };

        let bg4_win_main_en = win_en && self.bg4_win_main_enabled();
        let bg4_win_sub_en = win_en && self.bg4_win_sub_enabled();

        (bg4_win_main_en, bg4_win_sub_en)
    }

    fn obj_win_active_signals(&self, screen_x: usize) -> (bool, bool) {
        let win_en = if self.obj_win_main_enabled() || self.obj_win_sub_enabled() {
            self.win_active_signal(screen_x,
                self.obj_w1_enabled(),
                self.obj_w2_enabled(),
                self.obj_w1_inverted(),
                self.obj_w2_inverted(),
                self.obj_win_logic()
            )
        } else {
            false
        };

        let obj_win_main_en = win_en && self.obj_win_main_enabled();
        let obj_win_sub_en = win_en && self.obj_win_sub_enabled();

        (obj_win_main_en, obj_win_sub_en)
    }

    fn col_win_active_signal(&self, screen_x: usize) -> bool {
        let win_en = self.win_active_signal(screen_x,
            self.col_w1_enabled(),
            self.col_w2_enabled(),
            self.col_w1_inverted(),
            self.col_w2_inverted(),
            self.col_win_logic()
        );

        win_en
    }

    fn bg1_vram_base_addr(&self) -> u16 { (self.registers.bg1_vram_addr.get() as u16) << 10 }
    fn bg2_vram_base_addr(&self) -> u16 { (self.registers.bg2_vram_addr.get() as u16) << 10 }
    fn bg3_vram_base_addr(&self) -> u16 { (self.registers.bg3_vram_addr.get() as u16) << 10 }
    fn bg4_vram_base_addr(&self) -> u16 { (self.registers.bg4_vram_addr.get() as u16) << 10 }

    fn bg1_chr_base_addr(&self) -> u16 { (self.registers.bg1_chr_base_addr.get() as u16) << 12 }    
    fn bg2_chr_base_addr(&self) -> u16 { (self.registers.bg2_chr_base_addr.get() as u16) << 12 }
    fn bg3_chr_base_addr(&self) -> u16 { (self.registers.bg3_chr_base_addr.get() as u16) << 12 }
    fn bg4_chr_base_addr(&self) -> u16 { (self.registers.bg4_chr_base_addr.get() as u16) << 12 }

    fn in_fblank(&self) -> bool { self.registers.in_fblank.get() }
    fn in_hblank(&self) -> bool { self.registers.in_hblank.get() }
    fn in_vblank(&self) -> bool { self.registers.in_vblank.get() }
    
    fn bg4_tile_size(&self) -> TileSize { self.registers.bg4_char_size.get() }
    fn bg3_tile_size(&self) -> TileSize { self.registers.bg3_char_size.get() }
    fn bg2_tile_size(&self) -> TileSize { self.registers.bg2_char_size.get() }
    fn bg1_tile_size(&self) -> TileSize { self.registers.bg1_char_size.get() }
    fn obj_sprite_size(&self) -> ObjectSizeSelect { self.registers.obj_sprite_size.get() }

    fn bg_mode(&self) -> BgMode { self.registers.bg_mode.get() }
    fn fixed_color(&self) -> u16 { self.registers.fixed_color.get() }
    fn bg3_mode1_priority(&self) -> bool { self.registers.bg3_mode1_priority.get() }

    fn name_base_addr(&self) -> u16 { (self.registers.name_base_addr.get() as u16) << 13 }
    fn name_secondary_select(&self) -> u8 { self.registers.name_secondary_select.get() }
    
    fn bg1_m7_x_offset(&self) -> u16 { self.registers.bg1_m7_x_offset.get() }
    fn bg1_m7_y_offset(&self) -> u16 { self.registers.bg1_m7_y_offset.get() }
    fn bg2_x_offset(&self) -> u16 { self.registers.bg2_x_offset.get() }
    fn bg2_y_offset(&self) -> u16 { self.registers.bg2_y_offset.get() }
    fn bg3_x_offset(&self) -> u16 { self.registers.bg3_x_offset.get() }
    fn bg3_y_offset(&self) -> u16 { self.registers.bg3_y_offset.get() }
    fn bg4_x_offset(&self) -> u16 { self.registers.bg4_x_offset.get() }
    fn bg4_y_offset(&self) -> u16 { self.registers.bg4_y_offset.get() }

    // fn screen_brightness(&self) -> u8 { self.registers.screen_brightness.get() }
    // fn oam_addr(&self) -> u16 { self.registers.oam_addr.get() }
    // fn priority_rotation(&self) -> bool { self.registers.priority_rotation.get() }
    // fn oam_data_latch(&self) -> u8 { self.registers.oam_data_latch.get() }
    // fn bg_mode(&self) -> BgMode { self.registers.bg_mode.get() }
    // fn mosaic_size(&self) -> u8 { self.registers.mosaic_size.get() }
    // fn bg4_mosaic(&self) -> bool { self.registers.bg4_mosaic.get() }
    // fn bg3_mosaic(&self) -> bool { self.registers.bg3_mosaic.get() }
    // fn bg2_mosaic(&self) -> bool { self.registers.bg2_mosaic.get() }
    // fn bg1_mosaic(&self) -> bool { self.registers.bg1_mosaic.get() }
    // fn m7_latch(&self) -> u8 { self.registers.m7_latch.get() }
    // fn bg_offset_latch(&self) -> u8 { self.registers.bg_offset_latch.get() }
    // fn bg_offset_x_latch(&self) -> u8 { self.registers.bg_offset_x_latch.get() }
    // fn vram_addr_inc_mode(&self) -> VramIncMode { self.registers.vram_addr_inc_mode.get() }
    // fn addr_remap_mode(&self) -> AddressRemapping { self.registers.addr_remap_mode.get() }
    // fn addr_inc_size(&self) -> IncrSize { self.registers.addr_inc_size.get() }
    // fn vram_addr(&self) -> u16 { self.registers.vram_addr.get() }
    // fn vram_data(&self) -> u16 { self.registers.vram_data.get() }
    // fn m7_tilemap_repeat(&self) -> bool { self.registers.m7_tilemap_repeat.get() }
    // fn m7_fill_mode(&self) -> M7FillMode { self.registers.m7_fill_mode.get() }
    // fn m7_flip_bg_y(&self) -> bool { self.registers.m7_flip_bg_y.get() }
    // fn m7_flip_bg_x(&self) -> bool { self.registers.m7_flip_bg_x.get() }
    // fn m7_matrix_a(&self) -> u16 { self.registers.m7_matrix_a.get() }
    // fn m7_matrix_b(&self) -> u16 { self.registers.m7_matrix_b.get() }
    // fn m7_matrix_c(&self) -> u16 { self.registers.m7_matrix_c.get() }
    // fn m7_matrix_d(&self) -> u16 { self.registers.m7_matrix_d.get() }
    // fn m7_center_x(&self) -> u16 { self.registers.m7_center_x.get() }
    // fn m7_center_y(&self) -> u16 { self.registers.m7_center_y.get() }
    // fn cgram_toggle(&self) -> ToggleState { self.registers.cgram_toggle.get() }
    // fn cgram_addr(&self) -> u8 { self.registers.cgram_addr.get() }


    fn bg2_w2_enabled(&self) -> bool { self.registers.bg2_w2_enabled.get() }
    fn bg2_w2_inverted(&self) -> bool { self.registers.bg2_w2_inverted.get() }
    fn bg2_w1_enabled(&self) -> bool { self.registers.bg2_w1_enabled.get() }
    fn bg2_w1_inverted(&self) -> bool { self.registers.bg2_w1_inverted.get() }
    fn bg1_w2_enabled(&self) -> bool { self.registers.bg1_w2_enabled.get() }
    fn bg1_w2_inverted(&self) -> bool { self.registers.bg1_w2_inverted.get() }
    fn bg1_w1_enabled(&self) -> bool { self.registers.bg1_w1_enabled.get() }
    fn bg1_w1_inverted(&self) -> bool { self.registers.bg1_w1_inverted.get() }
    fn bg4_w2_enabled(&self) -> bool { self.registers.bg4_w2_enabled.get() }
    fn bg4_w2_inverted(&self) -> bool { self.registers.bg4_w2_inverted.get() }
    fn bg4_w1_enabled(&self) -> bool { self.registers.bg4_w1_enabled.get() }
    fn bg4_w1_inverted(&self) -> bool { self.registers.bg4_w1_inverted.get() }
    fn bg3_w2_enabled(&self) -> bool { self.registers.bg3_w2_enabled.get() }
    fn bg3_w2_inverted(&self) -> bool { self.registers.bg3_w2_inverted.get() }
    fn bg3_w1_enabled(&self) -> bool { self.registers.bg3_w1_enabled.get() }
    fn bg3_w1_inverted(&self) -> bool { self.registers.bg3_w1_inverted.get() }
    fn col_w2_enabled(&self) -> bool { self.registers.col_w2_enabled.get() }
    fn col_w2_inverted(&self) -> bool { self.registers.col_w2_inverted.get() }
    fn col_w1_enabled(&self) -> bool { self.registers.col_w1_enabled.get() }
    fn col_w1_inverted(&self) -> bool { self.registers.col_w1_inverted.get() }
    fn obj_w2_enabled(&self) -> bool { self.registers.obj_w2_enabled.get() }
    fn obj_w2_inverted(&self) -> bool { self.registers.obj_w2_inverted.get() }
    fn obj_w1_enabled(&self) -> bool { self.registers.obj_w1_enabled.get() }
    fn obj_w1_inverted(&self) -> bool { self.registers.obj_w1_inverted.get() }
    fn w1_left_pos(&self) -> u8 { self.registers.w1_left_pos.get() }
    fn w1_right_pos(&self) -> u8 { self.registers.w1_right_pos.get() }
    fn w2_left_pos(&self) -> u8 { self.registers.w2_left_pos.get() }
    fn w2_right_pos(&self) -> u8 { self.registers.w2_right_pos.get() }
    fn bg4_win_logic(&self) -> WindowLogic { self.registers.bg4_win_logic.get() }
    fn bg3_win_logic(&self) -> WindowLogic { self.registers.bg3_win_logic.get() }
    fn bg2_win_logic(&self) -> WindowLogic { self.registers.bg2_win_logic.get() }
    fn bg1_win_logic(&self) -> WindowLogic { self.registers.bg1_win_logic.get() }
    fn obj_win_logic(&self) -> WindowLogic { self.registers.obj_win_logic.get() }
    fn col_win_logic(&self) -> WindowLogic { self.registers.col_win_logic.get() }
    fn obj_main_enabled(&self) -> bool { self.registers.obj_main_enabled.get() }
    fn bg4_main_enabled(&self) -> bool { self.registers.bg4_main_enabled.get() }
    fn bg3_main_enabled(&self) -> bool { self.registers.bg3_main_enabled.get() }
    fn bg2_main_enabled(&self) -> bool { self.registers.bg2_main_enabled.get() }
    fn bg1_main_enabled(&self) -> bool { self.registers.bg1_main_enabled.get() }
    fn obj_sub_enabled(&self) -> bool { self.registers.obj_sub_enabled.get() }
    fn bg4_sub_enabled(&self) -> bool { self.registers.bg4_sub_enabled.get() }
    fn bg3_sub_enabled(&self) -> bool { self.registers.bg3_sub_enabled.get() }
    fn bg2_sub_enabled(&self) -> bool { self.registers.bg2_sub_enabled.get() }
    fn bg1_sub_enabled(&self) -> bool { self.registers.bg1_sub_enabled.get() }
    fn obj_win_main_enabled(&self) -> bool { self.registers.obj_win_main_enabled.get() }
    fn bg4_win_main_enabled(&self) -> bool { self.registers.bg4_win_main_enabled.get() }
    fn bg3_win_main_enabled(&self) -> bool { self.registers.bg3_win_main_enabled.get() }
    fn bg2_win_main_enabled(&self) -> bool { self.registers.bg2_win_main_enabled.get() }
    fn bg1_win_main_enabled(&self) -> bool { self.registers.bg1_win_main_enabled.get() }
    fn obj_win_sub_enabled(&self) -> bool { self.registers.obj_win_sub_enabled.get() }
    fn bg4_win_sub_enabled(&self) -> bool { self.registers.bg4_win_sub_enabled.get() }
    fn bg3_win_sub_enabled(&self) -> bool { self.registers.bg3_win_sub_enabled.get() }
    fn bg2_win_sub_enabled(&self) -> bool { self.registers.bg2_win_sub_enabled.get() }
    fn bg1_win_sub_enabled(&self) -> bool { self.registers.bg1_win_sub_enabled.get() }
    fn col_win_main_region(&self) -> WindowColorRegion { self.registers.col_win_main_region.get() }
    fn col_win_sub_region(&self) -> WindowColorRegion { self.registers.col_win_sub_region.get() }
    fn sub_color_fixed(&self) -> bool { self.registers.sub_color_fixed.get() }
    fn cmath_operator(&self) -> CMathOperator { self.registers.cmath_operator.get() }
    fn cmath_half(&self) -> bool { self.registers.cmath_half.get() }
    fn back_cmath_enabled(&self) -> bool { self.registers.back_cmath_enabled.get() }
    fn obj_cmath_enabled(&self) -> bool { self.registers.obj_cmath_enabled.get() }
    fn bg4_cmath_enabled(&self) -> bool { self.registers.bg4_cmath_enabled.get() }
    fn bg3_cmath_enabled(&self) -> bool { self.registers.bg3_cmath_enabled.get() }
    fn bg2_cmath_enabled(&self) -> bool { self.registers.bg2_cmath_enabled.get() }
    fn bg1_cmath_enabled(&self) -> bool { self.registers.bg1_cmath_enabled.get() }
    fn bg1_tilemap_count_y(&self) -> TilemapCount { self.registers.bg1_tilemap_count_y.get() }
    fn bg1_tilemap_count_x(&self) -> TilemapCount { self.registers.bg1_tilemap_count_x.get() }
    fn bg2_tilemap_count_y(&self) -> TilemapCount { self.registers.bg2_tilemap_count_y.get() }
    fn bg2_tilemap_count_x(&self) -> TilemapCount { self.registers.bg2_tilemap_count_x.get() }
    fn bg3_tilemap_count_y(&self) -> TilemapCount { self.registers.bg3_tilemap_count_y.get() }
    fn bg3_tilemap_count_x(&self) -> TilemapCount { self.registers.bg3_tilemap_count_x.get() }
    fn bg4_tilemap_count_y(&self) -> TilemapCount { self.registers.bg4_tilemap_count_y.get() }
    fn bg4_tilemap_count_x(&self) -> TilemapCount { self.registers.bg4_tilemap_count_x.get() }

    // fn ext_bg_enabled(&self) -> bool { self.registers.ext_bg_enabled.get() }
    // fn hi_res_enabled(&self) -> bool { self.registers.hi_res_enabled.get() }
    // fn overscan_enabled(&self) -> bool { self.registers.overscan_enabled.get() }
    // fn obj_interlace_enabled(&self) -> bool { self.registers.obj_interlace_enabled.get() }
    // fn screen_interlace_enabled(&self) -> bool { self.registers.screen_interlace_enabled.get() }

    fn vram_read(&self, address: u16) -> u16 { self.registers.vram[(address & 0x7FFF) as usize].get() }
}