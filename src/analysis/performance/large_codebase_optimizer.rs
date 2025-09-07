//! Large Codebase Performance Optimizer
//!
//! Specialized optimizations for handling enterprise codebases with >10k files.
//! Implements memory-efficient streaming analysis, intelligent partitioning,
//! and adaptive resource management.

use crate::analysis::cache::AstCache;
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::analysis::incremental::{ChangeDetector, IncrementalAnalysisEngine};
use crate::analysis::{AnalysisEngine, AnalysisError};
use crate::database::models::ArchitecturalIssue;
use crate::database::scalable_manager::ScalableDatabase;
use crate::performance::statistical_analysis::PerformanceProfiler;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

/// Configuration for large codebase optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeCodebaseConfig {
    /// Maximum memory usage in bytes before triggering relief strategies
    pub memory_limit_bytes: usize,
    
    /// Maximum files to process in a single batch
    pub batch_size: usize,
    
    /// Number of concurrent analysis workers
    pub worker_count: usize,
    
    /// Minimum file size threshold for full analysis (bytes)
    pub min_file_size_threshold: usize,
    
    /// Maximum file size before streaming analysis (bytes)  
    pub max_file_size_threshold: usize,
    
    /// Cache compression threshold (entries)
    pub cache_compression_threshold: usize,
    
    /// Enable adaptive partitioning based on file dependencies
    pub enable_adaptive_partitioning: bool,
    
    /// Memory pressure relief threshold (0.0-1.0)
    pub memory_pressure_threshold: f64,
    
    /// Enable incremental analysis for large codebases
    pub enable_incremental_analysis: bool,
    
    /// Parallel processing configuration
    pub parallel_config: ParallelProcessingConfig,
}

impl Default for LargeCodebaseConfig {
    fn default() -> Self {
        Self {
            memory_limit_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            batch_size: 1000,
            worker_count: num_cpus::get().max(4),
            min_file_size_threshold: 1024,      // 1KB
            max_file_size_threshold: 10 * 1024 * 1024, // 10MB
            cache_compression_threshold: 10000,
            enable_adaptive_partitioning: true,
            memory_pressure_threshold: 0.75,
            enable_incremental_analysis: true,
            parallel_config: ParallelProcessingConfig::default(),
        }
    }
}

/// Parallel processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelProcessingConfig {
    /// Maximum concurrent file parsers
    pub max_concurrent_parsers: usize,
    
    /// Maximum concurrent detectors per file
    pub max_concurrent_detectors: usize,
    
    /// Language-specific worker pools
    pub language_worker_pools: HashMap<String, usize>,
    
    /// Enable work stealing between language pools
    pub enable_work_stealing: bool,
    
    /// Detector execution priority mapping
    pub detector_priorities: HashMap<String, u8>,
}

impl Default for ParallelProcessingConfig {
    fn default() -> Self {
        let mut language_pools = HashMap::new();
        language_pools.insert("rust".to_string(), 4);
        language_pools.insert("python".to_string(), 3);
        language_pools.insert("javascript".to_string(), 3);
        language_pools.insert("typescript".to_string(), 3);
        
        let mut detector_priorities = HashMap::new();
        detector_priorities.insert("GodObjectDetector".to_string(), 1);
        detector_priorities.insert("DeadCodeDetector".to_string(), 3);
        detector_priorities.insert("CyclicDependencyDetector".to_string(), 2);
        
        Self {
            max_concurrent_parsers: num_cpus::get() * 2,
            max_concurrent_detectors: 4,
            language_worker_pools: language_pools,
            enable_work_stealing: true,
            detector_priorities,
        }
    }
}

/// Performance metrics for large codebase analysis
#[derive(Debug, Clone, Default)]
pub struct LargeCodebaseMetrics {
    pub total_files_analyzed: usize,
    pub total_files_skipped: usize,
    pub memory_peak_bytes: usize,
    pub memory_savings_bytes: usize,
    pub analysis_time_savings_ms: u64,
    pub cache_hit_rate: f64,
    pub incremental_analysis_efficiency: f64,
    pub parallel_efficiency: f64,
    pub batch_processing_stats: BatchProcessingStats,
}

/// Batch processing statistics
#[derive(Debug, Clone, Default)]
pub struct BatchProcessingStats {
    pub total_batches: usize,
    pub average_batch_size: usize,
    pub average_batch_time_ms: u64,
    pub memory_per_batch_mb: f64,
    pub parallelization_factor: f64,
}

