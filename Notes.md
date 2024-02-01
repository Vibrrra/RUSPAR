## ToDos:

-----------
31.01.2024: (Chrissi)
* Split IMSDelayLine Structure in Server- und Audio-Part
* Lookup structures erstellen um variablen im Audio call back schneller erstellen zu können
* ...

------------

01.02.2024 (Idead for basic engine structure)

cargo.toml
```
[package]
name = "test_scoped_threads"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
once_cell = "1.19.0"
```
main.rs
``` 
use std::{ops::Deref, sync::{mpsc, Arc}, thread, time::Duration};  

use once_cell::sync::Lazy;

fn main() {

    thread::scope(|scope| {

        let mut source = Source{hrtf: None};
        let mut ims = ISM {sources: Vec::<Source>::new()};

        // buidling a static database
        static filter_database: Lazy<FilterDataBase> = Lazy::new(|| {
            FilterDataBase { 
                storage: vec![BinauralFilter { v: vec![0.0f32;2] }, BinauralFilter { v: vec![1.0f32;2] 
                }] }
        } );

        // create dummy data 
        let num_srcs: usize = 10;
        let num_ism_srcs: usize = 37;

        let mut delayline = DelayLine { buffer: vec![0.0f32; 128], hrtf: None }; 
        let mut update_container = UpdateContainer {num_srcs: num_srcs,inner: Vec::<Option<&BinauralFilter>>::new()};
        update_container.inner.push(None);
        update_container.inner.push(None);


        // start audio thread
        let _thread_handle_server = scope.spawn(move|| {
            let (tx,rx) = mpsc::sync_channel::<UpdateContainer>(1);
        
            let _thread_handle_audio = scope.spawn(move || {
                let mut update_container_audio_side = UpdateContainer {num_srcs: num_srcs, inner: Vec::<Option<&BinauralFilter>>::with_capacity(20)};

                loop {
                     match rx.try_recv() {
                        Ok(r) => {
                            update_container_audio_side.inner[0..r.num_srcs].copy_from_slice(&r.inner);// = r.inner[..];
                            let o  = &update_container_audio_side.inner[0];
                            delayline.hrtf  = Some(o.clone().unwrap());
                            println!("from:audio: {:?}",delayline.hrtf);
                            
                        },
                        Err(_) => {},
                    }; 
                    thread::sleep(Duration::from_millis(500))
                   
                }
          
            });
          
            loop {
                let mut hrtf = &filter_database.storage[0];

                // update_container.update_at(0, Arc::new(hrtf));
                update_container.update_at(0, hrtf);
                let _ = tx.send(update_container.clone());
                println!("from:server: {:?}",hrtf);
                thread::sleep(Duration::from_millis(1000));

                hrtf = &filter_database.storage[1];
                // update_container.update_at(0, Arc::new(hrtf));
                update_container.update_at(0, hrtf);
                let _ = tx.send(update_container.clone());
                print!("from:audio: {:?}",delayline.hrtf);
            } 
        });
    })
}

fn struct_builder() -> (DelayLine<'static>, FilterDataBase) {
    let delayline = DelayLine { buffer: vec![0.0f32; 256], hrtf: None };
    let filter_database = {
        let hrtf = BinauralFilter {v: vec![0.0f32; 128]};
        FilterDataBase {
        storage: vec![hrtf]}
    };
    (delayline, filter_database)
}

#[derive(Clone)]
pub struct UpdateContainer<'a> {
    // inner: Vec<Option<Arc<&'a BinauralFilter>>>
    num_srcs: usize,
    inner: Vec<Option<&'a BinauralFilter>>
}
impl<'a>  UpdateContainer <'a> {
    pub fn update_at(&mut self, pos: usize, hrtf: &'a BinauralFilter) {
        self.inner[pos] = Some(hrtf);
        // self.inner[pos] = Some(hrtf.clone());
    }

}

pub struct ISMUpdateMessage<'a> {
    hrtf_old: Arc<&'a BinauralFilter>,
    hrtf_new: Arc<&'a BinauralFilter>,
    delay_time: f32,
}

pub struct ISM<'a> {
    pub sources: Vec<Source<'a>>
}
pub struct Source<'a> {
    pub hrtf: Option<&'a BinauralFilter>
    
}

pub struct DelayLine<'a> {
    pub buffer: Vec<f32>,
    pub hrtf: Option<&'a BinauralFilter>,
}

#[derive(Debug, Clone)]
pub struct BinauralFilter {
    pub v: Vec<f32>
}

#[derive(Default)]
pub struct FilterDataBase {
    pub storage: Vec<BinauralFilter>
}

```