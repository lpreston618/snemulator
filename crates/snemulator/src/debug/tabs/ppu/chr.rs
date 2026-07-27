use snemcore::Snemulator;
use snemcore::sppu::{BgMode, Color, ColorDepth};
use crate::app::theme::AppTheme;

const ATLAS_TILES_WIDE: usize = 16;
const ATLAS_TILES_TALL: usize = 16;
const TILE_PX: usize = 8;
const ATLAS_PIXELS_WIDE: usize = ATLAS_TILES_WIDE * TILE_PX;
const ATLAS_PIXELS_TALL: usize = ATLAS_TILES_TALL * TILE_PX;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ChrTab {
    Bg1, Bg2, Bg3, Bg4, Obj1, Obj2,
}

impl ChrTab {
    fn all() -> &'static [ChrTab] {
        &[ChrTab::Bg1, ChrTab::Bg2, ChrTab::Bg3, ChrTab::Bg4, ChrTab::Obj1, ChrTab::Obj2]
    }

    fn label(&self) -> &'static str {
        match self {
            ChrTab::Bg1  => "BG1",
            ChrTab::Bg2  => "BG2",
            ChrTab::Bg3  => "BG3",
            ChrTab::Bg4  => "BG4",
            ChrTab::Obj1 => "OBJ1",
            ChrTab::Obj2 => "OBJ2",
        }
    }

    // Returns (atlas_index, base_addr, bpp, palette_type)
    // palette_type: true = bg, false = obj
    fn atlas_index(&self) -> usize {
        match self {
            ChrTab::Bg1  => 0,
            ChrTab::Bg2  => 1,
            ChrTab::Bg3  => 2,
            ChrTab::Bg4  => 3,
            ChrTab::Obj1 => 4,
            ChrTab::Obj2 => 5,
        }
    }

    fn resolve(&self, core: &Snemulator) -> (usize, ColorDepth, bool) {
        let bg_mode = core.ppu_regs.bg_mode;

        match self {
            ChrTab::Bg1  => (core.ppu_regs.bg_settings[0].chr_base_addr as usize, self.color_depth(bg_mode), true),
            ChrTab::Bg2  => (core.ppu_regs.bg_settings[1].chr_base_addr as usize, self.color_depth(bg_mode), true),
            ChrTab::Bg3  => (core.ppu_regs.bg_settings[2].chr_base_addr as usize, self.color_depth(bg_mode), true),
            ChrTab::Bg4  => (core.ppu_regs.bg_settings[3].chr_base_addr as usize, self.color_depth(bg_mode), true),
            ChrTab::Obj1 => (core.ppu_regs.name_base_addr as usize,           self.color_depth(bg_mode), false),
            ChrTab::Obj2 => (core.ppu_regs.name_secondary_base_addr as usize, self.color_depth(bg_mode), false),
        }
    }

    fn color_depth(&self, bg_mode: BgMode) -> ColorDepth {
        match (self, bg_mode) {
            (ChrTab::Bg1, BgMode::Mode0) => Some(ColorDepth::Bpp2),
            (ChrTab::Bg2, BgMode::Mode0) => Some(ColorDepth::Bpp2),
            (ChrTab::Bg3, BgMode::Mode0) => Some(ColorDepth::Bpp2),
            (ChrTab::Bg4, BgMode::Mode0) => Some(ColorDepth::Bpp2),

            (ChrTab::Bg1, BgMode::Mode1) => Some(ColorDepth::Bpp4),
            (ChrTab::Bg2, BgMode::Mode1) => Some(ColorDepth::Bpp4),
            (ChrTab::Bg3, BgMode::Mode1) => Some(ColorDepth::Bpp2),
            (ChrTab::Bg4, BgMode::Mode1) => None,

            (ChrTab::Bg1, BgMode::Mode2) => Some(ColorDepth::Bpp4),
            (ChrTab::Bg2, BgMode::Mode2) => Some(ColorDepth::Bpp4),
            (ChrTab::Bg3, BgMode::Mode2) => None,
            (ChrTab::Bg4, BgMode::Mode2) => None,

            (ChrTab::Bg1, BgMode::Mode3) => Some(ColorDepth::Bpp8),
            (ChrTab::Bg2, BgMode::Mode3) => Some(ColorDepth::Bpp4),
            (ChrTab::Bg3, BgMode::Mode3) => None,
            (ChrTab::Bg4, BgMode::Mode3) => None,

            (ChrTab::Bg1, BgMode::Mode4) => Some(ColorDepth::Bpp8),
            (ChrTab::Bg2, BgMode::Mode4) => Some(ColorDepth::Bpp2),
            (ChrTab::Bg3, BgMode::Mode4) => None,
            (ChrTab::Bg4, BgMode::Mode4) => None,

            (ChrTab::Bg1, BgMode::Mode5) => Some(ColorDepth::Bpp4),
            (ChrTab::Bg2, BgMode::Mode5) => Some(ColorDepth::Bpp2),
            (ChrTab::Bg3, BgMode::Mode5) => None,
            (ChrTab::Bg4, BgMode::Mode5) => None,

            (ChrTab::Bg1, BgMode::Mode6) => Some(ColorDepth::Bpp4),
            (ChrTab::Bg2, BgMode::Mode6) => None,
            (ChrTab::Bg3, BgMode::Mode6) => None,
            (ChrTab::Bg4, BgMode::Mode6) => None,

            (ChrTab::Bg1, BgMode::Mode7) => Some(ColorDepth::Bpp8),
            (ChrTab::Bg2, BgMode::Mode7) => None,
            (ChrTab::Bg3, BgMode::Mode7) => None,
            (ChrTab::Bg4, BgMode::Mode7) => None,

            (ChrTab::Obj1, _) => Some(ColorDepth::Bpp4),
            (ChrTab::Obj2, _) => Some(ColorDepth::Bpp4),
        }.unwrap_or(ColorDepth::Bpp2)
    }
}

