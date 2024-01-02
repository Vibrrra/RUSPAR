use std::sync::mpsc::Sender;

use protobuf::Message;

use crate::{
    audioSceneHandlerData::Scene_data, image_source_method::ISMAcousticScene, osc::OSCHandler,
};
pub fn start_server(port: u32, tx: Sender<Scene_data>) -> ! {
    // init server
    let mut ip_addr: String = String::new();
    ip_addr = "127.0.0.1".to_string() + ":" + &port.to_string();
    let mut osc_handle = OSCHandler::new(&ip_addr);

    let mut acoustic_scene = ISMAcousticScene::default();
    //let mut scene_data = Scene_data::default();
    // maybe start audio module here
    //
    loop {
        // receive from adress
        let byte_string = osc_handle.try_recv();

        // parse byte string to protobuf struct
        let scene_data = Scene_data::parse_from_bytes(&byte_string[..]).unwrap();
        acoustic_scene.from_protobuf_scene(&scene_data);
        // Do something with scene;
        //
        // calc delays
        // updateRoom
        //
        // update audio engine
        tx.send(acoustic_scene).unwrap();
        //
    }
}
