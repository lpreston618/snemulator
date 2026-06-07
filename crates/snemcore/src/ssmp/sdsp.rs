use crate::{get_bit_n, get_byte_n};

use bus::SdspBus;

pub mod bus;
pub mod regs;
pub mod voices;

const fn sign_extend<const WIDTH: usize>(value: i32) -> i32 {
    let shift = 32 - WIDTH;
    (value << shift) >> shift
}

#[derive(Clone, Copy, Debug)]
pub enum ADSRStage {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy, Debug)]
pub enum GainMode {
    Fixed,
    Decrease,
    ExpDecrease,
    Increase,
    BentIncrease,
}

#[derive(Clone, Copy, Debug)]
pub enum BrrFilter {
    Filter0,
    Filter1,
    Filter2,
    Filter3,
}

pub struct SuperDSP {
    envelope_counter: usize,
    noise_output: u16,
    echo_ptr: usize,
}

impl SuperDSP {
    const GAUSS_LOOKUP_TABLE: [i16; 512] = [
        0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000,
        0x000, 0x000, 0x000, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001,
        0x001, 0x002, 0x002, 0x002, 0x002, 0x002, 0x002, 0x002, 0x003, 0x003, 0x003, 0x003, 0x003,
        0x004, 0x004, 0x004, 0x004, 0x004, 0x005, 0x005, 0x005, 0x005, 0x006, 0x006, 0x006, 0x006,
        0x007, 0x007, 0x007, 0x008, 0x008, 0x008, 0x009, 0x009, 0x009, 0x00A, 0x00A, 0x00A, 0x00B,
        0x00B, 0x00B, 0x00C, 0x00C, 0x00D, 0x00D, 0x00E, 0x00E, 0x00F, 0x00F, 0x00F, 0x010, 0x010,
        0x011, 0x011, 0x012, 0x013, 0x013, 0x014, 0x014, 0x015, 0x015, 0x016, 0x017, 0x017, 0x018,
        0x018, 0x019, 0x01A, 0x01B, 0x01B, 0x01C, 0x01D, 0x01D, 0x01E, 0x01F, 0x020, 0x020, 0x021,
        0x022, 0x023, 0x024, 0x024, 0x025, 0x026, 0x027, 0x028, 0x029, 0x02A, 0x02B, 0x02C, 0x02D,
        0x02E, 0x02F, 0x030, 0x031, 0x032, 0x033, 0x034, 0x035, 0x036, 0x037, 0x038, 0x03A, 0x03B,
        0x03C, 0x03D, 0x03E, 0x040, 0x041, 0x042, 0x043, 0x045, 0x046, 0x047, 0x049, 0x04A, 0x04C,
        0x04D, 0x04E, 0x050, 0x051, 0x053, 0x054, 0x056, 0x057, 0x059, 0x05A, 0x05C, 0x05E, 0x05F,
        0x061, 0x063, 0x064, 0x066, 0x068, 0x06A, 0x06B, 0x06D, 0x06F, 0x071, 0x073, 0x075, 0x076,
        0x078, 0x07A, 0x07C, 0x07E, 0x080, 0x082, 0x084, 0x086, 0x089, 0x08B, 0x08D, 0x08F, 0x091,
        0x093, 0x096, 0x098, 0x09A, 0x09C, 0x09F, 0x0A1, 0x0A3, 0x0A6, 0x0A8, 0x0AB, 0x0AD, 0x0AF,
        0x0B2, 0x0B4, 0x0B7, 0x0BA, 0x0BC, 0x0BF, 0x0C1, 0x0C4, 0x0C7, 0x0C9, 0x0CC, 0x0CF, 0x0D2,
        0x0D4, 0x0D7, 0x0DA, 0x0DD, 0x0E0, 0x0E3, 0x0E6, 0x0E9, 0x0EC, 0x0EF, 0x0F2, 0x0F5, 0x0F8,
        0x0FB, 0x0FE, 0x101, 0x104, 0x107, 0x10B, 0x10E, 0x111, 0x114, 0x118, 0x11B, 0x11E, 0x122,
        0x125, 0x129, 0x12C, 0x130, 0x133, 0x137, 0x13A, 0x13E, 0x141, 0x145, 0x148, 0x14C, 0x150,
        0x153, 0x157, 0x15B, 0x15F, 0x162, 0x166, 0x16A, 0x16E, 0x172, 0x176, 0x17A, 0x17D, 0x181,
        0x185, 0x189, 0x18D, 0x191, 0x195, 0x19A, 0x19E, 0x1A2, 0x1A6, 0x1AA, 0x1AE, 0x1B2, 0x1B7,
        0x1BB, 0x1BF, 0x1C3, 0x1C8, 0x1CC, 0x1D0, 0x1D5, 0x1D9, 0x1DD, 0x1E2, 0x1E6, 0x1EB, 0x1EF,
        0x1F3, 0x1F8, 0x1FC, 0x201, 0x205, 0x20A, 0x20F, 0x213, 0x218, 0x21C, 0x221, 0x226, 0x22A,
        0x22F, 0x233, 0x238, 0x23D, 0x241, 0x246, 0x24B, 0x250, 0x254, 0x259, 0x25E, 0x263, 0x267,
        0x26C, 0x271, 0x276, 0x27B, 0x280, 0x284, 0x289, 0x28E, 0x293, 0x298, 0x29D, 0x2A2, 0x2A6,
        0x2AB, 0x2B0, 0x2B5, 0x2BA, 0x2BF, 0x2C4, 0x2C9, 0x2CE, 0x2D3, 0x2D8, 0x2DC, 0x2E1, 0x2E6,
        0x2EB, 0x2F0, 0x2F5, 0x2FA, 0x2FF, 0x304, 0x309, 0x30E, 0x313, 0x318, 0x31D, 0x322, 0x326,
        0x32B, 0x330, 0x335, 0x33A, 0x33F, 0x344, 0x349, 0x34E, 0x353, 0x357, 0x35C, 0x361, 0x366,
        0x36B, 0x370, 0x374, 0x379, 0x37E, 0x383, 0x388, 0x38C, 0x391, 0x396, 0x39B, 0x39F, 0x3A4,
        0x3A9, 0x3AD, 0x3B2, 0x3B7, 0x3BB, 0x3C0, 0x3C5, 0x3C9, 0x3CE, 0x3D2, 0x3D7, 0x3DC, 0x3E0,
        0x3E5, 0x3E9, 0x3ED, 0x3F2, 0x3F6, 0x3FB, 0x3FF, 0x403, 0x408, 0x40C, 0x410, 0x415, 0x419,
        0x41D, 0x421, 0x425, 0x42A, 0x42E, 0x432, 0x436, 0x43A, 0x43E, 0x442, 0x446, 0x44A, 0x44E,
        0x452, 0x455, 0x459, 0x45D, 0x461, 0x465, 0x468, 0x46C, 0x470, 0x473, 0x477, 0x47A, 0x47E,
        0x481, 0x485, 0x488, 0x48C, 0x48F, 0x492, 0x496, 0x499, 0x49C, 0x49F, 0x4A2, 0x4A6, 0x4A9,
        0x4AC, 0x4AF, 0x4B2, 0x4B5, 0x4B7, 0x4BA, 0x4BD, 0x4C0, 0x4C3, 0x4C5, 0x4C8, 0x4CB, 0x4CD,
        0x4D0, 0x4D2, 0x4D5, 0x4D7, 0x4D9, 0x4DC, 0x4DE, 0x4E0, 0x4E3, 0x4E5, 0x4E7, 0x4E9, 0x4EB,
        0x4ED, 0x4EF, 0x4F1, 0x4F3, 0x4F5, 0x4F6, 0x4F8, 0x4FA, 0x4FB, 0x4FD, 0x4FF, 0x500, 0x502,
        0x503, 0x504, 0x506, 0x507, 0x508, 0x50A, 0x50B, 0x50C, 0x50D, 0x50E, 0x50F, 0x510, 0x511,
        0x511, 0x512, 0x513, 0x514, 0x514, 0x515, 0x516, 0x516, 0x517, 0x517, 0x517, 0x518, 0x518,
        0x518, 0x518, 0x518, 0x519, 0x519,
    ];

