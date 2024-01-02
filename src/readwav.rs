use hound::{WavReader};
use std::path::Path;

pub fn readwav_stereo(path: &str) -> Vec<Vec<f32>> {
    let reader = WavReader::open(Path::new(path));
    let mut reader = match reader {
        Ok(file) => file,
        Err(_) => panic!("cannot find file {:?}", path)
    };
    let bit_depth = reader.spec().bits_per_sample;
    



    let sf = reader.spec().sample_format;
    let channels = reader.spec().channels as usize;
    let num_samples = reader.duration() as usize;
    let mut wavfile: Vec<Vec<f32>> = vec![vec![0.0; num_samples]; channels];
    let max_val = (2.0f32).powf(bit_depth as f32 - 1.0) ; 
    match sf {
        hound::SampleFormat::Int => {
            for idx in 0..num_samples {
                for ch in 0..channels {
                    let sample = reader.samples::<i32>().next().unwrap();
                    wavfile[ch][idx] = sample.unwrap() as f32/ max_val ; 
                }
            }   
        },
        hound::SampleFormat::Float => {
            for idx in 0..num_samples {
                for ch in 0..channels {
                    let sample = reader.samples::<f32>().next().unwrap();
                    wavfile[ch][idx] = sample.unwrap() as f32; 
                }
            }
        }
    }
    wavfile    
}
pub fn readwav_mono(path: &str) -> Vec<f32> {
    let reader = WavReader::open(Path::new(path));
    let mut reader = match reader {
        Ok(file) => file,
        Err(_) => panic!("cannot find file {:?}", path)
    };
    let bit_depth = reader.spec().bits_per_sample;
    



    let sf = reader.spec().sample_format;
    let channels = reader.spec().channels as usize;
    assert!(channels == 1,"This audio file is supposed to have only 1 channel");
    let num_samples = reader.duration() as usize;
    let mut wavfile: Vec<f32> = vec![0.0; num_samples];
    let max_val = (2.0f32).powf(bit_depth as f32 - 1.0) ; 
    match sf {
        hound::SampleFormat::Int => {
            for idx in 0..num_samples {
                for _ch in 0..channels {
                    let sample = reader.samples::<i32>().next().unwrap();
                    wavfile[idx] = sample.unwrap() as f32/ max_val ; 
                }
            }   
        },
        hound::SampleFormat::Float => {
            for idx in 0..num_samples {
                for _ch in 0..channels {
                    let sample = reader.samples::<f32>().next().unwrap();
                    wavfile[idx] = sample.unwrap() as f32; 
                }
            }
        }
    }
    wavfile    
}


#[cfg(test)]
#[test]
fn test_wav_read() {
    let path = "./assets/test.wav";
    let rd = readwav_stereo(path);
    println!("{:?}",rd);
    let max_val = (2.0f32).powf(24.0-1 as f32) - 1.0; 

    println!("{}", max_val)
}





// fn read_wav_from_string(s: &str) -> Vec<f32> {
//     let mut reader = Cursor::new(s.as_bytes());

//     // Read the WAV header to get the sample rate and number of channels
//     let mut header = [0; 44];
//     reader.read_exact(&mut header).unwrap();
//     let sample_rate = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
//     let num_channels = u16::from_le_bytes([header[22], header[23]]);

//     // Read the audio data as raw bytes
//     let mut data = Vec::new();
//     reader.read_to_end(&mut data).unwrap();

//     // Convert the raw audio data to floating point format
//     let mut result = Vec::new();
//     for i in (0..data.len()).step_by(num_channels as usize * 2) {
//         for j in 0..num_channels as usize{
//             let sample = i + j * 2;
//             let value = i16::from_le_bytes([data[sample], data[sample + 1]]) as f32 / 32768.0;
//             result.push(value);
//         }
//     }

//     result
// }