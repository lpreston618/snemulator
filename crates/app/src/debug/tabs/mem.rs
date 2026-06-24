use snemcore::{
    Snemulator, sppu::{Color, OAMSprite, ObjectSizeSelect},
};

use crate::app::theme::AppTheme;

#[derive(PartialEq, Clone, Copy)]
enum MemViewRegion {
    Wram,
    Sram,
    Vram,
    Aram,
    Rom,
    Oam,
    Cgram,
}

impl MemViewRegion {
    fn label(&self) -> &'static str {
        match self {
            MemViewRegion::Wram => "WRAM",
            MemViewRegion::Sram => "SRAM",
            MemViewRegion::Vram => "VRAM",
            MemViewRegion::Aram => "ARAM",
            MemViewRegion::Rom => "ROM",
            MemViewRegion::Oam => "OAM",
            MemViewRegion::Cgram => "CGRAM",
        }
    }
    // Address display width: WRAM/ROM are 24-bit, rest are 16-bit offsets into their own space
    fn addr_width(&self) -> usize {
        match self {
            MemViewRegion::Wram | MemViewRegion::Rom => 6,
            _ => 4,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum OamViewMode {
    Raw,
    Sprites,
}

impl OamViewMode {
    fn label(&self) -> &'static str {
        match self {
            OamViewMode::Raw => "Raw Memory",
            OamViewMode::Sprites => "Sprites",
        }
    }
}

pub struct MemoryTab {
    region: MemViewRegion,
    oam_view_mode: OamViewMode,
    sprite_texture: Option<egui::TextureHandle>,
    selected_sprite: Option<usize>,
}

impl MemoryTab {
    pub fn new() -> Self {
        Self {
            region: MemViewRegion::Wram,
            oam_view_mode: OamViewMode::Sprites,
            sprite_texture: None,
            selected_sprite: None,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, core: &Snemulator, app_theme: &AppTheme) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Region:").color(app_theme.text_secondary));
            egui::ComboBox::from_id_salt("mem_region")
                .selected_text(self.region.label())
                .show_ui(ui, |ui| {
                    for region in [
                        MemViewRegion::Wram,
                        MemViewRegion::Sram,
                        MemViewRegion::Vram,
                        MemViewRegion::Aram,
                        MemViewRegion::Rom,
                        MemViewRegion::Oam,
                        MemViewRegion::Cgram,
                    ] {
                        ui.selectable_value(&mut self.region, region, region.label());
                    }
                });

            // Show OAM mode toggle if OAM is selected
            if self.region == MemViewRegion::Oam {
                ui.separator();
                ui.label(egui::RichText::new("View:").color(app_theme.text_secondary));
                egui::ComboBox::from_id_salt("oam_mode")
                    .selected_text(self.oam_view_mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.oam_view_mode,
                            OamViewMode::Raw,
                            OamViewMode::Raw.label(),
                        );
                        ui.selectable_value(
                            &mut self.oam_view_mode,
                            OamViewMode::Sprites,
                            OamViewMode::Sprites.label(),
                        );
                    });
            }
        });
        app_theme.debugger_separator(ui);

        let addr_w = self.region.addr_width();

        match self.region {
            MemViewRegion::Vram => Self::render_vram_dump(ui, &core.vram[..], app_theme),
            MemViewRegion::Cgram => Self::render_cgram_dump(ui, &core.cgram[..], app_theme),
            MemViewRegion::Oam if self.oam_view_mode == OamViewMode::Sprites => {
                self.render_oam_sprites(ui, core, app_theme);
            }
            _ => {
                let data: &[u8] = match self.region {
                    MemViewRegion::Wram => &core.wram[..],
                    MemViewRegion::Sram => &core.cart.as_ref().unwrap().ram[..],
                    MemViewRegion::Aram => &core.ssmp.aram_slice(),
                    MemViewRegion::Rom => &core.cart.as_ref().unwrap().rom[..],
                    MemViewRegion::Oam => &core.raw_oam[..],
                    _ => unreachable!(),
                };
                Self::render_byte_dump(ui, data, addr_w, app_theme);
            }
        }
    }

    fn render_oam_sprites(&mut self, ui: &mut egui::Ui, core: &Snemulator, app_theme: &AppTheme) {
        let available_height = ui.available_height();

        ui.horizontal(|ui| {
            ui.set_min_height(available_height);

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width() * 0.4, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false]) // Now this only fills the left column's height
                        .show(ui, |ui| {
                            egui::Grid::new("oam_sprites_grid")
                                .striped(true)
                                .show(ui, |ui| {
                                    let header = |ui: &mut egui::Ui, text: &str| {
                                        ui.label(
                                            egui::RichText::new(text)
                                                .color(app_theme.text_secondary)
                                                .strong(),
                                        );
                                    };
                                    header(ui, "Idx");
                                    header(ui, "X, Y");
                                    header(ui, "Tile");
                                    header(ui, "Pal");
                                    header(ui, "Pri");
                                    header(ui, "Size");
                                    ui.end_row();

                                    for i in 0..128 {
                                        let sprite = &core.oam[i];

                                        // Dim off-screen or unused sprites
                                        let is_active =
                                            sprite.y < 224 && sprite.x < 0x100 && sprite.x < 256;
                                        let color = if is_active {
                                            app_theme.text_primary
                                        } else {
                                            app_theme.text_disabled
                                        };

                                        let is_selected = self.selected_sprite == Some(i);
                                        if ui
                                            .selectable_label(
                                                is_selected,
                                                egui::RichText::new(format!("{:03}", i))
                                                    .monospace()
                                                    .color(if is_selected {
                                                        app_theme.accent
                                                    } else {
                                                        color
                                                    }),
                                            )
                                            .clicked()
                                        {
                                            self.selected_sprite = Some(i);
                                        }

                                        ui.colored_label(
                                            color,
                                            egui::RichText::new(format!(
                                                "{}, {}",
                                                sprite.x, sprite.y
                                            ))
                                            .monospace(),
                                        );
                                        ui.colored_label(
                                            color,
                                            egui::RichText::new(format!("${}{:02X}", sprite.use_second_obj_table as u8, sprite.tile_idx))
                                                .monospace(),
                                        );
                                        ui.colored_label(
                                            color,
                                            egui::RichText::new(format!("{}", sprite.palette))
                                                .monospace(),
                                        );
                                        ui.colored_label(
                                            color,
                                            egui::RichText::new(format!("{}", sprite.priority))
                                                .monospace(),
                                        );
                                        ui.colored_label(
                                            color,
                                            egui::RichText::new(if sprite.size_select {
                                                "L"
                                            } else {
                                                "S"
                                            })
                                            .monospace(),
                                        );
                                        ui.end_row();
                                    }
                                });
                        });
                },
            );

            ui.separator();

            ui.vertical(|ui| {
                if let Some(idx) = self.selected_sprite {
                    let sprite = &core.oam[idx];

                    let (pixels, width, height) = render_sprite(
                        &sprite,
                        &core.vram[..],
                        &core.cgram[..],
                        core.ppu_regs.name_base_addr,
                        core.ppu_regs.name_secondary_base_addr,
                        core.ppu_regs.obj_sprite_size,
                    );

                    // Load (or replace) an egui-managed texture sized exactly to this sprite.
                    // No need to hand-roll a GL texture here: the image is tiny (at most 64x64),
                    // changes only when the selection changes, and egui's own texture manager
                    // already handles upload/caching/cleanup for us.
                    let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &pixels);
                    let texture = self.sprite_texture.get_or_insert_with(|| {
                        ui.ctx()
                            .load_texture("oam_sprite_preview", image.clone(), egui::TextureOptions::NEAREST)
                    });
                    texture.set(image, egui::TextureOptions::NEAREST);

                    app_theme.section_header(ui, &format!("Sprite {:03}", idx));

                    let key = |ui: &mut egui::Ui, s: &str| {
                        ui.label(
                            egui::RichText::new(s)
                                .monospace()
                                .color(app_theme.syntax_register),
                        );
                    };
                    let val = |ui: &mut egui::Ui, s: String| {
                        ui.label(egui::RichText::new(s).monospace().color(app_theme.syntax_number));
                    };

                    ui.horizontal(|ui| {
                        key(ui, "Size: ");
                        val(ui, format!("{}x{}", width, height));
                        ui.add_space(12.0);
                        key(ui, "Tile: ");
                        val(ui, format!("${}{:02X}", sprite.use_second_obj_table as u8, sprite.tile_idx));
                    });
                    ui.horizontal(|ui| {
                        key(ui, "Palette: ");
                        val(ui, format!("{}", sprite.palette));
                        ui.add_space(12.0);
                        key(ui, "Pos: ");
                        val(ui, format!("({}, {})", sprite.x, sprite.y));
                    });
                    ui.horizontal(|ui| {
                        key(ui, "Flip: ");
                        let flip_color = |on: bool| if on { app_theme.warning } else { app_theme.text_disabled };
                        ui.label(
                            egui::RichText::new("H")
                                .monospace()
                                .color(flip_color(sprite.flip_x)),
                        );
                        ui.label(
                            egui::RichText::new("V")
                                .monospace()
                                .color(flip_color(sprite.flip_y)),
                        );
                    });
                    ui.add_space(20.0);

                    // Scale up for visibility
                    let scale = 16.0;
                    let image_size = egui::Vec2::new(width as f32, height as f32) * scale;

                    ui.image(egui::load::SizedTexture::new(texture.id(), image_size));
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("Select a sprite from the list to view its texture.")
                                .color(app_theme.text_muted)
                                .italics(),
                        );
                    });
                }
            });
        }); // End of horizontal
    }

    fn render_vram_dump(ui: &mut egui::Ui, vram: &[u16], app_theme: &AppTheme) {
        const COLS: usize = 8;
        let total_rows = vram.len().div_ceil(COLS);
        let row_height = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                for row in row_range {
                    let base = row * COLS;
                    let chunk = &vram[base..vram.len().min(base + COLS)];

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:04X}:", base))
                                .monospace()
                                .color(app_theme.syntax_address),
                        );

                        for word in chunk {
                            let color = app_theme.memory_word_color(*word);
                            ui.label(
                                egui::RichText::new(format!(" {:04X}", word))
                                    .monospace()
                                    .color(color),
                            );
                        }
                    });
                }
            });
    }

    fn render_cgram_dump(ui: &mut egui::Ui, cgram: &[Color], app_theme: &AppTheme) {
        const COLS: usize = 16;
        let total_rows = cgram.len().div_ceil(COLS);
        let row_height = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                for row in row_range {
                    let base = row * COLS;
                    let chunk = &cgram[base..cgram.len().min(base + COLS)];
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:03X}:", base))
                                .monospace()
                                .color(app_theme.syntax_address),
                        );
                        for color in chunk {
                            let egui_color = egui::Color32::from_rgb(color.r, color.g, color.b);
                            // Color swatch
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(row_height, row_height),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(
                                rect,
                                app_theme.widget_corner_radius as f32,
                                egui_color,
                            );
                            ui.painter().rect_stroke(
                                rect,
                                app_theme.widget_corner_radius as f32,
                                egui::Stroke::new(1.0, app_theme.border),
                                egui::StrokeKind::Outside,
                            );
                            response.on_hover_text(format!(
                                "#{:02X}{:02X}{:02X}",
                                color.r, color.g, color.b
                            ));
                        }
                    });
                }
            });
    }

    fn render_byte_dump(ui: &mut egui::Ui, data: &[u8], addr_w: usize, app_theme: &AppTheme) {
        const COLS: usize = 16;

        // let anchor = self.mem.anchor() as usize;
        let total_rows = data.len().div_ceil(COLS);
        let row_height = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                for row in row_range {
                    let base = row * COLS;
                    let chunk = &data[base..data.len().min(base + COLS)];

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(
                                format!("{:0>width$X}:", base, width = addr_w), // Note: for ROM/WRAM the base IS the absolute offset since data starts at 0
                                                                                // For banked views you'd add a base_addr offset here
                            )
                            .monospace()
                            .color(app_theme.syntax_address),
                        );

                        // Hex bytes — group in sets of 8 for readability
                        for (i, &byte) in chunk.iter().enumerate() {
                            if i == 8 {
                                ui.label(egui::RichText::new("·").color(app_theme.text_muted));
                            }
                            ui.label(
                                egui::RichText::new(format!("{:02X}", byte))
                                .monospace()
                                .color(app_theme.memory_byte_color(byte))
                            );
                        }
                        // Pad if last row is short
                        for i in chunk.len()..COLS {
                            if i == 8 {
                                ui.label(egui::RichText::new("·").color(app_theme.text_muted));
                            }
                            ui.label(egui::RichText::new("   ").monospace());
                        }

                        ui.separator();

                        // ASCII sidebar
                        let ascii: String = chunk
                            .iter()
                            .map(|&b| {
                                if b.is_ascii_graphic() || b == b' ' {
                                    b as char
                                } else {
                                    '.'
                                }
                            })
                            .collect();

                        ui.label(
                            egui::RichText::new(ascii)
                            .monospace()
                            .color(app_theme.text_primary)
                        );
                    });
                }
            });
    }
}

