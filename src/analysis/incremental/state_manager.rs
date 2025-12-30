//! State Management System for Incremental Analysis
//!
//! Provides persistent storage and recovery of incremental analysis state including:
//! - File states and dependency graphs
//! - Analysis results and cache metadata
//! - Configuration and performance metrics
//!
//! Designed for enterprise-scale state management with transactional safety.

use super::{
    ChangeDetector, DependencyGraph, DependencyTracker, FileState, IncrementalAnalysisConfig,
    IncrementalAnalysisError, Result,
};
use crate::core::logging::{debug, info, warn};
use crate::database::models::ArchitecturalIssue;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Comprehensive state for incremental analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for incrementalstate.
pub struct IncrementalState {
    /// State metadata and versioning
    pub metadata: StateMetadata,

    /// File states for change detection
    pub file_states: HashMap<PathBuf, FileState>,

    /// Dependency graph state
    pub dependency_graph: DependencyGraph,

    /// Cached analysis results indexed by file path
    pub analysis_cache: HashMap<PathBuf, CachedAnalysisResult>,

    /// Configuration used for this state
    pub config: IncrementalAnalysisConfig,

    /// Performance metrics from last analysis
    pub performance_metrics: StatePerformanceMetrics,

    /// Environment information when state was created
    pub environment_info: EnvironmentInfo,
}

/// Metadata about the incremental state
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for statemetadata.
pub struct StateMetadata {
    /// Unique identifier for this state
    pub state_id: String,

    /// State format version for migration compatibility
    pub version: u32,

    /// When this state was created
    pub created_at: DateTime<Utc>,

    /// When this state was last updated
    pub updated_at: DateTime<Utc>,

    /// Git commit hash when state was created (if available)
    pub git_commit: Option<String>,

    /// Project root path for validation
    pub project_root: PathBuf,

    /// State checksum for integrity validation
    pub checksum: String,
}

/// Cached analysis result for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for cachedanalysisresult.
pub struct CachedAnalysisResult {
    /// Issues found in the file
    pub issues: Vec<ArchitecturalIssue>,

    /// When this result was cached
    pub cached_at: DateTime<Utc>,

    /// File state when analysis was performed
    pub file_state_at_analysis: FileState,

    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,

    /// Whether this result is still valid
    pub is_valid: bool,
}

/// Performance metrics stored in state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Data structure for stateperformancemetrics.
pub struct StatePerformanceMetrics {
    /// Total files analyzed in last run
    pub files_analyzed: usize,

    /// Files that were incrementally skipped
    pub files_skipped: usize,

    /// Total analysis time in milliseconds
    pub total_analysis_time_ms: u64,

    /// Time saved through incremental analysis
    pub time_saved_ms: u64,

    /// Memory usage metrics
    pub memory_usage: MemoryUsageMetrics,

    /// Cache performance metrics
    pub cache_metrics: CachePerformanceMetrics,
}

/// Memory usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Data structure for memoryusagemetrics.
pub struct MemoryUsageMetrics {
    /// Peak memory usage during analysis (MB)
    pub peak_memory_mb: f64,

    /// State storage size on disk (bytes)
    pub state_size_bytes: u64,

    /// Memory efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Data structure for cacheperformancemetrics.
pub struct CachePerformanceMetrics {
    /// Analysis result cache hit rate
    pub analysis_cache_hit_rate: f64,

    /// Dependency graph cache hit rate
    pub dependency_cache_hit_rate: f64,

    /// Average cache lookup time (ms)
    pub average_lookup_time_ms: f64,

    /// Cache invalidation rate
    pub invalidation_rate: f64,
}

/// Environment information for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data structure for environmentinfo.
pub struct EnvironmentInfo {
    /// Rust compiler version
    pub rust_version: String,

    /// Operating system
    pub os_version: String,

    /// Uveddi version
    pub tool_version: String,

    /// CPU architecture
    pub cpu_arch: String,

    /// Timestamp of environment capture
    pub captured_at: DateTime<Utc>,
}

/// State management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for statemanager.
///
/// Provides sensible defaults that can be overridden via builder methods
/// or loaded from configuration files.
pub struct StateManagerConfig {
    /// Maximum state file size in MB
    pub max_state_size_mb: u64,

    /// Enable state compression
    pub enable_compression: bool,

    /// Backup retention policy
    pub backup_retention_days: u32,

    /// Automatic cleanup settings
    pub auto_cleanup: bool,

    /// State validation settings
    pub validation: StateValidationConfig,
}

/// State validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration options for statevalidation.
///
/// Provides sensible defaults that can be overridden via builder methods
/// or loaded from configuration files.
pub struct StateValidationConfig {
    /// Validate state checksum on load
    pub validate_checksum: bool,

