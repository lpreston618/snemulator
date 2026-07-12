use crate::debug::DebugHarness;
use crate::savestate;
use crate::ssmp::ioports::ApuIoPorts;
use crate::ssmp::sdsp::{SuperDSP, bus::SdspBus, regs::SdspRegs, voices::VoiceRegs};
use crate::ssmp::spc::{Spc700, bus::SpcBus, ioregs::SpcIoRegs};
use crate::ssmp::timers::Timer;
use crate::sysinfo::{
    ARAM_SIZE,
    AUDIO_SAMPLE_HZ, MASTER_CLOCK_HZ, SPC_CLOCK_HZ,
    SLOW_TIMER_CLOCK_PERIOD, FAST_TIMER_CLOCK_PERIOD,
};

pub mod ioports;
pub mod sdsp;
pub mod spc;
mod timers;

/// The sound processor chip of the S-NES. Contains the SPC700 and S-DSP.
pub struct Ssmp {
    pub spc: Spc700,
    pub sdsp: sdsp::SuperDSP,

    pub aram: Box<[u8; ARAM_SIZE]>,
    pub spc_regs: SpcIoRegs,
    pub sdsp_regs: SdspRegs,
    pub timer0: Timer<SLOW_TIMER_CLOCK_PERIOD>,
    pub timer1: Timer<SLOW_TIMER_CLOCK_PERIOD>,
    pub timer2: Timer<FAST_TIMER_CLOCK_PERIOD>,
    pub voice_regs: [VoiceRegs; 8],

    pub samples_generated: u64,

    sample_cycle_accumulator: usize,
    spc_cycle_accumulator: usize,
}

impl Ssmp {
    pub fn new() -> Ssmp {
        Ssmp {
            spc: Spc700::default(),
            sdsp: SuperDSP::new(),

            aram: Box::new([0u8; ARAM_SIZE]),
            spc_regs: SpcIoRegs::default(),
            sdsp_regs: SdspRegs::default(),
            timer0: Timer::new(),
            timer1: Timer::new(),
            timer2: Timer::new(),
            voice_regs: [VoiceRegs::new(); 8],

            samples_generated: 0,

            sample_cycle_accumulator: 0,
            spc_cycle_accumulator: 0,
        }
    }

    pub fn save_state(&self) -> savestate::ApuState {
        savestate::ApuState {
            sample_cycle_accumulator: self.sample_cycle_accumulator,
            spc_cycle_accumulator: self.spc_cycle_accumulator,
            spc: self.spc.save_state(&self.spc_regs),
            sdsp: self.sdsp.save_state(&self.sdsp_regs),
            voices: [
                self.voice_regs[0].save_state(), self.voice_regs[1].save_state(),
                self.voice_regs[2].save_state(), self.voice_regs[3].save_state(),
                self.voice_regs[4].save_state(), self.voice_regs[5].save_state(),
                self.voice_regs[6].save_state(), self.voice_regs[7].save_state(),
            ],
            timers: [
                self.timer0.save_state(),
                self.timer1.save_state(),
                self.timer2.save_state(),
            ],
        }
    }

    pub fn load_state(&mut self, state: &savestate::ApuState, version: u32) {
        self.sample_cycle_accumulator = state.sample_cycle_accumulator;
        self.spc_cycle_accumulator = state.spc_cycle_accumulator;
        self.spc.load_state(&mut self.spc_regs, &state.spc, version);
        self.sdsp.load_state(&mut self.sdsp_regs, &state.sdsp, version);

        for (v, voice) in self.voice_regs.iter_mut().enumerate() {
            voice.load_state(&state.voices[v], version);
        }

        self.timer0.load_state(&state.timers[0], version);
        self.timer1.load_state(&state.timers[1], version);
        self.timer2.load_state(&state.timers[2], version);
    }

    pub fn power_on(&mut self) {
        self.sample_cycle_accumulator = 0;
        self.spc_cycle_accumulator = 0;

        self.spc.power_on();
        self.spc_regs.power_on();
        self.sdsp.power_on();
        self.sdsp_regs.power_on();

        self.aram.chunks_mut(32).enumerate().for_each(|(i, chunk)| {
            if i % 2 == 0 {
                chunk.copy_from_slice(&[0x00; 32]);
            } else {
                chunk.copy_from_slice(&[0xFF; 32]);
            }
        });

        self.timer0.reset();
        self.timer1.reset();
        self.timer2.reset();

        self.voice_regs.iter_mut().for_each(|voice| voice.power_on());

        self.samples_generated = 0;
    }

    pub fn reset(&mut self) {
        self.sample_cycle_accumulator = 0;
        self.spc_cycle_accumulator = 0;

        self.spc.reset();
        self.spc_regs.reset();
        self.sdsp.reset();
        self.sdsp_regs.reset();

        self.timer0.reset();
        self.timer1.reset();
        self.timer2.reset();

        self.voice_regs.iter_mut().for_each(|voice| voice.reset());

        self.samples_generated = 0;
    }

    /// Clocks the sound processor, checking if it is time to generate a new
    /// sample and/or clock the S-DSP and SPC700 processors.
    pub fn cycle<H: DebugHarness>(&mut self, clocks: usize, audio_buffer: &mut Vec<i16>, apu_regs: &mut ApuIoPorts, harness: &mut H) {
        self.sample_cycle_accumulator += clocks * AUDIO_SAMPLE_HZ;
        self.spc_cycle_accumulator += clocks * SPC_CLOCK_HZ;

        while self.sample_cycle_accumulator >= MASTER_CLOCK_HZ {
            self.sample_cycle_accumulator -= MASTER_CLOCK_HZ;

            let mut sdsp_bus = SdspBus {
                aram: &mut self.aram,
                sdsp_regs: &mut self.sdsp_regs,
                voice_regs: &mut self.voice_regs,
            };

            self.sdsp.clock_envelopes(&mut sdsp_bus, harness);
            self.sdsp.generate_sample(audio_buffer, &mut sdsp_bus);

            self.samples_generated += 1;

            if H::IS_DEBUGGING_HARNESS && H::TRACK_SAMPLE_OUTPUT {
                harness.on_sample_generated(self);
            }
        }

        while self.spc_cycle_accumulator >= MASTER_CLOCK_HZ {
            self.spc_cycle_accumulator -= MASTER_CLOCK_HZ;

            let mut spc_bus = SpcBus {
                aram: &mut self.aram,
                spc_regs: &mut self.spc_regs,
                sdsp_regs: &mut self.sdsp_regs,
                timer0: &mut self.timer0,
                timer1: &mut self.timer1,
                timer2: &mut self.timer2,
                voice_regs: &mut self.voice_regs,
                apuio_regs: apu_regs,
                harness,
            };

            self.spc.clock(&mut spc_bus);
            
            self.timer0.clock();
            self.timer1.clock();
            self.timer2.clock();
        }
    }

    pub fn aram_slice(&self) -> &[u8] {
        &self.aram[..]
    }
}