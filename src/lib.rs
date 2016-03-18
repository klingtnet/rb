use std::cmp;
use std::sync::Mutex;

trait RB<T: Clone+Default> {
    /// Returns true if the buffer is empty.
    fn is_empty(&self) -> bool;
    /// Returns true if the buffer is full.
    fn is_full(&self) -> bool;
    /// Returns the total capacity of the ring buffer.
    fn capacity(&self) -> usize;
    /// Returns the number of values that can be written until the buffer is full.
    fn slots_free(&self) -> usize;
    /// Returns the number of values that are available in the buffer.
    fn count(&self) -> usize;
    /// Resets the whole buffer to the default value of type `T`.
    fn clear(&mut self);
    /// Stores the given slice of data into the ring buffer.
    /// TODO: The operation blocks until there are free slots if the buffer is full.
    /// Returns the number of written elements or an Error.
    fn write(&mut self, &[T]) -> Result<usize>;
    /// Fills the given slice with values or, if the buffer is empty, does not modify it.
    /// Returns the number of written values or an error.
    fn read(&mut self, &mut [T]) -> Result<usize>;
}

enum Err {
    Unknown,
}

type Result<T> = ::std::result::Result<T, Err>;

/// A *thread-safe* Single-Producer-Single-Consumer RingBuffer
///
/// - mutually exclusive access for producer and consumer
/// - TODO: synchronization between producer and consumer when the
///   is full or empty
struct SPSC_RB<T> {
    read_pos: usize,
    write_pos: usize,
    v: Mutex<Vec<T>>,
    size: usize,
}
impl<T: Clone + Default> SPSC_RB<T> {
    fn new(size: usize) -> Self {
        SPSC_RB {
            v: Mutex::new(vec![T::default(); size + 1]),
            read_pos: 0,
            write_pos: 0,
            // the additional element is used to distinct between empty and full state
            size: size + 1,
        }
    }
}
// TODO: check this implementation
unsafe impl<T> ::std::marker::Sync for SPSC_RB<T> {}

// TODO: use `#[inline]` and benchmark
impl<T: Clone + Default> RB<T> for SPSC_RB<T> {
    // TODO: always inline

    fn is_empty(&self) -> bool {
        self.slots_free() == self.capacity()
    }

    fn is_full(&self) -> bool {
        self.slots_free() == 0
    }

    fn capacity(&self) -> usize {
        self.size - 1
    }

    fn slots_free(&self) -> usize {
        match self.write_pos < self.read_pos {
            true => self.read_pos - self.write_pos - 1,
            false => self.capacity() - self.write_pos + self.read_pos,
        }
    }

    fn count(&self) -> usize {
        self.capacity() - self.slots_free()
    }

    fn clear(&mut self) {
        let mut buf = self.v.lock().unwrap();
        buf.iter_mut().map(|_| T::default()).count();
    }

    fn write(&mut self, data: &[T]) -> Result<usize> {
        if self.is_full() {
            // TODO: use a `::std::sync::Condvar` for blocking wait until something was read
            // TODO: Return an `Error::Full`
            return Ok(0);
        }
        let cnt = cmp::min(data.len(), self.slots_free());
        let mut buf = self.v.lock().unwrap();
        for idx in 0..cnt {
            buf[self.write_pos] = data[idx].clone();
            self.write_pos = (self.write_pos + 1) % buf.len();
        }
        return Ok(cnt);
    }

    fn read(&mut self, data: &mut [T]) -> Result<usize> {
        if self.is_empty() {
            return Ok(0);
        }
        let cnt = cmp::min(data.len(), self.count());
        let mut buf = self.v.lock().unwrap();
        for idx in 0..cnt {
            data[idx] = buf[self.read_pos].clone();
            self.read_pos = (self.read_pos + 1) % buf.len();
        }
        Ok(cnt)
    }
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
