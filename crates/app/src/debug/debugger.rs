use std::collections::HashSet;

use snemcore::probe::DebugProbe;

use crate::debug::breakpoints::BreakpointInfo;

pub struct Debugger {
    breakpoints: HashSet<BreakpointInfo>,
    // pause_emulation: bool,
    breakpoint_hit: bool,
    watchpoint_hit: bool,
}

impl DebugProbe for Debugger {
    fn resume_emulation(&mut self) {
        self.breakpoint_hit = false;
        self.watchpoint_hit = false;
    }
    
    fn on_instruction(&mut self, full_pc: u32) {
        if self.breakpoints.contains(&BreakpointInfo::new(full_pc)) {
            self.breakpoint_hit = true;
        }
    }
    
    fn should_stop(&mut self) -> bool {
        self.breakpoint_hit || self.watchpoint_hit
    }
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            breakpoint_hit: false,
            watchpoint_hit: false,
        }
    }
}
