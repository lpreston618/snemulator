use crate::core::snemcore;

mod bg;
mod chr;
mod layers;
mod regs;

#[derive(PartialEq, Clone, Copy)]
enum PpuSubTab {
    Chr,
    Layers,
    Bg1,
    Bg2,
    Bg3,
    Bg4
}

impl PpuSubTab {
    fn label(&self) -> &'static str {
        match self {
            PpuSubTab::Chr => "Char Viewer",
            PpuSubTab::Layers => "PPU Layers",
            PpuSubTab::Bg1 => "BG1",
            PpuSubTab::Bg2 => "BG2",
            PpuSubTab::Bg3 => "BG3",
            PpuSubTab::Bg4 => "BG4",
        }
    }
}

pub struct PpuTab {
    chr_viewer: chr::ChrViewer,
    layer_viewer: layers::LayerViewer,
    bg1_viewer: bg::BgView<1>,
    bg2_viewer: bg::BgView<2>,
    bg3_viewer: bg::BgView<3>,
    bg4_viewer: bg::BgView<4>,
    selected_tab: PpuSubTab,
}

impl PpuTab {
    pub fn new(painter: &mut egui_glow::Painter) -> Self {
        Self {
            chr_viewer: chr::ChrViewer::new(painter),
            bg1_viewer: bg::BgView::new(painter),
            bg2_viewer: bg::BgView::new(painter),
            bg3_viewer: bg::BgView::new(painter),
            bg4_viewer: bg::BgView::new(painter),
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
                        PpuSubTab::Bg1,
                        PpuSubTab::Bg2,
                        PpuSubTab::Bg3,
                        PpuSubTab::Bg4
                    ] {
                        ui.selectable_value(&mut self.selected_tab, tab, tab.label());
                    }
                });
            });
            
            ui.separator();
            
            match self.selected_tab {
                PpuSubTab::Chr => self.chr_viewer.render(ui, snem_core, painter),
                PpuSubTab::Layers => self.layer_viewer.render(ui, snem_core, painter),
                PpuSubTab::Bg1 => self.bg1_viewer.render(ui, snem_core, painter),
                PpuSubTab::Bg2 => self.bg2_viewer.render(ui, snem_core, painter),
                PpuSubTab::Bg3 => self.bg3_viewer.render(ui, snem_core, painter),
                PpuSubTab::Bg4 => self.bg4_viewer.render(ui, snem_core, painter),
                _ => {}
            } 
        });
    }
}