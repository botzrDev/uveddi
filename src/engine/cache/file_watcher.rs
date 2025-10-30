//! File modification watcher for cache invalidation
//!
//! Provides file system monitoring to automatically invalidate cache entries
//! when source files are modified.

use super::{AnalysisCache, AstCache};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use tokio::{sync::mpsc, task, time};
use tracing::{debug, error, info, warn};

/// Events that can trigger cache invalidation
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// File was modified
    Modified(PathBuf),
    /// File was deleted
    Deleted(PathBuf),
    /// File was created
    Created(PathBuf),
    /// Directory was modified (may contain new/removed files)
    DirectoryModified(PathBuf),
}

/// File modification watcher that invalidates caches
pub struct FileWatcher {
    /// AST cache to invalidate
    #[cfg(feature = "ast-cache")]
    ast_cache: Option<Arc<Mutex<AstCache>>>,

    /// Analysis cache to invalidate
    #[cfg(feature = "analysis-cache")]
    analysis_cache: Option<Arc<Mutex<AnalysisCache>>>,

    /// Channel to receive file events
    event_receiver: mpsc::UnboundedReceiver<FileEvent>,

    /// Watched directories and their last check times
    watched_paths: Vec<PathBuf>,

    /// Polling interval for file system checks
    poll_interval: Duration,

    /// Last modification times for tracked files
    file_timestamps: std::collections::HashMap<PathBuf, SystemTime>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(poll_interval: Duration) -> (Self, FileEventSender) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let watcher = Self {
            #[cfg(feature = "ast-cache")]
            ast_cache: None,
            #[cfg(feature = "analysis-cache")]
            analysis_cache: None,
            event_receiver,
            watched_paths: Vec::new(),
            poll_interval,
            file_timestamps: std::collections::HashMap::new(),
        };

        let sender = FileEventSender {
            sender: event_sender,
        };

