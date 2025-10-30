//! More advanced archiving with relative pointers and collections

use std::marker::PhantomData;
use std::ptr::NonNull;

/// A relative pointer that stores offset from itself to target
#[repr(transparent)]
/// Data structure for relptr.
pub struct RelPtr<T> {
    offset: i32,
    _phantom: PhantomData<T>,
}

impl<T> RelPtr<T> {
    /// Creates a new instance.
    pub fn new(from: usize, to: usize) -> Self {
        let offset = (to as isize - from as isize) as i32;
        Self {
            offset,
            _phantom: PhantomData,
        }
    }
    
    /// Performs get operation.
    pub fn get(&self) -> &T {
        unsafe {
            let self_addr = self as *const Self as *const u8;
            let target_addr = self_addr.offset(self.offset as isize);
            &*(target_addr as *const T)
        }
    }
    
    /// Returns the mut.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            let self_addr = self as *mut Self as *mut u8;
            let target_addr = self_addr.offset(self.offset as isize);
            &mut *(target_addr as *mut T)
        }
    }
}

/// Archived vector using relative pointer
#[repr(C)]
/// Data structure for archivedvec.
pub struct ArchivedVec<T> {
    len: u32,
    cap: u32,
    data: RelPtr<T>,
}

impl<T> ArchivedVec<T> {
    /// Performs len operation.
    pub fn len(&self) -> usize {
        self.len as usize
    }
    
    /// Checks whether empty is true.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Performs as slice operation.
    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(self.data.get(), self.len as usize)
            }
        }
    }
}

/// Writer that tracks position and alignment
pub struct ArchiveWriter {
    buffer: Vec<u8>,
    position: usize,
}

impl ArchiveWriter {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
        }
    }
    
    /// Performs align operation.
    pub fn align(&mut self, alignment: usize) {
        let remainder = self.position % alignment;
        if remainder != 0 {
            let padding = alignment - remainder;
            self.buffer.resize(self.buffer.len() + padding, 0);
            self.position += padding;
        }
    }
    
    /// Performs write operation.
    pub fn write<T>(&mut self, value: &T) -> usize
    where
        T: Copy,
    {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        
        self.align(align);
        let offset = self.position;
        
        unsafe {
            let bytes = std::slice::from_raw_parts(
                value as *const T as *const u8,
                size
            );
            self.buffer.extend_from_slice(bytes);
        }
        
        self.position += size;
        offset
    }
    
    /// Performs write slice operation.
    pub fn write_slice<T>(&mut self, slice: &[T]) -> usize
    where
        T: Copy,
    {
        if slice.is_empty() {
            return self.position;
        }
        
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        
        self.align(align);
        let offset = self.position;
        
        unsafe {
            let bytes = std::slice::from_raw_parts(
                slice.as_ptr() as *const u8,
                size * slice.len()
            );
            self.buffer.extend_from_slice(bytes);
        }
        
        self.position += size * slice.len();
        offset
    }
    
    /// Performs write vec operation.
    pub fn write_vec<T>(&mut self, vec: &[T]) -> usize
    where
        T: Copy,
    {
        // Write the data first
        let data_offset = if vec.is_empty() {
            0
        } else {
            self.write_slice(vec)
        };
        
        // Write the ArchivedVec header
        self.align(std::mem::align_of::<ArchivedVec<T>>());
        let header_offset = self.position;
        
        // Calculate relative pointer
        let rel_ptr_offset = header_offset + 
            std::mem::offset_of!(ArchivedVec<T>, data);
        let rel_offset = (data_offset as isize - rel_ptr_offset as isize) as i32;
        
        // Write header fields
        self.write(&(vec.len() as u32));
        self.write(&(vec.len() as u32)); // cap = len for archived
        self.write(&rel_offset);
        
        header_offset
    }
    
    /// Performs finish operation.
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }
}

/// Example: Archive a complex structure
#[derive(Debug, Clone)]
/// Data structure for document.
pub struct Document {
    pub title: String,
    pub pages: Vec<String>,
    pub word_count: u32,
}

