#[cfg(test)]
mod tests;

use std::cmp;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

/// Managment interface for the ring buffer.
pub trait RB<T: Clone + Copy + Default> {
    /// Resets the whole buffer to the default value of type `T`.
    /// The buffer is empty after this call.
    fn clear(&self);
    /// Creates a *producer* view inside the buffer.
    fn producer(&self) -> Producer<T>;
    /// Creates a *consumer* view inside the buffer.
    fn consumer(&self) -> Consumer<T>;
}

/// RbInspector provides non-modifying operations on the ring buffer.
pub trait RbInspector {
    /// Returns true if the buffer is empty.
    fn is_empty(&self) -> bool;
    /// Returns true if the buffer is full.
    fn is_full(&self) -> bool;
    /// Returns the total capacity of the ring buffer.
    /// This is the size with which the buffer was initialized.
    fn capacity(&self) -> usize;
    /// Returns the number of values that can be written until the buffer until it is full.
    fn slots_free(&self) -> usize;
    /// Returns the number of values from the buffer that are available to read.
    fn count(&self) -> usize;
}

/// Defines *write* methods for a producer view.
pub trait RbProducer<T> {
    /// Stores the given slice of data into the ring buffer.
    /// Returns the number of written elements or an error.
    ///
    /// Possible errors:
    ///
    /// - `RbError::Full`
    fn write(&self, &[T]) -> Result<usize>;
    /// Works analog to `write` but blocks until there are free slots in the ring buffer.
    /// The number of actual blocks written is returned in the `Option` value.
    ///
    /// Returns `None` if the given slice has zero length.
    fn write_blocking(&self, &[T]) -> Option<usize>;
}

/// Defines *read* methods for a consumer view.
pub trait RbConsumer<T> {
    /// Skips all pending values.
    /// Technically it sets the consumer's read pointer to the position
    /// of the producer's write pointer.
    ///
    /// Returns the number of skipped elements.
    ///
    /// Possible errors:
    ///
    /// - `RbError::Empty` no pending elements
    fn skip_pending(&self) -> Result<usize>;
    /// Skips `cnt` number of elements.
    ///
    /// Returns the number of skipped elements.
    ///
    /// Possible errors:
    ///
    /// - `RbError::Empty` no pending elements
    fn skip(&self, cnt: usize) -> Result<usize>;
    /// Fills the given slice with values or, if the buffer is empty, does not modify it.
    /// This method does not change the state of the buffer, this means that the read pointer
    /// isn't changed if you call `get`. Consecutive calls to this method are idempotent, i.e. they
    /// will fill the given slice with the same data.
    /// Using `get` can be beneficial to `read` when a successive call has failed and you want to
    /// try again with same data. You can use `skip` to move the read pointer i.e. mark the values
    /// as read after the call succeeded.
    ///
    /// Returns the number of written values or an error.
    ///
    /// Possible errors:
    ///
    /// - RbError::Empty
    fn get(&self, &mut [T]) -> Result<usize>;
    /// Fills the given slice with values or, if the buffer is empty, does not modify it.
    /// Returns the number of written values or an error.
    ///
    /// Possible errors:
    ///
    /// - RbError::Empty
    fn read(&self, &mut [T]) -> Result<usize>;
    /// Works analog to `read` but blocks until it can read elements to fill
    /// the given buffer slice.
    /// The number of blocks read is not necessarily equal to the length of the given buffer slice,
    /// the exact number is returned in the `Option` value.
    ///
    /// Returns `None` if the given slice has zero length.
    fn read_blocking(&self, &mut [T]) -> Option<usize>;
}

/// Ring buffer errors.
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

/// Result type used inside the module.
pub type Result<T> = ::std::result::Result<T, RbError>;

struct Inspector {
    read_pos: Arc<AtomicUsize>,
    write_pos: Arc<AtomicUsize>,
    size: usize,
}

