// use crate::core::snemcore;
// use crate::core::debug::ppulayers::LayerBuffers;

// mod chr;
// mod regs;
// mod layers;
// mod texture;

// #[derive(PartialEq, Clone, Copy)]
// enum PpuSubTab {
//     Chr,
//     Bg1,
//     Bg2,
//     Bg3,
//     Bg4,
//     Obj,
// }

// impl PpuSubTab {
//     fn label(&self) -> &'static str {
//         match self {
//             PpuSubTab::Chr => "Chr Viewer",
//             PpuSubTab::Bg1 => "BG1",
//             PpuSubTab::Bg2 => "BG2",
//             PpuSubTab::Bg3 => "BG3",
//             PpuSubTab::Bg4 => "BG4",
//             PpuSubTab::Obj => "Obj",
//         }
//     }
// }

// pub struct PpuTab {
//     chr_viewer: chr::ChrViewer,
//     // layer_viewer: layers::LayerView,
//     bg1_viewer: layers::LayerView,
//     bg2_viewer: layers::LayerView,
//     bg3_viewer: layers::LayerView,
//     bg4_viewer: layers::LayerView,
//     obj_viewer: layers::LayerView,
//     selected_tab: PpuSubTab,
// }

// impl PpuTab {
//     pub fn new(painter: &mut egui_glow::Painter) -> Self {
//         Self {
//             chr_viewer: chr::ChrViewer::new(painter),
//             bg1_viewer: layers::LayerView::new(painter),
//             bg2_viewer: layers::LayerView::new(painter),
//             bg3_viewer: layers::LayerView::new(painter),
//             bg4_viewer: layers::LayerView::new(painter),
//             obj_viewer: layers::LayerView::new(painter),
//             // layer_viewer: layers::LayerViewer::new(painter),
//             selected_tab: PpuSubTab::Chr,
//         }
//     }
    
//     pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator) {
//         ui.vertical(|ui| {
//             egui::TopBottomPanel::top("tabs").show_inside(ui, |ui| {
//                 ui.horizontal(|ui| {
//                     for tab in [
//                         PpuSubTab::Chr,
//                         PpuSubTab::Bg1,
//                         PpuSubTab::Bg2,
//                         PpuSubTab::Bg3,
//                         PpuSubTab::Bg4,
//                         PpuSubTab::Obj,
//                     ] {
//                         ui.selectable_value(&mut self.selected_tab, tab, tab.label());
//                     }
//                 });
//             });
            
//             ui.separator();
            
//             match self.selected_tab {
//                 PpuSubTab::Chr => self.chr_viewer.render(ui, snem_core),
//                 // PpuSubTab::Layers => self.layer_viewer.render(ui, snem_core, painter),
//                 PpuSubTab::Bg1 => self.bg1_viewer.render(ui),
//                 PpuSubTab::Bg2 => self.bg2_viewer.render(ui),
//                 PpuSubTab::Bg3 => self.bg3_viewer.render(ui),
//                 PpuSubTab::Bg4 => self.bg4_viewer.render(ui),
//                 PpuSubTab::Obj => self.obj_viewer.render(ui),
//                 _ => {}
//             }
//         });
//     }
    
//     pub fn layer_buffers(&mut self) -> LayerBuffers {
//         LayerBuffers {
//             bg1: self.bg1_viewer.texture.take_pixels(),
//             bg2: self.bg2_viewer.texture.take_pixels(),
//             bg3: self.bg3_viewer.texture.take_pixels(),
//             bg4: self.bg4_viewer.texture.take_pixels(),
//             obj: self.obj_viewer.texture.take_pixels(),
//         }
//     }
    
//     pub fn restore_buffers(&mut self, layer_buffers: LayerBuffers) {
//         self.bg1_viewer.texture.give_pixels(layer_buffers.bg1);
//         self.bg2_viewer.texture.give_pixels(layer_buffers.bg2);
//         self.bg3_viewer.texture.give_pixels(layer_buffers.bg3);
//         self.bg4_viewer.texture.give_pixels(layer_buffers.bg4);
//         self.obj_viewer.texture.give_pixels(layer_buffers.obj);
//     }
// }