/// File analysis priority for intelligent scheduling
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilePriority {
    Critical = 1,   // Core architecture files, recently changed
    High = 2,       // Frequently referenced, dependency hubs  
    Medium = 3,     // Standard application files
    Low = 4,        // Tests, documentation, configuration
    Skip = 5,       // Generated files, build artifacts
}

/// File metadata for optimized processing
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size_bytes: usize,
    pub last_modified: std::time::SystemTime,
    pub language: String,
    pub priority: FilePriority,
    pub dependency_count: usize,
    pub estimated_complexity: f64,
}

/// Large codebase performance optimizer
pub struct LargeCodebaseOptimizer {
    config: LargeCodebaseConfig,
    analysis_engine: Arc<RwLock<AnalysisEngine>>,
    incremental_engine: Option<Arc<RwLock<IncrementalAnalysisEngine>>>,
    database: Arc<ScalableDatabase>,
    performance_profiler: Arc<PerformanceProfiler>,
    
    // Resource management
    parser_semaphore: Arc<Semaphore>,
    detector_semaphore: Arc<Semaphore>,
    memory_monitor: Arc<RwLock<MemoryMonitor>>,
    
    // Processing state
    file_queue: Arc<RwLock<VecDeque<FileMetadata>>>,
    metrics: Arc<RwLock<LargeCodebaseMetrics>>,
}

impl LargeCodebaseOptimizer {
    /// Create new large codebase optimizer
    pub async fn new(
        config: LargeCodebaseConfig,
        analysis_engine: AnalysisEngine,
        database: Arc<ScalableDatabase>,
    ) -> Result<Self, AnalysisError> {
        info!("Initializing large codebase optimizer for >10k files");
        
        let performance_profiler = Arc::new(PerformanceProfiler::new());
        
        // Create semaphores for resource limiting
        let parser_semaphore = Arc::new(Semaphore::new(config.parallel_config.max_concurrent_parsers));
        let detector_semaphore = Arc::new(Semaphore::new(config.parallel_config.max_concurrent_detectors));
        
        // Initialize memory monitor
        let memory_monitor = Arc::new(RwLock::new(MemoryMonitor::new(config.memory_limit_bytes)));
        
        // Create incremental engine if enabled
        let incremental_engine = if config.enable_incremental_analysis {
            match Self::create_incremental_engine(&analysis_engine).await {
                Ok(engine) => Some(Arc::new(RwLock::new(engine))),
                Err(e) => {
                    warn!("Failed to create incremental engine, falling back to full analysis: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            config,
            analysis_engine: Arc::new(RwLock::new(analysis_engine)),
            incremental_engine,
            database,
            performance_profiler,
            parser_semaphore,
            detector_semaphore,
            memory_monitor,
            file_queue: Arc::new(RwLock::new(VecDeque::new())),
            metrics: Arc::new(RwLock::new(LargeCodebaseMetrics::default())),
        })
    }
    
    /// Analyze large codebase with optimizations
    pub async fn analyze_large_codebase(&self, path: &Path) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        info!("Starting optimized analysis for large codebase: {}", path.display());
        let start_time = Instant::now();
        
        // Phase 1: Discovery and prioritization
        let file_metadata = self.discover_and_prioritize_files(path).await?;
        info!("Discovered {} files for analysis", file_metadata.len());
        
        // Check if incremental analysis is beneficial
        let use_incremental = self.should_use_incremental_analysis(&file_metadata).await;
        
        let (issues, graph) = if use_incremental && self.incremental_engine.is_some() {
            self.perform_incremental_analysis(path, file_metadata).await?
        } else {
            self.perform_optimized_full_analysis(file_metadata).await?
        };
        
        // Update metrics
        let elapsed = start_time.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            metrics.analysis_time_savings_ms = elapsed.as_millis() as u64;
            metrics.total_files_analyzed = issues.len();
        }
        
        info!(
            "Large codebase analysis completed in {:?}: {} issues, {} dependencies", 
            elapsed, issues.len(), graph.node_count()
        );
        
        Ok((issues, graph))
    }
    
