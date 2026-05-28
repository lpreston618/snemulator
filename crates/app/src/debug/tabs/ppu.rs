use snemcore::Snemulator;

use crate::debug::debugger::Debugger;
use crate::debug::texture::Texture;

mod chr;
mod regs;
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
    chr_viewer: chr::ChrViewer,
    // layer_viewer: layers::LayerView,
    bg1_viewer: layers::LayerView,
    bg2_viewer: layers::LayerView,
    bg3_viewer: layers::LayerView,
    bg4_viewer: layers::LayerView,
    obj_viewer: layers::LayerView,
    selected_tab: PpuSubTab,
}

impl PpuTab {
    pub fn new(painter: &mut egui_glow::Painter) -> Self {
        Self {
            chr_viewer: chr::ChrViewer::new(painter),
            bg1_viewer: layers::LayerView::new(painter),
            bg2_viewer: layers::LayerView::new(painter),
            bg3_viewer: layers::LayerView::new(painter),
            bg4_viewer: layers::LayerView::new(painter),
            obj_viewer: layers::LayerView::new(painter),
            // layer_viewer: layers::LayerViewer::new(painter),
            selected_tab: PpuSubTab::Chr,
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, core: &Snemulator<Debugger>) {
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
                PpuSubTab::Chr => self.chr_viewer.render(ui, core),
                PpuSubTab::Bg1 => self.bg1_viewer.render(ui, &core.probe.as_ref().unwrap().layer_buffers.bg1[..]),
                PpuSubTab::Bg2 => self.bg2_viewer.render(ui, &core.probe.as_ref().unwrap().layer_buffers.bg2[..]),
                PpuSubTab::Bg3 => self.bg3_viewer.render(ui, &core.probe.as_ref().unwrap().layer_buffers.bg3[..]),
                PpuSubTab::Bg4 => self.bg4_viewer.render(ui, &core.probe.as_ref().unwrap().layer_buffers.bg4[..]),
                PpuSubTab::Obj => self.obj_viewer.render(ui, &core.probe.as_ref().unwrap().layer_buffers.obj[..]),
                _ => {}
            }
        });
    }
}