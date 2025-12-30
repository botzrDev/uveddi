//! Incremental Analysis Engine for UV-91 Phase 2
//!
//! Orchestrates intelligent incremental analysis to achieve 50%+ reduction in re-analysis time.
//! Integrates change detection, dependency tracking, and state management for enterprise codebases.
//!
//! Key Features:
//! - Intelligent change-driven analysis
//! - Dependency-aware impact propagation
//! - Performance-optimized execution
//! - Graceful fallback to full analysis

use super::dependency_tracker::DependencyExtractionConfig;
use super::state_manager::StateManagerConfig;
use super::{
    CacheHitRates, ChangeDetectionConfig, ChangeDetector, ChangeSet, ChangeSummary,
    DependencyTracker, IncrementalAnalysisConfig, IncrementalAnalysisError,
    IncrementalAnalysisResult, IncrementalPerformanceMetrics, IncrementalState,
    IncrementalStateManager, Result,
};
use crate::analysis::diagram_cache::DiagramCacheEngine;
use crate::analysis::AnalysisEngine;
use crate::core::logging::{debug, info, warn};
use crate::database::models::ArchitecturalIssue;
use chrono::Utc;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Configuration for incremental analysis engine
#[derive(Debug, Clone)]
/// Configuration options for incremental.
///
/// Provides sensible defaults that can be overridden via builder methods
/// or loaded from configuration files.
pub struct IncrementalConfig {
    /// Main incremental analysis configuration
    pub analysis_config: IncrementalAnalysisConfig,

    /// State management configuration  
    pub state_config: StateManagerConfig,

    /// Dependency extraction configuration
    pub dependency_config: DependencyExtractionConfig,

    /// Change detection configuration
    pub change_config: ChangeDetectionConfig,
}

/// Main incremental analysis engine
pub struct IncrementalAnalysisEngine {
    /// Core analysis engine for actual analysis work
    analysis_engine: AnalysisEngine,

    /// Change detector for file modification tracking
    change_detector: ChangeDetector,

    /// Dependency tracker for impact analysis
    dependency_tracker: DependencyTracker,

    /// State manager for persistence
    state_manager: IncrementalStateManager,

    /// Configuration
    config: IncrementalConfig,

    /// Performance metrics
    performance_metrics: IncrementalPerformanceMetrics,

    /// Statistics for monitoring
    stats: EngineStats,

    /// Diagram cache engine for Phase 3 integration
    diagram_cache: Option<std::sync::Arc<tokio::sync::RwLock<DiagramCacheEngine>>>,
}

/// Statistics for the incremental analysis engine
#[derive(Debug, Clone, Default)]
/// Data structure for enginestats.
pub struct EngineStats {
    pub total_analyses: u64,
    pub incremental_analyses: u64,
    pub full_analyses: u64,
    pub average_time_savings_percent: f64,
    pub total_files_processed: u64,
    pub total_files_skipped: u64,
    pub cache_hit_rate: f64,
    pub dependency_accuracy: f64,
}

impl IncrementalAnalysisEngine {
    /// Creates a new incremental analysis engine
    pub async fn new(
        analysis_engine: AnalysisEngine,
        config: IncrementalConfig,
        state_file_path: PathBuf,
    ) -> Result<Self> {
        info!("Initializing incremental analysis engine");

        // Initialize components
        let change_detector = ChangeDetector::new(config.change_config.clone())?;
        let dependency_tracker = DependencyTracker::new(config.dependency_config.clone())?;
        let state_manager =
            IncrementalStateManager::new(config.state_config.clone(), state_file_path)?;

        let engine = Self {
            analysis_engine,
            change_detector,
            dependency_tracker,
            state_manager,
            config,
            performance_metrics: IncrementalPerformanceMetrics::default(),
            stats: EngineStats::default(),
            diagram_cache: None,
        };

        info!("Incremental analysis engine initialized successfully");
        Ok(engine)
    }

    /// Sets the diagram cache engine for Phase 3 integration
    pub fn set_diagram_cache(
        &mut self,
        diagram_cache: std::sync::Arc<tokio::sync::RwLock<DiagramCacheEngine>>,
    ) {
        self.diagram_cache = Some(diagram_cache);
        info!("Diagram cache engine integrated with incremental analysis");
    }

