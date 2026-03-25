use crate::core::snemcore;

pub struct LayerViewer {
    pub needs_updating: bool,
}

impl LayerViewer {
    pub fn new(painter: &mut egui_glow::Painter) -> Self {
        Self {
            needs_updating: true,
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator, painter: &mut egui_glow::Painter) {
        
    }
    
    fn update_textures(&mut self) {
        self.needs_updating = false;
    }
}