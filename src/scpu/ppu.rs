use std::cell::Cell;

enum CharSize {
    Small,
    Large,
}

enum TilemapCount {
    One,
    Two,
}

enum VramIncMode {
    LowByte,
    HighByte
}

enum AddressRemapping {
    None,
    ColDepth2,
    ColDepth4,
    ColDepth8,
}

enum IncrSize {
    Size1,
    Size32,
    Size128,
}

struct PpuRegs {
    registers: [Cell<u8>; 0x40],
    bg1_m7_horizontal_scroll: Cell<u16>,
    bg1_m7_vertical_scroll: Cell<u16>,
    bg2_horizontal_scroll: Cell<u16>,
    bg2_vertical_scroll: Cell<u16>,
    bg3_horizontal_scroll: Cell<u16>,
    bg3_vertical_scroll: Cell<u16>,
    bg4_horizontal_scroll: Cell<u16>,
    bg4_vertical_scroll: Cell<u16>,
    vram_latch: Cell<u16>,
}

impl PpuRegs {
    fn get_forced_blanking_enabled(&self) -> bool {
        self.registers[0].get() & 0x80 != 0
    }

    fn get_screen_brightness(&self) -> u8 {
        self.registers[0].get() & 0x0F
    }

    fn get_obj_sprite_size(&self) -> u8 {
        self.registers[1].get() >> 5
    }

    fn get_name_secondary_select(&self) -> u8 {
        (self.registers[1].get() & 0x18) >> 3
    }

    fn get_name_base_address(&self) -> u8 {
        self.registers[1].get() & 0x07
    }

    // NOTE: comment out OAM internal registers that we suspect don't need get/setters
    // fn get_oam_word_address(&self) -> u8 {
    //     self.registers[2].get()
    // }

    fn get_priority_rotation_enabled(&self) -> bool {
        self.registers[3].get() & 0x80 != 0
    }

    fn get_address_high_bit(&self) -> u8 {
        self.registers[3].get() & 0x01
    }

    // fn get_oam_data(&self) -> u8 {
    //     
    // }

    fn get_bg4_char_size(&self) -> CharSize {
        if self.registers[0x05].get() & 0x80 != 0 {
            CharSize::Large
        } else {
            CharSize::Small
        }
    }

    fn get_bg3_char_size(&self) -> CharSize {
        if self.registers[0x05].get() & 0x40 != 0 {
            CharSize::Large
        } else {
            CharSize::Small
        }
    }

    fn get_bg2_char_size(&self) -> CharSize {
        if self.registers[0x05].get() & 0x20 != 0 {
            CharSize::Large
        } else {
            CharSize::Small
        }
    }

    fn get_bg1_char_size(&self) -> CharSize {
        if self.registers[0x05].get() & 0x10 != 0 {
            CharSize::Large
        } else {
            CharSize::Small
        }
    }

    fn get_bg3_priority_high(&self) -> bool {
        self.registers[0x05].get() & 0x08 != 0
    }

    fn get_background_mode(&self) -> u8 {
        self.registers[0x05].get() & 0x07
    }

    fn get_mosaic_size(&self) -> u8 {
        self.registers[0x06].get() >> 4
    }

    fn get_bg4_mosaic_enabled(&self) -> bool {
        self.registers[0x06].get() & 0x08 != 0
    }

    fn get_bg3_mosaic_enabled(&self) -> bool {
        self.registers[0x06].get() & 0x04 != 0
    }

    fn get_bg2_mosaic_enabled(&self) -> bool {
        self.registers[0x06].get() & 0x02 != 0
    }

    fn get_bg1_mosaic_enabled(&self) -> bool {
        self.registers[0x06].get() & 0x01 != 0
    }

    fn get_bg1_tilemap_vram_address(&self) -> u8 {
        self.registers[0x07].get() >> 2
    }

    fn get_bg1_vertical_tilemap_count(&self) -> TilemapCount {
        if self.registers[0x07].get() & 0x02 != 0 {
            TilemapCount::Two
        } else {
            TilemapCount::One
        }
    }

    fn get_bg1_horizontal_tilemap_count(&self) -> TilemapCount {
        if self.registers[0x07].get() & 0x01 != 0 {
            TilemapCount::Two
        } else {
            TilemapCount::One
        }
    }

    fn get_bg2_tilemap_vram_address(&self) -> u8 {
        self.registers[0x08].get() >> 2
    }

