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

#[test]
fn test() {
}
