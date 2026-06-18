use snemcore::{Snemulator, sppu::{self, BgMode, BgSettings, Color, TileSize, TilemapCount}, sysinfo::VRAM_SIZE};

use crate::debug::{harness::MainDebugHarness};
use egui::{Color32, ColorImage, TextureHandle, TextureOptions, Vec2};

pub struct BgDebugView<const BG_LAYER: usize> {
    /// The renderer for this background
    renderer: BgRenderer,
    /// CPU-side pixel buffer (RGBA8888)
    pixel_buffer: Vec<u8>,
    /// Cached texture handle for egui
    texture: Option<TextureHandle>,
    /// Last rendered dimensions
    rendered_size: (u32, u32),
    /// Zoom level for display
    pub zoom: f32,
    /// Whether to show the screen viewport overlay
    show_viewport: bool,
    /// How much extra tilemap to show around the screen (in screens)
    /// 1.0 = show one extra screen worth on each side
    padding_screens: f32,
    /// The tile coordinate (tile_x, tile_y) that was last hovered over
    hovered_tile: Option<(u32, u32)>,
    /// The tile coordinate (tile_x, tile_y) that was last clicked
    selected_tile: Option<(u32, u32)>,
}

/// Split a UV range that may cross the 1.0 boundary into at most two clamped segments.
/// Returns a list of `(uv_start, uv_end, rect_norm_start, rect_norm_end)` tuples,
/// where uv_* are in [0, 1] and rect_norm_* are the proportional positions in the display rect.
fn uv_segments(uv_start: f32, uv_end: f32) -> Vec<(f32, f32, f32, f32)> {
    let total = uv_end - uv_start;
    if uv_end <= 1.0 {
        // No wrap — single segment
        vec![(uv_start, uv_end, 0.0, 1.0)]
    } else {
        // Wraps past 1.0 — split at 1.0
        let split = (1.0 - uv_start) / total; // normalised rect position of the wrap point
        vec![
            (uv_start, 1.0, 0.0, split),
            (0.0, uv_end - 1.0, split, 1.0),
        ]
    }
}

impl<const BG_LAYER: usize> BgDebugView<BG_LAYER> {
    pub fn new() -> Self {
        Self {
            renderer: BgRenderer::new(),
            pixel_buffer: Vec::new(),
            texture: None,
            rendered_size: (0, 0),
            zoom: 1.0,
            show_viewport: true,
            padding_screens: 0.5,
            hovered_tile: None,
            selected_tile: None,
        }
    }

    fn color_depth(&self, core: &Snemulator) -> u8 {
        match core.ppu_regs.bg_mode {
            BgMode::Mode0 => 2,
            BgMode::Mode1 if BG_LAYER == 0 || BG_LAYER == 1 => 4,
            BgMode::Mode1 if BG_LAYER == 2 => 2,
            BgMode::Mode2 if BG_LAYER == 0 || BG_LAYER == 1 => 4,
            BgMode::Mode3 if BG_LAYER == 0 => 8,
            BgMode::Mode3 if BG_LAYER == 1 => 4,
            BgMode::Mode4 if BG_LAYER == 0 => 8,
            BgMode::Mode4 if BG_LAYER == 1 => 2,
            BgMode::Mode5 if BG_LAYER == 0 => 4,
            BgMode::Mode5 if BG_LAYER == 1 => 2,
            BgMode::Mode6 if BG_LAYER == 0 => 4,
            BgMode::Mode7 if BG_LAYER == 0 => 8,
            _ => 0,
        }
    }

    /// Update the background texture from current PPU state
    pub fn update(&mut self, core: &Snemulator, harness: &mut MainDebugHarness) {
        // Only re-render when VRAM/CGRAM/tilemap registers change
        if !self.needs_update(core, harness) {
            return;
        }

        let color_depth = self.color_depth(core);

        if color_depth == 0 {
            self.texture = None;
            
            return;
        }

        // Render the background layer
        self.rendered_size = self.renderer.render_bg_layer(
            &mut self.pixel_buffer,
            &core.vram,
            &core.ppu_regs.bg_settings[BG_LAYER],
            &core.cgram,
            color_depth,
            core.ppu_regs.use_direct_col,
        );
        
        // Invalidate texture so it gets re-uploaded on next render
        self.texture = None;
    }

    fn needs_update(&mut self, _core: &Snemulator, _harness: &mut MainDebugHarness) -> bool {
        true // TODO: Track vram writes and bg settings to determine when bg needs re-rendering
    }

