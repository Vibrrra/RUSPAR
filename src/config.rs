
// Audio Device
pub static TARGET_AUDIO_DEVICE: &str = "StudioLive AR ASIO";
pub static SAMPLE_RATE: u32 = 48000;
pub static BUFFER_SIZE: u32 = 64;
// Config structure
pub static IMAGE_SOURCE_METHOD_ORDER: usize = 3;
pub const C: f32 = 343.0;
pub const MAX_SOURCES: usize = 10;

// Audio files for testing
pub static audio_file_list: [&str; MAX_SOURCES] = [r"E:\RustProjects\Assets\_Stimuli__speech__HarvardMale.wav",
                                          "1","2","3","4","5","6","7","8","9"]; 

