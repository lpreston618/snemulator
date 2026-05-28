use snemcore::{Snemulator, sppu::{Color, ObjectSizeSelect}};

use crate::debug::{debugger::Debugger, texture::Texture};

#[derive(PartialEq, Clone, Copy)]
enum MemViewRegion { Wram, Rom, Vram, Oam, Cgram }

impl MemViewRegion {
    fn label(&self) -> &'static str {
        match self {
            MemViewRegion::Wram  => "WRAM",
            MemViewRegion::Rom   => "ROM",
            MemViewRegion::Vram  => "VRAM",
            MemViewRegion::Oam   => "OAM",
            MemViewRegion::Cgram => "CGRAM",
        }
    }
    // Address display width: WRAM/ROM are 24-bit, rest are 16-bit offsets into their own space
    fn addr_width(&self) -> usize {
        match self { MemViewRegion::Wram | MemViewRegion::Rom => 6, _ => 4 }
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

/// Parsed OAM sprite entry for display
struct OamSprite {
    index: usize,
    x: i16,           // X position (signed, can be negative for partial offscreen)
    y: u8,            // Y position
    tile: u16,        // Tile number (9 bits: high bit from attr, low 8 from tile byte)
    palette: u8,      // Palette (0-7)
    priority: u8,     // Priority (0-3)
    h_flip: bool,     // Horizontal flip
    v_flip: bool,     // Vertical flip
    size_large: bool, // Size select (false = small, true = large)
}

impl OamSprite {
    fn from_oam(oam: &[u8], index: usize) -> Self {
        // Main table: 4 bytes per sprite at offset index * 4
        let base = index * 4;
        let x_low = oam[base] as u16;
        let y = oam[base + 1];
        let tile_low = oam[base + 2] as u16;
        let attr = oam[base + 3];

        // Extended table: 2 bits per sprite starting at offset 512
        // Each byte holds data for 4 sprites
        let ext_byte_idx = 512 + (index / 4);
        let ext_bit_shift = (index % 4) * 2;
        let ext_bits = (oam[ext_byte_idx] >> ext_bit_shift) & 0x03;

        let x_high = (ext_bits & 0x01) != 0;
        let size_large = (ext_bits & 0x02) != 0;

        // X position is 9-bit signed (bit 8 from ext table)
        let x_full = x_low | ((x_high as u16) << 8);
        let x = if x_full >= 256 {
            x_full as i16 - 512
        } else {
            x_full as i16
        };

        // Tile number: bit 8 from attr bit 0, low 8 bits from tile byte
        let tile = tile_low | (((attr & 0x01) as u16) << 8);

        // Attributes: vhoopppc
        let palette = (attr >> 1) & 0x07;
        let priority = (attr >> 4) & 0x03;
        let h_flip = (attr & 0x40) != 0;
        let v_flip = (attr & 0x80) != 0;

        OamSprite {
            index,
            x,
            y,
            tile,
            palette,
            priority,
            h_flip,
            v_flip,
            size_large,
        }
    }
}

pub struct MemoryTab {
    region: MemViewRegion,
    oam_view_mode: OamViewMode,
    sprite_texture: Texture,
    selected_sprite: Option<usize>,
}

impl MemoryTab {
    pub fn new(painter: &mut egui_glow::Painter) -> Self {
        Self {
            region: MemViewRegion::Wram,
            oam_view_mode: OamViewMode::Sprites,
            // Initialize at maximum possible SNES sprite size
            sprite_texture: Texture::new(painter, 64, 64),
            selected_sprite: None,
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &Snemulator<Debugger>) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("mem_region")
                .selected_text(self.region.label())
                .show_ui(ui, |ui| {
                    for region in [
                        MemViewRegion::Wram, 
                        MemViewRegion::Rom,
                        MemViewRegion::Vram,
                        MemViewRegion::Oam,
                        MemViewRegion::Cgram,
                    ] {
                        ui.selectable_value(&mut self.region, region, region.label());
                    }
                });

            // Show OAM mode toggle if OAM is selected
            if self.region == MemViewRegion::Oam {
                ui.separator();
                egui::ComboBox::from_id_salt("oam_mode")
                    .selected_text(self.oam_view_mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.oam_view_mode, OamViewMode::Raw, OamViewMode::Raw.label());
                        ui.selectable_value(&mut self.oam_view_mode, OamViewMode::Sprites, OamViewMode::Sprites.label());
                    });
            }
        });
        ui.separator();

        let addr_w = self.region.addr_width();

        match self.region {
            MemViewRegion::Vram  => Self::render_vram_dump(ui, &snem_core.vram[..]),
            MemViewRegion::Cgram => Self::render_cgram_dump(ui, &snem_core.cgram[..]),
            MemViewRegion::Oam if self.oam_view_mode == OamViewMode::Sprites => {
                self.render_oam_sprites(ui, snem_core);
            }
            _ => {
                let data: &[u8] = match self.region {
                    MemViewRegion::Wram => &snem_core.wram[..],
                    MemViewRegion::Rom  => &snem_core.rom_slice(),
                    MemViewRegion::Oam  => &snem_core.oam[..],
                    _                   => unreachable!(),
                };
                Self::render_byte_dump(ui, data, addr_w);
            }
        }
    }

    fn render_oam_sprites(&mut self, ui: &mut egui::Ui, core: &Snemulator<Debugger>) {
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
                            egui::Grid::new("oam_sprites_grid").striped(true).show(ui, |ui| {
                                ui.label("Idx");
                                ui.label("X, Y");
                                ui.label("Tile");
                                ui.label("Pal");
                                ui.label("Pri");
                                ui.label("Size");
                                ui.end_row();

                                for i in 0..128 {
                                    let sprite = OamSprite::from_oam(&core.oam[..], i);
                                    
                                    // Dim off-screen or unused sprites
                                    let is_active = sprite.y < 224 && sprite.x > -64 && sprite.x < 256;
                                    let color = if is_active { egui::Color32::WHITE } else { egui::Color32::DARK_GRAY };

                                    let is_selected = self.selected_sprite == Some(i);
                                    if ui.selectable_label(is_selected, format!("{:03}", i)).clicked() {
                                        self.selected_sprite = Some(i);
                                    }
                                    
                                    ui.colored_label(color, format!("{}, {}", sprite.x, sprite.y));
                                    ui.colored_label(color, format!("${:03X}", sprite.tile));
                                    ui.colored_label(color, format!("{}", sprite.palette));
                                    ui.colored_label(color, format!("{}", sprite.priority));
                                    ui.colored_label(color, if sprite.size_large { "L" } else { "S" });
                                    ui.end_row();
                                }
                            });
                        });
                }
            );

            ui.separator(); 

            ui.vertical(|ui| {
                if let Some(idx) = self.selected_sprite {
                    let sprite = OamSprite::from_oam(&core.oam[..], idx);
                    
                    let (pixels, width, height) = decode_sprite(
                        &sprite,
                        &core.vram[..],
                        &core.cgram[..],
                        core.ppu_regs.obj_sprite_size,
                        core.ppu_regs.name_base_addr,
                        core.ppu_regs.name_secondary_base_addr,
                    );

                    // Resize the OpenGL texture dynamically to tightly fit the current sprite
                    if self.sprite_texture.resize(width, height).is_ok() {
                        self.sprite_texture.update_texture(&pixels);
                    }

                    ui.heading(format!("Sprite {:03}", idx));
                    ui.horizontal(|ui| {
                        ui.label(format!("Size: {}x{}", width, height));
                        ui.label(format!("Tile: ${:03X}", sprite.tile));
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Palette: {}", sprite.palette));
                        ui.label(format!("Pos: ({}, {})", sprite.x, sprite.y));
                    });
                    ui.label(format!("Flip: H:{}, V:{}", sprite.h_flip, sprite.v_flip));
                    ui.add_space(20.0);

                    // Scale up for visibility
                    let scale = 16.0;
                    let image_size = egui::Vec2::new(width as f32, height as f32) * scale;
                    
                    // Render the texture
                    ui.image(egui::load::SizedTexture::new(self.sprite_texture.texture_id(), image_size));
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a sprite from the list to view its texture.");
                    });
                }
            });
        }); // End of horizontal
    }
    
    fn render_vram_dump(ui: &mut egui::Ui, vram: &[u16]) {
        const COLS: usize = 8;
        let total_rows = vram.len().div_ceil(COLS);
        let row_height = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                for row in row_range {
                    let base  = row * COLS;
                    let chunk = &vram[base..vram.len().min(base + COLS)];
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{:04X}:", base)).monospace().weak());
                        for word in chunk {
                            ui.label(egui::RichText::new(format!(" {:04X}", word)).monospace());
                        }
                    });
                }
            });
    }
    
    fn render_cgram_dump(ui: &mut egui::Ui, cgram: &[Color]) {
        const COLS: usize = 16;
        let total_rows = cgram.len().div_ceil(COLS);
        let row_height = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;
    
        egui::ScrollArea::vertical().auto_shrink([false, false])
            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                for row in row_range {
                    let base  = row * COLS;
                    let chunk = &cgram[base..cgram.len().min(base + COLS)];
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{:03X}:", base)).monospace().weak());
                        for color in chunk {
                            let egui_color = egui::Color32::from_rgb(color.r, color.g, color.b);
                            // Color swatch
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(row_height, row_height),
                                egui::Sense::hover()
                            );
                            ui.painter().rect_filled(rect, 1.0, egui_color);
                            response.on_hover_text(format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b));
                        }
                    });
                }
            });
    }
    
    fn render_byte_dump(ui: &mut egui::Ui, data: &[u8], addr_w: usize) {
        const COLS: usize = 16;
    
        // let anchor = self.mem.anchor() as usize;
        let total_rows  = data.len().div_ceil(COLS);
        let row_height  = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;

        egui::ScrollArea::vertical().auto_shrink([false, false])
            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                for row in row_range {
                    let base = row * COLS;
                    let chunk = &data[base..data.len().min(base + COLS)];
        
                    ui.horizontal(|ui| {
                        // Address gutter
                        ui.label(egui::RichText::new(
                            format!("{:0>width$X}:", base, width = addr_w)
                            // Note: for ROM/WRAM the base IS the absolute offset since data starts at 0
                            // For banked views you'd add a base_addr offset here
                        ).monospace().weak());
        
                        // Hex bytes — group in sets of 8 for readability
                        for (i, byte) in chunk.iter().enumerate() {
                            if i == 8 { ui.label(egui::RichText::new("·").weak()); }
                            ui.label(egui::RichText::new(format!("{:02X}", byte)).monospace());
                        }
                        // Pad if last row is short
                        for i in chunk.len()..COLS {
                            if i == 8 { ui.label(egui::RichText::new("·").weak()); }
                            ui.label(egui::RichText::new("   ").monospace());
                        }
        
                        ui.separator();
        
                        // ASCII sidebar
                        let ascii: String = chunk.iter().map(|&b| {
                            if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }
                        }).collect();
                        ui.label(egui::RichText::new(ascii).monospace().weak());
                    });
                }
            });
    }
}

