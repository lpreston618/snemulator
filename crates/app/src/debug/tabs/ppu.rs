use snemcore::Snemulator;

use crate::debug::harness::MainDebugHarness;
use crate::debug::texture::Texture;

// mod chr;
mod layers;

#[derive(PartialEq, Clone, Copy)]
enum PpuSubTab {
    Chr,
    Bg1,
    Bg2,
    Bg3,
    Bg4,
    Obj,
}

impl PpuSubTab {
    fn label(&self) -> &'static str {
        match self {
            PpuSubTab::Chr => "Chr Viewer",
            PpuSubTab::Bg1 => "BG1",
            PpuSubTab::Bg2 => "BG2",
            PpuSubTab::Bg3 => "BG3",
            PpuSubTab::Bg4 => "BG4",
            PpuSubTab::Obj => "Obj",
        }
    }
}

pub struct PpuTab {
    // chr_viewer: chr::ChrViewer,
    // layer_viewer: layers::LayerView,
    bg1_viewer: layers::BgDebugView<0>,
    bg2_viewer: layers::BgDebugView<1>,
    bg3_viewer: layers::BgDebugView<2>,
    bg4_viewer: layers::BgDebugView<3>,
    // obj_viewer: layers::LayerView,
    selected_tab: PpuSubTab,
}

impl PpuTab {
    pub fn new() -> Self {
        Self {
            // chr_viewer: chr::ChrViewer::new(painter),
            bg1_viewer: layers::BgDebugView::new(),
            bg2_viewer: layers::BgDebugView::new(),
            bg3_viewer: layers::BgDebugView::new(),
            bg4_viewer: layers::BgDebugView::new(),
            // obj_viewer: layers::LayerView::new(painter),
            // layer_viewer: layers::LayerViewer::new(painter),
            selected_tab: PpuSubTab::Chr,
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, core: &Snemulator, harness: &mut MainDebugHarness) {
        ui.vertical(|ui| {
            egui::TopBottomPanel::top("tabs").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in [
                        PpuSubTab::Chr,
                        PpuSubTab::Bg1,
                        PpuSubTab::Bg2,
                        PpuSubTab::Bg3,
                        PpuSubTab::Bg4,
                        PpuSubTab::Obj,
                    ] {
                        ui.selectable_value(&mut self.selected_tab, tab, tab.label());
                    }
                });
            });
            
            ui.separator();
            
            match self.selected_tab {
                // PpuSubTab::Chr => self.chr_viewer.render(ui, core),
                PpuSubTab::Bg1 => {
                    self.bg1_viewer.update(core, harness);
                    self.bg1_viewer.render(ui, core);
                }
                PpuSubTab::Bg2 => {
                    self.bg2_viewer.update(core, harness);
                    self.bg2_viewer.render(ui, core);
                }
                PpuSubTab::Bg3 => {
                    self.bg3_viewer.update(core, harness);
                    self.bg3_viewer.render(ui, core);
                }
                PpuSubTab::Bg4 => {
                    self.bg4_viewer.update(core, harness);
                    self.bg4_viewer.render(ui, core);
                }
                // PpuSubTab::Obj => self.obj_viewer.render(ui, &core.probe.as_ref().unwrap().layer_buffers.obj[..]),
                _ => {}
            }
        });
    }
}