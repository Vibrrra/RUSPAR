use nalgebra::{Vector3, Quaternion, UnitQuaternion};

use crate::audioSceneHandlerData::{Scene_data, Transform};
use crate::image_source_method::{SourceTrees, Room, update_source_tree_from_roots};

pub fn update_scene(scene: &Scene_data, source_trees: &mut SourceTrees) {
    
    scene.sources.transforms.iter().zip(source_trees.roots.iter()).zip(source_trees.arenas.iter_mut()).for_each(|((transform, nodeId), arena)| {
        let src = arena.get_mut(*nodeId).unwrap().get_mut();
        src.position.x = transform.position.x;  
        src.position.y = transform.position.y;  
        src.position.z = transform.position.z;  
        let listener = scene.listener.transform.clone().unwrap();
        (src.dist, src.listener_source_orientation.azimuth, src.listener_source_orientation.elevation) = calculate_azimuth_and_elevation_with_rotation(&listener, transform);
        (_, src.source_listener_orientation.azimuth, src.source_listener_orientation.elevation) = calculate_azimuth_and_elevation_with_rotation(transform, &listener);
    });
    let room: Room = Room{ dimension: Vector3::new(scene.room.width, scene.room.height, scene.room.length)
    };
    update_source_tree_from_roots(source_trees, &room);
}


fn calculate_azimuth_and_elevation_with_rotation(a: &Transform, b: &Transform) -> (f32, f32, f32) {//Rotation<f32,3> {
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

fn cartesian_to_spherical(a: [f32; 3]) -> (f32, f32, f32) {
    let r = (a[0].powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt();
    let azimuth = a[0].atan2(a[2]);
    let elevation =  a[1].atan2( (a[2].powi(2) + a[0].powi(2)).sqrt());

   (r, azimuth, elevation)
}

fn get_quaternion(t: &Transform) -> Quaternion<f32> {
    let w = t.orientation.w;
    let i = t.orientation.x;
    let j = t.orientation.y;
    let k = t.orientation.z;

    Quaternion::new(w, i, j, k)
}