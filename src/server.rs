use std::{sync::mpsc, thread::sleep, time::Duration};

use crate::{
    audioSceneHandlerData::Scene_data,
    audio_module::start_audio_thread,
    config::{IMAGE_SOURCE_METHOD_ORDER, MAX_SOURCES},
    image_source_method::{Room, SourceTrees, SourceType},
    ism_test_structure::IMS,
    osc::OSCHandler,
    scene_parser::update_scene,
};
use protobuf::Message;

// test struct. Don't what to send to the audio lopp yet
#[derive(Debug, Default, Clone)]
pub struct IsmMetaData {
    pub az: f32,
    pub el: f32,
    pub dist: f32,
}

pub fn start_server(port: u32, BUFFER_SIZE: usize) -> ! {
    // init some engine defaults for now
    // let ism_order: usize = 2;
    // let max_n_sources: usize = 10;
    // init server
    // let mut ip_addr: String = String::new();
    let ip_addr = "127.0.0.1".to_string() + ":" + &port.to_string();
    let mut osc_handle = OSCHandler::new(&ip_addr);

    // config the engine hereo
    // old
    // let mut source_trees: SourceTrees<crate::image_source_method::Source> = SourceTrees::create(MAX_SOURCES, IMAGE_SOURCE_METHOD_ORDER, None);
    //new
    let mut isms = IMS::create_raw(MAX_SOURCES);
    let room = Room::new(4.0, 3.0, 5.0);
    let (tx, rx) = mpsc::sync_channel(1);
    // let mut ism_meta_data_vector = Arc::new(Mutex::new(vec![IsmMetaData::default(); 36]));
    start_audio_thread(rx, isms.clone(), room, BUFFER_SIZE); //acoustic_scene.clone());

    let byte_string = osc_handle.try_recv();
    let scene_data = Scene_data::parse_from_bytes(&byte_string[..]).unwrap();

    sleep(Duration::from_millis(2000));
    loop {
        // receive from adress
        let byte_string = osc_handle.try_recv();

        // parse byte string to protobuf struct
        let scene_data = Scene_data::parse_from_bytes(&byte_string[..]).unwrap();

        // old
        // update_scene(&scene_data, &mut source_trees);
        // let src = source_trees.arenas[0].get(source_trees.roots[0]).unwrap().get();
        //let tx_res = tx.send(source_trees.clone()); //.unwrap();
        // println!("Az: {}, El: {}", isms.sources[0][0].listener_source_orientation.azimuth, isms.sources[0][0].listener_source_orientation.elevation);
        // new
        let x = &scene_data.listener.transform.position.x;
        let y = &scene_data.listener.transform.position.y;
        let z = &scene_data.listener.transform.position.z;
        // println!("x: {}, y: {}, z:{}", x, y, z); // isms.sources[0][0].get_pos());
        isms.update_from_scene(scene_data);
        let tx_res = tx.try_send(isms.clone());
        // let src = &isms.sources[0][0];

        match tx_res {
            Ok(_) => {

                // println!("Send!")
            }
            Err(e) => {
                // println!("{:?}",e)
            }
        }

        // experimental. forcing loop to be a bit chill
        // sleep(Duration::from_millis(10));
    }
}
