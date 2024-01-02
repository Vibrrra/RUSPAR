pub mod bind;
pub mod audio_module;
pub mod osc;
pub mod server;
pub mod audioSceneHandlerData;
pub mod filter;
pub mod convolver;
pub mod readwav;
use std::{sync::mpsc};
mod scene;
mod image_source_method;
use audioSceneHandlerData::Scene_data;
use audio_module::start_audio_thread;
use interoptopus::ffi_function;

// use protobuf::ext;
use server::start_server;

#[ffi_function]
#[no_mangle]
pub extern "C" fn add_one(x: u32) -> u32 {
    x + 1
}


#[ffi_function]
#[no_mangle]
pub extern "C" fn StartAudioSceneHandler(port: u32) {

    // create channel btw. audio thread and scene_handler thread
    let (tx, rx) = mpsc::channel::<Scene_data>();
    
    // start audio thread
    let _thread_audio: () = start_audio_thread(rx);


    // start scene handler thread
    let _server: () = start_server(port, tx);
    
    println!("Server terminated.")
}