    /// Validate project root matches
    pub validate_project_root: bool,

    /// Validate environment compatibility
    pub validate_environment: bool,

    /// Maximum age for state validity (hours)
    pub max_age_hours: u32,
}

/// State manager that handles persistence and recovery
pub struct IncrementalStateManager {
    /// Configuration for state management
    config: StateManagerConfig,

    /// Current state (protected by RwLock for concurrent access)
    state: RwLock<Option<IncrementalState>>,

    /// State file path
    state_file_path: PathBuf,

    /// Backup directory for state files
    backup_dir: PathBuf,

    /// State validation statistics
    validation_stats: RwLock<ValidationStats>,
}

/// Statistics for state validation operations
#[derive(Debug, Clone, Default)]
/// Data structure for validationstats.
pub struct ValidationStats {
    pub successful_loads: u64,
    pub failed_loads: u64,
    pub successful_saves: u64,
    pub failed_saves: u64,
    pub checksum_failures: u64,
    pub environment_mismatches: u64,
}

impl IncrementalStateManager {
    /// Creates a new state manager with the given configuration
    pub fn new(config: StateManagerConfig, state_file_path: PathBuf) -> Result<Self> {
        let backup_dir = state_file_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("backups");

        Ok(Self {
            config,
            state: RwLock::new(None),
            state_file_path,
            backup_dir,
            validation_stats: RwLock::new(ValidationStats::default()),
        })
    }

    /// Loads incremental state from disk
    pub async fn load_state(&self) -> Result<Option<IncrementalState>> {
        info!(
            "Loading incremental state from: {}",
            self.state_file_path.display()
        );

        if !self.state_file_path.exists() {
            info!("No existing state file found");
            return Ok(None);
        }

        let start_time = std::time::Instant::now();

        // Read state file
        let state_data = fs::read(&self.state_file_path).await.map_err(|e| {
            IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to read state file: {}", e),
            }
        })?;

        // Deserialize state
        let state: IncrementalState = if self.config.enable_compression {
            self.decompress_and_deserialize(&state_data)?
        } else {
            serde_json::from_slice(&state_data).map_err(|e| {
                IncrementalAnalysisError::StatePersistenceError {
                    message: format!("Failed to deserialize state: {}", e),
                }
            })?
        };

        // Validate state
        self.validate_state(&state).await?;

        // Update statistics
        {
            let mut stats = self.validation_stats.write().await;
            stats.successful_loads += 1;
        }

        let elapsed = start_time.elapsed();
        info!(
            "State loaded successfully in {:?}: {} files, {} dependencies",
            elapsed,
            state.file_states.len(),
            state.dependency_graph.total_dependencies()
        );

        // Store in memory
        {
            let mut current_state = self.state.write().await;
            *current_state = Some(state.clone());
        }

        Ok(Some(state))
    }

