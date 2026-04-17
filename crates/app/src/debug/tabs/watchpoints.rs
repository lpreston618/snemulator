// use crate::app;
// use crate::core::debug::watchpoints::Counter;
// use crate::core::debug::watchpoints::Logpoint;
// use crate::core::debug::watchpoints::Watchpoint;
// use crate::core::snemcore;
// use crate::app::debug::watchpoints::editor::Editor;
// use crate::core::debug::watchpoints::NodeKind;
// use crate::core::debug::watchpoints::CompiledGraph;

use std::io::Read;

use rfd::FileDialog;
use snemcore::Snemulator;

use anyhow::Result;

use crate::{app::AppState, debug::{debugger::Debugger, watchpoints::luainclude::EXAMPLE_SCRIPT}};

pub struct WatchpointsTab {
    loaded_script_path: Option<std::path::PathBuf>,
    loaded_script: Option<String>,
}

impl WatchpointsTab {
    pub fn new() -> Self {
        Self {
            loaded_script_path: None,
            loaded_script: None,
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, app_state: &AppState) {
        let script_loaded = self.loaded_script_path.is_some();
        
        ui.horizontal(|ui| {
            let button_text = if script_loaded { "Load New Script" } else { "Load Script" };
            
            if ui.button(button_text).clicked() {
                if let Err(e) = self.try_select_script(core) {
                    log::error!("failed to load watchpoint script: {}", e);
                }
            }
            
            ui.add_enabled_ui(script_loaded, |ui| {
                if ui.button("Reload Script").clicked() {
                    if let Err(e) = self.try_reload_script(core) {
                        log::error!("failed to reload watchpoint script: {}", e);
                    }
                }
                
                if ui.button("Unload Script").clicked() {
                    self.loaded_script_path = None;
                    core.do_with_probe(|probe, core| {
                        probe.wp_engine.unload_script(core, &mut probe.control);
                        self.loaded_script = None;
                        self.loaded_script_path = None;
                    });
                }
            });
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical().id_salt(ui.next_auto_id()).show(ui, |ui| {
            ui.code(
                match &self.loaded_script {
                    Some(script) => script,
                    None => EXAMPLE_SCRIPT,
                }
            );
        });
    }
    
    fn try_load_watchpoint_script(&mut self, wp_script_file: std::path::PathBuf, core: &mut Snemulator<Debugger>) -> Result<()> {
        let mut file = std::fs::File::open(wp_script_file.clone())?;
        let mut script = String::new();
        let _ = file.read_to_string(&mut script)?;
        
        let prev_script_loaded = self.loaded_script.is_some();
        
        self.loaded_script_path = Some(wp_script_file.clone());
        self.loaded_script = Some(script.clone());
        
        core.do_with_probe(|probe, core| {
            if prev_script_loaded {
                probe.wp_engine.unload_script(core, &mut probe.control);
            }
            
            probe.wp_engine.load_script(&mut script)
                .map_err(|e| {
                    self.loaded_script = None;
                    self.loaded_script_path = None;
                    anyhow::anyhow!("Failed to load script: {}", e)
                })
        }).unwrap()
    }
    
    fn try_select_script(&mut self, core: &mut Snemulator<Debugger>) -> Result<()> {
        let wp_script_file = FileDialog::new()
            .add_filter("Watchpoint Script", &["lua"])
            .set_directory("/")
            .pick_file()
            .ok_or(anyhow::anyhow!("Invalid file chosen"))?;
        
        self.try_load_watchpoint_script(wp_script_file, core)
    }
    
    fn try_reload_script(&mut self, core: &mut Snemulator<Debugger>) -> Result<()> {
        if let Some(script_path) = self.loaded_script_path.take() {
            self.try_load_watchpoint_script(script_path, core)
        } else {
            Err(anyhow::anyhow!("No script loaded"))
        }
    }
}