use cpal::{
    self,
    traits::{DeviceTrait, StreamTrait},
    BufferSize, FromSample, Sample, SizedSample, StreamConfig,
};



use std::thread;
use std::{
    f32::consts::PI,  path::Path, sync::mpsc::Receiver, vec,
};

use crate::{
    assets::{A_FDN, A_FDN_TC, B_FDN, B_FDN_TC, DL_S},
    audio_devices::get_output_device,
    config::{
        audio_file_list, C, IMAGE_SOURCE_METHOD_ORDER, MAX_SOURCES, SAMPLE_RATE,
        TARGET_AUDIO_DEVICE,
    },
    convolver::Spatializer,
    delaylines::{self, CircularBuffer, DelayLine},
    fdn::{
        calc_fdn_delayline_lengths, calc_hrtf_sphere_points, map_ism_to_fdn_channel,
        FDNInputBuffer, FeedbackDelayNetwork,
    },
    filter::{impulse, BinauralFilter, FFTManager, FilterStorage, FilterStorageIIR},
    iir_filter::{proc_tpdf2, HrtfFilterIIR, HrtfProcessorIIR},
    image_source_method::{
        from_source_tree, is_per_model, ISMLine, Room, Source, SourceTrees, SourceType,
        N_IS_INDEX_RANGES,
    },
    ism_test_structure::{ISMDelayLines, IMS, ISM_INDEX_RANGES},
    readwav::AudioFileManager,
};

