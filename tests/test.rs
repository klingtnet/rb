extern crate rb;

use rb::{RB, SpscRb, RbInspector, RbProducer, RbConsumer};

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
fn test_clear() {
    let size = 128;
    let rb = SpscRb::new(size);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    assert!(rb.is_empty());
    let in_data = (0..size).map(|i| i * 2).collect::<Vec<_>>();
    producer.write(&in_data).unwrap();
    assert!(rb.is_full());
    rb.clear();
    assert!(rb.is_empty());
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
