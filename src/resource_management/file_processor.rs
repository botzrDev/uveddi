//! Streaming file processor for handling large files with memory constraints

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader};
use std::collections::VecDeque;
use super::{
    error::{ResourceError, ResourceResult},
    memory_tracker::MemoryTracker,
    resource_config::FileHandlingLimits,
};

/// Strategy for processing files based on size and memory constraints
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStrategy {
    /// Load entire file into memory
    InMemory,
    /// Stream file in chunks
    Streaming { chunk_size: usize },
    /// Skip file due to size or type restrictions
    Skip { reason: String },
}

/// Result of file processing
#[derive(Debug, Clone)]
pub struct ProcessedFile {
    pub path: PathBuf,
    pub size: u64,
    pub content_hash: Option<String>,
    pub processing_strategy: ProcessingStrategy,
    pub processing_time_ms: u64,
    pub chunks_processed: usize,
    pub lines_processed: usize,
    pub metadata: FileMetadata,
}

/// Metadata about processed file
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub file_type: String,
    pub encoding: String,
    pub line_count: usize,
    pub character_count: usize,
    pub binary_data_detected: bool,
    pub max_line_length: usize,
}

/// Streaming file processor that respects memory limits
pub struct StreamingFileProcessor {
    memory_tracker: Arc<MemoryTracker>,
    config: FileHandlingLimits,
    chunk_buffer: VecDeque<Vec<u8>>,
}

impl StreamingFileProcessor {
    /// Creates a new streaming file processor
    pub fn new(
        memory_tracker: Arc<MemoryTracker>,
        config: FileHandlingLimits,
    ) -> Self {
        Self {
            memory_tracker,
            config,
            chunk_buffer: VecDeque::with_capacity(4), // Pre-allocate for better performance
        }
    }
    
