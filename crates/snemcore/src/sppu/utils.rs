macro_rules! win_active_signals {
    ($ppu:ident, $bus:ident, $win_settings:expr) => {
        paste!( {
            let win_en = if $win_settings.win_main_en || $win_settings.win_sub_en {
                Ppu5C7x::win_active_signal(
                    $bus.ppu_regs,
                    $ppu.x,
                    $win_settings,
                )
            } else {
                false
            };

            let win_main_en = win_en && $win_settings.win_main_en;
            let win_sub_en = win_en && $win_settings.win_sub_en;

            (win_main_en, win_sub_en)
        } )
    };
}

macro_rules! _bg_colors {
    ($ppu:ident, $bus:expr, $col_depth:expr, $cgram_base_addr:expr, $bg_name:ident, $bg_layer:expr) => {
        paste!( {
            let bg_data = Self::fetch_bg_data($bus.ppu_regs, $bg_layer);
            
            let (win_main, win_sub) = win_active_signals!($ppu, $bus, &bg_data.window);

            let mut bg_main_col = None;
            let mut bg_sub_col = None;

            if $bus.ppu_regs.[<$bg_name _main_en>] && !win_main {
                bg_main_col = Some($ppu.bg_col(
                    $bus.ppu_regs,
                    &$bus.vram[..],
                    &$bus.cgram[..],
                    $bg_layer, $col_depth,
                    $cgram_base_addr
                ));
            }

            if $bus.ppu_regs.[<$bg_name _sub_en>] && !win_sub {
                bg_sub_col = Some(match bg_main_col {
                        Some(c) => c,
                        None => $ppu.bg_col(
                            $bus.ppu_regs,
                            &$bus.vram[..],
                            &$bus.cgram[..],
                            $bg_layer, $col_depth,
                            $cgram_base_addr
                        ),
                    });
            }

            let bg_main_col = bg_main_col.unwrap_or($ppu.transparent_color_data($bus));
            let bg_sub_col = bg_sub_col.unwrap_or($ppu.transparent_color_data($bus));

            (bg_main_col, bg_sub_col)
        } )
    };
}

macro_rules! layer_colors {
    ($ppu:ident, $bus:ident, $col_depth:expr, $cgram_base_addr:expr, ColorLayer::Bg1) => {
        _bg_colors!(
            $ppu,
            $bus,
            $col_depth,
            $cgram_base_addr,
            bg1,
            ColorLayer::Bg1
        )
    };
    ($ppu:ident, $bus:ident, $col_depth:expr, $cgram_base_addr:expr, ColorLayer::Bg2) => {
        _bg_colors!(
            $ppu,
            $bus,
            $col_depth,
            $cgram_base_addr,
            bg2,
            ColorLayer::Bg2
        )
    };
    ($ppu:ident, $bus:ident, $col_depth:expr, $cgram_base_addr:expr, ColorLayer::Bg3) => {
        _bg_colors!(
            $ppu,
            $bus,
            $col_depth,
            $cgram_base_addr,
            bg3,
            ColorLayer::Bg3
        )
    };
    ($ppu:ident, $bus:ident, $col_depth:expr, $cgram_base_addr:expr, ColorLayer::Bg4) => {
        _bg_colors!(
            $ppu,
            $bus,
            $col_depth,
            $cgram_base_addr,
            bg4,
            ColorLayer::Bg4
        )
    };
    ($ppu:ident, $bus:ident, ColorLayer::Obj) => {{
        let (obj_win_main, obj_win_sub) = win_active_signals!($ppu, $bus, &$bus.ppu_regs.col_win);

        let mut obj_main_col = None;
        let mut obj_sub_col = None;

        if $bus.ppu_regs.obj_main_en && !obj_win_main {
            obj_main_col = Some($ppu.sprite_col($bus));
        }

        if $bus.ppu_regs.obj_sub_en && !obj_win_sub {
            obj_sub_col = Some(obj_main_col.unwrap_or($ppu.sprite_col($bus)));
        }

        let obj_main_col = obj_main_col.unwrap_or($ppu.transparent_color_data($bus));
        let obj_sub_col = obj_sub_col.unwrap_or($ppu.transparent_color_data($bus));

        (obj_main_col, obj_sub_col)
    }};
}

