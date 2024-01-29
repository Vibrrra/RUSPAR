use std::{f32::consts::PI, vec};

use nalgebra::{Quaternion, UnitQuaternion, Vector2, Vector3};

use crate::{audioSceneHandlerData::{Scene_data, Sources, Transform}, audio_module::ISMDelayLine, config::IMAGE_SOURCE_METHOD_ORDER, convolver::Spatializer, delaylines::DelayLine, image_source_method::{calc_ism_position, create_ism, Listener, Reflected, Room, Source, SourceType, SphericalCoordinates, N_IS_INDEX_RANGES}};

// Lookup table for iterating over the corresponging image sources (order)
// in the vectors
pub const ISM_INDEX_RANGES: [(usize, usize, usize); 7] = [
    (0, 1, 7),
    (1, 7, 37),
    (2, 37, 67),
    (3, 67, 97),
    (4, 97, 127),
    (5, 127, 157),
    (6, 157, 187), 
];

// 
pub struct ISMDelayLines {
    pub sources: Vec<Vec<ISMDelayLine>>,
}

impl ISMDelayLines {
    pub fn new(sources: IMS, room: &Room, c: f32, sample_rate: f32, buffer_size: usize, ims_order: usize, spatializer: Spatializer) -> Self {
        let max_distance: f32 = (room.dimension.x.powi(2)+
                                 room.dimension.y.powi(2)+
                                 room.dimension.z.powi(2)).sqrt(); 
        let max_length: usize = (max_distance / c * sample_rate).ceil() as usize;
        let mut v = Vec::new();
        for i in 0..sources.sources.len() {
            let mut v2 = Vec::new();
            let ism_delayline: ISMDelayLine = ISMDelayLine::new(max_length, buffer_size, spatializer.clone());
            for i2 in 0..sources.sources[0].len() {
                // let ism_delayline = ISMDelayLine::new(max_delay, buffer_size);
                v2.push(ism_delayline.clone());
            }
            v.push(v2);
        }
        Self {
            sources: v,
        }
    }
}


#[derive(Clone)]
pub struct IMS {
    pub sources: Vec<Vec<Source>>,
    ism_order: usize,
}
impl IMS {
    pub fn create_raw(num_sources: usize) -> Self {
        // let sources = Vec::with_capacity(187);
        let sources: Vec<Vec<Source>> = 
            (0..num_sources).into_iter().map(|_| {
                (0..187usize).into_iter().map(|_| Source::default()).collect()
            }).collect();

        Self {
            sources,
            ism_order: 2,
           
        }
    }
    pub fn init_from_scene(scene: Scene_data, ism_order: usize) -> Self {
        let mut sources: Vec<Vec<Source>> = Vec::new();
        let listener_transform: Transform = scene.listener.transform.clone().unwrap();
        let listener: Listener = Listener {
            position: Vector3::new(listener_transform.position.x, listener_transform.position.y, listener_transform.position.z),
            orientation: get_quaternion(&listener_transform),
        };
        let mut scene_iter = scene.sources.transforms.iter();
        let room: Room = Room {dimension: Vector3::new(scene.room.width, scene.room.height, scene.room.length)};
        // iter over all sources
        while let Some(scene_src) = scene_iter.next() {
            let mut source_vec = Vec::new();
            // init root source
            let mut src: Source = Source::default();
            src.set_pos(Vector3::new(scene_src.position.x, scene_src.position.y, scene_src.position.z));
            src.reflector = Reflected::No;
            src.set_dist(calc_distance(&src.get_pos(), &listener.position));
            src.set_remaining_dist(src.get_dist());
            update_lst_src_orientation(&listener_transform, scene_src, &mut src);
            update_src_lst_orientation( scene_src, &listener_transform,&mut src);
            source_vec.push(src);
            for parent_idx in 0 .. ISM_INDEX_RANGES.len() {
                let parent_src = source_vec[ISM_INDEX_RANGES[parent_idx].0].clone();
                for boundary in Reflected::VALUES {
                    let mut src = create_ism(&parent_src, &room, &boundary);
                    src.set_dist(calc_distance(&src.get_pos(), &listener.position));
                    src.set_remaining_dist(src.get_dist()-parent_src.get_dist());
                    update_lst_src_orientation(&listener_transform, scene_src, &mut src);
                    update_src_lst_orientation( scene_src, &listener_transform,&mut src);           
                    source_vec.push(src);
                }
            }
            sources.push(source_vec);
        }
        Self {
            sources,
            ism_order
        } 
    }

