use crate::core::snemcore;

pub struct LayerViewer {
    
}

impl LayerViewer {
    pub fn new(painter: &mut egui_glow::Painter) -> Self {
        Self {}
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator, egui_renderer: &mut egui_glow::Painter) {
    }
}