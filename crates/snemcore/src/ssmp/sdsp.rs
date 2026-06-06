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
    time: std::time::Instant,
}

impl SuperDSP {
    pub fn new() -> Self {
        Self {
            time: std::time::Instant::now(),
        }
    }
    
    pub fn clock_envelopes(&mut self) {
        
    }
    
    pub fn generate_sample(&mut self, audio_buffer: &mut Vec<i16>) {
        const FREQ: f64 = 440.0;

        let t = self.time.elapsed().as_secs_f64();

        let sample = (t * FREQ * std::f64::consts::TAU).sin();
        let sample = (sample * i16::MAX as f64) as i16;

        audio_buffer.push(sample);
        audio_buffer.push(sample);
    }
}