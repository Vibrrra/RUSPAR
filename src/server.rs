use std::sync::{mpsc::Sender, Arc, Mutex};

use crate::{
    audioSceneHandlerData::Scene_data, audio_module::start_audio_thread,
    image_source_method::ISMAcousticScene, osc::OSCHandler,
};
use nalgebra::{distance, OPoint, Point3};
use protobuf::Message;

// test struct. Don't what to send to the audio lopp yet
#[derive(Debug, Default, Clone)]
struct IsmMetaData {
    pub az: f32,
    pub el: f32,
    pub dist: f32,
}

pub fn start_server(port: u32) -> ! {
    // init server
    let mut ip_addr: String = String::new();
    ip_addr = "127.0.0.1".to_string() + ":" + &port.to_string();
    let mut osc_handle = OSCHandler::new(&ip_addr);

    // config the engine hereo

    let acoustic_scene = Arc::new(Mutex::new(ISMAcousticScene::default()));
    //let mut scene_data = Scene_data::default();
    // maybe start audio module here
    //

    let mut ism_meta_data_vector = vec![IsmMetaData::default(); 36];
    start_audio_thread(acoustic_scene.clone());

    loop {
        // receive from adress
        let byte_string = osc_handle.try_recv();

        // parse byte string to protobuf struct
        let scene_data = Scene_data::parse_from_bytes(&byte_string[..]).unwrap();

        match acoustic_scene.try_lock() {
            Ok(mut data) => {
                data.update_from_psd(&scene_data);
                for i in 0..ism_meta_data_vector.len() {
                    ism_meta_data_vector[i].dist = nalgebra::distance(
                        &data.sound_sources[i].position,
                        &data.listener.position,
                    ) as f32;
                }
            }
            Err(_) => todo!(), // update_from_psd(&scene_data)
        }

        // Do something with scene;
        //
        // calc delays
        // updateRoom
        //
        // update audio engine
        // tx.send(acoustic_scene).unwrap();
        //
    }
}
