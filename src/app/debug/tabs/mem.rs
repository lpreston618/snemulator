use crate::core::sppu;
use crate::core::snemcore;

// Found emperically, just a mutliplier for jumping to addresses in the mem view
const SCROLL_OFFSET_MULT: f32 = 1.1752;
const HIGHLIGHT_DURATION: usize = 60 * 3;
const HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::RED;
const HIGHLIGHT_COLOR_TRANSPARENT: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(0xFF, 0xFF, 0xFF, 0);

#[derive(PartialEq, Clone, Copy)]
pub enum MemViewRegion { Wram, Rom, Vram, Oam, Cgram }

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

pub struct MemoryTab {
    pub region: MemViewRegion,
    pub forced_scroll: Option<f32>,
    highlighted_addrs: std::collections::HashSet<u32>,
    highlight_frame_cnt: usize,
}

impl MemoryTab {
    pub fn new() -> Self {
        Self { 
            region: MemViewRegion::Wram, 
            forced_scroll: None, 
            highlighted_addrs: std::collections::HashSet::new(),
            highlight_frame_cnt: 0,
        }
    }
    
    pub fn highlight_addresses(&mut self, addrs: Vec<u32>) {
        self.highlighted_addrs.clear();
        
        for addr in addrs {
            self.highlighted_addrs.insert(addr);
        }
        
        self.highlight_frame_cnt = HIGHLIGHT_DURATION;
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator) {
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
        
        ui.separator();
    
        let addr_w = self.region.addr_width();
    
        match self.region {
            MemViewRegion::Vram  => Self::render_vram_dump(ui, &snem_core.vram[..]),
            MemViewRegion::Cgram => Self::render_cgram_dump(ui, &snem_core.cgram[..]),
            _ => {
                let data: &[u8] = match self.region {
                    MemViewRegion::Wram => &snem_core.wram[..],
                    MemViewRegion::Rom  => &snem_core.rom_slice(),
                    MemViewRegion::Oam  => &snem_core.oam[..],
                    _               => unreachable!(),
                };
                self.render_byte_dump(ui, data, addr_w);
            }
        }
        
        if self.highlight_frame_cnt > 0 {
            self.highlight_frame_cnt -= 1;
        
            if self.highlight_frame_cnt == 0 {
                self.highlighted_addrs.clear();
            }
        }
    }
    
    fn render_vram_dump(ui: &mut egui::Ui, vram: &[u16]) {
        const COLS: usize = 8;
        let total_rows = vram.len().div_ceil(COLS);
        let row_height = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;
        
        egui::ScrollArea::vertical().auto_shrink([false, false])
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
    
    fn render_cgram_dump(ui: &mut egui::Ui, cgram: &[sppu::Color]) {
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
    
    fn render_byte_dump(&mut self, ui: &mut egui::Ui, data: &[u8], addr_w: usize) {
        const COLS: usize = 16;
    
        // let anchor = self.mem.anchor() as usize;
        let total_rows  = data.len().div_ceil(COLS);
        let row_height  = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;
            
        let scroll_area = egui::ScrollArea::vertical().auto_shrink([false, false]);
    
        let scroll_area = if let Some(offset) = self.forced_scroll.take() {
            scroll_area.vertical_scroll_offset(SCROLL_OFFSET_MULT * offset * row_height * total_rows as f32)
        } else {
            scroll_area
        };
        
        scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
            for row in row_range {
                let base = row * COLS;
                let chunk = &data[base..data.len().min(base + COLS)];
    
                ui.horizontal(|ui| {
                    // Address gutter
                    ui.label(egui::RichText::new(
                        format!("{:0>width$X}:", base, width = addr_w)
                    ).monospace().weak());
    
                    // Hex bytes — group in sets of 8 for readability
                    for (i, byte) in chunk.iter().enumerate() {
                        if i == 8 { ui.label(egui::RichText::new("·").weak()); }
                        
                        let addr = (base + i) as u32;
                        
                        let text = egui::RichText::new(format!("{:02X}", byte)).monospace();
                        
                        let text = if self.highlighted_addrs.contains(&addr) {
                            let lerp_t = 1.0 - (self.highlight_frame_cnt as f32 / HIGHLIGHT_DURATION as f32).sqrt();
                            text.background_color(HIGHLIGHT_COLOR.lerp_to_gamma(HIGHLIGHT_COLOR_TRANSPARENT, lerp_t))
                        } else {
                            text
                        };
                        
                        ui.label(text);
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