    /// Performs incremental analysis on the given path
    pub async fn analyze_incremental(
        &mut self,
        path: &Path,
    ) -> Result<(Vec<ArchitecturalIssue>, IncrementalAnalysisResult)> {
        info!("Starting incremental analysis for: {}", path.display());
        let overall_start = Instant::now();

        // Step 1: Load or create incremental state
        let mut state = self.load_or_create_state(path).await?;

        // Step 2: Detect changes
        let changeset = self.detect_and_analyze_changes(path, &state).await?;

        // Step 3: Decide between incremental and full analysis
        let should_use_incremental = self
            .should_use_incremental_analysis(&changeset, &state)
            .await;

        let (all_issues, analysis_result) = if should_use_incremental {
            info!("Performing incremental analysis");
            self.perform_incremental_analysis(path, changeset, &mut state)
                .await?
        } else {
            info!("Performing full analysis (fallback)");
            self.perform_full_analysis(path, &mut state).await?
        };

        // Step 4: Update and save state
        self.update_and_save_state(&state).await?;

        // Step 5: Update statistics
        self.update_engine_statistics(&analysis_result);

        let total_elapsed = overall_start.elapsed();
        info!(
            "Incremental analysis completed in {:?}: {} issues found, {:.1}% time savings",
            total_elapsed,
            all_issues.len(),
            if analysis_result.was_incremental {
                (analysis_result.time_saved_ms as f64 / total_elapsed.as_millis() as f64) * 100.0
            } else {
                0.0
            }
        );

        Ok((all_issues, analysis_result))
    }

    /// Loads existing state or creates new state for the project
    async fn load_or_create_state(&mut self, path: &Path) -> Result<IncrementalState> {
        let start_time = Instant::now();

        // First check if we already have state in memory (for reused engines)
        if let Some(state) = self.state_manager.get_current_state().await {
            info!("Using existing in-memory incremental state");

            let elapsed = start_time.elapsed();
            self.performance_metrics.state_persistence_time_ms = elapsed.as_millis() as u64;

            return Ok(state);
        }

        // Try to load existing state from disk
        if let Some(state) = self.state_manager.load_state().await? {
            info!("Loaded existing incremental state from disk");

            // Initialize components with loaded state
            self.change_detector
                .load_file_states(state.file_states.clone());

            let elapsed = start_time.elapsed();
            self.performance_metrics.state_persistence_time_ms = elapsed.as_millis() as u64;

            return Ok(state);
        }

        // Create new state if none exists
        info!("Creating new incremental state");
        let project_root = if path.is_file() {
            path.parent().unwrap_or(path).to_path_buf()
        } else {
            path.to_path_buf()
        };

        let state = self
            .state_manager
            .create_new_state(project_root, self.config.analysis_config.clone())
            .await?;

        let elapsed = start_time.elapsed();
        self.performance_metrics.state_persistence_time_ms = elapsed.as_millis() as u64;

        Ok(state)
    }

    /// Detects changes and analyzes their impact
    async fn detect_and_analyze_changes(
        &mut self,
        path: &Path,
        _state: &IncrementalState,
    ) -> Result<ChangeSet> {
        let start_time = Instant::now();
        info!("Detecting changes and analyzing impact");

        // Detect file changes
        let mut changeset = self.change_detector.detect_changes(path).await?;

        // Update file states for new files
        self.change_detector
            .update_file_states_for_new_files(&changeset.added)
            .await?;

        // Analyze dependency impact if enabled
        if self.config.analysis_config.enable_dependency_propagation && changeset.has_changes() {
            let impact = self.dependency_tracker.analyze_change_impact(&changeset)?;

            // Add transitively affected files to the changeset
            changeset.affected_by_dependencies = impact.transitively_affected.clone();

            info!(
                "Dependency impact analysis: {:.2}% of files affected",
                impact.impact_score * 100.0
            );
        }

        let elapsed = start_time.elapsed();
        self.performance_metrics.change_detection_time_ms = elapsed.as_millis() as u64;

        info!(
            "Change detection completed in {:?}: {} modified, {} added, {} deleted, {} affected by dependencies",
            elapsed,
            changeset.modified.len(),
            changeset.added.len(),
            changeset.deleted.len(),
            changeset.affected_by_dependencies.len()
        );

        Ok(changeset)
    }

