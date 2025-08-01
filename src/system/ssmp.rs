mod disassembler;
mod sdsp;
mod channel;
mod voices;
mod spc;
mod timer;

use std::{cell::Cell, rc::Rc};

use crate::log::{LogLevel, SnemLogger};
use crate::audio::AUDIO_FREQ;

const SDSP_CLOCK_HZ: usize = 3072000;
const SDSP_CLOCK_PERIOD: f32 = 1.0 / SDSP_CLOCK_HZ as f32;

/// Magic number used to increase the speed at which samples are generated 
/// faster than they are played to ensure we always have enough to play.
const MAGIC: f32 = 1e-5;

/// Time (in seconds) between playing each sample.
const TIME_PER_SAMPLE: f32 = 1.0 / AUDIO_FREQ as f32;

/// How long to wait between generating samples before deciding to drop all the
/// samples we are behind by. This is useful when the emulator is paused, for
/// example, so we don't try to generate millions of samples at once.
const SAMPLE_DROP_TIME: f32 = TIME_PER_SAMPLE * 100.0;

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
    sdsp_clocks: usize,

    next_sample: f32,
    last_sdsp_clock: std::time::Instant,
    start_time: std::time::Instant,

    logger: Rc<SnemLogger>
}

impl Ssmp {
    pub fn new(apuio_regs: Rc<ApuIORegs>, logger: Rc<SnemLogger>) -> Ssmp {
        let aram = Rc::new(vec![Cell::new(0); ARAM_SIZE]);
        let sdsp_regs = Rc::new(sdsp::Registers::new());

        Ssmp {
            spc: spc::Spc700::new(apuio_regs, aram.clone(), sdsp_regs.clone(), logger.clone()),
            sdsp: sdsp::SuperDSP::new(aram, sdsp_regs),
            sdsp_clocks: 0,
            
            next_sample: 0.0,
            last_sdsp_clock: std::time::Instant::now(),
            start_time: std::time::Instant::now(),

            logger,
        }
    }

    /// Used to purely generate samples in case of audio buffer underrun, i.e.,
    /// the S-DSP is not clocked.
    pub fn generate_samples(&mut self, audio_buffer: &mut Vec<i16>, num_samples: usize) {
        for _ in 0..num_samples {
            self.sdsp.generate_sample(audio_buffer);
        }
    }

    /// Clocks the sound processor, checking if it is time to generate a new
    /// sample and/or clock the S-DSP and SPC700 processors.
    pub fn clock(&mut self, audio_buffer: &mut Vec<i16>, generate_sample: bool) {
        let time = self.start_time.elapsed().as_secs_f32();

        // If we are too far behind, drop the missing samples
        if (time - self.next_sample).abs() >= SAMPLE_DROP_TIME {
            self.next_sample = time;
        }

        if generate_sample && time >= self.next_sample {
            self.next_sample += TIME_PER_SAMPLE - MAGIC;

            self.sdsp.generate_sample(audio_buffer);
        }

        let time_since_sdsp_clock = self.last_sdsp_clock.elapsed().as_secs_f32();

        if time_since_sdsp_clock >= SDSP_CLOCK_PERIOD {
            self.last_sdsp_clock = std::time::Instant::now();

            // 3.072 MHz
            self.sdsp.clock();

            self.sdsp_clocks += 1;

            // Spc700 clocks every 3 S-DSP cycles
            if self.sdsp_clocks == 3 {
                self.sdsp_clocks = 0;

                self.spc.clock();
            }
        }
    }
}