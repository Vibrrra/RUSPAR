
use num_traits::Zero;
use num_complex::Complex;
use crate::filter::{FilterStorage, FFTManager, BinauralFilterType, MonoFilterType, BinauralFilter, MonoFilter};
use std::f32::consts::PI;

#[allow(unused)]
pub struct Spatializer {
    // filter segmentation values
    n_points: usize,
    n_segments_ds: usize,
    // n_segments_er: usize,
    // n_segments_lr: usize,
    // n_segments_sd: usize,
    n_segments_total: usize, 

    // temporary buffers
    overlap: Vec<Vec<f32>>,    
    overlap_prev: Vec<Vec<f32>>,    
    input_buf: Vec<f32>,
    input_f: Vec<Vec<Complex<f32>>>,
    temp_buf_l: Vec<Complex<f32>>,
    temp_buf_r: Vec<Complex<f32>>,
    temp_buf_prev_l: Vec<Complex<f32>>,
    temp_buf_prev_r: Vec<Complex<f32>>,
    temp_output_buf_l: Vec<f32>,
    temp_output_buf_r: Vec<f32>,
    temp_output_prev_buf_l: Vec<f32>,
    temp_output_prev_buf_r: Vec<f32>,
    output_buf: Vec<Vec<f32>>,
    index: usize, 
 
    // FFTManager
    fft_manager: FFTManager,
    
    // filter crossfading 
    fade_in: Vec<f32>,
    fade_out: Vec<f32>,

    // risky
    old_slice: Vec<f32>
}

impl Spatializer {

    // pub fn new (blocksize: usize, fft_manager: FFTManager, filter_storage:  FilterStorage) -> Self {
    pub fn new (blocksize: usize, fft_manager: FFTManager, filter_storage: &FilterStorage) -> Self {
        let n_points = blocksize;
        
        // init crossfading // sin²
        let mut fade_in: Vec<f32> = vec![0.0; n_points];
        let mut fade_out: Vec<f32> = vec![0.0; n_points];
        for i in 0..n_points {
            fade_out[i] = (( (PI/2.0) * (i as f32)  / ((2*n_points-1) as f32)).cos()).powf(2.0);
            fade_in[i] =  (( (PI/2.0) * (i as f32)  / ((2*n_points-1) as f32)).sin()).powf(2.0);
        }
        
        // fade_in.reverse();
        // init segmentation values        
        let n_segments_ds: usize = filter_storage.get_n_stereo_segments(BinauralFilterType::DirectSound);
        let mut n_segments_total: usize = n_segments_ds;
      
        // init temporary buffers
        let overlap= vec![vec![0.0; n_points];2];
        let overlap_prev= vec![vec![0.0; n_points];2];
        let temp_output_buf_l = vec![0.0; 2*n_points];
        let temp_output_buf_r = vec![0.0; 2*n_points];
        let temp_output_prev_buf_l = vec![0.0; 2*n_points];
        let temp_output_prev_buf_r = vec![0.0; 2*n_points];
        let output_buf = vec![vec![0.0; n_points]; 2];
        let temp_buf_l = vec![Complex::zero(); n_points + 1];
        let temp_buf_r = vec![Complex::zero(); n_points + 1];
        let temp_buf_prev_l = vec![Complex::zero(); n_points + 1];
        let temp_buf_prev_r = vec![Complex::zero(); n_points + 1];
        let input_buf = vec![0.0; 2*n_points];
        let input_f = vec![vec![Complex::zero(); n_points+1]; n_segments_total];
        let old_slice = vec![0.0; n_points];

        Self {
            n_points,
            n_segments_ds,
            // n_segments_er,
            // n_segments_lr,
            // n_segments_sd,
            n_segments_total,
            overlap,
            overlap_prev,
            temp_output_buf_l,
            temp_output_buf_r,
            temp_output_prev_buf_l,
            temp_output_prev_buf_r,
            output_buf,
            temp_buf_l,
            temp_buf_r,
            temp_buf_prev_l,
            temp_buf_prev_r,
            fft_manager,
            index: 0,
            input_buf,
            input_f,
            fade_in,
            fade_out,
            // riksy
            old_slice,
        }
    }

