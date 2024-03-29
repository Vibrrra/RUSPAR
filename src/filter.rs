//use std::iter::Filter;

use std::{sync::Arc, collections::HashMap, fmt::{Debug, Formatter, self}, 
    io::{self, BufRead, BufReader}, fs::File, path::Path, hash::BuildHasherDefault};

use byteorder::{ReadBytesExt, LittleEndian};
use realfft::{num_complex::Complex, RealFftPlanner, RealToComplex, ComplexToReal};
use kdtree;
use nohash_hasher::NoHashHasher;

use crate::readwav;

#[allow(unused)]
#[derive(Clone)]
pub struct FFTManager {
    fft_length: usize,
    real2complex: Arc<dyn RealToComplex<f32>>,
    complex2real: Arc<dyn ComplexToReal<f32>>,
    r2c_scratch_buffer: Vec<Complex<f32>>,
    c2r_scratch_buffer: Vec<Complex<f32>>,

}

impl FFTManager {
    pub fn new(fft_length: usize) -> Self {
        let mut real_planner =  RealFftPlanner::<f32>::new();
        let real2complex = real_planner.plan_fft_forward(fft_length);
        let complex2real = real_planner.plan_fft_inverse(fft_length);
        let r2c_scratch_buffer = real2complex.make_scratch_vec();
        let c2r_scratch_buffer = complex2real.make_scratch_vec();
        Self{
            fft_length, 
            real2complex, 
            complex2real, 
            r2c_scratch_buffer, 
            c2r_scratch_buffer
        }
    }

    #[allow(unused)]
    pub fn transform_to_f_with_scratch(&mut self, input_data: &mut Vec<f32>, output_data: &mut Vec<Complex<f32>>) {
        self.real2complex.process_with_scratch(input_data, output_data, &mut self.r2c_scratch_buffer).unwrap();
    }

    #[allow(unused)]
    pub fn transform_to_t_with_scratch(&mut self, input_data: &mut Vec<Complex<f32>>, output_data: &mut Vec<f32>) {
        self.complex2real.process_with_scratch(input_data, output_data, &mut self.c2r_scratch_buffer).unwrap();
    }

    #[allow(unused)]
    pub fn transform_to_f(&self, input_data: &mut Vec<f32>, output_data: &mut Vec<Complex<f32>>) {
        self.real2complex.process(input_data, output_data);
    }

