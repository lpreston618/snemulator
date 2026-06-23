use sdl3::audio::AudioStreamOwner;

use crate::app::resampler::AudioResampler;

pub struct AudioManager {
    stream: AudioStreamOwner,
    resampler: Option<AudioResampler>,
}

impl AudioManager {
    pub fn new(audio_stream: AudioStreamOwner, audio_resampler: Option<AudioResampler>) -> Self {
        Self {
            stream: audio_stream,
            resampler: audio_resampler,
        }
    }

    pub fn pause(&mut self) {
        if let Err(e) = self.stream.pause() {
            log::warn!("Audio stream failure: {e}");
        }
    }

    pub fn resume(&mut self) {
        if let Err(e) = self.stream.resume() {
            log::warn!("Audio stream failure: {e}");
        }
    }

    pub fn upload_samples(&mut self, samples: &[i16]) -> usize {
        if let Some(resampler) = &mut self.resampler {
            let mut samples_left = samples.len();
            let mut samples_start = 0;

            while samples_left >= resampler.input_frames_needed() * 2 {
                let needed = resampler.input_frames_needed() * 2;
                let resampled = resampler.resample(&samples[samples_start..samples_start + needed]);
                
                if let Err(e) = self.stream.put_data_i16(&resampled) {
                    log::warn!("Audio stream write failed: {e}");
                }

                samples_left -= needed;
                samples_start += needed;
            }

            samples_start
        } else {
            if let Err(e) = self.stream.put_data_i16(&samples) {
                log::warn!("Audio stream write failed: {e}");
            }

            samples.len()
        }
    }

    pub fn clear_playing_samples(&mut self) {
        if let Err(e) = self.stream.clear() {
            log::warn!("Audio stream failure: {e}");
        }
    }

    pub fn queued_samples(&self) -> usize {
        (self.stream.queued_bytes().unwrap_or(0) as usize) / std::mem::size_of::<i16>()
    }
}