    const PERIOD_LOOKUP: [usize; 32] = [
        0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80, 64, 48, 40,
        32, 24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
    ];

    const PERIOD_OFFSET_LOOKUP: [usize; 3] = [536, 0, 1040];

    pub fn new() -> Self {
        Self {
            envelope_counter: 0,
            noise_output: 0,
            echo_ptr: 0,
        }
    }

    pub fn power_on(&mut self) {
        self.envelope_counter = 0;
        self.noise_output = 0x4000;
    }

    pub fn reset(&mut self) {
        self.envelope_counter = 0;
        self.noise_output = 0x4000;
    }

    fn should_do_envelope_op(&self, period_idx: usize) -> bool {
        let period = SuperDSP::PERIOD_LOOKUP[period_idx];
        let period_offset = SuperDSP::PERIOD_OFFSET_LOOKUP[period_idx % 3];

        if period == 0 {
            false
        } else {
            (self.envelope_counter + period_offset) % period == 0
        }
    }

    pub fn clock_envelopes(&mut self, bus: &mut SdspBus) {
        self.clock_noise_generator(bus);

        for voice_idx in 0..8 {
            let voice = &mut bus.voice_regs[voice_idx];

            if get_bit_n!(bus.sdsp_regs.key_off, voice_idx) {
                voice.adsr_stage = ADSRStage::Release;
            }

            // Release decreases envelope by 8 regardles of VxADSR and VxGAIN settings
            if voice.adsr_en || matches!(voice.adsr_stage, ADSRStage::Release) {
                self.clock_adsr_envelope(bus, voice_idx);
            } else {
                self.clock_gain_envelope(bus, voice_idx);
            }
        }

        self.envelope_counter = self.envelope_counter.checked_sub(1).unwrap_or(0x77FF);
    }

