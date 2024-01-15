use crate::{
    buffers,
    mixingmatrix::{process_hdm12, process_hdm24}, iir_filter::{IIRFilter, IIRFilterCoefficients},
};
use biquad::{Biquad, Coefficients, DirectForm2Transposed};
// use crate::mixingmatrix;
use cfg_if;
use nalgebra::Vector3;
use num_traits::Zero;
use rand::Rng;

#[allow(unused)]
pub struct FeedbackDelayNetwork {
    pub delaylines: Vec<FDNLine>,
    pub delayline_inputs: Vec<f32>,
    pub delayline_outpus: Vec<f32>,
    pub matrix_inputs: [f32; 24],
    pub matrix_outputs: [f32; 24],
}

impl FeedbackDelayNetwork {
    pub fn new(
        n: usize,
        blocksize: usize, 
        delay_line_lengths: Vec<usize>,
        fdn_b_vecs: Vec<Vec<f32>>,
        fdn_a_vecs: Vec<Vec<f32>>,
        fdn_bh_vec: Vec<f32>,
        fdn_ah_vec: Vec<f32>,
        // biquad_coeff_vec: Vec<Coefficients<f32>>,
    ) -> Self {
        let mut delaylines = Vec::new();
        for i in 0..n {
            // delaylines.push(FDNLine::new(delay_line_lengths[i], biquad_coeff_vec[i]));
            delaylines.push(FDNLine::new(delay_line_lengths[i], fdn_b_vecs[i].clone(), fdn_a_vecs[i].clone()));
        }
        Self {
            delaylines,
            delayline_inputs: vec![0.0; n],
            delayline_outpus: vec![0.0; n],
            matrix_inputs:  [0.0; 24],
            matrix_outputs: [0.0; 24],
            //mixing_matrix: create_hadamard(N),
            // mixing_matrix: todo()
        }
    }
    pub fn from_assets(
        n: usize,
        blocksize: usize,
        delay_line_lengths: Vec<usize>,
        fdn_b_vecs: [[f32;9];24],
        fdn_a_vecs: [[f32;9];24],
        fdn_bh_vec: [f32;9],
        fdn_ah_vec: [f32;9],
        // biquad_coeff_vec: Vec<Coefficients<f32>>,
    ) -> Self {
        let mut delaylines = Vec::new();
        for i in 0..n {
            // delaylines.push(FDNLine::new(delay_line_lengths[i], biquad_coeff_vec[i]));
            delaylines.push(FDNLine::new(delay_line_lengths[i].into(), fdn_b_vecs[i].clone().into(), fdn_a_vecs[i].clone().into()));
        }
        Self {
            delaylines,
            delayline_inputs: vec![0.0; n],
            delayline_outpus: vec![0.0; n],
            matrix_inputs:  [0.0; 24],
            matrix_outputs: [0.0; 24],
            //mixing_matrix: create_hadamard(N),
            // mixing_matrix: todo()
        }
    }
    pub fn process(&mut self, samples: &[f32], output: &mut [f32]) {
        for i in 0..self.delaylines.len() {
            print!("{}", samples[i]);

            self.delayline_inputs[i] = samples
                .into_iter()
                .skip(i)
                .step_by(24)
                .fold(0f32, |acc, &x| acc + x);
            self.delayline_inputs[i] += self.matrix_outputs[i];
            // self.delayline_outpus[i] = self.delaylines[i].tick(self.delayline_inputs[i]);
            output[i] = self.delaylines[i].tick(self.delayline_inputs[i]);
        }

        // process_hdm24(&self.delayline_outpus, &mut self.matrix_outputs);
        process_hdm24(&output, &mut self.matrix_outputs);

        // Mixing Matrix
        // cfg_if::cfg_if! {
        //     if #[cfg(FDN4)] {
        //         process_hdm4(self., self.matrix_outputs);
        //     } else if #[cfg(FDN8)] {
        //         process_HDM8(self.matrix_inputs, self.matrix_outputs);
        //     } else if #[cfg(FDN12)] {
        //         process_hdm12(self.matrix_inputs, self.matrix_outputs);
        //     } else if #[cfg(FDN16)] {
        //         process_hdm16(self.matrix_inputs, self.matrix_outputs);
        //     } else if #[cfg(FDN24)] {
        //         process_hdm24(self.matrix_inputs, self.matrix_outputs);
        //     } else if #[cfg(FDN32)] {
        //         process_hdm32(self.matrix_inputs, self.matrix_outputs);
        //     }
        // }
    }


}


