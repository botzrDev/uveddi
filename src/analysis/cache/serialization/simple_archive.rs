//! A simple zero-copy serialization system inspired by rkyv
//! This demonstrates the core concepts without all the complexity

use std::mem;
use std::slice;

/// Trait for types that can be archived
pub trait Archive {
    /// The archived representation of this type
    type Archived;
    
    /// Write the archived representation to a buffer
    fn write_archive(&self, buffer: &mut Vec<u8>) -> usize;
    
    /// Get a reference to the archived data from a buffer
    /// SAFETY: Buffer must contain valid archived data
    unsafe fn from_archive(buffer: &[u8], offset: usize) -> &Self::Archived;
}

/// Simple archived string (stores length + data)
#[repr(C)]
/// Data structure for archivedstring.
pub struct ArchivedString {
    len: u32,
    // Data follows immediately after
}

impl ArchivedString {
    /// Performs as str operation.
    pub fn as_str(&self) -> &str {
        unsafe {
            let data_ptr = (self as *const Self).add(1) as *const u8;
            let slice = slice::from_raw_parts(data_ptr, self.len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl Archive for String {
    type Archived = ArchivedString;
    
    fn write_archive(&self, buffer: &mut Vec<u8>) -> usize {
        let offset = buffer.len();
        
        // Write length
        buffer.extend_from_slice(&(self.len() as u32).to_le_bytes());
        
        // Write string data
        buffer.extend_from_slice(self.as_bytes());
        
        // Pad to alignment if needed
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }
        
        offset
    }
    
    unsafe fn from_archive(buffer: &[u8], offset: usize) -> &Self::Archived {
        &*(buffer.as_ptr().add(offset) as *const ArchivedString)
    }
}

/// Archive for primitive types (they're their own archived form)
macro_rules! impl_archive_for_primitive {
    ($($t:ty),*) => {
        $(
            impl Archive for $t {
                type Archived = $t;
                
                fn write_archive(&self, buffer: &mut Vec<u8>) -> usize {
                    let offset = buffer.len();
                    buffer.extend_from_slice(&self.to_le_bytes());
                    offset
                }
                
                unsafe fn from_archive(buffer: &[u8], offset: usize) -> &Self::Archived {
                    &*(buffer.as_ptr().add(offset) as *const $t)
                }
            }
        )*
    };
}

impl_archive_for_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

/// Example: Custom struct archiving
#[derive(Debug)]
/// Data structure for person.
pub struct Person {
    pub name: String,
    pub age: u32,
    pub height: f32,
}

#[repr(C)]
/// Data structure for archivedperson.
pub struct ArchivedPerson {
    name_offset: u32,  // Offset to archived string
    age: u32,
    height: f32,
}

impl ArchivedPerson {
    /// Performs name operation.
    pub fn name<'a>(&self, buffer: &'a [u8]) -> &'a str {
        unsafe {
            let archived_str = String::from_archive(buffer, self.name_offset as usize);
            archived_str.as_str()
        }
    }
    
    /// Performs age operation.
    pub fn age(&self) -> u32 {
        self.age
    }
    
    /// Performs height operation.
    pub fn height(&self) -> f32 {
        self.height
    }
}

impl Archive for Person {
    type Archived = ArchivedPerson;
    
    fn write_archive(&self, buffer: &mut Vec<u8>) -> usize {
        // First, write the string data
        let name_offset = self.name.write_archive(buffer);
        
        // Then write the struct itself
        let struct_offset = buffer.len();
        
        buffer.extend_from_slice(&(name_offset as u32).to_le_bytes());
        buffer.extend_from_slice(&self.age.to_le_bytes());
        buffer.extend_from_slice(&self.height.to_le_bytes());
        
        struct_offset
    }
    
    unsafe fn from_archive(buffer: &[u8], offset: usize) -> &Self::Archived {
        &*(buffer.as_ptr().add(offset) as *const ArchivedPerson)
    }
}

/// A simple archive buffer that owns the data
pub struct ArchiveBuffer {
    data: Vec<u8>,
    root_offset: usize,
}

impl ArchiveBuffer {
    /// Creates a new instance.
    pub fn new<T: Archive>(value: &T) -> Self {
        let mut data = Vec::new();
        let root_offset = value.write_archive(&mut data);
        Self { data, root_offset }
    }
    
    /// Performs get operation.
    pub fn get<T: Archive>(&self) -> &T::Archived {
        unsafe { T::from_archive(&self.data, self.root_offset) }
    }
    
    /// Performs to bytes operation.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }
    
    /// Performs from bytes operation.
    pub fn from_bytes(data: Vec<u8>, root_offset: usize) -> Self {
        Self { data, root_offset }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_primitive_archive() {
        let value: u32 = 42;
        let archive = ArchiveBuffer::new(&value);
        let archived = archive.get::<u32>();
        assert_eq!(*archived, 42);
    }
    
    #[test]
    fn test_string_archive() {
        let value = String::from("Hello, World!");
        let archive = ArchiveBuffer::new(&value);
        let archived = archive.get::<String>();
        assert_eq!(archived.as_str(), "Hello, World!");
    }
    
    #[test]
    fn test_person_archive() {
        let person = Person {
            name: String::from("Alice"),
            age: 30,
            height: 5.6,
        };
        
        let archive = ArchiveBuffer::new(&person);
        
        // Serialize to bytes
        let bytes = archive.to_bytes();
        
        // Deserialize from bytes
        let restored = ArchiveBuffer::from_bytes(bytes, archive.root_offset);
        let archived_person = restored.get::<Person>();
        
        assert_eq!(archived_person.name(&restored.data), "Alice");
        assert_eq!(archived_person.age(), 30);
        assert_eq!(archived_person.height(), 5.6);
    }
}