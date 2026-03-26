use crate::{
    app,
    core::{snemcore, sppu, sysinfo},
};

pub struct BgView<const BG_LAYER: usize> {
    needs_updating: bool,
    bg_layer_active: bool,
    bg_mode: Option<sppu::BgMode>,
    texture: app::Texture,
    // window: app::Texture,
    // cmath_en: app::Texture,
}

impl<const BG_LAYER: usize> BgView<BG_LAYER> {
    pub fn new(painter: &mut egui_glow::Painter) -> Self {
        let width = (sysinfo::SCREEN_WIDTH / 2) as usize;
        let height = (sysinfo::SCREEN_HEIGHT / 2) as usize;

        Self {
            needs_updating: true,
            bg_layer_active: true,
            bg_mode: None,
            texture: app::Texture::new(painter, width, height),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator, painter: &mut egui_glow::Painter) {
        if self.needs_updating {
            self.update_bg_pixels(snem_core);
            self.texture.update_texture(painter.gl());
            // self.needs_updating = false;
        }
        
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("bg_mode_sel")
                    .selected_text(format!("{:?}", self.bg_mode.unwrap_or(snem_core.ppu_regs.bg_mode)))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.bg_mode, Some(sppu::BgMode::Mode0), "Mode0");
                        ui.selectable_value(&mut self.bg_mode, Some(sppu::BgMode::Mode1), "Mode1");
                        ui.selectable_value(&mut self.bg_mode, Some(sppu::BgMode::Mode2), "Mode2");
                        ui.selectable_value(&mut self.bg_mode, Some(sppu::BgMode::Mode3), "Mode3");
                        ui.selectable_value(&mut self.bg_mode, Some(sppu::BgMode::Mode4), "Mode4");
                        ui.selectable_value(&mut self.bg_mode, Some(sppu::BgMode::Mode5), "Mode5");
                        ui.selectable_value(&mut self.bg_mode, Some(sppu::BgMode::Mode6), "Mode6");
                        ui.selectable_value(&mut self.bg_mode, Some(sppu::BgMode::Mode7), "Mode7");
                        ui.selectable_value(&mut self.bg_mode, None, "(Current in Program)");
                    });
            });
            
            ui.separator();
            
            let (width, height) = self.texture.size();
            let scale = 2.0;
            
            let image_size = egui::Vec2::new(width as f32, height as f32) * scale;
            
            ui.image(egui::load::SizedTexture::new(self.texture.texture_id(), image_size));
        });
    }

    fn update_bg_pixels(&mut self, snem_core: &snemcore::Snemulator) {
        let bg_layer = match BG_LAYER {
            1 => sppu::ColorLayer::Bg1,
            2 => sppu::ColorLayer::Bg2,
            3 => sppu::ColorLayer::Bg3,
            4 => sppu::ColorLayer::Bg4,
            _ => panic!("BgView must have BG_LAYER == 1,2,3, or 4"),
        };
        let color_depth = self.color_depth(snem_core);
        let bg_chr_base_addr = self.chr_base_addr(snem_core);
        let bg_cgram_base_addr = self.cgram_base_addr(snem_core);
        
        if bg_cgram_base_addr.is_none() {
            self.bg_layer_active = false;
            return;
        }
        
        self.bg_layer_active = true;
        
        let bg_cgram_base_addr = bg_cgram_base_addr.unwrap();
        let (width, height) = self.texture.size();
        let pixels = self.texture.pixels_mut();
        
        for y in 0..height {
            for x in 0..width {
                let tile_data = snem_core.ppu.bg_tile_idx(&snem_core.ppu_regs, bg_layer, x, y);
                
                let col = match color_depth {
                    sppu::ColorDepth::Bpp2 => snem_core.ppu.bg_col_2bpp(
                        &snem_core.ppu_regs, 
                        &snem_core.vram[..], 
                        &snem_core.cgram[..], 
                        tile_data, 
                        bg_chr_base_addr, 
                        bg_cgram_base_addr,
                    ),
                    sppu::ColorDepth::Bpp4 => snem_core.ppu.bg_col_4bpp(
                        &snem_core.ppu_regs, 
                        &snem_core.vram[..], 
                        &snem_core.cgram[..], 
                        tile_data, 
                        bg_chr_base_addr, 
                        bg_cgram_base_addr,
                    ),
                    sppu::ColorDepth::Bpp8 => snem_core.ppu.bg_col_8bpp(
                        &snem_core.ppu_regs, 
                        &snem_core.vram[..], 
                        &snem_core.cgram[..], 
                        tile_data, 
                        bg_chr_base_addr,
                    ),
                };
                
                let pixel_idx = (y * width + x) * 4;
                
                if col.transparent {
                    let checker = if (x / 2 + y / 2) % 2 == 0 { 0x50 } else { 0x30 };
                    pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[checker, checker, checker, 255]);
                } else {
                    pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[col.color.r, col.color.g, col.color.b, 255]);
                }
            }
        }
    }

    fn color_depth(&self, snem_core: &snemcore::Snemulator) -> sppu::ColorDepth {
        let bg_mode = self.bg_mode.unwrap_or(snem_core.ppu_regs.bg_mode);
        
        match BG_LAYER {
            1 => match bg_mode {
                sppu::BgMode::Mode0 => sppu::ColorDepth::Bpp2,
                sppu::BgMode::Mode1
                | sppu::BgMode::Mode2
                | sppu::BgMode::Mode5
                | sppu::BgMode::Mode6 => sppu::ColorDepth::Bpp4,
                _ => sppu::ColorDepth::Bpp8,
            },
            2 => match bg_mode {
                sppu::BgMode::Mode0 | sppu::BgMode::Mode4 | sppu::BgMode::Mode5 => {
                    sppu::ColorDepth::Bpp2
                }
                sppu::BgMode::Mode1 | sppu::BgMode::Mode2 | sppu::BgMode::Mode3 => {
                    sppu::ColorDepth::Bpp4
                }
                _ => sppu::ColorDepth::Bpp2,
            },
            3 => sppu::ColorDepth::Bpp2,
            4 => sppu::ColorDepth::Bpp2,
            _ => panic!("BgView must have BG_LAYER == 1,2,3, or 4"),
        }
    }

    fn chr_base_addr(&self, snem_core: &snemcore::Snemulator) -> u16 {
        match BG_LAYER {
            1 => (snem_core.ppu_regs.bg1_chr_base_addr as u16) << 12,
            2 => (snem_core.ppu_regs.bg2_chr_base_addr as u16) << 12,
            3 => (snem_core.ppu_regs.bg3_chr_base_addr as u16) << 12,
            4 => (snem_core.ppu_regs.bg4_chr_base_addr as u16) << 12,
            _ => panic!("BgView must have BG_LAYER == 1,2,3, or 4"),
        }
    }

    fn cgram_base_addr(&self, snem_core: &snemcore::Snemulator) -> Option<u8> {
        let bg_mode = self.bg_mode.unwrap_or(snem_core.ppu_regs.bg_mode);
        
        match BG_LAYER {
            1 => Some(0x00),
            2 => match bg_mode {
                sppu::BgMode::Mode0 => Some(0x20),
                sppu::BgMode::Mode1 |
                sppu::BgMode::Mode2 |
                sppu::BgMode::Mode3 | 
                sppu::BgMode::Mode4 |
                sppu::BgMode::Mode5 => Some(0x00),
                _ => None,
            },
            3 => match bg_mode {
                sppu::BgMode::Mode0 | sppu::BgMode::Mode1 => Some(0x00),
                _ => None,
            },
            4 => match bg_mode {
                sppu::BgMode::Mode0 => Some(0x00),
                _ => None,
            },
            _ => panic!("BgView must have BG_LAYER == 1,2,3, or 4"),
        }
    }
}
