use std::{sync::mpsc, thread::sleep, time::Duration};

use crate::{
    audioSceneHandlerData::Scene_data, audio_module::start_audio_thread, config::{IMAGE_SOURCE_METHOD_ORDER, MAX_SOURCES}, image_source_method::{SourceTrees, Room}, osc::OSCHandler, scene_parser::update_scene
};
use protobuf::Message;

// test struct. Don't what to send to the audio lopp yet
#[derive(Debug, Default, Clone)]
pub struct IsmMetaData {
    pub az: f32,
    pub el: f32,
    pub dist: f32,
}

pub fn start_server(port: u32) -> ! {

    // init some engine defaults for now
    // let ism_order: usize = 2;
    // let max_n_sources: usize = 10;
    // init server
    // let mut ip_addr: String = String::new();
    let ip_addr = "127.0.0.1".to_string() + ":" + &port.to_string();
    let mut osc_handle = OSCHandler::new(&ip_addr);

    // config the engine hereo
    let mut source_trees: SourceTrees<crate::image_source_method::Source> = SourceTrees::create(MAX_SOURCES, IMAGE_SOURCE_METHOD_ORDER, None);
    let room =  Room::new(4.0, 3.0, 5.0);
    let (tx, rx) = mpsc::channel();
    // let mut ism_meta_data_vector = Arc::new(Mutex::new(vec![IsmMetaData::default(); 36]));
    start_audio_thread(rx, source_trees.clone(), room); //acoustic_scene.clone());

    sleep(Duration::from_millis(2000));
    loop {
        // receive from adress
        let byte_string = osc_handle.try_recv();

        // parse byte string to protobuf struct
        let scene_data = Scene_data::parse_from_bytes(&byte_string[..]).unwrap();
        update_scene(&scene_data, &mut source_trees);
        let src = source_trees.arenas[0].get(source_trees.roots[0]).unwrap().get();
        println!("Az: {}, El: {}", src.listener_source_orientation.azimuth, src.listener_source_orientation.elevation);
        let tx_res = tx.send(source_trees.clone()); //.unwrap();
        match tx_res {
            Ok(_) => {},
            Err(e) => {println!("{:?}",e)},
        }
        // experimental. forcing loop to be a bit chill
        // sleep(Duration::from_millis(10));
    }
}
