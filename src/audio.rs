use libretro_rs::retro;

use crate::libretro::IDEAL_FPS;

/// SNES produces a 32 KHz sound wave
pub const AUDIO_FREQ: usize = 32000;
pub const IDEAL_FRAME_SAMPLES: usize = AUDIO_FREQ / IDEAL_FPS + 1;
pub const MAX_AUDIO_BUFFER_SIZE: usize = IDEAL_FRAME_SAMPLES * 2;

// #[derive(Debug)]
// pub struct AudioBufferStatus {
//     pub active: bool,
//     pub occupancy: usize,
//     pub underrun_likely: bool,
// }

// static mut BUFFER_STATUS: AudioBufferStatus = AudioBufferStatus {
//     active: false,
//     occupancy: 0,
//     underrun_likely: false,
// };

// pub unsafe extern "C" fn audio_buffer_status_cb(active: bool, occupancy: libretro_rs::ffi::c_uint, underrun_likely: bool) {
//     BUFFER_STATUS.active = active;
//     BUFFER_STATUS.occupancy = occupancy as usize;
//     BUFFER_STATUS.underrun_likely = underrun_likely;
// }

// pub fn get_audio_buffer_status() -> AudioBufferStatus {
//     unsafe {
//         AudioBufferStatus {
//             active: BUFFER_STATUS.active,
//             occupancy: BUFFER_STATUS.occupancy,
//             underrun_likely: BUFFER_STATUS.underrun_likely,
//         }
//     }
// }

// pub fn set_audio_buffer_status_callback(env: &mut impl retro::env::Init) -> bool {
//     unsafe {
//         if env.cmd::<u32, retro::audio::AudioBufferStatusCallback, retro::audio::AudioBufferStatusCallback>(
//             libretro_rs::ffi::RETRO_ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK,
//             retro::audio::AudioBufferStatusCallback::new(
//                 Some(crate::audio::audio_buffer_status_cb),
//             )
//         ).is_err() {
//             false
//         } else {
//             true
//         }
//     }
// }