    /// Render the debug view UI
    pub fn render(&mut self, ui: &mut egui::Ui, core: &Snemulator) {
        let bg_settings = &core.ppu_regs.bg_settings[BG_LAYER];

        // Controls panel
        ui.horizontal(|ui| {
            ui.label(format!("BG{}", BG_LAYER + 1));
            ui.separator();

            ui.label("Zoom:");
            ui.add(egui::Slider::new(&mut self.zoom, 0.25..=4.0).logarithmic(true));

            ui.separator();
            ui.checkbox(&mut self.show_viewport, "Show viewport");

            ui.separator();
            ui.label(format!(
                "{}x{} | {}bpp | Tile: {:?}",
                self.rendered_size.0,
                self.rendered_size.1,
                self.color_depth(core),
                bg_settings.chr_size,
            ));
        });

        ui.separator();

        // Ensure texture is uploaded
        self.ensure_texture(ui.ctx());

        // Side panel for tile info — reserves space before the background view is laid out,
        // so the scroll area always gets the correct remaining width.
        egui::SidePanel::right(format!("bg{}_tile_info", BG_LAYER))
            .resizable(true)
            .min_width(160.0)
            .show_inside(ui, |ui| {
                self.render_tile_info_panel(ui, core);
            });

        // Background tilemap view fills all remaining space
        let available_size = ui.available_size();
        egui::ScrollArea::both()
            .max_width(available_size.x)
            .max_height(available_size.y)
            .show(ui, |ui| {
                self.render_scrolling_background(ui, core);
            });
    }

