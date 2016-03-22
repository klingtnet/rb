extern crate rb;

use rb::{RB, SpscRb, RbInspector, RbProducer, RbConsumer};
use std::thread;

#[test]
fn test_threads() {
    let size = 128;
    let rb = SpscRb::new(size);
    let producer = rb.producer();
    let consumer = rb.consumer();
    let in_data = (0..size).map(|i| i * 2).collect::<Vec<_>>();
    let in_data_copy = in_data.clone();
    let mut out_data = Vec::with_capacity(size);

    const WRITE_BUF_SIZE: usize = 32;
    thread::spawn(move || {
        for i in 0..(size / WRITE_BUF_SIZE) {
            let cnt = producer.write(&in_data_copy[i * WRITE_BUF_SIZE..(i + 1) * WRITE_BUF_SIZE])
                              .unwrap();
            assert_eq!(cnt, WRITE_BUF_SIZE);
        }
    });

    const READ_BUF_SIZE: usize = 8;
    for _ in 0..(size / READ_BUF_SIZE) {
        let mut buf = [0; READ_BUF_SIZE];
        // TODO: Remove the busy wait by providing blocking receive and write functions
        while rb.count() < READ_BUF_SIZE {}
        let cnt = consumer.read(&mut buf).unwrap();
        assert_eq!(cnt, READ_BUF_SIZE);
        out_data.extend(buf.iter().cloned());
    }
    assert_eq!(in_data, out_data);
    assert!(rb.is_empty());
}

#[test]
fn test_threads_blocking() {
    let size = 1024;
    let rb = SpscRb::new(size);
    let producer = rb.producer();
    let consumer = rb.consumer();
    let in_data = (0..size).map(|i| i * 2).collect::<Vec<_>>();
    let in_data_copy = in_data.clone();
    let mut out_data = Vec::with_capacity(size);

    const WRITE_BUF_SIZE: usize = 32;
    thread::spawn(move || {
        for i in 0..(size / WRITE_BUF_SIZE) {
            let cnt = producer.write_blocking(&in_data_copy[i * WRITE_BUF_SIZE..(i + 1) *
                                                                                WRITE_BUF_SIZE])
                              .unwrap();
            assert_eq!(cnt, WRITE_BUF_SIZE);
        }
    });

    const READ_BUF_SIZE: usize = 8;
    for _ in 0..(size / READ_BUF_SIZE) {
        let mut buf = [0; READ_BUF_SIZE];
        let cnt = consumer.read_blocking(&mut buf).unwrap();
        assert_eq!(cnt, READ_BUF_SIZE);
        out_data.extend(buf.iter().cloned());
    }
    assert_eq!(in_data, out_data);
    assert!(rb.is_empty());
}
