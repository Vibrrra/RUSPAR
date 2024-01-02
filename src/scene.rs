use nalgebra::{Point3, Quaternion, UnitQuaternion, Vector3};

use crate::audioSceneHandlerData::{Listener, Scene_data, Transform};

pub fn update_scene_parameters(scene_data: Scene_data) {
    let listener: Listener = scene_data.listener.unwrap();
    let listener_tranform: &Listener = &listener;
    // let sources: Vec<crate::audioSceneHandlerData::Transform> = scene_data.sources.transforms;

    let mut source_positions: Vec<(f32, f32, f32)> =
        Vec::with_capacity(scene_data.sources.transforms.capacity());
    let mut source_listener_orientation: Vec<(f32, f32, f32)> =
        Vec::with_capacity(scene_data.sources.transforms.capacity());

    for src in &scene_data.sources.transforms {
        source_positions.push(calculate_azimuth_and_elevation_with_rotation(
            &listener.transform,
            src,
        ));
        source_listener_orientation.push(calculate_azimuth_and_elevation_with_rotation(
            src,
            &listener.transform,
        ));
    }
}

pub fn get_position(t: &Transform) -> Point3<f32> {
    Point3::from_slice(&[t.position.x, t.position.y, t.position.z])
}

pub fn get_quaternion(t: &Transform) -> Quaternion<f32> {
    let w = t.orientation.w;
    let i = t.orientation.x;
    let j = t.orientation.y;
    let k = t.orientation.z;
    Quaternion::new(w, i, j, k)
}

fn calculate_azimuth_and_elevation_with_rotation(a: &Transform, b: &Transform) -> (f32, f32, f32) {
    //Rotation<f32,3> {
    // Calculate relative position vector from A to B in world frame

    let relative_position: Vec<f32> = vec![
        b.position.x - a.position.x,
        b.position.y - a.position.y,
        b.position.z - a.position.z,
    ];

    // Transform the relative position vector to the local frame of A
    // let get_quaternion = getQuaternion(a);
    let a_quat = get_quaternion(&a);
    // let b_quat = getQuaternion(b);
    let a_uquat = UnitQuaternion::from_quaternion(a_quat);

    let temp1 = a_uquat.transform_vector(&Vector3::<f32>::from_vec(relative_position));
    let op = temp1.data.0[0];
    cartesian_to_spherical(op)
}

fn cartesian_to_spherical(a: [f32; 3]) -> (f32, f32, f32) {
    let r = (a[0].powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt();
    let azimuth = a[0].atan2(a[2]);
    let elevation = a[1].atan2((a[2].powi(2) + a[0].powi(2)).sqrt());

    (r, azimuth, elevation)
}