    /// Ensure the texture is uploaded to the GPU
    fn ensure_texture(&mut self, ctx: &egui::Context) {
        if self.texture.is_some() {
            return;
        }

        if self.pixel_buffer.is_empty() || self.rendered_size.0 == 0 || self.rendered_size.1 == 0 {
            return;
        }

        // Convert u32 RGBA to egui's ColorImage
        let pixels: Vec<Color32> = self.pixel_buffer
            .chunks(4)
            .map(|pixel| {
                Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3])
            })
            .collect();

        let size = [self.rendered_size.0 as usize, self.rendered_size.1 as usize];
        let image = ColorImage {
            size,
            pixels,
            source_size: egui::Vec2::new(self.rendered_size.0 as f32, self.rendered_size.1 as f32),
        };

        let texture = ctx.load_texture(
            format!("bg{}_debug", BG_LAYER + 1),
            image,
            TextureOptions::NEAREST, // Crispy pixels!
        );

        self.texture = Some(texture);
    }

    /// Render the background image with viewport overlay
    fn render_scrolling_background(&mut self, ui: &mut egui::Ui, core: &Snemulator) {
        let Some(texture) = &self.texture else {
            ui.label("No background rendered");
            return;
        };

        let bg_settings = &core.ppu_regs.bg_settings[BG_LAYER];

        // Screen dimensions
        let screen_w = 256.0;
        let screen_h = if core.ppu_regs.overscan_en { 239.0 } else { 224.0 };

        // Tilemap dimensions
        let tilemap_w = self.rendered_size.0 as f32;
        let tilemap_h = self.rendered_size.1 as f32;

        if tilemap_w == 0.0 || tilemap_h == 0.0 {
            return;
        }

        // Calculate display size: screen + padding on each side
        let padding_x = screen_w * self.padding_screens;
        let padding_y = screen_h * self.padding_screens;
        let display_w = (screen_w + padding_x * 2.0) * self.zoom;
        let display_h = (screen_h + padding_y * 2.0) * self.zoom;

        // How many "tilemaps" worth of UV space we need to cover the display
        let uv_width = (screen_w + padding_x * 2.0) / tilemap_w;
        let uv_height = (screen_h + padding_y * 2.0) / tilemap_h;

        // Scroll position in UV space (0.0 to 1.0 = one tilemap)
        let scroll_u = bg_settings.scroll_x as f32 / tilemap_w;
        let scroll_v = bg_settings.scroll_y as f32 / tilemap_h;

        // UV offset: we want the scroll position to appear at the screen box location
        // Screen box is centered, so scroll position should be at (padding / total_size) from UV start
        let padding_u = padding_x / tilemap_w;
        let padding_v = padding_y / tilemap_h;

        // Normalise UV origin into [0, 1) so the painter never receives out-of-range UVs.
        // egui's painter clamps rather than wraps, which would stretch edge pixels.
        let uv_min_x = (scroll_u - padding_u).rem_euclid(1.0);
        let uv_min_y = (scroll_v - padding_v).rem_euclid(1.0);
        let uv_max_x = uv_min_x + uv_width;
        let uv_max_y = uv_min_y + uv_height;

        // Build a UV rect representing the (possibly >1.0) range starting from the normalised origin.
        // We use this for hover/tooltip math only; actual painting is done per-tile below.
        let uv = egui::Rect::from_min_max(
            egui::pos2(uv_min_x, uv_min_y),
            egui::pos2(uv_max_x, uv_max_y),
        );

        // Clamp uniformly so the aspect ratio is preserved. If the desired size fits, scale = 1.0.
        let available = ui.available_size();
        let scale = (available.x / display_w).min(available.y / display_h).min(1.0);
        let display_size = Vec2::new(display_w * scale, display_h * scale);
        let effective_zoom = self.zoom * scale;

        let (rect, response) = ui.allocate_exact_size(display_size, egui::Sense::hover() | egui::Sense::click());

        // Draw the texture in tiled segments so UVs always stay within [0, 1].
        // When the UV range crosses 1.0 we split into two strips (left + right, or top + bottom,
        // or up to four quadrants), each clamped to [0, 1].
        let tex_id = texture.id();
        let painter = ui.painter();

        // Collect the one or two UV segments in each axis.
        // Each entry is (uv_start, uv_end, norm_rect_start, norm_rect_end)
        // where norm_rect_* are in [0,1] normalised rect space.
        let x_segs = uv_segments(uv_min_x, uv_max_x);
        let y_segs = uv_segments(uv_min_y, uv_max_y);

        for (uv_x0, uv_x1, rx0, rx1) in &x_segs {
            for (uv_y0, uv_y1, ry0, ry1) in &y_segs {
                let seg_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x + rx0 * rect.width(), rect.min.y + ry0 * rect.height()),
                    egui::pos2(rect.min.x + rx1 * rect.width(), rect.min.y + ry1 * rect.height()),
                );
                let seg_uv = egui::Rect::from_min_max(
                    egui::pos2(*uv_x0, *uv_y0),
                    egui::pos2(*uv_x1, *uv_y1),
                );
                painter.image(tex_id, seg_rect, seg_uv, Color32::WHITE);
            }
        }

        // Draw the fixed, centered screen viewport
        if self.show_viewport {
            self.draw_centered_viewport(ui, rect, screen_w, screen_h, effective_zoom);
        }

        // Hover highlight and click-to-select
        self.hovered_tile = if response.hovered() {
            response.hover_pos().and_then(|pos| {
                self.pos_to_tile_coords(pos, rect, uv)
            })
        } else {
            None
        };

        if response.clicked() {
            if self.hovered_tile.is_some() && self.selected_tile.is_some()
                && self.hovered_tile.unwrap() == self.selected_tile.unwrap() {
                self.selected_tile = None;
            } else {
                self.selected_tile = self.hovered_tile;
            }
        }

        // Draw highlight for hovered tile (red) and selected tile (yellow), if distinct
        if let Some((tx, ty)) = self.selected_tile {
            self.draw_tile_highlight(ui, tx, ty, rect, uv, Color32::from_rgb(255, 210, 50));
        }
        if let Some((tx, ty)) = self.hovered_tile {
            if self.hovered_tile != self.selected_tile {
                self.draw_tile_highlight(ui, tx, ty, rect, uv, Color32::from_rgb(255, 80, 80));
            }
        }
    }

    /// Draw the screen viewport centered in the display area
    fn draw_centered_viewport(
        &self,
        ui: &mut egui::Ui,
        image_rect: egui::Rect,
        screen_w: f32,
        screen_h: f32,
        effective_zoom: f32,
    ) {
        let screen_display_w = screen_w * effective_zoom;
        let screen_display_h = screen_h * effective_zoom;

        let screen_rect = egui::Rect::from_center_size(
            image_rect.center(),
            egui::vec2(screen_display_w, screen_display_h),
        );

        let stroke_width = (2.0 * effective_zoom.max(1.0)).min(4.0);
        ui.painter().rect_stroke(
            screen_rect,
            0.0,
            egui::Stroke::new(stroke_width, Color32::from_rgb(80, 140, 255)),
            egui::StrokeKind::Outside,
        );

        let font_size = (12.0 * effective_zoom.max(1.0)).min(16.0);
        ui.painter().text(
            screen_rect.center_top() + egui::vec2(0.0, -4.0 * effective_zoom.max(1.0)),
            egui::Align2::CENTER_BOTTOM,
            "Screen",
            egui::FontId::proportional(font_size),
            Color32::from_rgb(80, 140, 255),
        );
    }

    /// Convert a screen position to tile coordinates (tile_x, tile_y), or None if out of bounds.
    fn pos_to_tile_coords(&self, pos: egui::Pos2, image_rect: egui::Rect, uv: egui::Rect) -> Option<(u32, u32)> {
        let tilemap_w = self.rendered_size.0 as f32;
        let tilemap_h = self.rendered_size.1 as f32;
        if tilemap_w == 0.0 || tilemap_h == 0.0 { return None; }

        let norm_x = (pos.x - image_rect.min.x) / image_rect.width();
        let norm_y = (pos.y - image_rect.min.y) / image_rect.height();
        let u = uv.min.x + norm_x * uv.width();
        let v = uv.min.y + norm_y * uv.height();

        let pixel_x = (u * tilemap_w).rem_euclid(tilemap_w) as u32;
        let pixel_y = (v * tilemap_h).rem_euclid(tilemap_h) as u32;
        Some((pixel_x / 8, pixel_y / 8))
    }

    /// Draw a highlight outline around a specific tile coordinate.
    fn draw_tile_highlight(
        &self,
        ui: &mut egui::Ui,
        tile_x: u32,
        tile_y: u32,
        image_rect: egui::Rect,
        uv: egui::Rect,
        color: Color32,
    ) {
        let tilemap_w = self.rendered_size.0 as f32;
        let tilemap_h = self.rendered_size.1 as f32;
        if tilemap_w == 0.0 || tilemap_h == 0.0 { return; }

        let tile_u_min = (tile_x * 8) as f32 / tilemap_w;
        let tile_v_min = (tile_y * 8) as f32 / tilemap_h;

        let u_offset = (tile_u_min - uv.min.x).rem_euclid(uv.width());
        let v_offset = (tile_v_min - uv.min.y).rem_euclid(uv.height());

        let screen_x = image_rect.min.x + (u_offset / uv.width()) * image_rect.width();
        let screen_y = image_rect.min.y + (v_offset / uv.height()) * image_rect.height();
        let tile_screen_w = (8.0 / tilemap_w) * (image_rect.width() / uv.width());
        let tile_screen_h = (8.0 / tilemap_h) * (image_rect.height() / uv.height());

        let stroke_width = (1.5 * self.zoom.max(1.0)).min(3.0);
        ui.painter().rect_stroke(
            egui::Rect::from_min_size(egui::pos2(screen_x, screen_y), egui::vec2(tile_screen_w, tile_screen_h)),
            0.0,
            egui::Stroke::new(stroke_width, color),
            egui::StrokeKind::Outside,
        );
    }

    /// Render the tile info panel shown to the right of the background view.
    fn render_tile_info_panel(&mut self, ui: &mut egui::Ui, core: &Snemulator) {
        let bg_settings = &core.ppu_regs.bg_settings[BG_LAYER];

        if self.selected_tile.is_none() && self.hovered_tile.is_none() {
            ui.label("Click a tile to inspect it.");
            return;
        }

        let tile_x: u32;
        let tile_y: u32;

        if self.selected_tile.is_some() {
            ui.label("Click selected tile to de-select it.");
            (tile_x, tile_y) = self.selected_tile.unwrap();
        } else {
            ui.label("Click a tile to inspect it.");
            (tile_x, tile_y) = self.hovered_tile.unwrap();
        }

        let tilemap_addr = self.renderer.calc_tilemap_addr(bg_settings, tile_x, tile_y);
        let entry = core.vram[(tilemap_addr & 0x7FFF) as usize];

        let tile_num = entry & 0x3FF;
        let palette  = (entry >> 10) & 0x7;
        let priority = (entry >> 13) & 0x1;
        let flip_x   = entry & 0x4000 != 0;
        let flip_y   = entry & 0x8000 != 0;

        // --- Magnified tile preview (re-rendered every frame to reflect live VRAM changes) ---
        let preview_size = 128.0;
        {
            let color_depth = self.color_depth(core);
            let color_depth = if color_depth == 0 { 4 } else { color_depth };

            let mut buf = vec![0u8; 4 * 8 * 8];
            BgRenderer::fill_checkerboard(&mut buf, 8, 8);
            let chr_addr = self.renderer.calc_chr_addr(bg_settings, tile_num, 0, 0, color_depth);
            self.renderer.render_8x8_tile(
                &mut buf, &core.vram, &core.cgram,
                color_depth, chr_addr,
                palette as u8, flip_x, flip_y,
                0, 0, 8,
                core.ppu_regs.use_direct_col,
            );

            let pixels: Vec<Color32> = buf.chunks(4)
                .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                .collect();
            let tex = ui.ctx().load_texture(
                format!("bg{}_tile_preview", BG_LAYER + 1),
                ColorImage { size: [8, 8], pixels, source_size: egui::Vec2::splat(8.0) },
                TextureOptions::NEAREST,
            );
            ui.image(egui::load::SizedTexture::new(tex.id(), egui::vec2(preview_size, preview_size)));
        }

        // --- Tile metadata ---
        ui.separator();
        ui.label(format!("Tile:     ({}, {})", tile_x, tile_y));
        ui.label(format!("Tile #:   0x{:03X}  ({})", tile_num, tile_num));
        ui.label(format!("Palette:  {}", palette));
        ui.label(format!("Priority: {}", priority));
        ui.label(format!("Flip:     X={}  Y={}", flip_x, flip_y));
        ui.separator();
        ui.label(format!("Tilemap addr: 0x{:04X}", tilemap_addr));
        ui.label(format!("Entry:        0x{:04X}", entry));
    }
}

