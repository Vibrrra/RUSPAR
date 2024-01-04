use nalgebra::{Point, Quaternion, Point3};

use crate::buffers::CircularDelayBuffer;

use indextree::Arena;


#[derive(Debug)]
struct Source {
   pub position: Point3<f32>,
   pub orientation: Quaternion<f32>,
   pub buffer: CircularDelayBuffer,
   pub dist: f32,
}

#[derive(Debug)]
struct Listener {
   pub position: Point3<f32>,
   pub orientation: Quaternion<f32>,
}

#[cfg(test)]
#[test]

fn test1() { 

    // Init things

    use nalgebra::OPoint;
    use num_traits::Zero;

    // use Listener;
    let speed_of_sound: f32 = 343.0;
    let sample_rate: f32 = 48000.0;
    let room: [f32; 3] = [5.0, 3.0, 6.0];
    let max_delay = (sample_rate  
                       * (room.iter().fold(0.0,|acc, x| acc+x.powi(2))).sqrt()  
                       / speed_of_sound).round() as usize + 200;    
    let mut lis = Listener {
        position: Point3::default(),
        orientation: Quaternion::zero(),
    };


    let mut s1 = Source {
        position: Point3::default(), 
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        dist: 10.0,
    };

    let mut s1_is_a = Source {
        position: Point3::default(), 
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        dist: 20.0,
    };

    let s1_is_ab = Source {
        position: Point3::default(), 
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        dist: 30.0,
    };

    // update position
    lis.position = Point3::new(1.0, 1.5, 2.0);
    s1.position = Point3::new(2.0, 1.5, 4.0);
    s1.dist = nalgebra::distance(&lis.position,&s1.position);
    s1.buffer.set_delay_time_samples(sample_rate * s1.dist / 343.0);
    s1_is_a.position = Point3::new(-2.0, 1.5, 2.0);
    s1_is_a.dist = nalgebra::distance(&lis.position, &s1_is_a.position);
    s1_is_a.buffer.set_delay_time_samples(sample_rate * (s1_is_a.dist-s1.dist) / 343.0);

    let out: Vec<f32> = vec![0.0; 48000];
    let mut input: Vec<f32> = out;
    input[0] = 1.0;
    let mut temp = Vec::new();
    for i in 0 .. input.len() {
    
        let mut sout = 0.0;
        let sample = input[i];
        sout = s1.buffer.read();
        s1.buffer.write(sample);
        let s1_out = s1_is_a.buffer.read();
        s1_is_a.buffer.write(sout);
        sout += s1_out;

        if sout > 0.0 {
            temp.push( (i as f32) );
        }
        // print!("{:?}|", sout);
        
    }
    println!("Pulses found at: {:?}", temp);
    let arena =  &mut Arena::new();
    let mut node_ids = Vec::new();

    let root = arena.new_node(s1);
    node_ids.push(arena.new_node(s1_is_a));
    node_ids.push(arena.new_node(s1_is_ab));
  
    root.append(node_ids[0], arena);
    node_ids[0].append(node_ids[1], arena);

    //root.children(arena).for_each(|x| {dbg!(arena.get(x));});

    // 
    
    
    
}