/// A *thread-safe* Single-Producer-Single-Consumer RingBuffer
///
/// - blocking and non-blocking IO
/// - mutually exclusive access for producer and consumer
/// - no use of `unsafe`
/// - never under- or overflows
///
/// ```
/// use std::thread;
/// use rb::*;
///
/// let rb = SpscRb::new(1024);
/// let (prod, cons) = (rb.producer(), rb.consumer());
/// thread::spawn(move || {
///     let gen = || {(-16..16+1).cycle().map(|x| x as f32/16.0)};
///     loop {
///         let data = gen().take(32).collect::<Vec<f32>>();
///         prod.write(&data).unwrap();
///     }
/// });
/// let mut data = Vec::with_capacity(1024);
/// let mut buf = [0.0f32; 256];
/// while data.len() < 1024 {
///     let cnt = cons.read_blocking(&mut buf).unwrap();
///     data.extend_from_slice(&buf[..cnt]);
/// }
/// ```
pub struct SpscRb<T> {
    buf: Arc<Mutex<Vec<T>>>,
    inspector: Arc<Inspector>,
    slots_free: Arc<Condvar>,
    data_available: Arc<Condvar>,
}

impl<T: Clone + Copy + Default> SpscRb<T> {
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

impl<T: Clone + Copy + Default> RB<T> for SpscRb<T> {
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

impl<T: Clone + Copy + Default> RbInspector for SpscRb<T> {
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
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.slots_free() == self.capacity()
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        self.slots_free() == 0
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.size - 1
    }

    #[inline(always)]
    fn slots_free(&self) -> usize {
        let wr_pos = self.write_pos.load(Ordering::Relaxed);
        let re_pos = self.read_pos.load(Ordering::Relaxed);
        match wr_pos < re_pos {
            true => re_pos - wr_pos - 1,
            false => self.capacity() - wr_pos + re_pos,
        }
    }

    #[inline(always)]
    fn count(&self) -> usize {
        self.capacity() - self.slots_free()
    }
}

/// Producer view into the ring buffer.
pub struct Producer<T> {
    buf: Arc<Mutex<Vec<T>>>,
    inspector: Arc<Inspector>,
    slots_free: Arc<Condvar>,
    data_available: Arc<Condvar>,
}

/// Consumer view into the ring buffer.
pub struct Consumer<T> {
    buf: Arc<Mutex<Vec<T>>>,
    inspector: Arc<Inspector>,
    slots_free: Arc<Condvar>,
    data_available: Arc<Condvar>,
}

impl<T: Clone + Copy> RbProducer<T> for Producer<T> {
    fn write(&self, data: &[T]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }
        if self.inspector.is_full() {
            return Err(RbError::Full);
        }
        let cnt = cmp::min(data.len(), self.inspector.slots_free());
        let mut buf = self.buf.lock().unwrap();
        let buf_len = buf.len();
        let wr_pos = self.inspector.write_pos.load(Ordering::Relaxed);

        if (wr_pos + cnt) < buf_len {
            buf[wr_pos..wr_pos + cnt].copy_from_slice(&data[..cnt]);
        } else {
            let d = buf_len - wr_pos;
            buf[wr_pos..].copy_from_slice(&data[..d]);
            buf[..(cnt - d)].copy_from_slice(&data[d..cnt]);
        }
        self.inspector
            .write_pos
            .store((wr_pos + cnt) % buf_len, Ordering::Relaxed);

        self.data_available.notify_one();
        return Ok(cnt);
    }

    fn write_blocking(&self, data: &[T]) -> Option<usize> {
        if data.len() == 0 {
            return None;
        }
        let guard = self.buf.lock().unwrap();
        let mut buf = if self.inspector.is_full() {
            self.slots_free.wait(guard).unwrap()
        } else {
            guard
        };
        let buf_len = buf.len();
        let data_len = data.len();
        let wr_pos = self.inspector.write_pos.load(Ordering::Relaxed);
        let cnt = cmp::min(data_len, self.inspector.slots_free());

        if (wr_pos + cnt) < buf_len {
            buf[wr_pos..wr_pos + cnt].copy_from_slice(&data[..cnt]);
        } else {
            let d = buf_len - wr_pos;
            buf[wr_pos..].copy_from_slice(&data[..d]);
            buf[..(cnt - d)].copy_from_slice(&data[d..cnt]);
        }
        self.inspector
            .write_pos
            .store((wr_pos + cnt) % buf_len, Ordering::Relaxed);

        self.data_available.notify_one();
        return Some(cnt);
    }
}

