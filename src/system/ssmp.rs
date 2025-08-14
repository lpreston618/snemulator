mod disassembler;
mod sdsp;
mod channel;
mod spc;
mod timer;
mod state;
mod utils;

pub use state::SmpData;

use std::{cell::Cell, rc::Rc};

use crate::log::{LogLevel, SnemLogger};
use crate::audio::AUDIO_FREQ;

const MASTER_CLOCK_HZ: usize = 21477300;
const MASTER_CLOCK_PERIOD: f64 = 1.0 / MASTER_CLOCK_HZ as f64;
const SDSP_CLOCK_HZ: usize = 3072000;
const SMP_CLOCK_HZ: usize = SDSP_CLOCK_HZ / 3;
// const SMP_CLOCK_PERIOD: f64 = 1.0 / SDSP_CLOCK_HZ as f64;
const SMP_CLOCK_PERIOD: f64 = 1.0 / SMP_CLOCK_HZ as f64;

/// Time (in seconds) between playing each sample.
const TIME_PER_SAMPLE: f64 = 1.0 / AUDIO_FREQ as f64;

/// How long to wait between generating samples before deciding to drop all the
/// samples we are behind by. This is useful when the emulator is paused, for
/// example, so we don't try to generate millions of samples at once.
const SAMPLE_DROP_TIME: f64 = TIME_PER_SAMPLE * 64.0;

/// 64 KiB of Audio RAM
const ARAM_SIZE: usize = 0x10000;

/// Shared registers between the S-CPU and SPC700
pub struct ApuIORegs {
    /// SPC700 -> S-CPU register 0
    pub apuio0: Cell<u8>,
    /// SPC700 -> S-CPU register 1
    pub apuio1: Cell<u8>,
    /// SPC700 -> S-CPU register 2
    pub apuio2: Cell<u8>,
    /// SPC700 -> S-CPU register 3
    pub apuio3: Cell<u8>,

    // S-CPU -> SPC700 register 0
    pub cpuio0: Cell<u8>,
    // S-CPU -> SPC700 register 1
    pub cpuio1: Cell<u8>,
    // S-CPU -> SPC700 register 2
    pub cpuio2: Cell<u8>,
    // S-CPU -> SPC700 register 3
    pub cpuio3: Cell<u8>,
}

impl ApuIORegs {
    pub fn new() -> ApuIORegs {
        ApuIORegs {
            apuio0: Cell::new(0),
            apuio1: Cell::new(0),
            apuio2: Cell::new(0),
            apuio3: Cell::new(0),
            cpuio0: Cell::new(0),
            cpuio1: Cell::new(0),
            cpuio2: Cell::new(0),
            cpuio3: Cell::new(0),
        }
    }
}

/// The sound processor chip of the S-NES. Contains the SPC700 and S-DSP.
pub struct Ssmp {
    spc: spc::Spc700,
    sdsp: sdsp::SuperDSP,

    next_sample: f64,
    next_smp_clock: f64,
    frame_time: f64,
    last_smp_clock: std::time::Instant,
    start_time: std::time::Instant,

    samples_generated: usize,

    logger: Rc<SnemLogger>
}

impl Ssmp {
    pub fn new(apuio_regs: Rc<ApuIORegs>, logger: Rc<SnemLogger>) -> Ssmp {
        let smp_data = Rc::new(SmpData::new());

        Ssmp {
            spc: spc::Spc700::new(apuio_regs, smp_data.clone(), logger.clone()),
            sdsp: sdsp::SuperDSP::new(smp_data),
            
            next_sample: 0.0,
            next_smp_clock: 0.0,
            frame_time: 0.0,
            last_smp_clock: std::time::Instant::now(),
            start_time: std::time::Instant::now(),

            samples_generated: 0,

            logger,
        }
    }

    pub fn finish(&mut self) {
        self.sdsp.finish();

        self.logger.log(LogLevel::Info, "S-Smp finishing.");
    }

    /// Used to purely generate samples in case of audio buffer underrun, i.e.,
    /// the S-DSP is not clocked.
    // pub fn generate_samples(&mut self, audio_buffer: &mut Vec<i16>, num_samples: usize) {
    //     for _ in 0..num_samples {
    //         self.sdsp.generate_sample(audio_buffer);
    //     }
    // }

    pub fn start_frame(&mut self) {
        self.next_sample -= self.frame_time;
        self.next_smp_clock -= self.frame_time;
        self.frame_time = 0.0;
    }

    /// Clocks the sound processor, checking if it is time to generate a new
    /// sample and/or clock the S-DSP and SPC700 processors.
    pub fn clock(&mut self, audio_buffer: &mut Vec<i16>, master_clocks: usize) {
        self.frame_time += MASTER_CLOCK_PERIOD * master_clocks as f64;

        if self.frame_time >= self.next_sample {
            self.next_sample += TIME_PER_SAMPLE;
            self.samples_generated += 1;
            
            self.sdsp.clock_envelopes();
            self.sdsp.generate_sample(audio_buffer);
        }

        if self.frame_time >= self.next_smp_clock {
            self.next_smp_clock += SMP_CLOCK_PERIOD;

            self.spc.clock();
        }
    }
}