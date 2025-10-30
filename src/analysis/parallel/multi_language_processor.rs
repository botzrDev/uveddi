//! Multi-Language Parallel Processing Engine
//!
//! Advanced parallel processing system optimized for multi-language codebases
//! with work stealing, adaptive load balancing, and language-specific optimizations.

use crate::analysis::{AnalysisDetector, AnalysisEngine, AnalysisError};
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::ast::{SourceLanguage, ParsedFile};
use crate::database::models::ArchitecturalIssue;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn, error};

/// Configuration for multi-language parallel processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelProcessingConfig {
    /// Maximum number of concurrent workers
    pub max_workers: usize,
    
    /// Language-specific worker pool configurations
    pub language_pools: HashMap<String, LanguagePoolConfig>,
    
    /// Work stealing configuration
    pub work_stealing: WorkStealingConfig,
    
    /// Load balancing strategy
    pub load_balancing: LoadBalancingConfig,
    
    /// Task scheduling configuration
    pub scheduling: TaskSchedulingConfig,
    
    /// Resource management
    pub resource_management: ResourceManagementConfig,
    
    /// Performance monitoring
    pub monitoring: ParallelProcessingMonitoringConfig,
}

impl Default for ParallelProcessingConfig {
    fn default() -> Self {
        let num_cpus = num_cpus::get();
        let mut language_pools = HashMap::new();
        
        // Configure language-specific pools
        language_pools.insert("rust".to_string(), LanguagePoolConfig {
            min_workers: 2,
            max_workers: num_cpus / 2,
            priority: LanguagePriority::High,
            memory_per_worker_mb: 256,
            specialized_detectors: vec!["GodObjectDetector".to_string(), "DeadCodeDetector".to_string()],
        });
        
        language_pools.insert("python".to_string(), LanguagePoolConfig {
            min_workers: 1,
            max_workers: num_cpus / 3,
            priority: LanguagePriority::Medium,
            memory_per_worker_mb: 128,
            specialized_detectors: vec!["ComplexityDetector".to_string()],
        });
        
        language_pools.insert("javascript".to_string(), LanguagePoolConfig {
            min_workers: 1,
            max_workers: num_cpus / 3,
            priority: LanguagePriority::Medium,
            memory_per_worker_mb: 128,
            specialized_detectors: vec!["CallbackHellDetector".to_string()],
        });
        
        language_pools.insert("typescript".to_string(), LanguagePoolConfig {
            min_workers: 1,
            max_workers: num_cpus / 3,
            priority: LanguagePriority::Medium,
            memory_per_worker_mb: 128,
            specialized_detectors: vec!["TypeSafetyDetector".to_string()],
        });
        
        Self {
            max_workers: num_cpus,
            language_pools,
            work_stealing: WorkStealingConfig::default(),
            load_balancing: LoadBalancingConfig::default(),
            scheduling: TaskSchedulingConfig::default(),
            resource_management: ResourceManagementConfig::default(),
            monitoring: ParallelProcessingMonitoringConfig::default(),
        }
    }
}

/// Language-specific pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePoolConfig {
    /// Minimum workers for this language
    pub min_workers: usize,
    
    /// Maximum workers for this language
    pub max_workers: usize,
    
    /// Processing priority
    pub priority: LanguagePriority,
    
    /// Memory allocation per worker (MB)
    pub memory_per_worker_mb: usize,
    
    /// Specialized detectors for this language
    pub specialized_detectors: Vec<String>,
}

/// Language processing priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LanguagePriority {
    Critical = 1, // Must process first
    High = 2,     // High importance
    Medium = 3,   // Standard processing
    Low = 4,      // Background processing
}

/// Work stealing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStealingConfig {
    /// Enable work stealing between language pools
    pub enable_stealing: bool,
    
    /// Minimum queue size before allowing stealing
    pub steal_threshold: usize,
    
    /// Maximum items to steal per operation
    pub max_steal_count: usize,
    
    /// Work stealing strategy
    pub strategy: WorkStealingStrategy,
    
    /// Cooldown period between steal attempts
    pub steal_cooldown_ms: u64,
}

