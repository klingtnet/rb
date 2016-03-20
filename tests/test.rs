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
fn test_threads() {
    let size = 128;
    let mut rb = Arc::new(SPSC_RB::new(size));
    let rb_write = rb.clone();
    let in_data = (0..size).map(|i| i*2).collect::<Vec<_>>();
    let in_data_copy = in_data.clone();
    let mut out_data = Vec::with_capacity(size);

    const write_buf_size: usize = 32;
    thread::spawn(move || {
        for i in 0..(size/write_buf_size) {
            let cnt = rb_write.write(&in_data_copy[i*write_buf_size..(i+1)*write_buf_size]).unwrap();
            assert_eq!(cnt, write_buf_size);
        }
    });

    const read_buf_size: usize = 8;
    for _ in 0..(size/read_buf_size) {
        let mut buf = [0; read_buf_size];
        while rb.count() < read_buf_size {}
        let cnt = rb.read(&mut buf).unwrap();
        assert_eq!(cnt, read_buf_size);
        out_data.extend(buf.iter().cloned());
    }
    assert_eq!(in_data, out_data);
    assert!(rb.is_empty());
}