/// Background layer renderer for debug views
pub struct BgRenderer {
    /// Direct color LUT: [palette_bits][pixel_value] -> RGBA32
    direct_color_lut: [[u32; 256]; 8],
}

impl BgRenderer {
    pub fn new() -> Self {
        let mut renderer = Self {
            direct_color_lut: [[0; 256]; 8],
        };
        renderer.init_direct_color_lut();
        renderer
    }

    /// Initialize the direct color lookup table
    fn init_direct_color_lut(&mut self) {
        for palette_bits in 0..8u8 {
            let r_lsb = palette_bits & 0x1;
            let g_lsb = (palette_bits >> 1) & 0x1;
            let b_lsb = (palette_bits >> 2) & 0x1;

            for pixel in 0..256u16 {
                let pixel = pixel as u8;

                let r_high = pixel & 0x7;
                let g_high = (pixel >> 3) & 0x7;
                let b_high = (pixel >> 6) & 0x3;

                let r5 = (r_high << 2) | (r_lsb << 1);
                let g5 = (g_high << 2) | (g_lsb << 1);
                let b5 = (b_high << 3) | (b_lsb << 2);

                let r8 = (r5 << 3) | (r5 >> 2);
                let g8 = (g5 << 3) | (g5 >> 2);
                let b8 = (b5 << 3) | (b5 >> 2);

                self.direct_color_lut[palette_bits as usize][pixel as usize] =
                    u32::from_ne_bytes([r8, g8, b8, 255]);
            }

            // Pixel 0 is transparent
            self.direct_color_lut[palette_bits as usize][0] = 0;
        }
    }