#[repr(C)]
/// Data structure for archiveddocument.
pub struct ArchivedDocument {
    title: RelPtr<ArchivedString>,
    pages: RelPtr<ArchivedVec<RelPtr<ArchivedString>>>,
    word_count: u32,
}

#[repr(C)]
/// Data structure for archivedstring.
pub struct ArchivedString {
    len: u32,
    // UTF-8 bytes follow
}

impl ArchivedString {
    /// Performs as str operation.
    pub fn as_str(&self) -> &str {
        unsafe {
            let data_ptr = (self as *const Self).add(1) as *const u8;
            let slice = std::slice::from_raw_parts(data_ptr, self.len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl ArchivedDocument {
    /// Performs title operation.
    pub fn title(&self) -> &str {
        self.title.get().as_str()
    }
    
    /// Performs pages operation.
    pub fn pages(&self) -> Vec<&str> {
        self.pages
            .get()
            .as_slice()
            .iter()
            .map(|p| p.get().as_str())
            .collect()
    }
    
    /// Performs word count operation.
    pub fn word_count(&self) -> u32 {
        self.word_count
    }
}

impl Document {
    /// Performs archive operation.
    pub fn archive(&self) -> Vec<u8> {
        let mut writer = ArchiveWriter::new();
        
        // Write all strings first
        let title_offset = self.write_string(&mut writer, &self.title);
        
        let mut page_offsets = Vec::new();
        for page in &self.pages {
            let offset = self.write_string(&mut writer, page);
            page_offsets.push(offset);
        }
        
        // Write page offset array
        let pages_data_offset = writer.write_slice(&page_offsets);
        
        // Write pages ArchivedVec header
        writer.align(std::mem::align_of::<ArchivedVec<RelPtr<ArchivedString>>>());
        let pages_header_offset = writer.position;
        
        let rel_ptr_offset = pages_header_offset + 8; // After len and cap
        let rel_offset = (pages_data_offset as isize - rel_ptr_offset as isize) as i32;
        
        writer.write(&(page_offsets.len() as u32));
        writer.write(&(page_offsets.len() as u32));
        writer.write(&rel_offset);
        
        // Write document header
        writer.align(std::mem::align_of::<ArchivedDocument>());
        let doc_offset = writer.position;
        
        // Calculate relative pointers
        let title_rel = (title_offset as isize - doc_offset as isize) as i32;
        let pages_rel = (pages_header_offset as isize - 
                        (doc_offset + std::mem::size_of::<i32>()) as isize) as i32;
        
        writer.write(&title_rel);
        writer.write(&pages_rel);
        writer.write(&self.word_count);
        
        writer.finish()
    }
    
    fn write_string(&self, writer: &mut ArchiveWriter, s: &str) -> usize {
        writer.align(4);
        let offset = writer.position;
        
        writer.write(&(s.len() as u32));
        writer.buffer.extend_from_slice(s.as_bytes());
        writer.position += s.len();
        
        // Pad to alignment
        writer.align(4);
        
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complex_archive() {
        let doc = Document {
            title: String::from("My Document"),
            pages: vec![
                String::from("Page 1 content"),
                String::from("Page 2 content"),
                String::from("Page 3 content"),
            ],
            word_count: 42,
        };
        
        let archived_bytes = doc.archive();
        
        // Safety: In production, you'd validate the buffer
        unsafe {
            let archived = &*(archived_bytes.as_ptr().add(
                archived_bytes.len() - std::mem::size_of::<ArchivedDocument>()
            ) as *const ArchivedDocument);
            
            assert_eq!(archived.title(), "My Document");
            assert_eq!(archived.word_count(), 42);
            
            let pages = archived.pages();
            assert_eq!(pages.len(), 3);
            assert_eq!(pages[0], "Page 1 content");
            assert_eq!(pages[1], "Page 2 content");
            assert_eq!(pages[2], "Page 3 content");
        }
    }
}