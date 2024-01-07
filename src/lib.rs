pub mod audioSceneHandlerData;
pub mod audio_module;
pub mod bind;
pub mod buffers;
pub mod convolver;
pub mod fdn;
pub mod filter;
pub mod mixingmatrix;
pub mod osc;
pub mod readwav;
//pub mod scene;      
pub mod scene_parser;
pub mod server;
pub mod image_source_method;
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
    // start scene handler thread
    let _server: () = start_server(port);
}
