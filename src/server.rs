use std::sync::{mpsc::Sender, Arc, Mutex};

use crate::{
    audioSceneHandlerData::Scene_data, audio_module::start_audio_thread,
    image_source_method::ISMAcousticScene, osc::OSCHandler,
};
use nalgebra::{distance, OPoint, Point3};
use protobuf::Message;

// test struct. Don't what to send to the audio lopp yet
#[derive(Debug, Default, Clone)]
pub struct IsmMetaData {
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

    let mut acoustic_scene: ISMAcousticScene = ISMAcousticScene::default();
    // let acoustic_scene = Arc::new(Mutex::new(ISMAcousticScene::default()));
    //let mut scene_data = Scene_data::default();
    // maybe start audio module here
    //

    let mut ism_meta_data_vector = Arc::new(Mutex::new(vec![IsmMetaData::default(); 36]));
    start_audio_thread(ism_meta_data_vector.clone()); //acoustic_scene.clone());

    loop {
        // receive from adress
        let byte_string = osc_handle.try_recv();

        // parse byte string to protobuf struct
        let scene_data = Scene_data::parse_from_bytes(&byte_string[..]).unwrap();
        acoustic_scene.update_from_psd(&scene_data);
        match ism_meta_data_vector.try_lock() {
            Ok(mut data) => {
                
                for i in 0..data.len() {
                    data[i].dist = nalgebra::distance(
                        &acoustic_scene.sound_sources[i].position,
                        &acoustic_scene.listener.position,
                    ) as f32;

                    // calc az el here - Andi-Chrissi-Lösung hier
                    
                    data[i].az = 0.0;
                    data[i].el = 0.0;

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
