use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FrameCount, FromSample, Sample, SizedSample,
};
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::thread;

use crate::{
    audioSceneHandlerData::Scene_data,
    convolver::Spatializer,
    fdn::{self, FeedbackDelayNetwork},
    filter::{BinauralFilter, FFTManager, FilterStorage, FilterTree},
    image_source_method::ISMAcousticScene, buffers::CircularDelayBuffer, server::IsmMetaData,
};

//pub fn start_audio_thread(acoustic_scene: Arc<Mutex<ISMAcousticScene>>) {
pub fn start_audio_thread(meta_data: Arc<Mutex<Vec<IsmMetaData>>>) {
    //pub fn start_audio_thread(scene_data: Arc<Mutex<ISMAcousticScene>>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let output_device = host.default_output_device().unwrap();
        let output_config = output_device.default_output_config().unwrap();

        let audio_thread_result = match output_config.sample_format() {
            cpal::SampleFormat::I8 => {
                run::<i8>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::I16 => {
                run::<i16>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::I32 => {
                run::<i32>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::I64 => {
                run::<i64>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::U8 => {
                run::<u8>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::U16 => {
                run::<u16>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::U32 => {
                run::<u32>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::U64 => {
                run::<u64>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::F32 => {
                run::<f32>(&output_device, &output_config.into(), meta_data)
            }
            cpal::SampleFormat::F64 => {
                run::<f64>(&output_device, &output_config.into(), meta_data)
            }
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };

        audio_thread_result
    });
}

fn run<T>(
    devcice: &cpal::Device,
    config: &cpal::StreamConfig,
    meta_data: Arc<Mutex<Vec<IsmMetaData>>>,
) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let buffer_size = 512;
    let error_callback = |err| eprintln!("Error occured on stream: {}", err);

    let filterpath: &str = "./assets/hrtf_binaray.dat";
    let anglepath: &str = "./assets/angles.dat";
    // initialize Engine here
    let mut fft_manager = FFTManager::new(512);
    let (hrtf_storage, hrtf_tree) =
        FilterStorage::new(filterpath, anglepath, &mut fft_manager, buffer_size);

    let mut spatializer = Spatializer::new(buffer_size, fft_manager, &hrtf_storage);
    let prev_hrtfs: Vec<&BinauralFilter> = Vec::new();
    let active_hrtfs: Vec<&BinauralFilter> = Vec::new();
    let mut n_sources = 0;

    // let mut audio_scene = ISMAcousticScene::default();
    let ism_order = 2;
    let speed_of_sound = 343.0;
    let ism_buffer_len = unsafe {
     (sample_rate * 15.0 / speed_of_sound ).to_int_unchecked()    
    };


    let mut ism_buffers = vec![CircularDelayBuffer::new(ism_buffer_len);36];
     
    // let fdn = FeedbackDelayNetwork::new(n_delaylines, )
    // Create Stream
    let stream = devcice.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // read audio for every obejct.
            // collect
            if let Ok(ism_data_vector) = meta_data.try_lock() {
                ism_buffers[0].set_delay_time_samples(sample_rate * ism_data_vector[0].dist / speed_of_sound);
                // set air absoprtion 
            }
            //} else {
            
            //};
            for i in 0..n_sources {
                // calc image n_sources
                //todo!();
                //spatializer.process(input, data, active_hrtfs[i], prev_hrtfs[i]);
                //audio_process(data);
            }
        },
        error_callback,
        None,
    )?;

    stream.play()?;

    Ok(())
}

fn audio_process<T>(output_buffer: &mut [T])
where
    T: Sample + FromSample<f32>,
{

    todo!();
    // UpdateEngine(scene_data);
}