    pub fn update_from_scene(&mut self, scene: Scene_data) {

        let listener_transform: Transform = scene.listener.transform.clone().unwrap();
        let listener: Listener = Listener {
            position: Vector3::new(listener_transform.position.x, listener_transform.position.y, listener_transform.position.z),
            orientation: get_quaternion(&listener_transform),
        };
        let mut scene_iter = scene.sources.transforms.iter();
        let room: Room = Room {dimension: Vector3::new(scene.room.width, scene.room.height, scene.room.length)};
        // iter over all sources
        for mut sources in &mut self.sources {
            let mut scene_src = scene_iter.next().unwrap();
            // update non-image sources
            sources[0].set_pos(Vector3::new(scene_src.position.x, scene_src.position.y, scene_src.position.z));
            update_lst_src_orientation(&listener_transform, scene_src, &mut sources[0]);
            update_src_lst_orientation( scene_src, &listener_transform,&mut sources[0]);
            
            for parent_idx in 0 .. ISM_INDEX_RANGES.len() {
                
                let idx_start: usize = ISM_INDEX_RANGES[parent_idx].1;
                let idx_stop: usize = ISM_INDEX_RANGES[parent_idx].2;
                let parent_src: &Source = &sources[ISM_INDEX_RANGES[parent_idx].0];
                let parent_pos = &parent_src.get_pos();
                let parent_dist = parent_src.get_dist();
                
                for i in idx_start .. idx_stop {
                    assert!(sources.len() <= idx_stop);
                    let src = &mut sources[i];
                    src.set_pos(calc_ism_position(parent_pos, &room, src.get_reflector()));
                    src.set_dist(calc_distance(&src.get_pos(),&listener.get_pos()));
                    src.set_remaining_dist(src.get_dist()-parent_dist);
                }
            }
        }     
    } 
}

fn update_lst_src_orientation(transform_a: &Transform, transform_b: &Transform, src: &mut Source) {
    let temp = calculate_azimuth_and_elevation_with_rotation(&transform_a, transform_b);
    src.set_lst_src_transform(SphericalCoordinates::new(temp.1, temp.2));
} 
fn update_src_lst_orientation(transform_a: &Transform, transform_b: &Transform, src: &mut Source) {
    let temp = calculate_azimuth_and_elevation_with_rotation(&transform_a, transform_b);
    src.set_src_lst_transform(SphericalCoordinates::new(temp.1, temp.2));
} 

// Utility Functions
pub fn calculate_azimuth_and_elevation_with_rotation(a: &Transform, b: &Transform) -> (f32, f32, f32) {//Rotation<f32,3> {
    // Calculate relative position vector from A to B in world frame

    let relative_position: Vec<f32> = vec![b.position.x - a.position.x,
                                           b.position.y - a.position.y,
                                           b.position.z - a.position.z] ;

    // Transform the relative position vector to the local frame of A
    let a_quat = get_quaternion(a);
    // let b_quat = getQuaternion(b);
    let a_uquat = UnitQuaternion::from_quaternion(a_quat);
    

    let temp1= a_uquat.transform_vector(&Vector3::<f32>::from_vec(relative_position));
    let op =temp1.data.0[0];
    cartesian_to_spherical(op)
}

pub fn calc_lst_src_orientation(listener: &Listener, src_position: Vector3<f32>) -> SphericalCoordinates {
    let relative_position: Vec<f32> = vec![src_position.x - listener.position.x,
                                           src_position.y - listener.position.y,
                                           src_position.z - listener.position.z] ;
    let a_quat = UnitQuaternion::from_quaternion(listener.orientation);
    let temp = a_quat.transform_vector(&Vector3::<f32>::from_vec(relative_position));    
    let op =temp.data.0[0];
    let sph = cartesian_to_spherical(op);
    SphericalCoordinates::new(sph.1, sph.2)
}

pub fn calc_src_lst_orientation<T: SourceType<Source>>(source: &T, lst_position: Vector3<f32>) -> SphericalCoordinates {
    let position = source.get_pos();
    let relative_position: Vec<f32> = vec![lst_position.x - position.x,
                                           lst_position.y - position.y,
                                           lst_position.z - position.z] ;
    let a_quat = UnitQuaternion::from_quaternion(source.get_orientation());
    let temp = a_quat.transform_vector(&Vector3::<f32>::from_vec(relative_position));    
    let op =temp.data.0[0];
    let sph = cartesian_to_spherical(op);
    SphericalCoordinates::new(sph.1, sph.2)
}

fn cartesian_to_spherical(a: [f32; 3]) -> (f32, f32, f32) {
    let r = (a[0].powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt();
    let azimuth = a[0].atan2(a[2]);
    let elevation =  a[1].atan2( (a[2].powi(2) + a[0].powi(2)).sqrt());

   (r, rad2deg(azimuth).rem_euclid(360.0), rad2deg(elevation))
}

fn rad2deg(rad: f32) -> f32 {
    rad * 360.0 / 2.0 / PI
}

fn get_quaternion(t: &Transform) -> Quaternion<f32> {
    let w = t.orientation.w;
    let i = t.orientation.x;
    let j = t.orientation.y;
    let k = t.orientation.z;

    Quaternion::new(w, i, j, k)
}
fn calc_distance(v1: &Vector3<f32>, v2: &Vector3<f32>) -> f32 {
    ((v1.x-v2.x).powi(2) + 
     (v1.y-v2.y).powi(2) + 
     (v1.z-v2.z).powi(2)).sqrt()
}