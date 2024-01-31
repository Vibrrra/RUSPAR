use cpal::{
    self, traits::{DeviceTrait, HostTrait, StreamTrait}, BufferSize, ChannelCount, FromSample, Sample, SampleRate, SizedSample, StreamConfig
};
use indextree::{Arena, NodeId};
use nalgebra::Vector3;
use std::{cell::RefCell, os::windows::process, path::Path, sync::mpsc::Receiver, thread::sleep, time::Duration, vec};
use std::thread;

use crate::{
    assets::{DL_S, A_FDN, B_FDN, A_FDN_TC, B_FDN_TC}, audio_devices::get_output_device, buffers::CircularDelayBuffer, config::{audio_file_list,  C, IMAGE_SOURCE_METHOD_ORDER, MAX_SOURCES, SAMPLE_RATE, TARGET_AUDIO_DEVICE}, convolver::Spatializer, delaylines::{self, DelayLine}, fdn::{FeedbackDelayNetwork, calc_fdn_delayline_lengths, map_ism_to_fdn_channel, FDNInputBuffer, calc_hrtf_sphere_points}, filter::{BinauralFilter, FFTManager, FilterStorage}, image_source_method::{from_source_tree, is_per_model, ISMLine, Room, Source, SourceTrees, SourceType, N_IS_INDEX_RANGES}, ism_test_structure::{ISMDelayLines, IMS, ISM_INDEX_RANGES}, readwav::AudioFileManager
};

pub fn start_audio_thread(rx: Receiver<IMS>, mut sources: IMS, room: Room, BUFFER_SIZE: usize) {
    //pub fn start_audio_thread(scene_data: Arc<Mutex<ISMAcousticScene>>) {
    thread::spawn(move || {

    // Audio host & device configs
    let host = cpal::HostId::Asio;
    let target_device: Option<cpal::Device> = get_output_device(TARGET_AUDIO_DEVICE);
    
    let device = match target_device {
        Some(device) => {device},
        None => panic!{"Target Device not available!"},
    };

    let default_device_config = device.default_output_config().unwrap();

    println!("Host: {:?}", host);
    println!("Device: {:?}", device.name().unwrap());
    println!("Config: {:?}", default_device_config);
    let sample_format = default_device_config.sample_format();

    // hardcoded for now -> should be 
    let stream_config: StreamConfig = StreamConfig {
        channels: 2u16,
        sample_rate: cpal::SampleRate(48000u32),
        buffer_size: BufferSize::Fixed(BUFFER_SIZE as u32),
    };

        
    let audio_thread_result = match sample_format {
        cpal::SampleFormat::I8 => {
            run::<i8>(device, stream_config.into(),  rx, sources, room,BUFFER_SIZE)
        }
        cpal::SampleFormat::I16 => {
            run::<i16>(device,stream_config.into(),  rx, sources, room,BUFFER_SIZE)
        }
        cpal::SampleFormat::I32 => {
            run::<i32>(device,stream_config.into(),  rx, sources, room,BUFFER_SIZE)
        }
        cpal::SampleFormat::I64 => {
            run::<i64>(device,stream_config.into(),  rx, sources, room,BUFFER_SIZE)
        }
        cpal::SampleFormat::U8 => {
            run::<u8>(device, stream_config.into(),  rx, sources, room,BUFFER_SIZE)
        }
        cpal::SampleFormat::U16 => {
            run::<u16>(device, stream_config.into(), rx, sources, room,BUFFER_SIZE)
        }
        cpal::SampleFormat::U32 => {
            run::<u32>(device, stream_config.into(), rx, sources, room,BUFFER_SIZE)
        }
        cpal::SampleFormat::U64 => {
            run::<u64>(device, stream_config.into(), rx, sources, room,BUFFER_SIZE)
        }
        cpal::SampleFormat::F32 => {
            run::<f32>(device, stream_config.into(), rx, sources, room,BUFFER_SIZE)    
        }
        cpal::SampleFormat::F64 => {
            run::<f64>(device, stream_config.into(), rx, sources, room,BUFFER_SIZE)
        }
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    };

    audio_thread_result
    });
}