impl Default for WorkStealingConfig {
    fn default() -> Self {
        Self {
            enable_stealing: true,
            steal_threshold: 5,
            max_steal_count: 3,
            strategy: WorkStealingStrategy::Random,
            steal_cooldown_ms: 100,
        }
    }
}

/// Work stealing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkStealingStrategy {
    Random,        // Steal from random busy pool
    LeastLoaded,   // Steal from least loaded pool
    RoundRobin,    // Steal using round-robin
    Adaptive,      // Adaptive based on performance metrics
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,
    
    /// Rebalancing interval in seconds
    pub rebalance_interval_seconds: u64,
    
    /// Enable dynamic worker scaling
    pub enable_scaling: bool,
    
    /// Load imbalance threshold (0.0-1.0)
    pub imbalance_threshold: f64,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            algorithm: LoadBalancingAlgorithm::WeightedRoundRobin,
            rebalance_interval_seconds: 30,
            enable_scaling: true,
            imbalance_threshold: 0.3,
        }
    }
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,           // Simple round-robin
    WeightedRoundRobin,   // Weighted by language priority
    LeastConnections,     // Route to least busy worker
    ResourceBased,        // Based on memory/CPU usage
    Adaptive,             // ML-based adaptive balancing
}

/// Task scheduling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSchedulingConfig {
    /// Task priority algorithm
    pub priority_algorithm: TaskPriorityAlgorithm,
    
    /// Maximum task queue size per worker
    pub max_queue_size: usize,
    
    /// Task timeout in seconds
    pub task_timeout_seconds: u64,
    
    /// Enable task preemption for high-priority tasks
    pub enable_preemption: bool,
    
    /// Batch processing configuration
    pub batch_config: BatchProcessingConfig,
}

impl Default for TaskSchedulingConfig {
    fn default() -> Self {
        Self {
            priority_algorithm: TaskPriorityAlgorithm::FileSize,
            max_queue_size: 1000,
            task_timeout_seconds: 300, // 5 minutes
            enable_preemption: false,
            batch_config: BatchProcessingConfig::default(),
        }
    }
}

/// Task priority algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriorityAlgorithm {
    FileSize,         // Priority based on file size
    Complexity,       // Based on estimated complexity
    Dependencies,     // Based on dependency count
    Critical,         // Critical files first
    Balanced,         // Balanced approach
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingConfig {
    /// Enable batch processing
    pub enable_batching: bool,
    
    /// Minimum batch size
    pub min_batch_size: usize,
    
    /// Maximum batch size
    pub max_batch_size: usize,
    
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
}

impl Default for BatchProcessingConfig {
    fn default() -> Self {
        Self {
            enable_batching: true,
            min_batch_size: 5,
            max_batch_size: 50,
            batch_timeout_ms: 1000,
        }
    }
}

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
    /// Memory limit per language pool (MB)
    pub memory_limit_per_pool_mb: usize,
    
    /// CPU utilization threshold (0.0-1.0)
    pub cpu_utilization_threshold: f64,
    
    /// Memory utilization threshold (0.0-1.0)
    pub memory_utilization_threshold: f64,
    
    /// Enable adaptive resource scaling
    pub enable_adaptive_scaling: bool,
    
    /// Resource monitoring interval in seconds
    pub monitoring_interval_seconds: u64,
}

impl Default for ResourceManagementConfig {
    fn default() -> Self {
        Self {
            memory_limit_per_pool_mb: 1024, // 1GB per pool
            cpu_utilization_threshold: 0.8,
            memory_utilization_threshold: 0.85,
            enable_adaptive_scaling: true,
            monitoring_interval_seconds: 10,
        }
    }
}

