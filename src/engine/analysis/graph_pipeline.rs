//! # Graph-Aware Analysis Pipeline
//!
//! Integrates knowledge graph construction with the cached analysis pipeline.
//! Provides incremental graph building and relationship-aware detection.

use crate::engine::analysis::context::AnalysisContext;
use crate::engine::analysis::pipeline::AnalysisPipeline;
use crate::engine::cache::{GraphCache, SharedGraphCache, create_shared_graph_cache, GraphCacheConfig};
use crate::engine::knowledge_graph::{GraphBuilder, KnowledgeGraph};
use crate::engine::knowledge_graph::builder::GraphBuildError;
use crate::engine::knowledge_graph::query::QueryBuilder;
// Note: DetectionResult import removed as it's not available in current architecture
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Graph-aware analysis pipeline that builds knowledge graphs incrementally
pub struct GraphAwarePipeline {
    /// Base analysis pipeline
    base_pipeline: AnalysisPipeline,
    /// Graph cache for relationships and computations
    graph_cache: SharedGraphCache,
    /// Current knowledge graph
    knowledge_graph: Arc<Mutex<KnowledgeGraph>>,
    /// Graph builder for incremental updates
    graph_builder: Arc<Mutex<GraphBuilder>>,
    /// Pipeline configuration
    config: GraphPipelineConfig,
    /// Performance metrics
    metrics: GraphPipelineMetrics,
}

/// Configuration for graph-aware pipeline
#[derive(Debug, Clone)]
pub struct GraphPipelineConfig {
    /// Enable incremental graph building
    pub incremental_build: bool,
    /// Enable graph-based detector optimizations
    pub graph_optimizations: bool,
    /// Maximum graph size before partial rebuilds
    pub max_graph_size: usize,
    /// Enable dependency-aware analysis ordering
    pub dependency_ordering: bool,
    /// Graph cache configuration
    pub cache_config: GraphCacheConfig,
}

impl Default for GraphPipelineConfig {
    fn default() -> Self {
        Self {
            incremental_build: true,
            graph_optimizations: true,
            max_graph_size: 100_000,
            dependency_ordering: true,
            cache_config: GraphCacheConfig::default(),
        }
    }
}

/// Performance metrics for graph-aware pipeline
#[derive(Debug, Default, Clone)]
pub struct GraphPipelineMetrics {
    pub total_analyses: u64,
    pub graph_builds: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_build_time_ms: f64,
    pub avg_analysis_time_ms: f64,
}

impl GraphAwarePipeline {
    /// Create a new graph-aware analysis pipeline
    pub fn new(base_pipeline: AnalysisPipeline, config: GraphPipelineConfig) -> Self {
        let graph_cache = create_shared_graph_cache(config.cache_config.clone());

        #[cfg(feature = "analysis-cache")]
        let graph_builder = Arc::new(Mutex::new(GraphBuilder::with_cache()));
        #[cfg(not(feature = "analysis-cache"))]
        let graph_builder = Arc::new(Mutex::new(GraphBuilder::new()));

        Self {
            base_pipeline,
            graph_cache,
            knowledge_graph: Arc::new(Mutex::new(KnowledgeGraph::default())),
            graph_builder,
            config,
            metrics: GraphPipelineMetrics::default(),
        }
    }

    /// Analyze files with graph-aware enhancements
    pub async fn analyze_with_graph(&mut self, contexts: Vec<AnalysisContext>) -> Result<GraphAnalysisResult, GraphAnalysisError> {
        let start_time = Instant::now();

        // Build/update knowledge graph from contexts
        let graph_update_start = Instant::now();
        self.update_knowledge_graph(&contexts).await?;
        let graph_build_time = graph_update_start.elapsed();

        // Perform analysis with graph context
        let analysis_start = Instant::now();
        let base_results: Vec<serde_json::Value> = Vec::new(); // Placeholder for actual analysis results
        let analysis_time = analysis_start.elapsed();

        // Enhance results with graph insights
        let enhanced_results = self.enhance_with_graph_insights(base_results, &contexts).await?;

        // Update metrics
        self.update_metrics(graph_build_time, analysis_time, contexts.len());

        Ok(GraphAnalysisResult {
            detection_results: enhanced_results,
            graph_stats: self.get_graph_stats().await,
            performance_metrics: self.metrics.clone(),
            cache_efficiency: self.get_cache_efficiency().await,
        })
    }

    /// Update knowledge graph with new contexts
    async fn update_knowledge_graph(&mut self, contexts: &[AnalysisContext]) -> Result<(), GraphAnalysisError> {
        let mut builder = self.graph_builder.lock()
            .map_err(|e| GraphAnalysisError::LockError(format!("Graph builder lock failed: {}", e)))?;

        for context in contexts {
            // Check if we can use cached graph data
            let file_path = context.file_info.path.to_string_lossy();
            let file_hash = self.compute_file_hash(context);

            if let Ok(mut cache) = self.graph_cache.write() {
                // Try to get cached relations
                if let Some(cached_relations) = cache.get_relations(&file_path, file_hash) {
                    // Use cached data
                    continue;
                }
            }

            // Build fresh graph data
            builder.build_from_context(context)
                .map_err(|e| GraphAnalysisError::GraphBuildError(e))?;
        }

        // Update the main knowledge graph
        let new_graph = builder.get_graph().clone();
        if let Ok(mut graph) = self.knowledge_graph.lock() {
            *graph = new_graph;
        }

        Ok(())
    }

