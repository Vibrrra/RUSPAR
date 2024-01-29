use std::thread::current;
use std::vec;

use indextree::NodeId;
use nalgebra::coordinates::XYZ;
use nalgebra::Vector3;
use nalgebra::Quaternion;
use num_traits::Zero;
use indextree::Arena;
use strum_macros::EnumIter;

use crate::convolver::Spatializer;
use crate::scene_parser::calc_lst_src_orientation;
use crate::scene_parser::calc_src_lst_orientation;

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
    pub const VALUES: [Self; 6] = [Self::X0, Self::X1, Self::Y0, Self::Y1, Self::Z0, Self::Z1];
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Room {
    pub dimension: Vector3<f32>,
}


#[allow(dead_code)]
impl Room {
    pub fn new(width: f32, height: f32, length: f32) -> Self {
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
#[derive(Clone, Default)]
pub struct Source {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub source_listener_orientation: SphericalCoordinates,//Quaternion<f32>,
    pub listener_source_orientation: SphericalCoordinates,
    // pub buffer: CircularDelayBuffer,
    pub dist: f32,
    pub dist_rem: f32,
    pub remaining_dist: f32,
    pub reflector: Reflected,
    pub spatializer: Option<Spatializer>,
    pub curr_hrtf_id: usize,
    pub prev_hrtf_id: usize,
}

#[allow(unused)]
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
        let source_listener_orientation = SphericalCoordinates::default();//Quaternion::zero();
        let listener_source_orientation = SphericalCoordinates::default();//Quaternion::zero();
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
            dist_rem: 0.0,
            remaining_dist: 0.0,
            reflector: refl,
            source_listener_orientation,
            listener_source_orientation,
            spatializer: None,
            prev_hrtf_id: 0,
            curr_hrtf_id: 0,
        }
        
    }
    pub fn update_position(&mut self, position: Vector3<f32>, listener: &Listener) {
        self.position = position;
        self.dist = calc_distance(&self.position, &listener.position);
        // self.buffer.set_delay_time_samples(48000.0 * self.dist / 343.0f32);
    }

}
impl SourceType<Source> for Source {
    fn get_dist(&self) -> f32 {
        self.dist
    }
    fn set_dist(&mut self, dist: f32) {
        self.dist = dist;
    }
    fn get_orientation(&self) -> Quaternion<f32> {
        self.orientation
    }
    fn get_reflector(&self) -> &Reflected {
        &self.reflector
    }
    fn get_pos(&self) -> Vector3<f32> {
        self.position
    }
    fn set_pos(&mut self, position: Vector3<f32>) {
        self.position = position;
    }
    fn get_lst_src_transform(&self) -> SphericalCoordinates {
        self.listener_source_orientation
    }
    fn get_src_lst_transform(&self) -> SphericalCoordinates {
        self.source_listener_orientation
    }
    fn set_lst_src_transform(&mut self, spc: SphericalCoordinates) {
        self.listener_source_orientation = spc;
    }
    fn set_src_lst_transform(&mut self, spc: SphericalCoordinates) {
        self.source_listener_orientation = spc;
    }
    fn create_ism(s: &Source, r: &Room, b: &Reflected, _may_have_spatializer: Option<Spatializer>) -> Source {   
    
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
    fn create_default(may_have_spatializer: Option<Spatializer>) -> Self {
        Source::default()
    }
    fn get_spatializer(&self) -> Option<Spatializer> {
        self.spatializer.clone()
    }
    fn set_spatializer(&mut self, spatializer: Spatializer) {
        self.spatializer = Some(spatializer); 
    }
    fn get_curr_hrtf_id(&self) -> usize {
        self.curr_hrtf_id
    }
    fn get_prev_hrtf_id(&self) -> usize {
        self.prev_hrtf_id
    }
    fn set_curr_hrtf_id(&mut self, curr_hrtf_id: usize) {
        self.curr_hrtf_id = curr_hrtf_id;
    }
    fn set_prev_hrtf_id(&mut self, prev_hrtf_id: usize) {
        self.prev_hrtf_id = prev_hrtf_id;
    }
    fn set_remaining_dist(&mut self, dist: f32) {
        self.remaining_dist = dist;
    }
    fn get_remaining_dist(& self) -> f32 {
        self.remaining_dist
    }
}


pub trait SourceType<T: Clone> {
    fn get_pos(&self) -> Vector3<f32>;
    fn set_pos(&mut self, position: Vector3<f32>);
    fn get_orientation(&self) -> Quaternion<f32>;
    fn get_reflector(&self) -> &Reflected;
    fn get_dist(&self) -> f32;
    fn set_dist(&mut self, dist: f32);
    fn get_src_lst_transform(&self) -> SphericalCoordinates;
    fn get_lst_src_transform(&self) -> SphericalCoordinates;
    fn set_src_lst_transform(&mut self, spc: SphericalCoordinates);
    fn set_lst_src_transform(&mut self, spc: SphericalCoordinates);
    fn create_ism(src: &T, room: &Room, b: &Reflected,may_have_spatializer: Option<Spatializer>) -> T; 
    fn create_default(may_have_spatializer: Option<Spatializer>) -> Self;
    fn get_spatializer(&self) -> Option<Spatializer>;
    fn set_spatializer(&mut self, Spatializer: Spatializer);
    fn set_curr_hrtf_id(&mut self, curr_id_hrtf: usize);
    fn set_prev_hrtf_id(&mut self, curr_id_hrtf: usize);
    fn get_curr_hrtf_id(&self, ) -> usize;
    fn get_prev_hrtf_id(&self, ) -> usize;
    fn set_remaining_dist(&mut self, dist: f32);
    fn get_remaining_dist(&self) -> f32;
}

#[derive(Clone)]
pub struct ISMLine<U> 
where U: SourceType<Source>
{
    pub source: U,
    pub spatializer_input_buffer: Vec<f32>,
}

impl<U> ISMLine<U>  
where U: SourceType<Source>
{
    
    pub fn new(source: U, block_size: usize) -> Self {
        Self {
            source, 
            spatializer_input_buffer: vec![0.0; block_size],
        }
    }

    pub fn from(source: Source) -> Self {
        
        todo!()
    }
}
// for <Source>
impl SourceType<ISMLine<Source>> for ISMLine<Source> {
    fn get_dist(&self) -> f32 {
        self.source.dist
    }
    fn set_dist(&mut self, dist: f32) {
        self.source.dist = dist;
    }
    fn get_orientation(&self) -> Quaternion<f32> {
        self.source.orientation
    }
    fn get_reflector(&self) -> &Reflected {
        &self.source.reflector
    }
    fn get_pos(&self) -> Vector3<f32> {
        self.source.position
    }
    fn set_pos(&mut self, position: Vector3<f32>) {
        self.source.position = position;
    }
    fn get_lst_src_transform(&self) -> SphericalCoordinates {
        self.source.listener_source_orientation
    }
    fn get_src_lst_transform(&self) -> SphericalCoordinates {
        self.source.source_listener_orientation
    }
    fn set_lst_src_transform(&mut self, spc: SphericalCoordinates) {
        self.source.listener_source_orientation = spc;
    }
    fn set_src_lst_transform(&mut self, spc: SphericalCoordinates) {
        self.source.source_listener_orientation = spc;
    }
    fn create_ism(s: &ISMLine<Source>, r: &Room, b: &Reflected, may_have_spatializer: Option<Spatializer>) -> ISMLine<Source> {   
 
        let position = calc_ism_position(&s.source.position, r, b);
     
        let src = Source::new(
            position, 
            Quaternion::zero(), 
            r, 
            343.0, 
            48000.0, 
            Some(b.clone()), None
        );
        ISMLine::new(src, s.spatializer_input_buffer.len())// s.source.spatializer.clone(), s.source.curr_hrtf_id, s.prev_hrtf_id)
    }
    fn create_default(may_have_spatializer: Option<Spatializer>) -> Self {

        ISMLine::new(Source::default(), 512)
    }
    fn get_spatializer(&self) -> Option<Spatializer> {
        self.source.spatializer.clone()
    }
    fn set_spatializer(&mut self, spatializer: Spatializer) {
        self.source.spatializer = Some(spatializer); 
    }
    fn get_curr_hrtf_id(&self) -> usize {
        self.source.curr_hrtf_id
    }
    fn get_prev_hrtf_id(&self) -> usize {
        self.source.prev_hrtf_id
    }
    fn set_curr_hrtf_id(&mut self, curr_hrtf_id: usize) {
        self.source.curr_hrtf_id = curr_hrtf_id;
    }
    fn set_prev_hrtf_id(&mut self, prev_hrtf_id: usize) {
        self.source.prev_hrtf_id = prev_hrtf_id;
    }
    fn set_remaining_dist(&mut self, dist: f32) {
        self.source.remaining_dist = dist;
    }
    fn get_remaining_dist(&self) -> f32 {
        self.source.remaining_dist
    }
}

// for <U>


#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Listener {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
}
impl Listener {
    pub fn get_pos(&self) -> Vector3<f32> {
        self.position
    }
}


