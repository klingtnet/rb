#![feature(test)]

extern crate rand_core;
extern crate rand_xorshift;
extern crate rb;
extern crate test;

use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
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
    let mut rng = XorShiftRng::from_seed([0u8; 16]);
    let data = (0..SIZE)
        .map(|_| rand_float(&mut rng))
        .collect::<Vec<f64>>();
    thread::spawn(move || loop {
        producer.write_blocking(&data).unwrap();
    });
    let mut buf = [0f64; SIZE];
    b.iter(|| {
        let cnt = consumer.read_blocking(&mut buf).unwrap();
        assert_eq!(cnt, SIZE);
    });
}

fn rand_float(rng: &mut XorShiftRng) -> f64 {
    let r = rng.next_u32();
    if r == 0 {
        return 0.0;
    }
    f64::from(u32::max_value() / (r - i32::max_value() as u32))
}
