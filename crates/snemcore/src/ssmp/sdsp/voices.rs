use crate::ssmp::sdsp::{ADSRStage, GainMode};

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
    pub sample_out_high: u16,

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
    pub interpolation_idx: usize,
    pub brr_sample_buffer_idx: usize,
    pub brr_sample_buffer: [u16; 12],
    pub brr_group_addr: u16, // Base address of the BRR sample group (9 bytes)
    pub brr_group_step: usize, // Keeps track of how many sets of 4 BRR samples
                             // have been read into the buffer so far from
                             // the current BRR group.
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
            interpolation_idx: 0,
            brr_sample_buffer_idx: 0,
            brr_sample_buffer: [0; 12],
            brr_group_addr: 0,
            brr_group_step: 0,
        }
    }

    pub fn reset(&mut self) {
        self.adsr_stage = ADSRStage::Release;
        self.end_of_sample_flag = true;
        self.sample_out_high = 0;
        self.envelope = 0;
    }

    pub fn soft_reset(&mut self) {
        self.adsr_stage = ADSRStage::Release;
        self.envelope = 0;
    }
}
