use std::cmp;
use std::fmt;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait RB<T: Clone+Default> {
    /// Resets the whole buffer to the default value of type `T`.
    fn clear(&self);
    fn producer(&self) -> Producer<T>;
    fn consumer(&self) -> Consumer<T>;
}

pub trait RbInspector {
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
}

pub trait RbProducer<T> {
    /// Stores the given slice of data into the ring buffer.
    /// Returns the number of written elements or an Error.
    fn write(&self, &[T]) -> Result<usize>;
    /// Works analog to `write` but blocks until there are as much
    /// free slots in the ring buffer as there are elements in the given slice.
    fn write_blocking(&self, &[T]) -> Result<usize>;
}

pub trait RbConsumer<T> {
    /// Fills the given slice with values or, if the buffer is empty, does not modify it.
    /// Returns the number of written values or an error.
    fn read(&self, &mut [T]) -> Result<usize>;
    /// Works analog to `read` but blocks until the it can read enough elements to fill
    /// the given buffer slice.
    fn read_blocking(&self, &mut [T]) -> Result<usize>;
}

#[derive(Debug)]
pub enum RbError {
    Full,
    Empty,
}
impl fmt::Display for RbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RbError::Full => write!(f, "No free slots in the buffer"),
            &RbError::Empty => write!(f, "Buffer is empty"),
        }
    }
}

pub type Result<T> = ::std::result::Result<T, RbError>;

struct Inspector {
    read_pos: Arc<AtomicUsize>,
    write_pos: Arc<AtomicUsize>,
    size: usize,
}

/// A *thread-safe* Single-Producer-Single-Consumer RingBuffer
///
/// - mutually exclusive access for producer and consumer
/// TODO: Remove SPSC because the buffer can be used as MPMC
pub struct SpscRb<T> {
    buf: Arc<Mutex<Vec<T>>>,
    inspector: Arc<Inspector>,
    slots_free: Arc<Condvar>,
    data_available: Arc<Condvar>,
}
impl<T: Clone + Default> SpscRb<T> {
    pub fn new(size: usize) -> Self {
        let (read_pos, write_pos) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
        SpscRb {
            buf: Arc::new(Mutex::new(vec![T::default(); size + 1])),
            slots_free: Arc::new(Condvar::new()),
            data_available: Arc::new(Condvar::new()),
            // the additional element is used to distinct between empty and full state
            inspector: Arc::new(Inspector {
                read_pos: read_pos.clone(),
                write_pos: write_pos.clone(),
                size: size + 1,
            }),
        }
    }
}
impl<T: Clone + Default> RB<T> for SpscRb<T> {
    fn clear(&self) {
        let mut buf = self.buf.lock().unwrap();
        buf.iter_mut().map(|_| T::default()).count();
        self.inspector.read_pos.store(0, Ordering::Relaxed);
        self.inspector.write_pos.store(0, Ordering::Relaxed);
    }

    fn producer(&self) -> Producer<T> {
        Producer {
            buf: self.buf.clone(),
            inspector: self.inspector.clone(),
            slots_free: self.slots_free.clone(),
            data_available: self.data_available.clone(),
        }
    }

    fn consumer(&self) -> Consumer<T> {
        Consumer {
            buf: self.buf.clone(),
            inspector: self.inspector.clone(),
            slots_free: self.slots_free.clone(),
            data_available: self.data_available.clone(),
        }
    }
}
impl<T: Clone + Default> RbInspector for SpscRb<T> {
    fn is_empty(&self) -> bool {
        self.inspector.is_empty()
    }
    fn is_full(&self) -> bool {
        self.inspector.is_full()
    }
    fn capacity(&self) -> usize {
        self.inspector.capacity()
    }
    fn slots_free(&self) -> usize {
        self.inspector.slots_free()
    }
    fn count(&self) -> usize {
        self.inspector.count()
    }
}

impl RbInspector for Inspector {
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
}

/// Producer view into the ring buffer.
/// Provides a write method.
pub struct Producer<T> {
    buf: Arc<Mutex<Vec<T>>>,
    inspector: Arc<Inspector>,
    slots_free: Arc<Condvar>,
    data_available: Arc<Condvar>,
}

/// Consumer view into the ring buffer.
/// Provides a read method.
pub struct Consumer<T> {
    buf: Arc<Mutex<Vec<T>>>,
    inspector: Arc<Inspector>,
    slots_free: Arc<Condvar>,
    data_available: Arc<Condvar>,
}

impl<T: Clone> RbProducer<T> for Producer<T> {
    fn write(&self, data: &[T]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }
        if self.inspector.is_full() {
            return Err(RbError::Full);
        }
        let cnt = cmp::min(data.len(), self.inspector.slots_free());
        let mut buf = self.buf.lock().unwrap();
        for idx in 0..cnt {
            let wr_pos = self.inspector.write_pos.load(Ordering::Relaxed);
            buf[wr_pos] = data[idx].clone();
            let new_wr_pos = (wr_pos + 1) % buf.len();
            self.inspector.write_pos.store(new_wr_pos, Ordering::Relaxed);
        }
        self.data_available.notify_one();
        return Ok(cnt);
    }

    fn write_blocking(&self, data: &[T]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }
        let guard = self.buf.lock().unwrap();
        let mut buf = if self.inspector.is_full() {
            self.slots_free.wait(guard).unwrap()
        } else {
            guard
        };
        let cnt = cmp::min(data.len(), self.inspector.slots_free());
        for idx in 0..cnt {
            let wr_pos = self.inspector.write_pos.load(Ordering::Relaxed);
            buf[wr_pos] = data[idx].clone();
            let new_wr_pos = (wr_pos + 1) % buf.len();
            self.inspector.write_pos.store(new_wr_pos, Ordering::Relaxed);
        }
        self.data_available.notify_one();
        return Ok(cnt);
    }
}

impl<T: Clone> RbConsumer<T> for Consumer<T> {
    fn read(&self, data: &mut [T]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }
        if self.inspector.is_empty() {
            return Err(RbError::Empty);
        }
        let cnt = cmp::min(data.len(), self.inspector.count());
        let buf = self.buf.lock().unwrap();
        for idx in 0..cnt {
            let re_pos = self.inspector.read_pos.load(Ordering::Relaxed);
            data[idx] = buf[re_pos].clone();
            let new_re_pos = (re_pos + 1) % buf.len();
            self.inspector.read_pos.store(new_re_pos, Ordering::Relaxed);
        }
        self.slots_free.notify_one();
        Ok(cnt)
    }

    fn read_blocking(&self, data: &mut [T]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }
        let guard = self.buf.lock().unwrap();
        let buf = if self.inspector.is_empty() {
            self.data_available.wait(guard).unwrap()
        } else {
            guard
        };
        let cnt = cmp::min(data.len(), self.inspector.count());
        for idx in 0..cnt {
            let re_pos = self.inspector.read_pos.load(Ordering::Relaxed);
            data[idx] = buf[re_pos].clone();
            let new_re_pos = (re_pos + 1) % buf.len();
            self.inspector.read_pos.store(new_re_pos, Ordering::Relaxed);
        }
        self.slots_free.notify_one();
        Ok(cnt)
    }
}
