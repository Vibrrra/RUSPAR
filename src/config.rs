
// Audio Device
// pub static TARGET_AUDIO_DEVICE: &str = "Komplete Audio 6";
pub static TARGET_AUDIO_DEVICE: &str = "StudioLive AR ASIO";
pub static SAMPLE_RATE: u32 = 48000;
pub static BUFFER_SIZE_CONF: u32 = 128;
// Config structure
pub static IMAGE_SOURCE_METHOD_ORDER: usize = 2;
pub const C: f32 = 343.0;
pub const MAX_SOURCES: usize = 1;

// Audio files for testing
// pub static audio_file_list: [&str; 10] = [r"C:\Users\cschneiderwind\Documents\MATLAB\Test signals\vogeljagd_long_150_16000_normal.wav",
                                        //   "1","2","3","4","5","6","7","8","9"]; 
pub static audio_file_list: [&str; 10] = [r"E:\RustProjects\Assets\_Stimuli__speech__HarvardMale.wav",
                                          "1","2","3","4","5","6","7","8","9"]; 

// files for debugging release build dependency assets
pub static rel_angles: &str = r"E:\RustProjects\RUSPAR\target\release\assets\angles.dat";
pub static rel_hrtfs: &str = r"E:\RustProjects\RUSPAR\target\release\assets\hrtf_binary.dat";




#[cfg(test)]
#[test]

fn read_angles() {
    use std::{iter::Filter, path::Path};

    use crate::filter::{FFTManager, FilterStorage};

    let mut fftm = FFTManager::new(256);
    let (fst,ftt) = FilterStorage::new(Path::new(rel_hrtfs), Path::new(rel_angles),  &mut fftm, 128);

    let az: f32 = 90.0;
    let el: f32 = 90.0;

    let id = ftt.find_closest_stereo_filter_angle(az,el);
    println!("{id}");
}