/// Parallel processing monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelProcessingMonitoringConfig {
    /// Enable detailed performance monitoring
    pub enable_monitoring: bool,
    
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,
    
    /// Enable worker performance tracking
    pub track_worker_performance: bool,
    
    /// Enable queue depth monitoring
    pub monitor_queue_depths: bool,
}

impl Default for ParallelProcessingMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            metrics_interval_seconds: 30,
            track_worker_performance: true,
            monitor_queue_depths: true,
        }
    }
}

/// Analysis task for parallel processing
#[derive(Debug, Clone)]
pub struct AnalysisTask {
    /// File to analyze
    pub file_path: PathBuf,
    
    /// File metadata
    pub file_metadata: FileMetadata,
    
    /// Task priority
    pub priority: TaskPriority,
    
    /// Creation timestamp
    pub created_at: Instant,
    
    /// Dependencies on other tasks
    pub dependencies: Vec<PathBuf>,
    
    /// Estimated processing time
    pub estimated_duration: Duration,
}

/// File metadata for task scheduling
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size_bytes: usize,
    
    /// Programming language
    pub language: SourceLanguage,
    
    /// Last modification time
    pub last_modified: std::time::SystemTime,
    
    /// Estimated complexity score
    pub complexity_score: f64,
    
    /// Dependency count
    pub dependency_count: usize,
}

/// Task priority for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 1,  // Must process immediately
    High = 2,      // High importance
    Medium = 3,    // Standard processing
    Low = 4,       // Background processing
}

/// Worker pool statistics
#[derive(Debug, Clone, Default)]
pub struct WorkerPoolStats {
    pub active_workers: usize,
    pub idle_workers: usize,
    pub queue_depth: usize,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_task_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_utilization: f64,
    pub work_stolen: u64,
    pub work_provided: u64,
}

/// Parallel processing performance metrics
#[derive(Debug, Clone, Default)]
pub struct ParallelProcessingMetrics {
    /// Per-language worker pool statistics
    pub pool_stats: HashMap<String, WorkerPoolStats>,
    
    /// Overall processing metrics
    pub total_tasks_processed: u64,
    pub total_processing_time_ms: u64,
    pub average_parallelization_factor: f64,
    pub work_stealing_efficiency: f64,
    pub load_balance_factor: f64,
    
    /// Resource utilization
    pub peak_memory_usage_mb: f64,
    pub average_cpu_utilization: f64,
    pub resource_scaling_events: u64,
}

/// Multi-language parallel processing engine
pub struct MultiLanguageProcessor {
    config: ParallelProcessingConfig,
    
    // Worker pools by language
    language_pools: HashMap<String, Arc<LanguageWorkerPool>>,
    
    // Global task scheduler
    scheduler: Arc<TaskScheduler>,
    
    // Work stealing coordinator
    work_stealer: Option<Arc<WorkStealingCoordinator>>,
    
    // Load balancer
    load_balancer: Arc<LoadBalancer>,
    
    // Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
    
    // Performance metrics
    metrics: Arc<RwLock<ParallelProcessingMetrics>>,
    
    // Shutdown coordination
    shutdown_signal: Arc<AtomicBool>,
    
    // Background task handles
    background_tasks: Vec<JoinHandle<()>>,
}

