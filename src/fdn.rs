use std::f32::consts::PI;

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

fn cart2sph(x:f32, y:f32, z:f32) -> Vec<f32> {
    let azimuth = y.atan2(x);
    let elevation = z.atan2((x.powi(2)+y.powi(2)).sqrt());
    let r = (x.powi(2)+y.powi(2)+z.powi(2)).sqrt();
    vec![azimuth, elevation, r]
}

pub fn calc_hrtf_sphere_points(N: usize) -> Vec<(f32, f32)>{
    let mut az: Vec<f32> = Vec::with_capacity(N);
    let mut el: Vec<f32> = Vec::with_capacity(N);
    let mut spherical_coordinates = Vec::new();
    let phi = PI * ((5.0f32).sqrt() - 1.0f32);

    for i in 0..N {
        let y = 1.0 - ((i as f32) / ((N as f32) -1.0 )) * 2.0;
        let radius = (1.0 - y * y).sqrt();
        let theta = phi * (i as f32);
        let x = (theta).cos() * radius;
        let z = (theta).sin() * radius;
        let mut sphc = cart2sph(x, y, z);
        sphc.iter_mut().for_each(|x| {*x = 180.0*  (*x) / PI});
        spherical_coordinates.push((sphc[0].rem_euclid(360.0), sphc[1]));
    }
    spherical_coordinates
//         function [azi, eli] = get_equidist_points_on_sphere(N)

// phi = pi * (sqrt(5) -1);

// c = 1;
// for i = 0 : N-1
//     y = 1 - (i / (N-1)) * 2;
//     radius = sqrt(1- y*y);
//     theta  = phi * i;
//     x = cos(theta) * radius;
//     z = sin(theta) * radius;

//     [azi(c), eli(c), ~] = cart2sph(x,y,z);
//     c = c+1;
// end
// azi = rad2deg(azi);
// eli = rad2deg(eli);
// end
        

    
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

#[test]
fn test_spherical_coords() {
    let N: usize = 24;
    let points = calc_hrtf_sphere_points(N);
    points.iter().for_each(|x| {println!("{:?} ", x)});
}