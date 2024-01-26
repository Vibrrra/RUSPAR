use biquad::{Biquad, DirectForm2Transposed, Coefficients};

use crate::buffers::CircularDelayBuffer;

#[derive(Debug, Clone)]
pub struct DelayLine {
    pub delayline: CircularDelayBuffer,
    // air_absorption_filter
    air_absorption_filter: DirectForm2Transposed<f32>,
    air_absorption_coeff: f32,
}

impl DelayLine {
    pub fn new(delay_line_length: usize) -> Self {
        let air_absorption_coeff = getAirAttenuationCoeffFromDistance(0.0);
        let coeffs = Coefficients { a1: 0.0, a2: 0.0, b0: 0.0, b1: 1.0, b2: 0.0 };
        let air_absorption_filter = DirectForm2Transposed::<f32>::new(coeffs);
        Self {
            delayline: CircularDelayBuffer::new(delay_line_length),
            air_absorption_coeff,
            air_absorption_filter
        }
    }

    pub fn process(&mut self, sample_in: f32) -> f32 {
        let out = self.air_absorption_filter.run(self.delayline.read());
        self.delayline.write(sample_in);
        out
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
    }

}

#[inline(always)]
fn getAirAttenuationCoeffFromDistance(distance: f32) -> f32 {
	0.2 * ((distance/3.0) + 1.0).ln()
}