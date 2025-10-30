//! High-performance buffer types using const generics
//!
//! This module provides compile-time optimized buffer structures that can be used
//! throughout the analysis pipeline for improved performance and memory efficiency.

use std::error::Error;
use std::fmt;

/// A compile-time sized buffer for byte data
///
/// This buffer provides efficient storage for byte data with compile-time known size,
/// allowing for stack allocation and optimized memory access patterns.
///
/// # Examples
///
/// ```rust
/// use uveddi::analysis::buffer::FixedBuffer;
///
/// let mut buffer = FixedBuffer::<1024>::new();
/// buffer.push(b'H')?;
/// buffer.push(b'e')?;
/// buffer.push(b'l')?;
/// buffer.push(b'l')?;
/// buffer.push(b'o')?;
///
/// assert_eq!(buffer.as_slice(), b"Hello");
/// # Ok::<(), uveddi::analysis::buffer::BufferError>(())
/// ```
pub struct FixedBuffer<const SIZE: usize> {
    data: [u8; SIZE],
    len: usize,
}

/// Error type for buffer operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferError {
    /// Buffer is full and cannot accept more data
    BufferFull,
    /// Invalid operation on buffer
    InvalidOperation,
}

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BufferError::BufferFull => write!(f, "Buffer is full"),
            BufferError::InvalidOperation => write!(f, "Invalid buffer operation"),
        }
    }
}

impl Error for BufferError {}

impl<const SIZE: usize> FixedBuffer<SIZE> {
    /// Creates a new empty buffer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::buffer::FixedBuffer;
    ///
    /// let buffer = FixedBuffer::<256>::new();
    /// assert_eq!(buffer.len(), 0);
    /// assert_eq!(buffer.capacity(), 256);
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: [0; SIZE],
            len: 0,
        }
    }

    /// Pushes a single byte to the buffer
    ///
    /// # Errors
    ///
    /// Returns `BufferError::BufferFull` if the buffer is at capacity
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::buffer::FixedBuffer;
    ///
    /// let mut buffer = FixedBuffer::<4>::new();
    /// buffer.push(b'A')?;
    /// assert_eq!(buffer.len(), 1);
    /// # Ok::<(), uveddi::analysis::buffer::BufferError>(())
    /// ```
    #[inline]
    pub fn push(&mut self, byte: u8) -> Result<(), BufferError> {
        if self.len >= SIZE {
            return Err(BufferError::BufferFull);
        }
        self.data[self.len] = byte;
        self.len += 1;
        Ok(())
    }

    /// Extends the buffer with bytes from a slice
    ///
    /// # Errors
    ///
    /// Returns `BufferError::BufferFull` if the buffer doesn't have enough space
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::buffer::FixedBuffer;
    ///
    /// let mut buffer = FixedBuffer::<10>::new();
    /// buffer.extend_from_slice(b"Hello")?;
    /// assert_eq!(buffer.as_slice(), b"Hello");
    /// # Ok::<(), uveddi::analysis::buffer::BufferError>(())
    /// ```
    #[inline]
    pub fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), BufferError> {
        if self.len + bytes.len() > SIZE {
            return Err(BufferError::BufferFull);
        }
        self.data[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }

    /// Returns the current length of the buffer
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns the capacity of the buffer
    #[inline]
    pub const fn capacity(&self) -> usize {
        SIZE
    }

    /// Returns true if the buffer is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if the buffer is full
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == SIZE
    }

    /// Returns the available space in the buffer
    #[inline]
    pub const fn remaining(&self) -> usize {
        SIZE - self.len
    }

    /// Returns a slice of the buffer's contents
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Returns a mutable slice of the buffer's contents
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.len]
    }

    /// Clears the buffer
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Tries to convert the buffer contents to a UTF-8 string
    #[inline]
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_slice())
    }
}

impl<const SIZE: usize> Default for FixedBuffer<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> AsRef<[u8]> for FixedBuffer<SIZE> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const SIZE: usize> AsMut<[u8]> for FixedBuffer<SIZE> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

/// Type aliases for commonly used buffer sizes
pub type SmallBuffer = FixedBuffer<256>;
pub type MediumBuffer = FixedBuffer<1024>;
pub type LargeBuffer = FixedBuffer<8192>;
pub type HugeBuffer = FixedBuffer<65536>;

