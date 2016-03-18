use std::cmp;

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

/// A Single-Producer-Single-Consumer RingBuffer
struct SPSC_RB<T> {
    v: Vec<T>,
    read_pos: usize,
    write_pos: usize,
    size: usize,
}
impl<T: Clone + Default> SPSC_RB<T> {
    fn new(size: usize) -> Self {
        SPSC_RB {
            v: vec![T::default(); size + 1],
            read_pos: 0,
            write_pos: 0,
            // the additional element is used to distinct between empty and full state
            size: size + 1,
        }
    }
}
unsafe impl<T> ::std::marker::Sync for SPSC_RB<T> {}

impl<T: Clone + Default> RB<T> for SPSC_RB<T> {
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
        self.v.iter_mut().map(|_| T::default()).count();
    }

    fn write(&mut self, data: &[T]) -> Result<usize> {
        if self.is_full() {
            return Ok(0);
        }
        let cnt = cmp::min(data.len(), self.slots_free());
        println!("write cnt: {}", cnt);
        for idx in 0..cnt {
            self.v[self.write_pos] = data[idx].clone();
            self.write_pos = (self.write_pos + 1) % self.v.len();
        }
        return Ok(cnt);
    }

    fn read(&mut self, data: &mut [T]) -> Result<usize> {
        if self.is_empty() {
            return Ok(0);
        }
        let cnt = cmp::min(data.len(), self.count());
        println!("read cnt: {}", cnt);
        for idx in 0..cnt {
            data[idx] = self.v[self.read_pos].clone();
            self.read_pos = (self.read_pos + 1) % self.v.len();
        }
        Ok(cnt)
    }
}

#[test]
fn test() {
}
