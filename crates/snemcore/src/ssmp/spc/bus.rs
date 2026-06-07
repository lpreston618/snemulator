use crate::ssmp::sdsp::{ADSRStage, GainMode};
use crate::ssmp::{FAST_TIMER_CLOCK_PERIOD, SLOW_TIMER_CLOCK_PERIOD};
use crate::ssmp::ioports::ApuIoPorts;
use crate::ssmp::sdsp::regs::SdspRegs;
use crate::ssmp::sdsp::voices::VoiceRegs;
use crate::ssmp::spc::{
    Spc700, 
    ioregs::SpcIoRegs
};
use crate::ssmp::timers::Timer;
use crate::sysinfo::ARAM_SIZE;
use crate::{get_bit_n, get_byte_n, set_byte_n};

pub struct SpcBus<'a> {
    pub aram: &'a mut [u8; ARAM_SIZE],
    pub spc_regs: &'a mut SpcIoRegs,
    pub sdsp_regs: &'a mut SdspRegs,
    pub timer0: &'a mut Timer<SLOW_TIMER_CLOCK_PERIOD>,
    pub timer1: &'a mut Timer<SLOW_TIMER_CLOCK_PERIOD>,
    pub timer2: &'a mut Timer<FAST_TIMER_CLOCK_PERIOD>,
    pub voice_regs: &'a mut [VoiceRegs; 8],
    pub apuio_regs: &'a mut ApuIoPorts,
}

impl<'a> SpcBus<'a> {
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x00F0..=0x00FF => self.read_spcio_regs(addr as u8),
            0xFFC0..=0xFFFF => {
                match self.spc_regs.ipl_read_en {
                    true => Spc700::IPL_ROM[(addr as usize) - 0xFFC0],
                    false => self.aram[addr as usize],
                }
            },
            
