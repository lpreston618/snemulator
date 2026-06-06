use crate::sysinfo::AUDIO_SAMPLE_HZ;

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
    sample_count: u64,
}

impl SuperDSP {
    pub fn new() -> Self {
        Self {
            sample_count: 0,
        }
    }
    
    pub fn clock_envelopes(&mut self) {
        
    }
    
    pub fn generate_sample(&mut self, audio_buffer: &mut Vec<i16>) {
        const FREQ: f64 = 440.0;

        let t = self.sample_count as f64 / AUDIO_SAMPLE_HZ as f64;
        self.sample_count += 1;

        let sample = (t * FREQ * std::f64::consts::TAU).sin();
        let sample = (sample * i16::MAX as f64) as i16;

        audio_buffer.push(sample);
        audio_buffer.push(sample);
    }
}