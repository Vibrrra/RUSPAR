use crate::audioSceneHandlerData::Scene_data;
use crate::audioSceneHandlerData::Transform;
use crate::buffers::CircularDelayBuffer;
use indextree::NodeId;
use nalgebra::Vector3;
use nalgebra::Quaternion;
use nalgebra::zero;
use num_traits::Zero;
use indextree::Arena;
use strum_macros::EnumIter;

// index ranges for image sources vectors
pub const N_IS_INDEX_RANGES: [(usize, usize); 4] = [
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
#[derive(Debug, Default)]
pub struct Room {
    pub dimension: Vector3<f32>,
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

#[derive(Debug, Default, Clone, Copy)]
pub struct SphericalCoordinates {
    pub azimuth: f32,
    pub elevation: f32
}
impl SphericalCoordinates {
    pub fn new(azimuth: f32, elevation: f32) -> Self {
        Self { azimuth, elevation}
    }   
}



#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Source {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub source_listener_orientation: SphericalCoordinates,//Quaternion<f32>,
    pub listener_source_orientation: SphericalCoordinates,
    // pub buffer: CircularDelayBuffer,
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
        list: Option<&Listener>,        
    ) -> Self {
        let mut source_listener_orientation = SphericalCoordinates::default();//Quaternion::zero();
        let mut listener_source_orientation = SphericalCoordinates::default();//Quaternion::zero();
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
            // buffer: CircularDelayBuffer::new(
                // (sample_rate * room.diagonal() / speed_of_sound).ceil() as usize,
            // ),
            dist,
            reflector: refl,
            source_listener_orientation,
            listener_source_orientation
        }
    }
    pub fn update_position(&mut self, position: Vector3<f32>, listener: &Listener) {
        self.position = position;
        self.dist = calc_distance(&self.position, &listener.position);
        // self.buffer.set_delay_time_samples(48000.0 * self.dist / 343.0f32);
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

#[derive(Debug, Clone)]
pub struct SourceTrees {
    pub arenas: Vec<Arena<Source>>,
    pub node_lists: Vec<Vec<NodeId>>,
    pub roots: Vec<NodeId>
}
impl SourceTrees {
    pub fn create(number_of_sources: usize, ism_order: usize) -> Self {
        let mut sources = Vec::new();
        for i in 0 .. number_of_sources {
            sources.push(Source::default());
        }
        create_source_tree(sources, &Room::default(), ism_order)
    }
}
// 
pub fn create_source_tree(sources: Vec<Source>, room: &Room, ism_order: usize) -> SourceTrees {//(Arena<Source>, Vec<NodeId>) {
    
    let mut node_lists = Vec::new();
    let mut source_tree_vec = Vec::new(); 
    let mut roots = Vec::new(); 
    for n in 0 .. sources.len() {
        let mut arena = indextree::Arena::new();
        let mut node_list = Vec::new();
        let new_node = arena.new_node(sources[n].clone());
        node_list.push(new_node); 
        roots.push(new_node);
        
        for order in 0..ism_order {
            // for i in N_IS_INDEX_RANGES[order].0+n*(offset+1) .. N_IS_INDEX_RANGES[order].1+n*(offset+1) {
            for i in N_IS_INDEX_RANGES[order].0 .. N_IS_INDEX_RANGES[order].1 {
                for boundary in Reflected::VALUES {
                    
                    // let current_node = arena.get(node_list[i]).unwrap();
                    let current_source = arena.get(node_list[i]).unwrap().get();
                    
                    if current_source.reflector != boundary  {
                        let new_node = arena.new_node(create_ism(current_source, room, &boundary));
                        node_list[i].append(new_node, &mut arena);
                        node_list.push(new_node);                        
                    }
                }
            }
        }
        source_tree_vec.push(arena);
        node_lists.push(node_list);
    }
    let source_tree = SourceTrees{arenas: source_tree_vec, node_lists, roots};
    source_tree
    //(arena, node_list)
}

pub fn update_source_tree(source_trees: &mut SourceTrees, sources: Vec<Source>, room: &Room) {
    // let offset = is_per_model(ism_order, 6);

    assert!(source_trees.node_lists.len() == sources.len());
    assert!(source_trees.node_lists[0].len() == source_trees.arenas[0].count());
    for (n, new_source) in sources.iter().enumerate() {
        for i in 0 .. source_trees.arenas[n].count() {
            let parent_node = source_trees.arenas[n].get(source_trees.node_lists[n][i]).unwrap().parent();
            match parent_node {
                Some(pn) => {
                    let parent_position: nalgebra::Matrix<f32, nalgebra::Const<3>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 3, 1>> = 
                        source_trees.arenas[n].get(pn).unwrap().get().position;
                    let current_node: &mut Source = source_trees.arenas[n].get_mut(source_trees.node_lists[n][i]).unwrap().get_mut();
                    *current_node.position = *calc_ism_position(&parent_position, room, &current_node.reflector);
                },
                None => {
                    let data = source_trees.arenas[n].get_mut(source_trees.node_lists[n][i]).unwrap().get_mut();
                    *data.position = *new_source.position; 
                },
            }
        }  
    }
}

pub fn update_source_tree_from_roots(source_trees: &mut SourceTrees, room: &Room) {

    for n in 0 .. source_trees.node_lists.len() {
        for i in 0 .. source_trees.arenas.len() {

            let parent_node = source_trees.arenas[n].get(source_trees.node_lists[n][i]).unwrap().parent();
            match parent_node {
                Some(pn) => {
                    let parent_position: nalgebra::Matrix<f32, nalgebra::Const<3>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 3, 1>> = 
                        source_trees.arenas[n].get(pn).unwrap().get().position;
                    let current_node: &mut Source = source_trees.arenas[n].get_mut(source_trees.node_lists[n][i]).unwrap().get_mut();
                    *current_node.position = *calc_ism_position(&parent_position, room, &current_node.reflector);
                },
                None => {

                },
            }
        }
    }
}
// calc ISMs
pub fn generate_image_source_vec(sources: Vec<Source>, room: &Room, ism_order: usize) -> Vec<Source> {
    
    let mut source_list: Vec<Source> = Vec::new();
    let mut current_ism_order: usize = 1;
    for n in 0..sources.len() {
        
        // Add source to list! 
        source_list.push(sources[n].clone());
        
        // Calc image sources
        for order in 0..ism_order {
            for i in N_IS_INDEX_RANGES[order].0 .. N_IS_INDEX_RANGES[order].1 {
                
                for boundary in Reflected::VALUES {
                    if source_list[i].reflector != boundary {
                        source_list.push(create_ism(&source_list[i], room, &boundary));
                    }
                }
            }
        }
    };    source_list
}


// This function creates an image source from reflecting a source
// on an respective room boundary
fn create_ism(s: &Source, r: &Room, b: &Reflected) -> Source {   
    
    let position = calc_ism_position(&s.position, r, b);
 
    Source::new(
        position, 
        Quaternion::zero(), 
        r, 
        343.0, 
        48000.0, 
        Some(b.clone()), None
    )
}
// This function creates an image source from reflecting a source
// on an respective room boundary
fn calc_ism_position(source_position: &Vector3<f32>, r: &Room, b: &Reflected) -> Vector3<f32> {   
    
    let mut position = *source_position;
    match b {
        Reflected::X0 => {position.x = -position.x},
        Reflected::X1 => {position.x = 2.0 * r.dimension.x - position.x},
        Reflected::Y0 => {position.y = -position.y},
        Reflected::Y1 => {position.y = 2.0 * r.dimension.y - position.y},
        Reflected::Z0 => {position.z = -position.z},
        Reflected::Z1 => {position.z = 2.0 * r.dimension.z - position.z},
        Reflected::No => {panic!("This should not happen!")},
    } 
    position
}

// helper functions
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

// good ol' pythagoras
fn calc_distance(v1: &Vector3<f32>, v2: &Vector3<f32>) -> f32 {
    ((v1.x-v2.x).powi(2) + 
     (v1.y-v2.y).powi(2) + 
     (v1.z-v2.z).powi(2)).sqrt()
}


// TEEEEEESSSSSTS 
#[cfg(test)]

#[test]
fn test_ism_creation() {
    let ism_order: usize = 1;
    let speed_of_sound: f32 = 343.0;
    let sample_rate: f32 = 48000.0;
    let room: Room = Room::new(4.0, 3.0, 5.0);
    let listener = Listener::default();
    let ssrc: Source = Source::new(Vector3::new(2.0, 1.0, 2.0), Quaternion::zero(), &room, speed_of_sound, sample_rate, None, Some(&listener));

    let src_list = generate_image_source_vec(vec![ssrc], &room, ism_order);

    for i in src_list.iter().enumerate() {
        println!("Nr.: {}, {:?}", i.0, i.1.position);
    }
}
#[test]
fn test_ism_tree_creation() {
    let ism_order: usize = 1;
    let speed_of_sound: f32 = 343.0;
    let sample_rate: f32 = 48000.0;
    let room: Room = Room::new(4.0, 3.0, 5.0);
    let listener = Listener::default();
    let ssrc: Source = Source::new(Vector3::new(2.0, 1.0, 2.0), Quaternion::zero(), &room, speed_of_sound, sample_rate, None, Some(&listener));
    let ssrc2: Source = Source::new(Vector3::new(1.0, 2.0, 3.0), Quaternion::zero(), &room, speed_of_sound, sample_rate, None, Some(&listener));

    let src_tree = create_source_tree(vec![ssrc, ssrc2], &room, ism_order);

    for i in src_tree.node_lists[0].iter().enumerate() {
        println!("Nr.: {}, {:?}", i.0, src_tree.arenas[0].get(*i.1).unwrap().get().position);
    }
    for i in src_tree.node_lists[1].iter().enumerate() {
        println!("Nr.: {}, {:?}", i.0, src_tree.arenas[1].get(*i.1).unwrap().get().position);
    }
}
#[test]
fn test_ism_tree_update() {
    let ism_order: usize = 4;
    let speed_of_sound: f32 = 343.0;
    let sample_rate: f32 = 48000.0;
    let room: Room = Room::new(4.0, 3.0, 5.0);
    let listener = Listener::default();
    let ssrc: Source = Source::new(Vector3::new(2.0, 1.0, 2.0), Quaternion::zero(), &room, speed_of_sound, sample_rate, None, Some(&listener));
    let ssrc2: Source = Source::new(Vector3::new(1.0, 2.0, 3.0), Quaternion::zero(), &room, speed_of_sound, sample_rate, None, Some(&listener));

    let mut src_tree = create_source_tree(vec![ssrc], &room, ism_order);
    for i in src_tree.node_lists[0].iter().enumerate() {
        println!("Nr.: {}, {:?}", i.0, src_tree.arenas[0].get(*i.1).unwrap().get().position);
    }
    update_source_tree(&mut src_tree, vec![ssrc2], &room);
    
    for i in src_tree.node_lists[0].iter().enumerate() {
        println!("Nr.: {}, {:?}", i.0, src_tree.arenas[0].get(*i.1).unwrap().get().position);
    }
}

#[test]
// insanity check!
fn test_vector3() {
    let v1 = Vector3::new(3.0, 4.0, 0.0);
    let v2 = Vector3::new(0.0, 0.0, 0.0);
    println!("{:?}", calc_distance(&v1, &v2));
}

#[test]
fn n_is_pm(){
    let n = is_per_model(1, 6); 
    println!("N of sources: {}", n);
}
#[test]
fn test_vec_and_table() {
    let mut v = Vec::new();
    print!("Length of Vec: {}", v.len());
    v.push(1); 
    v.push(1); 
    v.push(1); 
    v.push(1); 
    print!("Length of Vec: {}", v.len());
}

#[test]
fn test_array_as_buff() {
    let a = [0.0; 13000];
}