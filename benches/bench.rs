#![feature(test)]

extern crate rand;
extern crate rb;
extern crate test;

use rand::{Rng, XorShiftRng};
use rb::{RbConsumer, RbProducer, SpscRb, RB};
use std::thread;
use test::Bencher;

#[bench]
/// Benchmark the time it takes to blocking read and write a 1k buffer of f32 elements.
fn bench_passing_a_1k_buffer_blocking(b: &mut Bencher) {
    const SIZE: usize = 1024;
    let rb = SpscRb::new(SIZE);
    let producer = rb.producer();
    let consumer = rb.consumer();
    let mut rng = XorShiftRng::new_unseeded();
    let data = (0..SIZE)
        .map(|_| rng.gen_range(-1.0f32, 1.0f32))
        .collect::<Vec<f32>>();
    thread::spawn(move || loop {
        producer.write_blocking(&data).unwrap();
    });
    let mut buf = [0f32; SIZE];
    b.iter(|| {
        let cnt = consumer.read_blocking(&mut buf).unwrap();
        assert_eq!(cnt, SIZE);
    });
}
