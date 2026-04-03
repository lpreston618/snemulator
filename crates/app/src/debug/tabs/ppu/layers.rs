use crate::app::debug::tabs::ppu::texture::Texture;
use crate::core::sysinfo;

pub struct LayerView {
    pub texture: Texture,
    // window: app::Texture,
    // cmath_en: app::Texture,
}

impl LayerView {
    pub fn new(painter: &mut egui_glow::Painter) -> Self {
        let width = (sysinfo::SCREEN_WIDTH / 2) as usize;
        let height = (sysinfo::SCREEN_HEIGHT / 2) as usize;

        Self {
            texture: Texture::new(painter, width, height),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        self.texture.update_texture();
        
        ui.vertical(|ui| {            
            let (width, height) = self.texture.size();
            let scale = 2.0;
            
            let image_size = egui::Vec2::new(width as f32, height as f32) * scale;
            
            ui.image(egui::load::SizedTexture::new(self.texture.texture_id(), image_size));
        });
    }
}
