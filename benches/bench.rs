#![feature(test)]

extern crate rb;
extern crate test;
extern crate rand;

use rb::{SpscRb, RB, RbProducer, RbConsumer};
use std::thread;
use rand::{Rng, XorShiftRng};
use test::Bencher;

/// Measure the time that it takes to pass 1 min of. audio through the buffer.
/// Given a sample-rate of 48kHz and duration of 60sec we get 2.880,000 samples for one channel of audio.
/// The ring-buffer's internal size will be 1024 samples, which results in a latency of 21ms = 1000ms*(1024/48_000).
#[bench]
fn bench_passing_1min_audio_samples_blocking(b: &mut Bencher) {
    let mut sample_cnt: isize = 48_000 * 60;
    let rb = SpscRb::new(1024);
    let producer = rb.producer();
    let consumer = rb.consumer();
    let mut rng = XorShiftRng::new_unseeded();
    // generate some noise
    let data = (0..1024).map(|_| rng.gen_range(-1.0f32, 1.0f32)).collect::<Vec<f32>>();
    thread::spawn(move|| {
        loop {
            producer.write_blocking(&data).unwrap();
        }
    });
    let mut buf = [0f32; 1024];
    b.iter(|| {
        while sample_cnt > 0 {
            let cnt = consumer.read_blocking(&mut buf).unwrap();
            sample_cnt -= cnt as isize;
        }
    })
}
