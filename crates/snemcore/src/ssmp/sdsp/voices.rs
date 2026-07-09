use crate::{savestate, ssmp::sdsp::{ADSRStage, GainMode}};

/// Contains all registers controlling a single voice of the S-DSP
#[derive(Clone, Copy)]
pub struct VoiceRegs {
    // $X0
    pub lchannel_volume: u8,

    // $X1
    pub rchannel_volume: u8,

    // $X2 (low), $X3 (high)
    pub pitch: u16,

    // $X4
    pub sample_source: u8,

    // $X5
    pub adsr_en: bool,
    pub adsr_decay: u8,
    pub adsr_attack: u8,

    // $X6
    pub adsr_sustain_level: u8,
    pub adsr_sustain_rate: u8,

    // $X7
    pub gain_reg_raw: u8,
    pub gain_fixed: u8,
    pub gain_rate: u8,
    pub gain_mode: GainMode,

    // $X8
    pub envelope: i16,

    // $X9
    pub sample_out_high: i16,

    // $XA, $XB
    pub ram_a: u8,
    pub ram_b: u8,

    // $7C + BRR header data
    pub end_of_sample_flag: bool,
    pub loop_flag: bool,

    // $2D
    pub pitchmod_en: bool,

    // $3D
    pub noise_en: bool,

    // $4D
    pub echo_en: bool,

    pub adsr_stage: ADSRStage,
    pub prev_interpolation_idx: usize,
    pub interpolation_idx: usize,
    pub brr_sample_buffer: [i16; 12],
    pub brr_group_addr: u16, // Base address of the BRR sample group (9 bytes)
    pub brr_group_step: usize, // Keeps track of how many sets of 4 BRR samples
    // have been read into the buffer so far from
    // the current BRR group.

    // For debuggign purposes
    pub last_generated_left: i16,
    pub last_generated_right: i16,
}

impl VoiceRegs {
    pub fn new() -> Self {
        Self {
            lchannel_volume: 0,
            rchannel_volume: 0,
            pitch: 0,
            sample_source: 0,
            adsr_en: false,
            adsr_decay: 0,
            adsr_attack: 0,
            adsr_sustain_level: 0,
            adsr_sustain_rate: 0,
            gain_reg_raw: 0,
            gain_fixed: 0,
            gain_rate: 0,
            gain_mode: GainMode::BentIncrease,
            envelope: 0,
            sample_out_high: 0,
            end_of_sample_flag: false,
            loop_flag: false,
            pitchmod_en: false,
            noise_en: false,
            echo_en: false,
            ram_a: 0,
            ram_b: 0,
            adsr_stage: ADSRStage::Attack,
            prev_interpolation_idx: 0,
            interpolation_idx: 0,
            brr_sample_buffer: [0; 12],
            brr_group_addr: 0,
            brr_group_step: 0,
            last_generated_left: 0,
            last_generated_right: 0,
        }
    }

    pub fn save_state(&self) -> savestate::VoiceState {
        savestate::VoiceState {
            lchannel_volume: self.lchannel_volume,
            rchannel_volume: self.rchannel_volume,
            pitch: self.pitch,
            sample_source: self.sample_source,
            adsr_en: self.adsr_en,
            adsr_decay: self.adsr_decay,
            adsr_attack: self.adsr_attack,
            adsr_sustain_level: self.adsr_sustain_level,
            adsr_sustain_rate: self.adsr_sustain_rate,
            gain_reg_raw: self.gain_reg_raw,
            gain_fixed: self.gain_fixed,
            gain_rate: self.gain_rate,
            gain_mode: self.gain_mode,
            envelope: self.envelope,
            sample_out_high: self.sample_out_high,
            ram_a: self.ram_a,
            ram_b: self.ram_b,
            end_of_sample_flag: self.end_of_sample_flag,
            loop_flag: self.loop_flag,
            pitchmod_en: self.pitchmod_en,
            noise_en: self.noise_en,
            echo_en: self.echo_en,
            adsr_stage: self.adsr_stage,
            interpolation_idx: self.interpolation_idx,
            brr_sample_buffer: self.brr_sample_buffer,
            brr_group_addr: self.brr_group_addr,
            brr_group_step: self.brr_group_step,
        }
    }

    pub fn load_state(&mut self, state: &savestate::VoiceState, _version: u32) {
        self.lchannel_volume = state.lchannel_volume;
        self.rchannel_volume = state.rchannel_volume;
        self.pitch = state.pitch;
        self.sample_source = state.sample_source;
        self.adsr_en = state.adsr_en;
        self.adsr_decay = state.adsr_decay;
        self.adsr_attack = state.adsr_attack;
        self.adsr_sustain_level = state.adsr_sustain_level;
        self.adsr_sustain_rate = state.adsr_sustain_rate;
        self.gain_reg_raw = state.gain_reg_raw;
        self.gain_fixed = state.gain_fixed;
        self.gain_rate = state.gain_rate;
        self.gain_mode = state.gain_mode;
        self.envelope = state.envelope;
        self.sample_out_high = state.sample_out_high;
        self.ram_a = state.ram_a;
        self.ram_b = state.ram_b;
        self.end_of_sample_flag = state.end_of_sample_flag;
        self.loop_flag = state.loop_flag;
        self.pitchmod_en = state.pitchmod_en;
        self.noise_en = state.noise_en;
        self.echo_en = state.echo_en;
        self.adsr_stage = state.adsr_stage;
        self.interpolation_idx = state.interpolation_idx;
        self.brr_sample_buffer = state.brr_sample_buffer;
        self.brr_group_addr = state.brr_group_addr;
        self.brr_group_step = state.brr_group_step;
    }

    pub fn power_on(&mut self) {
        *self = VoiceRegs::new();
        self.reset();
    }

    pub fn reset(&mut self) {
        self.adsr_stage = ADSRStage::Release;
        self.end_of_sample_flag = true;
        self.sample_out_high = 0;
        self.envelope = 0;
        self.last_generated_left = 0;
        self.last_generated_right = 0;
    }

    pub fn soft_reset(&mut self) {
        self.adsr_stage = ADSRStage::Release;
        self.envelope = 0;
    }
}
