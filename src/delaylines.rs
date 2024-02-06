use biquad::{Biquad, Coefficients, DirectForm2Transposed};

use crate::buffers::CircularDelayBuffer;

#[derive(Debug, Clone)]
pub struct DelayLine {
    // pub buffer: CBuf,
    pub buffer: CircularDelayBuffer,
    // air_absorption_filter
    air_absorption_filter: DirectForm2Transposed<f32>,
    air_absorption_coeff: f32,
}

impl DelayLine {
    pub fn new(delay_line_length: usize) -> Self {
        let air_absorption_coeff = getAirAttenuationCoeffFromDistance(0.0);
        let coeffs = Coefficients {
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
        };
        let air_absorption_filter = DirectForm2Transposed::<f32>::new(coeffs);
        Self {
            // buffer: CBuf::new(delay_line_length, 0.0),
            buffer: CircularDelayBuffer::new(delay_line_length),
            air_absorption_coeff,
            air_absorption_filter,
        }
    }

    pub fn process(&mut self, sample_in: f32) -> f32 {
        // let out = self.air_absorption_filter.run(self.delayline.read());
        // let out = self.buffer.read();
        let out = self.buffer.process(sample_in);
        // self.buffer.write(sample_in);
        out
    }
    pub fn process_iir(&mut self, sample_in: f32) -> f32 {
        // let out = self.air_absorption_filter.run(self.delayline.read());
        // let out = self.buffer.read();
        let mut out = self.buffer.process(sample_in);
        // self.buffer.write(sample_in);
        out
    }

    pub fn process_block(&mut self, audio_in: &[f32], audio_out: &mut [f32]) {
        audio_in
            .iter()
            .zip(audio_out.iter_mut())
            .for_each(|(i, o)| {
                *o = self.process(*i);
            })
    }
    pub fn process_block2(&mut self, audioblock: &mut [f32], gain: f32) {
        audioblock.iter_mut().for_each(|i| {
            *i = self.process(*i) * gain;
        })
    }
    // https://github.com/chrisyeoward/ScatteringDelayNetwork/blob/master/Source/Reverb/Delay.cpp#46
    // https://github.com/chrisyeoward/ScatteringDelayNetwork/blob/master/Source/Reverb/Filter.h
    pub fn set_air_absoprtion(&mut self, distance: f32) {
        self.air_absorption_coeff = getAirAttenuationCoeffFromDistance(distance);
        let coeffs: Coefficients<f32> = Coefficients {
            a1: -self.air_absorption_coeff,
            a2: 0.0,
            b0: 1.0 - self.air_absorption_coeff,
            b1: 0.0,
            b2: 0.0,
        };
        self.air_absorption_filter.update_coefficients(coeffs);
    }
}

#[inline(always)]
fn getAirAttenuationCoeffFromDistance(distance: f32) -> f32 {
    0.2 * ((distance / 3.0) + 1.0).ln()
}

#[derive(Debug, Clone)]
pub struct ITDBuffer {
    buffer: Vec<f32>,
    read_pos: usize,
    write_pos: usize,
    interp_pos: f32,
    delay_time: f32,
}


impl ITDBuffer {
    pub fn new(buffer_size: usize, delay_time: f32) -> Self {
        Self {
            buffer: vec![0.0; buffer_size],
            read_pos: 0,
            write_pos: 0,
            interp_pos: 0.0,
            delay_time,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.interpolate() * (1.0 - self.interp_pos)
            + self.interpolate_next() * self.interp_pos;
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        self.read_pos = ((self.write_pos as i32) + (self.buffer.len() as i32)
            - (self.delay_time.floor() as i32))
            .rem_euclid(self.buffer.len() as i32) as usize;

        self.interp_pos = self.delay_time - self.delay_time.floor();
        output
    }

    fn interpolate(&self) -> f32 {
        let x0 = self.buffer[self.read_pos];
        let x1 = self.buffer[(self.read_pos + 1) % self.buffer.len()];
        x0 + (x1 - x0) * self.interp_pos
    }

    fn interpolate_next(&self) -> f32 {
        let x0 = self.buffer[(self.read_pos + 1) % self.buffer.len()];
        let x1 = self.buffer[(self.read_pos + 2) % self.buffer.len()];
        x0 + (x1 - x0) * self.interp_pos
    }

    pub fn set_delay_time(&mut self, delay_time: f32) {
        self.delay_time = delay_time;
    }
}
 #[derive(Debug, Clone)]
pub struct ITDBuffer2 {
    
    // Circular buffer for delay
    buffer_len: f32,
    buffer: Vec<f32>,
    tail : f32,
    head : f32,
    delay_time: f32}

impl ITDBuffer2 {
    pub fn new( max_delay_samples: usize) -> ITDBuffer2 {
        let buffer: Vec<f32> = vec![0.0; max_delay_samples];
        ITDBuffer2 {

            buffer,
            head: 0.0,
            tail: 0.0,
            delay_time: 0.0,
            buffer_len: max_delay_samples as f32,
        }
    }
    pub fn set_delay_time(&mut self, delay_time: f32) {
        self.delay_time = delay_time;
    }
    pub fn read_linear(&mut self,index: f32) -> f32 {
        // Store the input in the delay buffer
        let upper = (index + 1.0).floor();
        let lower = index.floor();
        let interp_amount = index - lower;
        return self.get_sample(upper) * interp_amount + (1.0 - interp_amount) * self.get_sample(lower);   
    }

    pub fn write(&mut self, in_value: f32) {
        self.head = (self.head+1.0) % self.buffer_len;
        self.buffer[self.head as usize] = in_value;
    }

    pub fn get_sample(&mut self, in_value: f32) -> f32 {
        self.tail = self.head - in_value;
        if self.tail > (self.buffer_len -1.0) {
            self.tail -= self.buffer_len ;
        } else if self.tail < 0.0 {
            self. tail+= self.buffer_len;
        }

        return self.buffer[self.tail as usize];
    }
    pub fn process(&mut self, in_value: f32) -> f32 {
        let out = self.read_linear(self.delay_time);
        self.write(in_value);
        out
    }

}
#[cfg(test)]
#[test]

fn test_delay_line() {
    use std::vec;

    let mut cb = DelayLine::new(5);
    let signal: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    // cb.buffer.set_delay_time_samples(3.0);
    cb.buffer.set_delay_time(1.5); // set_delay_time_samples(3.0);

    // let mut o = vec![0.0; signal.len()];
    for i in 0..signal.len() {
        let o = cb.process(signal[i]);

        print!("{:#?} ", o)
    }
    // let o = cb.process(&signal, &mut o);

    // print!("{:#?} ", o)
}


#[test]
fn test_itd_Delay () {
    let x = [1.0, 0f32,0f32,0f32,0f32,0f32,0f32,0f32,0f32];
    let mut itdd = ITDBuffer2::new(5);
    let delay_time = 1.5;

    for i in x {
        itdd.set_delay_time(delay_time);
        let p = itdd.process(  i);
        print!("{} ", p)
    }
}