    /// Saves incremental state to disk
    pub async fn save_state(&self, state: &IncrementalState) -> Result<()> {
        info!(
            "Saving incremental state to: {}",
            self.state_file_path.display()
        );
        let start_time = std::time::Instant::now();

        // Create backup if existing state file exists
        if self.state_file_path.exists() {
            self.create_backup().await?;
        }

        // Prepare state for saving
        let mut state_to_save = state.clone();
        state_to_save.metadata.updated_at = Utc::now();
        state_to_save.metadata.checksum = self.calculate_state_checksum(&state_to_save)?;

        // Serialize state
        let state_data = if self.config.enable_compression {
            self.serialize_and_compress(&state_to_save)?
        } else {
            serde_json::to_vec_pretty(&state_to_save).map_err(|e| {
                IncrementalAnalysisError::StatePersistenceError {
                    message: format!("Failed to serialize state: {}", e),
                }
            })?
        };

        // Ensure parent directory exists
        if let Some(parent) = self.state_file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                IncrementalAnalysisError::StatePersistenceError {
                    message: format!("Failed to create state directory: {}", e),
                }
            })?;
        }

        // Write state file atomically
        let temp_file = self.state_file_path.with_extension("tmp");
        fs::write(&temp_file, &state_data).await.map_err(|e| {
            IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to write temporary state file: {}", e),
            }
        })?;

        fs::rename(&temp_file, &self.state_file_path)
            .await
            .map_err(|e| IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to rename state file: {}", e),
            })?;

        // Update in-memory state
        {
            let mut current_state = self.state.write().await;
            *current_state = Some(state_to_save);
        }

        // Update statistics
        {
            let mut stats = self.validation_stats.write().await;
            stats.successful_saves += 1;
        }

        let elapsed = start_time.elapsed();
        info!(
            "State saved successfully in {:?}: {} bytes",
            elapsed,
            state_data.len()
        );

        // Cleanup old backups if auto cleanup is enabled
        if self.config.auto_cleanup {
            self.cleanup_old_backups().await?;
        }

        Ok(())
    }

    /// Creates a new incremental state
    pub async fn create_new_state(
        &self,
        project_root: PathBuf,
        config: IncrementalAnalysisConfig,
    ) -> Result<IncrementalState> {
        info!(
            "Creating new incremental state for project: {}",
            project_root.display()
        );

        let state_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let metadata = StateMetadata {
            state_id,
            version: 1, // Current state format version
            created_at: now,
            updated_at: now,
            git_commit: self.get_git_commit().await,
            project_root: project_root.clone(),
            checksum: String::new(), // Will be calculated when saving
        };

        let environment_info = self.capture_environment_info().await;

        let state = IncrementalState {
            metadata,
            file_states: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            analysis_cache: HashMap::new(),
            config,
            performance_metrics: StatePerformanceMetrics::default(),
            environment_info,
        };

        // Store the new state in memory so subsequent operations can find it
        {
            let mut current_state = self.state.write().await;
            *current_state = Some(state.clone());
        }

        info!(
            "New incremental state created with ID: {}",
            state.metadata.state_id
        );
        Ok(state)
    }

    /// Updates the state with change detector data
    pub async fn update_state_with_change_detector(
        &self,
        change_detector: &ChangeDetector,
    ) -> Result<()> {
        let mut state_guard = self.state.write().await;
        if let Some(ref mut state) = *state_guard {
            // Update file states from change detector
            state.file_states = change_detector.get_file_states().clone();
            state.metadata.updated_at = Utc::now();

            debug!("Updated state with {} file states", state.file_states.len());
        } else {
            return Err(IncrementalAnalysisError::StatePersistenceError {
                message: "No state loaded to update".to_string(),
            });
        }

        Ok(())
    }

    /// Updates the state with dependency tracker data
    pub async fn update_state_with_dependency_tracker(
        &self,
        dependency_tracker: &DependencyTracker,
    ) -> Result<()> {
        let mut state_guard = self.state.write().await;
        if let Some(ref mut state) = *state_guard {
            // Update dependency graph from tracker
            state.dependency_graph = dependency_tracker.get_dependency_graph().clone();
            state.metadata.updated_at = Utc::now();

            debug!(
                "Updated state with dependency graph: {} files, {} dependencies",
                state.dependency_graph.total_files(),
                state.dependency_graph.total_dependencies()
            );
        } else {
            return Err(IncrementalAnalysisError::StatePersistenceError {
                message: "No state loaded to update".to_string(),
            });
        }

        Ok(())
    }

    /// Updates analysis cache with new results
    pub async fn update_analysis_cache(
        &self,
        file_path: PathBuf,
        issues: Vec<ArchitecturalIssue>,
        analysis_duration_ms: u64,
    ) -> Result<()> {
        let mut state_guard = self.state.write().await;
        if let Some(ref mut state) = *state_guard {
            // Get current file state
            let file_state = state
                .file_states
                .get(&file_path)
                .cloned()
                .unwrap_or_else(|| FileState {
                    last_modified: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    content_hash: String::new(),
                    file_size: 0,
                    dependencies: std::collections::HashSet::new(),
                    dependents: std::collections::HashSet::new(),
                    last_analyzed: Utc::now(),
                    exists: true,
                });

            let cached_result = CachedAnalysisResult {
                issues,
                cached_at: Utc::now(),
                file_state_at_analysis: file_state,
                analysis_duration_ms,
                is_valid: true,
            };

            state
                .analysis_cache
                .insert(file_path.clone(), cached_result);
            state.metadata.updated_at = Utc::now();

            debug!("Updated analysis cache for: {}", file_path.display());
        } else {
            return Err(IncrementalAnalysisError::StatePersistenceError {
                message: "No state loaded to update".to_string(),
            });
        }

        Ok(())
    }

    /// Gets cached analysis result for a file
    pub async fn get_cached_analysis(
        &self,
        file_path: &PathBuf,
    ) -> Result<Option<CachedAnalysisResult>> {
        let state_guard = self.state.read().await;
        if let Some(ref state) = *state_guard {
            Ok(state.analysis_cache.get(file_path).cloned())
        } else {
            Ok(None)
        }
    }

    /// Invalidates cache entries for changed files
    pub async fn invalidate_cache_for_files(
        &self,
        changed_files: &std::collections::HashSet<PathBuf>,
    ) -> Result<()> {
        let mut state_guard = self.state.write().await;
        if let Some(ref mut state) = *state_guard {
            let mut invalidated_count = 0;

            for file_path in changed_files {
                if let Some(cached_result) = state.analysis_cache.get_mut(file_path) {
                    cached_result.is_valid = false;
                    invalidated_count += 1;
                }
            }

            state.metadata.updated_at = Utc::now();
            debug!("Invalidated {} cache entries", invalidated_count);
        }

        Ok(())
    }

    /// Gets the current state (read-only)
    pub async fn get_current_state(&self) -> Option<IncrementalState> {
        let state_guard = self.state.read().await;
        state_guard.clone()
    }

    /// Validates the loaded state
    async fn validate_state(&self, state: &IncrementalState) -> Result<()> {
        debug!("Validating incremental state");

        // Validate checksum if enabled
        if self.config.validation.validate_checksum {
            let calculated_checksum = self.calculate_state_checksum(state)?;
            if calculated_checksum != state.metadata.checksum {
                let mut stats = self.validation_stats.write().await;
                stats.checksum_failures += 1;
                return Err(IncrementalAnalysisError::StatePersistenceError {
                    message: "State checksum validation failed".to_string(),
                });
            }
        }

        // Validate state age
        if self.config.validation.max_age_hours > 0 {
            let age_hours = (Utc::now() - state.metadata.updated_at).num_hours();
            if age_hours > self.config.validation.max_age_hours as i64 {
                warn!(
                    "State is {} hours old, exceeds maximum age of {} hours",
                    age_hours, self.config.validation.max_age_hours
                );
            }
        }

        // Validate environment compatibility if enabled
        if self.config.validation.validate_environment {
            let current_env = self.capture_environment_info().await;
            if current_env.rust_version != state.environment_info.rust_version {
                let mut stats = self.validation_stats.write().await;
                stats.environment_mismatches += 1;
                warn!(
                    "Environment mismatch: Rust version changed from {} to {}",
                    state.environment_info.rust_version, current_env.rust_version
                );
            }
        }

        debug!("State validation completed successfully");
        Ok(())
    }

    /// Calculates checksum for state integrity validation
    fn calculate_state_checksum(&self, state: &IncrementalState) -> Result<String> {
        // Create a copy without checksum for calculation
        let mut state_for_checksum = state.clone();
        state_for_checksum.metadata.checksum = String::new();

        let serialized = serde_json::to_vec(&state_for_checksum).map_err(|e| {
            IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to serialize state for checksum: {}", e),
            }
        })?;

        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&serialized);
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Creates a backup of the current state file
    async fn create_backup(&self) -> Result<()> {
        if !self.state_file_path.exists() {
            return Ok(());
        }

        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir).await.map_err(|e| {
            IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to create backup directory: {}", e),
            }
        })?;

        // Create timestamped backup filename
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("state_backup_{}.json", timestamp);
        let backup_path = self.backup_dir.join(backup_filename);

        // Copy current state file to backup
        fs::copy(&self.state_file_path, &backup_path)
            .await
            .map_err(|e| IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to create backup: {}", e),
            })?;

        debug!("Created backup: {}", backup_path.display());
        Ok(())
    }

    /// Cleans up old backup files
    async fn cleanup_old_backups(&self) -> Result<()> {
        if !self.backup_dir.exists() {
            return Ok(());
        }

        let retention_seconds = self.config.backup_retention_days as i64 * 24 * 3600;
        let cutoff_time = Utc::now() - chrono::Duration::seconds(retention_seconds);

        let mut entries = fs::read_dir(&self.backup_dir).await.map_err(|e| {
            IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to read backup directory: {}", e),
            }
        })?;

        let mut removed_count = 0;
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to read backup directory entry: {}", e),
            }
        })? {
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("state_backup_"))
                    .unwrap_or(false)
            {
                if let Ok(metadata) = fs::metadata(&path).await {
                    if let Ok(modified) = metadata.modified() {
                        if modified < SystemTime::from(cutoff_time) {
                            if let Err(e) = fs::remove_file(&path).await {
                                warn!("Failed to remove old backup {}: {}", path.display(), e);
                            } else {
                                removed_count += 1;
                                debug!("Removed old backup: {}", path.display());
                            }
                        }
                    }
                }
            }
        }

        if removed_count > 0 {
            info!("Cleaned up {} old backup files", removed_count);
        }

        Ok(())
    }

    /// Captures current environment information
    async fn capture_environment_info(&self) -> EnvironmentInfo {
        EnvironmentInfo {
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            os_version: std::env::consts::OS.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            cpu_arch: std::env::consts::ARCH.to_string(),
            captured_at: Utc::now(),
        }
    }

    /// Gets the current git commit hash
    async fn get_git_commit(&self) -> Option<String> {
        // Try to get from environment variable first
        if let Ok(commit) = std::env::var("GIT_COMMIT") {
            return Some(commit);
        }

        // Try to read from git directory
        // In a real implementation, you might use the git2 crate
        None
    }

    /// Compresses and serializes state data
    fn serialize_and_compress(&self, state: &IncrementalState) -> Result<Vec<u8>> {
        let serialized = serde_json::to_vec(state).map_err(|e| {
            IncrementalAnalysisError::StatePersistenceError {
                message: format!("Failed to serialize state: {}", e),
            }
        })?;

        // In a real implementation, you would use a compression library like zstd or gzip
        // For now, just return the serialized data
        Ok(serialized)
    }

    /// Decompresses and deserializes state data
    fn decompress_and_deserialize(&self, data: &[u8]) -> Result<IncrementalState> {
        // In a real implementation, you would decompress first
        // For now, just deserialize directly
        serde_json::from_slice(data).map_err(|e| IncrementalAnalysisError::StatePersistenceError {
            message: format!("Failed to deserialize state: {}", e),
        })
    }

    /// Gets validation statistics
    pub async fn get_validation_stats(&self) -> ValidationStats {
        let stats = self.validation_stats.read().await;
        stats.clone()
    }
}

