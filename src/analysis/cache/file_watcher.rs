#![cfg(feature = "ast-cache")]

//! Configurable File Watcher for Cache Invalidation
//!
//! This module provides a flexible file watching system that monitors
//! source files for changes and triggers cache invalidation as needed.

use crate::core::logging::{debug, info, warn};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;

/// File watcher configuration
#[derive(Debug, Clone)]
/// Configuration options for filewatcher.
pub struct FileWatcherConfig {
    /// Enable file watching
    pub enabled: bool,
    /// Polling interval for file changes (in milliseconds)
    pub polling_interval_ms: u64,
    /// File patterns to watch for changes
    pub watch_patterns: Vec<String>,
    /// Patterns to ignore
    pub ignore_patterns: Vec<String>,
    /// Debounce duration to avoid excessive invalidations
    pub debounce_ms: u64,
    /// Maximum number of files to watch
    pub max_watched_files: Option<usize>,
    /// Watch subdirectories recursively
    pub recursive: bool,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            polling_interval_ms: 1000,
            watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
                "**/*.jsx".to_string(),
                "**/*.tsx".to_string(),
            ],
            ignore_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/.*".to_string(),
                "**/build/**".to_string(),
                "**/dist/**".to_string(),
            ],
            debounce_ms: 500,
            max_watched_files: Some(10000),
            recursive: true,
        }
    }
}

/// File change event
#[derive(Debug, Clone)]
/// Represents file change event in the system.
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub event_type: FileEventType,
    pub timestamp: Instant,
}

/// Types of file events we care about
#[derive(Debug, Clone, PartialEq, Eq)]
/// Enumeration of fileeventtype variants.
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Cache invalidation callback trait
pub trait CacheInvalidationCallback: Send + Sync {
    /// Called when files are modified and cache should be invalidated
    fn invalidate_cache(&self, files: Vec<PathBuf>);

    /// Called when specific cache keys should be invalidated
    fn invalidate_keys(&self, keys: Vec<String>);
}

/// File watcher implementation
pub struct ConfigurableFileWatcher {
    config: FileWatcherConfig,
    watched_paths: Arc<RwLock<HashSet<PathBuf>>>,
    pending_changes: Arc<RwLock<HashMap<PathBuf, Instant>>>,
    callbacks: Arc<RwLock<Vec<Arc<dyn CacheInvalidationCallback>>>>,
    _watcher: Option<RecommendedWatcher>,
    _debounce_handle: Option<tokio::task::JoinHandle<()>>,
}

use std::collections::HashMap;

