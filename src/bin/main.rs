use std::env;

use RUSPAR::{server::start_server, StartAudioSceneHandler};

// this is only for testing purposes

fn main () {
    let port: u32 = 7001;
    // StartAudioSceneHandler(port);
    // let args: Vec<String> = env::args().collect();
    
    // let port: u32 = args[0].trim().parse().unwrap();
     let _server: () = start_server(port);
}
