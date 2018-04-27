extern crate rb;

use rb::*;
use std::thread;

fn main() {
    const SIZE: usize = 128;
    let rb = SpscRb::new(SIZE);
    let (prod, cons) = (rb.producer(), rb.consumer());

    // producer thread
    const PERIOD: usize = 16;
    thread::spawn(move || {
        let mut skip = 0;
        // a sawtooth wave
        let saw = || {
            ((PERIOD as isize / -2)..(PERIOD as isize / 2 + 1))
                .cycle()
                .map(|x| x as f32 / (PERIOD / 2) as f32)
        };
        loop {
            // write a slice of size length 32 (length can be anything smaller than SIZE)
            let cnt = prod.write_blocking(&saw().skip(skip).take(PERIOD).collect::<Vec<f32>>())
                .unwrap();
            skip += cnt % (PERIOD);
        }
    });

    // consume data written by the producer thread
    let mut data = Vec::with_capacity(SIZE);
    let mut buf = [0.0f32; SIZE / 4];
    while data.len() < SIZE {
        let cnt = cons.read_blocking(&mut buf).unwrap();
        data.extend_from_slice(&buf[..cnt]);
    }
    // prints one period of the sawtooth wave
    println!("{:?}", &data[..PERIOD]);
}
