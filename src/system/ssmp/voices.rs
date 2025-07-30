use crate::utils::{GetBits, GetBytes, SetBytes};

#[derive(Clone, Copy, Debug)]
enum GainMode {
    Fixed,
    Decrease,
    ExpDecrease,
    Increase,
    BentIncrease,
}

#[derive(Clone)]
pub struct Voice {
    lchannel_volume: i8,
    rchannel_volume: i8,
    pitch: u16,
    sample_source: u8,
    adsr_enable: bool,
    adsr_attack: u8,
    adsr_decay: u8,
    adsr_sustain_rate: u8,
    adsr_sustain_level: u8,
    gain_fixed: u8,
    gain_rate: u8,
    gain_mode: GainMode,
    envelope: u8,
    sample_out: u8,
    raw_bytes: [u8;10],
}

impl Voice {
    pub fn new() -> Self {
        Voice {
            lchannel_volume: 0,
            rchannel_volume: 0,
            pitch: 0,
            sample_source: 0,
            adsr_enable: false,
            adsr_attack: 0,
            adsr_decay: 0,
            adsr_sustain_rate: 0,
            adsr_sustain_level: 0,
            gain_fixed: 0,
            gain_rate: 0,
            gain_mode: GainMode::Fixed,
            envelope: 0,
            sample_out: 0,
            raw_bytes: [0;10],
        }
    }

    pub fn read_voice_reg(&self, addr: u8) -> u8 {
        self.raw_bytes[addr as usize]
    }

    pub fn write_voice_reg(&mut self, addr: u8, data: u8) {
        self.raw_bytes[addr as usize] = data;

        match addr {
            0 => self.lchannel_volume = data as i8,
            1 => self.rchannel_volume = data as i8,
            2 => self.pitch.set_lo(data),
            3 => {
                self.pitch.set_hi(data & 0x3F);
                self.raw_bytes[addr as usize] = data & 0x3F;
            }
            4 => self.sample_source = data,
            5 => {
                self.adsr_enable = data.bit_en(7);
                self.adsr_decay = (data >> 4) & 0x07;
                self.adsr_attack = data & 0x0F;
            }
            6 => {
                self.adsr_sustain_level = data >> 5;
                self.adsr_sustain_rate = data & 0x1F;
            }
            7 => {
                if data.bit_en(7) {
                    self.gain_mode = match (data >> 5) & 0x03 {
                        0 => GainMode::Decrease,
                        1 => GainMode::ExpDecrease,
                        2 => GainMode::Increase,
                        3 => GainMode::BentIncrease,
                        _ => unreachable!("Improper gain mode"),
                    }
                } else {
                    self.gain_mode = GainMode::Fixed;
                    self.gain_rate = data;
                }
            }
            8 => {},
            9 => {},
            _ => unreachable!(),
        }
    }
}