impl MultiLanguageProcessor {
    /// Create new multi-language parallel processor
    pub async fn new(config: ParallelProcessingConfig) -> Result<Self, AnalysisError> {
        info!("Initializing multi-language parallel processor with {} max workers", config.max_workers);
        
        // Create language-specific worker pools
        let mut language_pools = HashMap::new();
        for (language, pool_config) in &config.language_pools {
            let pool = Arc::new(LanguageWorkerPool::new(language.clone(), pool_config.clone()).await?);
            language_pools.insert(language.clone(), pool);
        }
        
        // Create task scheduler
        let scheduler = Arc::new(TaskScheduler::new(config.scheduling.clone()));
        
        // Create work stealing coordinator if enabled
        let work_stealer = if config.work_stealing.enable_stealing {
            Some(Arc::new(WorkStealingCoordinator::new(config.work_stealing.clone())))
        } else {
            None
        };
        
        // Create load balancer
        let load_balancer = Arc::new(LoadBalancer::new(config.load_balancing.clone()));
        
        // Create resource monitor
        let resource_monitor = Arc::new(ResourceMonitor::new(config.resource_management.clone()));
        
        let mut processor = Self {
            config,
            language_pools,
            scheduler,
            work_stealer,
            load_balancer,
            resource_monitor,
            metrics: Arc::new(RwLock::new(ParallelProcessingMetrics::default())),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            background_tasks: Vec::new(),
        };
        
        // Start background tasks
        processor.start_background_tasks().await?;
        
        info!("Multi-language parallel processor initialized successfully");
        Ok(processor)
    }
    
    /// Process files with parallel multi-language analysis
    pub async fn process_files_parallel(
        &self,
        file_paths: Vec<PathBuf>,
        analysis_engine: Arc<AnalysisEngine>,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        info!("Starting parallel processing of {} files", file_paths.len());
        let start_time = Instant::now();
        
        // Convert file paths to analysis tasks
        let tasks = self.create_analysis_tasks(file_paths).await?;
        
        // Schedule tasks across language pools
        self.schedule_tasks(tasks).await?;
        
        // Collect results from all pools
        let (all_issues, combined_graph) = self.collect_results(analysis_engine).await?;
        
        // Update performance metrics
        let total_time = start_time.elapsed();
        self.update_processing_metrics(total_time, all_issues.len()).await;
        
        info!(
            "Parallel processing completed in {:?}: {} issues, {} dependencies",
            total_time,
            all_issues.len(),
            combined_graph.node_count()
        );
        
        Ok((all_issues, combined_graph))
    }
    
