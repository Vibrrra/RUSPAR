//use biquad::{Hertz, Biquad};

// use crate::delay_line::{self, RingBuffer};

use std::marker::PhantomData;

use biquad::coefficients;
use num_traits::Float;

use crate::buffers::CircularDelayBuffer;



#[allow(dead_code)]
enum FilterType {
    FIR,
    IIR
}

#[derive(Clone)]
pub struct IIRFilterCoefficients<T>
where T: Float
{
    b: Vec<T>,
    a: Vec<T>,
    a0: T,
    b0: T,
    b_last: T,
    a_last: T,
    order: usize
}


impl<T> IIRFilterCoefficients<T> 
where T: Float
{
    pub fn new(mut b: Vec<T>, mut a: Vec<T>) -> Self {
        
        
        // extract a0 to scale all coefficients if it is != 1.0
        let a0: T = a[0];
        let b0: T = b[0] /a[0];
        
        
        let num_b = b.len();
        let num_a = a.len();

        if num_a == 1 {
            panic!("More 'a' coefficients needed to be a IIR filter!")
        }
        
        let order = (num_b as i32).max(num_a as i32) as usize -1;
        
        let mut bn: Vec<T> = vec![T::zero(); order-1];
        let mut an: Vec<T> = vec![T::zero(); order-1];
        
        
        [0..order-1].iter().enumerate().for_each(|(i, _)| {bn[i] = b[i+1] / a0});
        
        [0..order-1].iter().enumerate().for_each(|(i, _)| {an[i] = a[i+1] / a0});

        let b_last = b[num_b-1];
        let a_last = a[num_a-1];
        Self {b: bn, a: an, a0, b0, a_last, b_last, order}
    }
    fn required_internal_buffer(&self) -> usize{
        self.order        
    }
}

#[derive(Clone)]
pub struct IIRFilter 
{
    coefficients: IIRFilterCoefficients<f32>,
    buffer: Vec<f32>,
}

impl IIRFilter 
{

    pub fn from_filtercoefficients(coefficients: &IIRFilterCoefficients<f32>) -> Self {
        
        let mut buffer = vec![0.0f32; coefficients.required_internal_buffer() ];
                
        Self {
            coefficients: coefficients.clone(),
            buffer,        
        }
        
    }
    
    
    pub fn tick(&mut self, sample: f32) -> f32 {
      
        let yn = self.coefficients.b0 * sample + self.buffer[1];
        
        if self.coefficients.order > 1 {
        [0..self.coefficients.order-1].iter()
                                      .enumerate()
                                      .for_each(|(i,_)|
                                                {
                                                    self.buffer[i+1] = self.coefficients.b[i] * sample 
                                                                    +self.buffer[i+2]
                                                                    -self.coefficients.a[i] * yn;
                                                });
                                            }
        self.buffer[self.coefficients.order-1] = self.coefficients.b_last * sample
                                              -self.coefficients.a_last * yn;
        
        yn
        
        
    }}


#[derive(Default, Debug, Clone)]
pub struct HrtfFilterIIR {
    coeffs: HRTFFilterIIRCoefficients,
    buffer: [f32; 32],
   
}
impl HrtfFilterIIR {
    pub fn from_coeffs(coeffs: HRTFFilterIIRCoefficients) -> Self {
        Self {
            coeffs,
            buffer: [0.0; 32],
         
        }
    }
    pub fn process(&mut self, audio_in: f32) -> f32 {
        // let mut buffer_next = self.buffer.iter_mut();
        
        let mut y: f32 = 0.0;
        unsafe {
            y = *self.buffer.get_unchecked(0) + audio_in * *self.coeffs.b.get_unchecked(0);

            for i in 0 .. 15 {
                *self.buffer.get_unchecked_mut(i) = *self.buffer.get_unchecked(i+1) 
                + audio_in * *self.coeffs.b.get_unchecked(i+1) 
                + y * *self.coeffs.a.get_unchecked(i+1); 

            }
            for i in 15..30 {
                *self.buffer.get_unchecked_mut(i) = *self.buffer.get_unchecked(i+1) 
                + audio_in * *self.coeffs.b.get_unchecked(i+1); 
            }
            *self.buffer.get_unchecked_mut(31) = audio_in * *self.coeffs.b.get_unchecked(32); 
            
        }
        y
    }
   
    pub fn flush(&mut self) {
        self.buffer.iter_mut().for_each(|x| {*x = 0.0f32})
    }
}

#[derive(Debug, Clone)]
pub struct HRTFFilterIIRCoefficients {
    b: [f32; 33],
    a: [f32; 16],
}

impl Default for HRTFFilterIIRCoefficients {
    fn default() -> Self {
        let mut b = [0.0; 33];
        b[0] = 1.0;
        let mut a =  [0.0; 16]; 
        a[0] = 1.0; // just for convenience
        Self { b, a}        
    }
}

impl HRTFFilterIIRCoefficients{
    pub fn new(a_coeffs: &[f32], b_coeffs: &[f32]) -> HRTFFilterIIRCoefficients {
        let mut filter_coeffs = HRTFFilterIIRCoefficients::default();
        filter_coeffs.a.copy_from_slice(a_coeffs);
        filter_coeffs.b.copy_from_slice(b_coeffs);
        filter_coeffs

    } 
    pub fn update_coeffs(&mut self, new_coeffs: &HRTFFilterIIRCoefficients,) {
        self.a.copy_from_slice(&new_coeffs.a);
        self.b.copy_from_slice(&new_coeffs.b);
    }
}