#[derive(Debug, Clone)]
pub struct SourceTrees<T> {
    // pub arenas: Vec<Arena<Source>>,
    pub arenas: Vec<Arena<T>>,
    pub node_lists: Vec<Vec<NodeId>>,
    pub roots: Vec<NodeId>
}           
impl<T> SourceTrees<T> 
where T: SourceType<T> + Clone {
    pub fn create(number_of_sources: usize, ism_order: usize, may_have_spatializer: Option<Spatializer>) -> Self {
        let mut sources = Vec::new();
        for _ in 0 .. number_of_sources {
            sources.push(T::create_default(may_have_spatializer.clone()));
        }
        create_source_tree(sources, &Room::default(), ism_order, None)//, may_have_spatializer)
    }
    
}
// 
pub fn from_source_tree<U>(source_tree: SourceTrees<U>, block_size: usize) -> SourceTrees<ISMLine<U>> 
where U: SourceType<Source> + Clone
{
    
    let mut arenas: Vec<Arena<ISMLine<U>>> = Vec::new();
    let mut node_lists = Vec::new();
    let mut roots = Vec::new();

    for (src_arena, src_node_list) in source_tree.arenas.iter().zip(source_tree.node_lists.iter()) {
        
        let mut arena = indextree::Arena::new();
        let mut node_list = Vec::new();
        
        let mut src_node_list_iter = src_node_list.iter();
        let src = src_arena.get(*src_node_list_iter.next().unwrap()).unwrap().get();
        let ismline = ISMLine::new(src.clone(), block_size);
        let new_node = arena.new_node(ismline);
        roots.push(new_node);
        node_list.push(new_node);

        while let Some(src_node) = src_node_list_iter.next() {
            let src = src_arena.get(*src_node).unwrap().get();
            let ismline = ISMLine::new(src.clone(), block_size);
            let new_node = arena.new_node(ismline);
            node_list.push(new_node);
        }
        node_lists.push(node_list);
        arenas.push(arena);
    }
    SourceTrees { arenas, node_lists, roots}
}

