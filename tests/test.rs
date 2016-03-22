extern crate rb;

use rb::{RB, SpscRb, RbInspector, RbProducer, RbConsumer};
use std::thread;

#[test]
fn test_write() {
    let size = 128;
    let rb = SpscRb::new(size);
    let producer = rb.producer();
    assert!(rb.is_empty());
    assert_eq!(rb.slots_free(), size);
    assert_eq!(rb.count(), 0);
    let data = (0..size).collect::<Vec<_>>();
    for i in 0..8 {
        let slice = &data[i * 16..(i + 1) * 16];
        producer.write(slice).unwrap();
        assert_eq!(rb.count(), (i + 1) * 16);
        assert_eq!(rb.slots_free(), size - (i + 1) * 16);
    }
    assert!(rb.is_full());
}

#[test]
fn test_read() {
    let size = 128;
    let rb = SpscRb::new(size);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    assert!(rb.is_empty());
    let in_data = (0..size).map(|i| i * 2).collect::<Vec<_>>();
    producer.write(&in_data).unwrap();
    assert!(rb.is_full());
    let mut out_data = vec![0; size];
    consumer.read(&mut out_data).unwrap();
    assert_eq!(out_data, in_data);
    assert!(rb.is_empty());
}

#[test]
fn test_wrap_around() {
    let size = 128;
    let rb = SpscRb::new(size);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let in_data = (0..size * 2).map(|i| i * 2).collect::<Vec<_>>();
    producer.write(&in_data[0..64]).unwrap();
    assert_eq!(rb.count(), 64);
    let mut out_data = vec![0; size*2];
    // TODO: try to read more
    consumer.read(&mut out_data[0..64]).unwrap();
    assert!(rb.is_empty());
    producer.write(&in_data[64..64 + size]).unwrap();
    assert_eq!(rb.count(), 128);
    assert!(rb.is_full());
    consumer.read(&mut out_data[64..64 + size]).unwrap();
    assert!(rb.is_empty());
    producer.write(&in_data[64 + size..]).unwrap();
    assert_eq!(rb.count(), 64);
    consumer.read(&mut out_data[64 + size..]).unwrap();
    assert_eq!(in_data, out_data);
}

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