    fn clock_noise_generator(&mut self, bus: &mut SdspBus) {
        let noise_period_idx = bus.sdsp_regs.noise_freq as usize;

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

    fn clock_adsr_envelope(&mut self, bus: &mut SdspBus, voice_idx: usize) {
        let voice = &mut bus.voice_regs[voice_idx];

        match voice.adsr_stage {
            ADSRStage::Attack => {
                let adsr_attack = voice.adsr_attack as usize;
                let attack_period_idx = 2 * adsr_attack + 1;

                if self.should_do_envelope_op(attack_period_idx) {
                    let attack_rate = if adsr_attack == 15 { 1024 } else { 32 };

                    voice.envelope = (voice.envelope + attack_rate).min(0x7FF);

                    if voice.envelope >= 0x7E0 {
                        voice.adsr_stage = ADSRStage::Decay;
                    }

                    // TODO: Check Attack->Decay switch. Nocash says clip when > 0x7FF
                    //       and switch when >= 0x7e0, notes say switch when > 0x7FF
                    //       w/ no clipping/clamping.
                }
            }
            ADSRStage::Decay => {
                let adsr_decay = voice.adsr_decay as usize;
                let decay_period_idx = 2 * adsr_decay + 16;

                if self.should_do_envelope_op(decay_period_idx) {
                    // Don't need underflow check bc we start w/ envelope > 0,
                    // and adsr sustain level is >= 0, so if envelope == 0 after
                    // this calculation, we are guarenteed to transition to
                    // the sustain stage.
                    voice.envelope -= 1;
                    voice.envelope -= voice.envelope >> 8;

                    let sustain_threshold = ((voice.adsr_sustain_level as i16) + 1) << 8;

                    if voice.envelope <= sustain_threshold {
                        voice.adsr_stage = ADSRStage::Sustain;
                    }

                    // if (voice.envelope >> 8) == voice.adsr_sustain_level as i16 {
                    //     voice.adsr_stage = ADSRStage::Sustain;
                    // }

                    // if voice.envelope <= voice.adsr_sustain_level as i16 {
                    //     voice.adsr_stage = ADSRStage::Sustain;
                    // }
                }
            }
            ADSRStage::Sustain => {
                let sustain_period_idx = voice.adsr_sustain_rate as usize;

                if self.should_do_envelope_op(sustain_period_idx) {
                    let mut envelope_val = voice.envelope;

                    // This calculation underflows the envelope if it is 0, but
                    // it gets clamped back to 0 afterwards, resulting in no
                    // change.
                    if envelope_val > 0 {
                        envelope_val -= 1;
                        envelope_val -= envelope_val >> 8;

                        voice.envelope = envelope_val;
                    }
                }
            }
            ADSRStage::Release => {
                voice.envelope = (voice.envelope - 8).max(0);
            }
        }
    }

    fn clock_gain_envelope(&mut self, bus: &mut SdspBus, voice_idx: usize) {
        let voice = &mut bus.voice_regs[voice_idx];

        if matches!(voice.gain_mode, GainMode::Fixed) {
            voice.envelope = (voice.gain_fixed << 4) as i16;
            return;
        }

        let gain_period_idx = voice.gain_rate as usize;

        if !self.should_do_envelope_op(gain_period_idx) {
            return;
        }

        let new_envelope = match voice.gain_mode {
            GainMode::Decrease => voice.envelope - 32,
            GainMode::ExpDecrease => {
                let mut envelope_val = voice.envelope;
                envelope_val -= 1;
                envelope_val -= envelope_val >> 8;
                envelope_val
            }
            GainMode::Increase => voice.envelope + 32,
            GainMode::BentIncrease => {
                if voice.envelope < 0x600 {
                    voice.envelope + 32
                } else {
                    voice.envelope + 8
                }
            }
            GainMode::Fixed => unreachable!(), // Handled above match
        };

        // let clipped_envelope = new_envelope & 0x7FF;
        let clamped_envelope = new_envelope.clamp(0, 0x7FF);

        voice.envelope = clamped_envelope;
    }

    fn push_echo_samples(&mut self, left_sample: i16, right_sample: i16, bus: &mut SdspBus) {
        // TODO: Generate FIR sample and add to echo samples

        let left_echo_final = left_sample;
        let right_echo_final = right_sample;

        let echo_buffer_start = (bus.sdsp_regs.echo_page as usize) << 8;
        let echo_delay_length = (bus.sdsp_regs.echo_delay_time as usize) << 11;

        // Write four bytes to the echo buffer. Handle nasty wrapping logic.
        if bus.sdsp_regs.echo_en {
            let addr = (echo_buffer_start + self.echo_ptr) & 0xFFFF;
            bus.aram[addr] = get_byte_n!(left_echo_final, 0);
            self.echo_ptr += 1;
            self.echo_ptr %= echo_delay_length;
            let addr = (echo_buffer_start + self.echo_ptr) & 0xFFFF;
            bus.aram[addr] = get_byte_n!(left_echo_final, 1);
            self.echo_ptr += 1;
            self.echo_ptr %= echo_delay_length;
            let addr = (echo_buffer_start + self.echo_ptr) & 0xFFFF;
            bus.aram[addr] = get_byte_n!(right_echo_final, 0);
            self.echo_ptr += 1;
            self.echo_ptr %= echo_delay_length;
            let addr = (echo_buffer_start + self.echo_ptr) & 0xFFFF;
            bus.aram[addr] = get_byte_n!(right_echo_final, 1);
            self.echo_ptr += 1;
            self.echo_ptr %= echo_delay_length;
        }
    }

    // Read 16 samples (8 left, 8 right) from the end of the echo buffer for use in generating an
    // echo sample. There's going to be some horrendous index math involved.
    // Returns (left_samples, right_samples), ordered oldest->newest
    fn read_fir_samples(&mut self, bus: &mut SdspBus) -> (Vec<i16>, Vec<i16>) {
        // Special case: if echo delay is 0, we use 4 bytes starting at the echo page as our echo buffer.
        let raw_bytes = if bus.sdsp_regs.echo_delay_time == 0 {
            let echo_base_addr = (bus.sdsp_regs.echo_page as usize) << 8;
            let four_byte_slice = &bus.aram[echo_base_addr..echo_base_addr + 4];
            Vec::from(four_byte_slice)
                .into_iter()
                .cycle()
                .take(32)
                .collect()
        } else {
            let echo_base_addr = (bus.sdsp_regs.echo_page as usize) << 8;
            let echo_mod = (bus.sdsp_regs.echo_delay_time as usize) << 11;
            let mut bytes: Vec<u8> = Vec::new();
            for i in 0..32 {
                let idx = (i + self.echo_ptr) % echo_mod;
                let addr = (echo_base_addr + idx) & 0xFFFF;
                bytes.push(bus.aram[addr]);
            }
            bytes
        };

        let mut left_samples: Vec<i16> = Vec::new();
        let mut right_samples: Vec<i16> = Vec::new();

        for chunk in raw_bytes.chunks(4) {
            let left = i16::from_le_bytes([chunk[0], chunk[1]]);
            let right = i16::from_le_bytes([chunk[2], chunk[3]]);

            left_samples.push(left >> 1);
            right_samples.push(right >> 1);
        }

        (left_samples, right_samples)
    }

    fn generate_echo_samples(&mut self, bus: &mut SdspBus) -> (i16, i16) {
        let (left_samples, right_samples) = self.read_fir_samples(bus);

        let mut left_fir: i32 = 0;
        let mut right_fir: i32 = 0;

        for i in 0..7 {
            left_fir += (left_samples[i] as i32 * (bus.sdsp_regs.fir_regs[i] as i32)) >> 6;
            left_fir &= 0xFFFF;
            right_fir += (right_samples[i] as i32 * (bus.sdsp_regs.fir_regs[i] as i32)) >> 6;
            right_fir &= 0xFFFF;
        }
        left_fir += (left_samples[7] as i32 * (bus.sdsp_regs.fir_regs[7] as i32)) >> 6;
        right_fir += (right_samples[7] as i32 * (bus.sdsp_regs.fir_regs[7] as i32)) >> 6;

        let left_echo = left_fir.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        let right_echo = right_fir.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

        (left_echo, right_echo)
    }

    pub fn generate_sample(&mut self, audio_buffer: &mut Vec<i16>, bus: &mut SdspBus) {
        let mut left_sample: i16 = 0;
        let mut right_sample: i16 = 0;

        let mut left_echo: i16 = 0;
        let mut right_echo: i16 = 0;

        for voice_idx in 0..8 {
            let (voice_left, voice_right) = self.generate_voice_sample(bus, voice_idx);

            left_sample = left_sample.saturating_add(voice_left);
            right_sample = right_sample.saturating_add(voice_right);

            if bus.voice_regs[voice_idx].echo_en {
                left_echo = left_sample.saturating_add(voice_left);
                right_echo = right_sample.saturating_add(voice_right);
            }
        }

        let l_volume = (bus.sdsp_regs.lmain_volume as i8) as i32;
        let r_volume = (bus.sdsp_regs.rmain_volume as i8) as i32;

        left_sample = (((left_sample as i32) * l_volume) >> 7) as i16;
        right_sample = (((right_sample as i32) * r_volume) >> 7) as i16;

        left_sample *= 2;
        right_sample *= 2;

        if bus.sdsp_regs.mute_all {
            left_sample = 0;
            right_sample = 0;
        }

        self.push_echo_samples(left_echo, right_echo, bus);

        audio_buffer.push(left_sample);
        audio_buffer.push(right_sample);
    }

    fn generate_voice_sample(&mut self, bus: &mut SdspBus, voice_idx: usize) -> (i16, i16) {
        if voice_idx >= 8 {
            return (0, 0);
        }

        let raw_sample = if bus.voice_regs[voice_idx].noise_en {
            sign_extend::<15>(self.noise_output as i32)
        } else {
            sign_extend::<15>(self.generate_interpolated_sample(bus, voice_idx) as i32)
        };

        let voice = &mut bus.voice_regs[voice_idx];

        let volume_adjusted_sample = (raw_sample * voice.envelope as i32) >> 11;

        voice.sample_out_high = (volume_adjusted_sample as u16) & 0x7FFF;

        let l_volume = (voice.lchannel_volume as i8) as i32;
        let r_volume = (voice.rchannel_volume as i8) as i32;

        let left_sample = ((volume_adjusted_sample * l_volume) >> 7) as i16;
        let right_sample = ((volume_adjusted_sample * r_volume) >> 7) as i16;

        let mut step = voice.pitch as usize;

        if voice_idx != 0 && voice.pitchmod_en {
            let prev_out = sign_extend::<15>(bus.voice_regs[voice_idx - 1].sample_out_high as i32);

            let factor = ((prev_out >> 4) + 0x400) as usize;

            step = (step * factor) >> 10
        }

        bus.voice_regs[voice_idx].interpolation_idx += step;

        (left_sample, right_sample)
    }

    fn generate_interpolated_sample(&mut self, bus: &mut SdspBus, voice_idx: usize) -> u16 {
        if bus.voice_regs[voice_idx].interpolation_idx >= 0x4000 {
            self.load_new_brr_samples_into_buffer(bus, voice_idx);
            bus.voice_regs[voice_idx].interpolation_idx -= 0x4000;
        }

        let voice = &mut bus.voice_regs[voice_idx];

        let i = voice.interpolation_idx >> 12;
        let gauss_table_offset = (voice.interpolation_idx >> 4) & 0xFF;

        let g0 = Self::GAUSS_LOOKUP_TABLE[255 - gauss_table_offset] as i32;
        let g1 = Self::GAUSS_LOOKUP_TABLE[511 - gauss_table_offset] as i32;
        let g2 = Self::GAUSS_LOOKUP_TABLE[256 + gauss_table_offset] as i32;
        let g3 = Self::GAUSS_LOOKUP_TABLE[0 + gauss_table_offset] as i32;

        let s0 = sign_extend::<15>(
            voice.brr_sample_buffer[(voice.brr_sample_buffer_idx - (i + 1) + 12) % 12] as i32,
        );
        let s1 = sign_extend::<15>(
            voice.brr_sample_buffer[(voice.brr_sample_buffer_idx - (i + 2) + 12) % 12] as i32,
        );
        let s2 = sign_extend::<15>(
            voice.brr_sample_buffer[(voice.brr_sample_buffer_idx - (i + 3) + 12) % 12] as i32,
        );
        let s3 = sign_extend::<15>(
            voice.brr_sample_buffer[(voice.brr_sample_buffer_idx - (i + 4) + 12) % 12] as i32,
        );

        let mut gauss_sample = 0;
        gauss_sample += (g0 * s0) >> 10;
        gauss_sample += (g1 * s1) >> 10;
        gauss_sample += (g2 * s2) >> 10;
        gauss_sample += (g3 * s3) >> 10;
        // gauss_sample = gauss_sample.clamp(u16::MIN as i32, u16::MAX as i32);

        ((gauss_sample >> 1).clamp(-0x4000, 0x3FFF)) as u16
    }

    fn load_new_brr_samples_into_buffer(&mut self, bus: &mut SdspBus, voice_idx: usize) {
        let voice = &mut bus.voice_regs[voice_idx];

        if voice.brr_group_step == 4 {
            voice.brr_group_step = 0;

            if voice.end_of_sample_flag {
                let loop_addr_ptr = (bus.sdsp_regs.sample_directory_page as u16) << 8;
                let loop_addr_ptr = loop_addr_ptr + ((voice.sample_source as u16) << 2) + 2;

                let loop_addr = u16::from_le_bytes([
                    bus.aram[loop_addr_ptr as usize + 0],
                    bus.aram[loop_addr_ptr as usize + 1],
                ]);

                voice.brr_group_addr = loop_addr;
            } else {
                voice.brr_group_addr += 9;
            }
        }

        let brr_header = bus.aram[voice.brr_group_addr as usize];
        let shift = brr_header >> 4;
        let filter = match (brr_header >> 2) & 3 {
            0 => BrrFilter::Filter0,
            1 => BrrFilter::Filter1,
            2 => BrrFilter::Filter2,
            3 => BrrFilter::Filter3,
            _ => unreachable!(),
        };

        if voice.brr_group_step == 0 {
            voice.end_of_sample_flag = get_bit_n!(brr_header, 0);
            voice.loop_flag = get_bit_n!(brr_header, 1);

            if voice.end_of_sample_flag && !voice.loop_flag {
                voice.adsr_stage = ADSRStage::Release;
                voice.envelope = 0;
            }
        }

        let brr_sample_addr = voice.brr_group_addr + 2 * voice.brr_group_step as u16 + 1;
        let brr_samples01 = bus.aram[brr_sample_addr as usize + 0];
        let brr_samples23 = bus.aram[brr_sample_addr as usize + 1];

        let brr_sample0 = sign_extend::<4>((brr_samples01 >> 4) as i32);
        let brr_sample1 = sign_extend::<4>((brr_samples01 & 0xF) as i32);
        let brr_sample2 = sign_extend::<4>((brr_samples23 >> 4) as i32);
        let brr_sample3 = sign_extend::<4>((brr_samples23 & 0xF) as i32);

        let (brr_sample0, brr_sample1, brr_sample2, brr_sample3) = if shift > 12 {
            (
                (brr_sample0 >> 3) << 11, // TODO: Maybe 12?
                (brr_sample1 >> 3) << 11,
                (brr_sample2 >> 3) << 11,
                (brr_sample3 >> 3) << 11,
            )
        } else {
            (
                (brr_sample0 << shift) >> 1,
                (brr_sample1 << shift) >> 1,
                (brr_sample2 << shift) >> 1,
                (brr_sample3 << shift) >> 1,
            )
        };

        let prev_s1 = sign_extend::<15>(
            voice.brr_sample_buffer[(voice.brr_sample_buffer_idx - 1 + 12) % 12] as i32,
        );
        let prev_s2 = sign_extend::<15>(
            voice.brr_sample_buffer[(voice.brr_sample_buffer_idx - 2 + 12) % 12] as i32,
        );

        let (decoded_0, decoded_1, decoded_2, decoded_3) = match filter {
            BrrFilter::Filter0 => (brr_sample0, brr_sample1, brr_sample2, brr_sample3),
            BrrFilter::Filter1 => {
                let s0 = brr_filter1(brr_sample0, prev_s1);
                let s1 = brr_filter1(brr_sample1, s0);
                let s2 = brr_filter1(brr_sample2, s1);
                let s3 = brr_filter1(brr_sample3, s2);
                (s0, s1, s2, s3)
            }
            BrrFilter::Filter2 => {
                let s0 = brr_filter2(brr_sample0, prev_s1, prev_s2);
                let s1 = brr_filter2(brr_sample1, s0, prev_s1);
                let s2 = brr_filter2(brr_sample2, s1, s0);
                let s3 = brr_filter2(brr_sample3, s2, s1);
                (s0, s1, s2, s3)
            }
            BrrFilter::Filter3 => {
                let s0 = brr_filter3(brr_sample0, prev_s1, prev_s2);
                let s1 = brr_filter3(brr_sample1, s0, prev_s1);
                let s2 = brr_filter3(brr_sample2, s1, s0);
                let s3 = brr_filter3(brr_sample3, s2, s1);
                (s0, s1, s2, s3)
            }
        };

        let clamped_0 = decoded_0.clamp(i16::MIN as i32, i16::MAX as i32);
        let clamped_1 = decoded_1.clamp(i16::MIN as i32, i16::MAX as i32);
        let clamped_2 = decoded_2.clamp(i16::MIN as i32, i16::MAX as i32);
        let clamped_3 = decoded_3.clamp(i16::MIN as i32, i16::MAX as i32);

        let clipped_0 = clamped_0 & 0x7FFF;
        let clipped_1 = clamped_1 & 0x7FFF;
        let clipped_2 = clamped_2 & 0x7FFF;
        let clipped_3 = clamped_3 & 0x7FFF;

        voice.brr_sample_buffer[voice.brr_sample_buffer_idx + 0] = clipped_0 as u16;
        voice.brr_sample_buffer[voice.brr_sample_buffer_idx + 1] = clipped_1 as u16;
        voice.brr_sample_buffer[voice.brr_sample_buffer_idx + 2] = clipped_2 as u16;
        voice.brr_sample_buffer[voice.brr_sample_buffer_idx + 3] = clipped_3 as u16;
        voice.brr_sample_buffer_idx += 4;

        // 12 BRR samples in buffer
        if voice.brr_sample_buffer_idx == 12 {
            voice.brr_sample_buffer_idx = 0;
        }

        voice.brr_group_step += 1;
    }
}

fn brr_filter1(brr_sample: i32, prev_sample: i32) -> i32 {
    brr_sample + prev_sample + ((-prev_sample) >> 4)
}

fn brr_filter2(brr_sample: i32, prev_sample1: i32, prev_sample2: i32) -> i32 {
    brr_sample + prev_sample1 * 2 + ((-prev_sample1 * 3) >> 5) - prev_sample2
        + ((prev_sample2 * 1) >> 4)
}

fn brr_filter3(brr_sample: i32, prev_sample1: i32, prev_sample2: i32) -> i32 {
    brr_sample + prev_sample1 * 2 + ((-prev_sample1 * 13) >> 6) - prev_sample2
        + ((prev_sample2 * 3) >> 4)
}
