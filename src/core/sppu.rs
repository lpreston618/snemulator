use crate::core::{scpu::CpuInterrupt, sppu::{bus::PpuBus, regs::{HVTimerIRQ, ObjectSize, ObjectSizeSelect, PpuRegs, TileSize, TilemapCount}}};

pub mod bus;
pub mod color;
pub mod regs;

const VBLANK_START_SCANLINE: usize = 225;
const VBLANK_END_SCANLINE_NTSC: usize = 261;
const VISIBLE_SCANLINE_START_DOT: usize = 22;
const HBLANK_START_DOT: usize = 278;
const SCANLINE_END_DOT: usize = 340;

// const HBLANK_END_DOT: usize = VISIBLE_SCANLINE_START_DOT;
// const HBLANK_DISABLE_SCANLINE: usize = VBLANK_START_SCANLINE-1;
// const VBLANK_END_SCANLINE_PAL: u16 = 311;
// const VBLANK_INTERLACE_START_SCANLINE: u16 = 239;

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

struct BgData {
    scroll_x: u16,
    scroll_y: u16, 
    tilemap_cnt_x: TilemapCount,
    tilemap_cnt_y: TilemapCount, 
    tile_size: TileSize,
    tilemap_base_addr: u16,
    mosaic_en: bool,
}

pub struct Ppu5C7x {
    pub dot: usize,
    pub scanline: usize,
    pub frame: usize,
    
    scanline_sprites: Vec<OAMSprite>,
    scanline_spr_cnt: usize,
    
    /// Number of master clocks until the next dot
    pub clocks: usize,
}

impl Ppu5C7x {
    pub fn new() -> Self {
        Self {
            dot: 0,
            scanline: 0,
            frame: 0,
            scanline_sprites: Vec::new(),
            scanline_spr_cnt: 0,
            clocks: 0,
        }
    }
    
    /// Cycles the PPU for a certain number of master clocks
    pub fn cycle(&mut self, bus: &mut PpuBus) {
        if (0 < self.scanline && self.scanline < VBLANK_START_SCANLINE)
            && (VISIBLE_SCANLINE_START_DOT <= self.dot && self.dot < HBLANK_START_DOT)
        {
            // self.draw_dot(bus);
        }

        self.update_dot_and_scanline(bus);
        self.update_hv_timers(bus);

        self.clocks += 4;

        if self.dot >= SCANLINE_END_DOT-4 {
            self.clocks += 1;
        }
    }
    
    fn update_dot_and_scanline(&mut self, bus: &mut PpuBus) {
        let regs = &mut bus.ppu_regs;
        
        self.dot += 1;

        if self.dot == SCANLINE_END_DOT {
            self.dot = 0;
            self.scanline += 1;

            if self.scanline == VBLANK_END_SCANLINE_NTSC {
                self.scanline = 0;
            }
        }

        // End of v-blank, scanline 0 is not visible
        if self.dot == 0 && self.scanline == 0 {
            regs.vblank_flag = false;
            regs.vblank_nmi_flag = false;
        }

        // End of h-blank
        if self.dot == VISIBLE_SCANLINE_START_DOT {
            regs.hblank_flag = false;

            // Start of visible scanline
            if 0 < self.scanline && self.scanline < VBLANK_START_SCANLINE {
                self.find_scanline_sprites(bus);
            }
        }
        
        let regs = &mut bus.ppu_regs; // repeat to appease the borrow checker

        // Start of h-blank
        if self.dot == HBLANK_START_DOT && self.scanline < VBLANK_START_SCANLINE {
            regs.hblank_flag = true;
        }

        // Start of v-blank
        if self.dot == 0 && self.scanline == VBLANK_START_SCANLINE {
            regs.vblank_flag = true;
            
            if regs.vblank_nmi_en {
                regs.vblank_nmi_flag = true;
                bus.trigger_interrupt(CpuInterrupt::NMI);
            }
            
            bus.set_frame_finished();
        }
    }
    
    fn update_hv_timers(&self, bus: &mut PpuBus) {
        let regs = &mut bus.ppu_regs;
        
        regs.h_counter = self.dot as u16;
        regs.v_counter = self.scanline as u16;

        match regs.hv_timer_irq_mode {
            HVTimerIRQ::None => {}
            HVTimerIRQ::HTimer => {
                if regs.h_counter == regs.h_counter_target {
                    regs.hv_timer_irq_flag = true;
                    bus.trigger_interrupt(CpuInterrupt::IRQ);
                }
            }
            HVTimerIRQ::VTimer => {
                if regs.v_counter == regs.v_counter_target && regs.h_counter == 0 {
                    regs.hv_timer_irq_flag = true;
                    bus.trigger_interrupt(CpuInterrupt::IRQ);
                }
            }
            HVTimerIRQ::Both => {
                if regs.v_counter == regs.v_counter_target && regs.h_counter == regs.h_counter_target {
                    regs.hv_timer_irq_flag = true;
                    bus.trigger_interrupt(CpuInterrupt::IRQ);
                }
            }
        }
    }
    
    /// Finds all possible sprites that could be rendered on the current scanline
    /// based on the y-positions of the sprites
    fn find_scanline_sprites(&mut self, bus: &mut PpuBus) {
        let regs = &mut bus.ppu_regs;
        
        self.scanline_sprites.clear();

        let screen_y = if regs.screen_interlace_enabled && regs.obj_interlace_enabled {
            2*self.screen_y() + (self.frame & 1)
        } else {
            self.screen_y()
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
            if spr_y as usize <= screen_y && screen_y < spr_y_max as usize  {
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