    #[allow(unused)]
    pub fn transform_to_t(&self, input_data: &mut Vec<Complex<f32>>, output_data: &mut Vec<f32>) {
        self.complex2real.process(input_data, output_data);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinauralFilterType {
    DirectSound,
    EarlyReflection,
    LateReverberation,
}
pub enum MonoFilterType {
    SourceDirectivity,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct BinauralFilter {
    pub data_f_l: Vec<Vec<Complex<f32>>>,
    pub data_f_r: Vec<Vec<Complex<f32>>>,
    filter_type: BinauralFilterType,
    n_segments: usize,
} 

#[allow(unused)]
pub struct MonoFilter {
    pub data_f: Vec<Vec<Complex<f32>>>,
    filter_type: MonoFilterType,
    n_segments: usize,
    data_t_length: usize,
} 

#[allow(unused)]
impl BinauralFilter {
    pub fn from_time_domain(data_t: Vec<Vec<f32>>, fft: &mut FFTManager, filter_type: BinauralFilterType, buffer_size: usize) -> Self {
        
        
        let data_t_length = data_t[0].len();
       
        let mut n_segments = data_t_length / buffer_size;
        let r = data_t_length % buffer_size;
        
        if r > 0 {
            n_segments+=1;
        }

        // check for small filters
        let mut data_t_l = pad_zeros(& data_t[0], r);
        let mut data_t_r= pad_zeros(& data_t[1], r);
    
        if data_t_length < buffer_size {
            data_t_l = pad_zeros(&data_t_l, buffer_size-data_t_length);
            data_t_r = pad_zeros(&data_t_r, buffer_size-data_t_length);
            n_segments = 1;
        }
        
        // 
        let mut data_f_l = vec![vec![Complex::new(0.0, 0.0); fft.fft_length/2 + 1];n_segments];
        let mut data_f_r = vec![vec![Complex::new(0.0, 0.0); fft.fft_length/2 + 1];n_segments];

        for n_seg in 0..n_segments as usize {

            let mut fft_feed_l=  pad_zeros(&data_t_l[n_seg*buffer_size..(n_seg+1)*buffer_size], fft.fft_length-buffer_size) ;
            let mut fft_feed_r=  pad_zeros(&data_t_r[n_seg*buffer_size..(n_seg+1)*buffer_size], fft.fft_length-buffer_size) ;

            fft.real2complex.process(&mut fft_feed_l, &mut data_f_l[n_seg]);
            fft.real2complex.process(&mut fft_feed_r, &mut data_f_r[n_seg]);

            fft.real2complex.process_with_scratch(&mut fft_feed_l, &mut data_f_l[n_seg], &mut fft.r2c_scratch_buffer);
            fft.real2complex.process_with_scratch(&mut fft_feed_r, &mut data_f_r[n_seg], &mut fft.r2c_scratch_buffer);
            for f_bin in 0..data_f_l[n_seg].len() {
                data_f_l[n_seg][f_bin] = data_f_l[n_seg][f_bin] / fft.fft_length as f32;
                data_f_r[n_seg][f_bin] = data_f_r[n_seg][f_bin] / fft.fft_length as f32;
                
            }
        };

        // scaling of fft coefficients
        
        Self { 
            data_f_l,
            data_f_r,
            filter_type,
            n_segments
        }
    }
    pub fn from_wav(filepath: &str, fft: &mut FFTManager, filter_type: BinauralFilterType, buffer_size: usize) -> Self {
        let mut data_t = readwav::readwav_stereo(filepath);
        BinauralFilter::from_time_domain(data_t, fft, filter_type, buffer_size)
    }
    pub fn from_vec(lc: Vec<f32>, rc: Vec<f32>, fft: &mut FFTManager, filter_type: BinauralFilterType, buffer_size: usize) -> Self {
        let mut data_t = vec![lc, rc];
        BinauralFilter::from_time_domain(data_t, fft, filter_type, buffer_size)
    }
    pub fn default(filtertype: BinauralFilterType, fft: &mut FFTManager, buffer_size: usize) -> Self {
                BinauralFilter::from_time_domain(vec![vec![0.0; 256]; 2], fft, filtertype, buffer_size)
    }

    pub fn get_n_segments(&self) -> usize {
        self.n_segments
    }

}


#[allow(unused)]
impl MonoFilter {
    pub fn from_time_domain(data_t: Vec<f32>, fft: &mut FFTManager, filter_type: MonoFilterType, buffer_size: usize) -> Self {
        let data_t_length = data_t.len();

            let mut n_segments = data_t_length / buffer_size;
            let r = data_t_length % buffer_size;
            
            if r > 0 {
                n_segments+=1;
            }
            // check for small filters
            let mut data_t = pad_zeros(& data_t, r);
            
            if data_t_length < buffer_size {
                data_t = pad_zeros(&data_t, buffer_size-data_t_length);
                n_segments = 1;
            }
            
            // 
            let mut data_f = vec![vec![Complex::new(0.0, 0.0); fft.fft_length/2 + 1];n_segments];
            
            for n_seg in 0..n_segments as usize {

                let mut fft_feed=  pad_zeros(&data_t[n_seg*buffer_size..(n_seg+1)*buffer_size], fft.fft_length-buffer_size) ;
                fft.real2complex.process(&mut fft_feed, &mut data_f[n_seg]);
                fft.real2complex.process_with_scratch(&mut fft_feed, &mut data_f[n_seg], &mut fft.r2c_scratch_buffer);
                for f_bin in 0..data_f[n_seg].len() {
                    data_f[n_seg][f_bin] = data_f[n_seg][f_bin] / fft.fft_length as f32;
                }
            };

            // scaling of fft coefficients
            Self { 
                data_f,
                filter_type,
                n_segments,
                data_t_length,
            }
    }

    pub fn from_wav(filepath: &str, fft: &mut FFTManager, filter_type: MonoFilterType, buffer_size: usize) -> Self {
        let mut data_t = readwav::readwav_mono(filepath);
        MonoFilter::from_time_domain(data_t, fft, filter_type, buffer_size)
    }

    pub fn get_n_segments(&self) -> usize {
        self.n_segments
    }

    pub fn default(filter_type: MonoFilterType, buffer_size: usize, n_segments: usize) -> Self {
        MonoFilter {
            data_f: vec![vec![Complex::new(0.0, 0.0); buffer_size+1]; n_segments], 
            filter_type, 
            n_segments,
            data_t_length: buffer_size*n_segments
        }
    }

    pub fn get_t_len(&self) -> usize {
        self.data_t_length
    }
}

#[derive(Copy, Clone)]
pub enum ControlType {
    GAMEPAD,
    OSC,
    AUTO,
    NONE
}

impl Debug for ControlType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ControlType::GAMEPAD => write!(f,"Gamepad"),
            ControlType::OSC     => write!(f,"OSC"),
            ControlType::AUTO    => write!(f,"Auto function"),
            ControlType::NONE    => write!(f,"None"),
        }
    }  
}


pub struct FilterStorage {
    storage: HashMap<usize, BinauralFilter,BuildHasherDefault<NoHashHasher<usize>>>,
    available: bool,
}

#[allow(unused)]
pub struct FilterTree {
    angles: kdtree::KdTree<f32, usize, [f32; 2]>,
}

