extern crate rb;

use rb::{SPSC_RB, RB};

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