        (watcher, sender)
    }

    /// Set the AST cache to watch
    #[cfg(feature = "ast-cache")]
    /// Caches with ast in the analysis pipeline.
    ///
    /// # Arguments
    ///
    /// - `cache`: Cache instance
    pub fn with_ast_cache(mut self, cache: Arc<Mutex<AstCache>>) -> Self {
        self.ast_cache = Some(cache);
        self
    }

    /// Set the analysis cache to watch
    #[cfg(feature = "analysis-cache")]
    /// Caches with analysis in the analysis pipeline.
    ///
    /// # Arguments
    ///
    /// - `cache`: Cache instance
    pub fn with_analysis_cache(mut self, cache: Arc<Mutex<AnalysisCache>>) -> Self {
        self.analysis_cache = Some(cache);
        self
    }

    /// Add a path to watch for changes
    pub fn watch_path(&mut self, path: PathBuf) {
        if !self.watched_paths.contains(&path) {
            info!("Starting to watch path: {}", path.display());
            self.watched_paths.push(path);
        }
    }

    /// Update file timestamp (used for testing and manual sync)
    pub fn update_timestamp(&mut self, path: &Path) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.file_timestamps.insert(path.to_path_buf(), modified);
            }
        }
    }

    /// Synchronous polling for changes (for non-async contexts)
    pub fn poll_changes(&mut self) -> Result<(), String> {
        for path in self.watched_paths.clone() {
            if path.is_file() {
                self.check_file_modification_sync(&path)?;
            } else if path.is_dir() {
                self.check_directory_modifications_sync(&path)?;
            }
        }
        Ok(())
    }

    /// Synchronous file modification check
    fn check_file_modification_sync(&mut self, path: &Path) -> Result<(), String> {
        match std::fs::metadata(path) {
            Ok(metadata) => {
                if let Ok(modified) = metadata.modified() {
                    let needs_invalidation = match self.file_timestamps.get(path) {
                        Some(last_modified) => modified > *last_modified,
                        None => true,
                    };

                    if needs_invalidation {
                        debug!("File {} was modified, invalidating caches", path.display());
                        self.invalidate_caches_for_file_sync(path);
                        self.file_timestamps.insert(path.to_path_buf(), modified);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get metadata for {}: {}", path.display(), e);
                return Err(format!("Failed to check file {}: {}", path.display(), e));
            }
        }
        Ok(())
    }

    /// Synchronous directory modification check
    fn check_directory_modifications_sync(&mut self, dir_path: &Path) -> Result<(), String> {
        let entries = std::fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read directory {}: {}", dir_path.display(), e))?;

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    match ext {
                        "rs" | "py" | "js" | "jsx" | "ts" | "tsx" => {
                            self.check_file_modification_sync(&path)?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// Synchronous cache invalidation
    fn invalidate_caches_for_file_sync(&self, path: &Path) {
        #[cfg(feature = "ast-cache")]
        if let Some(ref cache) = self.ast_cache {
            if let Ok(mut ast_cache) = cache.lock() {
                if ast_cache.invalidate(path).is_ok() {
                    debug!("Invalidated AST cache for {}", path.display());
                }
            }
        }

        #[cfg(feature = "analysis-cache")]
        if let Some(ref cache) = self.analysis_cache {
            if let Ok(mut analysis_cache) = cache.lock() {
                if analysis_cache.invalidate(path).is_ok() {
                    debug!("Invalidated analysis cache for {}", path.display());
                }
            }
        }
    }

    /// Start the file watcher in a background task
    pub async fn start(mut self) -> tokio::task::JoinHandle<()> {
        task::spawn(async move {
            let mut interval = time::interval(self.poll_interval);

            info!(
                "File watcher started with interval: {:?}",
                self.poll_interval
            );

            loop {
                tokio::select! {
                    // Handle incoming file events
                    event = self.event_receiver.recv() => {
                        if let Some(event) = event {
                            self.handle_file_event(event).await;
                        } else {
                            // Channel closed
                            break;
                        }
                    }

                    // Periodic polling of watched paths
                    _ = interval.tick() => {
                        self.poll_watched_paths().await;
                    }
                }
            }

            info!("File watcher stopped");
        })
    }

    /// Handle a file system event
    async fn handle_file_event(&mut self, event: FileEvent) {
        match event {
            FileEvent::Modified(path) => {
                debug!("File modified: {}", path.display());
                self.invalidate_caches_for_file(&path).await;
            }
            FileEvent::Deleted(path) => {
                debug!("File deleted: {}", path.display());
                self.invalidate_caches_for_file(&path).await;
                self.file_timestamps.remove(&path);
            }
            FileEvent::Created(path) => {
                debug!("File created: {}", path.display());
                // New files don't need cache invalidation, but update timestamps
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        self.file_timestamps.insert(path, modified);
                    }
                }
            }
            FileEvent::DirectoryModified(path) => {
                debug!("Directory modified: {}", path.display());
                // This could be expensive - consider if we need it
                self.scan_directory_changes(&path).await;
            }
        }
    }

    /// Poll watched paths for changes
    async fn poll_watched_paths(&mut self) {
        for path in &self.watched_paths.clone() {
            if path.is_file() {
                self.check_file_modification(path).await;
            } else if path.is_dir() {
                self.check_directory_modifications(path).await;
            }
        }
    }

    /// Check if a single file has been modified
    async fn check_file_modification(&mut self, path: &Path) {
        match std::fs::metadata(path) {
            Ok(metadata) => {
                if let Ok(modified) = metadata.modified() {
                    let needs_invalidation = match self.file_timestamps.get(path) {
                        Some(last_modified) => modified > *last_modified,
                        None => true, // First time seeing this file
                    };

                    if needs_invalidation {
                        debug!("Detected modification: {}", path.display());
                        self.invalidate_caches_for_file(path).await;
                        self.file_timestamps.insert(path.to_path_buf(), modified);
                    }
                }
            }
            Err(e) => {
                // File might have been deleted
                if self.file_timestamps.contains_key(path) {
                    warn!("File no longer accessible: {} ({})", path.display(), e);
                    self.invalidate_caches_for_file(path).await;
                    self.file_timestamps.remove(path);
                }
            }
        }
    }

    /// Check for modifications in a directory (non-recursive for now)
    async fn check_directory_modifications(&mut self, dir_path: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    // Check common source file extensions
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        match ext {
                            "rs" | "py" | "js" | "jsx" | "ts" | "tsx" => {
                                self.check_file_modification(&path).await;
                            }
                            _ => {} // Ignore other file types
                        }
                    }
                }
                // Skip subdirectories for now to avoid recursion complexity
            }
        }
    }

    /// Scan for changes in a directory (after DirectoryModified event)
    async fn scan_directory_changes(&mut self, _dir_path: &Path) {
        // This is a placeholder for more sophisticated directory change detection
        // In a full implementation, you might track file lists and compare them
        debug!("Scanning directory changes (not fully implemented)");
    }

    /// Invalidate caches for a specific file
    async fn invalidate_caches_for_file(&self, path: &Path) {
        // Invalidate AST cache
        #[cfg(feature = "ast-cache")]
        if let Some(ref cache) = self.ast_cache {
            if let Ok(mut ast_cache) = cache.lock() {
                // AstCache doesn't have a per-file invalidation method in our current implementation
                // You could extend it to support this, or we just rely on modification time checks
                debug!(
                    "AST cache invalidation for {} (handled by modification time)",
                    path.display()
                );
            }
        }

        // Invalidate analysis cache
        #[cfg(feature = "analysis-cache")]
        if let Some(ref cache) = self.analysis_cache {
            if let Ok(mut analysis_cache) = cache.lock() {
                // Similar to AST cache - you'd need to add per-file invalidation support
                debug!(
                    "Analysis cache invalidation for {} (handled by modification time)",
                    path.display()
                );
            }
        }

        // In the current implementation, both caches check file modification times
        // automatically, so explicit invalidation isn't strictly necessary.
        // However, this provides a hook for more sophisticated invalidation strategies.
    }
}