impl Default for StateManagerConfig {
    fn default() -> Self {
        Self {
            max_state_size_mb: 100,
            enable_compression: true,
            backup_retention_days: 7,
            auto_cleanup: true,
            validation: StateValidationConfig::default(),
        }
    }
}

impl Default for StateValidationConfig {
    fn default() -> Self {
        Self {
            validate_checksum: true,
            validate_project_root: true,
            validate_environment: false, // Disabled by default to allow version updates
            max_age_hours: 24 * 7,       // 1 week
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_state_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test_state.json");
        let config = StateManagerConfig::default();

        let manager = IncrementalStateManager::new(config, state_file).unwrap();
        assert!(manager.get_current_state().await.is_none());
    }

    #[tokio::test]
    async fn test_new_state_creation() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test_state.json");
        let config = StateManagerConfig::default();
        let manager = IncrementalStateManager::new(config, state_file).unwrap();

        let project_root = temp_dir.path().to_path_buf();
        let analysis_config = IncrementalAnalysisConfig::default();

        let state = manager
            .create_new_state(project_root.clone(), analysis_config)
            .await
            .unwrap();

        assert_eq!(state.metadata.project_root, project_root);
        assert_eq!(state.metadata.version, 1);
        assert!(state.file_states.is_empty());
        assert_eq!(state.dependency_graph.total_files(), 0);
    }

