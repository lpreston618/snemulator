use crate::app;
use crate::core::debug::watchpoints::Counter;
use crate::core::debug::watchpoints::Logpoint;
use crate::core::debug::watchpoints::Watchpoint;
use crate::core::snemcore;
use crate::app::debug::watchpoints::editor::Editor;
use crate::core::debug::watchpoints::NodeKind;
use crate::core::debug::watchpoints::CompiledGraph;

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
    
    pub fn render(&mut self, ui: &mut egui::Ui, snem_core: &snemcore::Snemulator, app_state: &app::AppState, jump_to_wps_on_hit: &mut bool) {
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
                    
                    if ui.button("Counter").clicked() {
                        self.add_node(NodeKind::Counter(Counter::default()));
                        ui.close();
                    }
                });
                
                if ui.button("Add Break").clicked() {
                    self.add_node(NodeKind::Break { lit: false });
                }
                
                if ui.button("Add Log Point").clicked() {
                    self.add_node(NodeKind::Log(Logpoint::default()));
                }
                
                ui.checkbox(&mut self.watchpoints_en, "Enable Watchpoints");
                
                ui.add_space(5.0);
                
                ui.checkbox(jump_to_wps_on_hit, "Jump to Watchpoints on Hit");
            });
            
        });
        
        ui.separator();
        
        self.editor.show(ui, app_state, snem_core);
    }
    
    fn add_watchpoint(&mut self) {
        self.editor.create_new_watchpoint(Watchpoint::default());
    }
    
    fn add_node(&mut self, kind: NodeKind) {
        match kind {
            NodeKind::Condition { .. } => {},
            _ => self.editor.create_new_logic(kind),
        }
    }
    
    pub fn compile_watchpoints(&mut self, snem_core: &snemcore::Snemulator) {        
        self.compiled_wps = self.editor.graph.compile(snem_core);
    }
    
    pub fn clear_compiled_watchpoints(&mut self) {
        self.compiled_wps = CompiledGraph::default();
    }
    
    pub fn update_watchpoint_graph(&mut self) {
        self.editor.update_watchpoints(&self.compiled_wps);
    }
    
    pub fn watchpoints(&self) -> &CompiledGraph {
        &self.compiled_wps
    }
    
    pub fn watchpoints_enabled(&self) -> bool {
        self.watchpoints_en
    }
}