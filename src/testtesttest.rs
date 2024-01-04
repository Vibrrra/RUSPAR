use crate::buffers::CircularDelayBuffer;
use nalgebra::Point3;
use nalgebra::Quaternion;
use protobuf::reflect;

// Enum for ISM Algorithm
// "No" => True Sound Source
// "X0 - Z1" => Reflected on respective shoebox boundary
#[derive(Debug, Default)]
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

#[allow(dead_code)]
#[derive(Debug)]
struct Room {
    dimension: Point3<f32>,
}

#[allow(dead_code)]
impl Room {
    fn new(width: f32, height: f32, length: f32) -> Self {
        Self {
            dimension: Point3::new(width, height, length),
        }
    }
    fn diagonal(&self) -> f32 {
        (self.dimension.x.powi(2) + self.dimension.y.powi(2) + self.dimension.z.powi(2)).sqrt()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Source {
    pub position: Point3<f32>,
    pub orientation: Quaternion<f32>,
    pub buffer: CircularDelayBuffer,
    pub dist: f32,
    pub reflector: Reflected,
}

#[allow(dead_code)]
impl Source {
    pub fn new(
        position: Point3<f32>,
        orientation: Quaternion<f32>,
        room: &Room,
        speed_of_sound: f32,
        sample_rate: f32,
        reflector: Option<Reflected>,
        list: Option<Listener>,
    ) -> Self {
        let dist = if let Some(x) = list {
            nalgebra::distance(&x.position, &position)
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
        }
    }
    pub fn update_position(&mut self, position: Point3<f32>, listener: &Listener) {
        self.position = position;
        self.dist = nalgebra::distance(&self.position, &listener.position);
        self.buffer
            .set_delay_time_samples(48000.0 * self.dist / 343.0f32);
    }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
struct Listener {
    pub position: Point3<f32>,
    pub orientation: Quaternion<f32>,
}

#[cfg(test)]
#[test]
fn test_bufs() {
    // Init things

    use indextree::Arena;
    use nalgebra::Point3;
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
        position: Point3::default(),
        orientation: Quaternion::zero(),
    };

    let mut s1 = Source {
        position: Point3::default(),
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        reflector: Reflected::No,
        dist: 0.0,
    };

    let mut s1_is_a = Source {
        position: Point3::default(),
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        reflector: Reflected::No,
        dist: 0.0,
    };

    let s1_is_ab = Source {
        position: Point3::default(),
        orientation: Quaternion::default(),
        buffer: CircularDelayBuffer::new(max_delay),
        reflector: Reflected::No,
        dist: 0.0,
    };

    // update position
    lis.position = Point3::new(1.0, 1.5, 2.0);
    s1.position = Point3::new(2.0, 1.5, 4.0);
    s1.dist = nalgebra::distance(&lis.position, &s1.position);
    s1.buffer
        .set_delay_time_samples(sample_rate * s1.dist / 343.0);
    s1_is_a.position = Point3::new(-2.0, 1.5, 2.0);
    s1_is_a.dist = nalgebra::distance(&lis.position, &s1_is_a.position);
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
