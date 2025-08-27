use crate::security;

// Async file walking utilities for non-blocking directory traversal
//
// This module provides async file walking functionality to replace the
// synchronous walkdir usage for better performance on large codebases.
use futures::stream::Stream;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::fs;
use tokio_stream::{wrappers::ReadDirStream, StreamExt as TokioStreamExt};

/// Async file walker that yields file paths with security validation
pub struct AsyncWalker {
    include_extensions: Vec<String>,
    max_depth: Option<usize>,
    max_files: Option<usize>,
    validate_security: bool,
}

impl AsyncWalker {
    /// Creates a new `AsyncWalker` with the specified file extensions to include.
    ///
    /// # Arguments
    ///
    /// * `include_extensions` - Vector of file extensions (e.g., `vec!["rs", "py"]`) to include in the walk.
    ///
    /// # Returns
    ///
    /// * `AsyncWalker` - A new instance configured for the given extensions.
    pub fn new(include_extensions: Vec<String>) -> Self {
        Self {
            include_extensions,
            max_depth: None,
            max_files: None,
            validate_security: false,
        }
    }

    /// Creates a new `AsyncWalker` with security validation enabled
    ///
    /// # Arguments
    ///
    /// * `include_extensions` - Vector of file extensions to include
    /// * `max_depth` - Maximum directory depth (None for default limit)
    /// * `max_files` - Maximum number of files (None for default limit)
    ///
    /// # Returns
    ///
    /// * `AsyncWalker` - A new instance with security validation
    pub fn new_with_security(
        include_extensions: Vec<String>,
        max_depth: Option<usize>,
        max_files: Option<usize>,
    ) -> Self {
        Self {
            include_extensions,
            max_depth,
            max_files,
            validate_security: true,
        }
    }

    /// Creates an `AsyncWalker` preconfigured for common source code file extensions.
    ///
    /// # Returns
    ///
    /// * `AsyncWalker` - A new instance for Rust, Python, JS, TS, etc.
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

    /// Creates a secure `AsyncWalker` for source code with validation enabled
    ///
    /// # Returns
    ///
    /// * `AsyncWalker` - A new instance with security validation
    pub fn for_source_code_secure() -> Self {
        Self::new_with_security(
            vec![
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "jsx".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
            ],
            None, // Use default max depth
            None, // Use default max files
        )
    }

    /// Asynchronously walks a directory, yielding file paths matching the configured extensions.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory to start walking from.
    ///
    /// # Returns
    ///
    /// * `impl Stream<Item = Result<PathBuf, std::io::Error>>` - Stream of file paths or errors.
    ///
    /// # Example
    /// ```rust,ignore
    /// use uveddi::ingestion::AsyncWalker;
    /// use tokio_stream::StreamExt;
    /// let walker = AsyncWalker::for_source_code();
    /// let mut stream = walker.walk(Path::new("./src"));
    /// while let Some(Ok(path)) = stream.next().await {
    ///     println!("Found file: {}", path.display());
    /// }
    /// ```
    pub fn walk(&self, root: &Path) -> impl Stream<Item = Result<PathBuf, std::io::Error>> + '_ {
        self.walk_recursive(root.to_path_buf())
    }

    fn walk_recursive(
        &self,
        path: PathBuf,
    ) -> Pin<Box<dyn Stream<Item = Result<PathBuf, std::io::Error>> + Send + '_>> {
        Box::pin(async_stream::stream! {
            tracing::debug!("🚶 AsyncWalker::walk_recursive starting for: {}", path.display());
            let mut stack = vec![(path, 0usize)]; // (path, depth)
            let mut file_count = 0usize;

            while let Some((current_path, depth)) = stack.pop() {
                tracing::debug!("🔍 Processing path: {} (depth: {})", current_path.display(), depth);
                // Validate directory depth if security is enabled
                if self.validate_security {
                    if let Err(e) = security::validate_directory_depth(depth) {
                        yield Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()));
                        continue;
                    }
                }
                tracing::debug!("📊 Getting metadata for: {}", current_path.display());
                let metadata = match fs::metadata(&current_path).await {
                    Ok(metadata) => {
                        tracing::debug!("✅ Metadata retrieved for: {}", current_path.display());
                        metadata
                    },
                    Err(e) => {
                        tracing::error!("❌ Failed to get metadata for {}: {}", current_path.display(), e);
                        yield Err(e);
                        continue;
                    }
                };

                if metadata.is_file() {
                    tracing::debug!("📄 Found file: {}", current_path.display());
                    if self.should_include_file(&current_path) {
                        tracing::debug!("✅ File included: {}", current_path.display());
                        // Validate file count if security is enabled
                        if self.validate_security {
                            file_count += 1;
                            let max_files = self.max_files.unwrap_or(security::MAX_FILES_PER_ANALYSIS);
                            if let Err(e) = security::validate_file_count(file_count) {
                                yield Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()));
                                continue;
                            }
                            if file_count > max_files {
                                yield Err(std::io::Error::new(std::io::ErrorKind::InvalidInput,
                                    format!("File count {} exceeds maximum {}", file_count, max_files)));
                                continue;
                            }

                            // Validate file size and type
                            if let Err(e) = security::validate_file_size(&current_path) {
                                yield Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()));
                                continue;
                            }
                            if let Err(e) = security::validate_file_type(&current_path) {
                                yield Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()));
                                continue;
                            }
                        }
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
                    while let Some(entry_result) = TokioStreamExt::next(&mut entries).await {
                        match entry_result {
                            Ok(entry) => {
                                stack.push((entry.path(), depth + 1));
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
        let files: Result<Vec<_>, _> =
            tokio_stream::StreamExt::collect(walker.walk(temp_path)).await;
        let files = files.unwrap();

        assert_eq!(files.len(), 2);
        let names: Vec<String> = files
            .iter()
            .filter_map(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect();
        assert!(names.contains(&"test.rs".to_string()));
        assert!(names.contains(&"test.py".to_string()));
        assert!(!names.contains(&"test.txt".to_string()));
    }
}
