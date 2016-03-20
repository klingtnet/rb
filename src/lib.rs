use std::cmp;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait RB<T: Clone+Default> {
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
}

pub trait RB_Producer<T> {
    /// Stores the given slice of data into the ring buffer.
    /// TODO: The operation blocks until there are free slots if the buffer is full.
    /// Returns the number of written elements or an Error.
    fn write(&mut self, &[T]) -> Result<usize>;
}

pub trait RB_Consumer<T> {
    /// Fills the given slice with values or, if the buffer is empty, does not modify it.
    /// Returns the number of written values or an error.
    fn read(&mut self, &mut [T]) -> Result<usize>;
}

#[derive(Debug)]
pub enum Err {
    Unknown,
}

pub type Result<T> = ::std::result::Result<T, Err>;

/// A *thread-safe* Single-Producer-Single-Consumer RingBuffer
///
/// - mutually exclusive access for producer and consumer
/// - TODO: synchronization between producer and consumer when the
///   is full or empty
pub struct SPSC_RB<T> {
    buf: Arc<Mutex<Vec<T>>>,
    read_pos: AtomicUsize,
    write_pos: AtomicUsize,
    size: usize,
}
impl<T: Clone + Default> SPSC_RB<T> {
    pub fn new(size: usize) -> Self {
        SPSC_RB {
            buf: Arc::new(Mutex::new(vec![T::default(); size + 1])),
            read_pos: AtomicUsize::new(0),
            write_pos: AtomicUsize::new(0),
            // the additional element is used to distinct between empty and full state
            size: size + 1,
        }
    }

    pub fn producer(&self) -> Producer<T> {
       Producer {
            buf: self.buf.clone(),
       }
    }

    pub fn consumer(&self) -> Consumer<T> {
        Consumer {
            buf: self.buf.clone(),
        }
    }

    pub fn write(&mut self, data: &[T]) -> Result<usize> {
        if self.is_full() {
            // TODO: use a `::std::sync::Condvar` for blocking wait until something was read
            // TODO: Return an `Error::Full`
            return Ok(0);
        }
        let cnt = cmp::min(data.len(), self.slots_free());
        // TODO: try!(unlock)
        let mut buf = self.buf.lock().unwrap();
        for idx in 0..cnt {
            let wr_pos = self.write_pos.load(Ordering::Relaxed);
            buf[wr_pos] = data[idx].clone();
            let new_wr_pos = (wr_pos + 1) % buf.len();
            self.write_pos.store(new_wr_pos, Ordering::Relaxed);
        }
        return Ok(cnt);
    }

    pub fn read(&mut self, data: &mut [T]) -> Result<usize> {
        if self.is_empty() {
            return Ok(0);
        }
        let cnt = cmp::min(data.len(), self.count());
        let buf = self.buf.lock().unwrap();
        for idx in 0..cnt {
            let re_pos = self.read_pos.load(Ordering::Relaxed);
            data[idx] = buf[re_pos].clone();
            let new_re_pos = (re_pos + 1) % buf.len();
            self.read_pos.store(new_re_pos, Ordering::Relaxed);
        }
        Ok(cnt)
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
        let wr_pos = self.write_pos.load(Ordering::Relaxed);
        let re_pos = self.read_pos.load(Ordering::Relaxed);
        match wr_pos < re_pos {
            true => re_pos - wr_pos - 1,
            false => self.capacity() - wr_pos + re_pos,
        }
    }

    fn count(&self) -> usize {
        self.capacity() - self.slots_free()
    }

    fn clear(&mut self) {
        let mut buf = self.v.lock().unwrap();
        buf.iter_mut().map(|_| T::default()).count();
    }
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
        let buf = self.v.lock().unwrap();
        for idx in 0..cnt {
            data[idx] = buf[self.read_pos].clone();
            self.read_pos = (self.read_pos + 1) % buf.len();
        }
        Ok(cnt)
    }
}

pub struct Producer<T> {
    buf: Arc<Mutex<Vec<T>>>,
}

pub struct Consumer<T> {
    buf: Arc<Mutex<Vec<T>>>,
}
