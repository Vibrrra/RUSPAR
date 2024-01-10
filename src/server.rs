use std::sync::mpsc;

use crate::{
    audioSceneHandlerData::Scene_data, audio_module::start_audio_thread,
    osc::OSCHandler, image_source_method::{SourceTrees, Room}, scene_parser::update_scene,
};
use nalgebra::Vector3;
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
    let ism_order: usize = 2;
    let max_n_sources: usize = 10;
    // init server
    // let mut ip_addr: String = String::new();
    let ip_addr = "127.0.0.1".to_string() + ":" + &port.to_string();
    let mut osc_handle = OSCHandler::new(&ip_addr);

    // config the engine hereo
    let mut source_trees = SourceTrees::create(max_n_sources, ism_order);
    // let mut acoustic_scene: ISMAcousticScene = ISMAcousticScene::default();
    // let acoustic_scene = Arc::new(Mutex::new(ISMAcousticScene::default()));
    //let mut scene_data = Scene_data::default();
    // maybe start audio module here
    //
    let room =  Room::new(4.0, 3.0, 5.0);
    let (tx, rx) = mpsc::channel();
    // let mut ism_meta_data_vector = Arc::new(Mutex::new(vec![IsmMetaData::default(); 36]));
    start_audio_thread(rx, source_trees.clone(), room); //acoustic_scene.clone());

    loop {
        // receive from adress
        let byte_string = osc_handle.try_recv();

        // parse byte string to protobuf struct
        let scene_data = Scene_data::parse_from_bytes(&byte_string[..]).unwrap();
        update_scene(&scene_data, &mut source_trees);
        tx.send(source_trees.clone()).unwrap();
        //
    }
}