    /// Determines whether to use incremental analysis or fall back to full analysis
    async fn should_use_incremental_analysis(
        &self,
        changeset: &ChangeSet,
        state: &IncrementalState,
    ) -> bool {
        // Always use full analysis if incremental is disabled
        if !self.config.analysis_config.enabled {
            debug!("Incremental analysis disabled in configuration");
            return false;
        }

        // Use full analysis if no existing state
        if state.file_states.is_empty() {
            debug!("No existing file states, using full analysis");
            return false;
        }

        // Use full analysis if no changes detected
        if !changeset.has_changes() {
            debug!("No changes detected, but using incremental for cache validation");
            return true;
        }

        // Calculate change percentage
        let total_files = state.file_states.len();
        let change_percentage = changeset.change_percentage(total_files);

        // Use full analysis if change percentage exceeds threshold
        if change_percentage > self.config.analysis_config.full_analysis_threshold * 100.0 {
            info!(
                "Change percentage ({:.1}%) exceeds threshold ({:.1}%), using full analysis",
                change_percentage,
                self.config.analysis_config.full_analysis_threshold * 100.0
            );
            return false;
        }

        // Check for dependency graph integrity
        if self.config.analysis_config.enable_dependency_propagation {
            let dependency_coverage = self.calculate_dependency_coverage(changeset, state);
            if dependency_coverage < 0.9 {
                // 90% coverage threshold
                info!(
                    "Dependency coverage ({:.1}%) below threshold, using full analysis",
                    dependency_coverage * 100.0
                );
                return false;
            }
        }

        debug!("Conditions met for incremental analysis");
        true
    }

    /// Calculates dependency coverage for incremental analysis reliability
    fn calculate_dependency_coverage(
        &self,
        changeset: &ChangeSet,
        state: &IncrementalState,
    ) -> f64 {
        let all_affected_files = changeset.all_affected_files();
        let covered_files: HashSet<_> = all_affected_files
            .iter()
            .filter(|file| state.dependency_graph.get_dependencies(file).is_some())
            .collect();

        if all_affected_files.is_empty() {
            1.0
        } else {
            covered_files.len() as f64 / all_affected_files.len() as f64
        }
    }

    /// Performs incremental analysis on changed files only
    async fn perform_incremental_analysis(
        &mut self,
        _path: &Path,
        changeset: ChangeSet,
        state: &mut IncrementalState,
    ) -> Result<(Vec<ArchitecturalIssue>, IncrementalAnalysisResult)> {
        let start_time = Instant::now();
        let reanalysis_start = Instant::now();

        // Determine files that need re-analysis
        let files_to_analyze = self.determine_files_to_reanalyze(&changeset, state).await?;

        info!(
            "Re-analyzing {} files out of {} total",
            files_to_analyze.len(),
            state.file_states.len()
        );

        // Invalidate cache for changed files
        self.state_manager
            .invalidate_cache_for_files(&files_to_analyze)
            .await?;

        // Perform analysis on selected files
        let mut new_issues = Vec::new();
        let mut analysis_results = Vec::new();

        for file_path in files_to_analyze.iter() {
            let result = self.analyze_single_file(file_path).await;
            analysis_results.push(result);
        }

        for result in analysis_results {
            match result {
                Ok((file_path, issues, duration_ms)) => {
                    new_issues.extend(issues.clone());

                    // Update cache with new results
                    self.state_manager
                        .update_analysis_cache(file_path, issues, duration_ms)
                        .await?;
                }
                Err(e) => {
                    warn!("Failed to analyze file: {}", e);
                }
            }
        }

        let reanalysis_elapsed = reanalysis_start.elapsed();

        // Collect existing results from cache for unchanged files
        let cached_issues = self.collect_cached_results(&changeset, state).await?;
        new_issues.extend(cached_issues);

        // Update dependency graph
        self.update_dependency_graph(&changeset).await?;

        // Calculate performance metrics
        let total_files = state.file_states.len();
        let files_reanalyzed = files_to_analyze.len();
        let files_skipped = total_files - files_reanalyzed;

        // Estimate time saved (compared to full analysis)
        let estimated_full_analysis_time = self.estimate_full_analysis_time(total_files);
        let actual_analysis_time = reanalysis_elapsed.as_millis() as u64;
        let time_saved = estimated_full_analysis_time.saturating_sub(actual_analysis_time);

        // Create analysis result
        let analysis_result = IncrementalAnalysisResult {
            was_incremental: true,
            files_reanalyzed,
            total_files,
            time_saved_ms: time_saved,
            analysis_time: Utc::now(),
            change_summary: self.create_change_summary(&changeset, total_files),
            performance_metrics: IncrementalPerformanceMetrics {
                change_detection_time_ms: self.performance_metrics.change_detection_time_ms,
                dependency_analysis_time_ms: self.performance_metrics.dependency_analysis_time_ms,
                reanalysis_time_ms: actual_analysis_time,
                state_persistence_time_ms: self.performance_metrics.state_persistence_time_ms,
                peak_memory_usage_mb: self.get_peak_memory_usage(),
                cache_hit_rates: self.calculate_cache_hit_rates(files_skipped, total_files),
            },
        };

        let total_elapsed = start_time.elapsed();
        info!(
            "Incremental analysis completed in {:?}: {:.1}% time savings",
            total_elapsed,
            if estimated_full_analysis_time > 0 {
                (time_saved as f64 / estimated_full_analysis_time as f64) * 100.0
            } else {
                0.0
            }
        );

        Ok((new_issues, analysis_result))
    }

