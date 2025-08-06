use std::cell::Cell;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::rc::Rc;

use crate::utils::{GetBits, SetCellBytes};

use crate::system::ssmp::{self, SmpData};

#[derive(Clone, Copy, PartialEq)]
enum ADSRStage {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GainMode {
    Fixed,
    Decrease,
    ExpDecrease,
    Increase,
    BentIncrease,
}

#[derive(Clone, Copy, Debug, Default)]
enum BrrFilter {
    #[default]
    Filter0,
    Filter1,
    Filter2,
    Filter3,
}

#[derive(Default)]
struct BrrSampleGroup {
    left_shift: u8,
    filter: BrrFilter,
    loop_flag: bool,
    end_flag: bool,
    samples: [u8; 16],

    sample_idx: usize,
    group_addr: u16,
}

struct BrrSample {
    sample: u8,
    is_last_sample_in_group: bool,
}

impl BrrSampleGroup {
    fn from_aram_slice(bytes: &[Cell<u8>], group_addr: u16) -> BrrSampleGroup {
        let header = bytes[0].get();
        let mut samples = [0; 16];

        for i in 0..8 {
            let data = bytes[i + 1].get();

            samples[i + 0] = data >> 4;
            samples[i + 1] = data & 0xF;
        }

        BrrSampleGroup {
            left_shift: header >> 4,
            filter: match (header >> 2) & 3 {
                0 => BrrFilter::Filter0,
                1 => BrrFilter::Filter1,
                2 => BrrFilter::Filter2,
                3 => BrrFilter::Filter3,
                _ => unreachable!(),
            },
            loop_flag: header & 2 != 0,
            end_flag: header & 1 != 0,
            samples,
            sample_idx: 0,
            group_addr,
        }
    }

    fn read_sample(&mut self) -> BrrSample {
        self.sample_idx += 1;
        
        BrrSample {
            sample: self.samples[self.sample_idx - 1],
            is_last_sample_in_group: self.sample_idx == 16,
        }
    }
}

/// Contains all registers controlling a single voice of the S-DSP
#[derive(Clone)]
pub struct VoiceRegisters {
    lchannel_volume: Cell<u8>,
    rchannel_volume: Cell<u8>,
    pitch: Cell<u16>,
    sample_source: Cell<u8>,
    adsr_enable: Cell<bool>,
    adsr_attack: Cell<u8>,
    adsr_decay: Cell<u8>,
    adsr_sustain_rate: Cell<u8>,
    adsr_sustain_level: Cell<u16>,
    gain_fixed: Cell<u16>,
    gain_rate: Cell<u8>,
    gain_mode: Cell<GainMode>,
    envelope: Cell<u16>,
    sample_out: Cell<u8>,
    raw_bytes: [Cell<u8>; 10],

    brr_start: Cell<bool>,
    adsr_stage: Cell<ADSRStage>,
}

impl VoiceRegisters {
    pub const fn new() -> VoiceRegisters {
        VoiceRegisters {
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
            brr_start: Cell::new(false),
            adsr_stage: Cell::new(ADSRStage::Attack),
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        match address {
            8 => (self.envelope.get() >> 4) as u8,
            _ => self.raw_bytes[address as usize].get(),
        }
    }

    pub fn write(&self, address: u8, data: u8) {
        self.raw_bytes[address as usize].set(data);

        match address {
            0 => self.lchannel_volume.set(data),
            1 => self.rchannel_volume.set(data),
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
                self.adsr_sustain_level.set((((data as u16) >> 5) + 1) << 8);
                self.adsr_sustain_rate.set(data & 0x1F);
            }
            7 => {
                if data.bit_en(7) {
                    self.gain_mode.set(
                        match (data >> 5) & 3 {
                            0 => GainMode::Decrease,
                            1 => GainMode::ExpDecrease,
                            2 => GainMode::Increase,
                            3 => GainMode::BentIncrease,
                            _ => unreachable!(), // &3 ensures 0..=3
                        }
                    );
                    self.gain_rate.set(data & 0x1F);
                } else {
                    self.gain_mode.set(GainMode::Fixed);
                    self.gain_fixed.set((data as u16) << 4);
                }
            }
            8 => {},
            9 => {},
            _ => {},
        }
    }
}

/// Contains all S-DSP registers accessible by the S-DSP or the SPC700.
pub struct SdspRegisters {
    lchannel_volume: Cell<u8>,
    rchannel_volume: Cell<u8>,
    lecho_volume: Cell<i8>,
    recho_volume: Cell<i8>,
    key_on: Cell<u8>,
    key_off: Cell<u8>,
    soft_reset: Cell<bool>,
    mute_all: Cell<bool>,
    echo_disable: Cell<bool>,
    noise_period: Cell<u8>,
    
