use crate::core::snemcore;

mod bg;
mod chr;
mod layers;
mod regs;

#[derive(PartialEq, Clone, Copy)]
enum PpuSubTab {
    Chr,
    Layers,
}

impl PpuSubTab {
    fn label(&self) -> &'static str {
        match self {
            PpuSubTab::Chr => "Char Viewer",
            PpuSubTab::Layers => "PPU Layers",
        }
    }
}

pub struct PpuTab {
    chr_viewer: chr::ChrViewer,
    layer_viewer: layers::LayerViewer,
    selected_tab: PpuSubTab,
}

impl PpuTab {
    pub fn new(painter: &mut egui_glow::Painter) -> Self {
        Self {
            chr_viewer: chr::ChrViewer::new(painter),
            layer_viewer: layers::LayerViewer::new(painter),
            selected_tab: PpuSubTab::Chr,
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator, painter: &mut egui_glow::Painter) {
        ui.vertical(|ui| {
            egui::TopBottomPanel::top("tabs").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in [
                        PpuSubTab::Chr,
                        PpuSubTab::Layers,
                    ] {
                        ui.selectable_value(&mut self.selected_tab, tab, tab.label());
                    }
                });
            });
            
            ui.separator();
            
            match self.selected_tab {
                PpuSubTab::Chr => self.chr_viewer.render(ui, snem_core, painter),
                PpuSubTab::Layers => self.layer_viewer.render(ui, snem_core, painter),
            } 
        });
    }
}