    fn get_bg2_vertical_tilemap_count(&self) -> TilemapCount {
        if self.registers[0x08].get() & 0x02 != 0 {
            TilemapCount::Two
        } else {
            TilemapCount::One
        }
    }

    fn get_bg2_horizontal_tilemap_count(&self) -> TilemapCount {
        if self.registers[0x08].get() & 0x01 != 0 {
            TilemapCount::Two
        } else {
            TilemapCount::One
        }
    }

    fn get_bg3_tilemap_vram_address(&self) -> u8 {
        self.registers[0x09].get() >> 2
    }

    fn get_bg3_vertical_tilemap_count(&self) -> TilemapCount {
        if self.registers[0x09].get() & 0x02 != 0 {
            TilemapCount::Two
        } else {
            TilemapCount::One
        }
    }

    fn get_bg3_horizontal_tilemap_count(&self) -> TilemapCount {
        if self.registers[0x09].get() & 0x01 != 0 {
            TilemapCount::Two
        } else {
            TilemapCount::One
        }
    }

    fn get_bg4_tilemap_vram_address(&self) -> u8 {
        self.registers[0x0A].get() >> 2
    }

    fn get_bg4_vertical_tilemap_count(&self) -> TilemapCount {
        if self.registers[0x0A].get() & 0x02 != 0 {
            TilemapCount::Two
        } else {
            TilemapCount::One
        }
    }

    fn get_bg4_horizontal_tilemap_count(&self) -> TilemapCount {
        if self.registers[0x0A].get() & 0x01 != 0 {
            TilemapCount::Two
        } else {
            TilemapCount::One
        }
    }

    fn get_bg2_chr_base_address(&self) -> u8 {
        self.registers[0x0B].get() >> 4
    }

    fn get_bg1_chr_base_address(&self) -> u8 {
        self.registers[0x0B].get() & 0x07
    }

    fn get_bg4_chr_base_address(&self) -> u8 {
        self.registers[0x0C].get() >> 4
    }

    fn get_bg3_chr_base_address(&self) -> u8 {
        self.registers[0x0C].get() & 0x07
    }
    
    fn get_bg1_horizontal_scroll(&self) -> u16 {
        self.bg1_m7_horizontal_scroll.get() & 0x03FF
    }
    fn get_mode7_horizontal_scroll(&self) -> u16 {
        self.bg1_m7_horizontal_scroll.get() & 0x1FFF
    }

    fn get_bg1_vertical_scroll(&self) -> u16 {
        self.bg1_m7_vertical_scroll.get() & 0x03FF
    }
    fn get_mode7_vertical_scroll(&self) -> u16 {
        self.bg1_m7_vertical_scroll.get() & 0x1FFF
    }

    fn get_bg2_horizontal_scroll(&self) -> u16 {
        self.bg2_horizontal_scroll.get() & 0x03FF
    }
    fn get_bg2_vertical_scroll(&self) -> u16 {
        self.bg2_vertical_scroll.get() & 0x03FF
    }

    fn get_bg3_horizontal_scroll(&self) -> u16 {
        self.bg3_horizontal_scroll.get() & 0x03FF
    }
    fn get_bg3_vertical_scroll(&self) -> u16 {
        self.bg3_vertical_scroll.get() & 0x03FF
    }

    fn get_bg4_horizontal_scroll(&self) -> u16 {
        self.bg4_horizontal_scroll.get() & 0x03FF
    }
    fn get_bg4_vertical_scroll(&self) -> u16 {
        self.bg4_vertical_scroll.get() & 0x03FF
    }

    fn get_vram_inc_mode(&self) -> VramIncMode {
        if self.registers[0x15].get() & 0x80 != 0 {
            VramIncMode::HighByte
        } else {
            VramIncMode::LowByte
        }
    }

    fn get_address_remapping(&self) -> AddressRemapping {
        match (self.registers[0x15].get() & 0x0C) >> 2 {
            1 => AddressRemapping::ColDepth2,
            2 => AddressRemapping::ColDepth4,
            3 => AddressRemapping::ColDepth8,
            _ => AddressRemapping::None,
        }
    }

    fn get_incr_size(&self) -> IncrSize {
        match self.registers[0x15].get() & 0x03 {
            0 => IncrSize::Size1,
            1 => IncrSize::Size32,
            _ => IncrSize::Size128,
        }
    }

    fn get_vram_word_address(&self) -> u16 {
        self.vram_latch.get() & 0x7FFF
    }

    
}