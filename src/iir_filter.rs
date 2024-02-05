//use biquad::{Hertz, Biquad};

// use crate::delay_line::{self, RingBuffer};

use std::marker::PhantomData;

use biquad::coefficients;
use num_traits::Float;

use crate::{buffers::CircularDelayBuffer, delaylines::CircularBuffer};

#[allow(dead_code)]
enum FilterType {
    FIR,
    IIR,
}

#[derive(Clone)]
pub struct IIRFilterCoefficients<T>
where
    T: Float,
{
    b: Vec<T>,
    a: Vec<T>,
    a0: T,
    b0: T,
    b_last: T,
    a_last: T,
    order: usize,
}

impl<T> IIRFilterCoefficients<T>
where
    T: Float,
{
    pub fn new(mut b: Vec<T>, mut a: Vec<T>) -> Self {
        // extract a0 to scale all coefficients if it is != 1.0
        let a0: T = a[0];
        let b0: T = b[0] / a[0];

        let num_b = b.len();
        let num_a = a.len();

        if num_a == 1 {
            panic!("More 'a' coefficients needed to be a IIR filter!")
        }

        let order = (num_b as i32).max(num_a as i32) as usize - 1;

        let mut bn: Vec<T> = vec![T::zero(); order - 1];
        let mut an: Vec<T> = vec![T::zero(); order - 1];

        [0..order - 1]
            .iter()
            .enumerate()
            .for_each(|(i, _)| bn[i] = b[i + 1] / a0);

        [0..order - 1]
            .iter()
            .enumerate()
            .for_each(|(i, _)| an[i] = a[i + 1] / a0);

        let b_last = b[num_b - 1];
        let a_last = a[num_a - 1];
        Self {
            b: bn,
            a: an,
            a0,
            b0,
            a_last,
            b_last,
            order,
        }
    }
    fn required_internal_buffer(&self) -> usize {
        self.order
    }
}

#[derive(Clone)]
pub struct IIRFilter {
    coefficients: IIRFilterCoefficients<f32>,
    buffer: Vec<f32>,
}

impl IIRFilter {
    pub fn from_filtercoefficients(coefficients: &IIRFilterCoefficients<f32>) -> Self {
        let mut buffer = vec![0.0f32; coefficients.required_internal_buffer()];

        Self {
            coefficients: coefficients.clone(),
            buffer,
        }
    }

    pub fn tick(&mut self, sample: f32) -> f32 {
        let yn = self.coefficients.b0 * sample + self.buffer[1];

        if self.coefficients.order > 1 {
            [0..self.coefficients.order - 1]
                .iter()
                .enumerate()
                .for_each(|(i, _)| {
                    self.buffer[i + 1] = self.coefficients.b[i] * sample + self.buffer[i + 2]
                        - self.coefficients.a[i] * yn;
                });
        }
        self.buffer[self.coefficients.order - 1] =
            self.coefficients.b_last * sample - self.coefficients.a_last * yn;

        yn
    }
}

