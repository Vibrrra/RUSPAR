use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait}, FromSample, Sample, SizedSample,
};
use indextree::{Arena, NodeId};
use std::sync::mpsc::Receiver;
use std::thread;

use crate::{
    convolver::Spatializer,
    filter::{BinauralFilter, FFTManager, FilterStorage},
    buffers::CircularDelayBuffer, image_source_method::{SourceTrees, N_IS_INDEX_RANGES}, readwav::AudioFileManager, config::{MAX_SOURCES, audio_file_list, C},
};

//pub fn start_audio_thread(acoustic_scene: Arc<Mutex<ISMAcousticScene>>) {
pub fn start_audio_thread(rx: Receiver<SourceTrees>, _source_trees: SourceTrees) {
    //pub fn start_audio_thread(scene_data: Arc<Mutex<ISMAcousticScene>>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let output_device = host.default_output_device().unwrap();
        let output_config = output_device.default_output_config().unwrap();

        let audio_thread_result = match output_config.sample_format() {
            cpal::SampleFormat::I8 => {
                run::<i8>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::I16 => {
                run::<i16>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::I32 => {
                run::<i32>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::I64 => {
                run::<i64>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::U8 => {
                run::<u8>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::U16 => {
                run::<u16>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::U32 => {
                run::<u32>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::U64 => {
                run::<u64>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::F32 => {
                run::<f32>(&output_device, &output_config.into(), rx)
            }
            cpal::SampleFormat::F64 => {
                run::<f64>(&output_device, &output_config.into(), rx)
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

    // let mut audio_scene = ISMAcousticScene::default();
    let ism_order = 2;
    // let speed_of_sound = 343.0;
    let ism_buffer_len =  (sample_rate * 15.0 / C ).ceil() as usize;

    // let mut ism_buffers = vec![CircularDelayBuffer::new(ism_buffer_len); n_sources];
    let mut buffer_trees: BufferTree = create_buffer_trees(MAX_SOURCES, ism_buffer_len, ism_order);
    let mut input_buffer: Vec<Vec<f32>> = vec![vec![0.0f32; buffer_size];MAX_SOURCES];
    let mut ism_output_buffers: Vec<Vec<f32>> = vec![vec![0.0f32; buffer_size];MAX_SOURCES];
    let mut audio_file_managers: Vec<AudioFileManager> = Vec::new();
    let mut n_active_sources = 1usize;
    for i in 0 .. MAX_SOURCES {
        audio_file_managers.push( AudioFileManager::new(audio_file_list[i].to_string(), buffer_size));
    }
    // Create Stream
    let stream = devcice.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            
            // Receive Updates
            let source_trees = match rx.try_recv() {
                Ok(data) => {
                    n_active_sources = data.roots.len();
                    data
                },
                Err(_) => todo!(),                
            };

            // Update ISM and probably (FDN)
            source_trees.arenas.iter()
                                .zip(source_trees.node_lists.iter())
                                .enumerate()
                                .zip(buffer_trees.buffer_arenas.iter_mut().zip(buffer_trees.node_lists.iter()))
                                .for_each(|((n,(src_arena, src_node_list)), (buffer_arena, buffer_node_list))| {
                src_node_list.iter().zip(buffer_node_list.iter()).for_each(|(src_node_id, buffer_node_id)| {
                    
                    // updating buffer read-pointers. We could maybe already write audio into these if we are already iterating. maybe even more processing?
                    let delay_time = src_arena.get(*src_node_id).unwrap().get().dist / C;
                    buffer_arena.get_mut(*buffer_node_id).unwrap().get_mut().set_delay_time_ms(delay_time, sample_rate);
                })
            });   

            // read audio in - Probably useless as audio is already in AudioFileManager buffers
            // for n in 0 .. source_trees.roots.len() {
            //     audio_file_managers[n].read_n_samples(buffer_size, &mut input_buffer[n][0..] );
            // }         

            
            //  Process everything here ...
            // for n in 0 .. source_trees.roots.len() {
            
                unimplemented!();
            todo!("Abstract Delay Lines!");    
            (0..n_active_sources).into_iter().zip(ism_output_buffers.iter_mut()).for_each(|(n, out_buffer)|{
                for i in 0..buffer_size {
                    out_buffer.write(air_attenuation_filter.process(input_buffer[n].read()));
                    input_buffer[n].write(audio_file_managers[n]);
                }
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
    pub buffer_arenas: Vec<Arena<CircularDelayBuffer>>,
    pub node_lists: Vec<Vec<NodeId>>
}

pub fn create_buffer_trees(n_sources: usize, buffer_length: usize, ism_order: usize) -> BufferTree { //} -> Vec<Arena<CircularDelayBuffer>>{
    let mut buffer_arenas: Vec<Arena<CircularDelayBuffer>> = Vec::new();
    let mut node_lists: Vec<Vec<indextree::NodeId>> = Vec::new();
    for n in 0 .. n_sources {
        let mut arena = indextree::Arena::new();
        let mut node_list = Vec::new();
        let root_buffer = arena.new_node(CircularDelayBuffer::new(buffer_length));
        node_list.push(root_buffer);
        for i in N_IS_INDEX_RANGES[0].0 .. N_IS_INDEX_RANGES[0].1 {
                for _ in 0..6 {
                    let parent_node = arena.get(node_list[i]).unwrap().get();
                    let new_buffer = arena.new_node(CircularDelayBuffer::new(buffer_length));
                    node_list[i].append(new_buffer, &mut arena);
                    node_list.push(new_buffer);
                }
        }
        
        for order in 1..ism_order {
            for i in N_IS_INDEX_RANGES[order].0 .. N_IS_INDEX_RANGES[order].1 {
                    for _ in 0..5 {
                        let parent_node = arena.get(node_list[i]).unwrap().get();
                        let new_buffer = arena.new_node(CircularDelayBuffer::new(buffer_length));
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