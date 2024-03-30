extern crate rwheel;

use rwheel::sync::mutex::Mutex;
use std::{sync::Arc, thread};

fn main() {
    let count = Arc::new(Mutex::new(0));

    let threads: Vec<_> = (0..2).map(|_| {
        let cnt_clone = Arc::clone(&count);
        thread::spawn(move || {
            let mut num = cnt_clone.lock();
            for _ in 0..1000 {
                *num += 1;
            }
        })
    }).collect();

    for t in threads {
        t.join().unwrap();
    }

    println!("The counter is {:?}", *count.lock());
}