/// A compile-time sized string buffer
///
/// This provides efficient string storage with compile-time known capacity,
/// useful for fixed-size string operations in the analysis pipeline.
///
/// # Examples
///
/// ```rust
/// use uveddi::analysis::buffer::FixedString;
///
/// let mut string = FixedString::<64>::new();
/// string.push_str("Hello")?;
/// string.push_str(", ")?;
/// string.push_str("World")?;
/// assert_eq!(string.as_str(), "Hello, World");
/// # Ok::<(), uveddi::analysis::buffer::BufferError>(())
/// ```
pub struct FixedString<const SIZE: usize> {
    buffer: FixedBuffer<SIZE>,
}

impl<const SIZE: usize> FixedString<SIZE> {
    /// Creates a new empty string buffer
    #[inline]
    pub const fn new() -> Self {
        Self {
            buffer: FixedBuffer::new(),
        }
    }

    /// Pushes a string slice to the buffer
    ///
    /// # Errors
    ///
    /// Returns `BufferError::BufferFull` if the buffer doesn't have enough space
    /// Returns `BufferError::InvalidOperation` if the string is not valid UTF-8
    #[inline]
    pub fn push_str(&mut self, s: &str) -> Result<(), BufferError> {
        self.buffer.extend_from_slice(s.as_bytes())
    }

    /// Pushes a single character to the buffer
    ///
    /// # Errors
    ///
    /// Returns `BufferError::BufferFull` if the buffer doesn't have enough space
    #[inline]
    pub fn push_char(&mut self, ch: char) -> Result<(), BufferError> {
        let mut utf8_buf = [0u8; 4];
        let utf8_str = ch.encode_utf8(&mut utf8_buf);
        self.buffer.extend_from_slice(utf8_str.as_bytes())
    }

    /// Returns the string contents
    #[inline]
    pub fn as_str(&self) -> &str {
        // Safety: We only add valid UTF-8 through push_str and push_char
        unsafe { std::str::from_utf8_unchecked(self.buffer.as_slice()) }
    }

    /// Returns the current length in bytes
    #[inline]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the capacity in bytes
    #[inline]
    pub const fn capacity(&self) -> usize {
        SIZE
    }

    /// Returns true if the string is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns true if the string buffer is full
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.buffer.is_full()
    }

    /// Clears the string buffer
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl<const SIZE: usize> Default for FixedString<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> AsRef<str> for FixedString<SIZE> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const SIZE: usize> fmt::Display for FixedString<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const SIZE: usize> fmt::Debug for FixedString<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixedString<{}>({:?})", SIZE, self.as_str())
    }
}

/// Type aliases for commonly used string buffer sizes
pub type SmallString = FixedString<256>;
pub type MediumString = FixedString<1024>;
pub type LargeString = FixedString<8192>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_buffer_basic_operations() {
        let mut buffer = FixedBuffer::<4>::new();

        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 4);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());

        buffer.push(b'A').unwrap();
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.as_slice(), b"A");

        buffer.extend_from_slice(b"BC").unwrap();
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.as_slice(), b"ABC");

        buffer.push(b'D').unwrap();
        assert!(buffer.is_full());
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.as_slice(), b"ABCD");

        // Should fail when buffer is full
        assert_eq!(buffer.push(b'E'), Err(BufferError::BufferFull));
    }

    #[test]
    fn test_fixed_string_basic_operations() {
        let mut string = FixedString::<10>::new();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 10);
        assert!(string.is_empty());

        string.push_str("Hello").unwrap();
        assert_eq!(string.len(), 5);
        assert_eq!(string.as_str(), "Hello");

        string.push_char(' ').unwrap();
        string.push_str("Hi").unwrap();
        assert_eq!(string.as_str(), "Hello Hi");

        // Should fail when buffer is full
        assert_eq!(string.push_str("World"), Err(BufferError::BufferFull));
    }

    #[test]
    fn test_unicode_support() {
        let mut string = FixedString::<20>::new();

        string.push_str("Hello").unwrap();
        string.push_char('🌍').unwrap(); // 4-byte UTF-8 character
        string.push_str("!").unwrap();

        assert_eq!(string.as_str(), "Hello🌍!");
    }
}