fn run<T>(
    device: cpal::Device,
    stream_config: cpal::StreamConfig,
    rx: Receiver<IMS>,
    mut source_trees: IMS,
    room: Room,
    BUFFER_SIZE: usize,
) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
// U: SourceType<Source> + Clone + Send + 'static,
{
    let sample_rate = SAMPLE_RATE as f32;
    let buffer_size = BUFFER_SIZE as usize;
    let error_callback = |err| eprintln!("Error occured on stream: {}", err);

    let filterpath: &Path  = Path::new("assets/hrtf_binary.dat");
    let anglepath: &Path = Path::new("assets/angles.dat");

    // Init Spatializer
    let mut fft_manager = FFTManager::new(buffer_size*2);
    let (hrtf_storage, hrtf_tree) =
        FilterStorage::new(filterpath, anglepath, &mut fft_manager, buffer_size);
    let mut spatializer = Spatializer::new(buffer_size, fft_manager, &hrtf_storage); 

    // Create (Image) Source Processing Delay Lines 
    let mut sources = ISMDelayLines::new(source_trees, &room, C, sample_rate, buffer_size, IMAGE_SOURCE_METHOD_ORDER ,spatializer.clone());

    // Create FDN
    // We init some constants for testing
    let fdn_n_dls: usize = 24;
    let delay_line_lengths: Vec<usize>  = DL_S.iter().map(|x| {(x*sample_rate).ceil() as usize}).collect();
    let mut fdn_input_buf: FDNInputBuffer = FDNInputBuffer::new(fdn_n_dls, buffer_size);
    let mut fdn_output_buf: FDNInputBuffer = FDNInputBuffer::new(fdn_n_dls, buffer_size);
    let mut fdn = FeedbackDelayNetwork::from_assets(fdn_n_dls, buffer_size,  delay_line_lengths, B_FDN, A_FDN, B_FDN_TC, A_FDN_TC);

    // Create HRTF spatializer
    let fdn_hrtf_coords = calc_hrtf_sphere_points(24);
    let mut fdn_spatializers: Vec<Spatializer> = Vec::with_capacity(24);
    let mut fdn_curr_hrtf_idx = Vec::new();

    // Init FDN spatializer & HRTF Ids
    for i in 0..24usize {
        fdn_spatializers.push(spatializer.clone());
        let idx = hrtf_tree.find_closest_stereo_filter_angle(fdn_hrtf_coords[i].0, fdn_hrtf_coords[i].1); 
        fdn_curr_hrtf_idx.push(idx);
    }

    // Init AudioFileManager
    let mut audio_file_managers: Vec<AudioFileManager> = Vec::new();
    for i in 0 .. 1{// MAX_SOURCES {
        audio_file_managers.push( AudioFileManager::new(audio_file_list[i].to_string(), buffer_size));
    }
    let mut test_audio_manager = audio_file_managers[0].buffer.clone();
    let mut output_buffers: Vec<Vec<f32>> = vec![vec![0.0f32; buffer_size]; 37];

    let mut audio_temp_buffer = vec![0.0f32; buffer_size];
    // INIT . This loop blocks the current fucntion for 5 secs and waits for 
    // a first update from the server to initialize all variables with sane data
    // loop {
    //     // match rx.recv_timeout(Duration::from_secs(5)) {
    //     match rx.try_recv() {
    //         Ok(data) => {
    //             sources.sources.iter_mut().zip(data.sources.iter()).for_each(|(rev, src)| {
    //                 rev.iter_mut().zip(src.iter()).for_each(|(r, s)|{
    //                     // set delays
    //                     let delaytime = s.get_remaining_dist() / C * 1000f32;
    //                     r.delayline.delayline.set_delay_time_ms(delaytime, sample_rate);
    //                     r.delayline.set_air_absoprtion(s.get_dist());
    //                     let orientation = s.get_lst_src_transform();
    //                     r.new_hrtf_id = hrtf_tree.find_closest_stereo_filter_angle(orientation.azimuth, orientation.elevation);                  
    //                     r.old_hrtf_id = r.new_hrtf_id;
    //                 })
    //             });
    //             break;
    //         },
    //         Err(e) => {
    //             // panic!("Initial receive from server has failed to to timeout: {e}")
    //             sleep(Duration::from_millis(1));
    //         },                
    //     };
    // }
    let mut ism_temp_buffer = vec![0.0f32; buffer_size];
    // Create Stream
    let mut temp_buffer = vec![0.0f32; 2*buffer_size];
    let stream:Result<cpal::Stream, cpal::BuildStreamError> = device.build_output_stream(
        &stream_config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            
            // MAYBE Flushing some buffers here ... 
            temp_buffer.iter_mut().for_each(|x|  {*x = 0.0});

            match rx.try_recv() {
                Ok(data) => {
                    // let o = data.sources[0][0].get_lst_src_transform();
                    // println!("{:?}",o);
                    sources.sources.iter_mut().zip(data.sources.iter()).for_each(|(rev, src)| {
                        rev.iter_mut().zip(src.iter()).for_each(|(r, s)|{
                            // set delays
                            let delaytime = s.get_remaining_dist() / C * 1000f32;
                            r.delayline.delayline.set_delay_time_ms(delaytime, sample_rate);
                            r.delayline.set_air_absoprtion(s.get_dist());
                            let orientation = s.get_lst_src_transform();
                            r.old_hrtf_id = r.new_hrtf_id;
                            let new_id = hrtf_tree.find_closest_stereo_filter_angle(orientation.azimuth, orientation.elevation); 
                            r.new_hrtf_id = new_id;
                            r.dist_gain = 1.0/s.dist;                                          
                        })
                    });    
                // println!("Reveived!");
                }, 
                Err(_) => {},                
            };
            
            // // new algo here
            // // Process Delay Lines
            // sources.sources.iter_mut().enumerate().for_each(|(n, source)| {
            //     // let mut par_src = unsafe { source.get_unchecked_mut(0) };
            //     let fdn_line_index = map_ism_to_fdn_channel(0, 24);
            //     // par_src.output_buffer.iter_mut().zip(fdn_input_buf.buffer[fdn_line_index].iter_mut()).for_each(|(op, fi)| {
            //     // output_buffers[0].iter_mut().zip(fdn_input_buf.buffer[fdn_line_index].iter_mut()).for_each(|(op, fi)| {
            //     //     *op = source[0].delayline.delayline.process(audio_file_managers[n].buffer.read());
            //     //     *fi = *op;
            //     // });
                
            //     let audio_in: Vec<f32> = (0..buffer_size).into_iter().map(|_| audio_file_managers[n].buffer.read()).collect();
            //     source[0].delayline.process_block(&audio_in, &mut output_buffers[0]);
            //     // println!("{:#?}", output_buffers[0]);
            //     for ism_idx in ISM_INDEX_RANGES {
            //         let par_src = &source[ism_idx.0];
            //         for i in ism_idx.1 .. ism_idx.2 {
            //             // let mut child_src = &source[i]; //&mut source[i];
            //             let fdn_line_index = map_ism_to_fdn_channel(i, 24);
            //             let temp = output_buffers[ism_idx.0].clone();
            //             output_buffers[i].iter_mut().zip(fdn_input_buf.buffer[fdn_line_index].iter_mut()).zip(temp.iter()).for_each(|((op, fi),ins)| {
            //                 let gain = source[i].dist_gain;
            //                 *op = source[i].delayline.delayline.process(*ins) * 1.0 / gain;
            //                 *fi = *op;
            //             });
            //         }
            //     }

            //     // all sources
            //     // for (n,src) in source.iter_mut().enumerate() {
            //     //     let new_hrtf = hrtf_storage.get_binaural_filter(src.new_hrtf_id);
            //     //     let old_hrtf = hrtf_storage.get_binaural_filter(src.old_hrtf_id);
            //     //     src.spatializer.process(&output_buffers[n], &mut temp_buffer, new_hrtf, old_hrtf);
            //     // }                                
            //     // proc only one source 
            //     let src = &mut source[0];
            //         let new_hrtf = hrtf_storage.get_binaural_filter(src.new_hrtf_id);
            //         let old_hrtf = hrtf_storage.get_binaural_filter(src.old_hrtf_id);
            //         src.spatializer.process(&output_buffers[0], &mut temp_buffer, new_hrtf, old_hrtf);
                
            // })  ;          

            // Process FDN
            // for i in 0..buffer_size {
            //     fdn.delaylines.iter_mut()
            //                   .zip(fdn.matrix_outputs.iter())
            //                   .zip(fdn.matrix_inputs.iter_mut())
            //                   .zip(fdn_output_buf.buffer.iter_mut())
            //                   .zip(fdn_input_buf.buffer.iter())
            //                   .for_each(|((((fdn_in, mat_out),mat_in),fdn_out), fdn_input_buf)| {
            //         *mat_in = fdn_in.tick(fdn_input_buf[i]+mat_out);
            //         fdn_out[i] = *mat_in;
            //     });
            // }

            // FDN HRTF Processing            
            // fdn_output_buf.buffer.iter_mut().zip(fdn_spatializers.iter_mut()).
            // zip(fdn_curr_hrtf_idx.iter()).for_each(|((fdn_out,fdn_spatializer), id)| {
            //     let hrtf = hrtf_storage.get_binaural_filter(*id);
            //     fdn_spatializer.process(&fdn_out, &mut temp_buffer, hrtf, hrtf);
            // });
            // let mut audio_in: Vec<f32> = (0..buffer_size).into_iter().map(|_| test_audio_manager.read()).collect();
            // let mut src = &mut sources.sources[0][0];
            // let nh = hrtf_storage.get_binaural_filter(src.new_hrtf_id);
            // let oh =hrtf_storage.get_binaural_filter(src.old_hrtf_id);
            // src.spatializer.process(&audio_in, &mut temp_buffer, nh, oh);
            
            sources.sources.iter_mut().take(1).zip(output_buffers.iter_mut()).for_each(|(src, dll_out)| {
                let mut audio_in: Vec<f32> = (0..buffer_size).into_iter().map(|_| test_audio_manager.read() * src[0].dist_gain).collect();
                src.iter_mut().for_each(|s| {
                    // audio_temp_buffer.copy_from_slice(&audio_in);
                    // audio_temp_buffer.iter_mut().for_each(|a| {*a *= s.dist_gain;});
                    s.delayline.delayline.process_block(&audio_in, dll_out);
                    let nh = hrtf_storage.get_binaural_filter(s.new_hrtf_id);
                    let oh =hrtf_storage.get_binaural_filter(s.old_hrtf_id);
                    s.spatializer.process(&dll_out, &mut temp_buffer, nh, oh, s.dist_gain);
                })
            });
          

            // temp_buffer.chunks_mut(2).zip(audio_in.iter()).for_each(|(o,i)| {
            //     o[0] =*i;
            //     o[1] =*i;
            // });

            for (frames, input) in data.chunks_mut(2).zip(temp_buffer.chunks(2)) {
                frames.iter_mut().zip(input.iter()).for_each(|(o,i)| {                    
                    //  0.5 -> hardcoded volume (safety) for now

                    *o = T::from_sample(*i*0.015f32);
                    // if *o > T::from_sample(1.0f32) {
                    //     println!{"clipping!"}
                    //     *o = T::from_sample(0.0f32);
                    // }
                });
            }
           
 
        },
        error_callback,
        None,// Some(Duration::from_millis(5)), //None,
    );
    let stream_res: cpal::Stream = match stream {
        Ok(stream) => stream,
        Err(e) => panic!("ERROR: {e}"),
    };

    let stream_play_res: Result<(), cpal::PlayStreamError> = stream_res.play();
    match stream_play_res {
        Ok(_) => { loop {}},
        Err(e) => {println!("Error opening stream: {:?}", e)},
    };
    println!("Stream terminated!");
    Ok(())
}

