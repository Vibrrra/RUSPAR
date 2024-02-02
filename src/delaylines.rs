use biquad::{Biquad, DirectForm2Transposed, Coefficients};

use crate::buffers::CircularDelayBuffer;

#[derive(Debug, Clone)]
pub struct DelayLine {
    pub buffer: CBuf,//CircularDelayBuffer,
    // air_absorption_filter
    air_absorption_filter: DirectForm2Transposed<f32>,
    air_absorption_coeff: f32,
}

impl DelayLine {
    pub fn new(delay_line_length: usize) -> Self {
        let air_absorption_coeff = getAirAttenuationCoeffFromDistance(0.0);
        let coeffs = Coefficients { a1: 0.0, a2: 0.0, b0: 1.0, b1: 0.0, b2: 0.0 };
        let air_absorption_filter = DirectForm2Transposed::<f32>::new(coeffs);
        Self {
            buffer: CBuf::new(delay_line_length, 0.0),//CircularDelayBuffer::new(delay_line_length),
            air_absorption_coeff,
            air_absorption_filter
        }
    }

    pub fn process(&mut self, sample_in: f32) -> f32 {
        // let out = self.air_absorption_filter.run(self.delayline.read());
        // let out = self.buffer.read();
        let out = self.buffer.process(sample_in);
        // self.buffer.write(sample_in);
        out
    }

    pub fn process_block(&mut self, audio_in: &[f32], audio_out: &mut [f32]) {
        audio_in.iter().zip(audio_out.iter_mut()).for_each(|(i,o)| {
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
        let coeffs: Coefficients<f32> = Coefficients{ 
            a1: -self.air_absorption_coeff, 
            a2: 0.0, 
            b0: 1.0 - self.air_absorption_coeff, 
            b1: 0.0, 
            b2: 0.0 };
            self.air_absorption_filter.update_coefficients(coeffs);
    }

}

#[inline(always)]
fn getAirAttenuationCoeffFromDistance(distance: f32) -> f32 {
	0.2 * ((distance/3.0) + 1.0).ln()
}


#[derive(Debug, Clone)]
pub struct CBuf {
    buffer: Vec<f32>,
    read_pos: usize,
    write_pos: usize,
    interp_pos: f32,
    delay_time: f32,
}

impl CBuf {
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
        let output = self.interpolate() * (1.0 - self.interp_pos) +
                     self.interpolate_next() * self.interp_pos;
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        self.read_pos = (self.write_pos + self.buffer.len() -
                         (self.delay_time.floor() as usize)) % self.buffer.len();
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


#[cfg(test)]
#[test]

fn test_delay_line() {
    use std::vec;

    let mut cb = DelayLine::new(5);
    let signal: Vec<f32> = vec![1.0,2.0,3.0,4.0,5.0];
    // cb.buffer.set_delay_time_samples(3.0);
    cb.buffer.set_delay_time(3.0);// set_delay_time_samples(3.0);

    let mut o = vec![0.0; signal.len()];
    cb.process_block(&signal, &mut o);
       
        print!("{:#?} ", o)
    

}