    /// Fill a pixel buffer with a 8x8 checkerboard pattern for transparent background display.
    fn fill_checkerboard(pixel_buffer: &mut [u8], width_px: u32, height_px: u32) {
        const LIGHT: [u8; 4] = [180, 180, 180, 255];
        const DARK:  [u8; 4] = [120, 120, 120, 255];
        for y in 0..height_px {
            for x in 0..width_px {
                let color = if (x / 8 + y / 8) % 2 == 0 { LIGHT } else { DARK };
                let i = 4 * (y * width_px + x) as usize;
                pixel_buffer[i..i + 4].copy_from_slice(&color);
            }
        }
    }

    /// Render a full background layer to a pixel buffer
    /// 
    /// Returns (width, height) in pixels
    pub fn render_bg_layer(
        &mut self,
        pixel_buffer: &mut Vec<u8>,
        vram: &[u16; VRAM_SIZE],
        bg: &BgSettings,
        cgram: &[Color; 256],
        color_depth: u8,
        direct_color: bool,
    ) -> (u32, u32) {
        // Tilemap dimensions in 32x32-tile screens
        let screens_x = match bg.tilemap_cnt_x {
            sppu::TilemapCount::One => 1,
            sppu::TilemapCount::Two => 2,
        };
        let screens_y = match bg.tilemap_cnt_y {
            sppu::TilemapCount::One => 1,
            sppu::TilemapCount::Two => 2,
        };

        // Total tilemap size in 8x8 tiles
        let tilemap_width_tiles = screens_x * 32;
        let tilemap_height_tiles = screens_y * 32;

        // Output dimensions in pixels
        let width_px = tilemap_width_tiles * 8;
        let height_px = tilemap_height_tiles * 8;

        pixel_buffer.resize(4 * (width_px * height_px) as usize, 0);
        Self::fill_checkerboard(pixel_buffer, width_px, height_px);

        // Iterate over tilemap entries
        for tilemap_y in 0..tilemap_height_tiles {
            for tilemap_x in 0..tilemap_width_tiles {
                self.render_tilemap_entry(
                    pixel_buffer,
                    vram,
                    cgram,
                    bg,
                    tilemap_x,
                    tilemap_y,
                    width_px,
                    color_depth,
                    direct_color,
                );
            }
        }

        (width_px, height_px)
    }