#[derive( Clone)]
pub struct ISMDelayLine {
    pub delayline: DelayLine,
    pub output_buffer: Vec<f32>,
    pub spatializer: Spatializer,
    pub new_hrtf_id: usize,
    pub old_hrtf_id: usize,
    pub dist_gain: f32,
}
impl ISMDelayLine {
    pub fn new(delayline_length: usize, buffer_length: usize, spatializer: Spatializer) -> Self {
        ISMDelayLine {delayline: DelayLine::new(delayline_length), 
                    output_buffer: vec![0.0f32; buffer_length].into(),
                     spatializer, new_hrtf_id: 1 , old_hrtf_id: 1, dist_gain: 1.0}
    }
}

// process template
#[allow(unused)]
fn audio_process(output: &mut [f32], renderer: &mut dyn FnMut() -> (Vec<f32>, Vec<f32>)){
    let (mut ism_output, mut fdn_output) = renderer();
    for (frame, ins) in output.chunks_mut(2).zip(ism_output.iter()) {
        
        let to_out_l =  *ins; //+ fdn_out[0];
        let to_out_r =  *ins; //+ fdn_out[0];
        
        if (to_out_l.abs() > 1.0) | (to_out_r.abs() > 1.0)  {
            // println!("Clipping!");
        }
        frame[0] = to_out_l;//r[0] * adjust_loudness(n_sources);
        frame[1] = to_out_r; //r[1] * adjust_loudness(n_sources);
    

    }
}

