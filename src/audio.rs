pub const AUDIO_FREQ: usize = 44100;
const IDEAL_FRAME_SAMPLES: usize = AUDIO_FREQ / 60;
pub const MAX_AUDIO_BUFFER_SIZE: usize = (IDEAL_FRAME_SAMPLES + 100) * 2;

#[derive(Debug)]
pub struct AudioBufferStatus {
    pub active: bool,
    pub occupancy: usize,
    pub underrun_likely: bool,
}

static mut BUFFER_STATUS: AudioBufferStatus = AudioBufferStatus {
    active: false,
    occupancy: 0,
    underrun_likely: false,
};


pub unsafe extern "C" fn audio_buffer_status_cb(active: bool, occupancy: libretro_rs::ffi::c_uint, underrun_likely: bool) {
    BUFFER_STATUS.active = active;
    BUFFER_STATUS.occupancy = occupancy as usize;
    BUFFER_STATUS.underrun_likely = underrun_likely;
}

pub fn get_audio_buffer_status() -> AudioBufferStatus {
    unsafe {
        AudioBufferStatus {
            active: BUFFER_STATUS.active,
            occupancy: BUFFER_STATUS.occupancy,
            underrun_likely: BUFFER_STATUS.underrun_likely,
        }
    }
}