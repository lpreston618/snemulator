pub mod regs;
pub mod voices;

#[derive(Clone, Copy, Debug)]
pub enum ADSRStage {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy, Debug)]
pub enum GainMode {
    Fixed,
    Decrease,
    ExpDecrease,
    Increase,
    BentIncrease,
}

#[derive(Clone, Copy, Debug)]
pub enum BrrFilter {
    Filter0,
    Filter1,
    Filter2,
    Filter3,
}

pub struct SuperDSP {
    
}

impl SuperDSP {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn clock_envelopes(&mut self) {
        
    }
    
    pub fn generate_sample(&mut self) {
        
    }
}