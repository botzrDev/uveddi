//! File processing service for scanning and filtering analysis targets
//!
//! This module provides parallel file processing capabilities for large
//! codebases, with support for filtering, progress tracking, and efficient
//! resource utilization.

use crate::core::logging::{debug, info, warn};
use crate::error::UveddiError;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use super::progress_tracker::{ProgressTracker, ProgressUpdate};
use super::traits::{HealthCheck, Service, ServiceHealth};

/// File processing service for handling analysis target files
pub struct FileProcessor {
    /// Current processing configuration
    config: FileProcessorConfig,
    /// Progress tracker for file processing operations
    progress_tracker: Option<Arc<ProgressTracker>>,
    /// Service running state
    is_running: bool,
    /// Statistics about processed files
    stats: ProcessingStats,
}

/// Configuration for file processing operations
#[derive(Debug, Clone)]
pub struct FileProcessorConfig {
    /// Maximum number of files to process concurrently
    pub max_concurrent_files: usize,
    /// File extensions to include in processing
    pub include_extensions: Vec<String>,
    /// File patterns to exclude from processing
    pub exclude_patterns: Vec<String>,
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    /// Enable recursive directory scanning
    pub recursive: bool,
    /// Follow symbolic links
    pub follow_symlinks: bool,
    /// Buffer size for file reading operations
    pub read_buffer_size: usize,
}

/// Statistics about file processing operations
#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    /// Total number of files discovered
    pub files_discovered: AtomicUsize,
    /// Number of files processed successfully
    pub files_processed: AtomicUsize,
    /// Number of files skipped due to filters
    pub files_skipped: AtomicUsize,
    /// Number of files that failed processing
    pub files_failed: AtomicUsize,
    /// Total bytes processed
    pub bytes_processed: AtomicUsize,
}

/// Information about a discovered file
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Full path to the file
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// File extension (if any)
    pub extension: Option<String>,
    /// Whether the file should be processed
    pub should_process: bool,
    /// Reason for skipping (if not processing)
    pub skip_reason: Option<String>,
}

/// Result of file processing operation
#[derive(Debug)]
pub struct ProcessingResult {
    /// Files that were discovered
    pub discovered_files: Vec<FileInfo>,
    /// Files that were successfully processed
    pub processed_files: Vec<PathBuf>,
    /// Processing statistics
    pub stats: ProcessingStats,
    /// Any errors encountered during processing
    pub errors: Vec<UveddiError>,
}

impl FileProcessor {
    /// Create a new file processor with default configuration
    pub fn new() -> Self {
        Self {
            config: FileProcessorConfig::default(),
            progress_tracker: None,
            is_running: false,
            stats: ProcessingStats::default(),
        }
    }

    /// Create a new file processor with custom configuration
    pub fn with_config(config: FileProcessorConfig) -> Self {
        Self {
            config,
            progress_tracker: None,
            is_running: false,
            stats: ProcessingStats::default(),
        }
    }

    /// Set the progress tracker for this processor
    pub fn with_progress_tracker(mut self, tracker: Arc<ProgressTracker>) -> Self {
        self.progress_tracker = Some(tracker);
        self
    }

    /// Discover files in the given directory
    pub async fn discover_files(&mut self, root_path: &Path) -> Result<Vec<FileInfo>, UveddiError> {
        if !self.is_running {
            return Err(UveddiError::config_error(
                "File processor is not running",
                "service state",
            ));
        }

        debug!("Starting file discovery in: {}", root_path.display());

        let mut discovered_files = Vec::new();
        let mut errors = Vec::new();

        // Create progress update channel
        let (tx, mut rx) = mpsc::channel(100);

        // Spawn discovery task
        let root_path = root_path.to_path_buf();
        let config = self.config.clone();
        let discovery_handle =
            tokio::spawn(
                async move { Self::discover_files_recursive(&root_path, &config, tx).await },
            );

        // Handle progress updates
        let progress_tracker = self.progress_tracker.clone();
        let progress_handle = tokio::spawn(async move {
            while let Some(update) = rx.recv().await {
                if let Some(tracker) = &progress_tracker {
                    tracker.update(update).await;
                }
            }
        });

        // Wait for discovery to complete
        match discovery_handle.await {
            Ok(Ok(files)) => {
                discovered_files = files;
                self.stats
                    .files_discovered
                    .store(discovered_files.len(), Ordering::Relaxed);
            }
            Ok(Err(e)) => errors.push(e),
            Err(e) => errors.push(UveddiError::config_error(
                &format!("File discovery task failed: {}", e),
                "task execution",
            )),
        }

        // Wait for progress updates to complete
        progress_handle.await.ok();

        if !errors.is_empty() {
            warn!("File discovery completed with {} errors", errors.len());
            return Err(errors.into_iter().next().unwrap());
        }

        info!("Discovered {} files", discovered_files.len());
        Ok(discovered_files)
    }

