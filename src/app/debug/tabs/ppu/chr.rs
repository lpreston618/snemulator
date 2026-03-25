use crate::app;
use crate::app::utils::monospace_text;
use crate::core::snemcore;
use crate::core::sppu::ColorDepth;

// Tiles per row in the texture atlas
const ATLAS_TILES_WIDE: usize = 16;
const ATLAS_TILES_TALL: usize = 16;
// Total tiles we decode (all of VRAM in the given bpp)
// 2bpp: 0x8000/8=4096, 4bpp: 0x8000/16=2048, 8bpp: 0x8000/32=1024
const TILE_PX: usize = 8; // each tile is always 8x8 pixels in the viewer
const ATLAS_PIXELS_WIDE: usize = ATLAS_TILES_WIDE * TILE_PX;
const ATLAS_PIXELS_TALL: usize = ATLAS_TILES_TALL * TILE_PX;

pub struct ChrViewer {
    atlases: [app::Texture; 6],
    bpp_mode: ColorDepth,
    bg_palette_index: usize,
    obj_palette_index: usize,
}

impl ChrViewer {
    pub fn new(painter: &mut egui_glow::Painter) -> Self { 
        Self {
            atlases: [
                app::Texture::new(painter, ATLAS_PIXELS_WIDE, ATLAS_PIXELS_TALL),
                app::Texture::new(painter, ATLAS_PIXELS_WIDE, ATLAS_PIXELS_TALL),
                app::Texture::new(painter, ATLAS_PIXELS_WIDE, ATLAS_PIXELS_TALL),
                app::Texture::new(painter, ATLAS_PIXELS_WIDE, ATLAS_PIXELS_TALL),
                app::Texture::new(painter, ATLAS_PIXELS_WIDE, ATLAS_PIXELS_TALL),
                app::Texture::new(painter, ATLAS_PIXELS_WIDE, ATLAS_PIXELS_TALL),
            ],
            bpp_mode: ColorDepth::Bpp4,
            bg_palette_index: 0,
            obj_palette_index: 0,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator, egui_renderer: &mut egui_glow::Painter) {
        let gl = egui_renderer.gl().as_ref();
        
        // Controls
        ui.horizontal(|ui| {
            ui.label("BPP:");
            ui.selectable_value(&mut self.bpp_mode, ColorDepth::Bpp2, "2bpp");
            ui.selectable_value(&mut self.bpp_mode, ColorDepth::Bpp4, "4bpp");
            ui.selectable_value(&mut self.bpp_mode, ColorDepth::Bpp8, "8bpp");

            ui.separator();

            let max_pal = match self.bpp_mode {
                ColorDepth::Bpp2 => 31,
                ColorDepth::Bpp4 => 15,
                ColorDepth::Bpp8 => 0,
            };
            self.bg_palette_index = self.bg_palette_index.min(max_pal);

            ui.label("Bg Palette:");
            ui.add_enabled(
                self.bpp_mode != ColorDepth::Bpp8, 
                egui::Slider::new(&mut self.bg_palette_index, 0..=max_pal)
            );
            
            ui.label("Obj Palette:");
            ui.add(egui::Slider::new(&mut self.obj_palette_index, 0..=15));
        });

        ui.separator();

        let atlas_w = ATLAS_TILES_WIDE * TILE_PX;
        let atlas_h = ATLAS_TILES_TALL * TILE_PX;
        
        let bg1_base_addr = (snem_core.ppu_regs.bg1_chr_base_addr as usize) << 12;
        let bg2_base_addr = (snem_core.ppu_regs.bg2_chr_base_addr as usize) << 12;
        let bg3_base_addr = (snem_core.ppu_regs.bg3_chr_base_addr as usize) << 12;
        let bg4_base_addr = (snem_core.ppu_regs.bg4_chr_base_addr as usize) << 12;;
        
        Self::update_atlas(&mut self.atlases[0], snem_core, bg1_base_addr, self.bpp_mode, self.bg_palette_index);
        Self::update_atlas(&mut self.atlases[1], snem_core, bg2_base_addr, self.bpp_mode, self.bg_palette_index);
        Self::update_atlas(&mut self.atlases[2], snem_core, bg3_base_addr, self.bpp_mode, self.bg_palette_index);
        Self::update_atlas(&mut self.atlases[3], snem_core, bg4_base_addr, self.bpp_mode, self.bg_palette_index);
        
        let obj1_base_addr = (snem_core.ppu_regs.name_base_addr as usize) << 13;
        let obj2_base_addr = obj1_base_addr + ((snem_core.ppu_regs.name_secondary_select as usize) << 12);
        
        Self::update_atlas(&mut self.atlases[4], snem_core, obj1_base_addr, ColorDepth::Bpp4, self.obj_palette_index);
        Self::update_atlas(&mut self.atlases[5], snem_core, obj2_base_addr, ColorDepth::Bpp4, self.obj_palette_index);

        for i in 0..6 {
            self.atlases[i].update_texture(gl);
        }

        // Display scaled up in a scroll area
        let display_scale = 1.5;
        let image_size = egui::vec2(
            atlas_w as f32 * display_scale,
            atlas_h as f32 * display_scale,
        );
        
        
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(monospace_text(format!("Bg1 Chr Mem (${:04X})", (snem_core.ppu_regs.bg1_chr_base_addr as u16) << 12)));
                    
                    ui.image(egui::load::SizedTexture::new(self.atlases[0].texture_id(), image_size));
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label(monospace_text(format!("Bg2 Chr Mem (${:04X})", (snem_core.ppu_regs.bg2_chr_base_addr as u16) << 12)));
                    
                    ui.image(egui::load::SizedTexture::new(self.atlases[1].texture_id(), image_size));
                });
      