    #[tokio::test]
    async fn test_state_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test_state.json");
        let config = StateManagerConfig::default();
        let manager = IncrementalStateManager::new(config, state_file).unwrap();

        let project_root = temp_dir.path().to_path_buf();
        let analysis_config = IncrementalAnalysisConfig::default();
        let state = manager
            .create_new_state(project_root, analysis_config)
            .await
            .unwrap();

        let checksum1 = manager.calculate_state_checksum(&state).unwrap();
        let checksum2 = manager.calculate_state_checksum(&state).unwrap();

        assert_eq!(checksum1, checksum2);
        assert!(!checksum1.is_empty());
    }

    #[test]
    fn test_environment_info() {
        let env = EnvironmentInfo {
            rust_version: "1.70.0".to_string(),
            os_version: "linux".to_string(),
            tool_version: "0.1.0".to_string(),
            cpu_arch: "x86_64".to_string(),
            captured_at: Utc::now(),
        };

        assert_eq!(env.rust_version, "1.70.0");
        assert_eq!(env.os_version, "linux");
    }

    #[test]
    fn test_cached_analysis_result() {
        let cached_result = CachedAnalysisResult {
            issues: Vec::new(),
            cached_at: Utc::now(),
            file_state_at_analysis: FileState {
                last_modified: 12345,
                content_hash: "test_hash".to_string(),
                file_size: 1024,
                dependencies: std::collections::HashSet::new(),
                dependents: std::collections::HashSet::new(),
                last_analyzed: Utc::now(),
                exists: true,
            },
            analysis_duration_ms: 100,
            is_valid: true,
        };

        assert!(cached_result.is_valid);
        assert_eq!(cached_result.analysis_duration_ms, 100);
        assert_eq!(cached_result.file_state_at_analysis.file_size, 1024);
    }
}
