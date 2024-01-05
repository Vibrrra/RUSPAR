use crate::buffers::CircularDelayBuffer;
use nalgebra::Vector3;
use nalgebra::Quaternion;
use num_traits::Zero;
use indextree::Arena;
use strum_macros::EnumIter;

// index ranges for image sources vectors
static N_IS_INDEX_RANGES: [(usize, usize); 4] = [
    (0, 1),
    (1, 7),
    (7, 37),
    (37, 187) 
];


// Enum for ISM Algorithm
// "No" => True Sound Source
// "X0 - Z1" => Reflected on respective shoebox boundary
#[derive(Debug, Default, Clone, EnumIter, PartialEq, Eq)]
pub enum Reflected {
    X0,
    X1,
    Y0,
    Y1,
    Z0,
    Z1,
    #[default]
    No,
}

// impl enum to be iteratable  
// src: https://stackoverflow.com/questions/21371534/in-rust-is-there-a-way-to-iterate-through-the-values-of-an-enum
impl Reflected {
    const VALUES: [Self; 6] = [Self::X0, Self::X1, Self::Y0, Self::Y1, Self::Z0, Self::Z1];
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Room {
    dimension: Vector3<f32>,
}

#[allow(dead_code)]
impl Room {
    fn new(width: f32, height: f32, length: f32) -> Self {
        Self {
            dimension: Vector3::new(width, height, length),
        }
    }
    fn diagonal(&self) -> f32 {
        (self.dimension.x.powi(2) + self.dimension.y.powi(2) + self.dimension.z.powi(2)).sqrt()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Source {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub source_listener_orientation: Quaternion<f32>,
    pub buffer: CircularDelayBuffer,
    pub dist: f32,
    pub reflector: Reflected,
}

#[allow(dead_code)]
impl Source {
    pub fn new(
        position: Vector3<f32>,
        orientation: Quaternion<f32>, 
        room: &Room,
        speed_of_sound: f32,
        sample_rate: f32,
        reflector: Option<Reflected>,
        list: Option<Listener>,        
    ) -> Self {
        let mut source_listener_orientation = Quaternion::zero();
        let dist = if let Some(x) = list {
            // source_listener_orientation = todo!();       
            calc_distance(&x.position, &position)
        } else {
            0.0
        };
        let refl = if let Some(r) = reflector {
            r
        } else {
            Reflected::No
        };
        
        Self {
            position,
            orientation,
            buffer: CircularDelayBuffer::new(
                (sample_rate * room.diagonal() / speed_of_sound).ceil() as usize,
            ),
            dist,
            reflector: refl,
            source_listener_orientation,
        }
    }
    pub fn update_position(&mut self, position: Vector3<f32>, listener: &Listener) {
        self.position = position;
        self.dist = calc_distance(&self.position, &listener.position);
        self.buffer.set_delay_time_samples(48000.0 * self.dist / 343.0f32);
    }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Listener {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
}

// Experimental Source View for ISM
#[allow(dead_code)]
#[derive(Debug, Default)]
struct ListenerSourceView {
    dist: f32,
    az: f32,
    el: f32
}
impl ListenerSourceView {
    pub fn create_from(s: &Source, l: &Listener) -> Self {
        let dist = calc_distance(&s.position,&l.position);
        todo!();

    }
}

struct AudioSceneHandle {
    sources: Vec<Source>,
    listenerView: Vec<ListenerSourceView>,
    ims_order: usize,
    
}

// calc ISMs
pub fn generate_image_source_tree(sources: Vec<Source>, room: &Room, ism_order: usize) -> Vec<Source> {
    
    // create Tree Arena
    let mut arena: Arena<&Source> = Arena::new();
    let mut root_nodes: Vec<indextree::NodeId> = Vec::new();
    // let mut node_list: Vec<indextree::NodeId> = Vec::new();
    let mut source_list: Vec<Source> = Vec::new();
    let mut current_ism_order: usize = 1;
    let root_offset = 
    // for all n real sources
    for n in 0..sources.len() {
        
        // Add source to list! 
        source_list.push(sources[n].clone());
        
        // Calc image sources
        for order in 0..ism_order {
            for i in N_IS_INDEX_RANGES[order].0 .. N_IS_INDEX_RANGES[order].1 {
                
                for boundary in Reflected::VALUES {
                    if source_list[i].reflector != boundary {
                        source_list.push(create_reflection(&source_list[i], room, &boundary));
                    }
                }
            }
        }
    };
    
    source_list
}

// This function creates an image source from reflecting a source
// on an respective room boundary
fn create_reflection(s: &Source, r: &Room, b: &Reflected) -> Source {   
    
    let mut position = s.position;
    match b {
        Reflected::X0 => {position.x = -position.x},
        Reflected::X1 => {position.x = 2.0 * r.dimension.x - position.x},
        Reflected::Y0 => {position.y = -position.y},
        Reflected::Y1 => {position.y = 2.0 * r.dimension.y - position.y},
        Reflected::Z0 => {position.z = -position.z},
        Reflected::Z1 => {position.z = 2.0 * r.dimension.z - position.z},
        Reflected::No => {panic!("This should not happen!")},
    } 
    Source::new(
        position, 
        Quaternion::zero(), 
        r, 
        343.0, 
        48000.0, 
        Some(b.clone()), None
    )
}

#[cfg(test)]
#[test]
fn test_bufs() {
    
    // Init things
    use indextree::Arena;
    use nalgebra::Vector3;
    use num_traits::Zero;

    // use Listener;
    let speed_of_sound: f32 = 343.0;
    let sample_rate: f32 = 48000.0;
    let room: [f32; 3] = [5.0, 3.0, 6.0];
    let max_delay = (sample_rate * (room.iter().fold(0.0, |acc, x| acc + x.powi(2))).sqrt()
        / speed_of_sound)
        .round() as usize
        + 200;
    let mut lis = Listener {
        position: Vector3::default(),
        orientation: Quaternion::zero(),
    };

    let mut s1 = Source {
        position: Vector3::default(),
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        reflector: Reflected::No,
        dist: 0.0,
        source_listener_orientation: Quaternion::zero(),
    };

    let mut s1_is_a = Source {
        position: Vector3::default(),
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        reflector: Reflected::No,
        dist: 0.0,
        source_listener_orientation: Quaternion::zero(),
    };

    let s1_is_ab = Source {
        position: Vector3::default(),
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        reflector: Reflected::No,
        dist: 0.0,
        source_listener_orientation: Quaternion::zero(),
    };

    // update position
    lis.position = Vector3::<f32>::new(1.0, 1.5, 2.0);
    s1.position = Vector3::<f32>::new(2.0, 1.5, 4.0);
    s1.dist = calc_distance(&lis.position, &s1.position);
    s1.buffer
        .set_delay_time_samples(sample_rate * s1.dist / 343.0);
    s1_is_a.position = Vector3::new(-2.0, 1.5, 2.0);
    s1_is_a.dist = calc_distance(&lis.position, &s1_is_a.position);
    s1_is_a
        .buffer
        .set_delay_time_samples(sample_rate * (s1_is_a.dist - s1.dist) / 343.0);

    let out: Vec<f32> = vec![0.0; 48000];
    let mut input: Vec<f32> = out;
    input[0] = 1.0;
    let mut temp = Vec::new();
    for i in 0..input.len() {
        let mut sout = 0.0;
        let sample = input[i];
        sout = s1.buffer.read();
        s1.buffer.write(sample);
        let s1_out = s1_is_a.buffer.read();
        s1_is_a.buffer.write(sout);
        sout += s1_out;

        if sout > 0.0 {
            temp.push(i as f32);
        }
        // print!("{:?}|", sout);
    }
    println!("Pulses found at: {:?}", temp);
    let arena = &mut Arena::new();
    let mut node_ids = Vec::new();

    let root = arena.new_node(s1);
    node_ids.push(arena.new_node(s1_is_a));
    node_ids.push(arena.new_node(s1_is_ab));

    root.append(node_ids[0], arena);
    node_ids[0].append(node_ids[1], arena);

    //root.children(arena).for_each(|x| {dbg!(arena.get(x));});

    //
}

// good ol' pythagoras
fn calc_distance(v1: &Vector3<f32>, v2: &Vector3<f32>) -> f32 {
    ((v1.x-v2.x).powi(2) + 
     (v1.y-v2.y).powi(2) + 
     (v1.z-v2.z).powi(2)).sqrt()
}

#[test]
fn test_ism_creation() {
    let ism_order: usize = 3;
    let speed_of_sound: f32 = 343.0;
    let sample_rate: f32 = 48000.0;
    let room: Room = Room::new(4.0, 3.0, 5.0);
    let listener = Listener::default();
    let ssrc: Source = Source::new(Vector3::new(2.0, 1.0, 2.0), Quaternion::zero(), &room, speed_of_sound, sample_rate, None, Some(listener));

    let src_list = generate_image_source_tree(vec![ssrc], &room, ism_order);

    for i in src_list {
        println!("{:?}", i.position);
    }
}

#[test]
// insanity check!
fn test_vector3() {
    let v1 = Vector3::new(3.0, 4.0, 0.0);
    let v2 = Vector3::new(0.0, 0.0, 0.0);
    println!("{:?}", calc_distance(&v1, &v2));
}

