use std::ops::Bound;

use nalgebra::{dimension, Point3, Quaternion, Vector3};
use num_traits::Zero;
use protobuf::reflect;
use strum_macros::EnumIter;

use crate::{
    audioSceneHandlerData::{Listener, Scene_data, Transform},
    scene::{get_position, get_quaternion},
};

static N_IS_INDEX_RANGES: [(usize, usize); 7] = [
    (0, 0),
    (0, 6),
    (6, 36),
    (36, 186),
    (186, 936),
    (936, 4686),
    (4686, 23436),
];

#[derive(Debug, Default, Clone, Copy, EnumIter, PartialEq, Eq)]
pub enum CardinalDirection {
    EAST,  // x = 0
    NORTH, // z =
    SOUTH, //
    WEST,
    FLOOR,
    CEILING,
    #[default]
    NONE,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Boundary {
    direction: CardinalDirection,
    location: f32,
    material: f32, // needs an implementation
}

impl Boundary {
    pub fn new(direction: CardinalDirection, location: f32, material: f32) -> Self {
        Self {
            direction,
            material,
            location,
        }
    }
    pub fn get_direction(&self) -> CardinalDirection {
        self.direction
    }
    pub fn get_material(&self) -> f32 {
        self.material
    }
}

#[derive(Debug, Default)]
pub struct ISMRoom {
    dimensions: Vector3<f32>,
    boundaries: [Boundary; 6],
}

impl ISMRoom {
    pub fn new(dimensions: Vector3<f32>, materials: [f32; 6], speed_of_sound: f32) -> Self {
        let boundaries: [Boundary; 6] = [
            Boundary::new(CardinalDirection::EAST, dimensions[0], materials[0]),
            Boundary::new(CardinalDirection::WEST, 0.0, materials[0]),
            Boundary::new(CardinalDirection::SOUTH, 0.0, materials[0]),
            Boundary::new(CardinalDirection::NORTH, dimensions[2], materials[0]),
            Boundary::new(CardinalDirection::FLOOR, 0.0, materials[0]),
            Boundary::new(CardinalDirection::CEILING, dimensions[1], materials[0]),
        ];
        Self {
            boundaries,
            dimensions,
        }
    }
    pub fn from_scene_data(scene_data: &Scene_data) -> Self {
        let dimensions = Vector3::from_vec(vec![
            scene_data.room.width,
            scene_data.room.length,
            scene_data.room.height,
        ]);
        let materials = [0.0f32; 6];
        let speed_of_sound = 343.0f32; // CHANGE THIS
        ISMRoom::new(dimensions, materials, speed_of_sound)
    }
    pub fn get_boundaries(&self) -> [Boundary; 6] {
        self.boundaries
    }
}

#[derive(Debug, Default)]
pub struct ISMListener {
    position: Point3<f32>,
    orientation: Quaternion<f32>,
}

impl ISMListener {
    pub fn new(position: Point3<f32>, orientation: Quaternion<f32>) -> Self {
        Self {
            position,
            orientation,
        }
    }
    pub fn from_scene_data(scene_data: &Scene_data) -> Self {
        Self {
            position: get_position(&scene_data.listener.transform),
            orientation: get_quaternion(&scene_data.listener.transform),
        }
    }
}

#[derive(Debug, Default)]
pub struct ISMSoundSource {
    position: Point3<f32>,
    orientation: Quaternion<f32>,
    reflector: CardinalDirection,
}
impl ISMSoundSource {
    pub fn new(position: Point3<f32>, orientation: Quaternion<f32>) -> Self {
        Self {
            position,
            orientation,
            reflector: CardinalDirection::NONE,
        }
    }
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            position: get_position(transform),
            orientation: get_quaternion(transform),
            reflector: CardinalDirection::NONE,
        }
    }
}
impl Source for ISMSoundSource {
    fn get_position(&self) -> Point3<f32> {
        self.position
    }
    fn update_position(&mut self, new_position: Point3<f32>) {
        self.position = new_position;
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct ISMImageSource {
    position: Point3<f32>,
    reflector: CardinalDirection,
    order: usize,
}
impl ISMImageSource {
    pub fn new(order: usize, position: Point3<f32>, reflector: CardinalDirection) -> Self {
        Self {
            position,
            reflector,
            order,
        }
    }

    pub fn init(&mut self, new_position: Point3<f32>, reflector: CardinalDirection, order: usize) {
        self.position = new_position;
        self.reflector = reflector;
        self.order = order;
    }

    pub fn get_reflector(&self) -> CardinalDirection {
        self.reflector
    }
}
impl Source for ISMImageSource {
    fn update_position(&mut self, new_position: Point3<f32>) {
        self.position = new_position;
    }
    fn get_position(&self) -> Point3<f32> {
        self.position
    }
}

pub trait Source {
    fn update_position(&mut self, new_position: Point3<f32>);
    fn get_position(&self) -> Point3<f32>;
}

#[derive(Debug, Default)]
pub struct ISMAcousticScene {
    room: ISMRoom,
    sound_sources: Vec<ISMSoundSource>,
    image_sources: Vec<Vec<ISMImageSource>>,
    listener: ISMListener,
    max_order: usize,
}

impl ISMAcousticScene {
    pub fn new(
        room: ISMRoom,
        listener: ISMListener,
        sound_sources: Vec<ISMSoundSource>,
        ism_max_order: usize,
    ) -> Self {
        let n_sources = sound_sources.len();
        let n_is_per_model = is_per_model(ism_max_order, room.boundaries.len());

        let mut image_sources: Vec<Vec<ISMImageSource>> =
            vec![vec![ISMImageSource::default(); n_is_per_model]; n_sources];

        for (i, snd_src) in sound_sources.iter().enumerate() {
            // first order
            for _ in 0..ism_max_order - (ism_max_order - 1) {
                for (n, boundary) in room.get_boundaries().iter().enumerate() {
                    let new_pos = reflect(snd_src, boundary);
                    image_sources[i][n].init(new_pos, boundary.get_direction(), 1);
                }
            }
            let mut n: usize = 6;
            for order in 1..ism_max_order {
                for is_idx in N_IS_INDEX_RANGES[order].0..N_IS_INDEX_RANGES[order].1 {
                    for boundary in room.get_boundaries().iter() {
                        if boundary.get_direction() != image_sources[i][is_idx].get_reflector() {
                            let new_position = reflect(&image_sources[i][is_idx], boundary);
                            image_sources[i][n].init(
                                new_position,
                                boundary.get_direction(),
                                order + 1,
                            );
                            n += 1;
                        }
                    }
                }
            }
        }
        Self {
            sound_sources,
            image_sources,
            room,
            listener,
            max_order: ism_max_order,
        }
    }
    pub fn default() -> Self {
        let listener: ISMListener =
            ISMListener::new(Point3::from_slice(&[0.0, 0.0, 0.0]), Quaternion::zero());
        let room: ISMRoom = ISMRoom::new(Vector3::from_vec(vec![0.0, 0.0, 0.0]), [0.0; 6], 343.0);
        let mut sound_sources: Vec<ISMSoundSource> = Vec::new();
        sound_sources.push(ISMSoundSource::new(
            Point3::from_slice(&[0.0, 0.0, 0.0]),
            Quaternion::new(0.0, 0.0, 0.0, 0.0),
        ));
        let image_sources = vec![vec![ISMImageSource::default()]];
        ISMAcousticScene {
            listener,
            room,
            sound_sources,
            image_sources,
            max_order: 2,
        }
    }

    pub fn from_scene_data(scene_data: &Scene_data) -> Self {
        let room = ISMRoom::from_scene_data(scene_data);
        let listener: ISMListener = ISMListener::from_scene_data(scene_data);
        let mut sound_sources = Vec::new();
        for source_transform in scene_data.sources.transforms {
            sound_sources.push(ISMSoundSource::from_transform(&source_transform));
        }

        ISMAcousticScene::new(room, listener, sound_sources, 2)
    }

    pub fn update(&mut self, new_source_positions: Vec<Point3<f32>>) {
        for (i, source) in self.sound_sources.iter_mut().enumerate() {
            source.update_position(new_source_positions[i]);
            // source -> first order
            for _ in 0..self.max_order - (self.max_order - 1) {
                for (n, boundary) in self.room.get_boundaries().iter().enumerate() {
                    let new_position = reflect(source, boundary);
                    self.image_sources[i][n].update_position(new_position);
                }
            }
            let mut n: usize = 6;
            // n-th order -> (n+1)-th order
            for order in 1..self.max_order {
                for is_idx in N_IS_INDEX_RANGES[order].0..N_IS_INDEX_RANGES[order].1 {
                    for boundary in self.room.get_boundaries().iter() {
                        if boundary.get_direction() != self.image_sources[i][is_idx].get_reflector()
                        {
                            let new_position = reflect(&self.image_sources[i][is_idx], boundary);
                            self.image_sources[i][n].update_position(new_position);
                            n += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn from_protobuf_scene(&mut self, scene_data: &Scene_data) {
        let mut new_positions: Vec<Point3<f32>> = Vec::new();
        for s in scene_data.sources.transforms.iter() {
            new_positions.push(Point3::new(s.position.x, s.position.y, s.position.z));
        }
        self.update(new_positions);
    }
}

fn reflect(source: &impl Source, boundary: &Boundary) -> Point3<f32> {
    let mut new_position = source.get_position();
    match boundary.get_direction() {
        CardinalDirection::EAST => {
            new_position[1] = 2.0 * boundary.location - new_position[1];
        }
        CardinalDirection::NORTH => {
            new_position[0] = 2.0 * boundary.location - new_position[0];
        }
        CardinalDirection::SOUTH => {
            new_position[0] = -new_position[0];
        }
        CardinalDirection::WEST => {
            new_position[1] = -new_position[1];
        }
        CardinalDirection::FLOOR => {
            new_position[2] = -new_position[2];
        }
        CardinalDirection::CEILING => {
            new_position[2] = 2.0 * boundary.location - new_position[2];
        }
        CardinalDirection::NONE => {
            panic!("(Image) Source has no reflector. That doesn't make any sense.")
        }
    }
    new_position
}

fn is_per_model(maxorder: usize, n_surfaces: usize) -> usize {
    let mut n_ism: usize = 0;
    for i in 1..=maxorder {
        n_ism += is_per_order(i as f64, n_surfaces as f64) as usize;
    }
    n_ism
}

fn is_per_order(order: f64, n_surfaces: f64) -> usize {
    ((n_surfaces) * (n_surfaces - 1f64).powf(order - 1f64)).floor() as usize
}