    /// Render a single tilemap entry (handles both 8x8 and 16x16 tile modes)
    fn render_tilemap_entry(
        &self,
        pixel_buffer: &mut [u8],
        vram: &[u16; VRAM_SIZE],
        cgram: &[Color; 256],
        bg: &BgSettings,
        tilemap_x: u32,
        tilemap_y: u32,
        stride: u32,
        color_depth: u8,
        direct_color: bool,
    ) {
        let tilemap_addr = self.calc_tilemap_addr(bg, tilemap_x, tilemap_y);
        let entry = vram[tilemap_addr as usize];

        let tile_num = entry & 0x3FF;
        let palette = ((entry >> 10) & 0x7) as u8;
        let flip_x = entry & 0x4000 != 0;
        let flip_y = entry & 0x8000 != 0;

        let output_x = tilemap_x * 8;
        let output_y = tilemap_y * 8;

        if matches!(bg.chr_size, TileSize::Size16x16) {
            // A 16x16 tile occupies 2x2 tilemap entries. Only render the full 2x2 block
            // when we are at the top-left entry of the group (even tile coordinates).
            // Odd entries are covered by the previous even entry's render, so skip them.
            if tilemap_x % 2 != 0 || tilemap_y % 2 != 0 {
                return;
            }

            // Render all four 8x8 quadrants
            for sub_y in 0..2u32 {
                for sub_x in 0..2u32 {
                    // Flip swaps which quadrant is drawn at which position
                    let chr_sub_x = if flip_x { 1 - sub_x } else { sub_x };
                    let chr_sub_y = if flip_y { 1 - sub_y } else { sub_y };

                    let chr_addr = self.calc_chr_addr(bg, tile_num, chr_sub_x, chr_sub_y, color_depth);
                    self.render_8x8_tile(
                        pixel_buffer, vram, cgram, color_depth, chr_addr,
                        palette, flip_x, flip_y,
                        output_x + sub_x * 8, output_y + sub_y * 8,
                        stride, direct_color,
                    );
                }
            }
        } else {
            let chr_addr = self.calc_chr_addr(bg, tile_num, 0, 0, color_depth);
            self.render_8x8_tile(
                pixel_buffer, vram, cgram, color_depth, chr_addr,
                palette, flip_x, flip_y,
                output_x, output_y,
                stride, direct_color,
            );
        }
    }

    /// Calculate tilemap word address for a given tile coordinate
    fn calc_tilemap_addr(&self, bg: &BgSettings, tile_x: u32, tile_y: u32) -> u16 {
        // Each 32x32 screen is 0x400 words (1KB)
        // Within a screen, tiles are arranged row-major: addr = y*32 + x
        
        let screen_x = tile_x / 32;
        let screen_y = tile_y / 32;
        let local_x = tile_x % 32;
        let local_y = tile_y % 32;

        // Screen offset based on tilemap configuration
        // Single: just screen 0
        // Wide (cnt_x): screen 0, screen 1 horizontally
        // Tall (cnt_y): screen 0, screen 1 vertically  
        // Both: screen 0, 1, 2, 3 in order: 0 1
        //                                   2 3
        let screen_offset = if matches!(bg.tilemap_cnt_x, TilemapCount::Two) && matches!(bg.tilemap_cnt_y, TilemapCount::Two) {
            // 2x2 arrangement
            (screen_y * 2 + screen_x) * 0x400
        } else if matches!(bg.tilemap_cnt_x, TilemapCount::Two) {
            // 2x1 horizontal
            screen_x * 0x400
        } else if matches!(bg.tilemap_cnt_y, TilemapCount::Two) {
            // 1x2 vertical
            screen_y * 0x400
        } else {
            // 1x1 single screen
            0
        };

        let tile_offset = local_y * 32 + local_x;

        bg.tilemap_base_addr
            .wrapping_add(screen_offset as u16)
            .wrapping_add(tile_offset as u16)
            & 0x7FFF
    }

    /// Calculate character data word address
    pub(super) fn calc_chr_addr(&self, bg: &BgSettings, tile_num: u16, sub_x: u32, sub_y: u32, color_depth: u8) -> u16 {
        // For 16x16 tiles, sub_x and sub_y select which 8x8 quadrant (0 or 1)
        let effective_tile = if matches!(bg.chr_size, TileSize::Size16x16) {
            // 16x16 tiles are arranged as:
            // [tile+0 ] [tile+1 ]
            // [tile+16] [tile+17]
            tile_num + sub_x as u16 + (sub_y as u16 * 16)
        } else {
            tile_num
        };

        // Bytes per 8x8 tile based on BPP
        let bytes_per_tile: u16 = match color_depth {
            2 => 16,
            4 => 32,
            8 => 64,
            _ => 16,
        };

        // VRAM is word-addressed, so divide by 2
        let words_per_tile = bytes_per_tile / 2;

        bg.chr_base_addr.wrapping_add(effective_tile * words_per_tile)
    }

