use crate::{
    buffers,
    image_source_method::ISMRoom,
    mixingmatrix::{self, process_HDM12, process_HDM4, process_HDM8},
};
use biquad::{Biquad, Coefficients, DirectForm2Transposed};
// use crate::mixingmatrix;
use cfg_if;
use ndarray::Array2;
pub struct FeedbackDelayNetwork {
    delaylines: Vec<FDNLine>,
    delayline_inputs: Vec<f32>,
    delayline_outpus: Vec<f32>,
    matrix_inputs: Vec<f32>,
    matrix_outputs: [f32; 12],
}

impl FeedbackDelayNetwork {
    pub fn new(
        N: usize,
        delay_line_lengths: Vec<usize>,
        biquad_coeff_vec: Vec<Coefficients<f32>>,
    ) -> Self {
        let mut delaylines = Vec::new();
        for i in 0..N {
            delaylines.push(FDNLine::new(delay_line_lengths[i], biquad_coeff_vec[i]));
        }
        Self {
            delaylines,
            delayline_inputs: vec![0.0; N],
            delayline_outpus: vec![0.0; N],
            matrix_inputs: vec![0.0; N],
            matrix_outputs: [0.0; 12],
            //mixing_matrix: create_hadamard(N),
            // mixing_matrix: todo()
        }
    }

    pub fn process(&mut self, samples: &[f32]) {
        for i in 0..self.delaylines.len() {
            print!("{}", samples[i]);

            self.delayline_inputs[i] = samples
                .into_iter()
                .skip(i)
                .step_by(12)
                .fold(0f32, |acc, &x| acc + x);
            self.delayline_inputs[i] += self.matrix_outputs[i];
            self.delayline_outpus[i] = self.delaylines[i].tick(self.delayline_inputs[i]);
        }

        process_HDM12(&self.delayline_outpus, &mut self.matrix_outputs);

        // Mixing Matrix
        cfg_if::cfg_if! {
            if #[cfg(FDN4)] {
                process_HDM4(self., self.matrix_outputs);
            } else if #[cfg(FDN8)] {
                process_HDM8(self.matrix_inputs, self.matrix_outputs);
            } else if #[cfg(FDN12)] {
                process_HDM12(self.matrix_inputs, self.matrix_outputs);
            } else if #[cfg(FDN16)] {
                process_HDM16(self.matrix_inputs, self.matrix_outputs);
            } else if #[cfg(FDN24)] {
                process_HDM24(self.matrix_inputs, self.matrix_outputs);
            } else if #[cfg(FDN32)] {
                process_HDM32(self.matrix_inputs, self.matrix_outputs);
            }
        }
    }
}

pub fn create_delay_line_lengths_from_room(room: ISMRoom) -> Vec<usize> {
    vec![0; 1]
}

struct FDNLine {
    delay_line_buffer: buffers::CircularDelayBuffer,
    filter: DirectForm2Transposed<f32>,
    length: usize,
}

impl FDNLine {
    pub fn new(length: usize, biquad_coeffs: Coefficients<f32>) -> Self {
        let mut delay_line_buffer = buffers::CircularDelayBuffer::new(length);
        // let coeffs = Coefficients {
        //     a1: biquad_coeffs.0[1],
        //     a2: biquad_coeffs.0[2],
        //     b0: biquad_coeffs.1[0],
        //     b1: biquad_coeffs.1[1],
        //     b2: biquad_coeffs.1[2],
        // };
        delay_line_buffer.set_delay_time_samples(length as f32);
        let mut filter: DirectForm2Transposed<f32> =
            DirectForm2Transposed::<f32>::new(biquad_coeffs);
        Self {
            delay_line_buffer,
            filter,
            length,
        }
    }

    pub fn tick(&mut self, sample: f32) -> f32 {
        let delay_line_output = self.delay_line_buffer.process(sample);
        self.filter.run(delay_line_output)
    }
}