pub fn start_audio_thread(rx: Receiver<IMS>, mut sources: IMS, room: Room, BUFFER_SIZE: usize) {
    //pub fn start_audio_thread(scene_data: Arc<Mutex<ISMAcousticScene>>) {
    thread::spawn(move || {
        // Audio host & device configs
        let host = cpal::HostId::Asio;
        let target_device: Option<cpal::Device> = get_output_device(TARGET_AUDIO_DEVICE);

        let device = match target_device {
            Some(device) => device,
            None => panic! {"Target Device not available!"},
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
                run::<i8>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::I16 => {
                run::<i16>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::I32 => {
                run::<i32>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::I64 => {
                run::<i64>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::U8 => {
                run::<u8>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::U16 => {
                run::<u16>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::U32 => {
                run::<u32>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::U64 => {
                run::<u64>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::F32 => {
                run::<f32>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
            }
            cpal::SampleFormat::F64 => {
                run::<f64>(device, stream_config.into(), rx, sources, room, BUFFER_SIZE)
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
    let channels = 2usize;
    let filterpath: &Path = Path::new("assets/hrtf_binary.dat");
    let anglepath: &Path = Path::new("assets/angles.dat");
    let iir_coeffs_path: &Path = Path::new("assets/hrir_iir_coeffs.dat");
    let iir_angles_path: &Path = Path::new("assets/hrir_iir_angles.dat");
    let iir_delays_path: &Path = Path::new("assets/hrir_iir_delays.dat");

    // init hrtf iir spatialiting
    let (iir_filterstorage, iir_filter_tree) =
        FilterStorageIIR::new(iir_coeffs_path, iir_angles_path, iir_delays_path);
    let fade_in = create_fade_in(buffer_size);
    let fade_out = create_fade_out(buffer_size);

    // Init Spatializer
    let mut fft_manager = FFTManager::new(buffer_size * 2);
    let (hrtf_storage, hrtf_tree) =
        FilterStorage::new(filterpath, anglepath, &mut fft_manager, buffer_size);
    let mut spatializer = Spatializer::new(buffer_size, fft_manager, &hrtf_storage);

    // Create (Image) Source Processing Delay Lines
    let mut sources = ISMDelayLines::new(
        source_trees,
        &room,
        C,
        sample_rate,
        buffer_size,
        IMAGE_SOURCE_METHOD_ORDER,
        spatializer.clone(),
    );

    // Create FDN
    // We init some constants for testing
    let fdn_n_dls: usize = 24;
    let delay_line_lengths: Vec<usize> = DL_S
        .iter()
        .map(|x| (x * sample_rate).ceil() as usize)
        .collect();
    let mut fdn_input_buf: FDNInputBuffer = FDNInputBuffer::new(fdn_n_dls, buffer_size);
    let mut fdn_output_buf: FDNInputBuffer = FDNInputBuffer::new(fdn_n_dls, buffer_size);
    let mut fdn = FeedbackDelayNetwork::from_assets(
        fdn_n_dls,
        buffer_size,
        delay_line_lengths,
        B_FDN,
        A_FDN,
        B_FDN_TC,
        A_FDN_TC,
    );

    // Create HRTF spatializer
    let fdn_hrtf_coords = calc_hrtf_sphere_points(24);
    let mut fdn_spatializers: Vec<Spatializer> = Vec::with_capacity(24);
    let mut fdn_curr_hrtf_idx = Vec::new();

    // Init FDN spatializer & HRTF Ids
    for i in 0..24usize {
        fdn_spatializers.push(spatializer.clone());
        let idx =
            hrtf_tree.find_closest_stereo_filter_angle(fdn_hrtf_coords[i].0, fdn_hrtf_coords[i].1);
        fdn_curr_hrtf_idx.push(idx);
    }

    // Init AudioFileManager
    let mut audio_file_managers: Vec<AudioFileManager> = Vec::new();
    for i in 0..1 {
        // MAX_SOURCES {
        audio_file_managers.push(AudioFileManager::new(
            audio_file_list[i].to_string(),
            buffer_size,
        ));
    }
    let mut test_audio_manager = audio_file_managers[0].buffer.clone();
    let mut output_buffers: Vec<Vec<f32>> = vec![vec![0.0f32; buffer_size]; 37];

    let mut audio_temp_buffer = vec![0.0f32; buffer_size];
    // INIT . This loop blocks the current fucntion for 5 secs and waits for
    // a first update from the server to initialize all variables with sane data
    loop {
        // match rx.recv_timeout(Duration::from_secs(5)) {
        match rx.try_recv() {
            Ok(data) => {
                sources
                    .sources
                    .iter_mut()
                    .zip(data.sources.iter())
                    .for_each(|(rev, src)| {
                        rev.iter_mut().zip(src.iter()).for_each(|(r, s)| {
                            // set delays
                            let delaytime = s.get_remaining_dist() / C * sample_rate;
                            r.delayline.buffer.set_delay_time(delaytime);
                            r.delayline.set_air_absoprtion(s.get_dist());
                            let orientation = s.get_lst_src_transform();
                            // r.new_hrtf_id = hrtf_tree.find_closest_stereo_filter_angle(orientation.azimuth, orientation.elevation);
                            // r.old_hrtf_id = r.new_hrtf_id;
                            let id = iir_filter_tree.find_closest_stereo_filter_angle(
                                orientation.azimuth,
                                orientation.elevation,
                            );
                            let new_coeffs = iir_filterstorage.get_filter(id);
                            r.hrtf_iir.hrir_iir.coeffs.update_coeffs(new_coeffs);
                            r.hrtf_iir.hrir_iir.coeffs.update_coeffs(new_coeffs);
                            // r.hrtf_iir.hrir_iir_old = r.hrtf_iir.hrir_iir.clone();
                        })
                    });
                break;
            }
            Err(e) => {
                // panic!("Initial receive from server has failed to to timeout: {e}")
                // sleep(Duration::from_millis(1));
            }
        };
    }

    let mut ism_temp_buffer = vec![0.0f32; buffer_size];
    // Create Stream
    let mut temp_buffer = vec![0.0f32; 2 * buffer_size];
    let stream: Result<cpal::Stream, cpal::BuildStreamError> = device.build_output_stream(
        &stream_config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // MAYBE Flushing some buffers here ...
            temp_buffer.iter_mut().for_each(|x| *x = 0.0);

            match rx.try_recv() {
                Ok(data) => {
                    // let o = data.sources[0][0].get_lst_src_transform();
                    // println!("{:?}",o);
                    sources
                        .sources
                        .iter_mut()
                        .zip(data.sources.iter())
                        .for_each(|(rev, src)| {
                            rev.iter_mut().zip(src.iter()).for_each(|(r, s)| {
                                // set delays
                                let delaytime = s.get_remaining_dist() / C * sample_rate;
                                r.delayline.buffer.set_delay_time(delaytime);
                                r.delayline.set_air_absoprtion(s.get_remaining_dist());
                                let orientation = s.get_lst_src_transform();
                                // r.old_hrtf_id = r.new_hrtf_id;
                                // let new_id = hrtf_tree.find_closest_stereo_filter_angle(orientation.azimuth, orientation.elevation);
                                // r.new_hrtf_id = new_id;
                                let id = iir_filter_tree.find_closest_stereo_filter_angle(
                                    orientation.azimuth,
                                    orientation.elevation,
                                );
                                let new_coeffs = iir_filterstorage.get_filter(id);
                                r.hrtf_iir.hrir_iir.coeffs.update_coeffs(new_coeffs);
                                r.dist_gain = 1.0 / s.dist;
                            })
                        });
                    // println!("Reveived!");
                }
                Err(_) => {}
            };

            // // Convolution version
            // for i in 0..1 {

            //     // read audio
            //     let mut audio_in: Vec<f32> = (0..buffer_size).into_iter().map(|_| test_audio_manager.read()).collect();
            //     let parent_src_idx = 0;
            //     let mut src = &mut sources.sources[i][parent_src_idx];
            //     src.delayline.process_block(&audio_in, &mut src.output_buffer);
            //     let nh = hrtf_storage.get_binaural_filter(src.new_hrtf_id);
            //     let oh =hrtf_storage.get_binaural_filter(src.old_hrtf_id);
            //     src.spatializer.process(&src.output_buffer, &mut temp_buffer, nh, oh, src.dist_gain);

            //     for ism_ranges_idx in 0..ISM_INDEX_RANGES.len() {
            //         let parent_src_idx = ISM_INDEX_RANGES[ism_ranges_idx].0;
            //         let src = &sources.sources[i][parent_src_idx];
            //         audio_temp_buffer.copy_from_slice(src.output_buffer.as_slice());
            //         // let o = sources.sources[i][parent_src_idx].output_buffer.as_slice();
            //         for ism_idx in ISM_INDEX_RANGES[ism_ranges_idx].1 .. ISM_INDEX_RANGES[ism_ranges_idx].2 {

            //             let src = &mut sources.sources[i][ism_idx];
            //             src.delayline.process_block(&audio_temp_buffer, &mut src.output_buffer);
            //             let nh = hrtf_storage.get_binaural_filter(src.new_hrtf_id);
            //             let oh =hrtf_storage.get_binaural_filter(src.old_hrtf_id);
            //             src.spatializer.process(&src.output_buffer, &mut temp_buffer, nh, oh,  src.dist_gain);

            //         }
            //     }
            // }
            // iir version
            for i in 0..1 {
                // read audio
                let mut audio_in: Vec<f32> = (0..buffer_size)
                    .into_iter()
                    .map(|_| test_audio_manager.read())
                    .collect();
                let parent_src_idx = 0;
                let mut src = &mut sources.sources[i][parent_src_idx];
                let mut y = 0f32;
                audio_in
                    .iter()
                    .zip(temp_buffer.chunks_exact_mut(channels))
                    .zip(src.output_buffer.iter_mut())
                    .zip(fade_in.iter())
                    .zip(fade_out.iter())
                    .for_each(|((((a_in, temp), outb), fin), fout)| {
                        y = src.delayline.process(*a_in);
                        *outb = y;
                        src.hrtf_iir.process(y*src.dist_gain, *fin, *fout, temp);
                    });
                // y = src.delayline.process(&audio_in);

                // let nh = hrtf_storage.get_binaural_filter(src.new_hrtf_id);
                // let oh =hrtf_storage.get_binaural_filter(src.old_hrtf_id);
                // src.spatializer.process(&src.output_buffer, &mut temp_buffer, nh, oh, src.dist_gain);

                for ism_ranges_idx in 0..ISM_INDEX_RANGES.len() {
                    let parent_src_idx = ISM_INDEX_RANGES[ism_ranges_idx].0;
                    let src = &sources.sources[i][parent_src_idx];
                    audio_temp_buffer.copy_from_slice(src.output_buffer.as_slice());
                    // let o = sources.sources[i][parent_src_idx].output_buffer.as_slice();
                    for ism_idx in
                        ISM_INDEX_RANGES[ism_ranges_idx].1..ISM_INDEX_RANGES[ism_ranges_idx].2
                    {
                        let src = &mut sources.sources[i][ism_idx];
                        // src.delayline
                        //     .process_block(&audio_temp_buffer, &mut src.output_buffer);
                        audio_temp_buffer
                            .iter()
                            .zip(temp_buffer.chunks_exact_mut(channels))
                            .zip(src.output_buffer.iter_mut())
                            .zip(fade_in.iter())
                            .zip(fade_out.iter())
                            .for_each(|((((a_in, temp), outb), fin), fout)| {
                                y = src.delayline.process(*a_in);
                                *outb = y;
                                src.hrtf_iir.process_add(y*src.dist_gain, *fin, *fout, temp);
                            });
                        // let nh = hrtf_storage.get_binaural_filter(src.new_hrtf_id);
                        // let oh =hrtf_storage.get_binaural_filter(src.old_hrtf_id);
                        // src.spatializer.process(&src.output_buffer, &mut temp_buffer, nh, oh,  src.dist_gain);
                    }
                }
            }

            for (frames, input) in data.chunks_mut(2).zip(temp_buffer.chunks(2)) {
                frames.iter_mut().zip(input.iter()).for_each(|(o, i)| {
                    //  0.5 -> hardcoded volume (safety) for now
                    *o = T::from_sample(*i * 0.015f32);
                });
            }

            // for (frames, input) in data.chunks_mut(2).zip(audio_in.iter()) {

            //         frames[0] = T::from_sample(*input*0.15f32);
            //         frames[1] = T::from_sample(*input*0.15f32);

            // }
        },
        error_callback,
        None, // Some(Duration::from_millis(5)), //None,
    );
    let stream_res: cpal::Stream = match stream {
        Ok(stream) => stream,
        Err(e) => panic!("ERROR: {e}"),
    };

    let stream_play_res: Result<(), cpal::PlayStreamError> = stream_res.play();
    match stream_play_res {
        Ok(_) => loop {},
        Err(e) => {
            println!("Error opening stream: {:?}", e)
        }
    };
    println!("Stream terminated!");
    Ok(())
}

#[derive(Clone)]
pub struct ISMDelayLine {
    pub delayline: DelayLine,
    pub output_buffer: Vec<f32>,
    pub spatializer: Spatializer,
    pub new_hrtf_id: usize,
    pub old_hrtf_id: usize,
    pub dist_gain: f32,
    pub hrtf_iir: HrtfProcessorIIR,
}
impl ISMDelayLine {
    pub fn new(delayline_length: usize, buffer_length: usize, spatializer: Spatializer) -> Self {
        ISMDelayLine {
            delayline: DelayLine::new(delayline_length),
            output_buffer: vec![0.0f32; buffer_length].into(),
            spatializer,
            new_hrtf_id: 1,
            old_hrtf_id: 1,
            dist_gain: 1.0,
            hrtf_iir: HrtfProcessorIIR::new(),
        }
    }
}

// process template
#[allow(unused)]
fn audio_process(output: &mut [f32], renderer: &mut dyn FnMut() -> (Vec<f32>, Vec<f32>)) {
    let (mut ism_output, mut fdn_output) = renderer();
    for (frame, ins) in output.chunks_mut(2).zip(ism_output.iter()) {
        let to_out_l = *ins; //+ fdn_out[0];
        let to_out_r = *ins; //+ fdn_out[0];

        if (to_out_l.abs() > 1.0) | (to_out_r.abs() > 1.0) {
            // println!("Clipping!");
        }
        frame[0] = to_out_l; //r[0] * adjust_loudness(n_sources);
        frame[1] = to_out_r; //r[1] * adjust_loudness(n_sources);
    }
}



// helper
pub fn create_fade_in(n_points: usize) -> Vec<f32> {
    let mut fade_in: Vec<f32> = vec![0.0; n_points];
    for i in 0..n_points {
        fade_in[i] = (((PI / 2.0) * (i as f32) / ((2 * n_points - 1) as f32)).sin()).powf(2.0);
    }
    fade_in
}
pub fn create_fade_out(n_points: usize) -> Vec<f32> {
    let mut fade_out: Vec<f32> = vec![0.0; n_points];
    for i in 0..n_points {
        fade_out[i] = (((PI / 2.0) * (i as f32) / ((2 * n_points - 1) as f32)).cos()).powf(2.0);
    }
    fade_out
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

#[cfg(test)]
#[test]
fn test_iir_function() {
    use crate::{filter::impulse, iir_filter::HRTFFilterIIRCoefficients};

    let iir_coeffs_path: &Path = Path::new("assets/hrir_iir_coeffs.dat");
    let iir_angles_path: &Path = Path::new("assets/hrir_iir_angles.dat");
    let iir_delays_path: &Path = Path::new("assets/hrir_iir_delays.dat");
    let (iir_filterstorage, iir_filter_tree) =
    FilterStorageIIR::new(iir_coeffs_path, iir_angles_path, iir_delays_path);
    let fade_in = create_fade_in(128);
    let fade_out = create_fade_out(128);
    let coeffs = iir_filterstorage.get_filter(1);
    let mut hrtf_prcocessor = HrtfProcessorIIR::new();
    hrtf_prcocessor.update(coeffs);
    
    let fade_in = create_fade_in(128);
    let fade_out = create_fade_in(128);

    let x = impulse(128);
    let mut y_l = Vec::new();
    let mut y_r = Vec::new();
    for i in 0 .. 128 {
        let mut out: [f32; 2] = [x[i],0.0];

        let out = hrtf_prcocessor.hrir_iir.process(&out) ;//, fade_in[i], fade_out[i], &mut out);
        y_l.push(out[0]);
        y_r.push(out[1]);
    }
    
    // println!("{:#?}", coeffs);
    println!("{:#?}", y_l);

}
#[test]
fn test_iir_hrtf_function() {
    use crate::{filter::impulse, iir_filter::HRTFFilterIIRCoefficients};

    let iir_coeffs_path: &Path = Path::new("assets/hrir_iir_coeffs.dat");
    let iir_angles_path: &Path = Path::new("assets/hrir_iir_angles.dat");
    let iir_delays_path: &Path = Path::new("assets/hrir_iir_delays.dat");
    let (iir_filterstorage, iir_filter_tree) =
    FilterStorageIIR::new(iir_coeffs_path, iir_angles_path, iir_delays_path);
    let fade_in = create_fade_in(128);
    let fade_out = create_fade_out(128);
    let coeffs = iir_filterstorage.get_filter(1);
    let mut hrtf_prcocessor = HrtfProcessorIIR::new();
    hrtf_prcocessor.update(coeffs);
    
    let fade_in = create_fade_in(128);
    let fade_out = create_fade_out(128);
    let dist_gain = 1.0/2.0;
    let x = impulse(128);
    let mut y_l = Vec::new();
    let mut y_r = Vec::new();
    for i in 0 .. 128 {
        let mut out: [f32; 2] = [x[i],0.0];

        hrtf_prcocessor.process(x[i],fade_in[i], fade_out[i],&mut out) ;//, fade_in[i], fade_out[i], &mut out);
        y_l.push(out[0] * dist_gain);
        y_r.push(out[1] * dist_gain);
    }
    
    // println!("{:#?}", coeffs);
    println!("{:#?}", y_r);

}
#[test]
fn test_iir_hrtf_indi_function() {
    use crate::{filter::impulse, iir_filter::HRTFFilterIIRCoefficients};

    let iir_coeffs_path: &Path = Path::new("assets/hrir_iir_coeffs.dat");
    let iir_angles_path: &Path = Path::new("assets/hrir_iir_angles.dat");
    let iir_delays_path: &Path = Path::new("assets/hrir_iir_delays.dat");
    let (iir_filterstorage, iir_filter_tree) =
    FilterStorageIIR::new(iir_coeffs_path, iir_angles_path, iir_delays_path);
    let fade_in = create_fade_in(128);
    let fade_out = create_fade_out(128);
    let coeffs = iir_filterstorage.get_filter(1);
    // let mut hrtf_prcocessor = HrtfProcessorIIR::new();
    let mut delay_left = CircularBuffer::new(32, coeffs.itd_delay_l);
    // hrtf_prcocessor.update(coeffs);
    let mut iir_filter = HrtfFilterIIR::default();
    iir_filter.coeffs = coeffs.clone();
    
    // let fade_in = create_fade_in(128);
    // let fade_out = create_fade_out(128);

    let x = impulse(128);
    let mut y_l = Vec::new();
    let mut y_r = Vec::new();
    for i in 0 .. 128 {
        // let mut o = delay_left.process(x[i]);
        let mut out: [f32; 2] = [x[i],x[i]];
        
        out =iir_filter.process(&out);
        // hrtf_prcocessor.process(x[i],fade_in[i], fade_out[i],&mut out) ;//, fade_in[i], fade_out[i], &mut out);
        y_l.push(out[0]);
        y_r.push(out[1]);
    }
    
    // println!("{:#?}", coeffs);
    println!("{:#?}", y_r);

}

#[test] 
fn test_df2t() {
    let iir_coeffs_path: &Path = Path::new("assets/hrir_iir_coeffs.dat");
    let iir_angles_path: &Path = Path::new("assets/hrir_iir_angles.dat");
    let iir_delays_path: &Path = Path::new("assets/hrir_iir_delays.dat");
    let (iir_filterstorage, iir_filter_tree) =
    FilterStorageIIR::new(iir_coeffs_path, iir_angles_path, iir_delays_path);
    let fade_in = create_fade_in(128);
    let fade_out = create_fade_out(128);
    let coeffs: &crate::iir_filter::HRTFFilterIIRCoefficients = iir_filterstorage.get_filter(1);
    let b = coeffs.b_l;
    let a = coeffs.a_l;
    let mut buf = vec![0.0f32; 32];
    let x = impulse(128);
    let mut y = vec![0.0f32; 128]; 
    for i in 0..128 {
        y[i] = proc_tpdf2(x[i], &b, &a, &mut buf);
    }
    println!("{:#?}", y)
}

