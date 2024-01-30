use std::env;
use lazy_static::lazy_static;
use RUSPAR::{config::BUFFER_SIZE_CONF, server::start_server, StartAudioSceneHandler};

// this is only for testing purposes

fn main () {

    lazy_static! {
        static ref BUFFER_SIZE: usize = {
            let args = env::args().nth(1);
                match args {
                    Some(arg) => {arg.parse::<usize>().unwrap_or(BUFFER_SIZE_CONF.try_into().unwrap())},
                    None =>BUFFER_SIZE_CONF.try_into().unwrap(),
                }   
        };
    }

    let port: u32 = 7001;
    // StartAudioSceneHandler(port);
    // let args: Vec<String> = env::args().collect();
    
    // let port: u32 = args[0].trim().parse().unwrap();
    let _server: () = start_server(port, *BUFFER_SIZE);
}
