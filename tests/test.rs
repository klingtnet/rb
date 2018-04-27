extern crate rb;

use rb::{RbConsumer, RbInspector, RbProducer, SpscRb, RB};

#[test]
fn test_write() {
    const SIZE: usize = 128;
    let rb = SpscRb::new(SIZE);
    let producer = rb.producer();
    assert!(rb.is_empty());
    assert_eq!(rb.slots_free(), SIZE);
    assert_eq!(rb.count(), 0);
    let data = (0..SIZE).collect::<Vec<_>>();
    for i in 0..8 {
        let slice = &data[i * 16..(i + 1) * 16];
        producer.write(slice).unwrap();
        assert_eq!(rb.count(), (i + 1) * 16);
        assert_eq!(rb.slots_free(), SIZE - (i + 1) * 16);
    }
    assert!(rb.is_full());
}

#[test]
fn test_read() {
    const SIZE: usize = 128;
    let rb = SpscRb::new(SIZE);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    assert!(rb.is_empty());
    let in_data = (0..SIZE).map(|i| i * 2).collect::<Vec<_>>();
    producer.write(&in_data).unwrap();
    assert!(rb.is_full());
    let mut out_data = vec![0; SIZE];
    consumer.read(&mut out_data).unwrap();
    assert_eq!(out_data, in_data);
    assert!(rb.is_empty());
}

#[test]
fn test_clear() {
    const SIZE: usize = 128;
    let rb = SpscRb::new(SIZE);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    assert!(rb.is_empty());
    let in_data = (0..SIZE).map(|i| i * 2).collect::<Vec<_>>();
    producer.write(&in_data).unwrap();
    assert!(rb.is_full());
    rb.clear();
    assert!(rb.is_empty());
    producer.write(&in_data).unwrap();
    assert!(rb.is_full());
    let mut out_data = vec![0; SIZE];
    consumer.read(&mut out_data).unwrap();
    assert_eq!(out_data, in_data);
    assert!(rb.is_empty());
}

#[test]
fn test_wrap_around() {
    const SIZE: usize = 128;
    let rb = SpscRb::new(SIZE);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let in_data = (0..SIZE * 2).map(|i| i * 2).collect::<Vec<_>>();
    producer.write(&in_data[0..64]).unwrap();
    assert_eq!(rb.count(), 64);
    let mut out_data = vec![0; SIZE * 2];
    // TODO: try to read more
    consumer.read(&mut out_data[0..64]).unwrap();
    assert!(rb.is_empty());
    producer.write(&in_data[64..64 + SIZE]).unwrap();
    assert_eq!(rb.count(), 128);
    assert!(rb.is_full());
    consumer.read(&mut out_data[64..64 + SIZE]).unwrap();
    assert!(rb.is_empty());
    producer.write(&in_data[64 + SIZE..]).unwrap();
    assert_eq!(rb.count(), 64);
    consumer.read(&mut out_data[64 + SIZE..]).unwrap();
    assert_eq!(in_data, out_data);
}

#[test]
fn test_skip() {
    const SIZE: usize = 128;
    let rb = SpscRb::new(SIZE);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let in_data = (0..SIZE / 2).collect::<Vec<_>>();
    let write_cnt = producer.write(&in_data).unwrap();
    assert_eq!(write_cnt, SIZE / 2);
    assert_eq!(rb.count(), SIZE / 2);
    let skipped = consumer.skip(10).unwrap();
    assert_eq!(skipped, 10);
    assert_eq!(rb.count(), (SIZE / 2) - 10);
    assert!(consumer.skip_pending().is_ok());
    assert!(rb.is_empty());
    assert!(consumer.skip(1).is_err());
    assert!(consumer.skip_pending().is_err());
}

#[test]
fn test_get() {
    const SIZE: usize = 128;
    let rb = SpscRb::new(SIZE);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let in_data = (0..SIZE).collect::<Vec<_>>();
    assert_eq!(producer.write(&in_data).unwrap(), SIZE);
    let mut out_data = vec![0; SIZE];
    assert_eq!(consumer.get(&mut out_data).unwrap(), SIZE);
    assert_eq!(out_data, in_data);
    assert!(rb.is_full());
    assert_eq!(consumer.get(&mut out_data).unwrap(), SIZE);
    assert_eq!(out_data, in_data);
    assert!(rb.is_full());
    assert_eq!(consumer.skip_pending().unwrap(), SIZE);
    assert!(rb.is_empty());
}

#[test]
fn test_read_write_wrap() {
    const SIZE: usize = 2;
    let rb = SpscRb::new(SIZE);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    assert!(rb.is_empty());
    let in_data = (0..SIZE).map(|i| i * 2).collect::<Vec<_>>();
    // Fill the buffer
    producer.write(&in_data).unwrap();
    assert!(rb.is_full());
    // Read half the data
    let mut out_data = vec![0; 1];
    consumer.read(&mut out_data).unwrap();
    assert_eq!(rb.count(), 1);
    assert_eq!(rb.slots_free(), 1);
    // Write more data so that it wraps around
    producer.write(&in_data).unwrap();
    assert_eq!(rb.count(), 2);
    assert_eq!(rb.slots_free(), 0);
    // Read data into a larger buffer, otherwise triggering a panic
    let mut out_data = vec![0; 3];
    consumer.read(&mut out_data).unwrap();
    assert_eq!(rb.count(), 0);
    assert_eq!(rb.slots_free(), 2);
}

#[test]
fn test_read_write_wrap_blocking() {
    const SIZE: usize = 2;
    let rb = SpscRb::new(SIZE);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    assert!(rb.is_empty());
    let in_data = (0..SIZE).map(|i| i * 2).collect::<Vec<_>>();
    // Fill the buffer
    producer.write_blocking(&in_data).unwrap();
    // Read half the data
    let mut out_data = vec![0; 1];
    consumer.read_blocking(&mut out_data).unwrap();
    assert_eq!(rb.count(), 1);
    assert_eq!(rb.slots_free(), 1);
    // Write more data so that it wraps around
    producer.write_blocking(&in_data).unwrap();
    assert_eq!(rb.count(), 2);
    assert_eq!(rb.slots_free(), 0);
    // Read data into a larger buffer, otherwise triggering a panic
    let mut out_data = vec![0; 3];
    consumer.read_blocking(&mut out_data).unwrap();
    assert_eq!(rb.count(), 0);
    assert_eq!(rb.slots_free(), 2);
}
