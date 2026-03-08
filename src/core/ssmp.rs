use crate::{app::AUDIO_SAMPLE_HZ, core::{ssmp::{ioports::ApuIoPorts, sdsp::{SuperDSP, regs::SdspRegs, voices::VoiceRegs}, spc::{Spc700, bus::SpcBus, ioregs::SpcIoRegs}, timers::Timer}, sysinfo::{ARAM_SIZE, MASTER_CLOCK_HZ, SPC_CLOCK_HZ}}};

pub mod ioports;
mod timers;
mod spc;
mod sdsp;

// TIMER2 runs at 64kHz, which translates to one tick per every 48 DSP clocks.
// The Spc700 clocks every 3, DSP clocks, so TIMER2 is clocked once every 48/3,
// or 16, Spc700 clocks.
// Timers 0 and 1 each run at 1/8th that speed (8kHz), so they are clocked every
// 16*8, or 128, SPC clocks.
pub const FAST_TIMER_CLOCK_PERIOD: usize = 16;
pub const SLOW_TIMER_CLOCK_PERIOD: usize = 128;

const MASTER_CLOCK_PERIOD: f32 = 1.0 / MASTER_CLOCK_HZ as f32;
const AUDIO_SAMPLE_PERIOD: f32 = 1.0 / AUDIO_SAMPLE_HZ as f32;
const SPC_CLOCK_PERIOD: f32 = 1.0 / SPC_CLOCK_HZ as f32;

/// The sound processor chip of the S-NES. Contains the SPC700 and S-DSP.
pub struct Ssmp {
    spc: Spc700,
    sdsp: sdsp::SuperDSP,
    
    aram: Box<[u8; ARAM_SIZE]>,
    spc_regs: SpcIoRegs,
    sdsp_regs: SdspRegs,
    timer0: Timer<SLOW_TIMER_CLOCK_PERIOD>,
    timer1: Timer<SLOW_TIMER_CLOCK_PERIOD>,
    timer2: Timer<FAST_TIMER_CLOCK_PERIOD>,
    voice_regs: [VoiceRegs; 8],

    next_sample: f32,
    next_spc_clock: f32,
    frame_time: f32,
}

impl Ssmp {
    pub fn new() -> Ssmp {
        Ssmp {
            spc: Spc700::new(),
            sdsp: SuperDSP::new(),
            
            aram: Box::new([0u8; ARAM_SIZE]),
            spc_regs: SpcIoRegs::new(),
            sdsp_regs: SdspRegs::new(),
            timer0: Timer::new(),
            timer1: Timer::new(),
            timer2: Timer::new(),
            voice_regs: [VoiceRegs::new(); 8],
            
            next_sample: 0.0,
            next_spc_clock: 0.0,
            frame_time: 0.0,
        }
    }

    // pub fn finish(&mut self) {
    //     self.sdsp.finish();
    // }

    pub fn start_frame(&mut self) {
        self.next_sample -= self.frame_time;
        self.next_spc_clock -= self.frame_time;
        self.frame_time = 0.0;
    }

    /// Clocks the sound processor, checking if it is time to generate a new
    /// sample and/or clock the S-DSP and SPC700 processors.
    pub fn clock(&mut self, clocks: usize, audio_buffer: &mut Vec<i16>, apu_regs: &mut ApuIoPorts) {
        self.frame_time += MASTER_CLOCK_PERIOD * clocks as f32;
        // self.debug_cnt += master_clocks;

        if self.frame_time >= self.next_sample {
            self.next_sample += AUDIO_SAMPLE_PERIOD;

            self.sdsp.clock_envelopes();
            self.sdsp.generate_sample();
        }

        if self.frame_time >= self.next_spc_clock {
            self.next_spc_clock += SPC_CLOCK_PERIOD;
            
            let mut bus = SpcBus {
                aram: &mut self.aram,
                spc_regs: &mut self.spc_regs,
                sdsp_regs: &mut self.sdsp_regs,
                timer0: &mut self.timer0,
                timer1: &mut self.timer1,
                timer2: &mut self.timer2,
                voice_regs: &mut self.voice_regs,
                apuio_regs: apu_regs,
            };

            self.spc.clock(&mut bus);
        }
    }
}