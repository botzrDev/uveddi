//! Change Detection System for Incremental Analysis
//!
//! Provides efficient file change detection using multiple strategies:
//! - File modification time checking
//! - Content hash comparison (Blake3 for performance)
//! - File system events (for future real-time detection)
//!
//! Designed for enterprise-scale codebases with <1% false positive rate.

use super::{ChangeDetectionConfig, IncrementalAnalysisError, Result};
use crate::core::logging::{debug, info, warn};
use blake3::Hasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tokio::fs;
use tokio::task::JoinSet;

/// Represents the state of a file for change detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Data structure for filestate.
pub struct FileState {
    /// Last modification time as Unix timestamp
    pub last_modified: u64,

    /// Content hash for precise change detection
    pub content_hash: String,

    /// File size in bytes
    pub file_size: u64,

    /// Direct dependencies of this file
    pub dependencies: HashSet<PathBuf>,

    /// Files that depend on this file
    pub dependents: HashSet<PathBuf>,

    /// Last time this file was analyzed
    pub last_analyzed: DateTime<Utc>,

    /// Whether this file exists (for tracking deletions)
    pub exists: bool,
}

/// Represents a set of detected changes
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for changeset.
pub struct ChangeSet {
    /// Files that have been modified
    pub modified: HashSet<PathBuf>,

    /// Files that have been newly created
    pub added: HashSet<PathBuf>,

    /// Files that have been deleted
    pub deleted: HashSet<PathBuf>,

    /// Files affected by dependency changes
    pub affected_by_dependencies: HashSet<PathBuf>,

    /// Timestamp when changes were detected
    pub detection_time: DateTime<Utc>,

    /// Change detection method used
    pub detection_method: ChangeDetectionMethod,
}

/// Method used for change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeDetectionMethod {
    /// Only modification time was checked
    ModificationTime,

    /// Content hash was computed and compared
    ContentHash,

    /// Both modification time and content hash were used
    Hybrid,

    /// File system events were used (future feature)
    FileSystemEvents,
}

/// Change detector that tracks file modifications efficiently
pub struct ChangeDetector {
    /// Configuration for change detection
    config: ChangeDetectionConfig,

    /// Current file states indexed by path
    file_states: HashMap<PathBuf, FileState>,

    /// Ignore patterns for change detection
    ignore_patterns: Vec<glob::Pattern>,

    /// Statistics for performance monitoring
    stats: ChangeDetectionStats,
}

/// Statistics for change detection performance
#[derive(Debug, Clone, Default)]
/// Data structure for changedetectionstats.
pub struct ChangeDetectionStats {
    pub files_checked: u64,
    pub files_changed: u64,
    pub cache_hits: u64,
    pub hash_computations: u64,
    pub total_detection_time_ms: u64,
    pub average_detection_time_ms: f64,
}