    /// Enhance detection results with graph insights
    async fn enhance_with_graph_insights(
        &self,
        mut results: Vec<serde_json::Value>,
        contexts: &[AnalysisContext],
    ) -> Result<Vec<serde_json::Value>, GraphAnalysisError> {
        if !self.config.graph_optimizations {
            return Ok(results);
        }

        let graph = self.knowledge_graph.lock()
            .map_err(|e| GraphAnalysisError::LockError(format!("Knowledge graph lock failed: {}", e)))?;

        // Build query engine for graph insights
        let query_builder = QueryBuilder::new();

        for (i, result) in results.iter_mut().enumerate() {
            if i < contexts.len() {
                let context = &contexts[i];

                // Create metadata object for graph insights
                let mut metadata = serde_json::Map::new();

                // Add dependency analysis
                if let Some(dependencies) = self.analyze_dependencies(&query_builder, context).await {
                    metadata.insert("dependencies".to_string(),
                                   serde_json::Value::Array(dependencies));
                }

                // Add coupling analysis
                if let Some(coupling_score) = self.analyze_coupling(&query_builder, context).await {
                    metadata.insert("coupling_score".to_string(),
                                   serde_json::Value::from(coupling_score));
                }

                // Add impact analysis
                if let Some(impact) = self.analyze_impact(&query_builder, context).await {
                    metadata.insert("change_impact".to_string(),
                                   serde_json::Value::Array(impact));
                }

                // Enhance the result with metadata
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("graph_metadata".to_string(), serde_json::Value::Object(metadata));
                }
            }
        }

        Ok(results)
    }

    /// Analyze dependencies for a file
    async fn analyze_dependencies(
        &self,
        query_builder: &QueryBuilder,
        context: &AnalysisContext,
    ) -> Option<Vec<serde_json::Value>> {
        let file_path = context.file_info.path.to_string_lossy();

        // Check cache first
        if let Ok(mut cache) = self.graph_cache.write() {
            let cache_key = format!("deps:{}", file_path);
            if let Some(cached_deps) = cache.get_dependencies(&cache_key) {
                return Some(cached_deps.nodes.iter()
                    .map(|dep| serde_json::Value::String(dep.clone()))
                    .collect());
            }
        }

        // Compute dependencies using graph
        let dependencies = query_builder.find_dependencies(&file_path)?;

        // Cache the result
        if let Ok(mut cache) = self.graph_cache.write() {
            let cache_key = format!("deps:{}", file_path);
            cache.cache_dependencies(cache_key, dependencies.clone(), vec![], vec![]);
        }

        Some(dependencies.iter()
            .map(|dep| serde_json::Value::String(dep.clone()))
            .collect())
    }

    /// Analyze coupling for a file
    async fn analyze_coupling(
        &self,
        query_builder: &QueryBuilder,
        context: &AnalysisContext,
    ) -> Option<f32> {
        let file_path = context.file_info.path.to_string_lossy();

        // Compute coupling metrics
        let incoming = query_builder.count_incoming_relations(&file_path).unwrap_or(0);
        let outgoing = query_builder.count_outgoing_relations(&file_path).unwrap_or(0);

        if incoming + outgoing == 0 {
            return Some(0.0);
        }

        // Simple coupling metric: ratio of connections to total possible connections
        let coupling_score = (incoming + outgoing) as f32 / (incoming + outgoing + 1) as f32;
        Some(coupling_score)
    }

    /// Analyze change impact for a file
    async fn analyze_impact(
        &self,
        query_builder: &QueryBuilder,
        context: &AnalysisContext,
    ) -> Option<Vec<serde_json::Value>> {
        let file_path = context.file_info.path.to_string_lossy();

        // Find all files that depend on this file
        let dependents = query_builder.find_dependents(&file_path)?;

        Some(dependents.iter()
            .map(|dep| serde_json::Value::String(dep.clone()))
            .collect())
    }

    /// Get current graph statistics
    async fn get_graph_stats(&self) -> GraphStats {
        if let Ok(graph) = self.knowledge_graph.lock() {
            let builder_stats = graph.stats();
            GraphStats {
                node_count: builder_stats.node_count,
                edge_count: builder_stats.edge_count,
                symbol_count: builder_stats.symbol_count,
            }
        } else {
            GraphStats {
                node_count: 0,
                edge_count: 0,
                symbol_count: 0,
            }
        }
    }

    /// Get cache efficiency metrics
    async fn get_cache_efficiency(&self) -> CacheEfficiency {
        if let Ok(cache) = self.graph_cache.read() {
            let cache_stats = cache.get_cache_stats();
            let metrics = cache.get_metrics();

            CacheEfficiency {
                hit_rate: metrics.hit_rate("graph_relations"),
                total_entries: cache_stats.total_entries,
                memory_usage_estimate: cache_stats.total_entries * 256, // Rough estimate
            }
        } else {
            CacheEfficiency::default()
        }
    }

    /// Update performance metrics
    fn update_metrics(&mut self, graph_build_time: std::time::Duration, analysis_time: std::time::Duration, context_count: usize) {
        self.metrics.total_analyses += context_count as u64;
        self.metrics.graph_builds += 1;

        let build_time_ms = graph_build_time.as_millis() as f64;
        let analysis_time_ms = analysis_time.as_millis() as f64;

        // Update running averages
        self.metrics.avg_build_time_ms = (self.metrics.avg_build_time_ms * (self.metrics.graph_builds - 1) as f64 + build_time_ms) / self.metrics.graph_builds as f64;
        self.metrics.avg_analysis_time_ms = (self.metrics.avg_analysis_time_ms * (self.metrics.total_analyses - context_count as u64) as f64 + analysis_time_ms) / self.metrics.total_analyses as f64;
    }

    /// Compute file hash for cache keys
    fn compute_file_hash(&self, context: &AnalysisContext) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        context.file_info.path.hash(&mut hasher);
        context.file_info.modified_at.hash(&mut hasher);
        context.source.len().hash(&mut hasher); // Simple content fingerprint
        hasher.finish()
    }
}

