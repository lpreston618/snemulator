// chr.rs
use glow::HasContext;
use crate::core::snemcore::Snemulator;
use crate::core::sppu::ColorDepth;

// Tiles per row in the texture atlas
const ATLAS_TILES_WIDE: usize = 16;
// Total tiles we decode (all of VRAM in the given bpp)
// 2bpp: 0x8000/8=4096, 4bpp: 0x8000/16=2048, 8bpp: 0x8000/32=1024
const TILE_PX: usize = 8; // each tile is always 8x8 pixels in the viewer

pub struct ChrTab {
    texture: Option<glow::Texture>,
    texture_id: Option<egui::TextureId>,
    bpp_mode: ColorDepth,
    palette_index: usize,
}

impl ChrTab {
    pub fn new() -> Self {
        Self {
            texture: None,
            texture_id: None,
            bpp_mode: ColorDepth::Bpp4,
            palette_index: 0,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &Snemulator, gl: &glow::Context, egui_renderer: &egui_glow::Painter) {
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
            self.palette_index = self.palette_index.min(max_pal);

            ui.label("Palette:");
            ui.add(egui::Slider::new(&mut self.palette_index, 0..=max_pal));
        });

        ui.separator();

        let (tile_count, words_per_tile) = match self.bpp_mode {
            ColorDepth::Bpp2 => (0x8000 / 8,  8usize),
            ColorDepth::Bpp4 => (0x8000 / 16, 16usize),
            ColorDepth::Bpp8 => (0x8000 / 32, 32usize),
        };

        let atlas_tiles_tall = (tile_count + ATLAS_TILES_WIDE - 1) / ATLAS_TILES_WIDE;
        let atlas_w = ATLAS_TILES_WIDE * TILE_PX;
        let atlas_h = atlas_tiles_tall * TILE_PX;

        // Build RGBA pixel buffer
        let mut pixels = vec![0u8; atlas_w * atlas_h * 4];

        for tile_idx in 0..tile_count {
            let tile_x = (tile_idx % ATLAS_TILES_WIDE) * TILE_PX;
            let tile_y = (tile_idx / ATLAS_TILES_WIDE) * TILE_PX;

            for row in 0..8usize {
                let base_addr = tile_idx * words_per_tile + row;

                let (bp01, bp23, bp45, bp67) = match self.bpp_mode {
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

                    let pal_idx = match self.bpp_mode {
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

                    let cgram_addr = match self.bpp_mode {
                        ColorDepth::Bpp2 => (self.palette_index << 2) | pal_idx as usize,
                        ColorDepth::Bpp4 => (self.palette_index << 4) | pal_idx as usize,
                        ColorDepth::Bpp8 => pal_idx as usize,
                    };

                    let color = snem_core.cgram[cgram_addr];

                    let px = tile_x + col;
                    let py = tile_y + row;
                    let i = (py * atlas_w + px) * 4;

                    // Transparent (index 0) shown as dark grey checkerboard
                    if pal_idx == 0 {
                        let checker = if (px / 2 + py / 2) % 2 == 0 { 80u8 } else { 50u8 };
                        pixels[i..i+4].copy_from_slice(&[checker, checker, checker, 255]);
                    } else {
                        pixels[i..i+4].copy_from_slice(&[color.r, color.g, color.b, 255]);
                    }
                }
            }
        }

        // Upload to GL texture
        let texture = self.texture.get_or_insert_with(|| unsafe {
            let tex = gl.create_texture().expect("Failed to create CHR texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            tex
        });

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(*texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0,
                glow::RGBA as i32,
                atlas_w as i32, atlas_h as i32,
                0, glow::RGBA, glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&pixels)),
            );
        }

        // Register or re-register with egui (size may change between bpp modes)
        let tex_id = egui_renderer.register_native_texture(*texture);
        // Free the previous ID if bpp changed (avoids leaking texture IDs)
        if let Some(old_id) = self.texture_id.replace(tex_id) {
            if old_id != tex_id {
                egui_renderer.free_texture(old_id);
            }
        }

        // Display scaled up in a scroll area
        let display_scale = 2.0;
        let image_size = egui::vec2(
            atlas_w as f32 * display_scale,
            atlas_h as f32 * display_scale,
        );

        egui::ScrollArea::both().show(ui, |ui| {
            ui.image(egui::load::SizedTexture::new(tex_id, image_size));
        });
    }
}