#[allow(unused)]
impl FilterStorage {
    pub fn new(filterpath: &str, anglepath: &str, fft: &mut FFTManager, blocksize: usize) -> (Self, FilterTree) {
        
        let mut angles: kdtree::KdTree<f32, usize, [f32; 2]> = kdtree::KdTree::new(2);
        let mut storage: HashMap<usize, BinauralFilter, BuildHasherDefault<NoHashHasher<usize>>> = HashMap::with_hasher(BuildHasherDefault::default());// HashMap::new();
        
        
        let mut filter_buf_reader: BufReader<File> = FilterStorage::read_f32_from_binary(filterpath);
        let mut angles_buf_reader: BufReader<File> = FilterStorage::read_f32_from_binary(anglepath);
            let mut id: usize = 1;
        for _ in 0..2558 {
            let mut left_channel: Vec<f32> = Vec::with_capacity(384);
            let mut right_channel: Vec<f32> = Vec::with_capacity(384);
            let mut azel: [f32; 2] = [0.0f32, 0.0f32]; 
            azel[0] = angles_buf_reader.read_f32::<LittleEndian>().unwrap();
            azel[1] = angles_buf_reader.read_f32::<LittleEndian>().unwrap();
            for _ in 0..384 {
                left_channel.push(filter_buf_reader.read_f32::<LittleEndian>().unwrap());
            }
            let mut right_channel = Vec::new();
            for _ in 0..384 {
                right_channel.push(filter_buf_reader.read_f32::<LittleEndian>().unwrap());
            }
            let binaural_filter: BinauralFilter = BinauralFilter::from_vec(left_channel, right_channel, fft, BinauralFilterType::DirectSound, blocksize);
            
            angles.add(azel, id);
            storage.insert(id, binaural_filter);
        }
        
        let available: bool = !storage.is_empty();
        

        // insert default filters
        angles.add([666.0, 420.0], 0);
    
  
        (Self {
             storage: storage, available: available
            },
        FilterTree {
            angles: angles
        })
    }
    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }  
    fn read_f32_from_binary(filename: &str) -> BufReader<File> {
        let file = File::open(filename).unwrap();
        let reader = io::BufReader::new(file);
        reader 
    }

    pub fn get_binaural_filter(&self, filter_type: BinauralFilterType, id: usize) -> &BinauralFilter{
               self.storage.get(&id).unwrap()
         
    }

    pub fn get_n_stereo_segments(&self, filter_type: BinauralFilterType) -> usize {    
                self.storage.values().next().unwrap().get_n_segments()
    }
}


impl FilterTree {
    pub fn find_closest_stereo_filter_angle(&self, filter_type: BinauralFilterType, azimuth: f32, elevation: f32) -> usize {
        let id: usize = *self.angles.nearest(&[azimuth, elevation], 1, &kdtree::distance::squared_euclidean).unwrap()[0].1; //[1].1                                            
        id
    }
    
}
// hlper functions
fn pad_zeros(vector: &[f32], n: usize) -> Vec<f32> {
    let length = vector.len() + n;
    let new_length= if vector.len() >= length  {
        vector.len()
    } else {
        length
    };
    let mut new_values: Vec<f32> = vec![0.0; new_length];
    new_values[0..vector.len()].copy_from_slice(vector);
    new_values
}

#[allow(unused)]
pub fn impulse(length: usize) -> Vec<f32> {
    let mut impulse = vec![0.0f32; length];
    impulse[0] = 2.0;
    impulse
}

#[cfg(test)]
#[test]

#[allow(unused)]
fn test_manager() {
    let mut input_data = vec![0.0; 32];
    let fft_length = 32;
    let mut output_data = vec![Complex::new(0.0, 0.0); fft_length/2+1];
    let mut fftmanager = FFTManager::new(32);
    
    fftmanager.real2complex.process_with_scratch(&mut input_data, &mut output_data,&mut fftmanager.r2c_scratch_buffer);
        
}

#[test]
fn test_from_time_domain() {
    let mut input_t = vec![vec![0.0; 16];2];
    input_t[0][0] = 1.0; 
    input_t[0][1] = 0.5; 
    input_t[1][0] = 1.0;
    input_t[1][1] = -0.5;
    let buffer_size: usize = 8;
    let fft_length = buffer_size*2;

    let mut fft_manager = FFTManager::new(fft_length);
    let binaural_filter = BinauralFilter::from_time_domain(input_t, &mut fft_manager, BinauralFilterType::DirectSound, buffer_size);
    println!("{:?}", &binaural_filter)
}

#[test]
fn test_with_wav() {
    //let path = "./assets/test.wav";
    let path  = r"D:\Programming\RUST\blubb\assets\test.wav";
    
    let buffer_size: usize = 8;
    let fft_length = buffer_size*2;

    let mut fft_manager = FFTManager::new(fft_length);
    let binaural_filter = BinauralFilter::from_wav(path, &mut fft_manager, BinauralFilterType::DirectSound, buffer_size);
    println!("{:?}", &binaural_filter)
}