#[derive(Debug, Clone)]
pub struct HrtfFilterIIR {
    pub coeffs: HRTFFilterIIRCoefficients,
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
}
impl Default for HrtfFilterIIR {
    fn default() -> Self {
        Self {
            coeffs: HRTFFilterIIRCoefficients::default(),
            buffer_l: vec![0.0f32; 32],
            buffer_r: vec![0.0f32; 32],
        }
    }
}
impl HrtfFilterIIR {
    pub fn from_coeffs(coeffs: HRTFFilterIIRCoefficients) -> Self {
        Self {
            coeffs,
            buffer_l: vec![0.0; 32],
            buffer_r: vec![0.0; 32],
        }
    }
    pub fn process(&mut self, audio_in: &[f32; 2]) -> [f32; 2] {
        // let mut buffer_next = self.buffer.iter_mut();

        let mut y_l: f32 = 0.0;
        let mut y_r: f32 = 0.0;
        // unsafe {
        //     y_l = *self.buffer_l.get_unchecked(0) + audio_in[0] * *self.coeffs.b_l.get_unchecked(0);
        //     y_r = *self.buffer_r.get_unchecked(0) + audio_in[1] * *self.coeffs.b_r.get_unchecked(0);

        //     for i in 0..15 {
        //         *self.buffer_l.get_unchecked_mut(i) = *self.buffer_l.get_unchecked(i + 1)
        //             + audio_in[0] * *self.coeffs.b_l.get_unchecked(i + 1)
        //             + y_l * -*self.coeffs.a_l.get_unchecked(i + 1);
        //         *self.buffer_r.get_unchecked_mut(i) = *self.buffer_r.get_unchecked(i + 1)
        //             + audio_in[1] * *self.coeffs.b_r.get_unchecked(i + 1)
        //             + y_r * -*self.coeffs.a_r.get_unchecked(i + 1);
        //     }
        //     for i in 15..30 {
        //         *self.buffer_l.get_unchecked_mut(i) = *self.buffer_l.get_unchecked(i + 1)
        //             + audio_in[0] * *self.coeffs.b_l.get_unchecked(i + 1);
        //         *self.buffer_r.get_unchecked_mut(i) = *self.buffer_r.get_unchecked(i + 1)
        //             + audio_in[1] * *self.coeffs.b_r.get_unchecked(i + 1);
        //     }
        //     *self.buffer_l.get_unchecked_mut(31) = audio_in[0] * *self.coeffs.b_l.get_unchecked(32);
        //     *self.buffer_r.get_unchecked_mut(31) = audio_in[1] * *self.coeffs.b_r.get_unchecked(32);
        // }
        let y_l = self.coeffs.b_l[0] * audio_in[0] + self.buffer_l[0];
        let y_r = self.coeffs.b_r[0] * audio_in[1] + self.buffer_r[0];
        for i in 0..16 {
            self.buffer_l[i] = self.buffer_l[i + 1] + self.coeffs.b_l[i + 1] * audio_in[0]
                - self.coeffs.a_l[i + 1] * y_l;
            self.buffer_r[i] = self.buffer_r[i + 1] + self.coeffs.b_r[i + 1] * audio_in[1]
                - self.coeffs.a_r[i + 1] * y_r;
        }
        for i in 16..31 {
            self.buffer_l[i] = self.buffer_l[i + 1] + audio_in[0] * self.coeffs.b_l[i + 1];
            self.buffer_r[i] = self.buffer_r[i + 1] + audio_in[1] * self.coeffs.b_r[i + 1];
        }
        self.buffer_l[31] = audio_in[0] * self.coeffs.b_l[32];
        self.buffer_r[31] = audio_in[1] * self.coeffs.b_r[32];

        [y_l, y_r]
    }

    pub fn flush(&mut self) {
        self.buffer_l.iter_mut().for_each(|x| *x = 0.0f32);
        self.buffer_r.iter_mut().for_each(|x| *x = 0.0f32);
    }
}

#[derive(Debug, Clone)]
pub struct HRTFFilterIIRCoefficients {
    pub b_l: [f32; 33],
    pub a_l: [f32; 17],
    pub itd_delay_l: f32,
    pub b_r: [f32; 33],
    pub a_r: [f32; 17],
    pub itd_delay_r: f32,
}

impl Default for HRTFFilterIIRCoefficients {
    fn default() -> Self {
        let mut b_l = [0.0; 33];
        b_l[0] = 1.0;
        let mut a_l = [0.0; 17];
        a_l[0] = 1.0; // just for convenience
        let mut b_r = [0.0; 33];
        b_r[0] = 1.0;
        let mut a_r = [0.0; 17];
        a_r[0] = 1.0; // just for convenience
        Self {
            b_l,
            a_l,
            b_r,
            a_r,
            itd_delay_l: 0.0,
            itd_delay_r: 0.0,
        }
    }
}

