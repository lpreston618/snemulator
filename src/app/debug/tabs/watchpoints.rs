use crate::app;
use crate::core::snemcore;
use crate::app::debug::watchpoints::editor::Editor;
use crate::app::debug::watchpoints::types::LogKind;
use crate::app::debug::watchpoints::types::NodeKind;
use crate::app::debug::watchpoints::types::CompiledGraph;
use crate::app::debug::watchpoints::types::WatchpointKind;

pub struct WatchpointsTab {
    editor: Editor,
    watchpoints_en: bool,
    compiled_wps: CompiledGraph,
}

impl WatchpointsTab {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
            watchpoints_en: true,
            compiled_wps: CompiledGraph::default(),
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator, app_state: &app::AppState) {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(app_state.is_paused, |ui| {
                if ui.button("Add Watchpoint").clicked() {
                    self.add_watchpoint();
                }
                
                ui.menu_button("Add Logic", |ui| {
                    if ui.button("And").clicked() {
                        self.add_node(NodeKind::And);
                        ui.close();
                    }
                    
                    if ui.button("Or").clicked() {
                        self.add_node(NodeKind::Or);
                        ui.close();
                    }
                    
                    if ui.button("Not").clicked() {
                        self.add_node(NodeKind::Not);
                        ui.close();
                    }
                });
                
                if ui.button("Add Break").clicked() {
                    self.add_node(NodeKind::Break { lit: false });
                }
                
                if ui.button("Add Log Point").clicked() {
                    self.add_node(NodeKind::Log(LogKind::default()));
                }
                
                ui.checkbox(&mut self.watchpoints_en, "Enable Watchpoints")
            });
            
        });
        
        ui.separator();
        
        self.editor.show(ui, app_state, snem_core);
    }
    
    fn add_watchpoint(&mut self) {
        self.editor.create_new_watchpoint(WatchpointKind::default());
    }
    
    fn add_node(&mut self, kind: NodeKind) {
        match kind {
            NodeKind::Condition { .. } => {},
            _ => self.editor.create_new_logic(kind),
        }
    }
    
    pub fn compile_watchpoints(&mut self, snem_core: &snemcore::Snemulator) {
        if !self.watchpoints_en {
            self.compiled_wps = CompiledGraph::default();
            return;
        }
        
        self.compiled_wps = self.editor.graph.compile(snem_core);
    }
    
    pub fn watchpoints(&self) -> &CompiledGraph {
        &self.compiled_wps
    }
}