use std::cell::Cell;
use std::rc::Rc;

use crate::system::ssmp::voices::{self, Voice};
use crate::utils::GetBits;

pub(super) struct Registers {
    lchannel_volume: Cell<i8>,
    rchannel_volume: Cell<i8>,
    lecho_volume: Cell<i8>,
    recho_volume: Cell<i8>,
    key_on: Cell<u8>,
    key_off: Cell<u8>,
    soft_reset: Cell<bool>,
    mute_all: Cell<bool>,
    echo_disable: Cell<bool>,
    noise_freq: Cell<u8>,
    
    echo_feedback: Cell<i8>,
    unused: Cell<u8>,
    voice_pitchmod_enable: Cell<u8>,
    voice_noise_enable: Cell<u8>,
    voice_echo_enable: Cell<u8>,
    sample_page: Cell<u8>,
    echo_page: Cell<u8>,
    echo_delay_time: Cell<u8>,
    filter_coeff: [Cell<u8>; 8],

    voice_regs: Vec<Rc<voices::Registers>>,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            lchannel_volume: Cell::new(0),
            rchannel_volume: Cell::new(0),
            lecho_volume: Cell::new(0),
            recho_volume: Cell::new(0),
            key_on: Cell::new(0),
            key_off: Cell::new(0),
            soft_reset: Cell::new(false),
            mute_all: Cell::new(false),
            echo_disable: Cell::new(false),
            noise_freq: Cell::new(0),
            echo_feedback: Cell::new(0),
            unused: Cell::new(0),
            voice_pitchmod_enable: Cell::new(0),
            voice_noise_enable: Cell::new(0),
            voice_echo_enable: Cell::new(0),
            sample_page: Cell::new(0),
            echo_page: Cell::new(0),
            echo_delay_time: Cell::new(0),
            filter_coeff: [
                Cell::new(0), Cell::new(0), Cell::new(0), Cell::new(0),
                Cell::new(0), Cell::new(0), Cell::new(0), Cell::new(0),
            ],
            voice_regs: vec![Rc::new(voices::Registers::new()); 8],
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        match (address >> 4, address & 0xF) {
            (voice @ 0..=7, addr @ 0..=9) => {
                self.voice_regs[voice as usize].read(addr)
            }
            (0, 0xC) => self.lchannel_volume.get() as u8,
            (1, 0xC) => self.rchannel_volume.get() as u8,
            (2, 0xC) => self.lecho_volume.get() as u8,
            (3, 0xC) => self.recho_volume.get() as u8,
            (4, 0xC) => self.key_on.get(),
            (5, 0xC) => self.key_off.get(),
            (6, 0xC) => {
                let r = self.soft_reset.get() as u8;
                let m = self.mute_all.get() as u8;
                let e = self.echo_disable.get() as u8;
                (r << 7) | (m << 6) | (e << 5) | self.noise_freq.get()
            }
            (7, 0xC) => todo!("Voice Sample ends"),
            (0, 0xD) => self.echo_feedback.get() as u8,
            (1, 0xD) => self.unused.get(),
            (2, 0xD) => self.voice_pitchmod_enable.get(),
            (3, 0xD) => self.voice_noise_enable.get(),
            (4, 0xD) => self.voice_echo_enable.get(),
            (5, 0xD) => self.sample_page.get(),
            (6, 0xD) => self.echo_page.get(),
            (7, 0xD) => self.echo_delay_time.get(),
            (x, 0xF) => self.filter_coeff[x as usize].get(),
            _ => {0}
        }
    }

    pub fn write(&self, address: u8, data: u8) {
        match (address >> 4, address & 0xF) {
            (voice @ 0..=7, addr @ 0..=9) => {
                self.voice_regs[voice as usize].write(addr, data);
            }
            (0, 0xC) => self.lchannel_volume.set(data as i8),
            (1, 0xC) => self.rchannel_volume.set(data as i8),
            (2, 0xC) => self.lecho_volume.set(data as i8),
            (3, 0xC) => self.recho_volume.set(data as i8),
            (4, 0xC) => self.key_on.set(data), // maybe do more stuff here
            (5, 0xC) => self.key_off.set(data),
            (6, 0xC) => {
                self.soft_reset.set(data.bit_en(7));
                self.mute_all.set(data.bit_en(6));
                self.echo_disable.set(data.bit_en(5));
                self.noise_freq.set(data & 0x1F);
            }
            (7, 0xC) => {},
            (0, 0xD) => self.echo_feedback.set(data as i8),
            (1, 0xD) => self.unused.set(data),
            (2, 0xD) => self.voice_pitchmod_enable.set(data),
            (3, 0xD) => self.voice_noise_enable.set(data),
            (4, 0xD) => self.voice_echo_enable.set(data),
            (5, 0xD) => self.sample_page.set(data),
            (6, 0xD) => self.echo_page.set(data),
            (7, 0xD) => self.echo_delay_time.set(data),
            (x, 0xF) => self.filter_coeff[x as usize].set(data),
            _ => {}
        }
    }
}

pub(super) struct SuperDSP {
    voices: Vec<Voice>,
    registers: Rc<Registers>,
    aram: Rc<Vec<Cell<u8>>>,
}

impl SuperDSP {
    pub fn new(aram: Rc<Vec<Cell<u8>>>, sdsp_regs: Rc<Registers>) -> SuperDSP {
        let mut voice_vec = Vec::new();
    
        for regs in sdsp_regs.voice_regs.iter() {
            voice_vec.push(voices::Voice::new(regs.clone()));
        }

        SuperDSP { 
            voices: voice_vec,
            registers: sdsp_regs,
            aram,
       }
    }

    pub fn clock(&mut self, time: f32) {
        
    }

    pub fn generate_sample(&mut self, audio_buffer: &mut Vec<i16>, time: f32) {
        const SAMPLE_FREQ: f32 = 440.0;

        audio_buffer.push( ((time * SAMPLE_FREQ * std::f32::consts::TAU).sin() * i16::MAX as f32) as i16 );
        audio_buffer.push( ((time * SAMPLE_FREQ * std::f32::consts::TAU).sin() * i16::MAX as f32) as i16 );
    }
}