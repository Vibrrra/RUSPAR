use std::f32::consts::PI;

use nalgebra::{Vector3, Quaternion, UnitQuaternion};

use crate::audioSceneHandlerData::{Scene_data, Transform};
use crate::image_source_method::{self, update_source_tree, update_source_tree_from_roots, Listener, Room, Source, SourceTrees, SourceType, SphericalCoordinates};

pub fn update_scene(scene: &Scene_data, source_trees: &mut SourceTrees<Source>)
 
{
    
    let listener_transform: Transform = scene.listener.transform.clone().unwrap();
    let listener: image_source_method::Listener = Listener {
        position: Vector3::new(listener_transform.position.x, listener_transform.position.y,listener_transform.position.z),
        orientation: get_quaternion(&listener_transform),
    };

    // update trees
    scene.sources.transforms.iter().zip(source_trees.roots.iter()).zip(source_trees.arenas.iter_mut()).for_each(|((transform, node_id), arena)| {
        let src: &mut Source = arena.get_mut(*node_id).unwrap().get_mut();
        src.set_pos(Vector3::new(transform.position.x,transform.position.y,transform.position.z));
        // src.position.x = transform.position.x;  
        // src.position.y = transform.position.y;  
        // src.position.z = transform.position.z;  
        
        // (src.dist, src.listener_source_orientation.azimuth, src.listener_source_orientation.elevation) = calculate_azimuth_and_elevation_with_rotation(&listener_transform, transform);
        update_lst_src_orientation(&listener_transform, transform, src);
        update_src_lst_orientation( transform,&listener_transform, src);
        (_, src.source_listener_orientation.azimuth, src.source_listener_orientation.elevation) = calculate_azimuth_and_elevation_with_rotation(transform, &listener_transform);
    });
    let room: Room = Room{ dimension: Vector3::new(scene.room.width, scene.room.height, scene.room.length)
    
    };

    // update_source_tree_from_roots(source_trees, &room);
    update_source_tree_from_roots(source_trees,  &room, &listener);
}


fn update_lst_src_orientation(transform_a: &Transform, transform_b: &Transform, src: &mut Source) {
    let temp = calculate_azimuth_and_elevation_with_rotation(&transform_a, transform_b);
    src.set_lst_src_transform(SphericalCoordinates::new(temp.1, temp.2));
} 
fn update_src_lst_orientation(transform_a: &Transform, transform_b: &Transform, src: &mut Source) {
    let temp = calculate_azimuth_and_elevation_with_rotation(&transform_a, transform_b);
    src.set_src_lst_transform(SphericalCoordinates::new(temp.1, temp.2));
} 

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