impl<T: Clone + Copy> RbConsumer<T> for Consumer<T> {
    fn skip_pending(&self) -> Result<usize> {
        if self.inspector.is_empty() {
            Err(RbError::Empty)
        } else {
            // TODO check Order value
            let write_pos = self.inspector.write_pos.load(Ordering::Relaxed);
            let count = self.inspector.count();
            self.inspector.read_pos.store(write_pos, Ordering::Relaxed);
            Ok(count)
        }
    }

    fn skip(&self, cnt: usize) -> Result<usize> {
        if self.inspector.is_empty() {
            Err(RbError::Empty)
        } else {
            let count = cmp::min(cnt, self.inspector.count());
            let prev_read_pos = self.inspector.read_pos.load(Ordering::Relaxed);
            self.inspector.read_pos.store(
                (prev_read_pos + count) % self.inspector.capacity(),
                Ordering::Relaxed,
            );
            Ok(count)
        }
    }

    fn get(&self, data: &mut [T]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }
        if self.inspector.is_empty() {
            return Err(RbError::Empty);
        }
        let cnt = cmp::min(data.len(), self.inspector.count());
        let buf = self.buf.lock().unwrap();
        let buf_len = buf.len();
        let re_pos = self.inspector.read_pos.load(Ordering::Relaxed);

        if (re_pos + cnt) < buf_len {
            data[..cnt].copy_from_slice(&buf[re_pos..re_pos + cnt]);
        } else {
            let d = buf_len - re_pos;
            data[..d].copy_from_slice(&buf[re_pos..]);
            data[d..cnt].copy_from_slice(&buf[..(cnt - d)]);
        }

        Ok(cnt)
    }

    fn read(&self, data: &mut [T]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }
        if self.inspector.is_empty() {
            return Err(RbError::Empty);
        }
        let cnt = cmp::min(data.len(), self.inspector.count());
        let buf = self.buf.lock().unwrap();
        let buf_len = buf.len();
        let re_pos = self.inspector.read_pos.load(Ordering::Relaxed);

        if (re_pos + cnt) < buf_len {
            data[..cnt].copy_from_slice(&buf[re_pos..re_pos + cnt]);
        } else {
            let d = buf_len - re_pos;
            data[..d].copy_from_slice(&buf[re_pos..]);
            data[d..cnt].copy_from_slice(&buf[..(cnt - d)]);
        }

        // TODO: Notify all? empty->slots_free
        self.inspector
            .read_pos
            .store((re_pos + cnt) % buf_len, Ordering::Relaxed);
        self.slots_free.notify_one();
        Ok(cnt)
    }

    fn read_blocking(&self, data: &mut [T]) -> Option<usize> {
        if data.len() == 0 {
            return None;
        }
        let guard = self.buf.lock().unwrap();
        let buf = if self.inspector.is_empty() {
            self.data_available.wait(guard).unwrap()
        } else {
            guard
        };
        let buf_len = buf.len();
        let cnt = cmp::min(data.len(), self.inspector.count());
        let re_pos = self.inspector.read_pos.load(Ordering::Relaxed);

        if (re_pos + cnt) < buf_len {
            data[..cnt].copy_from_slice(&buf[re_pos..re_pos + cnt]);
        } else {
            let d = buf_len - re_pos;
            data[..d].copy_from_slice(&buf[re_pos..]);
            data[d..cnt].copy_from_slice(&buf[..(cnt - d)]);
        }

        self.inspector
            .read_pos
            .store((re_pos + cnt) % buf_len, Ordering::Relaxed);
        self.slots_free.notify_one();
        Some(cnt)
    }
}
