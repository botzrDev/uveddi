//! Async file walking utilities for non-blocking directory traversal
//!
//! This module provides async file walking functionality to replace the
//! synchronous walkdir usage for better performance on large codebases.

use futures::stream::Stream;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::fs;
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

/// Async file walker that yields file paths
pub struct AsyncWalker {
    include_extensions: Vec<String>,
}

impl AsyncWalker {
    /// Create a new async walker with specific file extensions to include
    pub fn new(include_extensions: Vec<String>) -> Self {
        Self { include_extensions }
    }

    /// Create a walker for common source code files
    pub fn for_source_code() -> Self {
        Self::new(vec![
            "rs".to_string(),
            "py".to_string(),
            "js".to_string(),
            "jsx".to_string(),
            "ts".to_string(),
            "tsx".to_string(),
        ])
    }

    /// Walk directory asynchronously, yielding file paths that match the criteria
    pub fn walk(&self, root: &Path) -> impl Stream<Item = Result<PathBuf, std::io::Error>> + '_ {
        self.walk_recursive(root.to_path_buf())
    }

    fn walk_recursive(
        &self,
        path: PathBuf,
    ) -> Pin<Box<dyn Stream<Item = Result<PathBuf, std::io::Error>> + Send + '_>> {
        Box::pin(async_stream::stream! {
            let mut stack = vec![path];

            while let Some(current_path) = stack.pop() {
                let metadata = match fs::metadata(&current_path).await {
                    Ok(metadata) => metadata,
                    Err(e) => {
                        yield Err(e);
                        continue;
                    }
                };

                if metadata.is_file() {
                    if self.should_include_file(&current_path) {
                        yield Ok(current_path);
                    }
                } else if metadata.is_dir() {
                    let read_dir = match fs::read_dir(&current_path).await {
                        Ok(read_dir) => read_dir,
                        Err(e) => {
                            yield Err(e);
                            continue;
                        }
                    };

                    let mut entries = ReadDirStream::new(read_dir);
                    while let Some(entry_result) = entries.next().await {
                        match entry_result {
                            Ok(entry) => {
                                stack.push(entry.path());
                            }
                            Err(e) => {
                                yield Err(e);
                            }
                        }
                    }
                }
            }
        })
    }

    fn should_include_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            self.include_extensions.contains(&extension.to_string())
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_async_walker() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        let mut rust_file = File::create(temp_path.join("test.rs")).await.unwrap();
        rust_file.write_all(b"fn main() {}").await.unwrap();

        let mut python_file = File::create(temp_path.join("test.py")).await.unwrap();
        python_file.write_all(b"print('hello')").await.unwrap();

        let mut txt_file = File::create(temp_path.join("test.txt")).await.unwrap();
        txt_file.write_all(b"ignored").await.unwrap();

        // Test walker
        let walker = AsyncWalker::for_source_code();
        let files: Result<Vec<_>, _> = walker.walk(temp_path).collect().await;
        let files = files.unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.file_name().unwrap() == "test.rs"));
        assert!(files.iter().any(|p| p.file_name().unwrap() == "test.py"));
        assert!(!files.iter().any(|p| p.file_name().unwrap() == "test.txt"));
    }
}