    /// Performs full analysis as fallback
    async fn perform_full_analysis(
        &mut self,
        path: &Path,
        state: &mut IncrementalState,
    ) -> Result<(Vec<ArchitecturalIssue>, IncrementalAnalysisResult)> {
        let start_time = Instant::now();
        info!("Performing full analysis");

        // Clear existing state and rebuild
        state.file_states.clear();
        state.analysis_cache.clear();

        // Run full analysis using the core engine
        let (issues, _dependency_graph) = self
            .analysis_engine
            .analyze(path)
            .await
            .map_err(|e| IncrementalAnalysisError::AnalysisEngineError(e.into()))?;

        // Update state with new results
        let discovered_files = if path.is_file() {
            vec![path.to_path_buf()]
        } else {
            self.discover_all_files(path).await?
        };

        // Build file states for all discovered files
        for file_path in &discovered_files {
            self.change_detector
                .update_file_states_for_new_files(&vec![file_path.clone()].into_iter().collect())
                .await?;
        }

        // Rebuild dependency graph
        let files_set: HashSet<PathBuf> = discovered_files.into_iter().collect();
        self.dependency_tracker
            .build_dependency_graph(&files_set)
            .await?;

        // Create analysis result
        let total_elapsed = start_time.elapsed();
        let total_files = files_set.len();

        let analysis_result = IncrementalAnalysisResult {
            was_incremental: false,
            files_reanalyzed: total_files,
            total_files,
            time_saved_ms: 0,
            analysis_time: Utc::now(),
            change_summary: ChangeSummary {
                changed_files: Vec::new(),
                affected_files: Vec::new(),
                new_files: files_set.iter().cloned().collect(),
                deleted_files: Vec::new(),
                change_percentage: 100.0, // Full analysis
            },
            performance_metrics: IncrementalPerformanceMetrics {
                change_detection_time_ms: 0,
                dependency_analysis_time_ms: 0,
                reanalysis_time_ms: total_elapsed.as_millis() as u64,
                state_persistence_time_ms: 0,
                peak_memory_usage_mb: self.get_peak_memory_usage(),
                cache_hit_rates: CacheHitRates {
                    file_state_cache: 0.0,
                    dependency_cache: 0.0,
                    analysis_result_cache: 0.0,
                    ast_cache: 0.0,
                },
            },
        };

        info!(
            "Full analysis completed in {:?}: {} issues found",
            total_elapsed,
            issues.len()
        );

        Ok((issues, analysis_result))
    }

    /// Determines which files need re-analysis based on changes and dependencies
    async fn determine_files_to_reanalyze(
        &self,
        changeset: &ChangeSet,
        state: &IncrementalState,
    ) -> Result<HashSet<PathBuf>> {
        let mut files_to_analyze = HashSet::new();

        // Always re-analyze directly changed files
        files_to_analyze.extend(changeset.modified.iter().cloned());
        files_to_analyze.extend(changeset.added.iter().cloned());

        // Add files affected by dependency changes
        files_to_analyze.extend(changeset.affected_by_dependencies.iter().cloned());

        // Remove deleted files
        for deleted_file in &changeset.deleted {
            files_to_analyze.remove(deleted_file);
        }

        // Validate cache entries and mark invalid ones for re-analysis
        for (file_path, file_state) in &state.file_states {
            if let Some(cached_result) = state.analysis_cache.get(file_path) {
                if !cached_result.is_valid || cached_result.file_state_at_analysis != *file_state {
                    files_to_analyze.insert(file_path.clone());
                }
            } else {
                // No cached result exists, need to analyze
                files_to_analyze.insert(file_path.clone());
            }
        }

        debug!(
            "Determined {} files need re-analysis",
            files_to_analyze.len()
        );
        Ok(files_to_analyze)
    }