    /// Render a single 8x8 tile to the pixel buffer
    pub(super) fn render_8x8_tile(
        &self,
        pixel_buffer: &mut [u8],
        vram: &[u16; VRAM_SIZE],
        cgram: &[Color; 256],
        bpp: u8,
        chr_addr: u16,
        palette: u8,
        flip_x: bool,
        flip_y: bool,
        output_x: u32,
        output_y: u32,
        stride: u32,
        direct_color: bool,
    ) {
        for row in 0..8u32 {
            let actual_row = if flip_y { 7 - row } else { row };
            let dst_y = output_y + row;
            let dst_start = 4 * (dst_y * stride + output_x) as usize;
            let dst_end = dst_start + 4 * 8;
            let dst = &mut pixel_buffer[dst_start..dst_end];

            match bpp {
                2 => {
                    let row_data = self.decode_tile_row_2bpp(vram, chr_addr, actual_row);
                    let palette_base = (palette as usize) * 4;
                    self.write_pixels_2bpp(dst, cgram, row_data, palette_base, flip_x);
                }
                4 => {
                    let row_data = self.decode_tile_row_4bpp(vram, chr_addr, actual_row);
                    let palette_base = (palette as usize) * 16;
                    self.write_pixels_4bpp(dst, cgram, row_data, palette_base, flip_x);
                }
                8 => {
                    let row_data = self.decode_tile_row_8bpp(vram, chr_addr, actual_row);
                    self.write_pixels_8bpp(dst, cgram, row_data, flip_x, direct_color, palette);
                }
                _ => {}
            }
        }
    }

    // ==================== Tile Row Decoding ====================

    /// Decode one row of a 2bpp tile
    /// Returns 8 pixels packed: pixel 0 at bits 15:14, pixel 7 at bits 1:0
    #[inline]
    fn decode_tile_row_2bpp(&self, vram: &[u16; VRAM_SIZE], chr_addr: u16, row: u32) -> u16 {
        // 2bpp: 16 bytes per tile, 2 bytes per row
        // Each word contains one row: low byte = bp0, high byte = bp1
        let word_addr = (chr_addr as usize + row as usize) & 0x7FFF;
        let word = vram[word_addr];

        let bp0 = word as u8;
        let bp1 = (word >> 8) as u8;
        let bp10 = ((bp1 as u16) << 8) | (bp0 as u16);

        sppu::utils::interleave_2bpp(bp10)
    }

    /// Decode one row of a 4bpp tile
    /// Returns 8 pixels packed: pixel 0 at bits 31:28, pixel 7 at bits 3:0
    #[inline]
    fn decode_tile_row_4bpp(&self, vram: &[u16; VRAM_SIZE], chr_addr: u16, row: u32) -> u32 {
        // 4bpp: 32 bytes per tile
        // bp0,bp1 in first 16 bytes (8 words), bp2,bp3 in next 16 bytes
        let word_addr_01 = (chr_addr as usize + row as usize) & 0x7FFF;
        let word_addr_23 = (chr_addr as usize + row as usize + 8) & 0x7FFF;

        let word_01 = vram[word_addr_01];
        let word_23 = vram[word_addr_23];

        let bp0 = word_01 as u8;
        let bp1 = (word_01 >> 8) as u8;
        let bp2 = word_23 as u8;
        let bp3 = (word_23 >> 8) as u8;

        let bp10 = ((bp1 as u16) << 8) | (bp0 as u16);
        let bp32 = ((bp3 as u16) << 8) | (bp2 as u16);

        sppu::utils::interleave_4bpp(bp10, bp32)
    }

    /// Decode one row of an 8bpp tile
    /// Returns 8 pixels packed: pixel 0 at bits 63:56, pixel 7 at bits 7:0
    #[inline]
    fn decode_tile_row_8bpp(&self, vram: &[u16; VRAM_SIZE], chr_addr: u16, row: u32) -> u64 {
        // 8bpp: 64 bytes per tile
        // bp0,bp1 in words 0-7, bp2,bp3 in words 8-15, bp4,bp5 in 16-23, bp6,bp7 in 24-31
        let word_addr_01 = (chr_addr as usize + row as usize) & 0x7FFF;
        let word_addr_23 = (chr_addr as usize + row as usize + 8) & 0x7FFF;
        let word_addr_45 = (chr_addr as usize + row as usize + 16) & 0x7FFF;
        let word_addr_67 = (chr_addr as usize + row as usize + 24) & 0x7FFF;

        let word_01 = vram[word_addr_01];
        let word_23 = vram[word_addr_23];
        let word_45 = vram[word_addr_45];
        let word_67 = vram[word_addr_67];

        let bp0 = word_01 as u8;
        let bp1 = (word_01 >> 8) as u8;
        let bp2 = word_23 as u8;
        let bp3 = (word_23 >> 8) as u8;
        let bp4 = word_45 as u8;
        let bp5 = (word_45 >> 8) as u8;
        let bp6 = word_67 as u8;
        let bp7 = (word_67 >> 8) as u8;

        let bp10 = ((bp1 as u16) << 8) | (bp0 as u16);
        let bp32 = ((bp3 as u16) << 8) | (bp2 as u16);
        let bp54 = ((bp5 as u16) << 8) | (bp4 as u16);
        let bp76 = ((bp7 as u16) << 8) | (bp6 as u16);

        sppu::utils::interleave_8bpp(bp10, bp32, bp54, bp76)
    }