// Generate the 256-element Morton lookup table at compile time.
const fn generate_morton_lookup() -> [u16; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut z = 0;
        let mut bit = 0;
        while bit < 8 {
            z |= (i & (1 << bit)) << bit;
            bit += 1;
        }
        table[i as usize] = z;
        i += 1;
    }
    table
}

const MORTON_LOOKUP: [u16; 256] = generate_morton_lookup();

pub fn interleave_2bpp(bp10: u16) -> u16 {
    let bp1 = (bp10 >> 8) as u8;
    let bp0 = bp10 as u8;
    fast_interleave_u8(bp0 , bp1)
}

pub fn interleave_4bpp(bp10: u16, bp32: u16) -> u32 {
    let bp0 = bp10 as u8;         // 00000000
    let bp1 = (bp10 >> 8) as u8;  // 11111111
    let bp2 = bp32 as u8;         // 22222222
    let bp3 = (bp32 >> 8) as u8;  // 33333333
    
    // interleaved bitplanes
    let ibp20 = fast_interleave_u8(bp0, bp2); // 20202020 20202020
    let ibp31 = fast_interleave_u8(bp1, bp3); // 31313131 31313131
    
    // 32103210 32103210 32103210 32103210
    fast_interleave_u16(ibp20, ibp31)
}

pub fn interleave_8bpp(bp10: u16, bp32: u16, bp54: u16, bp76: u16) -> u64 {
    let bp0 = bp10 as u8;         // 00000000
    let bp1 = (bp10 >> 8) as u8;  // 11111111
    let bp2 = bp32 as u8;         // 22222222
    let bp3 = (bp32 >> 8) as u8;  // 33333333
    let bp4 = bp54 as u8;         // 44444444
    let bp5 = (bp54 >> 8) as u8;  // 55555555
    let bp6 = bp76 as u8;         // 66666666
    let bp7 = (bp76 >> 8) as u8;  // 77777777
    
    // interleaved bitplanes
    let ibp73 = fast_interleave_u8(bp3, bp7); // 73737373 73737373
    let ibp62 = fast_interleave_u8(bp2, bp6); // 62626262 62626262
    let ibp51 = fast_interleave_u8(bp1, bp5); // 51515151 51515151
    let ibp40 = fast_interleave_u8(bp0, bp4); // 40404040 40404040
    
    let ibp7531 = fast_interleave_u16(ibp51, ibp73); // 75317531 75317531 75317531 75317531
    let ibp6420 = fast_interleave_u16(ibp40, ibp62); // 64206420 64206420 64206420 64206420
    
    // 76543210 76543210 ... 76543210 76543210
    fast_interleave_u32(ibp6420, ibp7531)
}

/// Interleaves the bits of x and y using the fast lookup table.
/// The output will be (y31 x31 ... y1 x1 y0 x0)
fn fast_interleave_u32(x: u32, y: u32) -> u64 {
    let xhi = (x >> 16) as u16;
    let xlo = x as u16;
    let yhi = (y >> 16) as u16;
    let ylo = y as u16;
    
    let ihi = fast_interleave_u16(xhi, yhi) as u64;
    let ilo = fast_interleave_u16(xlo, ylo) as u64;

    (ihi << 32) | ilo
}

/// Interleaves the bits of x and y using the fast lookup table.
/// The output will be (y15 x15 ... y1 x1 y0 x0)
fn fast_interleave_u16(x: u16, y: u16) -> u32 {
    let xhi = (x >> 8) as u8;
    let xlo = x as u8;
    let yhi = (y >> 8) as u8;
    let ylo = y as u8;
    
    let ihi = fast_interleave_u8(xhi, yhi) as u32;
    let ilo = fast_interleave_u8(xlo, ylo) as u32;
    
    (ihi << 16) | ilo
}

/// Interleaves the bits of x and y using the fast lookup table.
/// The output will be (y7 x7 ... y1 x1 y0 x0)
fn fast_interleave_u8(x: u8, y: u8) -> u16 {
    (MORTON_LOOKUP[y as usize]) << 1 | MORTON_LOOKUP[x as usize]
}