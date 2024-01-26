use std::fmt::Error;

use cpal::{HostId, Device, traits::{HostTrait, DeviceTrait}, HostUnavailable, SampleFormat, StreamConfig, Host};


pub fn get_input_device(host_id: HostId, device_name: &str) -> Result<Device, Error> {

    let host = cpal::host_from_id(host_id).expect("Could not get Host from HostId");   
    let devices = host.devices().unwrap();   
    let devices = host.devices().unwrap();
    let mut device = host.default_input_device().unwrap(); 
    for dev in devices {
        if dev.name().unwrap().eq_ignore_ascii_case(device_name.trim()) {
            device = dev;
        }        
    }
    Ok(device)
}
pub fn get_output_device(host_id: HostId, device_name: &str) -> Result<Device, Error> {

    let host = cpal::host_from_id(host_id).expect("Could not get Host from HostId");   
    let devices = host.devices().unwrap();   
    let devices = host.devices().unwrap();
    let mut device = host.default_output_device().unwrap(); 
    for dev in devices {
        if dev.name().unwrap().eq_ignore_ascii_case(device_name.trim()) {
            device = dev;
        }        
    }
    Ok(device)
}

pub fn get_input_sample_format(device: Device) -> SampleFormat {
    device.default_input_config().unwrap().sample_format()
}
pub fn get_output_sample_format(device: Device) -> SampleFormat {
    device.default_input_config().unwrap().sample_format()
}

pub fn list_hosts() {
    let available_hosts = cpal::available_hosts();
    available_hosts.iter().for_each(|_| {println!("available hosts")});

    let mut available_host = Vec::new(); 
    for (i,host_id) in available_hosts.iter().enumerate() {
        let host = cpal::host_from_id(*host_id);
        available_host.push(host.unwrap());

    }
    
    for  (i, h) in available_host.iter().enumerate() {
        println!("[{}.]: {:#?}", i, h.id().name());
        list_devices(h);
    }    
}

pub fn list_devices(host: &Host) {
    let devices = host.devices().unwrap();
    for (i, dev) in devices.enumerate() {
        println!("[{}] {}", i, dev.name().unwrap());
    }
}

// pub fn check_input_configuration_valid(config: StreamConfig, device: Device, ) -> bool {
//     let supported_configs = device.supported_input_configs();
//     for supported_config in supported_configs {
//         // let s_config: StreamConfig = supported_config.into();
//         for sc in supported_config {
//             if (config.channels < sc.channels() & config.buffer_size sc.buffer_size()) {} 
//         }
//         if supported_config == config {
//             return true;
//         }
//     }
//     false
// }
// pub fn get_device_from_name(host_str: &str) -> device_name {

// }



#[cfg(test)]
#[test]
fn test_list_host() {
    list_hosts();
}