impl HRTFFilterIIRCoefficients {
    pub fn new(
        a_coeffs_l: &[f32],
        b_coeffs_l: &[f32],
        a_coeffs_r: &[f32],
        b_coeffs_r: &[f32],
        itd_delay_l: f32,
        itd_delay_r: f32,
    ) -> HRTFFilterIIRCoefficients {
        let mut filter_coeffs = HRTFFilterIIRCoefficients::default();
        filter_coeffs.a_l.copy_from_slice(a_coeffs_l);
        filter_coeffs.b_l.copy_from_slice(b_coeffs_l);
        filter_coeffs.a_r.copy_from_slice(a_coeffs_r);
        filter_coeffs.b_r.copy_from_slice(b_coeffs_r);
        filter_coeffs.itd_delay_l = itd_delay_l;
        filter_coeffs.itd_delay_r = itd_delay_r;
        filter_coeffs
    }
    pub fn update_coeffs(&mut self, new_coeffs: &HRTFFilterIIRCoefficients) {
        self.a_l.copy_from_slice(&new_coeffs.a_l);
        self.b_l.copy_from_slice(&new_coeffs.b_l);
        self.a_r.copy_from_slice(&new_coeffs.a_r);
        self.b_r.copy_from_slice(&new_coeffs.b_r);
        self.itd_delay_l = new_coeffs.itd_delay_l;
        self.itd_delay_r = new_coeffs.itd_delay_r;
    }
}

#[derive(Debug, Clone)]
pub struct HrtfProcessorIIR {
    left_delay: CircularBuffer,
    right_delay: CircularBuffer,
    left_delay_old: CircularBuffer,
    right_delay_old: CircularBuffer,
    // left_delay: CBuf,
    // right_delay: CBuf,
    // left_delay_old: CBuf,
    // right_delay_old: CBuf,
    pub hrir_iir: HrtfFilterIIR,
    pub hrir_iir_old: HrtfFilterIIR,
}

impl HrtfProcessorIIR {
    pub fn new() -> Self {
        Self {
            left_delay: CircularBuffer::new(32, 0.0),
            right_delay: CircularBuffer::new(32, 0.0),
            left_delay_old: CircularBuffer::new(32, 0.0),
            right_delay_old: CircularBuffer::new(32, 0.0),
            hrir_iir: HrtfFilterIIR::default(),
            hrir_iir_old: HrtfFilterIIR::default(),
        }
    }
    pub fn update(&mut self, new_coeffs: &HRTFFilterIIRCoefficients) {
        self.hrir_iir_old = self.hrir_iir.clone(); // coeffs.update_coeffs(&new_coeffs);
        self.hrir_iir.flush(); // maybe
        self.hrir_iir.coeffs = new_coeffs.clone();
        self.left_delay
            .set_delay_time(self.hrir_iir.coeffs.itd_delay_l);
        self.right_delay
            .set_delay_time(self.hrir_iir.coeffs.itd_delay_r);
        self.left_delay_old
            .set_delay_time(self.hrir_iir_old.coeffs.itd_delay_l);
        self.right_delay_old
            .set_delay_time(self.hrir_iir_old.coeffs.itd_delay_r);
    }
    pub fn process(
        &mut self,
        audio_in: f32,
        crossfade_in_val: f32,
        crossfade_out_val: f32,
        out: &mut [f32],
    ) {
        let l = self.left_delay.process(audio_in);
        let r = self.right_delay.process(audio_in);
        let l_old = self.left_delay_old.process(audio_in);
        let r_old = self.right_delay_old.process(audio_in);
        let temp = self.hrir_iir.process(&[l, r]); //* crossfade_in_val;
        let temp2 = self.hrir_iir_old.process(&[l_old, r_old]); // * crossfade_out_val;
                                                                // r = self.hrir_iir_l.process(r) * crossfade_in_val;
                                                                // r += self.hrir_iir_l_old.process(r_old) * crossfade_out_val;
                                                                // [temp[0]*crossfade_in_val+temp2[0]*crossfade_out_val,temp[1]*crossfade_in_val+temp2[1]*crossfade_out_val]
        out[0] = temp[0] * crossfade_in_val + temp2[0] * crossfade_out_val;
        out[1] = temp[1] * crossfade_in_val + temp2[1] * crossfade_out_val;
    }
    pub fn process_add(
        &mut self,
        audio_in: f32,
        crossfade_in_val: f32,
        crossfade_out_val: f32,
        out: &mut [f32],
    ) {
        let l = self.left_delay.process(audio_in);
        let r = self.right_delay.process(audio_in);
        let l_old = self.left_delay_old.process(audio_in);
        let r_old = self.right_delay_old.process(audio_in);
        let temp = self.hrir_iir.process(&[l, r]); //* crossfade_in_val;
        let temp2 = self.hrir_iir_old.process(&[l_old, r_old]); // * crossfade_out_val;
                                                                // r = self.hrir_iir_l.process(r) * crossfade_in_val;
                                                                // r += self.hrir_iir_l_old.process(r_old) * crossfade_out_val;
                                                                // [temp[0]*crossfade_in_val+temp2[0]*crossfade_out_val,temp[1]*crossfade_in_val+temp2[1]*crossfade_out_val]
        out[0] += temp[0] * crossfade_in_val + temp2[0] * crossfade_out_val;
        out[1] += temp[1] * crossfade_in_val + temp2[1] * crossfade_out_val;
    }
}