struct ASS {
    temp: Vec<f32>,
    delayline0: ISMDelayLine,
    delayline1: ISMDelayLine,
    delayline2: ISMDelayLine,
    delayline3: ISMDelayLine,
    delayline4: ISMDelayLine,
    delayline5: ISMDelayLine,
    delayline6: ISMDelayLine,
    delayline7: ISMDelayLine,
    delayline8: ISMDelayLine,
    delayline9: ISMDelayLine,
    delayline10: ISMDelayLine,
    delayline11: ISMDelayLine,
    delayline12: ISMDelayLine,
    delayline13: ISMDelayLine,
    delayline14: ISMDelayLine,
    delayline15: ISMDelayLine,
    delayline16: ISMDelayLine,
    delayline17: ISMDelayLine,
    delayline18: ISMDelayLine,
    delayline19: ISMDelayLine,
    delayline20: ISMDelayLine,
    delayline21: ISMDelayLine,
    delayline22: ISMDelayLine,
    delayline23: ISMDelayLine,
    delayline24: ISMDelayLine,
    delayline25: ISMDelayLine,
    delayline26: ISMDelayLine,
    delayline27: ISMDelayLine,
    delayline28: ISMDelayLine,
    delayline29: ISMDelayLine,
    delayline30: ISMDelayLine,
    delayline31: ISMDelayLine,
    delayline32: ISMDelayLine,
    delayline33: ISMDelayLine,
    delayline34: ISMDelayLine,
    delayline35: ISMDelayLine,
    delayline36: ISMDelayLine,
}