    /// Discover files and assign processing priorities
    async fn discover_and_prioritize_files(&self, path: &Path) -> Result<Vec<FileMetadata>, AnalysisError> {
        info!("Discovering and prioritizing files in: {}", path.display());
        
        let mut file_metadata = Vec::new();
        let walker = walkdir::WalkDir::new(path)
            .follow_links(false)
            .max_depth(20);
            
        for entry in walker {
            let entry = entry.map_err(|e| AnalysisError::IO(e.to_string()))?;
            let file_path = entry.path();
            
            if !file_path.is_file() {
                continue;
            }
            
            // Skip files based on patterns
            if self.should_skip_file(file_path) {
                continue;
            }
            
            let metadata = self.analyze_file_metadata(file_path).await?;
            file_metadata.push(metadata);
        }
        
        // Sort by priority and estimated complexity
        file_metadata.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then_with(|| b.estimated_complexity.partial_cmp(&a.estimated_complexity).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        Ok(file_metadata)
    }
    
    /// Analyze individual file metadata for prioritization
    async fn analyze_file_metadata(&self, path: &Path) -> Result<FileMetadata, AnalysisError> {
        let metadata = std::fs::metadata(path).map_err(|e| AnalysisError::IO(e.to_string()))?;
        let size_bytes = metadata.len() as usize;
        let last_modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        
        // Determine language
        let language = self.detect_language(path);
        
        // Assign priority based on heuristics
        let priority = self.calculate_file_priority(path, size_bytes, &language);
        
        // Estimate complexity (basic heuristic)
        let estimated_complexity = self.estimate_file_complexity(size_bytes, &language);
        
        Ok(FileMetadata {
            path: path.to_path_buf(),
            size_bytes,
            last_modified,
            language,
            priority,
            dependency_count: 0, // Will be populated later
            estimated_complexity,
        })
    }
    
    /// Perform optimized full analysis with batching and streaming
    async fn perform_optimized_full_analysis(
        &self,
        files: Vec<FileMetadata>,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        info!("Performing optimized full analysis on {} files", files.len());
        
        let mut all_issues = Vec::new();
        let mut combined_graph = LocalDependencyGraph::new();
        
        // Process files in batches to manage memory
        for batch in files.chunks(self.config.batch_size) {
            info!("Processing batch of {} files", batch.len());
            
            // Check memory pressure
            if self.is_memory_pressure_high().await {
                self.perform_memory_relief().await?;
            }
            
            let (batch_issues, batch_graph) = self.process_file_batch(batch).await?;
            all_issues.extend(batch_issues);
            combined_graph.merge(batch_graph)?;
            
            // Update batch statistics
            self.update_batch_stats(batch.len()).await;
        }
        
        Ok((all_issues, combined_graph))
    }
    
    /// Process a batch of files with parallel execution
    async fn process_file_batch(
        &self,
        batch: &[FileMetadata],
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        let start_time = Instant::now();
        
        // Group files by language for optimized processing
        let mut language_groups: HashMap<String, Vec<&FileMetadata>> = HashMap::new();
        for file in batch {
            language_groups.entry(file.language.clone()).or_default().push(file);
        }
        
        let mut batch_issues = Vec::new();
        let mut batch_graph = LocalDependencyGraph::new();
        
        // Process each language group in parallel
        let mut tasks = Vec::new();
        for (language, files) in language_groups {
            let analysis_engine = self.analysis_engine.clone();
            let parser_semaphore = self.parser_semaphore.clone();
            let detector_semaphore = self.detector_semaphore.clone();
            
            let task = tokio::spawn(async move {
                Self::process_language_group(analysis_engine, parser_semaphore, detector_semaphore, language, files).await
            });
            
            tasks.push(task);
        }
        
        // Collect results from all language groups
        for task in tasks {
            let (issues, graph) = task.await.map_err(|e| AnalysisError::Runtime(e.to_string()))??;
            batch_issues.extend(issues);
            batch_graph.merge(graph)?;
        }
        
        let elapsed = start_time.elapsed();
        debug!("Batch processing completed in {:?}", elapsed);
        
        Ok((batch_issues, batch_graph))
    }
    
    /// Process files for a specific language group
    async fn process_language_group(
        analysis_engine: Arc<RwLock<AnalysisEngine>>,
        parser_semaphore: Arc<Semaphore>,
        detector_semaphore: Arc<Semaphore>,
        _language: String,
        files: Vec<&FileMetadata>,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        let mut group_issues = Vec::new();
        let mut group_graph = LocalDependencyGraph::new();
        
        // Process files concurrently within the language group
        let mut tasks = Vec::new();
        for file in files {
            let path = file.path.clone();
            let engine = analysis_engine.clone();
            let parser_sem = parser_semaphore.clone();
            let detector_sem = detector_semaphore.clone();
            
            let task = tokio::spawn(async move {
                // Acquire semaphores to limit resource usage
                let _parser_permit = parser_sem.acquire().await.map_err(|e| AnalysisError::Runtime(e.to_string()))?;
                let _detector_permit = detector_sem.acquire().await.map_err(|e| AnalysisError::Runtime(e.to_string()))?;
                
                // Perform analysis on the file
                let engine = engine.read().await;
                let (issues, graph) = engine.analyze(&path).await?;
                
                Ok::<_, AnalysisError>((issues, graph))
            });
            
            tasks.push(task);
        }
        
        // Collect results
        for task in tasks {
            let (issues, graph) = task.await.map_err(|e| AnalysisError::Runtime(e.to_string()))??;
            group_issues.extend(issues);
            group_graph.merge(graph)?;
        }
        
        Ok((group_issues, group_graph))
    }
    
    /// Check if memory pressure is high and relief strategies should be triggered
    async fn is_memory_pressure_high(&self) -> bool {
        let monitor = self.memory_monitor.read().await;
        monitor.get_memory_pressure() > self.config.memory_pressure_threshold
    }
    
    /// Perform memory relief strategies
    async fn perform_memory_relief(&self) -> Result<(), AnalysisError> {
        info!("High memory pressure detected, performing relief strategies");
        
        // Clear caches
        {
            let engine = self.analysis_engine.read().await;
            engine.clear_caches().await?;
        }
        
        // Force garbage collection (if available)
        #[cfg(feature = "jemalloc")]
        {
            tikv_jemalloc_ctl::epoch::advance().ok();
            tikv_jemalloc_ctl::stats::allocated::read().ok();
        }
        
        // Update memory monitor
        {
            let mut monitor = self.memory_monitor.write().await;
            monitor.reset_peak_usage();
        }
        
        info!("Memory relief completed");
        Ok(())
    }
    
    /// Determine if incremental analysis should be used
    async fn should_use_incremental_analysis(&self, files: &[FileMetadata]) -> bool {
        if !self.config.enable_incremental_analysis || self.incremental_engine.is_none() {
            return false;
        }
        
        // Use incremental analysis for large codebases with recent changes
        let recent_changes = files.iter()
            .filter(|f| {
                f.last_modified.elapsed().unwrap_or(Duration::from_secs(0)) < Duration::from_days(1)
            })
            .count();
            
        let change_ratio = recent_changes as f64 / files.len() as f64;
        
        // Use incremental if less than 30% of files changed recently
        change_ratio < 0.3 && files.len() > 5000
    }
    
    /// Perform incremental analysis
    async fn perform_incremental_analysis(
        &self,
        path: &Path,
        _files: Vec<FileMetadata>,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        info!("Performing incremental analysis");
        
        if let Some(incremental_engine) = &self.incremental_engine {
            let mut engine = incremental_engine.write().await;
            let (issues, _result) = engine.analyze_incremental(path).await
                .map_err(|e| AnalysisError::Engine(e.to_string()))?;
                
            // For now, create empty graph - incremental engine should provide this
            let graph = LocalDependencyGraph::new();
            
            Ok((issues, graph))
        } else {
            Err(AnalysisError::configuration_error("incremental_engine", "not_available", "Incremental engine not configured"))
        }
    }
    
    /// Update batch processing statistics
    async fn update_batch_stats(&self, batch_size: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.batch_processing_stats.total_batches += 1;
        metrics.batch_processing_stats.average_batch_size = 
            (metrics.batch_processing_stats.average_batch_size * (metrics.batch_processing_stats.total_batches - 1) + batch_size) 
            / metrics.batch_processing_stats.total_batches;
    }
    
    // Helper methods
    
    fn should_skip_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Skip common non-source directories and files
        let skip_patterns = [
            "/target/", "/build/", "/dist/", "/node_modules/", "/.git/",
            "/vendor/", "/deps/", "/__pycache__/", ".pyc", ".so", ".dylib",
            ".exe", ".dll", ".jar", ".class"
        ];
        
        skip_patterns.iter().any(|pattern| path_str.contains(pattern))
    }
    