impl ConfigurableFileWatcher {
    /// Create a new file watcher with the given configuration
    pub fn new(
        config: FileWatcherConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            config,
            watched_paths: Arc::new(RwLock::new(HashSet::new())),
            pending_changes: Arc::new(RwLock::new(HashMap::new())),
            callbacks: Arc::new(RwLock::new(Vec::new())),
            _watcher: None,
            _debounce_handle: None,
        })
    }

    /// Add a cache invalidation callback
    pub async fn add_callback(&self, callback: Arc<dyn CacheInvalidationCallback>) {
        let mut callbacks = self.callbacks.write().await;
        callbacks.push(callback);
    }

    /// Start watching the specified directory
    pub async fn start_watching<P: AsRef<Path>>(
        &mut self,
        root_path: P,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            info!("File watching is disabled");
            return Ok(());
        }

        let root_path = root_path.as_ref().to_path_buf();
        info!("Starting file watcher for: {}", root_path.display());

        // Set up the file watcher
        let (tx, mut rx) = mpsc::channel(1000);
        let watched_paths = Arc::clone(&self.watched_paths);
        let pending_changes = Arc::clone(&self.pending_changes);

        // Create the watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| match res {
                Ok(event) => {
                    if let Err(e) = tx.blocking_send(event) {
                        warn!("Failed to send file event: {}", e);
                    }
                }
                Err(e) => {
                    warn!("File watcher error: {}", e);
                }
            },
            Config::default()
                .with_poll_interval(Duration::from_millis(self.config.polling_interval_ms)),
        )?;

        // Watch the root directory
        let watch_mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        watcher.watch(&root_path, watch_mode)?;

        // Store the watcher to keep it alive
        self._watcher = Some(watcher);

        // Start the event processing task
        let config = self.config.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                Self::process_file_event(event, &config, &watched_paths, &pending_changes).await;
            }
        });

        // Start the debounce task
        self.start_debounce_task().await;

        info!("File watcher started successfully");
        Ok(())
    }

    /// Process individual file events
    async fn process_file_event(
        event: notify::Event,
        config: &FileWatcherConfig,
        watched_paths: &Arc<RwLock<HashSet<PathBuf>>>,
        pending_changes: &Arc<RwLock<HashMap<PathBuf, Instant>>>,
    ) {
        // Filter events we care about
        let event_type = match event.kind {
            EventKind::Create(_) => FileEventType::Created,
            EventKind::Modify(_) => FileEventType::Modified,
            EventKind::Remove(_) => FileEventType::Deleted,
            _ => return, // Ignore other events
        };

        for path in event.paths {
            // Check if this file matches our patterns
            if !Self::should_watch_file(&path, config) {
                continue;
            }

            debug!("File event: {:?} - {}", event_type, path.display());

            // Add to watched files
            {
                let mut watched = watched_paths.write().await;

                // Check max file limit
                if let Some(max_files) = config.max_watched_files {
                    if watched.len() >= max_files {
                        warn!("Maximum watched files limit reached ({})", max_files);
                        continue;
                    }
                }

                watched.insert(path.clone());
            }

            // Add to pending changes for debouncing
            {
                let mut pending = pending_changes.write().await;
                pending.insert(path, Instant::now());
            }
        }
    }

    /// Check if a file should be watched based on patterns
    fn should_watch_file(path: &Path, config: &FileWatcherConfig) -> bool {
        let path_str = path.to_string_lossy();

        // Check ignore patterns first
        for ignore_pattern in &config.ignore_patterns {
            if Self::matches_pattern(&path_str, ignore_pattern) {
                return false;
            }
        }

        // Check watch patterns
        for watch_pattern in &config.watch_patterns {
            if Self::matches_pattern(&path_str, watch_pattern) {
                return true;
            }
        }

        false
    }

    /// Simple pattern matching (supports basic glob patterns)
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        if pattern == "**" {
            return true;
        }

        // Handle **/dirname/** pattern (matches directory anywhere in path)
        if pattern.starts_with("**/") && pattern.ends_with("/**") {
            let middle = &pattern[3..pattern.len() - 3];
            // Match if path contains /dirname/ or starts with dirname/ or ends with /dirname
            return path.contains(&format!("/{}/", middle))
                || path.starts_with(&format!("{}/", middle))
                || path.contains(&format!("/{}", middle));
        }

        // Handle **/*.ext pattern (any file with extension in any subdirectory)
        if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            // If suffix is like *.rs, check extension
            if suffix.starts_with("*.") {
                let extension = &suffix[1..]; // ".rs"
                return path.ends_with(extension);
            }
            // Otherwise check for literal suffix
            return path.contains(suffix) || path.ends_with(suffix);
        }

        // Handle prefix/** pattern (anything under a directory)
        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            return path.starts_with(prefix) || path.contains(&format!("/{}/", prefix));
        }

        // Handle *.ext pattern (file with extension in current directory)
        if pattern.starts_with("*.") {
            let extension = &pattern[1..]; // ".rs"
            return path.ends_with(extension);
        }

        // Exact match or contains
        path == pattern || path.contains(pattern)
    }

    /// Start the debounce task to batch file changes
    async fn start_debounce_task(&mut self) {
        let pending_changes = Arc::clone(&self.pending_changes);
        let debounce_duration = Duration::from_millis(self.config.debounce_ms);
        let callbacks = Arc::clone(&self.callbacks);

        let handle = tokio::spawn(async move {
            let mut interval_timer = interval(debounce_duration);

            loop {
                interval_timer.tick().await;

                let mut to_invalidate = Vec::new();

                // Check for changes that have been pending long enough
                {
                    let mut pending = pending_changes.write().await;
                    let now = Instant::now();

                    pending.retain(|path, timestamp| {
                        if now.duration_since(*timestamp) >= debounce_duration {
                            to_invalidate.push(path.clone());
                            false // Remove from pending
                        } else {
                            true // Keep pending
                        }
                    });
                }

                // Trigger invalidation if we have changes
                if !to_invalidate.is_empty() {
                    debug!("Invalidating cache for {} files", to_invalidate.len());

                    let callbacks_guard = callbacks.read().await;
                    for callback in callbacks_guard.iter() {
                        callback.invalidate_cache(to_invalidate.clone());

                        // Also invalidate cache keys (convert paths to keys)
                        let keys: Vec<String> = to_invalidate
                            .iter()
                            .map(|path| path.to_string_lossy().to_string())
                            .collect();
                        callback.invalidate_keys(keys);
                    }
                }
            }
        });

        self._debounce_handle = Some(handle);
    }

    /// Stop watching files
    pub async fn stop_watching(&mut self) {
        info!("Stopping file watcher");

        // Drop the watcher to stop watching
        self._watcher = None;

        // Cancel debounce task
        if let Some(handle) = self._debounce_handle.take() {
            handle.abort();
        }

        // Clear watched files
        {
            let mut watched = self.watched_paths.write().await;
            watched.clear();
        }

        // Clear pending changes
        {
            let mut pending = self.pending_changes.write().await;
            pending.clear();
        }

        info!("File watcher stopped");
    }

    /// Get statistics about the file watcher
    pub async fn get_stats(&self) -> FileWatcherStats {
        let watched_paths = self.watched_paths.read().await;
        let pending_changes = self.pending_changes.read().await;

        FileWatcherStats {
            enabled: self.config.enabled,
            watched_file_count: watched_paths.len(),
            pending_changes_count: pending_changes.len(),
            polling_interval_ms: self.config.polling_interval_ms,
            debounce_ms: self.config.debounce_ms,
            max_watched_files: self.config.max_watched_files,
        }
    }
}

