use crate::core::sppu;
use crate::core::snemcore;

pub struct ChrTab {
    texture: Option<glow::Texture>,
    bpp_mode: sppu::ColorDepth,
    palette_index: usize,
}

impl ChrTab {
    pub fn new() -> Self {
        Self {
            texture: None,
            bpp_mode: sppu::ColorDepth::Bpp4,
            palette_index: 0,
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator) {
        
    }
    
    // // Call once during DebugWindow::new(), same pattern as game_texture init
    // pub fn init_texture(gl: &glow::Context) -> Option<glow::Texture> {
    //     unsafe {
    //         let texture = gl.create_texture()?;
    //         gl.bind_texture(glow::TEXTURE_2D, Some(texture));
    //         gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
    //         gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
    //         gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
    //         gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
    //         gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, TILES_WIDE as i32, TILES_TALL as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, None);
    //         Some(texture)
    //     }
    // }

    // // Decode VRAM tiles -> RGBA pixels, upload via tex_sub_image_2d
    // pub fn update_texture(&self, gl: &glow::Context, vram: &[u8], cgram: &[u8]) {
    //     let mut pixels = vec![0u8; TILES_WIDE * TILES_TALL * 8 * 8 * 4];
    //     // decode tiles from vram into pixels using self.bpp_mode
    //     // ...
    //     unsafe {
    //         gl.bind_texture(glow::TEXTURE_2D, self.texture);
    //         gl.tex_sub_image_2d( ... pixels ... );
    //     }
    // }

    // // In egui, display with egui::Image using a TextureId registered via egui_painter
    // // NOTE: You'll need to register the raw GL texture with egui_glow to get a TextureId
}