    // process function! full implementation block
    pub fn process(&mut self, 
                    input: &[f32], 
                    output: &mut [f32], 
                    active_ds_filter: &BinauralFilter, 
                    // active_er_filter: &BinauralFilter, 
                    // active_lr_filter: &BinauralFilter, 
                    // active_sd_filter: &MonoFilter, 
                    prev_ds_filter: &BinauralFilter, 
                    // prev_er_filter: &BinauralFilter, 
                    // prev_lr_filter: &BinauralFilter,
                    // prev_sd_filter: &MonoFilter
                ) {
                     
        
        // // gather input
        self.input_buf[0..self.n_points].copy_from_slice(&self.old_slice);
        self.old_slice.copy_from_slice(input);
        self.input_buf[self.n_points..].copy_from_slice(input);

        // FFT stuff )
        self.index = (self.index + 1) % self.n_segments_total;
        self.fft_manager.transform_to_f_with_scratch(&mut self.input_buf, &mut self.input_f[self.index]);
        
        // Loop through history of input FTs, multiply with filter FTs, accumulate the result.
        // Side note: Due to the 1/N scaling in the real-FFT, the elements in frequency-transformed segments
        // of the source directivity have to be scaled again by the length of the original transform signal.
        // first convolve new block
        let segm = 0;
        let mut hist_idx = (self.index + self.n_segments_total - segm) % self.n_segments_total;
        if true { // deactivate active conv when false (debugging)
        self.temp_buf_l.iter_mut()
                        .zip(self.input_f[hist_idx]
                        .iter()
                        .zip(active_ds_filter.data_f_l[segm].iter()
                            ))
                        .for_each(|(c,(a,b))|{*c = a*b; });

        self.temp_buf_r.iter_mut()
                        .zip(self.input_f[hist_idx]
                        .iter()
                        .zip(active_ds_filter.data_f_r[segm].iter()
                            ))
                        .for_each(|(c,(a,b))|{*c = a*b;});

        for segm in 1..self.n_segments_ds {
            let hist_idx = (self.index + self.n_segments_total - segm) % self.n_segments_total;
            self.temp_buf_l.iter_mut()                        
                        .zip(self.input_f[hist_idx]
                        .iter()
                        .zip(active_ds_filter.data_f_l[segm].iter()
                            ))
                        .for_each(|(c,(a,b))|{*c += a*b;});
            self.temp_buf_r.iter_mut()
                        .zip(self.input_f[hist_idx]
                        .iter()
                        .zip(active_ds_filter.data_f_r[segm].iter()
                            ))
                        .for_each(|(c,(a,b))|{*c += a*b;});
        }

    } // deactivated active convolution
        // From here on the previous filters are applied for crossfading..

        // DS
        if true {        

            let segm = 0;
            hist_idx = (self.index + self.n_segments_total - segm) % self.n_segments_total;
            self.temp_buf_prev_l.iter_mut()
                        .zip(self.input_f[hist_idx]
                        .iter()
                        .zip(prev_ds_filter.data_f_l[segm].iter()
                            ))
                        .for_each(|(c,(a,b))|{*c = a*b;});
            self.temp_buf_prev_r.iter_mut()
                        .zip(self.input_f[hist_idx]
                        .iter()
                        .zip(prev_ds_filter.data_f_r[segm].iter()
                            ))
                        .for_each(|(c,(a,b))|{*c = a*b;});

            for segm in 1..self.n_segments_ds {
                hist_idx = (self.index + self.n_segments_total - segm) % self.n_segments_total;
                self.temp_buf_prev_l.iter_mut()
                        .zip(self.input_f[hist_idx]
                        .iter()
                        .zip(prev_ds_filter.data_f_l[segm].iter()
                            ))
                        .for_each(|(c,(a,b))|{*c += a*b;});
                self.temp_buf_prev_r.iter_mut()
                        .zip(self.input_f[hist_idx]
                        .iter()
                        .zip(prev_ds_filter.data_f_r[segm].iter()
                            ))
                        .for_each(|(c,(a,b))|{*c += a*b;});
            }
        }    
        
        // }

        // IFFT result, store result and overlap
        self.fft_manager.transform_to_t_with_scratch(
                &mut self.temp_buf_l,
                &mut self.temp_output_buf_l
            );
        self.fft_manager.transform_to_t_with_scratch(
                &mut self.temp_buf_r, 
                &mut self.temp_output_buf_r
            );           
        self.fft_manager.transform_to_t_with_scratch(
                &mut self.temp_buf_prev_l,
                &mut self.temp_output_prev_buf_l
            );

        self.fft_manager.transform_to_t_with_scratch(
                &mut self.temp_buf_prev_r, 
                &mut self.temp_output_prev_buf_r
            );

        // add previous filter output & crossfading
        output.chunks_mut(2).enumerate().for_each(|(i, s)| {
            s[0] += self.temp_output_buf_l[self.n_points + i] * self.fade_in[i] + self.temp_output_prev_buf_l[self.n_points + i] * self.fade_out[i];
            s[1] += self.temp_output_buf_r[self.n_points + i] * self.fade_in[i] + self.temp_output_prev_buf_r[self.n_points + i] * self.fade_out[i];
        }); // += -> = tu das weg im zweifel

                     
    }
           
}