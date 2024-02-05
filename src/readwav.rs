use bit_mask_ring_buf::BMRingBuf;
use hound::{Sample, WavReader};
use num_traits::{Float, FromPrimitive, ToPrimitive};

use std::{fs::File, io::BufReader, marker::PhantomData, path::Path, vec};

use crate::buffers::CircularDelayBuffer;

pub fn readwav_stereo(path: &str) -> Vec<Vec<f32>> {
    let reader: Result<WavReader<BufReader<File>>, hound::Error> = WavReader::open(Path::new(path));
    let mut reader: WavReader<BufReader<File>> = match reader {
        Ok(file) => file,
        Err(_) => panic!("cannot find file {:?}", path),
    };
    let bit_depth: u16 = reader.spec().bits_per_sample;
    let sf: hound::SampleFormat = reader.spec().sample_format;
    let channels: usize = reader.spec().channels as usize;
    let num_samples: usize = reader.duration() as usize;
    let mut wavfile: Vec<Vec<f32>> = vec![vec![0.0; num_samples]; channels];
    let max_val: f32 = (2.0f32).powf(bit_depth as f32 - 1.0);
    match sf {
        hound::SampleFormat::Int => {
            for idx in 0..num_samples {
                for ch in 0..channels {
                    let sample: Result<i32, hound::Error> = reader.samples::<i32>().next().unwrap();
                    wavfile[ch][idx] = sample.unwrap() as f32 / max_val;
                }
            }
        }
        hound::SampleFormat::Float => {
            for idx in 0..num_samples {
                for ch in 0..channels {
                    let sample: Result<f32, hound::Error> = reader.samples::<f32>().next().unwrap();
                    wavfile[ch][idx] = sample.unwrap() as f32;
                }
            }
        }
    }
    wavfile
}
pub fn readwav_mono(path: &str) -> Vec<f32> {
    let reader: Result<WavReader<std::io::BufReader<std::fs::File>>, hound::Error> =
        WavReader::open(Path::new(path));
    let mut reader = match reader {
        Ok(file) => file,
        Err(_) => panic!("cannot find file {:?}", path),
    };
    let bit_depth: u16 = reader.spec().bits_per_sample;

    let sf = reader.spec().sample_format;
    let channels = reader.spec().channels as usize;
    assert!(
        channels == 1,
        "This audio file is supposed to have only 1 channel"
    );
    let num_samples = reader.duration() as usize;
    let mut wavfile: Vec<f32> = vec![0.0; num_samples];
    let max_val = (2.0f32).powf(bit_depth as f32 - 1.0);
    match sf {
        hound::SampleFormat::Int => {
            for idx in 0..num_samples {
                for _ch in 0..channels {
                    let sample = reader.samples::<i32>().next().unwrap();
                    wavfile[idx] = sample.unwrap() as f32 / max_val;
                }
            }
        }
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

#[allow(unused)]
pub struct AudioFileManager {
    file_path: String,
    wav_reader: WavReader<BufReader<File>>,
    bit_depth: u16,                     // reader.spec().bits_per_sample;
    sample_format: hound::SampleFormat, // reader.spec().sample_format;
    channels: usize,                    // reader.spec().channels as usize;
    num_samples: usize,
    max_val: f32,
    pub buffer: CircularDelayBuffer, // (2.0f32).powf(bit_depth as f32 - 1.0) ;
}

impl AudioFileManager {
    pub fn new(file_path: String, buffer_size: usize) -> AudioFileManager {
        let mut wav_reader = match WavReader::open(Path::new(file_path.as_str())) {
            Ok(wav_reader) => wav_reader,
            Err(_) => panic!("Wav file could not be opened! Path: {:?}", file_path),
        };
        let bit_depth = wav_reader.spec().bits_per_sample;
        let sample_format: hound::SampleFormat = wav_reader.spec().sample_format;
        let channels = wav_reader.spec().channels as usize;
        let num_samples = wav_reader.duration() as usize;

        let max_val: f32 = (2.0).powf(bit_depth as f32 - 1.0);
        let mut buffer = CircularDelayBuffer::new(num_samples);
        match sample_format {
            hound::SampleFormat::Float => wav_reader
                .samples::<f32>()
                .step_by(channels)
                .enumerate()
                .for_each(|(n, sample_result)| match sample_result {
                    Ok(sample) => {
                        buffer.write(sample);
                    }
                    Err(_) => {}
                }),
            hound::SampleFormat::Int => wav_reader
                .samples::<i32>()
                .step_by(channels)
                .enumerate()
                .for_each(|(n, sample_result)| match sample_result {
                    Ok(sample) => {
                        buffer.write(sample as f32 / max_val);
                    }
                    Err(_) => {}
                }),
        }
        AudioFileManager {
            file_path,
            wav_reader,
            bit_depth,
            sample_format,
            channels,
            num_samples,
            max_val,
            buffer,
        }
    }

    pub fn read_n_samples(&mut self, n: usize, out: &mut [f32]) {
        for i in 0..n {
            match self.sample_format {
                hound::SampleFormat::Float => {
                    //self.buffer.iter_mut().for_each(|x| {
                    out[i] = self.read_sample_float();
                }
                hound::SampleFormat::Int => {
                    //self.buffer.iter_mut().for_each(|x| {
                    out[i] = self.read_sample_int();
                }
            }
        }
    }

    fn read_sample_int(&mut self) -> f32 {
        let sample = self.wav_reader.samples::<i32>().next().unwrap().unwrap();
        let sample: f32 = sample as f32 / self.max_val;
        sample
    }
    fn read_sample_float(&mut self) -> f32 {
        let sample = self.wav_reader.samples::<f32>().next().unwrap().unwrap();
        sample
    }
}

#[cfg(test)]
#[test]
fn test_wav_read() {
    let path = "./assets/test.wav";
    let rd = readwav_stereo(path);
    println!("{:?}", rd);
    let max_val = (2.0f32).powf(24.0 - 1 as f32) - 1.0;

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