pub fn proc_tpdf2(ina: f32, b: &[f32], a: &[f32], buf: &mut [f32]) -> f32 {
    //  let b = [0.0;33];
    //  let a = [0.0;17];
    //  let mut buf = vec![0.0;32];
    //  let ina = 0.0;
    let y = b[0] * ina + buf[0];
    for i in 0..16 {
        buf[i] = buf[i + 1] + b[i + 1] * ina - a[i + 1] * y;
    }
    for i in 16..31 {
        buf[i] = buf[i + 1] + b[i + 1] * ina;
    }
    buf[31] = b[32] * ina;
    y
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_filter() {
        let b = vec![2.0, 1.0, 3.2, -0.54, 2.4];
        let a = vec![1.0, 1.1, 0.6, -0.5];
        let filter_config = super::IIRFilterCoefficients::new(b, a);
        let mut rb = super::IIRFilter::from_filtercoefficients(&filter_config);

        let x: Vec<f32> = vec![1.0, 0.5, -0.7, 1.2, 1.0];
        let mut y: Vec<f32> = vec![0f32; x.len()];
        for (i, d) in x.iter().enumerate() {
            y[i] = rb.tick(*d);
        }
        println!("{:?}", y);
        //assert!((y[2] - 3.32f64).abs() < 0.00001)
    }
    #[test]
    fn run_small_filter() {
        let b = vec![2.0];
        let a = vec![1.0, 1.1];
        let filter_config = super::IIRFilterCoefficients::new(b, a);
        let mut rb = super::IIRFilter::from_filtercoefficients(&filter_config);

        let x = vec![1.0, 0.0, 0.0, 0.0, 0.0];
        let mut y = vec![0f32; x.len()];
        for (i, d) in x.iter().enumerate() {
            y[i] = rb.tick(*d);
        }
        println!("{:?}", y);
    }
    #[test]
    fn test_filter_perform() {
        let b = vec![2.0, 1.0, 3.2, 6.7, 5.4, 2.1, 0.3, 1.3];
        let a = vec![1.0, 1.1, 0.6, 1.0, 1.1, 0.6, 1.0, 1.1, 0.6];
        let filter_config = super::IIRFilterCoefficients::new(b, a);
        let mut rb = super::IIRFilter::from_filtercoefficients(&filter_config);

        let mut x = vec![0.0; 300_000_000];
        x[0] = 1.0;
        let mut y = vec![0f64; x.len()];
    }
    #[test]
    fn test_slice() {
        let mut a = vec![1.2, 3.0];
        a = a[0..a.len() - 1].to_vec();
        println!("{:?}", a);
    }
    #[test]
    fn z() {
        let z = vec![1, 2, 3];
        z[1..3]
            .iter()
            .enumerate()
            .for_each(|(i, a)| println!("{}, {}", i, a));
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
    fn test_loop() {
        println!("start!");
        for i in 0..0 {
            println!("{}", i);
        }
        print!("finished");
    }
    #[test]
    fn test_hrtf_iir() {}
}
