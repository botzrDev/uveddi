//! Enterprise Performance Metrics Collection for UV-91
//!
//! Comprehensive metrics collection system for enterprise-scale performance monitoring.
//! Supports 6 optimization areas with detailed baseline tracking and regression detection.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

/// Enterprise metrics collector for all 6 optimization areas
#[derive(Debug)]
pub struct EnterpriseMetricsCollector {
    pipeline_metrics: Arc<RwLock<PipelineMetrics>>,
    memory_metrics: Arc<RwLock<MemoryMetrics>>,
    ast_metrics: Arc<RwLock<AstParsingMetrics>>,
    diagram_metrics: Arc<RwLock<DiagramGenerationMetrics>>,
    cache_metrics: Arc<RwLock<CacheMetrics>>,
    incremental_metrics: Arc<RwLock<IncrementalAnalysisMetrics>>,
    baseline_data: Arc<RwLock<BaselineData>>,
    measurement_history: Arc<RwLock<Vec<MeasurementSnapshot>>>,
}

/// 1. Pipeline Performance Metrics (<70ms target)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetrics {
    pub total_analyses: u64,
    pub average_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub max_latency_ms: f64,
    pub min_latency_ms: f64,
    pub under_target_percentage: f64, // % under 70ms
    pub latency_distribution: Vec<LatencyBucket>,
    pub error_rate: f64,
    pub throughput_analyses_per_second: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyBucket {
    pub range_ms: (u64, u64),
    pub count: u64,
    pub percentage: f64,
}

/// 2. Memory Usage Metrics (<8GB target)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub peak_usage_bytes: u64,
    pub average_usage_bytes: u64,
    pub current_usage_bytes: u64,
    pub allocation_rate_per_second: f64,
    pub deallocation_rate_per_second: f64,
    pub gc_collections: u64,
    pub memory_fragmentation_ratio: f64,
    pub large_object_count: u64,
    pub memory_pressure_events: u64,
    pub under_target_percentage: f64, // % under 8GB
    pub memory_growth_rate_mb_per_hour: f64,
    pub memory_leaks_detected: Vec<MemoryLeak>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLeak {
    pub allocation_site: String,
    pub leaked_bytes: u64,
    pub detection_time: DateTime<Utc>,
    pub severity: LeakSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeakSeverity {
    Low,      // < 1MB
    Medium,   // 1MB - 100MB
    High,     // 100MB - 1GB
    Critical, // > 1GB
}

/// 3. AST Parsing Performance Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstParsingMetrics {
    pub total_files_parsed: u64,
    pub average_parse_time_ms: f64,
    pub parsing_throughput_files_per_second: f64,
    pub tree_sitter_optimization_level: u8,
    pub parse_cache_hit_rate: f64,
    pub parse_error_rate: f64,
    pub average_ast_size_nodes: u64,
    pub parsing_memory_overhead_bytes: u64,
    pub language_specific_metrics: HashMap<String, LanguageParsingMetrics>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageParsingMetrics {
    pub language: String,
    pub files_parsed: u64,
    pub average_parse_time_ms: f64,
    pub average_ast_size_nodes: u64,
    pub success_rate: f64,
}

/// 4. Diagram Generation Metrics (1000+/min target)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramGenerationMetrics {
    pub total_diagrams_generated: u64,
    pub generation_rate_per_minute: f64,
    pub average_generation_time_ms: f64,
    pub rendering_service_hit_rate: f64,
    pub diagram_cache_efficiency: f64,
    pub over_target_percentage: f64, // % over 1000/min
    pub diagram_complexity_distribution: Vec<ComplexityBucket>,
    pub diagram_types: HashMap<String, DiagramTypeMetrics>,
    pub rendering_errors: u64,
    pub queue_length: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityBucket {
    pub complexity_range: (u32, u32),
    pub count: u64,
    pub average_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramTypeMetrics {
    pub diagram_type: String,
    pub count: u64,
    pub average_time_ms: f64,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
}

/// 5. Cache Performance Metrics (90%+ hit rate target)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub overall_hit_rate: f64,
    pub ast_cache_hit_rate: f64,
    pub diagram_cache_hit_rate: f64,
    pub result_cache_hit_rate: f64,
    pub total_cache_requests: u64,
    pub cache_size_bytes: u64,
    pub cache_evictions: u64,
    pub cache_invalidations: u64,
    pub average_lookup_time_microseconds: f64,
    pub cache_warming_efficiency: f64,
    pub over_target_percentage: f64, // % over 90%
    pub cache_layers: HashMap<String, CacheLayerMetrics>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLayerMetrics {
    pub layer_name: String,
    pub hit_rate: f64,
    pub size_bytes: u64,
    pub entry_count: u64,
    pub average_lookup_time_microseconds: f64,
}

/// 6. Incremental Analysis Metrics (50%+ improvement target)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalAnalysisMetrics {
    pub total_incremental_analyses: u64,
    pub average_improvement_percentage: f64,
    pub full_analysis_time_baseline_ms: f64,
    pub incremental_analysis_time_ms: f64,
    pub files_changed_percentage: f64,
    pub dependency_impact_ratio: f64,
    pub incremental_accuracy: f64, // % of results matching full analysis
    pub over_target_percentage: f64, // % over 50% improvement
    pub invalidation_cascade_metrics: InvalidationMetrics,
    pub dependency_graph_efficiency: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationMetrics {
    pub average_cascade_size: f64,
    pub max_cascade_size: u64,
    pub cascade_efficiency: f64,
    pub false_invalidations: u64,
}

/// Baseline data for comparison and regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineData {
    pub baseline_timestamp: DateTime<Utc>,
    pub pipeline_baseline: PipelineMetrics,
    pub memory_baseline: MemoryMetrics,
    pub ast_baseline: AstParsingMetrics,
    pub diagram_baseline: DiagramGenerationMetrics,
    pub cache_baseline: CacheMetrics,
    pub incremental_baseline: IncrementalAnalysisMetrics,
    pub environment_info: EnvironmentInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub cpu_cores: u32,
    pub total_memory_gb: f64,
    pub rust_version: String,
    pub os_version: String,
    pub build_mode: String, // debug/release
    pub git_commit: String,
}

/// Snapshot of all metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementSnapshot {
    pub timestamp: DateTime<Utc>,
    pub pipeline: PipelineMetrics,
    pub memory: MemoryMetrics,
    pub ast: AstParsingMetrics,
    pub diagram: DiagramGenerationMetrics,
    pub cache: CacheMetrics,
    pub incremental: IncrementalAnalysisMetrics,
    pub overall_score: f64, // Composite performance score
}

