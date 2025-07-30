use crate::system::ssmp::voices::Voice;
use crate::system::ssmp::SDSP_CLOCK_PERIOD;
use crate::utils::GetBits;

pub(super) struct SuperDSP {
    voices: Vec<Voice>,
    lchannel_volume: i8,
    rchannel_volume: i8,
    lecho_volume: i8,
    recho_volume: i8,
    key_on: u8,
    key_off: u8,
    soft_reset: bool,
    mute_all: bool,
    echo_disable: bool,
    noise_freq: u8,
    
    echo_feedback: i8,
    unused: u8,
    voice_pitchmod_enable: u8,
    voice_noise_enable: u8,
    voice_echo_enable: u8,
    sample_page: u8,
    echo_page: u8,
    echo_delay_time: u8,
    filter_coeff: [u8; 8],
}

impl SuperDSP {
    pub fn new() -> SuperDSP {
        SuperDSP { 
            voices: vec![Voice::new(); 8],
            lchannel_volume: 0,
            rchannel_volume: 0,
            lecho_volume: 0,
            recho_volume: 0,
            key_on: 0,
            key_off: 0,
            soft_reset: false,
            mute_all: false,
            echo_disable: false,
            noise_freq: 0,
            echo_feedback: 0,
            unused: 0,
            voice_pitchmod_enable: 0,
            voice_noise_enable: 0,
            voice_echo_enable: 0,
            sample_page: 0,
            echo_page: 0,
            echo_delay_time: 0,
            filter_coeff: [0; 8],
       }
    }

    pub fn clock(&mut self) {
        
    }

    pub fn generate_sample(&mut self, audio_buffer: &mut Vec<i16>, time: f32) {
        const SAMPLE_FREQ: f32 = 440.0;

        audio_buffer.push( ((time * SAMPLE_FREQ * std::f32::consts::TAU).sin() * i16::MAX as f32) as i16 );
        audio_buffer.push( ((time * SAMPLE_FREQ * std::f32::consts::TAU).sin() * i16::MAX as f32) as i16 );
    }
}

impl SuperDSP {
    pub fn read_reg(&mut self, address: u8) -> u8 {
        // println!("Read SDSP reg at ${address:02X}");
        match (address >> 4, address & 0xF) {
            (voice @ 0..=7, addr @ 0..=9) => {
                self.voices[voice as usize].read_voice_reg(addr)
            }
            (0, 0xC) => self.lchannel_volume as u8,
            (1, 0xC) => self.rchannel_volume as u8,
            (2, 0xC) => self.lecho_volume as u8,
            (3, 0xC) => self.recho_volume as u8,
            (4, 0xC) => self.key_on,
            (5, 0xC) => self.key_off,
            (6, 0xC) => {
                let r = self.soft_reset as u8;
                let m = self.mute_all as u8;
                let e = self.echo_disable as u8;
                (r << 7) | (m << 6) | (e << 5) | self.noise_freq
            }
            (7, 0xC) => self.get_voice_sample_ends(),
            (0, 0xD) => self.echo_feedback as u8,
            (1, 0xD) => self.unused,
            (2, 0xD) => self.voice_pitchmod_enable,
            (3, 0xD) => self.voice_noise_enable,
            (4, 0xD) => self.voice_echo_enable,
            (5, 0xD) => self.sample_page,
            (6, 0xD) => self.echo_page,
            (7, 0xD) => self.echo_delay_time,
            (x, 0xF) => self.filter_coeff[x as usize],
            _ => {0}
        }
    }

    pub fn write_reg(&mut self, address: u8, data: u8) {
        match (address >> 4, address & 0xF) {
            (voice @ 0..=7, addr @ 0..=9) => {
                self.voices[voice as usize].write_voice_reg(addr, data);
            }
            (0, 0xC) => self.lchannel_volume = data as i8,
            (1, 0xC) => self.rchannel_volume = data as i8,
            (2, 0xC) => self.lecho_volume = data as i8,
            (3, 0xC) => self.recho_volume = data as i8,
            (4, 0xC) => self.key_on = data, // maybe do more stuff here
            (5, 0xC) => self.key_off = data,
            (6, 0xC) => {
                self.soft_reset = data.bit_en(7);
                self.mute_all = data.bit_en(6);
                self.echo_disable = data.bit_en(5);
                self.noise_freq = data & 0x1F;
            }
            (7, 0xC) => {},
            (0, 0xD) => self.echo_feedback = data as i8,
            (1, 0xD) => self.unused = data,
            (2, 0xD) => self.voice_pitchmod_enable = data,
            (3, 0xD) => self.voice_noise_enable = data,
            (4, 0xD) => self.voice_echo_enable = data,
            (5, 0xD) => self.sample_page = data,
            (6, 0xD) => self.echo_page = data,
            (7, 0xD) => self.echo_delay_time = data,
            (x, 0xF) => self.filter_coeff[x as usize] = data,
            _ => {}
        }
    }

    fn get_voice_sample_ends(&self) -> u8 {
        // do stuff here later
        0
    }
}