use snemcore::debug::DebugHarness;

pub struct DebugControl {
    
}

pub struct MainDebugHarness {
    finished_ipl: bool,
}

impl MainDebugHarness {
    pub fn new() -> Self {
        Self {
            finished_ipl: false,
        }
    }
}

impl DebugHarness for MainDebugHarness {
    const IS_DEBUGGING_HARNESS: bool = true;

    fn on_power(&mut self, _core: &mut snemcore::Snemulator) {
        log::debug!("Core powered on");
    }

    fn on_spc_instruction(&mut self, spc: &mut snemcore::ssmp::spc::Spc700, prg_bytes: &[u8]) {
        if !self.finished_ipl && spc.pc < 0xFFC0 {
            self.finished_ipl = true;

            log::debug!("Finished IPL Boot ROM.")
        }
    }
}