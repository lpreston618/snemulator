use std::{cell::Cell, rc::Rc};

use crate::utils::{GetBits, SetCellBytes};

#[derive(Clone, Copy, Debug)]
enum GainMode {
    Fixed,
    Decrease,
    ExpDecrease,
    Increase,
    BentIncrease,
}

#[derive(Clone)]
pub struct Registers {
    lchannel_volume: Cell<i8>,
    rchannel_volume: Cell<i8>,
    pitch: Cell<u16>,
    sample_source: Cell<u8>,
    adsr_enable: Cell<bool>,
    adsr_attack: Cell<u8>,
    adsr_decay: Cell<u8>,
    adsr_sustain_rate: Cell<u8>,
    adsr_sustain_level: Cell<u8>,
    gain_fixed: Cell<u8>,
    gain_rate: Cell<u8>,
    gain_mode: Cell<GainMode>,
    envelope: Cell<u8>,
    sample_out: Cell<u8>,
    raw_bytes: [Cell<u8>; 10],
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            lchannel_volume: Cell::new(0),
            rchannel_volume: Cell::new(0),
            pitch: Cell::new(0),
            sample_source: Cell::new(0),
            adsr_enable: Cell::new(false),
            adsr_attack: Cell::new(0),
            adsr_decay: Cell::new(0),
            adsr_sustain_rate: Cell::new(0),
            adsr_sustain_level: Cell::new(0),
            gain_fixed: Cell::new(0),
            gain_rate: Cell::new(0),
            gain_mode: Cell::new(GainMode::Fixed),
            envelope: Cell::new(0),
            sample_out: Cell::new(0),
            raw_bytes: [
                Cell::new(0), Cell::new(0), Cell::new(0), Cell::new(0), Cell::new(0),
                Cell::new(0), Cell::new(0), Cell::new(0), Cell::new(0), Cell::new(0),
            ],
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        self.raw_bytes[address as usize].get()
    }

    pub fn write(&self, address: u8, data: u8) {
        self.raw_bytes[address as usize].set(data);

        match address {
            0 => self.lchannel_volume.set(data as i8),
            1 => self.rchannel_volume.set(data as i8),
            2 => self.pitch.set_lo(data),
            3 => {
                self.pitch.set_hi(data & 0x3F);
                self.raw_bytes[address as usize].set(data & 0x3F);
            }
            4 => self.sample_source.set(data),
            5 => {
                self.adsr_enable.set(data.bit_en(7));
                self.adsr_decay.set((data >> 4) & 0x07);
                self.adsr_attack.set(data & 0x0F);
            }
            6 => {
                self.adsr_sustain_level.set(data >> 5);
                self.adsr_sustain_rate.set(data & 0x1F);
            }
            7 => {
                if data.bit_en(7) {
                    self.gain_mode.set(
                        match (data >> 5) & 0x03 {
                            0 => GainMode::Decrease,
                            1 => GainMode::ExpDecrease,
                            2 => GainMode::Increase,
                            3 => GainMode::BentIncrease,
                            _ => unreachable!("Improper gain mode"),
                        }
                    );
                } else {
                    self.gain_mode.set(GainMode::Fixed);
                    self.gain_rate.set(data);
                }
            }
            8 => {},
            9 => {},
            _ => unreachable!("Should never be called with other address"),
        }
    }
}

#[derive(Clone)]
pub struct Voice {
    registers: Rc<Registers>
}

impl Voice {
    pub fn new(voice_regs: Rc<Registers>) -> Self {
        Voice {
            registers: voice_regs,
        }
    }
}