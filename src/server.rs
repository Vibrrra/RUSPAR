use std::sync::{mpsc::Sender, Arc, Mutex};

use protobuf::Message;

use crate::{
    audioSceneHandlerData::Scene_data, audio_module::start_audio_thread,
    image_source_method::ISMAcousticScene, osc::OSCHandler,
};
pub fn start_server(port: u32) -> ! {
    // init server
    let mut ip_addr: String = String::new();
    ip_addr = "127.0.0.1".to_string() + ":" + &port.to_string();
    let mut osc_handle = OSCHandler::new(&ip_addr);

    let acoustic_scene = Arc::new(Mutex::new(ISMAcousticScene::default()));
    //let mut scene_data = Scene_data::default();
    // maybe start audio module here
    start_audio_thread(acoustic_scene.clone());

    loop {
        // receive from adress
        let byte_string = osc_handle.try_recv();

        // parse byte string to protobuf struct
        let scene_data = Scene_data::parse_from_bytes(&byte_string[..]).unwrap();

        if let Ok(mut data) = acoustic_scene.try_lock() {
            data.update_from_psd(&scene_data)
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
