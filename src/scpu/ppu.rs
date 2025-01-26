use std::cell::Cell;

trait GetBits {
    fn get_bit(self, bit: Self) -> Self;
    fn bit_en(self, bit: Self) -> bool;
}

impl GetBits for u8 {
    fn get_bit(self, bit: Self) -> Self { (self >> bit) & 1 }
    fn bit_en(self, bit: Self) -> bool { (self >> bit) & 1 != 0 }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ToggleState {
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

#[derive(Clone, Copy)]
enum ObjectSize {
    Size8x8_16x16,
    Size8x8_32x32,
    Size8x8_64x64,
    Size16x16_32x32,
    Size16x16_64x64,
    Size32x32_64x64,
    Size16x32_32x64,
    Size16x32_32x32,
}

#[derive(Clone, Copy)]
enum CharSize {
    Small,
    Large,
}

#[derive(Clone, Copy)]
enum BgPriority {
    High,
    Low,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
enum TilemapCount {
    One,
    Two,
}

#[derive(Clone, Copy, PartialEq)]
enum VramIncMode {
    LowByte,
    HighByte
}

#[derive(Clone, Copy)]
enum AddressRemapping {
    None,
    ColDepth2,
    ColDepth4,
    ColDepth8,
}

#[derive(Clone, Copy)]
enum IncrSize {
    Bytes2,
    Bytes64,
    Bytes256,
}

#[derive(Clone, Copy)]
enum M7FillMode {
    Transparent,
    Character,
}

#[derive(Clone, Copy)]
enum WindowLogic {
    Or,
    And,
    Xor,
    Xnor,
}

#[derive(Clone, Copy)]
enum WindowColorRegion {
    Nowhere,
    Outside,
    Inside,
    Everywhere,
}

#[derive(Clone, Copy)]
enum CMathAddend {
    Fixed,
    Subscreen,
}

#[derive(Clone, Copy)]
enum DirectColorMode {
    Palette,
    Direct,
}

#[derive(Clone, Copy)]
enum CMathOperator {
    Add,
    Subtract,
}

#[derive(Clone, Copy)]
enum MasterSlave {
    Master,
    Slave,
}

#[derive(Clone, Copy)]
enum VideoType {
    Ntsc,
    Pal,
}


pub struct PpuData {
    // $2100    F... BBBB    Write only
    //       - Forced blanking (F)
    //       - Screen brightness (B)
    forced_blanking: Cell<bool>,
    screen_brightness: Cell<u8>,

    // $2101    SSSN NbBB    Write only    
    //       - OBJ sprite size (S)
    //       - Name secondary select (N)
    //       - Name base address (B)
    obj_sprite_size: Cell<ObjectSize>,
    name_secondary_select: Cell<u8>,
    name_base_addr: Cell<u8>,

    // $2102    AAAA AAAA
    // $2103    P... ...B    Write x2 Only
    //       - OAM word address (A)
    //       - Priority rotation (P)
    //       - Address high bit / table select (B)
    oam_addr: Cell<u16>,
    priority_rotation: Cell<bool>,

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
    bg2_char_base_addr: Cell<u8>,
    bg1_char_base_addr: Cell<u8>,

    // $210C    DDDD CCCC    W8
    //       - BG4 CHR base address (D)
    //       - BG3 CHR base address (C)
    bg4_char_base_addr: Cell<u8>,
    bg3_char_base_addr: Cell<u8>,

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
    vram_addr: Cell<u16>,

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
    m7_matrix_a_16bit_factor: Cell<u16>,

    // $211C    DDDD DDDD LLLL LLLL    Write Only
    //       - Mode 7 matrix B or signed 8-bit multiplication factor (D)
    //       - mode7_data_latch (L), writing sets new data latch to (D)
    m7_matrix_b_8bit_factor: Cell<u16>,

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
    w4_logic: Cell<WindowLogic>,
    w3_logic: Cell<WindowLogic>,
    w2_logic: Cell<WindowLogic>,
    w1_logic: Cell<WindowLogic>,
    
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
    oam: [Cell<u8>; 0x220], // 544 Bytes of OAM
    vram: [Cell<u8>; 64 * 1024], // 64 KiB of VRAM
    cgram: [Cell<u16>; 256],

    // PPU State
    in_vblank: Cell<bool>,
    in_hblank: Cell<bool>,
    in_fblank: Cell<bool>,
    h_counter: Cell<u16>,
    v_counter: Cell<u16>,
}

// CPU Access
impl PpuData {
    pub fn write(&self, address: u8, data: u8) {
        match address {
            0x00 => {
                self.forced_blanking.replace(data.bit_en(7));
                self.screen_brightness.replace(data & 0x0F);
            }

            0x01 => {
                let new_obj_size = match data >> 5 {
                    0 => ObjectSize::Size8x8_16x16,
                    1 => ObjectSize::Size8x8_32x32,
                    2 => ObjectSize::Size8x8_64x64,
                    3 => ObjectSize::Size16x16_32x32,
                    4 => ObjectSize::Size16x16_64x64,
                    5 => ObjectSize::Size32x32_64x64,
                    6 => ObjectSize::Size16x32_32x64,
                    7 => ObjectSize::Size16x32_32x32,
                    _ => unreachable!()
                };

                self.obj_sprite_size.replace(new_obj_size);
                self.name_secondary_select.replace((data >> 3) & 0x03);
                self.name_base_addr.replace(data & 0x03);
            }

            0x02 => {
                let new_addr = self.oam_addr.get() & 0x0200 | ((data as u16) << 1);

                self.oam_addr.replace(new_addr);
            }

            0x03 => {
                let new_addr = self.oam_addr.get() & 0x01FE | (((data & 0x01) as u16) << 9);

                self.oam_addr.replace(new_addr);
                self.priority_rotation.replace(data.bit_en(7));
            }

            0x04 => {
                let oam_addr = self.oam_addr.get() as usize;

                if oam_addr & 1 == 0 {
                    self.oam_data_latch.replace(data);
                } else if oam_addr < 0x200 {
                    self.oam[oam_addr - 1].replace(self.oam_data_latch.get());
                    self.oam[oam_addr].replace(data);
                }

                if oam_addr >= 0x200 {
                    self.oam[oam_addr].replace(data);
                }

                self.oam_addr.replace(oam_addr as u16 + 1);
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
            }

            0x06 => {
                self.mosaic_size.replace(data >> 4);
                self.bg4_mosaic.replace(data.bit_en(3));
                self.bg3_mosaic.replace(data.bit_en(2));
                self.bg2_mosaic.replace(data.bit_en(1));
                self.bg1_mosaic.replace(data.bit_en(0));
            }

            0x07 => {
                self.bg1_vram_addr.replace(data >> 2);
                self.bg1_tilemap_count_y.replace(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg1_tilemap_count_x.replace(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
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
            }

            0x09 => {
                self.bg3_vram_addr.replace(data >> 2);
                self.bg3_tilemap_count_y.replace(
                    if data.bit_en(1) { TilemapCount::Two } else { TilemapCount::One }
                );
                self.bg3_tilemap_count_x.replace(
                    if data.bit_en(0) { TilemapCount::Two } else { TilemapCount::One }
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
            }

            0x0B => {
                self.bg2_char_base_addr.replace(data >> 4);
                self.bg1_char_base_addr.replace(data & 0x0F);
            }

            0x0C => {
                self.bg4_char_base_addr.replace(data >> 4);
                self.bg3_char_base_addr.replace(data & 0x0F);
            }

            0x0D => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg1_m7_x_offset.replace(
                    ((data as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );
            }

            0x0E => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg1_m7_y_offset.replace(((data as u16) << 8) | bgofs_latch);
            }

            0x0F => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg2_x_offset.replace(
                    ((data as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );
            }

            0x10 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg2_y_offset.replace(((data as u16) << 8) | bgofs_latch);
            }

            0x11 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg3_x_offset.replace(
                    ((data as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );
            }

            0x12 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg3_y_offset.replace(((data as u16) << 8) | bgofs_latch);
            }

            0x13 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;
                let bghofs_latch = self.bg_offset_x_latch.replace(data) as u16;

                self.bg4_x_offset.replace(
                    ((data as u16) << 8) | (bgofs_latch & 0x00F8) | (bghofs_latch & 0x07)
                );
            }

            0x14 => {
                let bgofs_latch = self.bg_offset_latch.replace(data) as u16;

                self.bg4_y_offset.replace(((data as u16) << 8) | bgofs_latch);
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
            }

            0x16 => {
                self.vram_addr.replace(
                    (self.vram_addr.get() & 0xFF00) | (data as u16)
                );
                self.vram_latch.replace(
                    u16::from_le_bytes([
                        self.vram[self.vram_addr.get() as usize].get(),
                        self.vram[(self.vram_addr.get() + 1) as usize].get()
                    ])
                );
            }

            0x17 => {
                self.vram_addr.replace(
                    (self.vram_addr.get() & 0x00FF) | ((data as u16) << 8)
                );
                self.vram_latch.replace(
                    u16::from_le_bytes([
                        self.vram[self.vram_addr.get() as usize].get(),
                        self.vram[(self.vram_addr.get() + 1) as usize].get()
                    ])
                );
            }

            0x18 => {
                if self.in_fblank.get() || self.in_vblank.get() {
                    let addr = self.get_vram_addr();
                    self.vram[addr + 1].replace(data);
                }
                    
                if self.vram_addr_inc_mode.get() == VramIncMode::LowByte {
                    self.inc_vram_addr();
                }
            }

            0x19 => {
                if self.in_fblank.get() || self.in_vblank.get() {
                    let addr = self.get_vram_addr();
                    self.vram[addr].replace(data);
                }
                    
                if self.vram_addr_inc_mode.get() == VramIncMode::HighByte {
                    self.inc_vram_addr();
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

                self.m7_matrix_a_16bit_factor.replace(
                    ((data as u16) << 8) | latched_val
                );

                self.update_multiply_result();
            }

            0x1C => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_b_8bit_factor.replace(
                    ((data as u16) << 8) | latched_val
                );

                self.update_multiply_result();
            }

            0x1D => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_c.replace(((data as u16) << 8) | latched_val);
            }

            0x1E => {
                let latched_val = self.m7_latch.replace(data) as u16;

                self.m7_matrix_d.replace(((data as u16) << 8) | latched_val);
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
                self.cgram_toggle.toggle();
            }

            0x22 => {
                if self.cgram_toggle.toggle() {
                    let addr = self.cgram_addr.get() as usize;
                    let new_col = ((data as u16) << 8) | self.cgram_latch.get() as u16;

                    self.cgram[addr].replace(new_col);

                    self.cgram_addr.replace((addr as u8) + 1);
                } else {
                    self.cgram_latch.replace(data);
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
                self.w4_logic.replace(
                    match data >> 6 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.w3_logic.replace(
                    match (data >> 4) & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.w2_logic.replace(
                    match (data >> 2) & 3 {
                        0 => WindowLogic::Or,
                        1 => WindowLogic::And,
                        2 => WindowLogic::Xor,
                        3 => WindowLogic::Xnor,
                        _ => unreachable!(),
                    }
                );
                self.w1_logic.replace(
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

    fn read(&self, address: u8) -> u8 {
        let data = match address {
            0x34 => { self.multiply_result.get() as u8 }
            0x35 => { (self.multiply_result.get() >> 8) as u8 }
            0x36 => { (self.multiply_result.get() >> 16) as u8 }

            0x37 => {
                // When counter_latch transitions from 0 to 1
                if !self.counter_toggle.is_high() {
                    self.h_counter_latch.replace(self.h_counter.get());
                    self.v_counter_latch.replace(self.v_counter.get());
                }

                self.counter_toggle.set_hi();

                0 // CPU OPEN BUS
            }

            0x38 => {
                let addr = self.oam_addr.replace(self.oam_addr.get() + 1);

                self.oam[addr as usize].get()
            }

            0x39 => {
                let val = self.vram_latch.get() as u8;

                if self.vram_addr_inc_mode.get() == VramIncMode::LowByte {
                    self.vram_latch.replace(
                        u16::from_le_bytes([
                            self.vram[self.vram_addr.get() as usize].get(),
                            self.vram[(self.vram_addr.get() + 1) as usize].get()
                        ])
                    );
                    self.inc_vram_addr();
                }

                val
            }

            0x3A => {
                let val = (self.vram_latch.get() >> 8) as u8;

                if self.vram_addr_inc_mode.get() == VramIncMode::HighByte {
                    self.vram_latch.replace(
                        u16::from_le_bytes([
                            self.vram[self.vram_addr.get() as usize].get(),
                            self.vram[(self.vram_addr.get() + 1) as usize].get()
                        ])
                    );
                    self.inc_vram_addr();
                }

                val
            }

            0x3B => {
                if self.cgram_toggle.toggle() {
                    (self.cgram[self.cgram_addr.get() as usize].get() >> 8) as u8 // HIGH BIT IS PPU2 OPEN BUS
                } else {
                    self.cgram[self.cgram_addr.get() as usize].get() as u8
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
        let lhs = self.m7_matrix_a_16bit_factor.get() as i16;
        let rhs = self.m7_matrix_b_8bit_factor.get() as i8;
        let result = ((lhs as i32) * (rhs as i32)) as u32;

        self.multiply_result.replace(result & 0x00FFFFFF);
    }

    fn get_vram_addr(&self) -> usize {
        match self.addr_remap_mode.get() {
            AddressRemapping::None => { self.vram_addr.get() as usize },
            AddressRemapping::ColDepth2 => {
                // rrrrrrrr YYYccccc -> rrrrrrrr cccccYYY
                let addr = self.vram_addr.get();

                let r = (addr & 0xFF00) as usize;
                let y = ((addr & 0x00E0) >> 5) as usize;
                let c = ((addr & 0x1F) << 3) as usize;

                r | c | y
            }
            AddressRemapping::ColDepth4 => {
                // rrrrrrrY YYcccccP -> rrrrrrrc ccccPYYY
                let addr = self.vram_addr.get();

                let r = (addr & 0xFE00) as usize;
                let y = ((addr & 0x01C0) >> 6) as usize;
                let cp = ((addr & 0x003F) << 3) as usize;

                r | cp | y
            }
            AddressRemapping::ColDepth8 => {
                // rrrrrrYY YcccccPP -> rrrrrrcc cccPPYYY
                let addr = self.vram_addr.get();

                let r = (addr & 0xFC00) as usize;
                let y = ((addr & 0x0380) >> 7) as usize;
                let cp = ((addr & 0x007F) << 3) as usize;

                r | cp | y
            }
        }
    }

    fn inc_vram_addr(&self) {
        let inc = match self.addr_inc_size.get() {
            IncrSize::Bytes2 => 2,
            IncrSize::Bytes64 => 64,
            IncrSize::Bytes256 => 256,
        };

        self.vram_addr.replace(self.vram_addr.get() + inc);
    }
}

// PPU Internal Access
impl PpuData {

}

struct Ppu<'a> {
    registers: &'a PpuData,
}

impl Ppu<'_> {
    
}