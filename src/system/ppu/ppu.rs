use std::{cell::Cell, rc::Rc};

use libretro_rs::retro::pixel::format::XRGB8888;

use crate::log::SnemLogger;

const VBLANK_START_SCANLINE: u16 = 225;
const VBLANK_END_SCANLINE_NTSC: u16 = 261;
// const VBLANK_END_SCANLINE_PAL: u16 = 311;
// const VBLANK_INTERLACE_START_SCANLINE: u16 = 239;
const VISIBLE_SCANLINE_START_DOT: usize = 22;
const HBLANK_END_DOT: u16 = VISIBLE_SCANLINE_START_DOT as u16;
const HBLANK_START_DOT: u16 = 278;
const HBLANK_DISABLE_SCANLINE: u16 = VBLANK_START_SCANLINE-1;
const SCANLINE_END_DOT: u16 = 340;


trait GetBits {
    fn get_bit(self, bit: Self) -> Self;
    fn bit_en(self, bit: Self) -> bool;
}

impl GetBits for u8 {
    fn get_bit(self, bit: Self) -> Self { (self >> bit) & 1 }
    fn bit_en(self, bit: Self) -> bool { (self >> bit) & 1 != 0 }
}

trait SetBytes {
    fn set_hi(&self, data: u8);
    fn set_lo(&self, data: u8);
}