    echo_feedback: Cell<i8>,
    unused: Cell<u8>,
    voice_pitchmod_enable: Cell<u8>,
    voice_noise_enable: Cell<u8>,
    voice_echo_enable: Cell<u8>,
    sample_page: Cell<u8>,
    echo_page: Cell<u8>,
    echo_delay_time: Cell<u8>,
    filter_coeff: [Cell<u8>; 8],

    endx: Cell<u8>,

    voice_regs: [VoiceRegisters; 8],
}

impl SdspRegisters {
    pub const fn new() -> SdspRegisters {
        SdspRegisters {
            lchannel_volume: Cell::new(0),
            rchannel_volume: Cell::new(0),
            lecho_volume: Cell::new(0),
            recho_volume: Cell::new(0),
            key_on: Cell::new(0),
            key_off: Cell::new(0),
            soft_reset: Cell::new(false),
            mute_all: Cell::new(false),
            echo_disable: Cell::new(false),
            noise_period: Cell::new(0),
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
            voice_regs: [
                VoiceRegisters::new(), VoiceRegisters::new(),
                VoiceRegisters::new(), VoiceRegisters::new(),
                VoiceRegisters::new(), VoiceRegisters::new(),
                VoiceRegisters::new(), VoiceRegisters::new(),
            ],
            endx: Cell::new(0),
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        match (address >> 4, address & 0xF) {
            (voice @ 0..=7, addr @ 0..=9) => {
                self.voice_regs[voice as usize].read(addr)
            }
            (0, 0xC) => self.lchannel_volume.get(),
            (1, 0xC) => self.rchannel_volume.get(),
            (2, 0xC) => self.lecho_volume.get() as u8,
            (3, 0xC) => self.recho_volume.get() as u8,
            (4, 0xC) => self.key_on.get(),
            (5, 0xC) => self.key_off.get(),
            (6, 0xC) => {
                let r = self.soft_reset.get() as u8;
                let m = self.mute_all.get() as u8;
                let e = self.echo_disable.get() as u8;
                (r << 7) | (m << 6) | (e << 5) | self.noise_period.get()
            }
            (7, 0xC) => self.endx.get(),
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
            (0, 0xC) => self.lchannel_volume.set(data),
            (1, 0xC) => self.rchannel_volume.set(data),
            (2, 0xC) => self.lecho_volume.set(data as i8),
            (3, 0xC) => self.recho_volume.set(data as i8),
            (4, 0xC) => {
                self.key_on.set(data);

                self.endx.set(self.endx.get() & !data);

                for voice in 0..8 {
                    if data.bit_en(voice as u8) {
                        self.voice_regs[voice].envelope.set(0);
                        self.voice_regs[voice].adsr_stage.set(ADSRStage::Attack);
                        self.voice_regs[voice].brr_start.set(true);
                    }
                }
            }
            (5, 0xC) => {
                self.key_off.set(data);

                for voice in 0..8 {
                    if data.bit_en(voice as u8) {
                        self.voice_regs[voice].adsr_stage.set(ADSRStage::Release);
                    }
                }
            }
            (6, 0xC) => {
                self.soft_reset.set(data.bit_en(7));
                self.mute_all.set(data.bit_en(6));
                self.echo_disable.set(data.bit_en(5));
                self.noise_period.set(data & 0x1F);
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

/// Implementation of the S-DSP. Responsible for generating sound samples to
/// add to the audio buffer.
pub struct SuperDSP {
    smp_data: Rc<SmpData>,

    envelope_cnt: usize,

    noise_output: u16,

    writer: Option<hound::WavWriter<BufWriter<File>>>,

    brr_sample_groups: [BrrSampleGroup; 8],
    surrounding_brr_samples: [[u16; 4]; 8],
    voice_intermediate_time_step: [f64; 8],
}

impl SuperDSP {
    const GAUSS_LOOKUP: [u16; 512] = [
        0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,0x000,
        0x001,0x001,0x001,0x001,0x001,0x001,0x001,0x001,0x001,0x001,0x001,0x002,0x002,0x002,0x002,0x002,
        0x002,0x002,0x003,0x003,0x003,0x003,0x003,0x004,0x004,0x004,0x004,0x004,0x005,0x005,0x005,0x005,
        0x006,0x006,0x006,0x006,0x007,0x007,0x007,0x008,0x008,0x008,0x009,0x009,0x009,0x00A,0x00A,0x00A,
        0x00B,0x00B,0x00B,0x00C,0x00C,0x00D,0x00D,0x00E,0x00E,0x00F,0x00F,0x00F,0x010,0x010,0x011,0x011,
        0x012,0x013,0x013,0x014,0x014,0x015,0x015,0x016,0x017,0x017,0x018,0x018,0x019,0x01A,0x01B,0x01B,
        0x01C,0x01D,0x01D,0x01E,0x01F,0x020,0x020,0x021,0x022,0x023,0x024,0x024,0x025,0x026,0x027,0x028,
        0x029,0x02A,0x02B,0x02C,0x02D,0x02E,0x02F,0x030,0x031,0x032,0x033,0x034,0x035,0x036,0x037,0x038,
        0x03A,0x03B,0x03C,0x03D,0x03E,0x040,0x041,0x042,0x043,0x045,0x046,0x047,0x049,0x04A,0x04C,0x04D,
        0x04E,0x050,0x051,0x053,0x054,0x056,0x057,0x059,0x05A,0x05C,0x05E,0x05F,0x061,0x063,0x064,0x066,
        0x068,0x06A,0x06B,0x06D,0x06F,0x071,0x073,0x075,0x076,0x078,0x07A,0x07C,0x07E,0x080,0x082,0x084,
        0x086,0x089,0x08B,0x08D,0x08F,0x091,0x093,0x096,0x098,0x09A,0x09C,0x09F,0x0A1,0x0A3,0x0A6,0x0A8,
        0x0AB,0x0AD,0x0AF,0x0B2,0x0B4,0x0B7,0x0BA,0x0BC,0x0BF,0x0C1,0x0C4,0x0C7,0x0C9,0x0CC,0x0CF,0x0D2,
        0x0D4,0x0D7,0x0DA,0x0DD,0x0E0,0x0E3,0x0E6,0x0E9,0x0EC,0x0EF,0x0F2,0x0F5,0x0F8,0x0FB,0x0FE,0x101,
        0x104,0x107,0x10B,0x10E,0x111,0x114,0x118,0x11B,0x11E,0x122,0x125,0x129,0x12C,0x130,0x133,0x137,
        0x13A,0x13E,0x141,0x145,0x148,0x14C,0x150,0x153,0x157,0x15B,0x15F,0x162,0x166,0x16A,0x16E,0x172,
        0x176,0x17A,0x17D,0x181,0x185,0x189,0x18D,0x191,0x195,0x19A,0x19E,0x1A2,0x1A6,0x1AA,0x1AE,0x1B2,
        0x1B7,0x1BB,0x1BF,0x1C3,0x1C8,0x1CC,0x1D0,0x1D5,0x1D9,0x1DD,0x1E2,0x1E6,0x1EB,0x1EF,0x1F3,0x1F8,
        0x1FC,0x201,0x205,0x20A,0x20F,0x213,0x218,0x21C,0x221,0x226,0x22A,0x22F,0x233,0x238,0x23D,0x241,
        0x246,0x24B,0x250,0x254,0x259,0x25E,0x263,0x267,0x26C,0x271,0x276,0x27B,0x280,0x284,0x289,0x28E,
        0x293,0x298,0x29D,0x2A2,0x2A6,0x2AB,0x2B0,0x2B5,0x2BA,0x2BF,0x2C4,0x2C9,0x2CE,0x2D3,0x2D8,0x2DC,
        0x2E1,0x2E6,0x2EB,0x2F0,0x2F5,0x2FA,0x2FF,0x304,0x309,0x30E,0x313,0x318,0x31D,0x322,0x326,0x32B,
        0x330,0x335,0x33A,0x33F,0x344,0x349,0x34E,0x353,0x357,0x35C,0x361,0x366,0x36B,0x370,0x374,0x379,
        0x37E,0x383,0x388,0x38C,0x391,0x396,0x39B,0x39F,0x3A4,0x3A9,0x3AD,0x3B2,0x3B7,0x3BB,0x3C0,0x3C5,
        0x3C9,0x3CE,0x3D2,0x3D7,0x3DC,0x3E0,0x3E5,0x3E9,0x3ED,0x3F2,0x3F6,0x3FB,0x3FF,0x403,0x408,0x40C,
        0x410,0x415,0x419,0x41D,0x421,0x425,0x42A,0x42E,0x432,0x436,0x43A,0x43E,0x442,0x446,0x44A,0x44E,
        0x452,0x455,0x459,0x45D,0x461,0x465,0x468,0x46C,0x470,0x473,0x477,0x47A,0x47E,0x481,0x485,0x488,
        0x48C,0x48F,0x492,0x496,0x499,0x49C,0x49F,0x4A2,0x4A6,0x4A9,0x4AC,0x4AF,0x4B2,0x4B5,0x4B7,0x4BA,
        0x4BD,0x4C0,0x4C3,0x4C5,0x4C8,0x4CB,0x4CD,0x4D0,0x4D2,0x4D5,0x4D7,0x4D9,0x4DC,0x4DE,0x4E0,0x4E3,
        0x4E5,0x4E7,0x4E9,0x4EB,0x4ED,0x4EF,0x4F1,0x4F3,0x4F5,0x4F6,0x4F8,0x4FA,0x4FB,0x4FD,0x4FF,0x500,
        0x502,0x503,0x504,0x506,0x507,0x508,0x50A,0x50B,0x50C,0x50D,0x50E,0x50F,0x510,0x511,0x511,0x512,
        0x513,0x514,0x514,0x515,0x516,0x516,0x517,0x517,0x517,0x518,0x518,0x518,0x518,0x518,0x519,0x519,
    ];
    
    const PERIOD_LOOKUP: [usize; 32] = [
        0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 
        128, 96, 80, 64, 48, 40, 32, 24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
    ];

    const PERIOD_OFFSET_LOOKUP: [usize; 3] = [536, 0, 1040];

    const RELEASE_PERIOD_IDX: usize = 31;

    pub fn new(smp_data: Rc<SmpData>) -> SuperDSP {
        let wav_spec = hound::WavSpec {
            channels: 2,
            sample_rate: 32000,
            sample_format: hound::SampleFormat::Int,
            bits_per_sample: 16,
        };
        let writer = hound::WavWriter::create("sound.wav", wav_spec).unwrap();

        SuperDSP {
            smp_data,

            envelope_cnt: 0,

            noise_output: 0x4000,

            brr_sample_groups: [
                BrrSampleGroup::default(), BrrSampleGroup::default(),
                BrrSampleGroup::default(), BrrSampleGroup::default(),
                BrrSampleGroup::default(), BrrSampleGroup::default(),
                BrrSampleGroup::default(), BrrSampleGroup::default(),
            ],
            surrounding_brr_samples: [[0; 4]; 8],
            voice_intermediate_time_step: [0.0; 8],

            writer: Some(writer),
       }
    }

    pub fn clock_envelopes(&mut self) {
        self.clock_noise_generator();

        for voice in 0..8 {
            let voice_regs = &self.smp_data.sdsp_regs.voice_regs[voice];

            // Release decreases envelope by 8 regardles of VxADSR and VxGAIN settings
            if voice_regs.adsr_enable.get() || voice_regs.adsr_stage.get() == ADSRStage::Release {
                self.clock_adsr_envelope(voice);
            } else {
                self.clock_gain_envelope(voice);
            }
        }

        self.envelope_cnt = self.envelope_cnt.checked_sub(1).unwrap_or(0x77FF);
    }

    fn should_do_envelope_op(&self, period_idx: usize) -> bool {
        let period = SuperDSP::PERIOD_LOOKUP[period_idx];
        let period_offset = SuperDSP::PERIOD_OFFSET_LOOKUP[period_idx % 3];

        if period == 0 {
            false
        } else {
            (self.envelope_cnt + period_offset) % period == 0
        }
    }

    fn clock_noise_generator(&mut self) {
        let noise_period_idx = self.smp_data.sdsp_regs.noise_period.get() as usize;

        if noise_period_idx == 0 {
            return;
        }

        if self.should_do_envelope_op(noise_period_idx) {
            let b0 = (self.noise_output >> 0) & 1;
            let b1 = (self.noise_output >> 1) & 1;
    
            self.noise_output |= (b0 ^ b1) << 15;
            self.noise_output >>= 1;
        }
    }

    fn clock_adsr_envelope(&mut self, voice: usize) {
        let voice_regs = &self.smp_data.sdsp_regs.voice_regs[voice];

        match voice_regs.adsr_stage.get() {
            ADSRStage::Attack => {
                let adsr_attack = voice_regs.adsr_attack.get() as usize;
                let attack_period_idx = 2 * adsr_attack + 1;

                if self.should_do_envelope_op(attack_period_idx) {
                    let attack_rate = if attack_period_idx == 15 { 1024 } else { 32 };

                    let envelope_val = (voice_regs.envelope.get() + attack_rate).min(0x7FF);

                    if envelope_val >= 0x7E0 {
                        voice_regs.adsr_stage.set(ADSRStage::Decay);
                    }
                    
                    voice_regs.envelope.set(envelope_val + attack_rate);
                }
            }
            ADSRStage::Decay => {
                let adsr_decay = voice_regs.adsr_decay.get() as usize;
                let decay_period_idx = 2 * adsr_decay + 16;

                if self.should_do_envelope_op(decay_period_idx) {
                    let mut envelope_val = voice_regs.envelope.get();

                    // Don't need underflow check bc we start w/ envelope > 0,
                    // and adsr sustain level is >= 0, so if envelope == 0 after
                    // this calculation, we are guarenteed to transition to
                    // the sustain stage.
                    envelope_val = (envelope_val - 1) - (envelope_val >> 8);

                    if envelope_val <= voice_regs.adsr_sustain_level.get() as u16 {
                        voice_regs.adsr_stage.set(ADSRStage::Sustain);
                    }
                    
                    voice_regs.envelope.set(envelope_val);
                }
            }
            ADSRStage::Sustain => {
                let sustain_period_idx = voice_regs.adsr_sustain_rate.get() as usize;

                if self.should_do_envelope_op(sustain_period_idx) {
                    let mut envelope_val = voice_regs.envelope.get();

                    // This calculation underflows the envelope if it is 0, but
                    // it gets clamped back to 0 afterwards, resulting in no
                    // change.
                    if envelope_val > 0 {
                        envelope_val = (envelope_val - 1) - (envelope_val >> 8);
    
                        voice_regs.envelope.set(envelope_val);
                    }
                }
            }
            ADSRStage::Release => {
                if self.should_do_envelope_op(SuperDSP::RELEASE_PERIOD_IDX) {
                    let envelope_val = voice_regs.envelope.get()
                        .checked_sub(8)
                        .unwrap_or(0);
    
                    voice_regs.envelope.set(envelope_val);
                }
            }
        }
    }

    fn clock_gain_envelope(&mut self, voice: usize) {
        let voice_regs = &self.smp_data.sdsp_regs.voice_regs[voice];

        if voice_regs.gain_mode.get() == GainMode::Fixed {
            voice_regs.envelope.set(voice_regs.gain_fixed.get());
            return;
        }

        let gain_period_idx = voice_regs.gain_rate.get() as usize;

        if !self.should_do_envelope_op(gain_period_idx) {
            return;
        }

        let new_envelope = match voice_regs.gain_mode.get() {
            GainMode::Decrease => {
                voice_regs.envelope.get() - 32
            }
            GainMode::ExpDecrease => {
                let envelope_val = voice_regs.envelope.get();
                (envelope_val - 1) - (envelope_val >> 8)
            }
            GainMode::Increase => {
                voice_regs.envelope.get() + 32
            }
            GainMode::BentIncrease => {
                if voice_regs.envelope.get() < 0x600 {
                    voice_regs.envelope.get() + 32
                } else {
                    voice_regs.envelope.get() + 8
                }
            }
            GainMode::Fixed => unreachable!(), // Handled above match
        };

        let clipped_envelope = new_envelope & 0x7FF;

        voice_regs.envelope.set(clipped_envelope);
    }

    /// Reads the next BRR sample into the surrounding_brr_samples array, shifting
    /// the previously read samples into the preceding position, and the 0th sample
    /// out of the array.
    fn read_voice_brr_sample(&mut self, voice: usize) {
        let brr_sample = self.brr_sample_groups[voice].read_sample();

        let filtered_sample = self.filter_brr_sample(brr_sample.sample, voice);

        // if voice == 0 {
        //     if let Some(mut writer) = self.writer.take() {
        //         writer.write_sample(sign_extend::<15>(filtered_sample as i32) as i16).unwrap();
        //         writer.write_sample(sign_extend::<15>(filtered_sample as i32) as i16).unwrap();
        //         self.writer = Some(writer);
        //     }
        // }

        self.surrounding_brr_samples[voice][0] = self.surrounding_brr_samples[voice][1];
        self.surrounding_brr_samples[voice][1] = self.surrounding_brr_samples[voice][2];
        self.surrounding_brr_samples[voice][2] = self.surrounding_brr_samples[voice][3];
        self.surrounding_brr_samples[voice][3] = filtered_sample;

        let voice_regs = &self.smp_data.sdsp_regs.voice_regs[voice];
        let brr_group = &mut self.brr_sample_groups[voice];

        if brr_sample.is_last_sample_in_group {
            if brr_group.end_flag {
                if !brr_group.loop_flag {
                    voice_regs.adsr_stage.set(ADSRStage::Release);
                    voice_regs.envelope.set(0);   
                }

                let directory_page = (self.smp_data.sdsp_regs.sample_page.get() as usize) << 8;
                let directory_addr = directory_page + ((voice_regs.sample_source.get() as usize) << 2);

                let loop_addr = u16::from_le_bytes([
                    self.smp_data.aram[directory_addr + 2].get(),
                    self.smp_data.aram[directory_addr + 3].get(),
                ]);

                self.brr_sample_groups[voice] = self.read_next_voice_brr_group(voice, loop_addr);
            } else {
                let group_addr = brr_group.group_addr + 9;

                self.brr_sample_groups[voice] = self.read_next_voice_brr_group(voice, group_addr);
            }
        }
    }

    fn read_next_voice_brr_group(&mut self, voice: usize, group_addr: u16) -> BrrSampleGroup {
        let brr_group = BrrSampleGroup::from_aram_slice(
            &self.smp_data.aram[(group_addr as usize)..(group_addr + 9) as usize],
            group_addr,
        );

        let endx_bit = if brr_group.end_flag { 1 << voice } else { 0 };

        self.smp_data.sdsp_regs.endx.set(
            self.smp_data.sdsp_regs.endx.get() | endx_bit
        );

        brr_group
    }

    fn filter_brr_sample(&mut self, sample: u8, voice: usize) -> u16 {
        let brr_group = &mut self.brr_sample_groups[voice];

        let signed_sample = sign_extend::<4>(sample as i32);

        let shifted_sample = if brr_group.left_shift > 12 {
            (signed_sample >> 3) << 11
        } else {
            (signed_sample << brr_group.left_shift) >> 1
        };

        let (a, b) = match brr_group.filter {
            BrrFilter::Filter0 => (0., 0.),
            BrrFilter::Filter1 => (0.9375, 0.),
            BrrFilter::Filter2 => (1.90625, -0.9375),
            BrrFilter::Filter3 => (1.796875, -0.8125)
        };

        let s3 = sign_extend::<15>(self.surrounding_brr_samples[voice][3] as i32);
        let s2 = sign_extend::<15>(self.surrounding_brr_samples[voice][2] as i32);

        let filtered_sample = (
            (shifted_sample as f64)
            + a * (s3 as f64)
            + b * (s2 as f64)
        ) as i32;

        let clamped_sample = if filtered_sample < -0x8000 {
            0x8000u16
        } else if filtered_sample > 0x7FFF {
            0x7FFFu16
        } else {
            filtered_sample as u16
        };

        let clipped_sample = clamped_sample & 0x7FFF;

        clipped_sample
    }

    fn start_voice_brr(&mut self, voice: usize) {
        let voice_regs = &self.smp_data.sdsp_regs.voice_regs[voice];

        let directory_page = (self.smp_data.sdsp_regs.sample_page.get() as usize) << 8;
        let directory_addr = directory_page + ((voice_regs.sample_source.get() as usize) << 2);

        let group_addr = u16::from_le_bytes([
            self.smp_data.aram[directory_addr + 0].get(),
            self.smp_data.aram[directory_addr + 1].get(),
        ]);

        self.brr_sample_groups[voice] = self.read_next_voice_brr_group(voice, group_addr);

        // read the first 4 BRR samples into the surrounding brr samples array
        for _ in 0..4 {
            self.read_voice_brr_sample(voice);
        }

        self.voice_intermediate_time_step[voice] = 0.0;
    }

    pub fn finish(&mut self) {
        self.writer.take().unwrap().finalize().unwrap();
    }

    fn generate_voice_brr_sample(&mut self, voice: usize) -> u16 {
        // Multiplied to a 14 bit (2.12) fixed-width number to convert to a float
        const PITCH_MULT: f64 = 1.0 / 0x1000 as f64;
        
        let voice_data = &self.smp_data.sdsp_regs.voice_regs[voice];

        // If we haven't yet read the first BRR sample group, do so
        if voice_data.brr_start.get() {
            voice_data.brr_start.set(false);

            self.start_voice_brr(voice);
        }
        
        let voice_data = &self.smp_data.sdsp_regs.voice_regs[voice];
        
        let pitch_factor = PITCH_MULT * voice_data.pitch.get() as f64;
        let brr_sample_advance = pitch_factor + self.voice_intermediate_time_step[voice];
        let steps = brr_sample_advance as usize;
        self.voice_intermediate_time_step[voice] = brr_sample_advance - steps as f64;

        for _ in 0..steps {
            self.read_voice_brr_sample(voice);
        }

        let gauss_lookup_idx = (self.voice_intermediate_time_step[voice] * 256.0) as usize;
        
        let s0 = sign_extend::<15>(self.surrounding_brr_samples[voice][0] as i32);
        let s1 = sign_extend::<15>(self.surrounding_brr_samples[voice][1] as i32);
        let s2 = sign_extend::<15>(self.surrounding_brr_samples[voice][2] as i32);
        let s3 = sign_extend::<15>(self.surrounding_brr_samples[voice][3] as i32);
    
        let sample0 = (s0 * SuperDSP::GAUSS_LOOKUP[255 - gauss_lookup_idx] as i32) >> 10;
        let sample1 = (s1 * SuperDSP::GAUSS_LOOKUP[511 - gauss_lookup_idx] as i32) >> 10;
        let sample2 = (s2 * SuperDSP::GAUSS_LOOKUP[256 + gauss_lookup_idx] as i32) >> 10;
        let sample3 = (s3 * SuperDSP::GAUSS_LOOKUP[  0 + gauss_lookup_idx] as i32) >> 10;

        let final_sample = ((sample0 + sample1 + sample2 + sample3) as u16) >> 1;

        final_sample
    }

    fn generate_voice_samples(&mut self, voice: usize) -> (u16, u16) {
        let noise_en = self.smp_data.sdsp_regs.voice_noise_enable.get().bit_en(voice as u8);

        let brr_sample = self.generate_voice_brr_sample(voice);

        let voice_out = if noise_en {
            sign_extend::<15>(self.noise_output as i32)
        } else {
            sign_extend::<15>(brr_sample as i32)
        };

        let voice_regs = &self.smp_data.sdsp_regs.voice_regs[voice];

        let voice_envelope = voice_regs.envelope.get() as i32;

        let voice_out = (voice_out * voice_envelope) >> 11;

        voice_regs.sample_out.set(((voice_out >> 8) as u8) & 0x7F);

        let lvolume = sign_extend::<8>(voice_regs.lchannel_volume.get() as i32);
        let rvolume = sign_extend::<8>(voice_regs.rchannel_volume.get() as i32);

        let lsample = (((voice_out * lvolume) >> 7) as u16) & 0x7FFF;
        let rsample = (((voice_out * rvolume) >> 7) as u16) & 0x7FFF;

        (lsample, rsample)
    }

    pub fn generate_sample(&mut self, audio_buffer: &mut Vec<i16>) {
        let mut lsum = 0;
        let mut rsum = 0;
        
        for voice in 0..8 {
            let (lsample, rsample) =  self.generate_voice_samples(voice);

            lsum = (lsum + lsample) & 0x7FFF;
            rsum = (rsum + rsample) & 0x7FFF;
        }

        // if let Some(mut writer) = self.writer.take() {
        //     writer.write_sample(sign_extend::<15>(lsum as i32) as i16).unwrap();
        //     writer.write_sample(sign_extend::<15>(rsum as i32) as i16).unwrap();
        //     self.writer = Some(writer);
        // }

        let lvolume = sign_extend::<8>(self.smp_data.sdsp_regs.lchannel_volume.get() as i32);
        let rvolume = sign_extend::<8>(self.smp_data.sdsp_regs.rchannel_volume.get() as i32);

        let lsample = sign_extend::<15>(lsum as i32) * lvolume;
        let rsample = sign_extend::<15>(rsum as i32) * rvolume;
        
        let lsample = (lsample >> 7) as i16;
        let rsample = (rsample >> 7) as i16;

        audio_buffer.push(lsample);
        audio_buffer.push(rsample);
    }
}

pub const fn sign_extend<const WIDTH: usize>(n: i32) -> i32 {
    let shift = 32 - WIDTH;
    (n << shift) >> shift
}

