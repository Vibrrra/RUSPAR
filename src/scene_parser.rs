use crate::audioSceneHandlerData::Scene_data;
use crate::testtesttest::{self, AudioSceneHandle};

pub fn update_scene(scene: &Scene_data, audio_scene_handle: &mut AudioSceneHandle) {
    
    audio_scene_handle.sources.iter_mut().zip(scene.sources.iter()).for_each(|(ashs, ss)| { 

    });
    unimplemented!();
}