fn decode_sprite(
    sprite: &OamSprite,
    vram: &[u16],
    cgram: &[Color], // Update this type name to match your codebase
    obsel: ObjectSizeSelect,
    name_base: u16,
    name_second_base: u16,
) -> (Vec<u8>, usize, usize) {
    // 1. Determine physical dimensions
    let (w, h) = match (obsel, sprite.size_large) {
        (ObjectSizeSelect::Size8x8_16x16, false) => (8, 8),
        (ObjectSizeSelect::Size8x8_16x16, true)  => (16, 16),
        (ObjectSizeSelect::Size8x8_32x32, false) => (8, 8),
        (ObjectSizeSelect::Size8x8_32x32, true)  => (32, 32),
        (ObjectSizeSelect::Size8x8_64x64, false) => (8, 8),
        (ObjectSizeSelect::Size8x8_64x64, true)  => (64, 64),
        (ObjectSizeSelect::Size16x16_32x32, false) => (16, 16),
        (ObjectSizeSelect::Size16x16_32x32, true)  => (32, 32),
        (ObjectSizeSelect::Size16x16_64x64, false) => (16, 16),
        (ObjectSizeSelect::Size16x16_64x64, true)  => (64, 64),
        (ObjectSizeSelect::Size32x32_64x64, false) => (32, 32),
        (ObjectSizeSelect::Size32x32_64x64, true)  => (64, 64),
        (ObjectSizeSelect::Size16x32_32x64, false) => (16, 32),
        (ObjectSizeSelect::Size16x32_32x64, true)  => (32, 64),
        (ObjectSizeSelect::Size16x32_32x32, false) => (16, 32),
        (ObjectSizeSelect::Size16x32_32x32, true)  => (32, 32),
    };

    let mut pixels = vec![0u8; w * h * 4];

    for y in 0..h {
        for x in 0..w {
            // Apply H/V flips
            let src_x = if sprite.h_flip { w - 1 - x } else { x };
            let src_y = if sprite.v_flip { h - 1 - y } else { y };

            let tile_x = src_x / 8;
            let tile_y = src_y / 8;
            let px = src_x % 8;
            let py = src_y % 8;

            // SNES multi-tile offsets shift by 16 horizontally across the 16x16 256-tile page
            let tile_offset = tile_x + (tile_y * 16);
            
            // The 9th bit (0x100) dictates the name table base, lower 8 bits wrap
            let current_tile = (sprite.tile & 0x100) | ((sprite.tile + tile_offset as u16) & 0xFF);

            let base_addr = if (current_tile & 0x100) == 0 { name_base } else { name_second_base };
            
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