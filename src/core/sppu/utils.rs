macro_rules! win_active_signals {
    ($ppu:ident, $bus:ident, $bg:ident) => {
        paste!( {
            let win_en = if $bus.ppu_regs.[<$bg _win_main_en>] || $bus.ppu_regs.[<$bg _win_sub_en>] {
                Ppu5C7x::win_active_signal(
                    $bus.ppu_regs,
                    $ppu.x,
                    $bus.ppu_regs.[<$bg _w1_en>],
                    $bus.ppu_regs.[<$bg _w2_en>],
                    $bus.ppu_regs.[<$bg _w1_inv>],
                    $bus.ppu_regs.[<$bg _w2_inv>],
                    $bus.ppu_regs.[<$bg _win_logic>],
                )
            } else {
                false
            };

            let [<$bg _win_main_en>] = win_en && $bus.ppu_regs.[<$bg _win_main_en>];
            let [<$bg _win_sub_en>] = win_en && $bus.ppu_regs.[<$bg _win_sub_en>];

            ([<$bg _win_main_en>], [<$bg _win_sub_en>])
        } )
    };
}

macro_rules! _bg_colors {
    ($ppu:ident, $bus:ident, $col_depth:expr, $cgram_base_addr:expr, $bg_name:ident, $bg_layer:expr) => {
        paste!( {
            let ([<$bg_name _win_main>], [<$bg_name _win_sub>]) = win_active_signals!($ppu, $bus, $bg_name);

            let mut bg_main_col = None;
            let mut bg_sub_col = None;

            if $bus.ppu_regs.[<$bg_name _main_en>] && ![<$bg_name _win_main>] {
                bg_main_col = Some($ppu.bg_col(
                    $bus,
                    $bg_layer, $col_depth,
                    $cgram_base_addr
                ));
            }

            if $bus.ppu_regs.[<$bg_name _sub_en>] && ![<$bg_name _win_sub>] {
                bg_sub_col = Some(bg_main_col.unwrap_or($ppu.bg_col(
                    $bus,
                    $bg_layer, $col_depth,
                    $cgram_base_addr
                )));
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
        let (obj_win_main, obj_win_sub) = win_active_signals!($ppu, $bus, obj);

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