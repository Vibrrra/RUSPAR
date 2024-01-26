use std::os::windows::process;

use biquad::{Biquad, DirectForm2Transposed};
use num_complex::{Complex, Complex32};

use crate::{buffers::CircularDelayBuffer, convolver::Spatializer, delaylines::DelayLine, filter::BinauralFilter, iir_filter::IIRFilter};







pub(crate) struct ISMLine {
    delayline: DelayLine,
    filter: DirectForm2Transposed<f32>,
    r2c_scratch_buffer: Vec<Complex<f32>>,
    c2r_scratch_buffer: Vec<Complex<f32>>,
    spatializer: Spatializer,
}

impl ISMLine {
    pub fn new(delaylength: usize, filter: DirectForm2Transposed<f32>, block_size: usize, convolver: Spatializer) -> Self {
        Self {
            delayline: DelayLine::new(delaylength),
            filter,
            r2c_scratch_buffer: vec![Complex::new(0.0, 0.0); block_size * 2],
            c2r_scratch_buffer: vec![Complex::new(0.0, 0.0); block_size * 2],
            spatializer: convolver
        }
    }
    pub fn process(&mut self, audio_in_block: &[f32], 
                              fdn_in_block: &mut [f32], 
                              spatializer_in: &mut [f32], 
                              active_ds_filter: &BinauralFilter, 
                              prev_ds_filter: &BinauralFilter, 
                              audio_out_block: &mut [f32]) {
        audio_in_block.iter().zip(fdn_in_block.iter_mut()).zip(spatializer_in.iter_mut()).zip(audio_out_block.chunks_exact_mut(2))
        .for_each(|(((sample, fdn_in), spat_in), frame)| {
            let line_out = self.filter.run(self.delayline.process(*sample));
            *spat_in = line_out;
            *fdn_in = line_out;
        });
        self.spatializer.process(&spatializer_in, audio_out_block, active_ds_filter, prev_ds_filter)

    }
}