    /// Analyzes a single file and returns results
    async fn analyze_single_file(
        &mut self,
        file_path: &PathBuf,
    ) -> Result<(PathBuf, Vec<ArchitecturalIssue>, u64)> {
        let start_time = Instant::now();

        // Use the core analysis engine for single file analysis
        let (issues, _) = self
            .analysis_engine
            .analyze(file_path)
            .await
            .map_err(|e| IncrementalAnalysisError::AnalysisEngineError(e.into()))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        debug!(
            "Analyzed {} in {:?}: {} issues",
            file_path.display(),
            start_time.elapsed(),
            issues.len()
        );

        Ok((file_path.clone(), issues, duration_ms))
    }

    /// Collects cached analysis results for unchanged files
    async fn collect_cached_results(
        &self,
        changeset: &ChangeSet,
        state: &IncrementalState,
    ) -> Result<Vec<ArchitecturalIssue>> {
        let changed_files = changeset.all_affected_files();
        let mut cached_issues = Vec::new();

        for (file_path, _) in &state.file_states {
            if !changed_files.contains(file_path) {
                if let Some(cached_result) = state.analysis_cache.get(file_path) {
                    if cached_result.is_valid {
                        cached_issues.extend(cached_result.issues.clone());
                    }
                }
            }
        }

        debug!("Collected {} cached analysis results", cached_issues.len());
        Ok(cached_issues)
    }

    /// Updates the dependency graph with changes
    async fn update_dependency_graph(&mut self, changeset: &ChangeSet) -> Result<()> {
        let start_time = Instant::now();

        self.dependency_tracker.update_dependencies(changeset)?;

        let elapsed = start_time.elapsed();
        self.performance_metrics.dependency_analysis_time_ms = elapsed.as_millis() as u64;

        debug!("Updated dependency graph in {:?}", elapsed);
        Ok(())
    }

    /// Updates and saves the incremental state
    async fn update_and_save_state(&mut self, _state: &IncrementalState) -> Result<()> {
        let start_time = Instant::now();

        // Update state with current detector and tracker data
        self.state_manager
            .update_state_with_change_detector(&self.change_detector)
            .await?;
        self.state_manager
            .update_state_with_dependency_tracker(&self.dependency_tracker)
            .await?;

        // Save the updated state
        if let Some(current_state) = self.state_manager.get_current_state().await {
            self.state_manager.save_state(&current_state).await?;
        }

        let elapsed = start_time.elapsed();
        self.performance_metrics.state_persistence_time_ms += elapsed.as_millis() as u64;

        debug!("Updated and saved state in {:?}", elapsed);
        Ok(())
    }

    /// Discovers all files in the given path
    async fn discover_all_files(&mut self, path: &Path) -> Result<Vec<PathBuf>> {
        // Use the change detector to discover files
        let changeset = self.change_detector.detect_changes(path).await?;
        let mut all_files = Vec::new();

        all_files.extend(changeset.modified);
        all_files.extend(changeset.added);

        Ok(all_files)
    }

    /// Estimates how long a full analysis would take
    fn estimate_full_analysis_time(&self, total_files: usize) -> u64 {
        // Use historical data or heuristics to estimate
        // For now, use a simple heuristic: 50ms per file
        (total_files as u64) * 50
    }

    /// Gets current peak memory usage
    fn get_peak_memory_usage(&self) -> f64 {
        // In a real implementation, you would track actual memory usage
        // For now, return a placeholder value
        0.0
    }

    /// Calculates cache hit rates
    fn calculate_cache_hit_rates(&self, files_skipped: usize, total_files: usize) -> CacheHitRates {
        let analysis_cache_hit_rate = if total_files > 0 {
            files_skipped as f64 / total_files as f64
        } else {
            0.0
        };

        CacheHitRates {
            file_state_cache: analysis_cache_hit_rate,
            dependency_cache: analysis_cache_hit_rate,
            analysis_result_cache: analysis_cache_hit_rate,
            ast_cache: 0.0, // Would be calculated from actual AST cache
        }
    }

