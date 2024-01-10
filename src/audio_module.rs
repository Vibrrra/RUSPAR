use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait}, FromSample, Sample, SizedSample,
};
use indextree::{Arena, NodeId};
use nalgebra::Vector3;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::{
    convolver::Spatializer,
    filter::{BinauralFilter, FFTManager, FilterStorage},
    buffers::CircularDelayBuffer, 
    image_source_method::{SourceTrees, N_IS_INDEX_RANGES, is_per_model, Room}, 
    readwav::AudioFileManager, 
    config::{MAX_SOURCES, audio_file_list, C}, 
    delaylines::DelayLine, 
    fdn::{FeedbackDelayNetwork, calc_fdn_delayline_lengths, map_ism_to_fdn_channel, FDNInputBuffer},
};

//pub fn start_audio_thread(acoustic_scene: Arc<Mutex<ISMAcousticScene>>) {
pub fn start_audio_thread(rx: Receiver<SourceTrees>, mut source_trees: SourceTrees, room: Room) {
    //pub fn start_audio_thread(scene_data: Arc<Mutex<ISMAcousticScene>>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let output_device = host.default_output_device().unwrap();
        let output_config = output_device.default_output_config().unwrap();

        let audio_thread_result = match output_config.sample_format() {
            cpal::SampleFormat::I8 => {
                run::<i8>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::I16 => {
                run::<i16>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::I32 => {
                run::<i32>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::I64 => {
                run::<i64>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::U8 => {
                run::<u8>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::U16 => {
                run::<u16>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::U32 => {
                run::<u32>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::U64 => {
                run::<u64>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::F32 => {
                run::<f32>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::F64 => {
                run::<f64>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };

        audio_thread_result
    });
}

fn run<T>(
    devcice: &cpal::Device,
    config: &cpal::StreamConfig,
    rx: Receiver<SourceTrees>,
    mut source_trees: SourceTrees,
    room: Room,
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
    
    let ism_order = 2;
    
    // Init receive 
      source_trees = rx.recv().unwrap();

    // Init Spatializer
    let mut fft_manager = FFTManager::new(buffer_size);
    let (hrtf_storage, hrtf_tree) =
        FilterStorage::new(filterpath, anglepath, &mut fft_manager, buffer_size);

    let mut spatializers: Vec<Spatializer> = vec![Spatializer::new(buffer_size, fft_manager, &hrtf_storage); MAX_SOURCES * is_per_model(ism_order, 6usize)];

    // TODO:: This should be handled by an init method providing start-up data from Unity for 
    let init_az_el: [f32; 2] = [0.0, 0.0];
    let mut prev_hrtf_ids:Vec<usize>  = vec![hrtf_tree.find_closest_stereo_filter_angle(init_az_el[0], init_az_el[1]); MAX_SOURCES];
    let mut curr_hrtf_ids:Vec<usize>  = vec![hrtf_tree.find_closest_stereo_filter_angle(init_az_el[0], init_az_el[1]); MAX_SOURCES];

    // let mut audio_scene = ISMAcousticScene::default();

    let ism_buffer_len =  (sample_rate * 15.0 / C ).ceil() as usize;

    // Init ISM 
    let mut buffer_trees: BufferTree = create_buffer_trees(MAX_SOURCES, ism_buffer_len, ism_order);
    let mut input_buffer: Vec<Vec<f32>> = vec![vec![0.0f32; buffer_size]; MAX_SOURCES];
    let mut ism_output_buffers: Vec<Vec<f32>> = vec![vec![0.0f32; buffer_size]; MAX_SOURCES];
    let mut ism_delay_lines: Vec<DelayLine> = vec![DelayLine::new(ism_buffer_len); MAX_SOURCES];
    let mut n_active_sources = 1usize;

    // Init FDN
    // let default_room_dims = Vector3::new(4.0, 3.0, 5.0);
    let fdn_n_dls: usize = 24;
    let mut fnd_input_buf: FDNInputBuffer = FDNInputBuffer::new(fdn_n_dls, buffer_size);
    let fdn_dl_lengths = calc_fdn_delayline_lengths(fdn_n_dls, &room.dimension, C);
    // let mut fdn = FeedbackDelayNetwork::new(fdn_dl_lengths);
    // Init AudioFileManager
    let mut audio_file_managers: Vec<AudioFileManager> = Vec::new();
    for i in 0 .. MAX_SOURCES {
        audio_file_managers.push( AudioFileManager::new(audio_file_list[i].to_string(), buffer_size));
    }

  

    // Create Stream
    let stream = devcice.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // flush some buffers
            fnd_input_buf.flush();

            // Receive Updates
            match rx.try_recv() {
                Ok(data) => {
                    n_active_sources = data.roots.len();
                    source_trees = data
                },
                Err(_) => {},                
            };

            // Update ISM and probably (FDN)

            // OuterLoop 0:
            // iterate over all source- and buffer-trees and their respective node_lists 
            source_trees.arenas.iter()
                    .zip(source_trees.node_lists.iter())
                    .enumerate()
                    .zip(buffer_trees.buffer_arenas.iter_mut().zip(buffer_trees.node_lists.iter()))
                    .for_each(|((n,(src_arena, src_node_list)), (buffer_arena, buffer_node_list))| {

                // InnerLoop 1:
                // for every source- and buffer-tree iterate over the individual (image) sources, hrtfs, buffers, delaylines.
                src_node_list.iter()
                        .zip(buffer_node_list.iter())
                        .zip(prev_hrtf_ids.iter_mut().zip(curr_hrtf_ids.iter_mut()))
                        .zip(ism_output_buffers.iter_mut())        
                        .enumerate()
                        .for_each(|(n, (((src_node_id, buffer_node_id), (prev_hrtf_id, curr_hrtf_id)), ism_output_buffer))| {
                    // ---------------------------------------------                   
                    //      -set delaytimes for every delayline
                    //      -assign prev and curr hrtf filters
                    //      -calc mapping index to FDN input
                    let src = src_arena.get(*src_node_id).unwrap().get();
                    let delay_time = src.dist / C;
                    let delayline = buffer_arena.get_mut(*buffer_node_id).unwrap().get_mut(); 
                    delayline.delayline.set_delay_time_ms(delay_time, sample_rate);
                    *prev_hrtf_id = *curr_hrtf_id;
                    *curr_hrtf_id = hrtf_tree.find_closest_stereo_filter_angle(src.listener_source_orientation.azimuth, src.listener_source_orientation.elevation);  
                    let fdn_delayline_idx = map_ism_to_fdn_channel(n, fdn_n_dls);
                    // InnerLoop 2: 
                    // Iterate over samples (buffersize)
                    ism_output_buffer.iter_mut().zip(fnd_input_buf.buffer[n].iter_mut()).for_each(|(mut ism_line_output, fdn_input)| {
                        //---------------------------------------
                        // read audio in per source
                        let sample_in = audio_file_managers[n].buffer.read();
                        
                        // process delaylines and store output buffer (-> spatializer)
                        *ism_line_output = delayline.delayline.process(sample_in);           
                        *fdn_input += *ism_line_output;
                        // map to FDN input channels
                        // let fdn_delayline_idx = map_ism_to_fdn_channel(n, fdn_n_dls);


                    }) 
                })
            });   

            // read audio in - Probably useless as audio is already in AudioFileManager buffers
            // for n in 0 .. source_trees.roots.len() {
            //     audio_file_managers[n].read_n_samples(buffer_size, &mut input_buffer[n][0..] );
            // }         

            
            //  Process everything here ...
            // for n in 0 .. source_trees.roots.len() {
            
            //     unimplemented!();
            // todo!("Abstract Delay Lines!");    
            (0..n_active_sources).into_iter()
                                .zip(ism_output_buffers.iter_mut())
                                .for_each(|(n, out_buffer)|{
                out_buffer.iter_mut().for_each(|x|{
                    *x = ism_delay_lines[n].process(audio_file_managers[n].buffer.read());
                    // map_ism_to_fdn_channel(x, channel_index, fdn_buff)

                });
            });
                


                // ism_output_buffer[n];
            
            //    ...
            //    ... 
            //    ...
            todo!()
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

#[derive(Debug)]
pub struct BufferTree {
    pub buffer_arenas: Vec<Arena<DelayLine>>,
    pub node_lists: Vec<Vec<NodeId>>
}

pub fn create_buffer_trees(n_sources: usize, buffer_length: usize, ism_order: usize) -> BufferTree { //} -> Vec<Arena<CircularDelayBuffer>>{
    let mut buffer_arenas: Vec<Arena<DelayLine>> = Vec::new();
    let mut node_lists: Vec<Vec<indextree::NodeId>> = Vec::new();
    for n in 0 .. n_sources {
        let mut arena = indextree::Arena::new();
        let mut node_list = Vec::new();
        let root_buffer = arena.new_node(DelayLine::new(buffer_length));
        node_list.push(root_buffer);
        for i in N_IS_INDEX_RANGES[0].0 .. N_IS_INDEX_RANGES[0].1 {
                for _ in 0..6 {
                    let parent_node = arena.get(node_list[i]).unwrap().get();
                    let new_buffer = arena.new_node(DelayLine::new(buffer_length));
                    node_list[i].append(new_buffer, &mut arena);
                    node_list.push(new_buffer);
                }
        }
        
        for order in 1..ism_order {
            for i in N_IS_INDEX_RANGES[order].0 .. N_IS_INDEX_RANGES[order].1 {
                    for _ in 0..5 {
                        let parent_node = arena.get(node_list[i]).unwrap().get();
                        let new_buffer = arena.new_node(DelayLine::new(buffer_length));
                        node_list[i].append(new_buffer, &mut arena);
                        node_list.push(new_buffer);
                    }
            }
        }
        buffer_arenas.push(arena);
        node_lists.push(node_list);
    }
    BufferTree {buffer_arenas, node_lists}
}

#[cfg(test)]
#[test]
fn test_buffer_tree() {
    let ism_order = 1;
    let n = 1; 
    let bl = 5;
    let bt = create_buffer_trees(n, bl, ism_order);

    for i in bt.node_lists[0].iter().enumerate() {
        println!("Nr: {}, {:?}", i.0, i.1)
    }
}