/// Result of graph-aware analysis
#[derive(Debug)]
pub struct GraphAnalysisResult {
    pub detection_results: Vec<serde_json::Value>, // Generic result format
    pub graph_stats: GraphStats,
    pub performance_metrics: GraphPipelineMetrics,
    pub cache_efficiency: CacheEfficiency,
}

/// Cache efficiency metrics
#[derive(Debug, Default)]
pub struct CacheEfficiency {
    pub hit_rate: f64,
    pub total_entries: usize,
    pub memory_usage_estimate: usize,
}

/// Graph statistics
#[derive(Debug, Default)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub symbol_count: usize,
}

/// Errors that can occur during graph-aware analysis
#[derive(Debug)]
pub enum GraphAnalysisError {
    /// Graph building failed
    GraphBuildError(GraphBuildError),
    /// Base analysis pipeline failed
    AnalysisError(String),
    /// Lock acquisition failed
    LockError(String),
    /// Cache operation failed
    CacheError(String),
}

impl std::fmt::Display for GraphAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphAnalysisError::GraphBuildError(e) => write!(f, "Graph build error: {}", e),
            GraphAnalysisError::AnalysisError(e) => write!(f, "Analysis error: {}", e),
            GraphAnalysisError::LockError(e) => write!(f, "Lock error: {}", e),
            GraphAnalysisError::CacheError(e) => write!(f, "Cache error: {}", e),
        }
    }
}

impl std::error::Error for GraphAnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GraphAnalysisError::GraphBuildError(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::analysis::context::{FileInfo, ProjectContext};
    use crate::ast::SourceLanguage;
    use std::path::PathBuf;

    fn create_test_context() -> AnalysisContext {
        AnalysisContext::new(
            FileInfo {
                path: PathBuf::from("test.rs"),
                language: SourceLanguage::Rust,
                lines_of_code: 100,
                size_bytes: 2048,
                modified_at: std::time::SystemTime::now(),
            },
            None,
            "fn test() {}".to_string(),
            vec![],
            vec![],
            ProjectContext {
                project_root: PathBuf::from("/test"),
                project_files: vec![],
                dependencies: vec![],
                global_symbols: vec![],
            },
        )
    }

    #[tokio::test]
    async fn test_graph_pipeline_creation() {
        let base_pipeline = AnalysisPipeline::new();
        let config = GraphPipelineConfig::default();
        let pipeline = GraphAwarePipeline::new(base_pipeline, config);

        // Pipeline should be created successfully
        assert_eq!(pipeline.metrics.total_analyses, 0);
    }

    #[tokio::test]
    async fn test_knowledge_graph_update() {
        let base_pipeline = AnalysisPipeline::new();
        let config = GraphPipelineConfig::default();
        let mut pipeline = GraphAwarePipeline::new(base_pipeline, config);

        let contexts = vec![create_test_context()];

        // Should update graph without errors
        let result = pipeline.update_knowledge_graph(&contexts).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_efficiency_tracking() {
        let base_pipeline = AnalysisPipeline::new();
        let config = GraphPipelineConfig::default();
        let pipeline = GraphAwarePipeline::new(base_pipeline, config);

        let efficiency = pipeline.get_cache_efficiency().await;

        // Should return valid efficiency metrics
        assert!(efficiency.hit_rate >= 0.0 && efficiency.hit_rate <= 1.0);
    }
}