use super::*;

#[test]
fn write_empty_buffer_returns_zero() {
    let rb = SpscRb::new(1);
    let (_, producer) = (rb.consumer(), rb.producer());
    let a: [u8; 0] = [];
    match producer.write(&a) {
        Ok(v) => assert_eq!(v, 0),
        e => panic!("Error occured on get: {:?}", e),
    }
}

#[test]
fn write_blocking_empty_buffer_returns_zero() {
    let rb = SpscRb::new(1);
    let (_, producer) = (rb.consumer(), rb.producer());
    let a: [u8; 0] = [];
    match producer.write_blocking(&a) {
        None => {}
        v => panic!("`write_blocking` didn't return None, but {:?}", v),
    }
}

#[test]
fn write_to_full_queue_returns_error() {
    let rb = SpscRb::new(1);
    let (_, producer) = (rb.consumer(), rb.producer());
    let a = [1];
    producer.write(&a).unwrap();
    match producer.write(&a) {
        Err(RbError::Full) => {}
        v => panic!("No error or incorrect error: {:?}", v),
    }
}

#[test]
fn get_to_empty_buffer_returns_zero() {
    let rb = SpscRb::new(1);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let a = [1];
    producer.write(&a).unwrap();
    let mut b: [u8; 0] = [];
    match consumer.get(&mut b) {
        Ok(v) => assert_eq!(v, 0),
        e => panic!("Error occured on get: {:?}", e),
    }
}

#[test]
fn get_from_empty_buffer_returns_error() {
    let rb = SpscRb::new(1);
    let (consumer, _) = (rb.consumer(), rb.producer());
    let mut b = [42];
    match consumer.get(&mut b) {
        Err(RbError::Empty) => {}
        v => panic!("No error or incorrect error: {:?}", v),
    }
    assert_eq!(b[0], 42);
}

#[test]
fn read_to_empty_buffer_returns_zero() {
    let rb = SpscRb::new(1);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let a = [1];
    producer.write(&a).unwrap();
    let mut b: [u8; 0] = [];
    match consumer.read(&mut b) {
        Ok(v) => assert_eq!(v, 0),
        e => panic!("Error occured on get: {:?}", e),
    }
}

#[test]
fn read_from_empty_buffer_returns_error() {
    let rb = SpscRb::new(1);
    let (consumer, _) = (rb.consumer(), rb.producer());
    let mut b = [42];
    match consumer.read(&mut b) {
        Err(RbError::Empty) => {}
        v => panic!("No error or incorrect error: {:?}", v),
    }
    assert_eq!(b[0], 42);
}

#[test]
fn read_blocking_to_empty_buffer_returns_none() {
    let rb = SpscRb::new(1);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let a = [1];
    producer.write(&a).unwrap();
    let mut b: [u8; 0] = [];
    match consumer.read_blocking(&mut b) {
        None => {}
        v => panic!("`read_blocking` unexpectedly returned {:?}", v),
    }
}

#[test]
fn get_with_wrapping() {
    let rb = SpscRb::new(1);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let a = [1];
    assert_eq!(producer.write(&a).unwrap(), 1);
    let mut b = [0];
    consumer.read(&mut b).unwrap();
    assert_eq!(b[0], 1);
    let c = [2, 3];
    assert_eq!(producer.write(&c).unwrap(), 1);
    let mut d = [0, 0];
    consumer.get(&mut d).unwrap();
    assert_eq!(d[0], 2);
    assert_eq!(d[1], 0);
}

#[test]
fn get_with_wrapping_2() {
    let rb = SpscRb::new(2);
    let (consumer, producer) = (rb.consumer(), rb.producer());
    let a = [1];
    assert_eq!(producer.write(&a).unwrap(), 1);
    let mut b = [0];
    consumer.read(&mut b).unwrap();
    assert_eq!(b[0], 1);
    let c = [2, 3];
    assert_eq!(producer.write(&c).unwrap(), 2);
    let mut d = [0, 0];
    consumer.get(&mut d).unwrap();
    assert_eq!(d[0], 2);
    assert_eq!(d[1], 3);
}