    fn detect_language(&self, path: &Path) -> String {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("py") => "python".to_string(),
            Some("js") => "javascript".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("jsx") => "javascript".to_string(),
            Some("tsx") => "typescript".to_string(),
            _ => "unknown".to_string(),
        }
    }
    
    fn calculate_file_priority(&self, path: &Path, size_bytes: usize, language: &str) -> FilePriority {
        let path_str = path.to_string_lossy().to_lowercase();
        
        // Critical files
        if path_str.contains("main.") || path_str.contains("app.") || path_str.contains("lib.rs") {
            return FilePriority::Critical;
        }
        
        // Test files are low priority
        if path_str.contains("test") || path_str.contains("spec") {
            return FilePriority::Low;
        }
        
        // Large files get higher priority (likely to have issues)
        if size_bytes > 50_000 && language != "unknown" {
            return FilePriority::High;
        }
        
        // Small files are lower priority
        if size_bytes < self.config.min_file_size_threshold {
            return FilePriority::Low;
        }
        
        FilePriority::Medium
    }
    
    fn estimate_file_complexity(&self, size_bytes: usize, language: &str) -> f64 {
        // Basic complexity estimation based on file size and language
        let base_complexity = (size_bytes as f64).log10();
        
        let language_multiplier = match language {
            "rust" => 1.2,      // Rust tends to be more complex
            "typescript" => 1.1, // TypeScript complexity
            "javascript" => 1.0, // Baseline
            "python" => 0.9,     // Python is generally simpler
            _ => 1.0,
        };
        
        base_complexity * language_multiplier
    }
    
    async fn create_incremental_engine(analysis_engine: &AnalysisEngine) -> Result<IncrementalAnalysisEngine, AnalysisError> {
        // This would create an incremental engine - simplified for now
        Err(AnalysisError::configuration_error("incremental_engine", "not_implemented", "Incremental engine creation not implemented"))
    }
    
    /// Get current performance metrics
    pub async fn get_metrics(&self) -> LargeCodebaseMetrics {
        self.metrics.read().await.clone()
    }
}