fn render_sprite(
    sprite: &OAMSprite,
    vram: &[u16],
    cgram: &[Color],
    name_base: u16,
    name_second_base: u16,
    obj_sprite_size: ObjectSizeSelect,
) -> (Vec<u8>, usize, usize) {
    let (w, h) = sprite.sprite_size(obj_sprite_size);

    let mut pixels = vec![0u8; w * h * 4];

    for y in 0..h {
        for x in 0..w {
            // Apply H/V flips
            let src_x = if sprite.flip_x { w - 1 - x } else { x };
            let src_y = if sprite.flip_y { h - 1 - y } else { y };

            let tile_x = src_x / 8;
            let tile_y = src_y / 8;
            let px = src_x % 8;
            let py = src_y % 8;

            // SNES multi-tile offsets shift by 16 horizontally across the 16x16 256-tile page
            let tile_offset = tile_x + (tile_y * 16);

            // The 9th bit (0x100) dictates the name table base, lower 8 bits wrap
            let current_tile = (sprite.use_second_obj_table as u16) << 8 | ((sprite.tile_idx as u16 + tile_offset as u16) & 0xFF);

            let base_addr = if (current_tile & 0x100) == 0 {
                name_base
            } else {
                name_second_base
            };

            // 1 tile = 16 words. VRAM slice is presumed to be u16 word-indexed
            let tile_addr = base_addr as usize + (current_tile & 0xFF) as usize * 16;

            // Bounds protection
            if tile_addr + py + 8 >= vram.len() {
                continue;
            }

            // Planar 4bpp Decoding
            let w1 = vram[tile_addr + py];
            let w2 = vram[tile_addr + py + 8];

            let shift = 7 - px;
            let bp0 = ((w1 & 0xFF) >> shift) & 1;
            let bp1 = (((w1 >> 8) & 0xFF) >> shift) & 1;
            let bp2 = ((w2 & 0xFF) >> shift) & 1;
            let bp3 = (((w2 >> 8) & 0xFF) >> shift) & 1;

            let color_idx = (bp3 << 3) | (bp2 << 2) | (bp1 << 1) | bp0;
            let pixel_index = (y * w + x) * 4;

            if color_idx == 0 {
                // Transparent -> leave Alpha 0
                pixels[pixel_index] = 0;
                pixels[pixel_index + 1] = 0;
                pixels[pixel_index + 2] = 0;
                pixels[pixel_index + 3] = 0;
            } else {
                // Palettes 128-255. 8 palettes, 16 colors each.
                let cgram_idx = 128 + (sprite.palette as usize * 16) + color_idx as usize;

                if cgram_idx < cgram.len() {
                    let color = &cgram[cgram_idx];
                    pixels[pixel_index + 0] = color.r;
                    pixels[pixel_index + 1] = color.g;
                    pixels[pixel_index + 2] = color.b;
                    pixels[pixel_index + 3] = 255;
                }
            }
        }
    }
    (pixels, w, h)
}