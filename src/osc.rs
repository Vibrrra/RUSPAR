use std::{net::{UdpSocket, SocketAddrV4}, str::FromStr};

use rosc::OscPacket;




pub enum OscChannelType {
    SceneData(String),
    TransportCmd(TransportCmdType)
}
pub enum TransportCmdType {
    Play(String),
    Pause(String),
    Mute(String),
    Exit(String),
    Stop(String)
}


#[allow(unused)]
pub struct OSCHandler {   
   address: SocketAddrV4,
   sock: UdpSocket,
   buf: [u8; 2048],
}

impl OSCHandler {
    pub fn new(ip_addr: &str) -> Self {
        
        let address: SocketAddrV4 = match SocketAddrV4::from_str(ip_addr) {
            Ok(addr) => addr,
            Err(_) => panic!("{ip_addr} is no valid IP Adrees. [Usage:  127.0.0.1:8000]")
        };        
        let sock: UdpSocket = UdpSocket::bind(address).unwrap();
        OSCHandler {
           address,
           sock, 
           buf: [0; 2048]
        }
    }

    pub fn try_recv(&mut self) -> Vec<u8> {
        match self.sock.recv_from(&mut self.buf) {
            Ok((size, _addr)) => {
                let (_, osc_packet) = rosc::decoder::decode_udp(&self.buf[..size]).unwrap();
                let byte_string: Vec<u8> = OSCHandler::handle_osc_packet(osc_packet);
                byte_string 
            },
            Err(_) => todo!(),
        }

    }

    fn handle_osc_packet(packet: OscPacket) -> Vec<u8> {
        match packet {
            OscPacket::Bundle(_bundle) => panic!("OSC Pcket of type Bundle not (yet) supported!"),
            OscPacket::Message(message) => {
                message.args[0].clone().blob().expect("Expected OscMessage found None")               
            },
        }
    }
}