    /// Process the discovered files in parallel
    pub async fn process_files(
        &mut self,
        files: Vec<FileInfo>,
    ) -> Result<ProcessingResult, UveddiError> {
        if !self.is_running {
            return Err(UveddiError::config_error(
                "File processor is not running",
                "service state",
            ));
        }

        debug!("Starting parallel processing of {} files", files.len());

        let processable_files: Vec<_> = files.iter().filter(|f| f.should_process).collect();

        let skipped_files: Vec<_> = files.iter().filter(|f| !f.should_process).collect();

        self.stats
            .files_skipped
            .store(skipped_files.len(), Ordering::Relaxed);

        info!(
            "Processing {} files ({} skipped)",
            processable_files.len(),
            skipped_files.len()
        );

        // Set up parallel processing
        let processed_files = Arc::new(std::sync::Mutex::new(Vec::new()));
        let errors = Arc::new(std::sync::Mutex::new(Vec::new()));
        let stats = Arc::new(self.stats.clone());

        // Process files in parallel using rayon
        processable_files.par_iter().for_each(|file_info| {
            match Self::process_single_file(&file_info.path, &self.config) {
                Ok(bytes_read) => {
                    if let Ok(mut processed) = processed_files.lock() {
                        processed.push(file_info.path.clone());
                    }
                    stats.files_processed.fetch_add(1, Ordering::Relaxed);
                    stats
                        .bytes_processed
                        .fetch_add(bytes_read, Ordering::Relaxed);
                }
                Err(e) => {
                    if let Ok(mut errors_vec) = errors.lock() {
                        errors_vec.push(e);
                    }
                    stats.files_failed.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        let processed_files = Arc::try_unwrap(processed_files)
            .map_err(|_| {
                UveddiError::config_error("Failed to unwrap processed files", "synchronization")
            })?
            .into_inner()
            .map_err(|_| {
                UveddiError::config_error("Failed to lock processed files", "synchronization")
            })?;

        let errors = Arc::try_unwrap(errors)
            .map_err(|_| UveddiError::config_error("Failed to unwrap errors", "synchronization"))?
            .into_inner()
            .map_err(|_| UveddiError::config_error("Failed to lock errors", "synchronization"))?;

        let stats = Arc::try_unwrap(stats)
            .map_err(|_| UveddiError::config_error("Failed to unwrap stats", "synchronization"))?;

        info!(
            "File processing completed: {} processed, {} failed",
            processed_files.len(),
            errors.len()
        );

        Ok(ProcessingResult {
            discovered_files: files,
            processed_files,
            stats,
            errors,
        })
    }

    /// Get current processing statistics
    pub fn get_stats(&self) -> ProcessingStats {
        self.stats.clone()
    }

    /// Reset processing statistics
    pub fn reset_stats(&mut self) {
        self.stats = ProcessingStats::default();
    }

    /// File discovery implementation (iterative to avoid async recursion)
    async fn discover_files_recursive(
        path: &Path,
        config: &FileProcessorConfig,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<Vec<FileInfo>, UveddiError> {
        let mut files = Vec::new();

        if path.is_file() {
            let file_info = Self::create_file_info(path, config)?;
            files.push(file_info);
            return Ok(files);
        }

        if !path.is_dir() {
            return Err(UveddiError::PathError {
                path: path.display().to_string(),
                reason: "Path is neither a file nor a directory".to_string(),
                suggestion: "Verify the path is valid".to_string(),
            });
        }

        // Use a stack to avoid recursive async calls
        let mut stack: Vec<PathBuf> = vec![path.to_path_buf()];

        while let Some(dir_path) = stack.pop() {
            let read_dir = match std::fs::read_dir(&dir_path) {
                Ok(rd) => rd,
                Err(e) => {
                    return Err(UveddiError::PathError {
                        path: dir_path.display().to_string(),
                        reason: format!("Failed to read directory: {}", e),
                        suggestion: "Check directory permissions".to_string(),
                    });
                }
            };

            for entry in read_dir {
                let entry = entry.map_err(|e| UveddiError::PathError {
                    path: dir_path.display().to_string(),
                    reason: format!("Failed to read directory entry: {}", e),
                    suggestion: "Check file system integrity".to_string(),
                })?;

                let entry_path = entry.path();

                // Skip symbolic links if not following them
                if entry_path.is_symlink() && !config.follow_symlinks {
                    continue;
                }

                if entry_path.is_dir() {
                    if config.recursive {
                        stack.push(entry_path);
                    }
                } else if entry_path.is_file() {
                    let file_info = Self::create_file_info(&entry_path, config)?;
                    files.push(file_info);

                    // Send progress update
                    let _ = progress_tx
                        .send(ProgressUpdate {
                            current: files.len(),
                            total: None,
                            message: format!("Discovered: {}", entry_path.display()),
                            stage: "file_discovery".to_string(),
                        })
                        .await;
                }
            }
        }

        Ok(files)
    }

    /// Create file info with filtering logic
    fn create_file_info(
        path: &Path,
        config: &FileProcessorConfig,
    ) -> Result<FileInfo, UveddiError> {
        let metadata = std::fs::metadata(path).map_err(|e| UveddiError::PathError {
            path: path.display().to_string(),
            reason: format!("Failed to read file metadata: {}", e),
            suggestion: "Check file permissions".to_string(),
        })?;

        let size = metadata.len();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase());

        let (should_process, skip_reason) =
            Self::should_process_file(path, size, &extension, config);

        Ok(FileInfo {
            path: path.to_path_buf(),
            size,
            extension,
            should_process,
            skip_reason,
        })
    }

    /// Determine if a file should be processed based on configuration
    fn should_process_file(
        path: &Path,
        size: u64,
        extension: &Option<String>,
        config: &FileProcessorConfig,
    ) -> (bool, Option<String>) {
        // Check file size
        if size > config.max_file_size as u64 {
            return (false, Some(format!("File too large: {} bytes", size)));
        }

        // Check extension inclusion
        if !config.include_extensions.is_empty() {
            if let Some(ext) = extension {
                if !config.include_extensions.contains(ext) {
                    return (
                        false,
                        Some(format!("Extension '{}' not in include list", ext)),
                    );
                }
            } else {
                return (
                    false,
                    Some("No extension and include list specified".to_string()),
                );
            }
        }

        // Check exclusion patterns
        let path_str = path.to_string_lossy();
        for pattern in &config.exclude_patterns {
            if path_str.contains(pattern) {
                return (false, Some(format!("Matches exclude pattern: {}", pattern)));
            }
        }

        (true, None)
    }

    /// Process a single file
    fn process_single_file(
        path: &Path,
        config: &FileProcessorConfig,
    ) -> Result<usize, UveddiError> {
        let content = std::fs::read(path).map_err(|e| UveddiError::PathError {
            path: path.display().to_string(),
            reason: format!("Failed to read file: {}", e),
            suggestion: "Check file permissions and availability".to_string(),
        })?;

        // Basic validation - ensure it's text content for source code analysis
        if content.len() > config.max_file_size {
            return Err(UveddiError::config_error(
                &format!("File {} exceeds maximum size", path.display()),
                "file size validation",
            ));
        }

        Ok(content.len())
    }
}

impl Default for FileProcessorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_files: num_cpus::get(),
            include_extensions: vec![
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "jsx".to_string(),
                "go".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "h".to_string(),
                "hpp".to_string(),
            ],
            exclude_patterns: vec![
                "target/".to_string(),
                "node_modules/".to_string(),
                ".git/".to_string(),
                "build/".to_string(),
                "dist/".to_string(),
                "__pycache__/".to_string(),
            ],
            max_file_size: 1024 * 1024, // 1MB
            recursive: true,
            follow_symlinks: false,
            read_buffer_size: 8192,
        }
    }
}

impl Service for FileProcessor {
    async fn start(&mut self) -> Result<(), UveddiError> {
        if self.is_running {
            return Ok(());
        }

        debug!("Starting file processor service");
        self.is_running = true;
        self.reset_stats();
        info!("File processor service started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), UveddiError> {
        if !self.is_running {
            return Ok(());
        }

        debug!("Stopping file processor service");
        self.is_running = false;
        info!("File processor service stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn name(&self) -> &str {
        "file_processor"
    }
}

impl HealthCheck for FileProcessor {
    async fn health_check(&self) -> Result<ServiceHealth, UveddiError> {
        if !self.is_running {
            return Ok(ServiceHealth::Unhealthy("Service not running".to_string()));
        }

        // Check if we can access the file system
        let temp_dir = std::env::temp_dir();
        if !temp_dir.exists() {
            return Ok(ServiceHealth::Unhealthy(
                "Cannot access file system".to_string(),
            ));
        }

        Ok(ServiceHealth::Healthy)
    }
}