impl SetBytes for Cell<u16> {
    fn set_hi(&self, data: u8) {
        self.replace((self.get() & 0x00FF) | ((data as u16) << 8));
    }
    fn set_lo(&self, data: u8) {
        self.replace((self.get() & 0xFF00) | (data as u16));
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum ToggleState {
    #[default]
    LoByte,
    HiByte,
}

trait Togglable {
    /// Returns a bool reporting whether the latch state is high.
    fn is_high(&self) -> bool;
    /// Toggles the latch and returns a bool reporting whether the latch was high
    /// BEFORE the toggle.
    fn toggle(&self) -> bool;
    /// Sets the toggle state to low/0.
    fn set_lo(&self);
    /// Sets the toggle state to high/1.
    fn set_hi(&self);
}

impl Togglable for Cell<ToggleState> {
    fn is_high(&self) -> bool { self.get() == ToggleState::HiByte }
    fn toggle(&self) -> bool {
        self.replace(
            match self.get() {
                ToggleState::LoByte => ToggleState::HiByte,
                ToggleState::HiByte => ToggleState::LoByte,
            }
        ) == ToggleState::HiByte
    }
    fn set_lo(&self) { self.replace(ToggleState::LoByte); }
    fn set_hi(&self) { self.replace(ToggleState::HiByte); }
}

#[derive(Clone, Copy, Default, Debug)]
enum ObjectSizeSelect {
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
enum ObjectSize {
    Size8x8,
    Size16x16,
    Size32x32,
    Size64x64,
    Size16x32,
    Size32x64,
}

#[derive(Clone, Copy, Default, Debug)]
enum CharSize {
    #[default]
    Small,
    Large,
}

#[derive(Clone, Copy, Default, Debug)]
enum BgPriority {
    #[default]
    High,
    Low,
}

#[derive(Clone, Copy, Default, Debug)]
enum BgMode {
    #[default]
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Mode6,
    Mode7
}

#[derive(Clone, Copy, Default, Debug)]
enum TilemapCount {
    #[default]
    One,
    Two,
}

#[derive(Clone, Copy, Default, Debug)]
enum VramIncMode {
    LowByte,
    #[default]
    HighByte
}

#[derive(Clone, Copy, Default, Debug)]
enum AddressRemapping {
    #[default]
    None,
    ColDepth2,
    ColDepth4,
    ColDepth8,
}

#[derive(Clone, Copy, Default, Debug)]
enum IncrSize {
    #[default]
    Bytes2,
    Bytes64,
    Bytes256,
}

#[derive(Clone, Copy, Default, Debug)]
enum M7FillMode {
    #[default]
    Transparent,
    Character,
}

#[derive(Clone, Copy, Default, Debug)]
enum WindowLogic {
    #[default]
    Or,
    And,
    Xor,
    Xnor,
}

#[derive(Clone, Copy, Default, Debug)]
enum WindowColorRegion {
    #[default]
    Nowhere,
    Outside,
    Inside,
    Everywhere,
}

#[derive(Clone, Copy, Default, Debug)]
enum CMathAddend {
    #[default]
    Fixed,
    Subscreen,
}

#[derive(Clone, Copy, Default, Debug)]
enum DirectColorMode {
    #[default]
    Palette,
    Direct,
}

#[derive(Clone, Copy, Default, Debug)]
enum CMathOperator {
    #[default]
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

struct OamData([Cell<u8>; 0x220]);
struct VramData([Cell<u16>; 32 * 1024]); // 64 KB VRAM (32768 words)
struct CgRamData([Cell<u16>; 256]);

impl Default for OamData {
    fn default() -> Self {
        let arr: [Cell<u8>; 0x220] = vec![Cell::new(0); 0x220].try_into().expect("Failed to make OAM arr");
        OamData(arr)
    }
}

impl Default for VramData {
    fn default() -> Self {
        let arr: [Cell<u16>; 32 * 1024] = vec![Cell::new(0); 32 * 1024].try_into().expect("Failed to make VRAM arr");
        VramData(arr)
    }
}

impl Default for CgRamData {
    fn default() -> Self {
        let arr: [Cell<u16>; 256] = vec![Cell::new(0); 256].try_into().expect("Failed to make CGRAM arr");
        CgRamData(arr)
    }
}


#[derive(Default)]
pub struct PpuData {
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
    oam_data: Cell<u8>,
    oam_data_latch: Cell<u8>,

    // $2105    4321 PMMM    Write Only
    //       - Tilemap tile size (#)
    //       - BG3 priority (P)
    //       - BG mode (M)
    bg4_char_size: Cell<CharSize>,
    bg3_char_size: Cell<CharSize>,
    bg2_char_size: Cell<CharSize>,
    bg1_char_size: Cell<CharSize>,
    bg3_priority: Cell<BgPriority>,
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
    pub vram_addr: Cell<u16>,

    // $2118    LLLL LLLL
    // $2119    HHHH HHHH    Write x2 Only
    //       - VRAM data Low (L)
    //       - VRAM data High (H), Increments VMADD after write according to VMAIN setting.
    vram_data: Cell<u16>,

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

    // $2122    .BBB BBGG GGGR RRRR    Write Only
    //       - CGRAM data write, increments CGADD byte address after each write
    cgram_data: Cell<u16>,

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
    main_obj_enabled: Cell<bool>,
    main_l4_enabled: Cell<bool>,
    main_l3_enabled: Cell<bool>,
    main_l2_enabled: Cell<bool>,
    main_l1_enabled: Cell<bool>,

    // $212D    ...O 4321    Write Only
    //       - Sub screen layer enable (#)
    sub_obj_enabled: Cell<bool>,
    sub_l4_enabled: Cell<bool>,
    sub_l3_enabled: Cell<bool>,
    sub_l2_enabled: Cell<bool>,
    sub_l1_enabled: Cell<bool>,

    // $212E    ...O 4321    Write Only
    //       - Main screen layer window enable
    main_obj_win_enabled: Cell<bool>,
    main_l4_win_enabled: Cell<bool>,
    main_l3_win_enabled: Cell<bool>,
    main_l2_win_enabled: Cell<bool>,
    main_l1_win_enabled: Cell<bool>,

    // $212F    ...O 4321    Write Only
    //       - Sub screen layer window enable
    sub_obj_win_enabled: Cell<bool>,
    sub_l4_win_enabled: Cell<bool>,
    sub_l3_win_enabled: Cell<bool>,
    sub_l2_win_enabled: Cell<bool>,
    sub_l1_win_enabled: Cell<bool>,

    // $2130    MMSS ..AD    Write Only
    //       - main/sub screen color window black/transparent regions (MS)
    //       - fixed/subscreen (A)
    //       - direct color (D)
    main_col_win_black_region: Cell<WindowColorRegion>,
    sub_col_win_transparent_region: Cell<WindowColorRegion>,
    cmath_addend: Cell<CMathAddend>,
    direct_col_mode: Cell<DirectColorMode>,

    // $2131    MHBO 4321    Write Only
    //       - Color math add/subtract (M)
    //       - half (H)
    //       - backdrop (B)
    //       - layer enable (O4321)
    cmath_operator: Cell<CMathOperator>,
    cmath_half: Cell<bool>,
    cmath_backdrop: Cell<bool>,
    cmath_obj_enabled: Cell<bool>,
    cmath_bg4_enabled: Cell<bool>,
    cmath_bg3_enabled: Cell<bool>,
    cmath_bg2_enabled: Cell<bool>,
    cmath_bg1_enabled: Cell<bool>,

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

    // PPU Memory
    oam: OamData, // 544 Bytes of OAM
    vram: VramData, // 64 KiB of VRAM
    cgram: CgRamData,

    // PPU State
    in_vblank: Cell<bool>,
    in_hblank: Cell<bool>,
    h_counter: Cell<u16>,
    v_counter: Cell<u16>,
}

impl PpuData {
    pub fn new() -> PpuData {
        PpuData::default()
    }
}

// CPU Access
impl PpuData {
    pub fn write(&self, address: u8, data: u8) {
        // println!("PPU write $21{address:02X} with 0x{data:02X}");

        match address {
            0x00 => {
                self.in_fblank.replace(data.bit_en(7));
                self.screen_brightness.replace(data & 0x0F);
                // println!("Set forced blanking to {} and screen brightness to {}", self.in_fblank.get(), self.screen_brightness.get());
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

                self.obj_sprite_size.replace(new_obj_size);
                self.name_secondary_select.replace((data >> 3) & 0x03);
                self.name_base_addr.replace(data & 0x03);

                // println!("Set obj spr size to {:?}, secondary select to {}, and name base addr to ${:X}", self.obj_sprite_size.get(), self.name_secondary_select.get(), self.name_base_addr.get());
            }

            0x02 => {
                let new_addr = (self.oam_addr.get() & 0xFF00) | (data as u16);

                self.oam_addr.replace(new_addr);
                self.priority_rotation_idx.replace(data & 0xFE);
                self.internal_oam_addr.replace((self.oam_addr.get() & 0x1FF) << 1);

                println!("Set OAM addr to ${:04X}, internal OAM addr to ${:04X}, and priority rotation idx to 0x{:02X}", self.oam_addr.get(), self.internal_oam_addr.get(), self.priority_rotation_idx.get());
            }

            0x03 => {
                let new_addr = self.oam_addr.get() & 0x00FF | ((data as u16) << 8);

                self.oam_addr.replace(new_addr);
                self.priority_rotation.replace(data.bit_en(7));
                self.internal_oam_addr.replace((self.oam_addr.get() & 0x1FF) << 1);

                println!("Set OAM addr to ${:04X}, internal OAM addr to ${:04X}, and priority rotation to {}", self.oam_addr.get(), self.internal_oam_addr.get(), self.priority_rotation.get());
            }

            0x04 => {
                let internal_oam_addr = self.internal_oam_addr.get() as usize;

                if internal_oam_addr & 1 == 0 {
                    self.oam_data_latch.replace(data);

                    println!("Set OAM data latch to 0x{:02X}", self.oam_data_latch.get());
                } else if internal_oam_addr < 0x200 {
                    self.oam.0[internal_oam_addr - 1].replace(self.oam_data_latch.get());
                    self.oam.0[internal_oam_addr].replace(data);

                    println!("Wrote 0x{:02X} and 0x{:02X} to OAM addrs ${:04X} and ${:04X}", self.oam_data_latch.get(), data, internal_oam_addr-1, internal_oam_addr);
                }
                
                if internal_oam_addr >= 0x200 {
                    self.oam.0[internal_oam_addr].replace(data);

                    println!("Wrote 0x{:02X} to OAM addr ${:04X}", data, internal_oam_addr);
                }

                self.internal_oam_addr.replace((internal_oam_addr as u16 + 1) & 0x1FF);

                println!("Incremented internal OAM addr to ${:04X}", self.internal_oam_addr.get());
            }

            0x05 => {
                self.bg4_char_size.replace(
                    if data.bit_en(7) { CharSize::Large } else { CharSize::Small }
                );
                self.bg3_char_size.replace(
                    if data.bit_en(6) { CharSize::Large } else { CharSize::Small }
                );
                self.bg2_char_size.replace(
                    if data.bit_en(5) { CharSize::Large } else { CharSize::Small }
                );
                self.bg1_char_size.replace(
                    if data.bit_en(4) { CharSize::Large } else { CharSize::Small }
                );
                self.bg3_priority.replace(
                    if data.bit_en(3) { BgPriority::High } else { BgPriority::Low }
                );
                self.bg_mode.replace(
                    match data & 0x07 {
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

                println!("Set bg char sizes to 4: {:?}, 3: {:?}, 2: {:?}, 1: {:?}, bg 3 priority to {:?}, and bg mode to {:?}",
                    self.bg4_char_size.get(),
                    self.bg3_char_size.get(),
                    self.bg2_char_size.get(),
                    self.bg1_char_size.get(),
                    self.bg3_priority.get(),
                    self.bg_mode.get(),
                );
            }

            0x06 => {
                self.mosaic_size.replace(data >> 4);
                self.bg4_mosaic.replace(data.bit_en(3));
                self.bg3_mosaic.replace(data.bit_en(2));
                self.bg2_mosaic.replace(data.bit_en(1));
                self.bg1_mosaic.replace(data.bit_en(0));

                println!("Set mosaic size to {} and mosaic enables to 4: {}, 3: {}, 2: {}, 1: {}",
                    self.mosaic_size.get(),
                    self.bg4_mosaic.get(),
                    self.bg3_mosaic.get(),
                    self.bg2_mosaic.get(),
                    self.bg1_mosaic.get(),
                );
            }

            0x07 => {
                self.bg1_vram_addr.replace(data >> 2);
                self.bg1_tilemap_count_y.replace(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg1_tilemap_count_x.replace(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
                );

                println!("Set bg1 tilemap base VRAM addr to ${:04X}, bg1 tilemap count y to {:?}, and bg1 tilemap count x to {:?}",
                    (self.bg1_vram_addr.get() as u16) << 10,
                    self.bg1_tilemap_count_y.get(),
                    self.bg1_tilemap_count_x.get(),
                );
            }

            0x08 => {
                self.bg2_vram_addr.replace(data >> 2);
                self.bg2_tilemap_count_y.replace(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg2_tilemap_count_x.replace(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
                );

                println!("Set bg2 tilemap base VRAM addr to ${:04X}, bg2 tilemap count y to {:?}, and bg2 tilemap count x to {:?}",
                    (self.bg2_vram_addr.get() as u16) << 10,
                    self.bg2_tilemap_count_y.get(),
                    self.bg2_tilemap_count_x.get(),
                );
            }

            0x09 => {
                self.bg3_vram_addr.replace(data >> 2);
                self.bg3_tilemap_count_y.replace(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg3_tilemap_count_x.replace(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
                );

                println!("Set bg3 tilemap base VRAM addr to ${:04X}, bg3 tilemap count y to {:?}, and bg3 tilemap count x to {:?}",
                    (self.bg3_vram_addr.get() as u16) << 10,
                    self.bg3_tilemap_count_y.get(),
                    self.bg3_tilemap_count_x.get(),
                );
            }

            0x0A => {
                self.bg4_vram_addr.replace(data >> 2);
                self.bg4_tilemap_count_y.replace(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg4_tilemap_count_x.replace(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
                );

                println!("Set bg4 tilemap base VRAM addr to ${:04X}, bg4 tilemap count y to {:?}, and bg4 tilemap count x to {:?}",
                    (self.bg4_vram_addr.get() as u16) << 10,
                    self.bg4_tilemap_count_y.get(),
                    self.bg4_tilemap_count_x.get(),
                );
            }

            0x0B => {
                self.bg2_chr_base_addr.replace(data >> 4);
                self.bg1_chr_base_addr.replace(data & 0x0F);

                println!("Set bg CHR word base addrs to 2: ${:04X}, 1: ${:04X}", 
                    (self.bg2_chr_base_addr.get() as u16) << 12,
                    (self.bg1_chr_base_addr.get() as u16) << 12,
                );
            }

            0x0C => {
                self.bg4_chr_base_addr.replace(data >> 4);
                self.bg3_chr_base_addr.replace(data & 0x0F);

                println!("Set bg CHR word base addrs to 4: ${:04X}, 3: ${:04X}", 
                    (self.bg4_chr_base_addr.get() as u16) << 12,
                    (self.bg3_chr_base_addr.get() as u16) << 12,
                );
            }

            0x0D => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg1_m7_x_offset.replace(
                    ((data as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );

                println!("Set bg offset latch to 0x{:02X}, bg horizontal offset latch to 0x{:02X}, and bg1 horizontal scroll to {:04X}",
                    self.bg_offset_latch.get(),
                    self.bg_offset_x_latch.get(),
                    self.bg1_m7_x_offset.get(),
                );
            }

            0x0E => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg1_m7_y_offset.replace(((data as u16) << 8) | bgofs_latch);

                println!("Set bg offset latch to 0x{:02X} and bg1 vertical scroll to 0x{:04X}",
                    self.bg_offset_latch.get(),
                    self.bg1_m7_y_offset.get()
                );
            }

            0x0F => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg2_x_offset.replace(
                    ((data as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );

                println!("Set bg offset latch to 0x{:02X}, bg horizontal offset latch to 0x{:02X}, and bg2 horizontal scroll to {:04X}",
                    self.bg_offset_latch.get(),
                    self.bg_offset_x_latch.get(),
                    self.bg2_x_offset.get(),
                );
            }

            0x10 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg2_y_offset.replace(((data as u16) << 8) | bgofs_latch);

                println!("Set bg offset latch to 0x{:02X} and bg2 vertical scroll to 0x{:04X}",
                    self.bg_offset_latch.get(),
                    self.bg2_y_offset.get()
                );
            }

            0x11 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg3_x_offset.replace(
                    ((data as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );

                println!("Set bg offset latch to 0x{:02X}, bg horizontal offset latch to 0x{:02X}, and bg3 horizontal scroll to {:04X}",
                    self.bg_offset_latch.get(),
                    self.bg_offset_x_latch.get(),
                    self.bg3_x_offset.get(),
                );
            }

            0x12 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg3_y_offset.replace(((data as u16) << 8) | bgofs_latch);

                println!("Set bg offset latch to 0x{:02X} and bg3 vertical scroll to 0x{:04X}",
                    self.bg_offset_latch.get(),
                    self.bg3_y_offset.get()
                );
            }

            0x13 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg4_x_offset.replace(
                    ((data as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );

                println!("Set bg offset latch to 0x{:02X}, bg horizontal offset latch to 0x{:02X}, and bg4 horizontal scroll to {:04X}",
                    self.bg_offset_latch.get(),
                    self.bg_offset_x_latch.get(),
                    self.bg4_x_offset.get(),
                );
            }

            0x14 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg4_y_offset.replace(((data as u16) << 8) | bgofs_latch);

                println!("Set bg offset latch to 0x{:02X} and bg4 vertical scroll to 0x{:04X}",
                    self.bg_offset_latch.get(),
                    self.bg4_y_offset.get()
                );
            }

            0x15 => {
                self.vram_addr_inc_mode.replace(
                    if data.bit_en(7) { VramIncMode::HighByte } else { VramIncMode::LowByte }
                );
                self.addr_remap_mode.replace(
                    match (data >> 2) & 3 {
                        0 => AddressRemapping::None,
                        1 => AddressRemapping::ColDepth2,
                        2 => AddressRemapping::ColDepth4,
                        3 => AddressRemapping::ColDepth8,
                        _ => unreachable!(),
                    }
                );
                self.addr_inc_size.replace(
                    match data & 3 {
                        0 => IncrSize::Bytes2,
                        1 => IncrSize::Bytes64,
                        2 => IncrSize::Bytes256,
                        3 => IncrSize::Bytes256,
                        _ => unreachable!(),
                    }
                );

                // println!("Set VRAM inc mode to {:?}, VRAM addr remap to {:?}, and VRAM addr inc size to {:?}",
                //     self.vram_addr_inc_mode.get(),
                //     self.addr_remap_mode.get(),
                //     self.addr_inc_size.get(),
                // );
            }

            0x16 => {
                self.vram_addr.set_lo(data);
                self.vram_latch.replace(
                    self.vram.0[self.get_vram_addr() as usize].get()
                );

                // println!("Set VRAM addr (lo) to ${:04X} and VRAM latch to {:04X}", self.vram_addr.get(), self.vram_latch.get());
            }

            0x17 => {
                self.vram_addr.set_hi(data);
                self.vram_latch.replace(
                    self.vram.0[self.get_vram_addr() as usize].get()
                );

                // println!("Set VRAM addr (hi) to ${:04X} and VRAM latch to {:04X}", self.vram_addr.get(), self.vram_latch.get());
            }

            0x18 => {
                if self.in_fblank.get() || self.in_vblank.get() {
                    self.vram.0[self.get_vram_addr() as usize].set_lo(data);
                }

                // println!("$2118 VRAM addr: ${:04X}, data written: {:02X}", addr, data);

                // println!("CPU wrote VRAM data (lo) to addr ${:04X} with data 0x{:02X}, new word = 0x{:04X}", self.vram_addr.get(), data, self.vram.0[addr].get());

                match self.vram_addr_inc_mode.get() {
                    VramIncMode::LowByte => self.inc_vram_addr(),
                    _ => {}
                }
            }

            0x19 => {
                if self.in_fblank.get() || self.in_vblank.get() {
                    self.vram.0[self.get_vram_addr() as usize].set_hi(data);
                }

                // println!("CPU wrote VRAM data (hi) to addr ${:04X} with data 0x{:02X}, new word = {:04X}", self.vram_addr.get(), data, self.vram.0[addr].get());

                match self.vram_addr_inc_mode.get() {
                    VramIncMode::HighByte => self.inc_vram_addr(),
                    _ => {}
                }
            }

            0x1A => {
                self.m7_tilemap_repeat.replace(data.bit_en(7));
                self.m7_fill_mode.replace(
                    if data.bit_en(6) { M7FillMode::Character } else { M7FillMode::Transparent }
                );
                self.m7_flip_bg_y.replace(data.bit_en(1));
                self.m7_flip_bg_x.replace(data.bit_en(0));
            }

            0x1B => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_a.replace(
                    ((data as u16) << 8) | latched_val
                );
                self.mult_factor_16.replace(
                    ((data as u16) << 8) | latched_val
                );

                self.update_multiply_result();

                println!("Set m7 latch to 0x{:02X}, mult factor 16bit/m7 matrix A to 0x{:04X}, and mult result to 0x{:08X}",
                    self.m7_latch.get(),
                    self.m7_matrix_a.get(),
                    self.multiply_result.get(),
                );
            }

            0x1C => {
                let latched_val = self.m7_latch.replace(data);

                self.m7_matrix_b.replace(
                    ((data as u16) << 8) | (latched_val as u16)
                );
                self.mult_factor_8.replace(latched_val);

                self.update_multiply_result();

                println!("Set m7 latch to 0x{:02X}, my matrix B to 0x{:04X}, mult factor 8bit to 0x{:02X}, and mult result to 0x{:08X}",
                    self.m7_latch.get(),
                    self.m7_matrix_b.get(),
                    self.mult_factor_8.get(),
                    self.multiply_result.get(),
                );
            }

            0x1D => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_c.replace(((data as u16) << 8) | latched_val);

                println!("Set m7 latch to 0x{:02X} and m7 matrix C to {:04X}",
                    self.m7_latch.get(),
                    self.m7_matrix_c.get(),
                );
            }

            0x1E => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_d.replace(((data as u16) << 8) | latched_val);

                println!("Set m7 latch to 0x{:02X} and m7 matrix D to {:04X}",
                    self.m7_latch.get(),
                    self.m7_matrix_d.get(),
                );
            }

            0x1F => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_center_x.replace(((data as u16) << 8) | latched_val);
            }

            0x20 => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_center_y.replace(((data as u16) << 8) | latched_val);
            }

            0x21 => {
                self.cgram_addr.replace(data);
                self.cgram_toggle.set_lo();

                println!("Set CGRAM addr to ${:02X} and CGRAM toggle to {:?}", self.cgram_addr.get(), self.cgram_toggle.get());
            }

            0x22 => {
                // print!("Write to CGRAM addr ${:02X} with data 0x{:02X}", self.cgram_addr.get(), data);

                if !self.cgram_toggle.toggle() {
                    let addr = self.cgram_addr.get();
                    let new_col = ((data as u16) << 8) | self.cgram_latch.get() as u16;

                    self.cgram.0[addr as usize].replace(new_col);

                    self.cgram_addr.replace(addr + 1);

                    if addr == 0 {
                        println!("Set transparent color to 0x{:04X}", self.cgram.0[0].get());
                    }

                    // println!(", new val = 0x{:04X}, new addr = ${:02X}", self.cgram.0[addr as usize].get(), addr+1);
                } else {
                    self.cgram_latch.replace(data);

                    // println!(", new latch = 0x{:02X}", data);
                }
            }

            0x23 => {
                self.bg2_w2_enabled.replace(data.bit_en(7));
                self.bg2_w2_inverted.replace(data.bit_en(6));
                self.bg2_w1_enabled.replace(data.bit_en(5));
                self.bg2_w1_inverted.replace(data.bit_en(4));
                self.bg1_w2_enabled.replace(data.bit_en(3));
                self.bg1_w2_inverted.replace(data.bit_en(2));
                self.bg1_w1_enabled.replace(data.bit_en(1));
                self.bg1_w1_inverted.replace(data.bit_en(0));
            }

            0x24 => {
                self.bg4_w2_enabled.replace(data.bit_en(7));
                self.bg4_w2_inverted.replace(data.bit_en(6));
                self.bg4_w1_enabled.replace(data.bit_en(5));
                self.bg4_w1_inverted.replace(data.bit_en(4));
                self.bg3_w2_enabled.replace(data.bit_en(3));
                self.bg3_w2_inverted.replace(data.bit_en(2));
                self.bg3_w1_enabled.replace(data.bit_en(1));
                self.bg3_w1_inverted.replace(data.bit_en(0));
            }

            0x25 => {
                self.col_w2_enabled.replace(data.bit_en(7));
                self.col_w2_inverted.replace(data.bit_en(6));
                self.col_w1_enabled.replace(data.bit_en(5));
                self.col_w1_inverted.replace(data.bit_en(4));
                self.obj_w2_enabled.replace(data.bit_en(3));
                self.obj_w2_inverted.replace(data.bit_en(2));
                self.obj_w1_enabled.replace(data.bit_en(1));
                self.obj_w1_inverted.replace(data.bit_en(0));
            }

            0x26 => {
                self.w1_left_pos.replace(data);
            }

            0x27 => {
                self.w1_right_pos.replace(data);
            }

            0x28 => {
                self.w2_left_pos.replace(data);
            }

            0x29 => {
                self.w2_right_pos.replace(data);
            }

            0x2A => {
                self.bg4_win_logic.replace(
                    match data >> 6 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.bg3_win_logic.replace(
                    match (data >> 4) & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.bg2_win_logic.replace(
                    match (data >> 2) & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.bg1_win_logic.replace(
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
                self.col_win_logic.replace(
                    match (data >> 2) & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.obj_win_logic.replace(
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
                self.main_obj_enabled.replace(data.bit_en(4));
                self.main_l4_enabled.replace(data.bit_en(3));
                self.main_l3_enabled.replace(data.bit_en(2));
                self.main_l2_enabled.replace(data.bit_en(1));
                self.main_l1_enabled.replace(data.bit_en(0));
            }

            0x2D => {
                self.sub_obj_enabled.replace(data.bit_en(4));
                self.sub_l4_enabled.replace(data.bit_en(3));
                self.sub_l3_enabled.replace(data.bit_en(2));
                self.sub_l2_enabled.replace(data.bit_en(1));
                self.sub_l1_enabled.replace(data.bit_en(0));
            }

            0x2E => {
                self.main_obj_win_enabled.replace(data.bit_en(4));
                self.main_l4_win_enabled.replace(data.bit_en(3));
                self.main_l3_win_enabled.replace(data.bit_en(2));
                self.main_l2_win_enabled.replace(data.bit_en(1));
                self.main_l1_win_enabled.replace(data.bit_en(0));
            }

            0x2F => {
                self.sub_obj_win_enabled.replace(data.bit_en(4));
                self.sub_l4_win_enabled.replace(data.bit_en(3));
                self.sub_l3_win_enabled.replace(data.bit_en(2));
                self.sub_l2_win_enabled.replace(data.bit_en(1));
                self.sub_l1_win_enabled.replace(data.bit_en(0));
            }

            0x30 => {
                self.main_col_win_black_region.replace(
                    match data >> 6 {
                        0 => WindowColorRegion::Nowhere,
                        1 => WindowColorRegion::Outside,
                        2 => WindowColorRegion::Inside,
                        3 => WindowColorRegion::Everywhere,
                        _ => unreachable!(),
                    }
                );
                self.sub_col_win_transparent_region.replace(
                    match (data >> 4) & 3 {
                        0 => WindowColorRegion::Nowhere,
                        1 => WindowColorRegion::Outside,
                        2 => WindowColorRegion::Inside,
                        3 => WindowColorRegion::Everywhere,
                        _ => unreachable!(),
                    }
                );
                self.cmath_addend.replace(
                    if data.bit_en(1) { CMathAddend::Subscreen } else { CMathAddend::Fixed }
                );
                self.direct_col_mode.replace(
                    if data.bit_en(0) { DirectColorMode::Palette } else { DirectColorMode::Direct }
                );
            }

            0x31 => {
                self.cmath_operator.replace(
                    match data >> 7 {
                        0 => CMathOperator::Add,
                        1 => CMathOperator::Subtract,
                        _ => unreachable!(),
                    }
                );
                self.cmath_half.replace(data.bit_en(6));
                self.cmath_backdrop.replace(data.bit_en(5));
                self.cmath_obj_enabled.replace(data.bit_en(4));
                self.cmath_bg4_enabled.replace(data.bit_en(3));
                self.cmath_bg3_enabled.replace(data.bit_en(2));
                self.cmath_bg2_enabled.replace(data.bit_en(1));
                self.cmath_bg1_enabled.replace(data.bit_en(0));
            }

            0x32 => {
                let prev_col = self.fixed_color.get();
                let new_val = (data & 0x1F) as u16;

                let new_r =  (new_val << 10) * data.get_bit(5) as u16;
                let new_g =  (new_val << 5) * data.get_bit(6) as u16;
                let new_b =  (new_val) * data.get_bit(7) as u16;
                let new_col = new_r | new_g | new_b;

                let mask_r = (data.get_bit(5) as u16) * 0x7C00;
                let mask_g = (data.get_bit(6) as u16) * 0x03E0;
                let mask_b = (data.get_bit(7) as u16) * 0x001F;
                let mask = mask_r | mask_g | mask_b;

                self.fixed_color.replace((prev_col & mask) | new_col);
            }

            0x33 => {
                self._external_sync.replace(data.bit_en(7));
                self.ext_bg_enabled.replace(data.bit_en(6));
                self.hi_res_enabled.replace(data.bit_en(3));
                self.overscan_enabled.replace(data.bit_en(2));
                self.obj_interlace_enabled.replace(data.bit_en(1));
                self.screen_interlace_enabled.replace(data.bit_en(0));
            }

            _ => {}
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        // println!("PPU read $21{address:02X}");

        let data = match address {
            0x34 => { self.multiply_result.get() as u8 }
            0x35 => { (self.multiply_result.get() >> 8) as u8 }
            0x36 => { (self.multiply_result.get() >> 16) as u8 }

            0x37 => {
                // When counter_latch transitions from 0 to 1
                // https://snes.nesdev.org/wiki/PPU_registers#OPVCT
                if !self.counter_toggle.is_high() {
                    self.h_counter_latch.replace(self.h_counter.get());
                    self.v_counter_latch.replace(self.v_counter.get());
                }

                self.counter_toggle.set_hi();

                0 // CPU OPEN BUS
            }

            0x38 => {
                let addr = self.internal_oam_addr.replace(self.internal_oam_addr.get() + 1);

                self.oam.0[addr as usize].get()
            }

            0x39 => {
                let val = self.vram_latch.get() as u8;

                match self.vram_addr_inc_mode.get() {
                    VramIncMode::LowByte => {
                        self.vram_latch.replace(
                            if self.in_fblank.get() || self.in_vblank.get() {
                                self.vram.0[self.get_vram_addr() as usize].get()
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
                        self.vram_latch.replace(
                            if self.in_fblank.get() || self.in_vblank.get() {
                                self.vram.0[self.get_vram_addr() as usize].get()
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
                if self.cgram_toggle.toggle() {
                    self.cgram.0[self.cgram_addr.get() as usize].get() as u8
                } else {
                    (self.cgram.0[self.cgram_addr.get() as usize].get() >> 8) as u8 // HIGH BIT IS PPU2 OPEN BUS
                }
            }

            0x3C => {
                if self.h_counter_toggle.toggle() {
                    self.h_counter_latch.get() as u8
                } else {
                    (self.h_counter_latch.get() >> 8) as u8 // HIGH 7 BITS ARE PPU2 OPEN BUS
                }
            }

            0x3D => {
                if self.v_counter_toggle.toggle() {
                    self.v_counter_latch.get() as u8
                } else {
                    (self.v_counter_latch.get() >> 8) as u8 // HIGH 7 BITS ARE PPU2 OPEN BUS
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

        self.multiply_result.replace(result & 0x00FFFFFF);
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

        self.vram_addr.replace(self.vram_addr.get() + inc);
    }
}

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
    size: ObjectSize,
}

pub struct Ppu5C7x {
    registers: Rc<PpuData>,

    dot: u16,
    scanline: u16,
    clocks_until_dot: u8,
    frame: usize,

    scanline_sprites: Vec<OAMSprite>,
    scanline_spr_cnt: usize,

    pub frame_finished: bool,
}

impl Ppu5C7x {
    pub fn new(ppu_data: Rc<PpuData>) -> Self {
        Ppu5C7x {
            registers: ppu_data,
            dot: 0,
            scanline: 0,
            clocks_until_dot: 1,
            frame: 0,
            scanline_sprites: Vec::with_capacity(32),
            scanline_spr_cnt: 0,
            frame_finished: false,
        }
    }

    pub fn clock(&mut self, frame_buffer: &mut [XRGB8888], logger: &mut SnemLogger) {
        self.clocks_until_dot -= 1;

        if self.clocks_until_dot == 0 {
            if !self.in_fblank() && !self.in_hblank() && !self.in_vblank() && self.scanline != 0 {
                self.dot(frame_buffer);
            }
    
            self.update_dot_and_scanline();

            self.clocks_until_dot += 4;

            if self.dot >= SCANLINE_END_DOT-4 {
                self.clocks_until_dot += 1;
            }
        }
    }

    fn update_dot_and_scanline(&mut self) {
        self.dot += 1;

        if self.dot == SCANLINE_END_DOT {
            self.dot = 0;
            self.scanline += 1;

            // if self.frame > 7 {
            //     std::thread::sleep(std::time::Duration::new(0, 50_000_000));
            //     self.frame_finished = true;

            //     println!("Scanline: {}, frame: {}", self.scanline, self.frame);
            // }

            if self.scanline == VBLANK_END_SCANLINE_NTSC {
                self.scanline = 0;
            }
        }

        match (self.dot, self.scanline) {
            // End of v-blank, scanline 0 is not visible
            (0, 0) => {
                self.registers.in_vblank.replace(false);
            }
            // Start of visible scanline, end of h-blank
            (HBLANK_END_DOT, 1..=HBLANK_DISABLE_SCANLINE) => {
                self.registers.in_hblank.replace(false);
                self.find_scanline_sprites();
            }
            // End of visible scanline, start of h-blank
            (HBLANK_START_DOT, 0..HBLANK_DISABLE_SCANLINE) => {
                self.registers.in_hblank.replace(true);
            }
            // Start of v-blank
            (0, VBLANK_START_SCANLINE) => {
                self.registers.in_vblank.replace(true);
                self.frame_finished = true;
                self.frame += 1;
            }
            _ => {}
        }
    }

    /// Finds all possible sprites that could be rendered on the current scanline
    /// based on the y-positions of the sprites
    fn find_scanline_sprites(&mut self) {
        self.scanline_sprites.clear();

        let screen_y = self.screen_y();

        self.scanline_spr_cnt = 0;
        for (spr_idx, spr_data) in self.registers.oam.0[..0x200].chunks(4).enumerate().rev() {
            // This bit munging is absolutely horrifying but works. We need to 1) get the packed byte containing
            // our data, 2) create a mask to get the bits within the packed byte, and 3) or the byte with the
            // mask to get the relevant bits. Each byte looks like DdCcBbAa, with each letter pair corresponding
            // to a single sprite (32 bytes * 4 pairs = 128, matching # of sprites in OAM).
            let spr_extra_data = (self.registers.oam.0[0x200 | (spr_idx >> 2)].get() >> ((spr_idx & 3) << 1)) & 3;
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
            let spr_y = spr_data[1].get();
            let spr_x = (((spr_extra_data as u16) & 1) << 8) | (spr_data[0].get() as u16);
            let (spr_max_x, spr_y_max) = match spr_size {
                ObjectSize::Size8x8 => (spr_x + 8, spr_y + 8),
                ObjectSize::Size16x16 => (spr_x + 16, spr_y + 16),
                ObjectSize::Size16x32 | ObjectSize::Size32x32 => (spr_x + 32, spr_y + 32),
                ObjectSize::Size32x64 | ObjectSize::Size64x64 => (spr_x + 64, spr_y + 64),
            };

            // Sprite should be on scanline
            if spr_y as usize <= screen_y && screen_y < spr_y_max as usize  {
                let sprite = OAMSprite {
                    x: spr_x,
                    max_x: spr_max_x,
                    y: spr_y,
                    tile_idx: spr_data[2].get(),
                    use_second_obj_table: (spr_data[3].get() & 1) != 0,
                    palette: (spr_data[3].get() >> 1) & 7,
                    priority: (spr_data[3].get() >> 4) & 3,
                    flip_x: (spr_data[3].get() & 0x40) != 0,
                    flip_y: (spr_data[3].get() & 0x80) != 0,
                    size: spr_size,
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

struct BgColorData {
    raw_color: u16,
    priority: bool,
    transparent: bool,
}

struct SpriteColorData {
    raw_color: u16,
    priority: u8,
    transparent: bool,
}

/// Converts a u16 of the form (.bbb bbgg gggr rrrr) into an XRGB8888 color
fn cgram_raw_color_to_xrgb(word: u16) -> XRGB8888 {
    let r = ((word & 0x1F) as u32) << 19;
    let g = ((word & 0x3E0) as u32) << 6;
    let b = ((word & 0x7C00) as u32) >> 7;

    XRGB8888::new_with_raw_value( 0xFF000000 | r | g | b )
}

impl Ppu5C7x {
    fn screen_x(&self) -> usize { self.dot as usize - VISIBLE_SCANLINE_START_DOT }
    fn screen_y(&self) -> usize { self.scanline as usize - 1 }
    fn transparent_color(&self) -> u16 { self.registers.cgram.0[0].get() }

    fn dot(&mut self, frame_buffer: &mut [XRGB8888]) {
        // let bg1_win_en = window_enable(
        //     self.registers.bg1_w1_enabled.get(),
        //     self.registers.bg1_w1_inverted.get(),
        //     self.registers.bg1_w2_enabled.get(),
        //     self.registers.bg1_w2_inverted.get(),
        //     self.registers.bg1_win_logic.get(),
        // );
        // let bg2_win_en = window_enable(            
        //     self.registers.bg2_w1_enabled.get(),    
        //     self.registers.bg2_w1_inverted.get(),    
        //     self.registers.bg2_w2_enabled.get(),     
        //     self.registers.bg2_w2_inverted.get(),    
        //     self.registers.bg2_win_logic.get(),    
        // );                                                  
        // let bg3_win_en = window_enable(            
        //     self.registers.bg3_w1_enabled.get(),    
        //     self.registers.bg3_w1_inverted.get(),    
        //     self.registers.bg3_w2_enabled.get(),     
        //     self.registers.bg3_w2_inverted.get(),    
        //     self.registers.bg3_win_logic.get(),    
        // );                                                  
        // let bg4_win_en = window_enable(            
        //     self.registers.bg4_w1_enabled.get(),    
        //     self.registers.bg4_w1_inverted.get(),    
        //     self.registers.bg4_w2_enabled.get(),     
        //     self.registers.bg4_w2_inverted.get(),    
        //     self.registers.bg4_win_logic.get(),    
        // );                                                  
        // let col_win_en = window_enable(            
        //     self.registers.col_w1_enabled.get(),    
        //     self.registers.col_w1_inverted.get(),    
        //     self.registers.col_w2_enabled.get(),     
        //     self.registers.col_w2_inverted.get(),    
        //     self.registers.col_win_logic.get(),    
        // );                                                  
        // let obj_win_en = window_enable(            
        //     self.registers.obj_w1_enabled.get(),    
        //     self.registers.obj_w1_inverted.get(),    
        //     self.registers.obj_w2_enabled.get(),     
        //     self.registers.obj_w2_inverted.get(),    
        //     self.registers.obj_win_logic.get(),    
        // );       

        // self.bg1_display_chr_table(frame_buffer);
        // return;                                           

        let screen_x = self.screen_x();
        let screen_y = self.screen_y();

        // Get color of sprite on this pixel. Different bg modes will use it
        // differently depending on priority and register settings.
        let spr_col = self.sprite_dot(screen_x, screen_y);

        let dot_col = match self.bg_mode() {
            BgMode::Mode0 => self.bg_mode0_dot(screen_x, screen_y, spr_col),
            // BgMode::Mode1 => self.bg_mode1_dot(frame_buffer, spr_col),
            // BgMode::Mode2 => self.bg_mode2_dot(frame_buffer, spr_col),
            // BgMode::Mode3 => self.bg_mode3_dot(frame_buffer, spr_col),
            // BgMode::Mode4 => self.bg_mode4_dot(frame_buffer, spr_col),
            // BgMode::Mode5 => self.bg_mode5_dot(frame_buffer, spr_col),
            // BgMode::Mode6 => self.bg_mode6_dot(frame_buffer, spr_col),
            // BgMode::Mode7 => self.bg_mode7_dot(frame_buffer, spr_col),
            _ => 0,
        };

        frame_buffer[screen_y * 256 + screen_x] = cgram_raw_color_to_xrgb(dot_col);
    }

    /// Gets the color of the first visible sprite on the screen.
    fn sprite_dot(&mut self, screen_x: usize, screen_y: usize) -> SpriteColorData {
        let mut scanline_spr_cnt = self.scanline_spr_cnt;

        for i in 0..self.scanline_sprites.len() {
            scanline_spr_cnt -= 1;

            let sprite = &self.scanline_sprites[scanline_spr_cnt];

            if scanline_spr_cnt == 0 {
                scanline_spr_cnt = 32;
            }

            if sprite.x as usize <= screen_x && screen_x < sprite.max_x as usize  {
                let sprite_col = screen_x - sprite.x as usize;
                let sprite_row = screen_y - sprite.y as usize;
                let (tile_x, tile_col) = (sprite_col / 8, sprite_col % 8);
                let (tile_y, tile_row) = (sprite_row / 8, sprite_row % 8);
                
                let spr_width_tiles = match sprite.size {
                    ObjectSize::Size8x8 => 1,
                    ObjectSize::Size16x16 | ObjectSize::Size16x32 => 2,
                    ObjectSize::Size32x32 | ObjectSize::Size32x64 => 4,
                    ObjectSize::Size64x64 => 8,
                };

                let tile_idx = tile_y*spr_width_tiles + tile_x;

                let obj_table_base_addr = if sprite.use_second_obj_table {
                    ((self.name_base_addr() as u16) << 13) + ((self.name_secondary_select() as u16) << 12)
                } else {
                    (self.name_base_addr() as u16) << 13
                };

                let spr_tile_base_addr = obj_table_base_addr | ((sprite.tile_idx as u16) << 4);
                let spr_tile_addr = spr_tile_base_addr + ((tile_idx as u16) << 4);
                let spr_tile_row_addr = spr_tile_addr | ((tile_row as u16) << 1);
                
                let bp01 = self.vram_read(spr_tile_row_addr + 0);
                let bp23 = self.vram_read(spr_tile_row_addr + 1);

                let b0 = (bp01 >> (7-tile_col)) & 1;
                let b1 = (bp01 >> (15-tile_col)) & 1;
                let b2 = (bp23 >> (7-tile_col)) & 1;
                let b3 = (bp23 >> (15-tile_col)) & 1;

                let pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0;

                // Transparent sprite
                if pal_idx == 0 {
                    // If it's the last sprite, all sprites were transparent
                    if i == self.scanline_sprites.len() - 1 {
                        return SpriteColorData {
                            raw_color: 0,
                            priority: sprite.priority,
                            transparent: true,
                        };
                    }

                    continue;
                }

                let cgram_addr = 0x200 | ((sprite.palette as u16) << 4) | pal_idx;

                let spr_col = self.registers.cgram.0[cgram_addr as usize].get();

                return SpriteColorData {
                    raw_color: spr_col,
                    priority: sprite.priority,
                    transparent: false,
                };
            }
        }

        // No sprites on this dot, return a transparent color
        SpriteColorData {
            raw_color: 0,
            priority: 0,
            transparent: true,
        }
    }

    /// Compute the color of this dot, combining all bg layers and object color
    /// data. Computes only as many layers as it needs to before returning the
    /// color of the dot.
    fn bg_mode0_dot(&mut self, screen_x: usize, screen_y: usize, spr_col: SpriteColorData) -> u16 {
        const BG1_BASE_CGRAM_ADDR: u16 = 0x00;
        const BG2_BASE_CGRAM_ADDR: u16 = 0x20;
        const BG3_BASE_CGRAM_ADDR: u16 = 0x40;
        const BG4_BASE_CGRAM_ADDR: u16 = 0x60;

        if spr_col.priority == 3 && !spr_col.transparent {
            return spr_col.raw_color;
        }

        let tilemap_idx = ( (screen_y / 8) * 32 + (screen_x / 8) ) as u16;

        let chr_x = (screen_x & 0x7) as u8;
        let chr_y = (screen_y & 0x7) as u16;

        let mode0_bg_col_data = |bg_vram_addr: u8, bg_chr_base_addr: u8, bg_cgram_base_addr: u16| -> BgColorData {
            let bg_tile_addr = ((bg_vram_addr as u16) << 10) + tilemap_idx;
            let tile_data = self.vram_read(bg_tile_addr);

            let bg_priority = (tile_data & 0x2000) != 0;
            let bg_tile_pal = (tile_data >> 10) & 0x7;
            let bg_tile_idx = tile_data & 0x03FF;

            let bg_chr_word_addr = ((bg_chr_base_addr as u16) << 12) + (bg_tile_idx << 3);

            let bitplanes = self.vram_read(bg_chr_word_addr + chr_y);
            let bp0 = bitplanes as u8;
            let bp1 = (bitplanes >> 8) as u8;
            
            let bg_pal_idx = ((bp0 >> (7-chr_x)) & 1) | (((bp1 >> (7-chr_x)) & 1) << 1);

            let bg_cgram_addr = bg_cgram_base_addr | (bg_tile_pal << 2) | bg_pal_idx as u16;

            BgColorData {
                raw_color: self.registers.cgram.0[bg_cgram_addr as usize].get(),
                priority: bg_priority,
                transparent: (bg_pal_idx == 0)
            }
        };

        let bg1_col = mode0_bg_col_data(
            self.bg1_vram_addr(),
            self.bg1_chr_base_addr(),
            BG1_BASE_CGRAM_ADDR,
        );

        let bg2_col = mode0_bg_col_data(
            self.bg2_vram_addr(),
            self.bg2_chr_base_addr(),
            BG2_BASE_CGRAM_ADDR,
        );

        if bg2_col.priority && !bg2_col.transparent {
            return bg2_col.raw_color;
        }

        if spr_col.priority == 2 && !spr_col.transparent {
            return spr_col.raw_color;
        }

        if !bg1_col.transparent {
            return bg1_col.raw_color;
        }

        if !bg2_col.transparent {
            return bg2_col.raw_color;
        }

        if spr_col.priority == 1 && !spr_col.transparent {
            return spr_col.raw_color;
        }

        let bg3_col = mode0_bg_col_data(
            self.bg3_vram_addr(),
            self.bg3_chr_base_addr(),
            BG3_BASE_CGRAM_ADDR,
        );

        if bg3_col.priority && !bg3_col.transparent {
            return bg3_col.raw_color;
        }

        let bg4_col = mode0_bg_col_data(
            self.bg4_vram_addr(),
            self.bg4_chr_base_addr(),
            BG4_BASE_CGRAM_ADDR,
        );

        if bg4_col.priority && !bg4_col.transparent {
            return bg4_col.raw_color;
        }

        if !spr_col.transparent {
            return spr_col.raw_color;
        }

        if !bg3_col.transparent {
            return bg3_col.raw_color;
        }

        if !bg4_col.transparent {
            return bg4_col.raw_color;
        }

        self.transparent_color()
    }

    fn bg_mode1_dot(&mut self, frame_buffer: &mut [XRGB8888]) {

    }

    fn bg_mode2_dot(&mut self, frame_buffer: &mut [XRGB8888]) {

    }

    fn bg_mode3_dot(&mut self, frame_buffer: &mut [XRGB8888]) {

    }

    fn bg_mode4_dot(&mut self, frame_buffer: &mut [XRGB8888]) {

    }

    fn bg_mode5_dot(&mut self, frame_buffer: &mut [XRGB8888]) {

    }

    fn bg_mode6_dot(&mut self, frame_buffer: &mut [XRGB8888]) {

    }

    fn bg_mode7_dot(&mut self, frame_buffer: &mut [XRGB8888]) {

    }

}

// Getters & Setters for registers
impl Ppu5C7x {
    fn screen_brightness(&self) -> u8 { self.registers.screen_brightness.get() }
    fn obj_sprite_size(&self) -> ObjectSizeSelect { self.registers.obj_sprite_size.get() }
    fn name_secondary_select(&self) -> u8 { self.registers.name_secondary_select.get() }
    fn name_base_addr(&self) -> u8 { self.registers.name_base_addr.get() }
    fn oam_addr(&self) -> u16 { self.registers.oam_addr.get() }
    fn priority_rotation(&self) -> bool { self.registers.priority_rotation.get() }
    fn oam_data(&self) -> u8 { self.registers.oam_data.get() }
    fn oam_data_latch(&self) -> u8 { self.registers.oam_data_latch.get() }
    fn bg4_char_size(&self) -> CharSize { self.registers.bg4_char_size.get() }
    fn bg3_char_size(&self) -> CharSize { self.registers.bg3_char_size.get() }
    fn bg2_char_size(&self) -> CharSize { self.registers.bg2_char_size.get() }
    fn bg1_char_size(&self) -> CharSize { self.registers.bg1_char_size.get() }
    fn bg3_priority(&self) -> BgPriority { self.registers.bg3_priority.get() }
    fn bg_mode(&self) -> BgMode { self.registers.bg_mode.get() }
    fn mosaic_size(&self) -> u8 { self.registers.mosaic_size.get() }
    fn bg4_mosaic(&self) -> bool { self.registers.bg4_mosaic.get() }
    fn bg3_mosaic(&self) -> bool { self.registers.bg3_mosaic.get() }
    fn bg2_mosaic(&self) -> bool { self.registers.bg2_mosaic.get() }
    fn bg1_mosaic(&self) -> bool { self.registers.bg1_mosaic.get() }
    fn bg1_vram_addr(&self) -> u8 { self.registers.bg1_vram_addr.get() }
    fn bg1_tilemap_count_y(&self) -> TilemapCount { self.registers.bg1_tilemap_count_y.get() }
    fn bg1_tilemap_count_x(&self) -> TilemapCount { self.registers.bg1_tilemap_count_x.get() }
    fn bg2_vram_addr(&self) -> u8 { self.registers.bg2_vram_addr.get() }
    fn bg2_tilemap_count_y(&self) -> TilemapCount { self.registers.bg2_tilemap_count_y.get() }
    fn bg2_tilemap_count_x(&self) -> TilemapCount { self.registers.bg2_tilemap_count_x.get() }
    fn bg3_vram_addr(&self) -> u8 { self.registers.bg3_vram_addr.get() }
    fn bg3_tilemap_count_y(&self) -> TilemapCount { self.registers.bg3_tilemap_count_y.get() }
    fn bg3_tilemap_count_x(&self) -> TilemapCount { self.registers.bg3_tilemap_count_x.get() }
    fn bg4_vram_addr(&self) -> u8 { self.registers.bg4_vram_addr.get() }
    fn bg4_tilemap_count_y(&self) -> TilemapCount { self.registers.bg4_tilemap_count_y.get() }
    fn bg4_tilemap_count_x(&self) -> TilemapCount { self.registers.bg4_tilemap_count_x.get() }
    fn bg2_chr_base_addr(&self) -> u8 { self.registers.bg2_chr_base_addr.get() }
    fn bg1_chr_base_addr(&self) -> u8 { self.registers.bg1_chr_base_addr.get() }
    fn bg4_chr_base_addr(&self) -> u8 { self.registers.bg4_chr_base_addr.get() }
    fn bg3_chr_base_addr(&self) -> u8 { self.registers.bg3_chr_base_addr.get() }
    fn m7_latch(&self) -> u8 { self.registers.m7_latch.get() }
    fn bg_offset_latch(&self) -> u8 { self.registers.bg_offset_latch.get() }
    fn bg_offset_x_latch(&self) -> u8 { self.registers.bg_offset_x_latch.get() }
    fn bg1_m7_x_offset(&self) -> u16 { self.registers.bg1_m7_x_offset.get() }
    fn bg1_m7_y_offset(&self) -> u16 { self.registers.bg1_m7_y_offset.get() }
    fn bg2_x_offset(&self) -> u16 { self.registers.bg2_x_offset.get() }
    fn bg2_y_offset(&self) -> u16 { self.registers.bg2_y_offset.get() }
    fn bg3_x_offset(&self) -> u16 { self.registers.bg3_x_offset.get() }
    fn bg3_y_offset(&self) -> u16 { self.registers.bg3_y_offset.get() }
    fn bg4_x_offset(&self) -> u16 { self.registers.bg4_x_offset.get() }
    fn bg4_y_offset(&self) -> u16 { self.registers.bg4_y_offset.get() }
    fn vram_addr_inc_mode(&self) -> VramIncMode { self.registers.vram_addr_inc_mode.get() }
    fn addr_remap_mode(&self) -> AddressRemapping { self.registers.addr_remap_mode.get() }
    fn addr_inc_size(&self) -> IncrSize { self.registers.addr_inc_size.get() }
    fn vram_addr(&self) -> u16 { self.registers.vram_addr.get() }
    fn vram_data(&self) -> u16 { self.registers.vram_data.get() }
    fn m7_tilemap_repeat(&self) -> bool { self.registers.m7_tilemap_repeat.get() }
    fn m7_fill_mode(&self) -> M7FillMode { self.registers.m7_fill_mode.get() }
    fn m7_flip_bg_y(&self) -> bool { self.registers.m7_flip_bg_y.get() }
    fn m7_flip_bg_x(&self) -> bool { self.registers.m7_flip_bg_x.get() }
    fn m7_matrix_a(&self) -> u16 { self.registers.m7_matrix_a.get() }
    fn m7_matrix_b(&self) -> u16 { self.registers.m7_matrix_b.get() }
    fn m7_matrix_c(&self) -> u16 { self.registers.m7_matrix_c.get() }
    fn m7_matrix_d(&self) -> u16 { self.registers.m7_matrix_d.get() }
    fn m7_center_x(&self) -> u16 { self.registers.m7_center_x.get() }
    fn m7_center_y(&self) -> u16 { self.registers.m7_center_y.get() }
    fn cgram_toggle(&self) -> ToggleState { self.registers.cgram_toggle.get() }
    fn cgram_addr(&self) -> u8 { self.registers.cgram_addr.get() }
    fn cgram_latch(&self) -> u8 { self.registers.cgram_latch.get() }
    fn cgram_data(&self) -> u16 { self.registers.cgram_data.get() }
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
    fn main_obj_enabled(&self) -> bool { self.registers.main_obj_enabled.get() }
    fn main_l4_enabled(&self) -> bool { self.registers.main_l4_enabled.get() }
    fn main_l3_enabled(&self) -> bool { self.registers.main_l3_enabled.get() }
    fn main_l2_enabled(&self) -> bool { self.registers.main_l2_enabled.get() }
    fn main_l1_enabled(&self) -> bool { self.registers.main_l1_enabled.get() }
    fn sub_obj_enabled(&self) -> bool { self.registers.sub_obj_enabled.get() }
    fn sub_l4_enabled(&self) -> bool { self.registers.sub_l4_enabled.get() }
    fn sub_l3_enabled(&self) -> bool { self.registers.sub_l3_enabled.get() }
    fn sub_l2_enabled(&self) -> bool { self.registers.sub_l2_enabled.get() }
    fn sub_l1_enabled(&self) -> bool { self.registers.sub_l1_enabled.get() }
    fn main_obj_win_enabled(&self) -> bool { self.registers.main_obj_win_enabled.get() }
    fn main_l4_win_enabled(&self) -> bool { self.registers.main_l4_win_enabled.get() }
    fn main_l3_win_enabled(&self) -> bool { self.registers.main_l3_win_enabled.get() }
    fn main_l2_win_enabled(&self) -> bool { self.registers.main_l2_win_enabled.get() }
    fn main_l1_win_enabled(&self) -> bool { self.registers.main_l1_win_enabled.get() }
    fn sub_obj_win_enabled(&self) -> bool { self.registers.sub_obj_win_enabled.get() }
    fn sub_l4_win_enabled(&self) -> bool { self.registers.sub_l4_win_enabled.get() }
    fn sub_l3_win_enabled(&self) -> bool { self.registers.sub_l3_win_enabled.get() }
    fn sub_l2_win_enabled(&self) -> bool { self.registers.sub_l2_win_enabled.get() }
    fn sub_l1_win_enabled(&self) -> bool { self.registers.sub_l1_win_enabled.get() }
    fn main_col_win_black_region(&self) -> WindowColorRegion { self.registers.main_col_win_black_region.get() }
    fn sub_col_win_transparent_region(&self) -> WindowColorRegion { self.registers.sub_col_win_transparent_region.get() }
    fn cmath_addend(&self) -> CMathAddend { self.registers.cmath_addend.get() }
    fn direct_col_mode(&self) -> DirectColorMode { self.registers.direct_col_mode.get() }
    fn cmath_operator(&self) -> CMathOperator { self.registers.cmath_operator.get() }
    fn cmath_half(&self) -> bool { self.registers.cmath_half.get() }
    fn cmath_backdrop(&self) -> bool { self.registers.cmath_backdrop.get() }
    fn cmath_obj_enabled(&self) -> bool { self.registers.cmath_obj_enabled.get() }
    fn cmath_bg4_enabled(&self) -> bool { self.registers.cmath_bg4_enabled.get() }
    fn cmath_bg3_enabled(&self) -> bool { self.registers.cmath_bg3_enabled.get() }
    fn cmath_bg2_enabled(&self) -> bool { self.registers.cmath_bg2_enabled.get() }
    fn cmath_bg1_enabled(&self) -> bool { self.registers.cmath_bg1_enabled.get() }
    fn fixed_color(&self) -> u16 { self.registers.fixed_color.get() }
    fn _external_sync(&self) -> bool { self.registers._external_sync.get() }
    fn ext_bg_enabled(&self) -> bool { self.registers.ext_bg_enabled.get() }
    fn hi_res_enabled(&self) -> bool { self.registers.hi_res_enabled.get() }
    fn overscan_enabled(&self) -> bool { self.registers.overscan_enabled.get() }
    fn obj_interlace_enabled(&self) -> bool { self.registers.obj_interlace_enabled.get() }
    fn screen_interlace_enabled(&self) -> bool { self.registers.screen_interlace_enabled.get() }
    fn multiply_result(&self) -> u32 { self.registers.multiply_result.get() }
    fn vram_latch(&self) -> u16 { self.registers.vram_latch.get() }
    fn h_counter_toggle(&self) -> ToggleState { self.registers.h_counter_toggle.get() }
    fn h_counter_latch(&self) -> u16 { self.registers.h_counter_latch.get() }
    fn v_counter_toggle(&self) -> ToggleState { self.registers.v_counter_toggle.get() }
    fn v_counter_latch(&self) -> u16 { self.registers.v_counter_latch.get() }
    fn sprite_overflow(&self) -> bool { self.registers.sprite_overflow.get() }
    fn sprite_tile_overflow(&self) -> bool { self.registers.sprite_tile_overflow.get() }
    fn master_slave_state(&self) -> MasterSlave { self.registers.master_slave_state.get() }
    fn ppu1_version(&self) -> u8 { self.registers.ppu1_version.get() }
    fn interlace_field(&self) -> bool { self.registers.interlace_field.get() }
    fn counter_toggle(&self) -> ToggleState { self.registers.counter_toggle.get() }
    fn video_type(&self) -> VideoType { self.registers.video_type.get() }
    fn ppu2_version(&self) -> u8 { self.registers.ppu2_version.get() }
    fn in_vblank(&self) -> bool { self.registers.in_vblank.get() }
    fn in_hblank(&self) -> bool { self.registers.in_hblank.get() }
    fn in_fblank(&self) -> bool { self.registers.in_fblank.get() }
    fn h_counter(&self) -> u16 { self.registers.h_counter.get() }
    fn v_counter(&self) -> u16 { self.registers.v_counter.get() }

    fn vram_read(&self, address: u16) -> u16 { self.registers.vram.0[(address & 0x7FFF) as usize].get() }
}


pub fn dump_ppu_state(ppu: &Ppu5C7x) {
    println!("screen_brightness: {:?}", ppu.screen_brightness());
    println!("obj_sprite_size: {:?}", ppu.obj_sprite_size());
    println!("name_secondary_select: {:?}", ppu.name_secondary_select());
    println!("name_base_addr: {:?}", ppu.name_base_addr());
    println!("oam_addr: {:?}", ppu.oam_addr());
    println!("priority_rotation: {:?}", ppu.priority_rotation());
    println!("oam_data: {:?}", ppu.oam_data());
    println!("oam_data_latch: {:?}", ppu.oam_data_latch());
    println!("bg4_char_size: {:?}", ppu.bg4_char_size());
    println!("bg3_char_size: {:?}", ppu.bg3_char_size());
    println!("bg2_char_size: {:?}", ppu.bg2_char_size());
    println!("bg1_char_size: {:?}", ppu.bg1_char_size());
    println!("bg3_priority: {:?}", ppu.bg3_priority());
    println!("bg_mode: {:?}", ppu.bg_mode());
    println!("mosaic_size: {:?}", ppu.mosaic_size());
    println!("bg4_mosaic: {:?}", ppu.bg4_mosaic());
    println!("bg3_mosaic: {:?}", ppu.bg3_mosaic());
    println!("bg2_mosaic: {:?}", ppu.bg2_mosaic());
    println!("bg1_mosaic: {:?}", ppu.bg1_mosaic());
    println!("bg1_vram_addr: ${:04X}", ((ppu.bg1_vram_addr() as u16) << 10) & 0x7FFF);
    println!("bg1_tilemap_count_y: {:?}", ppu.bg1_tilemap_count_y());
    println!("bg1_tilemap_count_x: {:?}", ppu.bg1_tilemap_count_x());
    println!("bg2_vram_addr: {:?}", ppu.bg2_vram_addr());
    println!("bg2_tilemap_count_y: {:?}", ppu.bg2_tilemap_count_y());
    println!("bg2_tilemap_count_x: {:?}", ppu.bg2_tilemap_count_x());
    println!("bg3_vram_addr: {:?}", ppu.bg3_vram_addr());
    println!("bg3_tilemap_count_y: {:?}", ppu.bg3_tilemap_count_y());
    println!("bg3_tilemap_count_x: {:?}", ppu.bg3_tilemap_count_x());
    println!("bg4_vram_addr: {:?}", ppu.bg4_vram_addr());
    println!("bg4_tilemap_count_y: {:?}", ppu.bg4_tilemap_count_y());
    println!("bg4_tilemap_count_x: {:?}", ppu.bg4_tilemap_count_x());
    println!("bg2_chr_base_addr: {:?}", ppu.bg2_chr_base_addr());
    println!("bg1_chr_base_addr: {:?}", ppu.bg1_chr_base_addr());
    println!("bg4_chr_base_addr: {:?}", ppu.bg4_chr_base_addr());
    println!("bg3_chr_base_addr: {:?}", ppu.bg3_chr_base_addr());
    println!("m7_latch: {:?}", ppu.m7_latch());
    println!("bg_offset_latch: {:?}", ppu.bg_offset_latch());
    println!("bg_offset_x_latch: {:?}", ppu.bg_offset_x_latch());
    println!("bg1_m7_x_offset: {:?}", ppu.bg1_m7_x_offset());
    println!("bg1_m7_y_offset: {:?}", ppu.bg1_m7_y_offset());
    println!("bg2_x_offset: {:?}", ppu.bg2_x_offset());
    println!("bg2_y_offset: {:?}", ppu.bg2_y_offset());
    println!("bg3_x_offset: {:?}", ppu.bg3_x_offset());
    println!("bg3_y_offset: {:?}", ppu.bg3_y_offset());
    println!("bg4_x_offset: {:?}", ppu.bg4_x_offset());
    println!("bg4_y_offset: {:?}", ppu.bg4_y_offset());
    println!("vram_addr_inc_mode: {:?}", ppu.vram_addr_inc_mode());
    println!("addr_remap_mode: {:?}", ppu.addr_remap_mode());
    println!("addr_inc_size: {:?}", ppu.addr_inc_size());
    println!("vram_addr: ${:04X}", ppu.vram_addr());
    println!("vram_data: {:?}", ppu.vram_data());
    println!("m7_tilemap_repeat: {:?}", ppu.m7_tilemap_repeat());
    println!("m7_fill_mode: {:?}", ppu.m7_fill_mode());
    println!("m7_flip_bg_y: {:?}", ppu.m7_flip_bg_y());
    println!("m7_flip_bg_x: {:?}", ppu.m7_flip_bg_x());
    println!("m7_matrix_a: {:?}", ppu.m7_matrix_a());
    println!("m7_matrix_b: {:?}", ppu.m7_matrix_b());
    println!("m7_matrix_c: {:?}", ppu.m7_matrix_c());
    println!("m7_matrix_d: {:?}", ppu.m7_matrix_d());
    println!("m7_center_x: {:?}", ppu.m7_center_x());
    println!("m7_center_y: {:?}", ppu.m7_center_y());
    println!("cgram_toggle: {:?}", ppu.cgram_toggle());
    println!("cgram_addr: {:?}", ppu.cgram_addr());
    println!("cgram_latch: {:?}", ppu.cgram_latch());
    println!("cgram_data: {:?}", ppu.cgram_data());
    println!("bg2_w2_enabled: {:?}", ppu.bg2_w2_enabled());
    println!("bg2_w2_inverted: {:?}", ppu.bg2_w2_inverted());
    println!("bg2_w1_enabled: {:?}", ppu.bg2_w1_enabled());
    println!("bg2_w1_inverted: {:?}", ppu.bg2_w1_inverted());
    println!("bg1_w2_enabled: {:?}", ppu.bg1_w2_enabled());
    println!("bg1_w2_inverted: {:?}", ppu.bg1_w2_inverted());
    println!("bg1_w1_enabled: {:?}", ppu.bg1_w1_enabled());
    println!("bg1_w1_inverted: {:?}", ppu.bg1_w1_inverted());
    println!("bg4_w2_enabled: {:?}", ppu.bg4_w2_enabled());
    println!("bg4_w2_inverted: {:?}", ppu.bg4_w2_inverted());
    println!("bg4_w1_enabled: {:?}", ppu.bg4_w1_enabled());
    println!("bg4_w1_inverted: {:?}", ppu.bg4_w1_inverted());
    println!("bg3_w2_enabled: {:?}", ppu.bg3_w2_enabled());
    println!("bg3_w2_inverted: {:?}", ppu.bg3_w2_inverted());
    println!("bg3_w1_enabled: {:?}", ppu.bg3_w1_enabled());
    println!("bg3_w1_inverted: {:?}", ppu.bg3_w1_inverted());
    println!("col_w2_enabled: {:?}", ppu.col_w2_enabled());
    println!("col_w2_inverted: {:?}", ppu.col_w2_inverted());
    println!("col_w1_enabled: {:?}", ppu.col_w1_enabled());
    println!("col_w1_inverted: {:?}", ppu.col_w1_inverted());
    println!("obj_w2_enabled: {:?}", ppu.obj_w2_enabled());
    println!("obj_w2_inverted: {:?}", ppu.obj_w2_inverted());
    println!("obj_w1_enabled: {:?}", ppu.obj_w1_enabled());
    println!("obj_w1_inverted: {:?}", ppu.obj_w1_inverted());
    println!("w1_left_pos: {:?}", ppu.w1_left_pos());
    println!("w1_right_pos: {:?}", ppu.w1_right_pos());
    println!("w2_left_pos: {:?}", ppu.w2_left_pos());
    println!("w2_right_pos: {:?}", ppu.w2_right_pos());
    println!("bg4_win_logic: {:?}", ppu.bg4_win_logic());
    println!("bg3_win_logic: {:?}", ppu.bg3_win_logic());
    println!("bg2_win_logic: {:?}", ppu.bg2_win_logic());
    println!("bg1_win_logic: {:?}", ppu.bg1_win_logic());
    println!("obj_win_logic: {:?}", ppu.obj_win_logic());
    println!("col_win_logic: {:?}", ppu.col_win_logic());
    println!("main_obj_enabled: {:?}", ppu.main_obj_enabled());
    println!("main_l4_enabled: {:?}", ppu.main_l4_enabled());
    println!("main_l3_enabled: {:?}", ppu.main_l3_enabled());
    println!("main_l2_enabled: {:?}", ppu.main_l2_enabled());
    println!("main_l1_enabled: {:?}", ppu.main_l1_enabled());
    println!("sub_obj_enabled: {:?}", ppu.sub_obj_enabled());
    println!("sub_l4_enabled: {:?}", ppu.sub_l4_enabled());
    println!("sub_l3_enabled: {:?}", ppu.sub_l3_enabled());
    println!("sub_l2_enabled: {:?}", ppu.sub_l2_enabled());
    println!("sub_l1_enabled: {:?}", ppu.sub_l1_enabled());
    println!("main_obj_win_enabled: {:?}", ppu.main_obj_win_enabled());
    println!("main_l4_win_enabled: {:?}", ppu.main_l4_win_enabled());
    println!("main_l3_win_enabled: {:?}", ppu.main_l3_win_enabled());
    println!("main_l2_win_enabled: {:?}", ppu.main_l2_win_enabled());
    println!("main_l1_win_enabled: {:?}", ppu.main_l1_win_enabled());
    println!("sub_obj_win_enabled: {:?}", ppu.sub_obj_win_enabled());
    println!("sub_l4_win_enabled: {:?}", ppu.sub_l4_win_enabled());
    println!("sub_l3_win_enabled: {:?}", ppu.sub_l3_win_enabled());
    println!("sub_l2_win_enabled: {:?}", ppu.sub_l2_win_enabled());
    println!("sub_l1_win_enabled: {:?}", ppu.sub_l1_win_enabled());
    println!("main_col_win_black_region: {:?}", ppu.main_col_win_black_region());
    println!("sub_col_win_transparent_region: {:?}", ppu.sub_col_win_transparent_region());
    println!("cmath_addend: {:?}", ppu.cmath_addend());
    println!("direct_col_mode: {:?}", ppu.direct_col_mode());
    println!("cmath_operator: {:?}", ppu.cmath_operator());
    println!("cmath_half: {:?}", ppu.cmath_half());
    println!("cmath_backdrop: {:?}", ppu.cmath_backdrop());
    println!("cmath_obj_enabled: {:?}", ppu.cmath_obj_enabled());
    println!("cmath_bg4_enabled: {:?}", ppu.cmath_bg4_enabled());
    println!("cmath_bg3_enabled: {:?}", ppu.cmath_bg3_enabled());
    println!("cmath_bg2_enabled: {:?}", ppu.cmath_bg2_enabled());
    println!("cmath_bg1_enabled: {:?}", ppu.cmath_bg1_enabled());
    println!("fixed_color: {:?}", ppu.fixed_color());
    println!("_external_sync: {:?}", ppu._external_sync());
    println!("ext_bg_enabled: {:?}", ppu.ext_bg_enabled());
    println!("hi_res_enabled: {:?}", ppu.hi_res_enabled());
    println!("overscan_enabled: {:?}", ppu.overscan_enabled());
    println!("obj_interlace_enabled: {:?}", ppu.obj_interlace_enabled());
    println!("screen_interlace_enabled: {:?}", ppu.screen_interlace_enabled());
    println!("multiply_result: {:?}", ppu.multiply_result());
    println!("vram_latch: {:?}", ppu.vram_latch());
    println!("h_counter_toggle: {:?}", ppu.h_counter_toggle());
    println!("h_counter_latch: {:?}", ppu.h_counter_latch());
    println!("v_counter_toggle: {:?}", ppu.v_counter_toggle());
    println!("v_counter_latch: {:?}", ppu.v_counter_latch());
    println!("sprite_overflow: {:?}", ppu.sprite_overflow());
    println!("sprite_tile_overflow: {:?}", ppu.sprite_tile_overflow());
    println!("master_slave_state: {:?}", ppu.master_slave_state());
    println!("ppu1_version: {:?}", ppu.ppu1_version());
    println!("interlace_field: {:?}", ppu.interlace_field());
    println!("counter_toggle: {:?}", ppu.counter_toggle());
    println!("video_type: {:?}", ppu.video_type());
    println!("ppu2_version: {:?}", ppu.ppu2_version());
    println!("in_vblank: {:?}", ppu.in_vblank());
    println!("in_hblank: {:?}", ppu.in_hblank());
    println!("in_fblank: {:?}", ppu.in_fblank());
    println!("h_counter: {:?}", ppu.h_counter());
    println!("v_counter: {:?}", ppu.v_counter());
}