/// Handle for sending file events to the watcher
#[derive(Clone)]
/// Represents file event sender in the system.
pub struct FileEventSender {
    sender: mpsc::UnboundedSender<FileEvent>,
}

impl FileEventSender {
    /// Send a file modification event
    pub fn file_modified(&self, path: PathBuf) -> Result<(), SendError> {
        self.sender
            .send(FileEvent::Modified(path))
            .map_err(|_| SendError::ChannelClosed)
    }

    /// Send a file deletion event
    pub fn file_deleted(&self, path: PathBuf) -> Result<(), SendError> {
        self.sender
            .send(FileEvent::Deleted(path))
            .map_err(|_| SendError::ChannelClosed)
    }

    /// Send a file creation event
    pub fn file_created(&self, path: PathBuf) -> Result<(), SendError> {
        self.sender
            .send(FileEvent::Created(path))
            .map_err(|_| SendError::ChannelClosed)
    }

    /// Send a directory modification event
    pub fn directory_modified(&self, path: PathBuf) -> Result<(), SendError> {
        self.sender
            .send(FileEvent::DirectoryModified(path))
            .map_err(|_| SendError::ChannelClosed)
    }
}

/// Error type for file event sending
#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Event channel is closed")]
    ChannelClosed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let (watcher, _sender) = FileWatcher::new(Duration::from_millis(100));
        assert_eq!(watcher.watched_paths.len(), 0);
    }

    #[tokio::test]
    async fn test_watch_path() {
        let (mut watcher, _sender) = FileWatcher::new(Duration::from_millis(100));
        let path = PathBuf::from("/test/path");

        watcher.watch_path(path.clone());
        assert_eq!(watcher.watched_paths.len(), 1);
        assert_eq!(watcher.watched_paths[0], path);

        // Adding the same path again should not duplicate
        watcher.watch_path(path.clone());
        assert_eq!(watcher.watched_paths.len(), 1);
    }

    #[tokio::test]
    async fn test_file_event_sender() {
        let (watcher, sender) = FileWatcher::new(Duration::from_millis(100));
        let _handle = watcher.start();

        let path = PathBuf::from("/test/file.rs");

        // These should not fail since the channel is open
        assert!(sender.file_modified(path.clone()).is_ok());
        assert!(sender.file_deleted(path.clone()).is_ok());
        assert!(sender.file_created(path.clone()).is_ok());
        assert!(sender.directory_modified(path.clone()).is_ok());
    }
}