                ui.separator();
                
                ui.vertical(|ui| {
                    let base_addr = snem_core.ppu_regs.name_base_addr;
                    
                    ui.label(monospace_text(format!("Obj1 Chr Mem (${:04X})", (base_addr as u16) << 12)));
                    
                    ui.image(egui::load::SizedTexture::new(self.atlases[4].texture_id(), image_size));
                });
            });
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(monospace_text(format!("Bg3 Chr Mem (${:04X})", (snem_core.ppu_regs.bg3_chr_base_addr as u16) << 12)));
                    
                    ui.image(egui::load::SizedTexture::new(self.atlases[2].texture_id(), image_size));
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label(monospace_text(format!("Bg4 Chr Mem (${:04X})", (snem_core.ppu_regs.bg4_chr_base_addr as u16) << 12)));
                    
                    ui.image(egui::load::SizedTexture::new(self.atlases[3].texture_id(), image_size));
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    let base_addr = snem_core.ppu_regs.name_base_addr + snem_core.ppu_regs.name_secondary_select;
                    
                    ui.label(monospace_text(format!("Obj2 Chr Mem (${:04X})", (base_addr as u16) << 12)));
                    
                    ui.image(egui::load::SizedTexture::new(self.atlases[5].texture_id(), image_size));
                });
            });
        });
    }
    
    fn update_atlas(atlas: &mut app::Texture, snem_core: &snemcore::Snemulator, base_addr: usize, bpp: ColorDepth, palette_idx: usize) {
        let pixels = atlas.pixels_mut();
        
        let words_per_tile = match bpp {
            ColorDepth::Bpp2 => 8,
            ColorDepth::Bpp4 => 16,
            ColorDepth::Bpp8 => 32,
        };
        
        let tile_count = ATLAS_TILES_WIDE * ATLAS_TILES_TALL;
        
        for tile_idx in 0..tile_count {
            let tile_x = (tile_idx % ATLAS_TILES_WIDE) * TILE_PX;
            let tile_y = (tile_idx / ATLAS_TILES_WIDE) * TILE_PX;

            for row in 0..8usize {
                let base_addr = (base_addr + tile_idx * words_per_tile + row) & 0x7FFF;
                
                let (bp01, bp23, bp45, bp67) = match bpp {
                    ColorDepth::Bpp2 => (
                        snem_core.vram[base_addr],
                        0u16, 0u16, 0u16,
                    ),
                    ColorDepth::Bpp4 => (
                        snem_core.vram[base_addr],
                        snem_core.vram[base_addr + 8],
                        0u16, 0u16,
                    ),
                    ColorDepth::Bpp8 => (
                        snem_core.vram[base_addr],
                        snem_core.vram[base_addr + 8],
                        snem_core.vram[base_addr + 16],
                        snem_core.vram[base_addr + 24],
                    ),
                };
                
                for col in 0..8usize {
                    let shift_lo = 7 - col;
                    let shift_hi = 15 - col;

                    let pal_idx = match bpp {
                        ColorDepth::Bpp2 => {
                            let b0 = ((bp01 >> shift_lo) & 1) as u8;
                            let b1 = ((bp01 >> shift_hi) & 1) as u8;
                            (b1 << 1) | b0
                        }
                        ColorDepth::Bpp4 => {
                            let b0 = ((bp01 >> shift_lo) & 1) as u8;
                            let b1 = ((bp01 >> shift_hi) & 1) as u8;
                            let b2 = ((bp23 >> shift_lo) & 1) as u8;
                            let b3 = ((bp23 >> shift_hi) & 1) as u8;
                            (b3 << 3) | (b2 << 2) | (b1 << 1) | b0
                        }
                        ColorDepth::Bpp8 => {
                            let b0 = ((bp01 >> shift_lo) & 1) as u8;
                            let b1 = ((bp01 >> shift_hi) & 1) as u8;
                            let b2 = ((bp23 >> shift_lo) & 1) as u8;
                            let b3 = ((bp23 >> shift_hi) & 1) as u8;
                            let b4 = ((bp45 >> shift_lo) & 1) as u8;
                            let b5 = ((bp45 >> shift_hi) & 1) as u8;
                            let b6 = ((bp67 >> shift_lo) & 1) as u8;
                            let b7 = ((bp67 >> shift_hi) & 1) as u8;
                            (b7 << 7) | (b6 << 6) | (b5 << 5) | (b4 << 4)
                                | (b3 << 3) | (b2 << 2) | (b1 << 1) | b0
                        }
                    };

                    let cgram_addr = match bpp {
                        ColorDepth::Bpp2 => (palette_idx << 2) | pal_idx as usize,
                        ColorDepth::Bpp4 => (palette_idx << 4) | pal_idx as usize,
                        ColorDepth::Bpp8 => pal_idx as usize,
                    };

                    let color = snem_core.cgram[cgram_addr];

                    let px = tile_x + col;
                    let py = tile_y + row;
                    let pixel_idx = (py * ATLAS_TILES_WIDE * TILE_PX + px) * 4;

                    // Transparent (index 0) shown as dark grey checkerboard
                    if pal_idx == 0 {
                        let checker = if (px / 2 + py / 2) % 2 == 0 { 0x50 } else { 0x30 };
                        pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[checker, checker, checker, 255]);
                    } else {
                        pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[color.r, color.g, color.b, 255]);
                    }
                }
            }
        }
    }
}