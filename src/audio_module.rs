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
    image_source_method::{SourceTrees, N_IS_INDEX_RANGES, is_per_model, Room, SourceType, Source, from_source_tree}, 
    readwav::AudioFileManager, 
    config::{MAX_SOURCES, audio_file_list, C, IMAGE_SOURCE_METHOD_ORDER}, 
    delaylines::DelayLine, 
    fdn::{FeedbackDelayNetwork, calc_fdn_delayline_lengths, map_ism_to_fdn_channel, FDNInputBuffer},
assets::{DL_S, A_FDN, B_FDN, A_FDN_TC, B_FDN_TC}
};

//pub fn start_audio_thread(acoustic_scene: Arc<Mutex<ISMAcousticScene>>) {
pub fn start_audio_thread<U>(rx: Receiver<SourceTrees<U>>, mut source_trees: SourceTrees<U>, room: Room) 
where 
    U: Send+Clone+SourceType<Source> + 'static,
            
{
    //pub fn start_audio_thread(scene_data: Arc<Mutex<ISMAcousticScene>>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let output_device = host.default_output_device().unwrap();
        let output_config = output_device.default_output_config().unwrap();

        let audio_thread_result = match output_config.sample_format() {
            cpal::SampleFormat::I8 => {
                run::<i8, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::I16 => {
                run::<i16, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::I32 => {
                run::<i32, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::I64 => {
                run::<i64, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::U8 => {
                run::<u8, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::U16 => {
                run::<u16, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::U32 => {
                run::<u32, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::U64 => {
                run::<u64, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::F32 => {
                run::<f32, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            cpal::SampleFormat::F64 => {
                run::<f64, U>(&output_device, &output_config.into(), rx, source_trees, room)
            }
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };

        audio_thread_result
    });
}

fn run<T, U>(
    devcice: &cpal::Device,
    config: &cpal::StreamConfig,
    rx: Receiver<SourceTrees<U>>,
    mut source_trees: SourceTrees<U>,
    room: Room,
) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
U: SourceType<Source> + Clone + Send + 'static,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let buffer_size = 512;
    let error_callback = |err| eprintln!("Error occured on stream: {}", err);

    let filterpath: &str = "./assets/hrtf_binaray.dat";
    let anglepath: &str = "./assets/angles.dat";
    
    let ism_order = 2;
    
    // Init receive 
    let mut source_trees = from_source_tree(source_trees, buffer_size);
    let source_trees_update = rx.recv().unwrap();

    // Init Spatializer
    let mut fft_manager = FFTManager::new(buffer_size);
    let (hrtf_storage, hrtf_tree) =
        FilterStorage::new(filterpath, anglepath, &mut fft_manager, buffer_size);

    // let mut spatializers: Vec<Spatializer> = vec![Spatializer::new(buffer_size, fft_manager, &hrtf_storage); MAX_SOURCES * is_per_model(ism_order, 6usize)];
    let spatializer = Spatializer::new(buffer_size, fft_manager, &hrtf_storage); MAX_SOURCES * is_per_model(ism_order, 6usize);
    // set spatializer for sources in source tree
    source_trees.arenas.iter_mut().for_each(|arena| {
        arena.iter_mut().for_each(|src| {
            src.get_mut().source.set_spatializer(spatializer.clone());
        })
    });

    // TODO:: This should be handled by an init method providing start-up data from Unity for 
    let init_az_el: [f32; 2] = [0.0, 0.0];
    let mut prev_hrtf_ids:Vec<usize>  = vec![hrtf_tree.find_closest_stereo_filter_angle(init_az_el[0], init_az_el[1]); MAX_SOURCES* is_per_model(IMAGE_SOURCE_METHOD_ORDER, 6usize)];
    let mut curr_hrtf_ids:Vec<usize>  = vec![hrtf_tree.find_closest_stereo_filter_angle(init_az_el[0], init_az_el[1]); MAX_SOURCES* is_per_model(IMAGE_SOURCE_METHOD_ORDER, 6usize)];

    // let mut audio_scene = ISMAcousticScene::default();

    let ism_buffer_len =  (sample_rate * 15.0 / C ).ceil() as usize;

    // Init ISM 
    let mut buffer_trees: BufferTree = create_buffer_trees(MAX_SOURCES, ism_buffer_len, ism_order);
    let mut input_buffer: Vec<Vec<f32>> = vec![vec![0.0f32; buffer_size]; MAX_SOURCES];
    let mut ism_output_buffers: Vec<Vec<f32>> = vec![vec![0.0f32; buffer_size]; MAX_SOURCES];
    let mut ism_delay_lines: Vec<DelayLine> = vec![DelayLine::new(ism_buffer_len); MAX_SOURCES];
    let mut n_active_sources = 1usize;

    // Init FDN
    // We init some constants for testing
    let fdn_n_dls: usize = 24;
    let delay_line_lengths: Vec<usize>  = DL_S.iter().map(|x| {(x*sample_rate).ceil() as usize}).collect();
    let mut fdn_input_buf: FDNInputBuffer = FDNInputBuffer::new(fdn_n_dls, buffer_size);
    let mut fnd_output_buf: FDNInputBuffer = FDNInputBuffer::new(fdn_n_dls, buffer_size);
    // let fdn_dl_lengths = calc_fdn_delayline_lengths(fdn_n_dls, &room.dimension, C);
let mut fdn = FeedbackDelayNetwork::from_assets(fdn_n_dls, buffer_size,  delay_line_lengths, B_FDN, A_FDN, B_FDN_TC, A_FDN_TC);

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
            fdn_input_buf.flush();

            // Receive Updates
            match rx.try_recv() {
                Ok(data) => {
                    n_active_sources = data.roots.len();
                    for (((update_arena, update_vec), to_be_updated_arena),to_be_updated_vec) in (data.arenas.iter().zip(data.node_lists.iter()).zip(source_trees.arenas.iter_mut()).zip(source_trees.node_lists.iter())) {
                        for (update_node,to_be_updated_node) in update_vec.iter().zip(to_be_updated_vec.iter()) {
                            to_be_updated_arena.get_mut(*to_be_updated_node).unwrap().get_mut().source = update_arena.get(*update_node).unwrap().get().clone();
                        }
                    }
                    // source_trees;
                },
                Err(_) => {},                
            };

            // Update ISM and probably (FDN)

            // OuterLoop 0:
            // iterate over all source- and buffer-trees and their respective node_lists 
            source_trees.arenas.iter_mut()
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
                    let src = src_arena.get_mut(*src_node_id).unwrap().get_mut();
                    let delay_time = src.source.get_dist() / C;
                    let delayline = buffer_arena.get_mut(*buffer_node_id).unwrap().get_mut(); 
                    delayline.delayline.set_delay_time_ms(delay_time, sample_rate);
                    
                    src.source.set_prev_hrtf_id(src.source.get_curr_hrtf_id());
                    src.source.set_curr_hrtf_id(hrtf_tree.find_closest_stereo_filter_angle(src.source.get_lst_src_transform().azimuth, src.source.get_lst_src_transform().elevation));
                    // *prev_hrtf_id = *curr_hrtf_id;
                    // // *curr_hrtf_id = hrtf_tree.find_closest_stereo_filter_angle(src.listener_source_orientation.azimuth, src.listener_source_orientation.elevation);  
// *curr_hrtf_id = hrtf_tree.find_closest_stereo_filter_angle(src.get_lst_src_transform().azimuth, src.get_lst_src_transform().elevation);  
                    // *curr_hrtf_id = hrtf_tree.find_closest_stereo_filter_angle(src.get_.azimuth, src.listener_source_orientation.elevation);  
                    let fdn_delayline_idx = map_ism_to_fdn_channel(n, fdn_n_dls);
                    // InnerLoop 2: 
                    // Iterate over samples (buffersize)
                    ism_output_buffer.iter_mut().zip(fdn_input_buf.buffer[fdn_delayline_idx].iter_mut()).for_each(|(mut ism_line_output, fdn_input)| {
// ism_output_buffer.iter_mut().zip(fdn.matrix_inputs.iter_mut()).for_each(|(mut ism_line_output, fdn_input)| {
                        //---------------------------------------
                        // read audio in per source
                        let sample_in = audio_file_managers[n].buffer.read();
                        
                        // process delaylines and store output buffer (-> spatializer)
                        *ism_line_output = delayline.delayline.process(sample_in);           
                        // *fdn_input += *ism_line_output;
                        // map to FDN input channels
                        // let fdn_delayline_idx = map_ism_to_fdn_channel(n, fdn_n_dls);
                        

                    }) 
                })
            });   

            // fdn.matrix_inputs.iter_mut().for_each(||)
            // fdn.process(fnd_input_buf, fdn_ou)
            for i in 0..buffer_size {
                fdn.delaylines.iter_mut().zip(fdn.matrix_outputs.iter()).zip(fdn.matrix_inputs.iter_mut()).zip(fnd_output_buf.buffer.iter_mut()).zip(fdn_input_buf.buffer.iter())
                .for_each(|((((fdn_in, mat_out),mat_in),fdn_out), fdn_input_buf)| {
                    *mat_in = fdn_in.tick(fdn_input_buf[i]+mat_out);
                    fdn_out[i] = *mat_in;
                });
            }


            // HRTF stuff

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