            _ => self.aram[addr as usize],
        }
    }
    
    pub fn write(&mut self, addr: u16, value: u8) {
        self.aram[addr as usize] = value; // All writes to wram go through
        
        if 0x00F0 <= addr && addr <= 0x00FF {
            self.write_spcio_regs(addr as u8, value);
        }
    }
    
    fn read_spcio_regs(&mut self, addr: u8) -> u8 {
        match addr & 0xF {
            0x0 => 0, // Test Register (Unused)
            0x1 => 0, // Write-only reg
            0x2 => self.spc_regs.sdsp_addr,
            0x3 => self.read_sdsp_regs(),
            0x4 => self.apuio_regs.cpuio0,
            0x5 => self.apuio_regs.cpuio1,
            0x6 => self.apuio_regs.cpuio2,
            0x7 => self.apuio_regs.cpuio3,
            0x8 => self.aram[0x00F8], // Unused
            0x9 => self.aram[0x00F9], // Unused
            0xA => 0, // Write-only reg
            0xB => 0, // Write-only reg
            0xC => 0, // Write-only reg
            0xD => self.timer0.get_counter(),
            0xE => self.timer1.get_counter(),
            0xF => self.timer2.get_counter(),
            _ => unreachable!(),
        }
    }
    
    fn write_spcio_regs(&mut self, addr: u8, value: u8) {
        match addr & 0xF {
            0x0 => {}, // Test Register (Unused)

            0x1 => { // TODO: At reset this register is initialized as if $80 was written to it. 
                self.spc_regs.ipl_read_en = get_bit_n!(value, 7);
                
                self.timer0.set_enable(get_bit_n!(value, 0));
                self.timer1.set_enable(get_bit_n!(value, 1));
                self.timer2.set_enable(get_bit_n!(value, 2));

                if get_bit_n!(value, 4) {
                    self.apuio_regs.cpuio0 = 0;
                    self.apuio_regs.cpuio1 = 0;
                }

                if get_bit_n!(value, 5) {
                    self.apuio_regs.cpuio2 = 0;
                    self.apuio_regs.cpuio3 = 0;
                }
            }
            0x2 => {
                self.spc_regs.sdsp_read_only = get_bit_n!(value, 7);
                self.spc_regs.sdsp_addr = value & 0x7F;
            }
            0x3 => {
                if !self.spc_regs.sdsp_read_only {
                    self.write_sdsp_regs(value);
                }
            }
            0x4 => { self.apuio_regs.apuio0 = value; }
            0x5 => { self.apuio_regs.apuio1 = value; }
            0x6 => { self.apuio_regs.apuio2 = value; }
            0x7 => { self.apuio_regs.apuio3 = value; }
            0xA => { self.timer0.set_target(value); }
            0xB => { self.timer1.set_target(value); }
            0xC => { self.timer2.set_target(value); }
            0xD => {} // Read-only reg
            0xE => {} // Read-only reg
            0xF => {} // Read-only reg
            _ => {}
        }
    }
    
    fn read_sdsp_regs(&mut self) -> u8 {
        let addr = self.spc_regs.sdsp_addr & 0x7F;
        
        let nibble_lo = addr & 0xF;
        let nibble_hi = (addr >> 4) & 0x7;
        
        match nibble_lo {
            0x0..=0xB => {
                let voice = &self.voice_regs[nibble_hi as usize];
                
                match nibble_lo {
                    0 => voice.lchannel_volume,
                    1 => voice.rchannel_volume,
                    2 => get_byte_n!(voice.pitch, 0),
                    3 => get_byte_n!(voice.pitch, 1),
                    4 => voice.sample_source,
                    5 => {
                        let adsr_en = if voice.adsr_en { 0x80 } else { 0 };
                        adsr_en | (voice.adsr_decay << 4) | voice.adsr_attack
                    },
                    6 => (voice.adsr_sustain_level << 5) | voice.adsr_sustain_rate,
                    7 => voice.gain_reg_raw,
                    8 => (voice.envelope >> 4) as u8,
                    9 => voice.sample_out_high,
                    0xA => voice.ram_a,
                    0xB => voice.ram_b,
                    _ => 0,
                }
            }
            
            0xC => {
                match nibble_hi {
                    0 => self.sdsp_regs.lmain_volume,
                    1 => self.sdsp_regs.rmain_volume,
                    2 => self.sdsp_regs.lecho_volume,
                    3 => self.sdsp_regs.recho_volume,
                    4 => self.sdsp_regs.key_on, // TODO: The internal KON bits are cleared 63 clocks after the bit is polled. 
                    5 => self.sdsp_regs.key_off,
                    6 => {
                        let soft_reset = if self.sdsp_regs.soft_reset { 0x80 } else { 0 };
                        let mute_all = if self.sdsp_regs.mute_all { 0x40 } else { 0 };
                        let echo_disable = if !self.sdsp_regs.echo_en { 0x20 } else { 0 };
                        
                        soft_reset | mute_all | echo_disable | self.sdsp_regs.noise_freq
                    },
                    7 => {
                        let mut endx = 0;
    
                        endx |= if self.voice_regs[7].end_of_sample_flag { 0x80 } else { 0 };
                        endx |= if self.voice_regs[6].end_of_sample_flag { 0x40 } else { 0 };
                        endx |= if self.voice_regs[5].end_of_sample_flag { 0x20 } else { 0 };
                        endx |= if self.voice_regs[4].end_of_sample_flag { 0x10 } else { 0 };
                        endx |= if self.voice_regs[3].end_of_sample_flag { 0x08 } else { 0 };
                        endx |= if self.voice_regs[2].end_of_sample_flag { 0x04 } else { 0 };
                        endx |= if self.voice_regs[1].end_of_sample_flag { 0x02 } else { 0 };
                        endx |= if self.voice_regs[0].end_of_sample_flag { 0x01 } else { 0 };
    
                        endx
                    },
                    
                    _ => 0,
                }
            }
            
            0xD => {
                match nibble_hi {
                    0 => self.sdsp_regs.echo_feedback,
                    1 => self.sdsp_regs.unused,
                    2 => {
                        let mut pitchmod_en = 0;
                        
                        pitchmod_en |= if self.voice_regs[7].pitchmod_en { 0x80 } else { 0 };
                        pitchmod_en |= if self.voice_regs[6].pitchmod_en { 0x40 } else { 0 };
                        pitchmod_en |= if self.voice_regs[5].pitchmod_en { 0x20 } else { 0 };
                        pitchmod_en |= if self.voice_regs[4].pitchmod_en { 0x10 } else { 0 };
                        pitchmod_en |= if self.voice_regs[3].pitchmod_en { 0x08 } else { 0 };
                        pitchmod_en |= if self.voice_regs[2].pitchmod_en { 0x04 } else { 0 };
                        pitchmod_en |= if self.voice_regs[1].pitchmod_en { 0x02 } else { 0 };
                        // Voice 0 does not support pitch modulation
                         
                        pitchmod_en
                    },
                    3 => {
                        let mut noise_en = 0;
                        
                        noise_en |= if self.voice_regs[7].noise_en { 0x80 } else { 0 };
                        noise_en |= if self.voice_regs[6].noise_en { 0x40 } else { 0 };
                        noise_en |= if self.voice_regs[5].noise_en { 0x20 } else { 0 };
                        noise_en |= if self.voice_regs[4].noise_en { 0x10 } else { 0 };
                        noise_en |= if self.voice_regs[3].noise_en { 0x08 } else { 0 };
                        noise_en |= if self.voice_regs[2].noise_en { 0x04 } else { 0 };
                        noise_en |= if self.voice_regs[1].noise_en { 0x02 } else { 0 };
                        noise_en |= if self.voice_regs[0].noise_en { 0x01 } else { 0 };
                        
                        noise_en
                    }
                    4 => {
                        let mut echo_en = 0;
                        
                        echo_en |= if self.voice_regs[7].echo_en { 0x80 } else { 0 };
                        echo_en |= if self.voice_regs[6].echo_en { 0x40 } else { 0 };
                        echo_en |= if self.voice_regs[5].echo_en { 0x20 } else { 0 };
                        echo_en |= if self.voice_regs[4].echo_en { 0x10 } else { 0 };
                        echo_en |= if self.voice_regs[3].echo_en { 0x08 } else { 0 };
                        echo_en |= if self.voice_regs[2].echo_en { 0x04 } else { 0 };
                        echo_en |= if self.voice_regs[1].echo_en { 0x02 } else { 0 };
                        echo_en |= if self.voice_regs[0].echo_en { 0x01 } else { 0 };
                        
                        echo_en
                    }
                    5 => self.sdsp_regs.sample_directory_page,
                    6 => self.sdsp_regs.echo_page,
                    7 => self.sdsp_regs.echo_delay_time,
                    
                    _ => 0,
                }
            }
            
            0xF => self.voice_regs[nibble_hi as usize].filter_coeff,
            
            _ => 0,
        }
    }
    
    fn write_sdsp_regs(&mut self, value: u8) {
        let addr = self.spc_regs.sdsp_addr;
        
        // Mirrors of sdsp regs are read-only
        if addr >= 0x80 {
            return;
        }
        
        let nibble_lo = addr & 0xF;
        let nibble_hi = addr >> 4;
        
        match nibble_lo {
            0x0..=0xB => {
                let voice = &mut self.voice_regs[nibble_hi as usize];
                
                match nibble_lo {
                    0 => { voice.lchannel_volume = value; },
                    1 => { voice.rchannel_volume = value; },
                    2 => { set_byte_n!(voice.pitch, value as u16, 0); },
                    3 => { set_byte_n!(voice.pitch, (value & 0x3F) as u16, 1); },
                    4 => { voice.sample_source = value; },
                    5 => {
                        voice.adsr_en = get_bit_n!(value, 7);
                        voice.adsr_decay = (value >> 4) & 7;
                        voice.adsr_attack = value & 0xF;
                    },
                    6 => {
                        voice.adsr_sustain_level = value >> 5;
                        voice.adsr_sustain_rate = value & 0x1F;
                    },
                    7 => {
                        voice.gain_reg_raw = value;
                        
                        voice.gain_fixed = value & 0x7F;
                        voice.gain_mode = match value >> 5 {
                            0..=3 => GainMode::Fixed,
                            4 => GainMode::Decrease,
                            5 => GainMode::ExpDecrease,
                            6 => GainMode::Increase,
                            7 => GainMode::BentIncrease,
                            _ => unreachable!(),
                        };
                        voice.gain_rate = value & 0x1F;
                    },
                    8 => { voice.envelope = (value << 4) as i16; },
                    9 => { voice.sample_out_high = value; },
                    0xA => { voice.ram_a = value; },
                    0xB => { voice.ram_b = value; },
                    _ => {},
                }
            }
            
            0xC => {
                match nibble_hi {
                    0 => { self.sdsp_regs.lmain_volume = value; },
                    1 => { self.sdsp_regs.rmain_volume = value; },
                    2 => { self.sdsp_regs.lecho_volume = value; },
                    3 => { self.sdsp_regs.recho_volume = value; },
                    4 => {
                        self.sdsp_regs.key_on = value;
                        
                        for voice_idx in 0..8 {
                            let voice = &mut self.voice_regs[voice_idx];

                            if get_bit_n!(value, voice_idx) && !get_bit_n!(self.sdsp_regs.key_off, voice_idx) {
                                voice.adsr_stage = ADSRStage::Attack;
                                voice.envelope = 0;

                                let start_addr_ptr = (self.sdsp_regs.sample_directory_page as u16) << 8;
                                let start_addr_ptr = start_addr_ptr + ((voice.sample_source as u16) << 2);

                                let start_addr = u16::from_le_bytes([
                                    self.aram[start_addr_ptr as usize + 0],
                                    self.aram[start_addr_ptr as usize + 1],
                                ]);

                                if voice_idx == 6 {
                                    log::debug!("Voice 6 KeyOn w/ start addr: ${:04X}", start_addr);
                                }

                                voice.brr_group_addr = start_addr;
                                voice.brr_group_step = 0;
                            }
                        }
                    }, // TODO: The internal KON bits are cleared 63 clocks after the bit is polled. 
                    5 => {
                        self.sdsp_regs.key_off = value;
                    },
                    6 => {
                        self.sdsp_regs.soft_reset = get_bit_n!(value, 7);
                        self.sdsp_regs.mute_all = get_bit_n!(value, 6);
                        self.sdsp_regs.echo_en = get_bit_n!(value, 5);
                        self.sdsp_regs.noise_freq = value & 0x1F;

                        log::debug!("Write to FLG: Soft Reset: {}, Mute All: {}, Echo: {}, Noise Freq.: {}",
                            self.sdsp_regs.soft_reset,
                            self.sdsp_regs.mute_all,
                            self.sdsp_regs.echo_en,
                            self.sdsp_regs.noise_freq,
                        );
                    },
                    _ =>  {},
                }
            }
            
            0xD => {
                match nibble_hi {
                    0 => { self.sdsp_regs.echo_feedback = value; },
                    1 => { self.sdsp_regs.unused = value; },
                    2 => {                        
                        self.voice_regs[7].pitchmod_en = get_bit_n!(value, 7);
                        self.voice_regs[6].pitchmod_en = get_bit_n!(value, 6);
                        self.voice_regs[5].pitchmod_en = get_bit_n!(value, 5);
                        self.voice_regs[4].pitchmod_en = get_bit_n!(value, 4);
                        self.voice_regs[3].pitchmod_en = get_bit_n!(value, 3);
                        self.voice_regs[2].pitchmod_en = get_bit_n!(value, 2);
                        self.voice_regs[1].pitchmod_en = get_bit_n!(value, 1);
                        // Voice 0 does not support pitch modulation
                    },
                    3 => {
                        self.voice_regs[7].noise_en = get_bit_n!(value, 7);
                        self.voice_regs[6].noise_en = get_bit_n!(value, 6);
                        self.voice_regs[5].noise_en = get_bit_n!(value, 5);
                        self.voice_regs[4].noise_en = get_bit_n!(value, 4);
                        self.voice_regs[3].noise_en = get_bit_n!(value, 3);
                        self.voice_regs[2].noise_en = get_bit_n!(value, 2);
                        self.voice_regs[1].noise_en = get_bit_n!(value, 1);
                        self.voice_regs[0].noise_en = get_bit_n!(value, 0);
                    }
                    4 => {
                        self.voice_regs[7].echo_en = get_bit_n!(value, 7);
                        self.voice_regs[6].echo_en = get_bit_n!(value, 6);
                        self.voice_regs[5].echo_en = get_bit_n!(value, 5);
                        self.voice_regs[4].echo_en = get_bit_n!(value, 4);
                        self.voice_regs[3].echo_en = get_bit_n!(value, 3);
                        self.voice_regs[2].echo_en = get_bit_n!(value, 2);
                        self.voice_regs[1].echo_en = get_bit_n!(value, 1);
                        self.voice_regs[0].echo_en = get_bit_n!(value, 0);
                    }
                    5 => { self.sdsp_regs.sample_directory_page = value; },
                    6 => { self.sdsp_regs.echo_page = value; },
                    7 => { self.sdsp_regs.echo_delay_time = value; },
                    
                    _ => {},
                }
            }
            
            0xF => { self.voice_regs[nibble_hi as usize].filter_coeff = value; },
            
            _ => {},
        }
    }
}