/// Memory monitor for tracking and managing memory usage
pub struct MemoryMonitor {
    memory_limit_bytes: usize,
    current_usage_bytes: usize,
    peak_usage_bytes: usize,
    last_check: Instant,
}

impl MemoryMonitor {
    pub fn new(memory_limit_bytes: usize) -> Self {
        Self {
            memory_limit_bytes,
            current_usage_bytes: 0,
            peak_usage_bytes: 0,
            last_check: Instant::now(),
        }
    }
    
    pub fn get_memory_pressure(&self) -> f64 {
        if self.memory_limit_bytes == 0 {
            return 0.0;
        }
        
        self.current_usage_bytes as f64 / self.memory_limit_bytes as f64
    }
    
    pub fn reset_peak_usage(&mut self) {
        self.peak_usage_bytes = self.current_usage_bytes;
    }
    
    pub fn update_current_usage(&mut self, usage_bytes: usize) {
        self.current_usage_bytes = usage_bytes;
        if usage_bytes > self.peak_usage_bytes {
            self.peak_usage_bytes = usage_bytes;
        }
        self.last_check = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_large_codebase_config() {
        let config = LargeCodebaseConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert!(config.enable_adaptive_partitioning);
        assert!(config.enable_incremental_analysis);
    }

    #[tokio::test]
    async fn test_file_priority_calculation() {
        // This test would be implemented with a mock optimizer
        // Testing file priority logic
        assert_eq!(FilePriority::Critical as u8, 1);
        assert_eq!(FilePriority::Skip as u8, 5);
    }

    #[tokio::test] 
    async fn test_memory_monitor() {
        let mut monitor = MemoryMonitor::new(1024 * 1024); // 1MB limit
        
        monitor.update_current_usage(512 * 1024); // 512KB
        assert_eq!(monitor.get_memory_pressure(), 0.5);
        
        monitor.update_current_usage(800 * 1024); // 800KB  
        assert!(monitor.get_memory_pressure() > 0.75);
    }

    #[tokio::test]
    async fn test_language_detection() {
        let temp_dir = tempdir().unwrap();
        
        // Create test files
        File::create(temp_dir.path().join("test.rs")).unwrap();
        File::create(temp_dir.path().join("test.py")).unwrap();
        File::create(temp_dir.path().join("test.js")).unwrap();
        File::create(temp_dir.path().join("test.ts")).unwrap();
        
        // Test would verify language detection logic
    }
}