    #[inline]
    fn write_pixels_2bpp(
        &self,
        dst: &mut [u8],
        cgram: &[Color; 256],
        row_data: u16,
        palette_base: usize,
        flip_x: bool,
    ) {
        let pixels = [
            ((row_data >> 14) & 0x3) as usize,
            ((row_data >> 12) & 0x3) as usize,
            ((row_data >> 10) & 0x3) as usize,
            ((row_data >> 8) & 0x3) as usize,
            ((row_data >> 6) & 0x3) as usize,
            ((row_data >> 4) & 0x3) as usize,
            ((row_data >> 2) & 0x3) as usize,
            (row_data & 0x3) as usize,
        ];

        if flip_x {
            for col in 0..8 {
                let ci = pixels[7 - col];
                let color = cgram[palette_base + ci].to_rgba_bytes();
                if ci != 0 {
                    dst[4 * col..4 * col + 4].copy_from_slice(&color);
                }
            }
        } else {
            for col in 0..8 {
                let ci = pixels[col];
                let color = cgram[palette_base + ci].to_rgba_bytes();
                if ci != 0 {
                    dst[4 * col..4 * col + 4].copy_from_slice(&color);
                }
            }
        }
    }

    #[inline]
    fn write_pixels_4bpp(
        &self,
        dst: &mut [u8],
        cgram: &[Color; 256],
        row_data: u32,
        palette_base: usize,
        flip_x: bool,
    ) {
        let pixels = [
            ((row_data >> 28) & 0xF) as usize,
            ((row_data >> 24) & 0xF) as usize,
            ((row_data >> 20) & 0xF) as usize,
            ((row_data >> 16) & 0xF) as usize,
            ((row_data >> 12) & 0xF) as usize,
            ((row_data >> 8) & 0xF) as usize,
            ((row_data >> 4) & 0xF) as usize,
            (row_data & 0xF) as usize,
        ];

        if flip_x {
            for col in 0..8 {
                let ci = pixels[7 - col];
                let color = cgram[palette_base + ci].to_rgba_bytes();
                if ci != 0 {
                    dst[4 * col..4 * col + 4].copy_from_slice(&color);
                }
            }
        } else {
            for col in 0..8 {
                let ci = pixels[col];
                let color = cgram[palette_base + ci].to_rgba_bytes();
                if ci != 0 {
                    dst[4 * col..4 * col + 4].copy_from_slice(&color);
                }
            }
        }
    }

    #[inline]
    fn write_pixels_8bpp(
        &self,
        dst: &mut [u8],
        cgram: &[Color; 256],
        row_data: u64,
        flip_x: bool,
        direct_color: bool,
        palette_bits: u8,
    ) {
        let pixels = [
            ((row_data >> 56) & 0xFF) as u8,
            ((row_data >> 48) & 0xFF) as u8,
            ((row_data >> 40) & 0xFF) as u8,
            ((row_data >> 32) & 0xFF) as u8,
            ((row_data >> 24) & 0xFF) as u8,
            ((row_data >> 16) & 0xFF) as u8,
            ((row_data >> 8) & 0xFF) as u8,
            (row_data & 0xFF) as u8,
        ];

        if direct_color {
            let lut = &self.direct_color_lut[(palette_bits & 0x7) as usize];
            if flip_x {
                for col in 0..8 {
                    let ci = pixels[7 - col];
                    let color = lut[ci as usize].to_be_bytes();
                    if ci != 0 {
                        dst[4 * col..4 * col + 4].copy_from_slice(&color);
                    }
                }
            } else {
                for col in 0..8 {
                    let ci = pixels[col];
                    let color = lut[ci as usize].to_be_bytes();
                    if ci != 0 {
                        dst[4 * col..4 * col + 4].copy_from_slice(&color);
                    }
                }
            }
        } else {
            // 8bpp uses full 256-color palette (palette_base = 0)
            if flip_x {
                for col in 0..8 {
                    let ci = pixels[7 - col] as usize;
                    let color = cgram[ci].to_rgba_bytes();
                    if ci != 0 {
                        dst[4 * col..4 * col + 4].copy_from_slice(&color);
                    }
                }
            } else {
                for col in 0..8 {
                    let ci = pixels[col] as usize;
                    let color = cgram[ci].to_rgba_bytes();
                    if ci != 0 {
                        dst[4 * col..4 * col + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }
}