#[derive(Debug, Clone)]
pub struct HrtfProcessorIIR
{
    left_delay: CircularDelayBuffer,
    right_delay: CircularDelayBuffer,
    left_delay_old: CircularDelayBuffer,
    right_delay_old: CircularDelayBuffer,
    hrir_iir_r: HrtfFilterIIR,
    hrir_iir_r_old: HrtfFilterIIR,
    hrir_iir_l: HrtfFilterIIR,
    hrir_iir_l_old: HrtfFilterIIR,   
}

impl HrtfProcessorIIR
{
    pub fn new() -> Self {
        Self {
            left_delay: CircularDelayBuffer::new(32),
            right_delay: CircularDelayBuffer::new(32),
            left_delay_old: CircularDelayBuffer::new(32),
            right_delay_old: CircularDelayBuffer::new(32),
            hrir_iir_l: HrtfFilterIIR::default(),
            hrir_iir_l_old: HrtfFilterIIR::default(),           
            hrir_iir_r: HrtfFilterIIR::default(),
            hrir_iir_r_old: HrtfFilterIIR::default(),           
        } 
    }
    pub fn update(&mut self, new_coeffs_l: HRTFFilterIIRCoefficients, new_coeffs_r: HRTFFilterIIRCoefficients) {
        self.hrir_iir_l_old.coeffs.update_coeffs(&self.hrir_iir_l.coeffs);
        self.hrir_iir_l.coeffs = new_coeffs_l;
        self.hrir_iir_r_old.coeffs.update_coeffs(&self.hrir_iir_r.coeffs);
        self.hrir_iir_r.coeffs = new_coeffs_r;
    }
    pub fn process(&mut self, audio_in: f32, crossfade_in_val: f32, crossfade_out_val: f32) -> [f32; 2] { 
        let mut l = self.left_delay.process(audio_in);
        let mut r = self.right_delay.process(audio_in);
        let mut l_old = self.left_delay_old.process(audio_in);
        let mut r_old = self.right_delay_old.process(audio_in);
        l = self.hrir_iir_l.process(l) * crossfade_in_val;
        l += self.hrir_iir_l_old.process(l_old) * crossfade_out_val;
        r = self.hrir_iir_l.process(r) * crossfade_in_val;
        r += self.hrir_iir_l_old.process(r_old) * crossfade_out_val;
        [l,r]

    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_filter() {
        let b =vec![2.0, 1.0, 3.2, -0.54, 2.4];
        let a= vec![ 1.0, 1.1, 0.6, -0.5];
        let filter_config = super::IIRFilterCoefficients::new(b, a);
        let mut rb = super::IIRFilter::from_filtercoefficients(&filter_config);
        
        let x: Vec<f32> = vec![1.0, 0.5, -0.7, 1.2, 1.0];
        let mut y: Vec<f32> = vec![0f32 ; x.len()];
        for (i, d) in x.iter().enumerate() {
            y[i] = rb.tick(*d);
        }
        println!("{:?}", y);
        //assert!((y[2] - 3.32f64).abs() < 0.00001)
    }
    #[test]
    fn run_small_filter() {
        let b =vec![2.0];
        let a= vec![ 1.0, 1.1];       
        let filter_config = super::IIRFilterCoefficients::new(b, a);
        let mut rb = super::IIRFilter::from_filtercoefficients(&filter_config);
        
        let x = vec![1.0, 0.0, 0.0, 0.0, 0.0];
        let mut y = vec![0f32 ; x.len()];
        for (i, d) in x.iter().enumerate() {
            y[i] = rb.tick(*d);
        }
        println!("{:?}", y);
    }
    #[test]
    fn test_filter_perform() {
        let b =vec![2.0, 1.0, 3.2, 6.7, 5.4, 2.1, 0.3, 1.3];
        let a= vec![ 1.0, 1.1, 0.6, 1.0, 1.1, 0.6, 1.0, 1.1, 0.6];
        let filter_config = super::IIRFilterCoefficients::new(b, a);
        let mut rb = super::IIRFilter::from_filtercoefficients(&filter_config);
        
        let mut x = vec![0.0; 300_000_000];
        x[0] = 1.0;
        let mut y = vec![0f64 ; x.len()];
    }
    #[test]
    fn test_slice() {
        let mut a = vec![1.2, 3.0];
        a = a[0..a.len()-1].to_vec();
        println!("{:?}", a);
    }
    #[test]
    fn z() {
        let z = vec![1,2,3];
        z[1..3].iter().enumerate().for_each(|(i, a)| {println!("{}, {}", i, a)});
    }

    #[test]
    fn test_filter_coefficients() {
        let b = vec![2.0, 1.2];
        let a = vec![1.0];
        let filter_coeffs = super::IIRFilterCoefficients::new(b, a);
        assert_eq!(filter_coeffs.a0, 1.0);
        //assert_eq!(filter_coeffs.a[0], 3.0 / filter_coeffs.a0);
    }
    #[test]
    fn test_loop(){
        println!("start!");
        for i in 0..0{
            println!("{}", i);
        }
        print!("finished");
    }
}
