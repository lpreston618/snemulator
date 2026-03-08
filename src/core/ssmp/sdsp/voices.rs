use crate::core::ssmp::sdsp::GainMode;

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
    pub envelope_high: u8,
    
    // $X9
    pub sample_out_high: u8,
    
    // $7C
    pub end_of_sample_flag: bool,
    
    // $2D
    pub pitchmod_en: bool,
    
    // $3D
    pub noise_en: bool,
    
    // $4D
    pub echo_en: bool,
    
    // $XF
    pub filter_coeff: u8,
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
            envelope_high: 0,
            sample_out_high: 0,
            end_of_sample_flag: false,
            pitchmod_en: false,
            noise_en: false,
            echo_en: false,
            filter_coeff: 0,
        }
    }
}