    /// Creates change summary from changeset
    fn create_change_summary(&self, changeset: &ChangeSet, total_files: usize) -> ChangeSummary {
        ChangeSummary {
            changed_files: changeset.modified.iter().cloned().collect(),
            affected_files: changeset.affected_by_dependencies.iter().cloned().collect(),
            new_files: changeset.added.iter().cloned().collect(),
            deleted_files: changeset.deleted.iter().cloned().collect(),
            change_percentage: changeset.change_percentage(total_files),
        }
    }

    /// Updates engine statistics
    fn update_engine_statistics(&mut self, result: &IncrementalAnalysisResult) {
        self.stats.total_analyses += 1;

        if result.was_incremental {
            self.stats.incremental_analyses += 1;
        } else {
            self.stats.full_analyses += 1;
        }

        self.stats.total_files_processed += result.files_reanalyzed as u64;
        self.stats.total_files_skipped += (result.total_files - result.files_reanalyzed) as u64;

        // Update running averages
        if result.was_incremental && result.time_saved_ms > 0 {
            let time_savings_percent = (result.time_saved_ms as f64
                / (result.time_saved_ms + result.performance_metrics.reanalysis_time_ms) as f64)
                * 100.0;

            self.stats.average_time_savings_percent = (self.stats.average_time_savings_percent
                * (self.stats.incremental_analyses - 1) as f64
                + time_savings_percent)
                / self.stats.incremental_analyses as f64;
        }

        self.stats.cache_hit_rate = result
            .performance_metrics
            .cache_hit_rates
            .analysis_result_cache;
    }

    /// Returns engine statistics
    pub fn get_stats(&self) -> &EngineStats {
        &self.stats
    }

    /// Returns current performance metrics
    pub fn get_performance_metrics(&self) -> &IncrementalPerformanceMetrics {
        &self.performance_metrics
    }

    /// Forces a full analysis on the next run
    pub async fn force_full_analysis(&mut self) -> Result<()> {
        info!("Forcing full analysis on next run");

        // Clear the state to force full analysis
        if let Some(mut state) = self.state_manager.get_current_state().await {
            state.file_states.clear();
            state.analysis_cache.clear();
            self.state_manager.save_state(&state).await?;
        }

        Ok(())
    }

    /// Validates the current incremental state
    pub async fn validate_state(&self) -> Result<bool> {
        if let Some(state) = self.state_manager.get_current_state().await {
            // Check state consistency
            let file_count = state.file_states.len();
            let cache_count = state.analysis_cache.len();
            let dependency_count = state.dependency_graph.total_files();

            if file_count == 0 {
                warn!("No file states in incremental state");
                return Ok(false);
            }

            // Validate cache consistency
            let invalid_cache_entries = state
                .analysis_cache
                .iter()
                .filter(|(file_path, cached_result)| {
                    if let Some(file_state) = state.file_states.get(*file_path) {
                        cached_result.file_state_at_analysis != *file_state
                    } else {
                        true // File not in state
                    }
                })
                .count();

            if invalid_cache_entries > cache_count / 2 {
                warn!("More than 50% of cache entries are invalid");
                return Ok(false);
            }

            info!(
                "State validation passed: {} files, {} cached results, {} dependencies",
                file_count, cache_count, dependency_count
            );
            Ok(true)
        } else {
            warn!("No incremental state available for validation");
            Ok(false)
        }
    }
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            analysis_config: IncrementalAnalysisConfig::default(),
            state_config: StateManagerConfig::default(),
            dependency_config: DependencyExtractionConfig::default(),
            change_config: ChangeDetectionConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::AnalysisEngine;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_incremental_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("state.json");
        let config = IncrementalConfig::default();
        let analysis_engine = AnalysisEngine::new().unwrap();

        let engine = IncrementalAnalysisEngine::new(analysis_engine, config, state_file)
            .await
            .unwrap();

        assert_eq!(engine.stats.total_analyses, 0);
        assert_eq!(engine.stats.incremental_analyses, 0);
    }

    #[test]
    fn test_engine_stats() {
        let stats = EngineStats::default();
        assert_eq!(stats.total_analyses, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
        assert_eq!(stats.average_time_savings_percent, 0.0);
    }

    #[test]
    fn test_incremental_config() {
        let config = IncrementalConfig::default();
        assert!(config.analysis_config.enabled);
        assert!(config.analysis_config.enable_dependency_propagation);
        assert_eq!(config.analysis_config.full_analysis_threshold, 0.30);
    }
}