impl ASS {
    pub fn process(&mut self, audio_in: &[f32], out: &mut [f32]) {
        ASS::proc_line(&mut self.delayline0, audio_in, &mut self.temp);
        // ASS::proc_line(&mut self.delayline1, &self.temp, temp);
        // ASS::proc_line(&mut self.delayline2, audio_in, temp);
        // ASS::proc_line(&mut self.delayline3, audio_in, temp);
        // ASS::proc_line(&mut self.delayline4, audio_in, temp);
        // ASS::proc_line(&mut self.delayline5, audio_in, temp);
        // ASS::proc_line(&mut self.delayline6, audio_in, temp);
        // ASS::proc_line(&mut self.delayline7, audio_in, temp);
        // ASS::proc_line(&mut self.delayline8, audio_in, temp);
        // ASS::proc_line(&mut self.delayline9, audio_in, temp);
        // ASS::proc_line(&mut self.delayline10, audio_in, temp);
        // ASS::proc_line(&mut self.delayline11, audio_in, temp);
        // ASS::proc_line(&mut self.delayline12, audio_in, temp);
        // ASS::proc_line(&mut self.delayline13, audio_in, temp);
        // ASS::proc_line(&mut self.delayline14, audio_in, temp);
        // ASS::proc_line(&mut self.delayline15, audio_in, temp);
        // ASS::proc_line(&mut self.delayline16, audio_in, temp);
        // ASS::proc_line(&mut self.delayline17, audio_in, temp);
        // ASS::proc_line(&mut self.delayline18, audio_in, temp);
        // ASS::proc_line(&mut self.delayline19, audio_in, temp);
        // ASS::proc_line(&mut self.delayline20, audio_in, temp);
        // ASS::proc_line(&mut self.delayline21, audio_in, temp);
        // ASS::proc_line(&mut self.delayline22, audio_in, temp);
        // ASS::proc_line(&mut self.delayline23, audio_in, temp);
        // ASS::proc_line(&mut self.delayline24, audio_in, temp);
        // ASS::proc_line(&mut self.delayline25, audio_in, temp);
        // ASS::proc_line(&mut self.delayline26, audio_in, temp);
        // ASS::proc_line(&mut self.delayline27, audio_in, temp);
        // ASS::proc_line(&mut self.delayline28, audio_in, temp);
        // ASS::proc_line(&mut self.delayline29, audio_in, temp);
        // ASS::proc_line(&mut self.delayline30, audio_in, temp);
        // ASS::proc_line(&mut self.delayline31, audio_in, temp);
        // ASS::proc_line(&mut self.delayline32, audio_in, temp);
        // ASS::proc_line(&mut self.delayline33, audio_in, temp);
        // ASS::proc_line(&mut self.delayline34, audio_in, temp);
        // ASS::proc_line(&mut self.delayline35, audio_in, temp);
        // ASS::proc_line(&mut self.delayline36, audio_in, temp);

    }

    fn proc_line(delayline: &mut ISMDelayLine, audio_in: &[f32], temp: &mut [f32]) {
        audio_in.iter().zip(delayline.output_buffer.iter_mut()).zip(temp.iter_mut()).for_each(|((ain, dout, ),t)| {
            *dout = delayline.delayline.process(*ain);
            *t = *dout;
        })
    }
}




// audio data generation template
#[allow(unused)]
fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
