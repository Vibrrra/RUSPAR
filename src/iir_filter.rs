//use biquad::{Hertz, Biquad};

// use crate::delay_line::{self, RingBuffer};



#[allow(dead_code)]
enum FilterType {
    FIR,
    IIR
}

#[derive(Clone)]
pub struct IIRFilterCoefficients{
    b: Vec<f32>,
    a: Vec<f32>,
    a0: f32,
    b0: f32,
    b_last: f32,
    a_last: f32,
    order: usize
}


impl IIRFilterCoefficients {
    pub fn new(mut b: Vec<f32>, mut a: Vec<f32>) -> Self {
        
        
        // extract a0 to scale all coefficients if it is != 1.0
        let a0 = a[0];
        let b0: f32 = b[0] /a[0];
        
        
        let num_b = b.len();
        let num_a = a.len();

        if num_a == 1 {
            panic!("More 'a' coefficients needed to be a IIR filter!")
        }
        
        let order = (num_b as i32).max(num_a as i32) as usize -1;
        
        let mut bn = vec![0.0; order-1];
        let mut an: Vec<f32> = vec![0.0; order-1];
        
        
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
pub struct IIRFilter {
    coefficients: IIRFilterCoefficients,
    buffer: Vec<f32>,
}

impl IIRFilter {

    pub fn from_filtercoefficients(coefficients: &IIRFilterCoefficients) -> Self {
        
        let mut buffer = vec![0.0; coefficients.required_internal_buffer() ];
                
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