// pub fn create_delay_line_lengths_from_room(room: Room) -> Vec<usize> {
//     vec![0; 1]
// }

#[allow(unused)]
pub struct FDNLine {
    delay_line_buffer: buffers::CircularDelayBuffer,
    filter: IIRFilter,//DirectForm2Transposed<f32>,
    length: usize,
}

impl FDNLine {
    // pub fn new(length: usize, biquad_coeffs: Coefficients<f32>) -> Self {
    pub fn new(length: usize, fdn_b: Vec<f32>, fdn_a: Vec<f32>) -> Self {
        let mut delay_line_buffer = buffers::CircularDelayBuffer::new(length);
        // let coeffs = Coefficients {
        //     a1: biquad_coeffs.0[1],
        //     a2: biquad_coeffs.0[2],
        //     b0: biquad_coeffs.1[0],
        //     b1: biquad_coeffs.1[1],
        //     b2: biquad_coeffs.1[2],
        // };
        delay_line_buffer.set_delay_time_samples(length as f32);
        //let filter: DirectForm2Transposed<f32> =
        //    DirectForm2Transposed::<f32>::new(biquad_coeffs);
        let filter = IIRFilter::from_filtercoefficients(&IIRFilterCoefficients::new(fdn_b, fdn_a));
        Self {
            delay_line_buffer,
            filter,
            length,
        }
    }

    pub fn tick(&mut self, sample: f32) -> f32 {
        let delay_line_output = self.delay_line_buffer.process(sample);
        self.filter.tick(delay_line_output)
    }
}

pub struct FDNInputBuffer {
    pub buffer: Vec<Vec<f32>>,
    wp: usize,
} 

impl FDNInputBuffer {
    pub fn new(number_of_lines: usize, buffer_size: usize) -> Self{
        Self {
            buffer: vec![vec![0.0; buffer_size]; number_of_lines],
            wp: 0, }
    }
    pub fn flush(&mut self) {
        self.buffer.iter_mut().for_each(|buf| {
            buf.iter_mut().for_each(|x|{*x = 0.0f32;})
        })
    }
}


// helper functions
pub fn calc_fdn_delayline_lengths(number_of_lines: usize, room_dims: &Vector3<f32>, speed_of_sound: f32) -> Vec<f32> {
    let mut tau = vec![0.0f32; number_of_lines];
     //Vec::<f32>::with_capacity(number_of_lines);
    let mut rng = rand::thread_rng();    
    let room_dim_mean = room_dims.mean()/3.0;
    let mut rdi = room_dims.iter();
    let mut eps = 0.0f32;
    tau.iter_mut().for_each(|t| {
        match rdi.next() {
            None => {
                rdi = room_dims.iter();
                eps = rng.gen();
            *   t = (1.0 / speed_of_sound) * (rdi.next().unwrap() * eps * room_dim_mean);}
            Some(d) => {
                eps = rng.gen();
                *t = (1.0 / speed_of_sound) * (d + eps * room_dim_mean);
         }   
    }});
    tau
}



pub fn map_ism_to_fdn_channel(channel_index: usize, n_fdn_lines: usize) -> usize {
    channel_index % n_fdn_lines
}

#[cfg(test)]
#[test]

fn test_fdn_dl_creation() {
    
    let room = Vector3::<f32>::new(4.0, 5.0, 6.0);
    let speed_of_sound=343.0f32;
    let nol = 4;

    let mut fdnls = calc_fdn_delayline_lengths(nol, &room, speed_of_sound);
    fdnls.iter_mut().for_each(|y|{*y*=48000.0; *y=y.round();});
    println!("{:?}",fdnls)

}