/// Performance regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub has_regression: bool,
    pub regression_details: Vec<RegressionDetail>,
    pub overall_performance_change: f64, // % change from baseline
    pub recommendation: String,
    pub severity: RegressionSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetail {
    pub metric_category: String,
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub change_percentage: f64,
    pub threshold_exceeded: bool,
    pub impact_level: RegressionImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    None,
    Minor,    // < 5% degradation
    Moderate, // 5-15% degradation
    Major,    // 15-30% degradation
    Critical, // > 30% degradation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionImpact {
    Low,
    Medium,
    High,
    Critical,
}

impl EnterpriseMetricsCollector {
    pub fn new() -> Self {
        Self {
            pipeline_metrics: Arc::new(RwLock::new(PipelineMetrics::default())),
            memory_metrics: Arc::new(RwLock::new(MemoryMetrics::default())),
            ast_metrics: Arc::new(RwLock::new(AstParsingMetrics::default())),
            diagram_metrics: Arc::new(RwLock::new(DiagramGenerationMetrics::default())),
            cache_metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            incremental_metrics: Arc::new(RwLock::new(IncrementalAnalysisMetrics::default())),
            baseline_data: Arc::new(RwLock::new(BaselineData::default())),
            measurement_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record pipeline performance measurement
    pub async fn record_pipeline_measurement(&self, latency: Duration, success: bool) {
        let mut metrics = self.pipeline_metrics.write().await;
        let latency_ms = latency.as_millis() as f64;

        metrics.total_analyses += 1;

        // Update latency statistics
        if metrics.total_analyses == 1 {
            metrics.average_latency_ms = latency_ms;
            metrics.min_latency_ms = latency_ms;
            metrics.max_latency_ms = latency_ms;
        } else {
            let count = metrics.total_analyses as f64;
            metrics.average_latency_ms =
                (metrics.average_latency_ms * (count - 1.0) + latency_ms) / count;
            metrics.min_latency_ms = metrics.min_latency_ms.min(latency_ms);
            metrics.max_latency_ms = metrics.max_latency_ms.max(latency_ms);
        }

        // Update error rate
        if !success {
            metrics.error_rate = (metrics.error_rate * (metrics.total_analyses - 1) as f64 + 1.0)
                / metrics.total_analyses as f64;
        } else {
            metrics.error_rate = (metrics.error_rate * (metrics.total_analyses - 1) as f64)
                / metrics.total_analyses as f64;
        }

        // Update target compliance
        if latency_ms < 70.0 {
            let under_target_count = (metrics.under_target_percentage / 100.0
                * (metrics.total_analyses - 1) as f64)
                + 1.0;
            metrics.under_target_percentage =
                (under_target_count / metrics.total_analyses as f64) * 100.0;
        } else {
            let under_target_count =
                metrics.under_target_percentage / 100.0 * (metrics.total_analyses - 1) as f64;
            metrics.under_target_percentage =
                (under_target_count / metrics.total_analyses as f64) * 100.0;
        }

        metrics.last_updated = Utc::now();
    }

    /// Record memory usage measurement
    pub async fn record_memory_measurement(&self, current_usage_bytes: u64, allocation_delta: i64) {
        let mut metrics = self.memory_metrics.write().await;

        metrics.current_usage_bytes = current_usage_bytes;
        metrics.peak_usage_bytes = metrics.peak_usage_bytes.max(current_usage_bytes);

        // Update average (moving average)
        if metrics.average_usage_bytes == 0 {
            metrics.average_usage_bytes = current_usage_bytes;
        } else {
            metrics.average_usage_bytes = ((metrics.average_usage_bytes as f64 * 0.95)
                + (current_usage_bytes as f64 * 0.05))
                as u64;
        }

        // Update allocation rate
        if allocation_delta > 0 {
            metrics.allocation_rate_per_second =
                (metrics.allocation_rate_per_second * 0.9) + (allocation_delta as f64 * 0.1);
        } else if allocation_delta < 0 {
            metrics.deallocation_rate_per_second =
                (metrics.deallocation_rate_per_second * 0.9) + ((-allocation_delta) as f64 * 0.1);
        }

        // Check target compliance (8GB = 8 * 1024^3 bytes)
        let target_bytes = 8 * 1024 * 1024 * 1024u64;
        if current_usage_bytes < target_bytes {
            metrics.under_target_percentage =
                (metrics.under_target_percentage * 0.95) + (100.0 * 0.05);
        } else {
            metrics.under_target_percentage =
                (metrics.under_target_percentage * 0.95) + (0.0 * 0.05);
        }

        metrics.last_updated = Utc::now();
    }

    /// Record AST parsing measurement
    pub async fn record_ast_parsing_measurement(
        &self,
        language: &str,
        parse_time: Duration,
        ast_size_nodes: u64,
        success: bool,
    ) {
        let mut metrics = self.ast_metrics.write().await;
        let parse_time_ms = parse_time.as_millis() as f64;

        metrics.total_files_parsed += 1;

        // Update average parse time
        let count = metrics.total_files_parsed as f64;
        metrics.average_parse_time_ms =
            (metrics.average_parse_time_ms * (count - 1.0) + parse_time_ms) / count;

        // Update average AST size
        metrics.average_ast_size_nodes =
            ((metrics.average_ast_size_nodes * (metrics.total_files_parsed - 1)) + ast_size_nodes)
                / metrics.total_files_parsed;

        // Update error rate
        if !success {
            metrics.parse_error_rate = (metrics.parse_error_rate * (count - 1.0) + 1.0) / count;
        } else {
            metrics.parse_error_rate = (metrics.parse_error_rate * (count - 1.0)) / count;
        }

        // Update language-specific metrics
        let lang_metrics = metrics
            .language_specific_metrics
            .entry(language.to_string())
            .or_insert_with(|| LanguageParsingMetrics {
                language: language.to_string(),
                files_parsed: 0,
                average_parse_time_ms: 0.0,
                average_ast_size_nodes: 0,
                success_rate: 100.0,
            });

        lang_metrics.files_parsed += 1;
        let lang_count = lang_metrics.files_parsed as f64;
        lang_metrics.average_parse_time_ms =
            (lang_metrics.average_parse_time_ms * (lang_count - 1.0) + parse_time_ms) / lang_count;
        lang_metrics.average_ast_size_nodes = ((lang_metrics.average_ast_size_nodes
            * (lang_metrics.files_parsed - 1))
            + ast_size_nodes)
            / lang_metrics.files_parsed;

        if success {
            lang_metrics.success_rate =
                (lang_metrics.success_rate * (lang_count - 1.0) + 100.0) / lang_count;
        } else {
            lang_metrics.success_rate =
                (lang_metrics.success_rate * (lang_count - 1.0) + 0.0) / lang_count;
        }

        metrics.last_updated = Utc::now();
    }

    /// Record diagram generation measurement
    pub async fn record_diagram_generation_measurement(
        &self,
        diagram_type: &str,
        generation_time: Duration,
        _complexity: u32,
        cache_hit: bool,
        success: bool,
    ) {
        let mut metrics = self.diagram_metrics.write().await;
        let generation_time_ms = generation_time.as_millis() as f64;

        if success {
            metrics.total_diagrams_generated += 1;

            // Update average generation time
            let count = metrics.total_diagrams_generated as f64;
            metrics.average_generation_time_ms =
                (metrics.average_generation_time_ms * (count - 1.0) + generation_time_ms) / count;

            // Update rendering service hit rate
            if cache_hit {
                metrics.rendering_service_hit_rate =
                    (metrics.rendering_service_hit_rate * (count - 1.0) + 100.0) / count;
            } else {
                metrics.rendering_service_hit_rate =
                    (metrics.rendering_service_hit_rate * (count - 1.0) + 0.0) / count;
            }

            // Update generation rate (diagrams per minute)
            // This is a simplified calculation - in production, use a time window
            metrics.generation_rate_per_minute = count / 1.0; // Placeholder

            // Check target compliance (1000+ per minute)
            if metrics.generation_rate_per_minute >= 1000.0 {
                metrics.over_target_percentage =
                    (metrics.over_target_percentage * 0.95) + (100.0 * 0.05);
            } else {
                metrics.over_target_percentage =
                    (metrics.over_target_percentage * 0.95) + (0.0 * 0.05);
            }

            // Update diagram type metrics
            let type_metrics = metrics
                .diagram_types
                .entry(diagram_type.to_string())
                .or_insert_with(|| DiagramTypeMetrics {
                    diagram_type: diagram_type.to_string(),
                    count: 0,
                    average_time_ms: 0.0,
                    success_rate: 100.0,
                    cache_hit_rate: 0.0,
                });

            type_metrics.count += 1;
            let type_count = type_metrics.count as f64;
            type_metrics.average_time_ms = (type_metrics.average_time_ms * (type_count - 1.0)
                + generation_time_ms)
                / type_count;

            if cache_hit {
                type_metrics.cache_hit_rate =
                    (type_metrics.cache_hit_rate * (type_count - 1.0) + 100.0) / type_count;
            } else {
                type_metrics.cache_hit_rate =
                    (type_metrics.cache_hit_rate * (type_count - 1.0) + 0.0) / type_count;
            }
        } else {
            metrics.rendering_errors += 1;
        }

        metrics.last_updated = Utc::now();
    }

    /// Record cache performance measurement
    pub async fn record_cache_measurement(
        &self,
        cache_layer: &str,
        hit: bool,
        lookup_time: Duration,
        cache_size_bytes: u64,
    ) {
        let mut metrics = self.cache_metrics.write().await;
        let lookup_time_microseconds = lookup_time.as_micros() as f64;

        metrics.total_cache_requests += 1;
        metrics.cache_size_bytes = cache_size_bytes;

        // Update overall hit rate
        let count = metrics.total_cache_requests as f64;
        if hit {
            metrics.overall_hit_rate = (metrics.overall_hit_rate * (count - 1.0) + 100.0) / count;
        } else {
            metrics.overall_hit_rate = (metrics.overall_hit_rate * (count - 1.0) + 0.0) / count;
        }

        // Update average lookup time
        metrics.average_lookup_time_microseconds =
            (metrics.average_lookup_time_microseconds * (count - 1.0) + lookup_time_microseconds)
                / count;

        // Check target compliance (90%+ hit rate)
        if metrics.overall_hit_rate >= 90.0 {
            metrics.over_target_percentage =
                (metrics.over_target_percentage * 0.95) + (100.0 * 0.05);
        } else {
            metrics.over_target_percentage = (metrics.over_target_percentage * 0.95) + (0.0 * 0.05);
        }

        // Update cache layer metrics
        let layer_metrics = metrics
            .cache_layers
            .entry(cache_layer.to_string())
            .or_insert_with(|| CacheLayerMetrics {
                layer_name: cache_layer.to_string(),
                hit_rate: 0.0,
                size_bytes: 0,
                entry_count: 0,
                average_lookup_time_microseconds: 0.0,
            });

        layer_metrics.entry_count += 1;
        layer_metrics.size_bytes = cache_size_bytes;
        let layer_count = layer_metrics.entry_count as f64;

        if hit {
            layer_metrics.hit_rate =
                (layer_metrics.hit_rate * (layer_count - 1.0) + 100.0) / layer_count;
        } else {
            layer_metrics.hit_rate =
                (layer_metrics.hit_rate * (layer_count - 1.0) + 0.0) / layer_count;
        }

        layer_metrics.average_lookup_time_microseconds =
            (layer_metrics.average_lookup_time_microseconds * (layer_count - 1.0)
                + lookup_time_microseconds)
                / layer_count;

        metrics.last_updated = Utc::now();
    }

    /// Record incremental analysis measurement
    pub async fn record_incremental_analysis_measurement(
        &self,
        full_analysis_time: Duration,
        incremental_time: Duration,
        files_changed_percentage: f64,
        accuracy: f64,
    ) {
        let mut metrics = self.incremental_metrics.write().await;

        metrics.total_incremental_analyses += 1;

        let full_time_ms = full_analysis_time.as_millis() as f64;
        let incremental_time_ms = incremental_time.as_millis() as f64;
        let improvement = ((full_time_ms - incremental_time_ms) / full_time_ms) * 100.0;

        // Update average improvement
        let count = metrics.total_incremental_analyses as f64;
        metrics.average_improvement_percentage =
            (metrics.average_improvement_percentage * (count - 1.0) + improvement) / count;

        // Update baseline times
        metrics.full_analysis_time_baseline_ms =
            (metrics.full_analysis_time_baseline_ms * (count - 1.0) + full_time_ms) / count;
        metrics.incremental_analysis_time_ms =
            (metrics.incremental_analysis_time_ms * (count - 1.0) + incremental_time_ms) / count;

        // Update accuracy
        metrics.incremental_accuracy =
            (metrics.incremental_accuracy * (count - 1.0) + accuracy) / count;

        // Update files changed percentage
        metrics.files_changed_percentage =
            (metrics.files_changed_percentage * (count - 1.0) + files_changed_percentage) / count;

        // Check target compliance (50%+ improvement)
        if improvement >= 50.0 {
            metrics.over_target_percentage =
                (metrics.over_target_percentage * 0.95) + (100.0 * 0.05);
        } else {
            metrics.over_target_percentage = (metrics.over_target_percentage * 0.95) + (0.0 * 0.05);
        }

        metrics.last_updated = Utc::now();
    }

    /// Create baseline from current measurements
    pub async fn create_baseline(&self) -> BaselineData {
        BaselineData {
            baseline_timestamp: Utc::now(),
            pipeline_baseline: self.pipeline_metrics.read().await.clone(),
            memory_baseline: self.memory_metrics.read().await.clone(),
            ast_baseline: self.ast_metrics.read().await.clone(),
            diagram_baseline: self.diagram_metrics.read().await.clone(),
            cache_baseline: self.cache_metrics.read().await.clone(),
            incremental_baseline: self.incremental_metrics.read().await.clone(),
            environment_info: self.collect_environment_info(),
        }
    }

    /// Store baseline for future comparisons
    pub async fn store_baseline(&self, baseline: BaselineData) {
        *self.baseline_data.write().await = baseline;
    }

    /// Take a snapshot of all current metrics
    pub async fn take_snapshot(&self) -> MeasurementSnapshot {
        let pipeline = self.pipeline_metrics.read().await.clone();
        let memory = self.memory_metrics.read().await.clone();
        let ast = self.ast_metrics.read().await.clone();
        let diagram = self.diagram_metrics.read().await.clone();
        let cache = self.cache_metrics.read().await.clone();
        let incremental = self.incremental_metrics.read().await.clone();

        let overall_score =
            self.calculate_overall_score(&pipeline, &memory, &ast, &diagram, &cache, &incremental);

        let snapshot = MeasurementSnapshot {
            timestamp: Utc::now(),
            pipeline,
            memory,
            ast,
            diagram,
            cache,
            incremental,
            overall_score,
        };

        // Store in history
        self.measurement_history
            .write()
            .await
            .push(snapshot.clone());

        snapshot
    }

    /// Detect performance regressions compared to baseline
    pub async fn detect_regressions(&self) -> RegressionAnalysis {
        let baseline = self.baseline_data.read().await.clone();
        let current = self.take_snapshot().await;

        let mut regression_details = Vec::new();
        let mut has_regression = false;

        // Check pipeline metrics
        if self.check_pipeline_regression(
            &baseline.pipeline_baseline,
            &current.pipeline,
            &mut regression_details,
        ) {
            has_regression = true;
        }

        // Check memory metrics
        if self.check_memory_regression(
            &baseline.memory_baseline,
            &current.memory,
            &mut regression_details,
        ) {
            has_regression = true;
        }

        // Check AST metrics
        if self.check_ast_regression(
            &baseline.ast_baseline,
            &current.ast,
            &mut regression_details,
        ) {
            has_regression = true;
        }

        // Check diagram metrics
        if self.check_diagram_regression(
            &baseline.diagram_baseline,
            &current.diagram,
            &mut regression_details,
        ) {
            has_regression = true;
        }

        // Check cache metrics
        if self.check_cache_regression(
            &baseline.cache_baseline,
            &current.cache,
            &mut regression_details,
        ) {
            has_regression = true;
        }

        // Check incremental metrics
        if self.check_incremental_regression(
            &baseline.incremental_baseline,
            &current.incremental,
            &mut regression_details,
        ) {
            has_regression = true;
        }

        let overall_change = ((current.overall_score
            - baseline.pipeline_baseline.average_latency_ms)
            / baseline.pipeline_baseline.average_latency_ms)
            * 100.0;

        let severity = if overall_change.abs() < 5.0 {
            RegressionSeverity::None
        } else if overall_change.abs() < 15.0 {
            RegressionSeverity::Minor
        } else if overall_change.abs() < 30.0 {
            RegressionSeverity::Moderate
        } else {
            RegressionSeverity::Major
        };

        let recommendation =
            self.generate_regression_recommendation(&regression_details, &severity);

        RegressionAnalysis {
            has_regression,
            regression_details,
            overall_performance_change: overall_change,
            recommendation,
            severity,
        }
    }

    fn collect_environment_info(&self) -> EnvironmentInfo {
        EnvironmentInfo {
            cpu_cores: num_cpus::get() as u32,
            total_memory_gb: 16.0, // Placeholder - get from system
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            os_version: std::env::consts::OS.to_string(),
            build_mode: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
            .to_string(),
            git_commit: "HEAD".to_string(), // Placeholder - get from git
        }
    }

    fn calculate_overall_score(
        &self,
        pipeline: &PipelineMetrics,
        memory: &MemoryMetrics,
        ast: &AstParsingMetrics,
        diagram: &DiagramGenerationMetrics,
        cache: &CacheMetrics,
        incremental: &IncrementalAnalysisMetrics,
    ) -> f64 {
        // Weighted composite score (0-100)
        let pipeline_score = if pipeline.average_latency_ms < 70.0 {
            100.0
        } else {
            70.0 / pipeline.average_latency_ms * 100.0
        };
        let memory_score = if memory.current_usage_bytes < 8 * 1024 * 1024 * 1024 {
            100.0
        } else {
            50.0
        };
        let ast_score = (100.0 - ast.parse_error_rate).max(0.0);
        let diagram_score = if diagram.generation_rate_per_minute >= 1000.0 {
            100.0
        } else {
            diagram.generation_rate_per_minute / 10.0
        };
        let cache_score = cache.overall_hit_rate;
        let incremental_score = incremental
            .average_improvement_percentage
            .max(0.0)
            .min(100.0);

        // Weighted average
        (pipeline_score * 0.25
            + memory_score * 0.20
            + ast_score * 0.15
            + diagram_score * 0.15
            + cache_score * 0.15
            + incremental_score * 0.10)
    }

    fn check_pipeline_regression(
        &self,
        baseline: &PipelineMetrics,
        current: &PipelineMetrics,
        details: &mut Vec<RegressionDetail>,
    ) -> bool {
        let latency_change = ((current.average_latency_ms - baseline.average_latency_ms)
            / baseline.average_latency_ms)
            * 100.0;

        if latency_change > 10.0 {
            // 10% threshold
            details.push(RegressionDetail {
                metric_category: "Pipeline Performance".to_string(),
                metric_name: "Average Latency".to_string(),
                baseline_value: baseline.average_latency_ms,
                current_value: current.average_latency_ms,
                change_percentage: latency_change,
                threshold_exceeded: true,
                impact_level: if latency_change > 30.0 {
                    RegressionImpact::Critical
                } else if latency_change > 20.0 {
                    RegressionImpact::High
                } else {
                    RegressionImpact::Medium
                },
            });
            return true;
        }
        false
    }

    fn check_memory_regression(
        &self,
        baseline: &MemoryMetrics,
        current: &MemoryMetrics,
        details: &mut Vec<RegressionDetail>,
    ) -> bool {
        let memory_change = ((current.peak_usage_bytes as f64 - baseline.peak_usage_bytes as f64)
            / baseline.peak_usage_bytes as f64)
            * 100.0;

        if memory_change > 15.0 {
            // 15% threshold
            details.push(RegressionDetail {
                metric_category: "Memory Usage".to_string(),
                metric_name: "Peak Usage".to_string(),
                baseline_value: baseline.peak_usage_bytes as f64,
                current_value: current.peak_usage_bytes as f64,
                change_percentage: memory_change,
                threshold_exceeded: true,
                impact_level: if memory_change > 50.0 {
                    RegressionImpact::Critical
                } else if memory_change > 30.0 {
                    RegressionImpact::High
                } else {
                    RegressionImpact::Medium
                },
            });
            return true;
        }
        false
    }

    fn check_ast_regression(
        &self,
        baseline: &AstParsingMetrics,
        current: &AstParsingMetrics,
        details: &mut Vec<RegressionDetail>,
    ) -> bool {
        let parse_time_change = ((current.average_parse_time_ms - baseline.average_parse_time_ms)
            / baseline.average_parse_time_ms)
            * 100.0;

        if parse_time_change > 20.0 {
            // 20% threshold
            details.push(RegressionDetail {
                metric_category: "AST Parsing".to_string(),
                metric_name: "Average Parse Time".to_string(),
                baseline_value: baseline.average_parse_time_ms,
                current_value: current.average_parse_time_ms,
                change_percentage: parse_time_change,
                threshold_exceeded: true,
                impact_level: if parse_time_change > 50.0 {
                    RegressionImpact::Critical
                } else {
                    RegressionImpact::Medium
                },
            });
            return true;
        }
        false
    }

    fn check_diagram_regression(
        &self,
        baseline: &DiagramGenerationMetrics,
        current: &DiagramGenerationMetrics,
        details: &mut Vec<RegressionDetail>,
    ) -> bool {
        let rate_change = ((current.generation_rate_per_minute
            - baseline.generation_rate_per_minute)
            / baseline.generation_rate_per_minute)
            * 100.0;

        if rate_change < -10.0 {
            // 10% decrease threshold
            details.push(RegressionDetail {
                metric_category: "Diagram Generation".to_string(),
                metric_name: "Generation Rate".to_string(),
                baseline_value: baseline.generation_rate_per_minute,
                current_value: current.generation_rate_per_minute,
                change_percentage: rate_change,
                threshold_exceeded: true,
                impact_level: if rate_change < -30.0 {
                    RegressionImpact::Critical
                } else {
                    RegressionImpact::Medium
                },
            });
            return true;
        }
        false
    }

    fn check_cache_regression(
        &self,
        baseline: &CacheMetrics,
        current: &CacheMetrics,
        details: &mut Vec<RegressionDetail>,
    ) -> bool {
        let hit_rate_change = current.overall_hit_rate - baseline.overall_hit_rate;

        if hit_rate_change < -5.0 {
            // 5% decrease threshold
            details.push(RegressionDetail {
                metric_category: "Cache Performance".to_string(),
                metric_name: "Overall Hit Rate".to_string(),
                baseline_value: baseline.overall_hit_rate,
                current_value: current.overall_hit_rate,
                change_percentage: hit_rate_change,
                threshold_exceeded: true,
                impact_level: if hit_rate_change < -15.0 {
                    RegressionImpact::High
                } else {
                    RegressionImpact::Medium
                },
            });
            return true;
        }
        false
    }

    fn check_incremental_regression(
        &self,
        baseline: &IncrementalAnalysisMetrics,
        current: &IncrementalAnalysisMetrics,
        details: &mut Vec<RegressionDetail>,
    ) -> bool {
        let improvement_change =
            current.average_improvement_percentage - baseline.average_improvement_percentage;

        if improvement_change < -10.0 {
            // 10% decrease threshold
            details.push(RegressionDetail {
                metric_category: "Incremental Analysis".to_string(),
                metric_name: "Average Improvement".to_string(),
                baseline_value: baseline.average_improvement_percentage,
                current_value: current.average_improvement_percentage,
                change_percentage: improvement_change,
                threshold_exceeded: true,
                impact_level: if improvement_change < -25.0 {
                    RegressionImpact::Critical
                } else {
                    RegressionImpact::Medium
                },
            });
            return true;
        }
        false
    }

    fn generate_regression_recommendation(
        &self,
        details: &[RegressionDetail],
        severity: &RegressionSeverity,
    ) -> String {
        if details.is_empty() {
            return "No performance regressions detected. System is performing within acceptable ranges.".to_string();
        }

        let mut recommendations = Vec::new();

        for detail in details {
            match detail.metric_category.as_str() {
                "Pipeline Performance" => recommendations.push("Consider pipeline optimizations, async processing improvements, or resource allocation adjustments".to_string()),
                "Memory Usage" => recommendations.push("Investigate memory leaks, optimize data structures, or implement memory pooling".to_string()),
                "AST Parsing" => recommendations.push("Optimize Tree-sitter queries, implement parse caching, or parallelize parsing operations".to_string()),
                "Diagram Generation" => recommendations.push("Optimize rendering service, implement diagram caching, or scale rendering resources".to_string()),
                "Cache Performance" => recommendations.push("Tune cache eviction policies, increase cache size, or optimize cache key strategies".to_string()),
                "Incremental Analysis" => recommendations.push("Optimize dependency tracking, improve invalidation algorithms, or enhance change detection".to_string()),
                _ => {}
            }
        }

        let severity_action = match severity {
            RegressionSeverity::Critical => "IMMEDIATE ACTION REQUIRED: ",
            RegressionSeverity::Major => "High priority investigation needed: ",
            RegressionSeverity::Moderate => "Medium priority optimization: ",
            RegressionSeverity::Minor => "Low priority improvement: ",
            RegressionSeverity::None => "",
        };

        format!("{}{}", severity_action, recommendations.join("; "))
    }
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self {
            total_analyses: 0,
            average_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            max_latency_ms: 0.0,
            min_latency_ms: 0.0,
            under_target_percentage: 0.0,
            latency_distribution: Vec::new(),
            error_rate: 0.0,
            throughput_analyses_per_second: 0.0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            peak_usage_bytes: 0,
            average_usage_bytes: 0,
            current_usage_bytes: 0,
            allocation_rate_per_second: 0.0,
            deallocation_rate_per_second: 0.0,
            gc_collections: 0,
            memory_fragmentation_ratio: 0.0,
            large_object_count: 0,
            memory_pressure_events: 0,
            under_target_percentage: 100.0,
            memory_growth_rate_mb_per_hour: 0.0,
            memory_leaks_detected: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for AstParsingMetrics {
    fn default() -> Self {
        Self {
            total_files_parsed: 0,
            average_parse_time_ms: 0.0,
            parsing_throughput_files_per_second: 0.0,
            tree_sitter_optimization_level: 1,
            parse_cache_hit_rate: 0.0,
            parse_error_rate: 0.0,
            average_ast_size_nodes: 0,
            parsing_memory_overhead_bytes: 0,
            language_specific_metrics: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for DiagramGenerationMetrics {
    fn default() -> Self {
        Self {
            total_diagrams_generated: 0,
            generation_rate_per_minute: 0.0,
            average_generation_time_ms: 0.0,
            rendering_service_hit_rate: 0.0,
            diagram_cache_efficiency: 0.0,
            over_target_percentage: 0.0,
            diagram_complexity_distribution: Vec::new(),
            diagram_types: HashMap::new(),
            rendering_errors: 0,
            queue_length: 0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            overall_hit_rate: 0.0,
            ast_cache_hit_rate: 0.0,
            diagram_cache_hit_rate: 0.0,
            result_cache_hit_rate: 0.0,
            total_cache_requests: 0,
            cache_size_bytes: 0,
            cache_evictions: 0,
            cache_invalidations: 0,
            average_lookup_time_microseconds: 0.0,
            cache_warming_efficiency: 0.0,
            over_target_percentage: 0.0,
            cache_layers: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for IncrementalAnalysisMetrics {
    fn default() -> Self {
        Self {
            total_incremental_analyses: 0,
            average_improvement_percentage: 0.0,
            full_analysis_time_baseline_ms: 0.0,
            incremental_analysis_time_ms: 0.0,
            files_changed_percentage: 0.0,
            dependency_impact_ratio: 0.0,
            incremental_accuracy: 100.0,
            over_target_percentage: 0.0,
            invalidation_cascade_metrics: InvalidationMetrics {
                average_cascade_size: 0.0,
                max_cascade_size: 0,
                cascade_efficiency: 100.0,
                false_invalidations: 0,
            },
            dependency_graph_efficiency: 100.0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for BaselineData {
    fn default() -> Self {
        Self {
            baseline_timestamp: Utc::now(),
            pipeline_baseline: PipelineMetrics::default(),
            memory_baseline: MemoryMetrics::default(),
            ast_baseline: AstParsingMetrics::default(),
            diagram_baseline: DiagramGenerationMetrics::default(),
            cache_baseline: CacheMetrics::default(),
            incremental_baseline: IncrementalAnalysisMetrics::default(),
            environment_info: EnvironmentInfo {
                cpu_cores: num_cpus::get() as u32,
                total_memory_gb: 16.0,
                rust_version: env!("CARGO_PKG_VERSION").to_string(),
                os_version: std::env::consts::OS.to_string(),
                build_mode: if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                }
                .to_string(),
                git_commit: "HEAD".to_string(),
            },
        }
    }
}