pub struct ChrViewer {
    atlases: [Option<egui::TextureHandle>; 6],
    atlas_pixels: [Vec<u8>; 6],
    bpp_mode: ColorDepth,
    palette_index: usize,
    selected_tab: ChrTab,
}

impl ChrViewer {
    pub fn new() -> Self {
        Self {
            atlases: [None, None, None, None, None, None],
            atlas_pixels: std::array::from_fn(|_| {
                vec![0u8; ATLAS_PIXELS_WIDE * ATLAS_PIXELS_TALL * 4]
            }),
            bpp_mode: ColorDepth::Bpp4,
            palette_index: 0,
            selected_tab: ChrTab::Bg1,
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        core: &Snemulator,
        app_theme: &AppTheme,
    ) {
        ui.horizontal(|ui| {
            for &tab in ChrTab::all() {
                ui.selectable_value(&mut self.selected_tab, tab, tab.label());
            }
        });

        ui.separator();

        let tab = self.selected_tab;
        let idx = tab.atlas_index();
        let (base_addr, _, is_bg) = tab.resolve(core);
        let bpp = if is_bg { self.bpp_mode } else { ColorDepth::Bpp4 };
        let is_mode7 = matches!(core.ppu_regs.bg_mode, BgMode::Mode7) && matches!(tab, ChrTab::Bg1 | ChrTab::Bg2);

        // Controls
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("BPP:").color(app_theme.text_primary));

            let mut bpp = self.bpp_mode;

            if is_bg {
                bpp = ColorDepth::Bpp4;
            }

            if is_mode7 {
                bpp = ColorDepth::Bpp8;
            }

            ui.add_enabled_ui(is_bg && !is_mode7, |ui| {
                ui.selectable_value(&mut bpp, ColorDepth::Bpp2, "2bpp");
                ui.selectable_value(&mut bpp, ColorDepth::Bpp4, "4bpp");
                ui.selectable_value(&mut bpp, ColorDepth::Bpp8, "8bpp");
            });

            if is_bg && !is_mode7 {
                self.bpp_mode = bpp;
            }

            ui.separator();

            let max_pal = match bpp {
                ColorDepth::Bpp2 => 31,
                ColorDepth::Bpp4 => 15,
                ColorDepth::Bpp8 => 0,
            };
            self.palette_index = self.palette_index.min(max_pal);

            ui.label(egui::RichText::new("Palette:").color(app_theme.text_primary));
            ui.add_enabled(
                bpp != ColorDepth::Bpp8,
                egui::Slider::new(&mut self.palette_index, 0..=max_pal),
            );
        });

        ui.separator();

        if is_mode7 {
            Self::update_mode7_atlas(&mut self.atlas_pixels[idx], core);
        } else {
            Self::update_atlas(&mut self.atlas_pixels[idx], core, base_addr, bpp, self.palette_index);
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [ATLAS_PIXELS_WIDE, ATLAS_PIXELS_TALL],
            &self.atlas_pixels[idx],
        );
        match &mut self.atlases[idx] {
            Some(handle) => handle.set(color_image, egui::TextureOptions::NEAREST),
            None => {
                self.atlases[idx] = Some(ui.ctx().load_texture(
                    format!("chr_atlas_{idx}"),
                    color_image,
                    egui::TextureOptions::NEAREST,
                ));
            }
        }

        // Scale to fill available space, maintaining square aspect ratio
        let margin = 10.0;
        let available = ui.available_size();
        let side = available.x.min(available.y - margin) - margin;
        let image_size = egui::vec2(side, side);

        if let Some(handle) = &self.atlases[idx] {
            let mut job = egui::text::LayoutJob::default();
            job.append(
                tab.label(),
                0.0,
                egui::TextFormat::simple(egui::FontId::monospace(12.0), app_theme.text_primary),
            );
            job.append(
                &format!(" (${base_addr:04X})"),
                0.0,
                egui::TextFormat::simple(egui::FontId::monospace(12.0), app_theme.syntax_address),
            );
            ui.label(job);
            ui.image(egui::load::SizedTexture::new(handle.id(), image_size));
        }
    }

    fn update_atlas(
        pixels: &mut [u8],
        core: &Snemulator,
        base_addr: usize,
        bpp: ColorDepth,
        palette_idx: usize,
    ) {
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
                    ColorDepth::Bpp2 => (core.vram[base_addr], 0u16, 0u16, 0u16),
                    ColorDepth::Bpp4 => (
                        core.vram[base_addr],
                        core.vram[base_addr + 8],
                        0u16,
                        0u16,
                    ),
                    ColorDepth::Bpp8 => (
                        core.vram[base_addr],
                        core.vram[base_addr + 8],
                        core.vram[base_addr + 16],
                        core.vram[base_addr + 24],
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
                            (b7 << 7)
                                | (b6 << 6)
                                | (b5 << 5)
                                | (b4 << 4)
                                | (b3 << 3)
                                | (b2 << 2)
                                | (b1 << 1)
                                | b0
                        }
                    };

                    let cgram_addr = match bpp {
                        ColorDepth::Bpp2 => (palette_idx << 2) | pal_idx as usize,
                        ColorDepth::Bpp4 => (palette_idx << 4) | pal_idx as usize,
                        ColorDepth::Bpp8 => pal_idx as usize,
                    };

                    let color = core.cgram[cgram_addr];

                    let px = tile_x + col;
                    let py = tile_y + row;
                    let pixel_idx = (py * ATLAS_TILES_WIDE * TILE_PX + px) * 4;

                    // Transparent (index 0) shown as dark grey checkerboard
                    if pal_idx == 0 {
                        let checker = if (px / 2 + py / 2) % 2 == 0 {
                            0x50
                        } else {
                            0x30
                        };
                        pixels[pixel_idx..pixel_idx + 4]
                            .copy_from_slice(&[checker, checker, checker, 255]);
                    } else {
                        pixels[pixel_idx..pixel_idx + 4]
                            .copy_from_slice(&[color.r, color.g, color.b, 255]);
                    }
                }
            }
        }
    }

    fn update_mode7_atlas(
        pixels: &mut [u8],
        core: &Snemulator,
    ) {
        log::debug!("Here");

        let words_per_tile = 64;

        let tile_count = ATLAS_TILES_WIDE * ATLAS_TILES_TALL;

        for tile_idx in 0..tile_count {
            let tile_y = tile_idx / ATLAS_TILES_WIDE;
            let tile_x = tile_idx % ATLAS_TILES_WIDE;
            
            let chr_addr = tile_idx * words_per_tile;

            for row in 0..8usize {
                for col in 0..8usize {
                    let pixel_addr = (chr_addr as usize) + (row * 8) + col;
                    let pal_idx = (core.vram[pixel_addr] >> 8) as u8;

                    let color = if core.ppu_regs.use_direct_col {
                        // treat our color index as color information: BBGGGRRR -> RRR00 GGG00 BB000
                        let r = (pal_idx & 0x7) << 2;
                        let g = (pal_idx & 0x38) >> 1;
                        let b = (pal_idx & 0xC0) >> 3;
                        Color { r: r, g: g, b: b }
                    } else {
                        core.cgram[pal_idx as usize]
                    };

                    let pixel_y = tile_y * 8 + row;
                    let pixel_x = tile_x * 8 + col;

                    let dst_idx = 4 * (pixel_y * ATLAS_PIXELS_WIDE + pixel_x);

                    pixels[dst_idx..dst_idx + 4].copy_from_slice(&color.to_rgba_bytes());
                }
            }
        }
    }
}