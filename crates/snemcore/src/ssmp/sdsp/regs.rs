pub struct SdspRegs {
    // $0C
    pub lmain_volume: u8,
    
    // $1C
    pub rmain_volume: u8,
    
    // $2C
    pub lecho_volume: u8,

    // $3C
    pub recho_volume: u8,
    
    // $4C
    pub key_on: u8,

    // $5C
    pub key_off: u8,
    
    // $6C
    pub soft_reset: bool,
    pub mute_all: bool,
    pub echo_en: bool,
    pub noise_freq: u8,
    
    
    // $0D
    pub echo_feedback: u8,
    
    // $1D
    pub unused: u8,
    
    // $5D
    pub sample_directory_page: u8,
    
    // $6D
    pub echo_page: u8,

    // $7D
    pub echo_delay_time: u8,
}

impl SdspRegs {
    pub fn new() -> Self {
        // TODO: Initialize w/ power on vals
        Self {
            lmain_volume: 0,
            rmain_volume: 0,
            lecho_volume: 0,
            recho_volume: 0,
            key_on: 0,
            key_off: 0,
            soft_reset: false,
            mute_all: false,
            echo_en: false,
            noise_freq: 0,
            echo_feedback: 0,
            unused: 0,
            sample_directory_page: 0,
            echo_page: 0,
            echo_delay_time: 0,
        }
    }
}