    /// Determines the best processing strategy for a file
    pub async fn determine_strategy(&self, file_path: &Path) -> ResourceResult<ProcessingStrategy> {
        // Check if file type is allowed
        if !self.is_file_type_allowed(file_path) {
            return Ok(ProcessingStrategy::Skip {
                reason: format!("File type not allowed: {}", 
                    file_path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("unknown")
                )
            });
        }
        
        // Get file size
        let metadata = tokio::fs::metadata(file_path).await
            .map_err(|e| ResourceError::FileProcessingError {
                file_path: file_path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;
        
        let file_size = metadata.len();
        
        // Check if file is too large for any processing
        let memory_stats = self.memory_tracker.get_usage_stats();
        let available_memory = memory_stats.available;
        
        if file_size > available_memory {
            return Ok(ProcessingStrategy::Skip {
                reason: format!(
                    "File size ({} bytes) exceeds available memory ({} bytes)",
                    file_size, available_memory
                )
            });
        }
        
        // Determine strategy based on file size and memory constraints
        if file_size <= self.config.max_file_size_memory {
            Ok(ProcessingStrategy::InMemory)
        } else if self.config.enable_streaming {
            Ok(ProcessingStrategy::Streaming {
                chunk_size: self.config.streaming_chunk_size,
            })
        } else {
            Ok(ProcessingStrategy::Skip {
                reason: "File too large and streaming disabled".to_string(),
            })
        }
    }
    
    /// Processes a file using the optimal strategy
    pub async fn process_file(&mut self, file_path: &Path) -> ResourceResult<ProcessedFile> {
        let start_time = std::time::Instant::now();
        let strategy = self.determine_strategy(file_path).await?;
        
        let result = match strategy.clone() {
            ProcessingStrategy::InMemory => {
                self.process_in_memory(file_path).await
            },
            ProcessingStrategy::Streaming { chunk_size } => {
                self.process_streaming(file_path, chunk_size).await
            },
            ProcessingStrategy::Skip { reason } => {
                return Ok(ProcessedFile {
                    path: file_path.to_path_buf(),
                    size: 0,
                    content_hash: None,
                    processing_strategy: strategy,
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    chunks_processed: 0,
                    lines_processed: 0,
                    metadata: FileMetadata::default(),
                });
            },
        }?;
        
        Ok(ProcessedFile {
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            processing_strategy: strategy,
            ..result
        })
    }
    
    /// Processes file by loading it entirely into memory
    async fn process_in_memory(&mut self, file_path: &Path) -> ResourceResult<ProcessedFile> {
        let file_size = tokio::fs::metadata(file_path).await?.len();
        
        // Allocate memory for the entire file
        let _memory_guard = self.memory_tracker
            .allocate("file_in_memory", file_size)?;
        
        let mut file = File::open(file_path).await
            .map_err(|e| ResourceError::FileProcessingError {
                file_path: file_path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;
        
        let mut content = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut content).await
            .map_err(|e| ResourceError::FileProcessingError {
                file_path: file_path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;
        
        let metadata = self.analyze_content(&content)?;
        let content_hash = self.calculate_hash(&content);
        
        Ok(ProcessedFile {
            path: file_path.to_path_buf(),
            size: file_size,
            content_hash: Some(content_hash),
            processing_strategy: ProcessingStrategy::InMemory,
            processing_time_ms: 0, // Will be filled by caller
            chunks_processed: 1,
            lines_processed: metadata.line_count,
            metadata,
        })
    }
    
    /// Processes file by streaming it in chunks
    async fn process_streaming(
        &mut self, 
        file_path: &Path, 
        chunk_size: usize
    ) -> ResourceResult<ProcessedFile> {
        let file_size = tokio::fs::metadata(file_path).await?.len();
        let mut file = File::open(file_path).await
            .map_err(|e| ResourceError::FileProcessingError {
                file_path: file_path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;
        
        let mut reader = BufReader::new(file);
        let mut chunks_processed = 0;
        let mut total_lines = 0;
        let mut total_characters = 0;
        let mut max_line_length = 0;
        let mut binary_detected = false;
        let mut hasher = sha2::Sha256::new();
        
        // Process file in chunks
        let mut buffer = vec![0u8; chunk_size];
        let mut leftover = Vec::new();
        
        loop {
            // Allocate memory for this chunk
            let _chunk_guard = self.memory_tracker
                .allocate("streaming_chunk", chunk_size as u64)?;
            
            let bytes_read = reader.read(&mut buffer).await
                .map_err(|e| ResourceError::FileProcessingError {
                    file_path: file_path.to_string_lossy().to_string(),
                    error: e.to_string(),
                })?;
            
            if bytes_read == 0 {
                break; // End of file
            }
            
            chunks_processed += 1;
            
            // Combine leftover from previous chunk with new data
            let mut chunk_data = leftover;
            chunk_data.extend_from_slice(&buffer[..bytes_read]);
            
            // Update hash with chunk data
            use sha2::Digest;
            hasher.update(&chunk_data);
            
            // Find complete lines in this chunk
            let (lines, remaining) = self.split_lines(&chunk_data);
            leftover = remaining;
            
            // Analyze lines in this chunk
            for line in lines {
                total_lines += 1;
                total_characters += line.len();
                max_line_length = max_line_length.max(line.len());
                
                // Check for binary data
                if !binary_detected && self.contains_binary_data(&line) {
                    binary_detected = true;
                }
            }
        }
        
        // Process any remaining data
        if !leftover.is_empty() {
            total_lines += 1;
            total_characters += leftover.len();
            max_line_length = max_line_length.max(leftover.len());
            
            if !binary_detected && self.contains_binary_data(&leftover) {
                binary_detected = true;
            }
        }
        
        let content_hash = format!("{:x}", hasher.finalize());
        
        let metadata = FileMetadata {
            file_type: file_path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
                .to_string(),
            encoding: "utf-8".to_string(), // Assumption for text files
            line_count: total_lines,
            character_count: total_characters,
            binary_data_detected: binary_detected,
            max_line_length,
        };
        
        Ok(ProcessedFile {
            path: file_path.to_path_buf(),
            size: file_size,
            content_hash: Some(content_hash),
            processing_strategy: ProcessingStrategy::Streaming { chunk_size },
            processing_time_ms: 0, // Will be filled by caller
            chunks_processed,
            lines_processed: total_lines,
            metadata,
        })
    }
    
    /// Splits data into lines, returning complete lines and remaining data
    fn split_lines(&self, data: &[u8]) -> (Vec<Vec<u8>>, Vec<u8>) {
        let mut lines = Vec::new();
        let mut start = 0;
        
        for (i, &byte) in data.iter().enumerate() {
            if byte == b'\n' {
                lines.push(data[start..=i].to_vec());
                start = i + 1;
            }
        }
        
        // Return remaining data that doesn't end with newline
        let remaining = if start < data.len() {
            data[start..].to_vec()
        } else {
            Vec::new()
        };
        
        (lines, remaining)
    }
    
    /// Checks if data contains binary (non-text) content
    fn contains_binary_data(&self, data: &[u8]) -> bool {
        for &byte in data {
            if byte < 32 && byte != b'\t' && byte != b'\n' && byte != b'\r' {
                return true;
            }
            if byte > 126 && byte < 160 {
                return true;
            }
        }
        false
    }
    
    /// Analyzes file content to extract metadata
    fn analyze_content(&self, content: &[u8]) -> ResourceResult<FileMetadata> {
        let mut line_count = 0;
        let mut max_line_length = 0;
        let mut current_line_length = 0;
        let binary_detected = self.contains_binary_data(content);
        
        for &byte in content {
            if byte == b'\n' {
                line_count += 1;
                max_line_length = max_line_length.max(current_line_length);
                current_line_length = 0;
            } else {
                current_line_length += 1;
            }
        }
        
        // Count last line if it doesn't end with newline
        if current_line_length > 0 {
            line_count += 1;
            max_line_length = max_line_length.max(current_line_length);
        }
        
        Ok(FileMetadata {
            file_type: "unknown".to_string(), // Will be filled by caller
            encoding: if binary_detected { "binary".to_string() } else { "utf-8".to_string() },
            line_count,
            character_count: content.len(),
            binary_data_detected: binary_detected,
            max_line_length,
        })
    }
    
    /// Calculates hash of content
    fn calculate_hash(&self, content: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }
    
    /// Checks if a file type is allowed based on configuration
    fn is_file_type_allowed(&self, file_path: &Path) -> bool {
        if self.config.allowed_file_types.is_empty() {
            return true; // All types allowed if list is empty
        }
        
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            self.config.allowed_file_types.iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(extension))
        } else {
            false // No extension, not allowed
        }
    }
}

impl Default for FileMetadata {
    fn default() -> Self {
        Self {
            file_type: "unknown".to_string(),
            encoding: "utf-8".to_string(),
            line_count: 0,
            character_count: 0,
            binary_data_detected: false,
            max_line_length: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::write;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_small_file_in_memory_processing() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello\nWorld\nTest file";
        write(&file_path, content).await.unwrap();
        
        let memory_tracker = Arc::new(MemoryTracker::new(1024 * 1024).unwrap());
        let config = FileHandlingLimits::default();
        let mut processor = StreamingFileProcessor::new(memory_tracker, config);
        
        let result = processor.process_file(&file_path).await.unwrap();
        
        assert_eq!(result.processing_strategy, ProcessingStrategy::InMemory);
        assert_eq!(result.metadata.line_count, 3);
        assert_eq!(result.chunks_processed, 1);
    }
    
    #[tokio::test]
    async fn test_file_type_filtering() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.xyz");
        write(&file_path, "content").await.unwrap();
        
        let memory_tracker = Arc::new(MemoryTracker::new(1024 * 1024).unwrap());
        let mut config = FileHandlingLimits::default();
        config.allowed_file_types = vec!["rs".to_string(), "py".to_string()];
        
        let processor = StreamingFileProcessor::new(memory_tracker, config);
        
        let strategy = processor.determine_strategy(&file_path).await.unwrap();
        assert!(matches!(strategy, ProcessingStrategy::Skip { .. }));
    }
    
    #[test]
    fn test_binary_data_detection() {
        let memory_tracker = Arc::new(MemoryTracker::new(1024).unwrap());
        let config = FileHandlingLimits::default();
        let processor = StreamingFileProcessor::new(memory_tracker, config);
        
        // Text data should not be detected as binary
        let text_data = b"Hello World\n";
        assert!(!processor.contains_binary_data(text_data));
        
        // Data with null bytes should be detected as binary
        let binary_data = b"Hello\x00World";
        assert!(processor.contains_binary_data(binary_data));
        
        // Data with control characters should be detected as binary
        let control_data = b"Hello\x01World";
        assert!(processor.contains_binary_data(control_data));
    }
}