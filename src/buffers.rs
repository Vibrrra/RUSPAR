use bit_mask_ring_buf::{self, next_pow_of_2, BMRingBuf};
use num_traits::Float;

#[derive(Debug, Clone)]
pub struct CircularDelayBuffer {
    rb: BMRingBuf<f32>,
    wp: isize,
    rp: f32,
}

impl CircularDelayBuffer {
    pub fn new(length: usize) -> Self {
        let len = next_pow_of_2(length);
        Self {
            rb: BMRingBuf::<f32>::from_len(len),
            wp: 0,
            rp: 1.0,
        }
    }
    pub fn write(&mut self, sample: f32) {
        self.rb[self.wp] = sample;
        self.wp += 1;
    }
    pub fn read(&mut self) -> f32 {
        self.rp += 1.0;
        self.rb.lin_interp_f32(self.rp - 1.0)
    }

    #[allow(unused)]
    fn read_lin_interp(&self, index: f32) -> f32 {
        self.rb.lin_interp_f32(index)
    }
    pub fn set_delay_time(&mut self, delay_in_samples: f32) {
        self.rp = self.wp as f32 - delay_in_samples;
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let out = self.read();
        self.write(sample);
        out
    }
    pub fn process_block(&mut self, audio_in: &[f32], output: &mut [f32]) {
        audio_in.iter().zip(output.iter_mut()).for_each(|(i, o)| {
            *o = self.process(*i);
        })
    }
    pub fn set_delay_time_ms(&mut self, delay_time: f32, sample_rate: f32) {
        let mut delay_time_in_samples = delay_time / 1000.0 * sample_rate;
        if delay_time_in_samples > self.rb.capacity() as f32 {
            delay_time_in_samples = self.wp as f32 + 1.0;
        }
        self.set_delay_time(delay_time_in_samples);
    }
    pub fn read_chunk(&mut self, n: usize, out: &mut [f32]) {
        out.iter_mut().for_each(|y| {
            *y = self.read();
        })
    }
}

pub struct StereoBuffer {
    l_buffer: CircularDelayBuffer,
    r_buffer: CircularDelayBuffer,
}

impl StereoBuffer {
    pub fn new(length: usize) -> Self {
        Self {
            l_buffer: CircularDelayBuffer::new(length),
            r_buffer: CircularDelayBuffer::new(length),
        }
    }
    pub fn write_stereo_slice(&mut self, stereo_slice: &mut [f32; 2]) {
        self.l_buffer.write(stereo_slice[0]);
        self.r_buffer.write(stereo_slice[1]);
    }

    pub fn read(&mut self) -> (f32, f32) {
        (self.l_buffer.read(), self.r_buffer.read())
    }

    pub fn set_delay_time_ms(&mut self, delay_time: f32, sample_rate: f32) {
        let mut delay_time_in_samples = delay_time / 1000.0 * sample_rate;
        if delay_time_in_samples > self.l_buffer.rb.capacity() as f32 {
            delay_time_in_samples = self.l_buffer.wp as f32 + 1.0;
        }
        self.l_buffer.rp = delay_time_in_samples;
        self.r_buffer.rp = delay_time_in_samples;
    }
}

struct CBuffer {
    length: usize,
    buffer: Vec<f32>,
}
// pub fn lin_interp_f32(&self, index: f32) -> f32 {
//     let index_floor = index.floor();
//     let fract = index - index_floor;
//     let index_isize = index_floor as isize;

//     let val_1 = self[index_isize];
//     let val_2 = self[index_isize + 1];

//     val_1 + ((val_2 - val_1) * fract)
// }

#[cfg(test)]
#[test]
fn test_lin_intp() {
    let mut buf = CircularDelayBuffer::new(5);
    for i in 0..buf.rb.len() {
        buf.write(i as f32);
    }

    println!("{}", buf.read_lin_interp(-61.1));
}
