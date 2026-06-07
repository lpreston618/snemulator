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
mod sdsp;
mod spc;
mod timers;

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

            sample_cycle_accumulator: 0,
            spc_cycle_accumulator: 0,
        }
    }

    pub fn power_on(&mut self) {
        self.sample_cycle_accumulator = 0;
        self.spc_cycle_accumulator = 0;

        self.spc.power_on();
        self.spc_regs.power_on();
        self.sdsp.power_on();
        self.sdsp_regs.reset();

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

        self.voice_regs.iter_mut().for_each(|voice| voice.reset());
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
    }

    /// Clocks the sound processor, checking if it is time to generate a new
    /// sample and/or clock the S-DSP and SPC700 processors.
    pub fn cycle(&mut self, clocks: usize, audio_buffer: &mut Vec<i16>, apu_regs: &mut ApuIoPorts) {
        self.sample_cycle_accumulator += clocks * AUDIO_SAMPLE_HZ;
        self.spc_cycle_accumulator += clocks * SPC_CLOCK_HZ;

        if self.sample_cycle_accumulator >= MASTER_CLOCK_HZ {
            self.sample_cycle_accumulator -= MASTER_CLOCK_HZ;

            let mut sdsp_bus = SdspBus {
                aram: &mut self.aram,
                sdsp_regs: &mut self.sdsp_regs,
                voice_regs: &mut self.voice_regs,
            };

            self.sdsp.clock_envelopes(&mut sdsp_bus);
            self.sdsp.generate_sample(audio_buffer, &mut sdsp_bus);
        }

        if self.spc_cycle_accumulator >= MASTER_CLOCK_HZ {
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
            };

            self.spc.clock(&mut spc_bus);

            self.timer0.clock();
            self.timer1.clock();
            self.timer2.clock();
        }
    }

    pub fn cycle_no_output(&mut self, clocks: usize, apu_regs: &mut ApuIoPorts) {
        self.sample_cycle_accumulator += clocks * AUDIO_SAMPLE_HZ;
        self.spc_cycle_accumulator += clocks * SPC_CLOCK_HZ;

        if self.sample_cycle_accumulator >= MASTER_CLOCK_HZ {
            self.sample_cycle_accumulator -= MASTER_CLOCK_HZ;

            // self.sdsp.clock_envelopes();
            // self.sdsp.generate_sample(audio_buffer);
        }

        if self.spc_cycle_accumulator >= MASTER_CLOCK_HZ {
            self.spc_cycle_accumulator -= MASTER_CLOCK_HZ;

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

            self.timer0.clock();
            self.timer1.clock();
            self.timer2.clock();
        }
    }

    pub fn aram_slice(&self) -> &[u8] {
        &self.aram[..]
    }
}
