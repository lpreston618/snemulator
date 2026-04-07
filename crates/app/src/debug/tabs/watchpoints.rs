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

use crate::{app::AppState, debug::debugger::Debugger};

pub struct WatchpointsTab {
    
}

impl WatchpointsTab {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, app_state: &AppState) {
        ui.horizontal(|ui| {
            if ui.button("Load Watchpoint Script").clicked() {
                if let Err(e) = try_load_watchpoint_script(core) {
                    log::error!("failed to load watchpoint script: {}", e);
                }
            }
        });
        
        ui.separator();
    }
}

fn try_load_watchpoint_script(core: &mut Snemulator<Debugger>) -> Result<()> {
    let wp_script_file = FileDialog::new()
        .add_filter("Watchpoint Script", &["lua"])
        .set_directory("/")
        .pick_file()
        .ok_or(anyhow::anyhow!("Invalid file chosen"))?;
    
    let mut file = std::fs::File::open(wp_script_file)?;
    let mut script = String::new();
    let _ = file.read_to_string(&mut script)?;
    
    core.probe.as_mut().unwrap().wp_engine
        .load_script(&mut script)
        .map_err(|e| anyhow::anyhow!("Failed to load script: {}", e))?;
    
    Ok(())
}