    /// Create analysis tasks from file paths with metadata
    async fn create_analysis_tasks(&self, file_paths: Vec<PathBuf>) -> Result<Vec<AnalysisTask>, AnalysisError> {
        let mut tasks = Vec::new();
        
        for file_path in file_paths {
            let metadata = self.analyze_file_metadata(&file_path).await?;
            let priority = self.calculate_task_priority(&metadata);
            let estimated_duration = self.estimate_processing_time(&metadata);
            
            let task = AnalysisTask {
                file_path,
                file_metadata: metadata,
                priority,
                created_at: Instant::now(),
                dependencies: Vec::new(), // TODO: Implement dependency analysis
                estimated_duration,
            };
            
            tasks.push(task);
        }
        
        // Sort tasks by priority and estimated complexity
        tasks.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then_with(|| b.file_metadata.complexity_score.partial_cmp(&a.file_metadata.complexity_score).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        Ok(tasks)
    }
    
    /// Schedule tasks across appropriate language pools
    async fn schedule_tasks(&self, tasks: Vec<AnalysisTask>) -> Result<(), AnalysisError> {
        info!("Scheduling {} tasks across language pools", tasks.len());
        
        for task in tasks {
            let language_str = match task.file_metadata.language {
                SourceLanguage::Rust => "rust",
                SourceLanguage::Python => "python",
                SourceLanguage::JavaScript => "javascript",
                SourceLanguage::TypeScript => "typescript",
                _ => "unknown",
            };
            
            // Get appropriate worker pool
            if let Some(pool) = self.language_pools.get(language_str) {
                pool.submit_task(task).await?;
            } else {
                warn!("No worker pool found for language: {}", language_str);
                // Fall back to a default pool or create a generic one
                if let Some((_, default_pool)) = self.language_pools.iter().next() {
                    default_pool.submit_task(task).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Collect results from all language pools
    async fn collect_results(
        &self,
        analysis_engine: Arc<AnalysisEngine>,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        let mut all_issues = Vec::new();
        let mut combined_graph = LocalDependencyGraph::new();
        
        // Wait for all pools to complete their work
        let results = futures::future::join_all(
            self.language_pools.values()
                .map(|pool| pool.wait_for_completion(analysis_engine.clone()))
        ).await;
        
        // Combine results from all pools
        for result in results {
            match result {
                Ok((issues, graph)) => {
                    all_issues.extend(issues);
                    combined_graph.merge(graph)?;
                }
                Err(e) => {
                    error!("Worker pool failed: {}", e);
                    return Err(e);
                }
            }
        }
        
        Ok((all_issues, combined_graph))
    }
    
    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> ParallelProcessingMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// Get worker pool statistics
    pub async fn get_pool_statistics(&self) -> HashMap<String, WorkerPoolStats> {
        let mut stats = HashMap::new();
        
        for (language, pool) in &self.language_pools {
            stats.insert(language.clone(), pool.get_stats().await);
        }
        
        stats
    }
    
    /// Optimize processing configuration for current workload
    pub async fn optimize_for_workload(&self, workload_characteristics: WorkloadCharacteristics) -> Result<(), AnalysisError> {
        info!("Optimizing parallel processing for workload: {:?}", workload_characteristics);
        
        // Adjust worker pool sizes based on workload
        for (language, characteristics) in workload_characteristics.language_distributions {
            if let Some(pool) = self.language_pools.get(&language) {
                let optimal_workers = self.calculate_optimal_workers(&characteristics);
                pool.adjust_worker_count(optimal_workers).await?;
            }
        }
        
        // Update load balancing strategy if needed
        self.load_balancer.optimize_for_workload(&workload_characteristics).await?;
        
        info!("Parallel processing optimization completed");
        Ok(())
    }
    
    /// Shutdown the parallel processor gracefully
    pub async fn shutdown(&mut self) -> Result<(), AnalysisError> {
        info!("Shutting down multi-language parallel processor");
        
        // Signal shutdown
        self.shutdown_signal.store(true, Ordering::Relaxed);
        
        // Shutdown all worker pools
        for (language, pool) in &self.language_pools {
            info!("Shutting down {} worker pool", language);
            pool.shutdown().await?;
        }
        
        // Wait for background tasks to complete
        for handle in self.background_tasks.drain(..) {
            if let Err(e) = handle.await {
                warn!("Background task failed during shutdown: {}", e);
            }
        }
        
        info!("Multi-language parallel processor shutdown completed");
        Ok(())
    }
    
    // Helper methods
    
    async fn analyze_file_metadata(&self, file_path: &Path) -> Result<FileMetadata, AnalysisError> {
        let metadata = tokio::fs::metadata(file_path).await
            .map_err(|e| AnalysisError::IO(e.to_string()))?;
            
        let size_bytes = metadata.len() as usize;
        let last_modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        
        // Detect language
        let language = self.detect_file_language(file_path);
        
        // Estimate complexity (basic heuristic)
        let complexity_score = (size_bytes as f64).log10() * match language {
            SourceLanguage::Rust => 1.2,
            SourceLanguage::TypeScript => 1.1,
            SourceLanguage::JavaScript => 1.0,
            SourceLanguage::Python => 0.9,
            _ => 1.0,
        };
        
        Ok(FileMetadata {
            size_bytes,
            language,
            last_modified,
            complexity_score,
            dependency_count: 0, // TODO: Implement dependency counting
        })
    }
    
    fn detect_file_language(&self, file_path: &Path) -> SourceLanguage {
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceLanguage::Rust,
            Some("py") => SourceLanguage::Python,
            Some("js" | "jsx") => SourceLanguage::JavaScript,
            Some("ts" | "tsx") => SourceLanguage::TypeScript,
            _ => SourceLanguage::Unknown,
        }
    }
    
    fn calculate_task_priority(&self, metadata: &FileMetadata) -> TaskPriority {
        // Priority based on file size and complexity
        if metadata.complexity_score > 5.0 && metadata.size_bytes > 100_000 {
            TaskPriority::High
        } else if metadata.size_bytes < 1000 {
            TaskPriority::Low
        } else {
            TaskPriority::Medium
        }
    }
    
    fn estimate_processing_time(&self, metadata: &FileMetadata) -> Duration {
        // Basic time estimation based on file size and language
        let base_time_ms = (metadata.size_bytes as f64 / 1000.0) * match metadata.language {
            SourceLanguage::Rust => 2.0,      // Rust parsing is slower
            SourceLanguage::TypeScript => 1.5, // TypeScript has type checking overhead
            SourceLanguage::JavaScript => 1.0, // Baseline
            SourceLanguage::Python => 0.8,    // Python is generally faster to parse
            _ => 1.0,
        };
        
        Duration::from_millis(base_time_ms as u64)
    }
    
    async fn start_background_tasks(&mut self) -> Result<(), AnalysisError> {
        // Start work stealing coordinator if enabled
        if let Some(work_stealer) = &self.work_stealer {
            let task = self.start_work_stealing_task(work_stealer.clone()).await;
            self.background_tasks.push(task);
        }
        
        // Start load balancer task
        let lb_task = self.start_load_balancing_task().await;
        self.background_tasks.push(lb_task);
        
        // Start resource monitoring task
        let rm_task = self.start_resource_monitoring_task().await;
        self.background_tasks.push(rm_task);
        
        Ok(())
    }
    
    async fn start_work_stealing_task(&self, work_stealer: Arc<WorkStealingCoordinator>) -> JoinHandle<()> {
        let pools = self.language_pools.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let config = self.config.work_stealing.clone();
        
        tokio::spawn(async move {
            while !shutdown_signal.load(Ordering::Relaxed) {
                work_stealer.coordinate_work_stealing(&pools).await;
                tokio::time::sleep(Duration::from_millis(config.steal_cooldown_ms)).await;
            }
        })
    }
    
    async fn start_load_balancing_task(&self) -> JoinHandle<()> {
        let load_balancer = self.load_balancer.clone();
        let pools = self.language_pools.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let interval = self.config.load_balancing.rebalance_interval_seconds;
        
        tokio::spawn(async move {
            while !shutdown_signal.load(Ordering::Relaxed) {
                load_balancer.rebalance_loads(&pools).await;
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        })
    }
    
    async fn start_resource_monitoring_task(&self) -> JoinHandle<()> {
        let resource_monitor = self.resource_monitor.clone();
        let pools = self.language_pools.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let interval = self.config.resource_management.monitoring_interval_seconds;
        
        tokio::spawn(async move {
            while !shutdown_signal.load(Ordering::Relaxed) {
                resource_monitor.monitor_resources(&pools).await;
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        })
    }
    
    async fn update_processing_metrics(&self, total_time: Duration, total_issues: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.total_tasks_processed += total_issues as u64;
        metrics.total_processing_time_ms += total_time.as_millis() as u64;
        
        // Calculate parallelization factor
        let expected_serial_time = total_issues as f64 * 100.0; // Estimated 100ms per file
        let actual_parallel_time = total_time.as_millis() as f64;
        metrics.average_parallelization_factor = expected_serial_time / actual_parallel_time.max(1.0);
    }
    
    fn calculate_optimal_workers(&self, _characteristics: &LanguageWorkloadCharacteristics) -> usize {
        // Placeholder for optimal worker calculation
        self.config.max_workers / self.language_pools.len()
    }
}

/// Workload characteristics for optimization
#[derive(Debug, Clone)]
pub struct WorkloadCharacteristics {
    pub total_files: usize,
    pub average_file_size: usize,
    pub language_distributions: HashMap<String, LanguageWorkloadCharacteristics>,
    pub peak_memory_requirement: usize,
    pub estimated_total_time: Duration,
}

/// Language-specific workload characteristics  
#[derive(Debug, Clone)]
pub struct LanguageWorkloadCharacteristics {
    pub file_count: usize,
    pub average_complexity: f64,
    pub memory_requirement: usize,
    pub processing_time_estimate: Duration,
}

// Additional supporting structs would be implemented here:
// - LanguageWorkerPool
// - TaskScheduler  
// - WorkStealingCoordinator
// - LoadBalancer
// - ResourceMonitor

/// Placeholder implementation for LanguageWorkerPool
pub struct LanguageWorkerPool {
    language: String,
    config: LanguagePoolConfig,
    // Additional fields would be implemented
}

impl LanguageWorkerPool {
    async fn new(language: String, config: LanguagePoolConfig) -> Result<Self, AnalysisError> {
        Ok(Self { language, config })
    }
    
    async fn submit_task(&self, _task: AnalysisTask) -> Result<(), AnalysisError> {
        Ok(())
    }
    
    async fn wait_for_completion(&self, _analysis_engine: Arc<AnalysisEngine>) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        Ok((Vec::new(), LocalDependencyGraph::new()))
    }
    
    async fn get_stats(&self) -> WorkerPoolStats {
        WorkerPoolStats::default()
    }
    
    async fn adjust_worker_count(&self, _count: usize) -> Result<(), AnalysisError> {
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), AnalysisError> {
        Ok(())
    }
}

/// Placeholder implementations for other supporting structures
pub struct TaskScheduler {
    config: TaskSchedulingConfig,
}

impl TaskScheduler {
    fn new(config: TaskSchedulingConfig) -> Self {
        Self { config }
    }
}

pub struct WorkStealingCoordinator {
    config: WorkStealingConfig,
}

impl WorkStealingCoordinator {
    fn new(config: WorkStealingConfig) -> Self {
        Self { config }
    }
    
    async fn coordinate_work_stealing(&self, _pools: &HashMap<String, Arc<LanguageWorkerPool>>) {
        // Implementation would coordinate work stealing
    }
}

pub struct LoadBalancer {
    config: LoadBalancingConfig,
}

impl LoadBalancer {
    fn new(config: LoadBalancingConfig) -> Self {
        Self { config }
    }
    
    async fn rebalance_loads(&self, _pools: &HashMap<String, Arc<LanguageWorkerPool>>) {
        // Implementation would rebalance loads
    }
    
    async fn optimize_for_workload(&self, _workload: &WorkloadCharacteristics) -> Result<(), AnalysisError> {
        Ok(())
    }
}

pub struct ResourceMonitor {
    config: ResourceManagementConfig,
}

impl ResourceMonitor {
    fn new(config: ResourceManagementConfig) -> Self {
        Self { config }
    }
    
    async fn monitor_resources(&self, _pools: &HashMap<String, Arc<LanguageWorkerPool>>) {
        // Implementation would monitor resource usage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_processing_config() {
        let config = ParallelProcessingConfig::default();
        assert!(config.max_workers > 0);
        assert!(config.language_pools.contains_key("rust"));
        assert!(config.work_stealing.enable_stealing);
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical < TaskPriority::High);
        assert!(TaskPriority::High < TaskPriority::Medium);
        assert!(TaskPriority::Medium < TaskPriority::Low);
    }

    #[test]
    fn test_language_priority_ordering() {
        assert!(LanguagePriority::Critical < LanguagePriority::High);
        assert!(LanguagePriority::High < LanguagePriority::Medium);
        assert!(LanguagePriority::Medium < LanguagePriority::Low);
    }

    #[tokio::test]
    async fn test_file_metadata_analysis() {
        // Test would verify file metadata extraction
        let processor = MultiLanguageProcessor::new(ParallelProcessingConfig::default()).await.unwrap();
        
        // Create a temporary test file
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
        write!(temp_file, "fn main() {{ println!(\"Hello, world!\"); }}").unwrap();
        
        let metadata = processor.analyze_file_metadata(temp_file.path()).await.unwrap();
        assert_eq!(metadata.language, SourceLanguage::Rust);
        assert!(metadata.size_bytes > 0);
    }
}