use rubato::{
    Async, FixedAsync, Indexing, Resampler, SincInterpolationParameters, SincInterpolationType, WindowFunction, audioadapter_buffers::number_to_float::InterleavedNumbers, calculate_cutoff
};

pub struct AudioResampler {
    resampler: Async<f32>,
}

impl AudioResampler {
    pub fn new(input_rate: usize, output_rate: usize) -> Self {
        let sinc_len = 128;
        let window = WindowFunction::Blackman2;
        let params = SincInterpolationParameters {
            sinc_len,
            f_cutoff: calculate_cutoff(sinc_len, window),
            interpolation: SincInterpolationType::Quadratic,
            oversampling_factor: 256,
            window,
        };

        let resampler = Async::<f32>::new_sinc(
            output_rate as f64 / input_rate as f64,
            1.1,
            &params,
            1024,
            2,
            FixedAsync::Input,
        ).unwrap();

        Self { resampler }
    }

    // input: interleaved i16 stereo, length must equal input_frames_needed() * 2
    // returns: interleaved i16 stereo resampled output
    pub fn resample(&mut self, input: &[i16]) -> Vec<i16> {
        let num_input_frames = input.len() / 2;
        let ratio = self.resampler.resample_ratio();
        let num_output_frames = (num_input_frames as f64 * ratio).ceil() as usize + 16; // +16 headroom

        // Allocate output as raw bytes for the adapter (2 channels * frames)
        let mut outdata: Vec<i16> = vec![0i16; num_output_frames * 2];

        let input_adapter = InterleavedNumbers::<&[i16], f32>::new(
            &input,
            2,
            num_input_frames,
        ).unwrap();

        let out_capacity = outdata.len() / 2;
        let mut output_adapter = InterleavedNumbers::<&mut [i16], f32>::new_mut(
            &mut outdata,
            2,
            out_capacity,
        ).unwrap();

        let indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            active_channels_mask: None,
            partial_len: None,
        };

        let (_in_used, out_produced) = self.resampler
            .process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing))
            .unwrap();

        outdata.resize(out_produced * 2, 0);

        outdata
    }

    pub fn input_frames_needed(&self) -> usize {
        self.resampler.input_frames_next()
    }
}