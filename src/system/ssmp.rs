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

/// Frequency of the SNES master clock
const MASTER_CLOCK_HZ: usize = 21477300;
/// Amount of time (in seconds) each system clock takes.
const MASTER_CLOCK_PERIOD: f64 = 1.0 / MASTER_CLOCK_HZ as f64;
/// Frequency of the SPC700 internal clock
const SPC_CLOCK_HZ: usize = 1024000;
/// Amount of time (in seconds) each SPC700 clock cycle takes.
const SPC_CLOCK_PERIOD: f64 = 1.0 / SPC_CLOCK_HZ as f64;
/// Amount of time (in seconds) between playing each sample.
const AUDIO_SAMPLE_PERIOD: f64 = 1.0 / AUDIO_FREQ as f64;

// // How long to wait between generating samples before deciding to drop all the
// // samples we are behind by. This is useful when the emulator is paused, for
// // example, so we don't try to generate millions of samples at once.
// const SAMPLE_DROP_TIME: f64 = TIME_PER_SAMPLE * 64.0;

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
    next_spc_clock: f64,
    frame_time: f64,

    debug_cnt: usize,

    logger: Rc<SnemLogger>
}

impl Ssmp {
    pub fn new(apuio_regs: Rc<ApuIORegs>, logger: Rc<SnemLogger>) -> Ssmp {
        let smp_data = Rc::new(SmpData::new());

        Ssmp {
            spc: spc::Spc700::new(apuio_regs, smp_data.clone(), logger.clone()),
            sdsp: sdsp::SuperDSP::new(smp_data),
            
            next_sample: 0.0,
            next_spc_clock: 0.0,
            frame_time: 0.0,

            debug_cnt: 0,

            logger,
        }
    }

    pub fn finish(&mut self) {
        self.sdsp.finish();

        self.logger.log(LogLevel::Info, "S-Smp finishing.");
    }

    pub fn start_frame(&mut self) {
        self.next_sample -= self.frame_time;
        self.next_spc_clock -= self.frame_time;
        self.frame_time = 0.0;
    }

    /// Clocks the sound processor, checking if it is time to generate a new
    /// sample and/or clock the S-DSP and SPC700 processors.
    pub fn clock(&mut self, master_clocks: usize, audio_buffer: &mut Vec<i16>) {
        self.frame_time += MASTER_CLOCK_PERIOD * master_clocks as f64;
        self.debug_cnt += master_clocks;

        if self.frame_time >= self.next_sample {
            self.next_sample += AUDIO_SAMPLE_PERIOD;

            self.sdsp.clock_envelopes();
            self.sdsp.generate_sample(audio_buffer);

            self.spc.inc_debug_cnt();
        }

        if self.frame_time >= self.next_spc_clock {
            self.next_spc_clock += SPC_CLOCK_PERIOD;

            self.spc.clock();
        }
    }
}