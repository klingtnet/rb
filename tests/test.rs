extern crate rb;

use rb::{SPSC_RB, RB, RB_Inspector};
use std::thread;
use std::sync::Arc;

#[test]
fn test_write() {
    let size = 128;
    let mut rb: SPSC_RB<_> = SPSC_RB::new(size);
    assert!(rb.is_empty());
    assert_eq!(rb.slots_free(), size);
    assert_eq!(rb.count(), 0);
    let data = (0..size).collect::<Vec<_>>();
    for i in 0..8 {
        let slice = &data[i*16..(i+1)*16];
        rb.write(slice).unwrap();
        assert_eq!(rb.count(), (i+1)*16);
        assert_eq!(rb.slots_free(), size - (i+1)*16);
    }
    assert!(rb.is_full());
}

#[test]
fn test_read() {
    let size = 128;
    let mut rb: SPSC_RB<_> = SPSC_RB::new(size);
    assert!(rb.is_empty());
    let in_data = (0..size).map(|i| i*2).collect::<Vec<_>>();
    rb.write(&in_data).unwrap();
    assert!(rb.is_full());
    let mut out_data = vec![0; size];
    rb.read(&mut out_data).unwrap();
    assert_eq!(out_data, in_data);
    assert!(rb.is_empty());
}

#[test]
fn test_wrap_around() {
    let size = 128;
    let mut rb: SPSC_RB<_> = SPSC_RB::new(size);
    let in_data = (0..size*2).map(|i| i*2).collect::<Vec<_>>();
    rb.write(&in_data[0..64]).unwrap();
    assert_eq!(rb.count(), 64);
    let mut out_data = vec![0; size*2];
    // TODO: try to read more
    rb.read(&mut out_data[0..64]).unwrap();
    assert!(rb.is_empty());
    rb.write(&in_data[64..64+size]).unwrap();
    assert_eq!(rb.count(), 128);
    assert!(rb.is_full());
    rb.read(&mut out_data[64..64+size]).unwrap();
    assert!(rb.is_empty());
    rb.write(&in_data[64+size..]).unwrap();
    assert_eq!(rb.count(), 64);
    rb.read(&mut out_data[64+size..]).unwrap();
    assert_eq!(in_data, out_data);
}

#[test]
fn test() {
    let size = 32;
    let mut rb: SPSC_RB<f32> = SPSC_RB::new(size);
    assert!(rb.is_empty());
    assert_eq!(rb.slots_free(), size);
    assert_eq!(rb.count(), 0);
    let in_data = [1.0f32, 2.0, 3.0, 4.0, 5.0];
    assert!(rb.write(&in_data).is_ok());
    assert_eq!(rb.slots_free(), size - 5);
    assert_eq!(rb.count(), 5);
    let mut out_data = [0f32; 5];
    assert!(rb.read(&mut out_data).is_ok());
    assert!(rb.is_empty());
    assert_eq!(rb.count(), 0);
    assert_eq!(rb.slots_free(), size);
    assert_eq!(out_data, in_data);
    let in_data = [1.0f32; 32];
    assert!(rb.write(&in_data).is_ok());
    assert_eq!(rb.slots_free(), 0);
    assert_eq!(rb.count(), 32);
    let mut out_data = [0f32; 31];
    assert!(rb.read(&mut out_data).is_ok());
    assert_eq!(&in_data[..31], &out_data);
    assert_eq!(rb.count(), 1);
}