impl ChangeDetector {
    /// Creates a new change detector with the given configuration
    pub fn new(config: ChangeDetectionConfig) -> Result<Self> {
        let ignore_patterns = config
            .ignore_patterns
            .iter()
            .map(|pattern| {
                glob::Pattern::new(pattern).map_err(|e| {
                    IncrementalAnalysisError::ConfigurationError {
                        message: format!("Invalid ignore pattern '{}': {}", pattern, e),
                    }
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            config,
            file_states: HashMap::new(),
            ignore_patterns,
            stats: ChangeDetectionStats::default(),
        })
    }

    /// Loads existing file states from the previous analysis
    pub fn load_file_states(&mut self, states: HashMap<PathBuf, FileState>) {
        self.file_states = states;
        info!(
            "Loaded {} file states for change detection",
            self.file_states.len()
        );
    }

    /// Detects changes in the given directory or file
    pub async fn detect_changes(&mut self, root_path: &Path) -> Result<ChangeSet> {
        info!("Starting change detection for: {}", root_path.display());
        let start_time = std::time::Instant::now();

        let mut changeset = ChangeSet {
            modified: HashSet::new(),
            added: HashSet::new(),
            deleted: HashSet::new(),
            affected_by_dependencies: HashSet::new(),
            detection_time: Utc::now(),
            detection_method: if self.config.use_content_hash {
                if self.config.check_mtime {
                    ChangeDetectionMethod::Hybrid
                } else {
                    ChangeDetectionMethod::ContentHash
                }
            } else {
                ChangeDetectionMethod::ModificationTime
            },
        };

        // Step 1: Discover all current files
        let current_files = self.discover_files(root_path).await?;

        // Step 2: Detect new and deleted files
        self.detect_additions_and_deletions(&current_files, &mut changeset)
            .await;

        // Step 3: Check existing files for modifications
        self.detect_modifications(&current_files, &mut changeset)
            .await?;

        // Update statistics
        let elapsed = start_time.elapsed();
        self.stats.total_detection_time_ms += elapsed.as_millis() as u64;
        self.stats.files_checked += current_files.len() as u64;
        self.stats.files_changed += (changeset.modified.len() + changeset.added.len()) as u64;

        if self.stats.files_checked > 0 {
            self.stats.average_detection_time_ms =
                self.stats.total_detection_time_ms as f64 / self.stats.files_checked as f64;
        }

        info!(
            "Change detection completed in {:?}: {} modified, {} added, {} deleted",
            elapsed,
            changeset.modified.len(),
            changeset.added.len(),
            changeset.deleted.len()
        );

        Ok(changeset)
    }

    /// Discovers all files in the given path that should be analyzed
    async fn discover_files(&self, root_path: &Path) -> Result<HashSet<PathBuf>> {
        let mut files = HashSet::new();

        if root_path.is_file() {
            if !self.should_ignore_file(root_path) {
                files.insert(root_path.to_path_buf());
            }
            return Ok(files);
        }

        // Use async directory traversal for better performance
        let mut entries = fs::read_dir(root_path).await.map_err(|e| {
            IncrementalAnalysisError::ChangeDetectionError {
                message: format!("Failed to read directory {}: {}", root_path.display(), e),
            }
        })?;

        let mut join_set = JoinSet::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            IncrementalAnalysisError::ChangeDetectionError {
                message: format!("Failed to read directory entry: {}", e),
            }
        })? {
            let path = entry.path();

            if self.should_ignore_file(&path) {
                continue;
            }

            if path.is_dir() {
                // Recursively scan subdirectories
                let ignore_patterns = self.ignore_patterns.clone();
                join_set.spawn(async move {
                    Self::discover_files_recursive(path, ignore_patterns).await
                });
            } else if self.is_source_file(&path) {
                files.insert(path);
            }
        }

        // Collect results from parallel directory scans
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(discovered_files)) => {
                    files.extend(discovered_files);
                }
                Ok(Err(e)) => {
                    warn!("Failed to discover files in subdirectory: {}", e);
                }
                Err(e) => {
                    warn!("Task join error during file discovery: {}", e);
                }
            }
        }

        debug!("Discovered {} files for analysis", files.len());
        Ok(files)
    }

    /// Recursively discovers files in a directory (for parallel execution)
    async fn discover_files_recursive(
        dir_path: PathBuf,
        ignore_patterns: Vec<glob::Pattern>,
    ) -> Result<HashSet<PathBuf>> {
        let mut files = HashSet::new();
        let mut stack = vec![dir_path];

        while let Some(current_path) = stack.pop() {
            if Self::should_ignore_with_patterns(&current_path, &ignore_patterns) {
                continue;
            }

            let mut entries = fs::read_dir(&current_path).await.map_err(|e| {
                IncrementalAnalysisError::ChangeDetectionError {
                    message: format!("Failed to read directory {}: {}", current_path.display(), e),
                }
            })?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                IncrementalAnalysisError::ChangeDetectionError {
                    message: format!("Failed to read directory entry: {}", e),
                }
            })? {
                let path = entry.path();

                if Self::should_ignore_with_patterns(&path, &ignore_patterns) {
                    continue;
                }

                if path.is_dir() {
                    stack.push(path);
                } else if Self::is_source_file_static(&path) {
                    files.insert(path);
                }
            }
        }

        Ok(files)
    }

    /// Detects new files and deleted files
    async fn detect_additions_and_deletions(
        &mut self,
        current_files: &HashSet<PathBuf>,
        changeset: &mut ChangeSet,
    ) {
        // Find new files (present in current_files but not in file_states)
        for file_path in current_files {
            if !self.file_states.contains_key(file_path) {
                changeset.added.insert(file_path.clone());
                debug!("New file detected: {}", file_path.display());
            }
        }

        // Find deleted files (present in file_states but not in current_files)
        let existing_files: HashSet<PathBuf> = self.file_states.keys().cloned().collect();
        for file_path in &existing_files {
            if !current_files.contains(file_path) {
                changeset.deleted.insert(file_path.clone());
                debug!("Deleted file detected: {}", file_path.display());

                // Mark as deleted in file state
                if let Some(file_state) = self.file_states.get_mut(file_path) {
                    file_state.exists = false;
                }
            }
        }
    }

    /// Detects modifications in existing files
    async fn detect_modifications(
        &mut self,
        current_files: &HashSet<PathBuf>,
        changeset: &mut ChangeSet,
    ) -> Result<()> {
        let mut join_set = JoinSet::new();
        let use_content_hash = self.config.use_content_hash;
        let check_mtime = self.config.check_mtime;

        // Process files in parallel for better performance
        for file_path in current_files {
            if self.file_states.contains_key(file_path) {
                let file_path_clone = file_path.clone();
                let existing_state = self.file_states.get(file_path).unwrap().clone();

                join_set.spawn(async move {
                    Self::check_file_modification(
                        file_path_clone,
                        existing_state,
                        use_content_hash,
                        check_mtime,
                    )
                    .await
                });
            }
        }

        // Collect modification check results
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok((file_path, is_modified, new_state))) => {
                    if is_modified {
                        changeset.modified.insert(file_path.clone());
                        debug!("Modified file detected: {}", file_path.display());
                    }

                    // Update file state regardless of modification status
                    self.file_states.insert(file_path, new_state);
                }
                Ok(Err(e)) => {
                    warn!("Failed to check file modification: {}", e);
                }
                Err(e) => {
                    warn!("Task join error during modification check: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Checks if a single file has been modified
    async fn check_file_modification(
        file_path: PathBuf,
        existing_state: FileState,
        use_content_hash: bool,
        check_mtime: bool,
    ) -> Result<(PathBuf, bool, FileState)> {
        let metadata = fs::metadata(&file_path).await.map_err(|e| {
            IncrementalAnalysisError::ChangeDetectionError {
                message: format!("Failed to get metadata for {}: {}", file_path.display(), e),
            }
        })?;

        let current_mtime = metadata
            .modified()
            .map_err(|e| IncrementalAnalysisError::ChangeDetectionError {
                message: format!(
                    "Failed to get modification time for {}: {}",
                    file_path.display(),
                    e
                ),
            })?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IncrementalAnalysisError::ChangeDetectionError {
                message: format!(
                    "Invalid modification time for {}: {}",
                    file_path.display(),
                    e
                ),
            })?
            .as_secs();

        let current_size = metadata.len();
        let mut is_modified = false;

        // Quick check: modification time and size
        if check_mtime
            && (current_mtime != existing_state.last_modified
                || current_size != existing_state.file_size)
        {
            is_modified = true;
        }

        // Detailed check: content hash (if configured and quick check suggests change)
        let content_hash = if use_content_hash && (is_modified || !check_mtime) {
            let hash = Self::compute_file_hash(&file_path).await?;
            if hash != existing_state.content_hash {
                is_modified = true;
            }
            hash
        } else {
            existing_state.content_hash.clone()
        };

        let new_state = FileState {
            last_modified: current_mtime,
            content_hash,
            file_size: current_size,
            dependencies: existing_state.dependencies, // Will be updated by dependency tracker
            dependents: existing_state.dependents,     // Will be updated by dependency tracker
            last_analyzed: if is_modified {
                Utc::now()
            } else {
                existing_state.last_analyzed
            },
            exists: true,
        };

        Ok((file_path, is_modified, new_state))
    }

    /// Computes Blake3 hash of a file's content
    async fn compute_file_hash(file_path: &Path) -> Result<String> {
        let content = fs::read(file_path).await.map_err(|e| {
            IncrementalAnalysisError::ChangeDetectionError {
                message: format!("Failed to read file {}: {}", file_path.display(), e),
            }
        })?;

        let mut hasher = Hasher::new();
        hasher.update(&content);
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Updates file states for new files
    pub async fn update_file_states_for_new_files(
        &mut self,
        new_files: &HashSet<PathBuf>,
    ) -> Result<()> {
        let mut join_set = JoinSet::new();
        let use_content_hash = self.config.use_content_hash;

        for file_path in new_files {
            let file_path_clone = file_path.clone();
            join_set.spawn(async move {
                Self::create_initial_file_state(file_path_clone, use_content_hash).await
            });
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok((file_path, file_state))) => {
                    self.file_states.insert(file_path, file_state);
                }
                Ok(Err(e)) => {
                    warn!("Failed to create file state for new file: {}", e);
                }
                Err(e) => {
                    warn!("Task join error during file state creation: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Creates initial file state for a new file
    async fn create_initial_file_state(
        file_path: PathBuf,
        use_content_hash: bool,
    ) -> Result<(PathBuf, FileState)> {
        let metadata = fs::metadata(&file_path).await.map_err(|e| {
            IncrementalAnalysisError::ChangeDetectionError {
                message: format!("Failed to get metadata for {}: {}", file_path.display(), e),
            }
        })?;

        let last_modified = metadata
            .modified()
            .map_err(|e| IncrementalAnalysisError::ChangeDetectionError {
                message: format!(
                    "Failed to get modification time for {}: {}",
                    file_path.display(),
                    e
                ),
            })?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IncrementalAnalysisError::ChangeDetectionError {
                message: format!(
                    "Invalid modification time for {}: {}",
                    file_path.display(),
                    e
                ),
            })?
            .as_secs();

        let content_hash = if use_content_hash {
            Self::compute_file_hash(&file_path).await?
        } else {
            String::new()
        };

        let file_state = FileState {
            last_modified,
            content_hash,
            file_size: metadata.len(),
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            last_analyzed: Utc::now(),
            exists: true,
        };

        Ok((file_path, file_state))
    }

    /// Returns the current file states
    pub fn get_file_states(&self) -> &HashMap<PathBuf, FileState> {
        &self.file_states
    }

    /// Returns change detection statistics
    pub fn get_stats(&self) -> &ChangeDetectionStats {
        &self.stats
    }

    /// Checks if a file should be ignored based on configured patterns
    fn should_ignore_file(&self, path: &Path) -> bool {
        Self::should_ignore_with_patterns(path, &self.ignore_patterns)
    }

    /// Checks if a file should be ignored using provided patterns
    fn should_ignore_with_patterns(path: &Path, patterns: &[glob::Pattern]) -> bool {
        let path_str = path.to_string_lossy();
        patterns.iter().any(|pattern| pattern.matches(&path_str))
    }

    /// Checks if a file is a source file that should be analyzed
    fn is_source_file(&self, path: &Path) -> bool {
        Self::is_source_file_static(path)
    }

    /// Static version of is_source_file for use in async contexts
    fn is_source_file_static(path: &Path) -> bool {
        // Support common source file extensions
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(
                extension.to_lowercase().as_str(),
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "java"
                    | "cpp"
                    | "c"
                    | "h"
                    | "hpp"
                    | "go"
                    | "rb"
                    | "php"
            )
        } else {
            false
        }
    }
}

impl Default for ChangeSet {
    fn default() -> Self {
        Self {
            modified: HashSet::new(),
            added: HashSet::new(),
            deleted: HashSet::new(),
            affected_by_dependencies: HashSet::new(),
            detection_time: Utc::now(),
            detection_method: ChangeDetectionMethod::Hybrid,
        }
    }
}

impl ChangeSet {
    /// Returns true if any changes were detected
    pub fn has_changes(&self) -> bool {
        !self.modified.is_empty() || !self.added.is_empty() || !self.deleted.is_empty()
    }

    /// Returns the total number of changed files
    pub fn total_changes(&self) -> usize {
        self.modified.len() + self.added.len() + self.deleted.len()
    }

    /// Returns all affected files (directly changed + affected by dependencies)
    pub fn all_affected_files(&self) -> HashSet<PathBuf> {
        let mut all_files = HashSet::new();
        all_files.extend(self.modified.iter().cloned());
        all_files.extend(self.added.iter().cloned());
        all_files.extend(self.affected_by_dependencies.iter().cloned());
        all_files
    }

    /// Calculates the change percentage for a given total file count
    pub fn change_percentage(&self, total_files: usize) -> f32 {
        if total_files == 0 {
            0.0
        } else {
            (self.total_changes() as f32 / total_files as f32) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_change_detector_creation() {
        let config = ChangeDetectionConfig::default();
        let detector = ChangeDetector::new(config).unwrap();
        assert_eq!(detector.file_states.len(), 0);
        assert!(!detector.ignore_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_file_hash_computation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"fn main() { println!(\"Hello, world!\"); }")
            .await
            .unwrap();
        file.flush().await.unwrap();

        let hash1 = ChangeDetector::compute_file_hash(&file_path).await.unwrap();
        let hash2 = ChangeDetector::compute_file_hash(&file_path).await.unwrap();

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[tokio::test]
    async fn test_change_detection_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = ChangeDetectionConfig::default();
        let mut detector = ChangeDetector::new(config).unwrap();

        // Create a new file
        let file_path = temp_dir.path().join("new_file.rs");
        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"fn test() {}").await.unwrap();
        file.flush().await.unwrap();

        let changeset = detector.detect_changes(temp_dir.path()).await.unwrap();
        assert_eq!(changeset.added.len(), 1);
        assert!(changeset.added.contains(&file_path));
        assert_eq!(changeset.modified.len(), 0);
        assert_eq!(changeset.deleted.len(), 0);
    }

    #[tokio::test]
    async fn test_change_set_operations() {
        let mut changeset = ChangeSet::default();
        assert!(!changeset.has_changes());
        assert_eq!(changeset.total_changes(), 0);
        assert_eq!(changeset.change_percentage(100), 0.0);

        changeset.modified.insert(PathBuf::from("file1.rs"));
        changeset.added.insert(PathBuf::from("file2.rs"));

        assert!(changeset.has_changes());
        assert_eq!(changeset.total_changes(), 2);
        assert_eq!(changeset.change_percentage(100), 2.0);
    }

    #[test]
    fn test_ignore_patterns() {
        let config = ChangeDetectionConfig::default();
        let detector = ChangeDetector::new(config).unwrap();

        assert!(detector.should_ignore_file(&PathBuf::from("test.tmp")));
        assert!(detector.should_ignore_file(&PathBuf::from(".git/config")));
        assert!(detector.should_ignore_file(&PathBuf::from("target/debug/main")));
        assert!(!detector.should_ignore_file(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_source_file_detection() {
        assert!(ChangeDetector::is_source_file_static(&PathBuf::from(
            "main.rs"
        )));
        assert!(ChangeDetector::is_source_file_static(&PathBuf::from(
            "script.py"
        )));
        assert!(ChangeDetector::is_source_file_static(&PathBuf::from(
            "app.js"
        )));
        assert!(ChangeDetector::is_source_file_static(&PathBuf::from(
            "component.tsx"
        )));
        assert!(!ChangeDetector::is_source_file_static(&PathBuf::from(
            "README.md"
        )));
        assert!(!ChangeDetector::is_source_file_static(&PathBuf::from(
            "Cargo.toml"
        )));
    }
}
