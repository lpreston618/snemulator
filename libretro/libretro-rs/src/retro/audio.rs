use libretro_rs_ffi::{
    retro_audio_buffer_status_callback,
    retro_audio_buffer_status_callback_t
};

use crate::retro::env::CommandData;

#[repr(transparent)]
pub struct AudioBufferStatusCallback(retro_audio_buffer_status_callback);

impl AudioBufferStatusCallback {
    pub fn new(cb: retro_audio_buffer_status_callback_t) -> AudioBufferStatusCallback {
        AudioBufferStatusCallback(
            retro_audio_buffer_status_callback {
                callback: cb
            }
        )
    }
}

impl CommandData for AudioBufferStatusCallback {}