pub fn create_source_tree<T>(sources: Vec<T>, room: &Room, ism_order: usize, may_have_spatializer: Option<Spatializer>) -> SourceTrees<T> 
where T: Clone + SourceType<T>
{
    // Init data structures
    let mut node_lists = Vec::new();
    let mut source_tree_vec = Vec::new(); 
    let mut roots = Vec::new(); 
    
    // Iterate over every sound source in the scene
    // - Create an arena for every node
    // - Create a list holding the ids of all roots ("non-image sources")
    for n in 0 .. sources.len() {
        let mut arena = indextree::Arena::new();
        let mut node_list = Vec::new();
        let new_node = arena.new_node(sources[n].clone());
        node_list.push(new_node); 
        roots.push(new_node);
        
        // Create image sources
        // - Iterate over all ism_order 
        for order in 0..ism_order {
            // - Iterate over the flattened mapped range of respective number of image sources
            for i in N_IS_INDEX_RANGES[order].0 .. N_IS_INDEX_RANGES[order].1 {
                for boundary in Reflected::VALUES {
                    
                    let current_source = arena.get(node_list[i]).unwrap().get();
                    
                    if *current_source.get_reflector() != boundary  {
                        let new_node = arena.new_node(T::create_ism(current_source, room, &boundary, current_source.get_spatializer().clone()));
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

pub fn update_source_tree(source_trees: &mut SourceTrees<Source>, sources: Vec<Source>, room: &Room, listener: &Listener) 
{
    assert!(source_trees.node_lists.len() == sources.len());
    assert!(source_trees.node_lists[0].len() == source_trees.arenas[0].count());

    // Iterate over all sound sources in the scene
    for (n, new_source) in sources.iter().enumerate() {
        
        // Iterate over every all (image) souces of the respective tree 
        for i in 0 .. source_trees.arenas[n].count() {

            // Parent node extraction
            let parent_node: Option<NodeId> = source_trees.arenas[n].get(source_trees.node_lists[n][i]).unwrap().parent();
            match parent_node {
                Some(pn) => {
                    let parent_dist = source_trees.arenas[n].get(pn).unwrap().get().get_dist();
                    let parent_position: nalgebra::Matrix<f32, nalgebra::Const<3>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 3, 1>> = 
                         source_trees.arenas[n].get(pn).unwrap().get().get_pos();
                    // let parent_position: nalgebra::Matrix<f32, nalgebra::Const<3>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 3, 1>> = 
                    //     parent_src.get_pos();
                    let current_node = source_trees.arenas[n].get_mut(source_trees.node_lists[n][i]).unwrap().get_mut();
                    *current_node.get_pos() = *calc_ism_position(&parent_position, room, &current_node.get_reflector());
                    current_node.set_dist(calc_distance(&current_node.get_pos(), &listener.get_pos()));
                    current_node.set_remaining_dist( current_node.get_dist()-parent_dist);
                },
                None => {
                    let data = source_trees.arenas[n].get_mut(source_trees.node_lists[n][i]).unwrap().get_mut();
                    // *data.position = *new_source.position; 
                    data.set_pos(new_source.get_pos()); 
                },
            }
        }  
    }
}

pub fn update_source_tree_from_roots(source_trees: &mut SourceTrees<Source>, room: &Room, listener: &Listener) 
{

    for n in 0 .. source_trees.node_lists.len() {
        for i in 0 .. source_trees.arenas.len() {

            let parent_node = source_trees.arenas[n].get(source_trees.node_lists[n][i]).unwrap().parent();
            match parent_node {
                Some(pn) => {
                    let parent_dist = source_trees.arenas[n].get(pn).unwrap().get().get_dist();
                    let parent_position: nalgebra::Matrix<f32, nalgebra::Const<3>, nalgebra::Const<1>, nalgebra::ArrayStorage<f32, 3, 1>> = 
                        source_trees.arenas[n].get(pn).unwrap().get().get_pos();
                    let current_node = source_trees.arenas[n].get_mut(source_trees.node_lists[n][i]).unwrap().get_mut();
                    // *current_node.position = *calc_ism_position(&parent_position, room, &current_node.reflector);
                    current_node.set_pos(calc_ism_position(&parent_position, room, &current_node.get_reflector()));
                    current_node.set_dist(calc_distance(&current_node.get_pos(), &listener.get_pos()));
                    current_node.set_remaining_dist( current_node.get_dist()-parent_dist);
                    let lst_src_orientation = calc_lst_src_orientation(&listener, current_node.get_pos());
                    let src_lst_orientation = calc_src_lst_orientation(current_node, current_node.get_pos());
                    current_node.set_lst_src_transform(lst_src_orientation);
                },
                None => {

                },
            }
        }
    }
}
// calc ISMs



// This function creates an image source from reflecting a source
// on an respective room boundary
// fn create_ism(s: &Source, r: &Room, b: &Reflected) -> Source {   
pub fn create_ism(s: &Source, r: &Room, b: &Reflected) -> Source {   
    
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
pub fn calc_ism_position(source_position: &Vector3<f32>, r: &Room, b: &Reflected) -> Vector3<f32> {   
    
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
#[allow(unused)]
pub fn is_per_model(maxorder: usize, n_surfaces: usize) -> usize {
    let mut n_ism: usize = 0;
    for i in 1..=maxorder {
        n_ism += is_per_order(i as f64, n_surfaces as f64) as usize;
    }
    n_ism
}

#[allow(unused)]
fn is_per_order(order: f64, n_surfaces: f64) -> usize {
    ((n_surfaces) * (n_surfaces - 1f64).powf(order - 1f64)).floor() as usize
}

// good ol' pythagoras
#[allow(unused)]
fn calc_distance(v1: &Vector3<f32>, v2: &Vector3<f32>) -> f32 {
    ((v1.x-v2.x).powi(2) + 
     (v1.y-v2.y).powi(2) + 
     (v1.z-v2.z).powi(2)).sqrt()
}

// fn calc_listener_drc_dist(v1: &Vector3<f32>, v2: &Vector3<f32>)

// TEEEEEESSSSSTS 
#[cfg(test)]


#[test]
fn test_ism_tree_creation() {
    let ism_order: usize = 3;
    let speed_of_sound: f32 = 343.0;
    let sample_rate: f32 = 48000.0;
    let room: Room = Room::new(4.0, 3.0, 5.0);
    let listener = Listener::default();
    let ssrc: Source = Source::new(Vector3::new(2.0, 1.0, 2.0), Quaternion::zero(), &room, speed_of_sound, sample_rate, None, Some(&listener));
    let ssrc2: Source = Source::new(Vector3::new(1.0, 2.0, 3.0), Quaternion::zero(), &room, speed_of_sound, sample_rate, None, Some(&listener));

    let src_tree = create_source_tree(vec![ssrc, ssrc2], &room, ism_order, None);

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

    let mut src_tree = create_source_tree(vec![ssrc], &room, ism_order, None);
    for i in src_tree.node_lists[0].iter().enumerate() {
        println!("Nr.: {}, {:?}", i.0, src_tree.arenas[0].get(*i.1).unwrap().get().position);
    }
    update_source_tree(&mut src_tree, vec![ssrc2], &room, &listener);
    
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
