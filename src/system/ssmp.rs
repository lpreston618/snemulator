mod disassembler;
mod sdsp;
mod channel;
mod voices;
mod spc;
mod timer;

use std::{cell::Cell, rc::Rc};

use crate::log::{LogLevel, SnemLogger};
use crate::libretro::AUDIO_FREQ;

const SDSP_CLOCK_HZ: usize = 3072000;
const SDSP_CLOCK_PERIOD: f32 = 1.0 / SDSP_CLOCK_HZ as f32;

const TIME_PER_SAMPLE: f32 = 1.0 / AUDIO_FREQ as f32;

const ARAM_SIZE: usize = 0x10000; // 64 KiB of Audio RAM

pub struct ApuIORegs {
    // APU -> CPU regs
    pub apuio0: Cell<u8>,
    pub apuio1: Cell<u8>,
    pub apuio2: Cell<u8>,
    pub apuio3: Cell<u8>,

    // CPU -> APU regs
    pub cpuio0: Cell<u8>,
    pub cpuio1: Cell<u8>,
    pub cpuio2: Cell<u8>,
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

pub struct Ssmp {
    spc: spc::Spc700,
    sdsp: sdsp::SuperDSP,
    sdsp_clocks: usize,

    last_sample: std::time::Instant,
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
            
            last_sample: std::time::Instant::now(),
            last_sdsp_clock: std::time::Instant::now(),
            start_time: std::time::Instant::now(),

            logger,
        }
    }

    pub fn clock(&mut self, audio_buffer: &mut Vec<i16>) {
        let time_since_last_sample = self.last_sample.elapsed().as_secs_f32();

        if time_since_last_sample >= TIME_PER_SAMPLE {
            self.last_sample = std::time::Instant::now();

            let time = self.start_time.elapsed().as_secs_f32();

            self.sdsp.generate_sample(audio_buffer, time);
        }

        let time_since_sdsp_clock = self.last_sdsp_clock.elapsed().as_secs_f32();

        if time_since_sdsp_clock >= SDSP_CLOCK_PERIOD {
            self.last_sdsp_clock = std::time::Instant::now();

            let time = self.start_time.elapsed().as_secs_f32();

            // 3.072 MHz
            self.sdsp.clock(time);

            self.sdsp_clocks += 1;

            // Spc700 clocks every 3 S-DSP cycles
            if self.sdsp_clocks == 3 {
                self.sdsp_clocks = 0;

                self.spc.clock();
            }
        }
    }
}