/// File watcher statistics
#[derive(Debug, Clone)]
/// Represents file watcher stats in the system.
pub struct FileWatcherStats {
    pub enabled: bool,
    pub watched_file_count: usize,
    pub pending_changes_count: usize,
    pub polling_interval_ms: u64,
    pub debounce_ms: u64,
    pub max_watched_files: Option<usize>,
}

/// Simple cache invalidation callback implementation for testing
pub struct SimpleCacheCallback {
    pub invalidated_files: Arc<RwLock<Vec<PathBuf>>>,
    pub invalidated_keys: Arc<RwLock<Vec<String>>>,
}

impl SimpleCacheCallback {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            invalidated_files: Arc::new(RwLock::new(Vec::new())),
            invalidated_keys: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl CacheInvalidationCallback for SimpleCacheCallback {
    fn invalidate_cache(&self, files: Vec<PathBuf>) {
        let invalidated_files = Arc::clone(&self.invalidated_files);
        tokio::spawn(async move {
            let mut guard = invalidated_files.write().await;
            guard.extend(files);
        });
    }

    fn invalidate_keys(&self, keys: Vec<String>) {
        let invalidated_keys = Arc::clone(&self.invalidated_keys);
        tokio::spawn(async move {
            let mut guard = invalidated_keys.write().await;
            guard.extend(keys);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::sleep;

    #[test]
    fn test_pattern_matching() {
        let config = FileWatcherConfig::default();

        // Test watch patterns
        assert!(ConfigurableFileWatcher::should_watch_file(
            Path::new("src/main.rs"),
            &config
        ));

        assert!(ConfigurableFileWatcher::should_watch_file(
            Path::new("test.py"),
            &config
        ));

        // Test ignore patterns
        assert!(!ConfigurableFileWatcher::should_watch_file(
            Path::new("target/debug/main"),
            &config
        ));

        assert!(!ConfigurableFileWatcher::should_watch_file(
            Path::new("node_modules/package/index.js"),
            &config
        ));
    }

    #[tokio::test]
    async fn test_file_watcher_configuration() {
        let config = FileWatcherConfig {
            enabled: true,
            polling_interval_ms: 100,
            debounce_ms: 50,
            max_watched_files: Some(5),
            ..Default::default()
        };

        let mut watcher = ConfigurableFileWatcher::new(config).unwrap();
        let callback = Arc::new(SimpleCacheCallback::new());
        let callback_files = Arc::clone(&callback.invalidated_files);

        watcher.add_callback(callback).await;

        // Create temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        // Start watching
        watcher.start_watching(temp_dir.path()).await.unwrap();

        // Create a file
        fs::write(&test_file, "fn main() {}").unwrap();

        // Wait for debounce
        sleep(Duration::from_millis(200)).await;

        // Check that callback was triggered
        let invalidated = callback_files.read().await;
        assert!(!invalidated.is_empty());

        // Stop watching
        watcher.stop_watching().await;

        let stats = watcher.get_stats().await;
        assert_eq!(stats.watched_file_count, 0); // Should be cleared after stopping
    }

    #[test]
    fn test_glob_pattern_matching() {
        assert!(ConfigurableFileWatcher::matches_pattern(
            "src/main.rs",
            "**/*.rs"
        ));
        assert!(ConfigurableFileWatcher::matches_pattern(
            "deep/nested/file.js",
            "**/*.js"
        ));
        assert!(!ConfigurableFileWatcher::matches_pattern(
            "file.txt", "**/*.rs"
        ));

        assert!(ConfigurableFileWatcher::matches_pattern(
            "target/debug/main",
            "**/target/**"
        ));
        assert!(ConfigurableFileWatcher::matches_pattern(
            "node_modules/react/index.js",
            "**/node_modules/**"
        ));

        assert!(ConfigurableFileWatcher::matches_pattern("test.rs", "*.rs"));
        assert!(!ConfigurableFileWatcher::matches_pattern("test.py", "*.rs"));
    }
}
