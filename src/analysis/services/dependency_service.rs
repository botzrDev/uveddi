//! Dependency Analysis Service
//!
//! This service handles dependency graph analysis, including building dependency graphs,
//! detecting cycles, and analyzing dependency relationships. It replaces the dependency
//! analysis logic from the monolithic AnalysisEngine.

use crate::analysis::components::{AstProviderImpl, CacheManagerImpl, DependencyGraphBuilderImpl};
use crate::analysis::components::traits::{
    AstProvider as AstProviderTrait,
    DependencyGraphBuilder as DependencyGraphBuilderTrait,
};
use crate::analysis::components::CacheManager as CacheManagerTrait;
use crate::analysis::detectors::dependency::{Dependency, DependencyExtractor};
use crate::analysis::graph::dependency::{LocalDependencyGraph, ComponentNode, LocalDependencyType, EdgeCount, IntoEdges};
use crate::analysis::traits::DependencyExtractorTrait;
use crate::analysis::components::ast_provider::ParsedFile;
use crate::database::models::ArchitecturalIssue;

use super::{AnalysisResult, ServiceConfiguration};

use crate::core::logging::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// A circular dependency detected in the codebase
#[derive(Debug, Clone)]
pub struct CycleDependency {
    pub cycle_path: Vec<String>,
    pub cycle_length: usize,
    pub severity: CycleSeverity,
    pub affected_files: Vec<PathBuf>,
}

/// Severity level of detected cycles
#[derive(Debug, Clone, PartialEq)]
pub enum CycleSeverity {
    Low,    // Simple 2-node cycles
    Medium, // 3-5 node cycles
    High,   // 6+ node cycles or complex patterns
}

/// Statistics about dependency analysis
#[derive(Debug, Clone, Default)]
pub struct DependencyStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub cycles_detected: usize,
    pub max_cycle_length: usize,
    pub analysis_duration_ms: u64,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

/// Service responsible for dependency analysis and graph construction
pub struct DependencyAnalysisService {
    ast_provider: Arc<AstProviderImpl>,
    dependency_builder: Arc<DependencyGraphBuilderImpl>,
    cache_manager: Arc<CacheManagerImpl>,
    dependency_extractor: Arc<DependencyExtractor>,
    
    // Internal state
    cached_graphs: Arc<RwLock<HashMap<String, LocalDependencyGraph>>>,
    stats: Arc<RwLock<DependencyStats>>,
}

impl DependencyAnalysisService {
    /// Create new dependency analysis service
    pub fn new(
        ast_provider: Arc<AstProviderImpl>,
        dependency_builder: Arc<DependencyGraphBuilderImpl>,
        cache_manager: Arc<CacheManagerImpl>,
    ) -> Self {
        Self {
            ast_provider,
            dependency_builder,
            cache_manager,
            dependency_extractor: Arc::new(DependencyExtractor::new().expect("Failed to create dependency extractor")),
            cached_graphs: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(DependencyStats::default())),
        }
    }

    /// Build comprehensive dependency graph for a path
    pub async fn build_dependency_graph(&self, path: &Path) -> AnalysisResult<LocalDependencyGraph> {
        let start_time = std::time::Instant::now();
        info!("Building dependency graph for: {}", path.display());

        // Generate cache key
        let cache_key = self.generate_cache_key(path);

        // Check cache first
        if let Some(cached_graph) = self.get_cached_graph(&cache_key).await {
            debug!("Using cached dependency graph for {}", path.display());
            self.update_cache_stats(true).await;
            return Ok(cached_graph);
        }

        self.update_cache_stats(false).await;

        // Build new graph
        let graph = if path.is_file() {
            self.build_file_dependency_graph(path).await?
        } else {
            self.build_directory_dependency_graph(path).await?
        };

        // Cache the result
        self.cache_dependency_graph(cache_key, &graph).await?;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_nodes = graph.node_count();
            stats.total_edges = graph.edge_count();
            stats.analysis_duration_ms = start_time.elapsed().as_millis() as u64;
        }

        info!("Dependency graph built: {} nodes, {} edges in {}ms", 
              graph.node_count(), graph.edge_count(), start_time.elapsed().as_millis());
        Ok(graph)
    }

    /// Build dependency graph for a single file
    async fn build_file_dependency_graph(&self, file_path: &Path) -> AnalysisResult<LocalDependencyGraph> {
        debug!("Building file dependency graph for: {}", file_path.display());

        // Parse the file
        let parsed_file = self.ast_provider.parse_file(file_path).await?;
        
        // Extract dependencies from the parsed file
        let dependencies = self.dependency_extractor.extract_dependencies(&parsed_file).await?;

        // Build graph from dependencies
        Ok(self.dependency_builder.build_from_dependencies(dependencies))
    }

    /// Build dependency graph for a directory
    async fn build_directory_dependency_graph(&self, dir_path: &Path) -> AnalysisResult<LocalDependencyGraph> {
        debug!("Building directory dependency graph for: {}", dir_path.display());

        // Use the dependency builder to scan the directory
        self.dependency_builder.build_graph(dir_path).await.map_err(Into::into)
    }

    /// Analyze circular dependencies in the graph
    pub async fn analyze_cycles(&self, graph: &LocalDependencyGraph) -> AnalysisResult<Vec<CycleDependency>> {
        info!("Analyzing cycles in dependency graph with {} nodes", graph.node_count());

        let cycles = self.dependency_builder.detect_cycles(graph).await?;
        
        // Convert internal cycle representation to our CycleDependency format
        let mut cycle_dependencies = Vec::new();
        for cycle in cycles {
            let cycle_dependency = CycleDependency {
                cycle_length: cycle.len(),
                severity: self.determine_cycle_severity(cycle.len()),
                cycle_path: cycle.clone(),
                affected_files: self.get_affected_files_for_cycle(&cycle, graph).await,
            };
            cycle_dependencies.push(cycle_dependency);
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.cycles_detected = cycle_dependencies.len();
            stats.max_cycle_length = cycle_dependencies.iter()
                .map(|c| c.cycle_length)
                .max()
                .unwrap_or(0);
        }

        info!("Found {} cycles", cycle_dependencies.len());
        Ok(cycle_dependencies)
    }

    /// Extract dependencies from parsed files
    pub async fn extract_dependencies(&self, files: &[ParsedFile]) -> AnalysisResult<Vec<Dependency>> {
        debug!("Extracting dependencies from {} parsed files", files.len());

        let mut all_dependencies = Vec::new();
        
        for file in files {
            match self.dependency_extractor.extract_dependencies(file).await {
                Ok(deps) => all_dependencies.extend(deps),
                Err(e) => warn!("Failed to extract dependencies from {}: {}", 
                               file.file_path.display(), e),
            }
        }

        info!("Extracted {} total dependencies", all_dependencies.len());
        Ok(all_dependencies)
    }

    /// Get dependency analysis statistics
    pub async fn get_dependency_stats(&self) -> DependencyStats {
        self.stats.read().await.clone()
    }

    /// Reset dependency analysis statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = DependencyStats::default();
    }

    /// Clear dependency graph cache
    pub async fn clear_cache(&self) {
        let mut cached_graphs = self.cached_graphs.write().await;
        cached_graphs.clear();
        info!("Dependency graph cache cleared");
    }

    /// Get cache status information
    pub async fn get_cache_info(&self) -> (usize, usize, usize) {
        let cached_graphs = self.cached_graphs.read().await;
        let stats = self.stats.read().await;
        (cached_graphs.len(), stats.cache_hits, stats.cache_misses)
    }

    /// Analyze specific dependency types in the graph
    pub async fn analyze_dependency_types(&self, graph: &LocalDependencyGraph) -> HashMap<LocalDependencyType, usize> {
        let mut type_counts = HashMap::new();
        
        for edge in graph.edges() {
            let dep_type = edge.dependency_type.clone();
            *type_counts.entry(dep_type).or_insert(0) += 1;
        }

        type_counts
    }

    /// Find strongly connected components in the graph
    pub async fn find_strongly_connected_components(&self, graph: &LocalDependencyGraph) -> AnalysisResult<Vec<Vec<String>>> {
        self.dependency_builder.find_strongly_connected_components(graph).await.map_err(Into::into)
    }

    // Private helper methods

    fn generate_cache_key(&self, path: &Path) -> String {
        format!("dep_graph_{}", path.display())
    }

    async fn get_cached_graph(&self, cache_key: &str) -> Option<LocalDependencyGraph> {
        let cached_graphs = self.cached_graphs.read().await;
        cached_graphs.get(cache_key).cloned()
    }

    async fn cache_dependency_graph(&self, cache_key: String, graph: &LocalDependencyGraph) -> AnalysisResult<()> {
        let mut cached_graphs = self.cached_graphs.write().await;
        cached_graphs.insert(cache_key.clone(), graph.clone());
        
        // Also cache in the persistent cache manager
        self.cache_manager.cache_dependency_graph(&cache_key, graph).await.map_err(Into::into)
    }

    async fn update_cache_stats(&self, cache_hit: bool) {
        let mut stats = self.stats.write().await;
        if cache_hit {
            stats.cache_hits += 1;
        } else {
            stats.cache_misses += 1;
        }
    }

    fn determine_cycle_severity(&self, cycle_length: usize) -> CycleSeverity {
        match cycle_length {
            2 => CycleSeverity::Low,
            3..=5 => CycleSeverity::Medium,
            _ => CycleSeverity::High,
        }
    }

    async fn get_affected_files_for_cycle(&self, cycle: &[String], graph: &LocalDependencyGraph) -> Vec<PathBuf> {
        let mut affected_files = HashSet::new();
        
        for node_name in cycle {
            if let Some(node) = graph.get_node(node_name) {
                if let Some(file_path) = node.file_path() {
                    affected_files.insert(file_path.clone());
                }
            }
        }
        
        affected_files.into_iter().map(|s| PathBuf::from(s)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_dependency_service() -> DependencyAnalysisService {
        let ast_provider = Arc::new(AstProviderImpl::new());
        let dependency_builder = Arc::new(DependencyGraphBuilderImpl::new());
        let cache_manager = Arc::new(CacheManagerImpl::new_with_defaults());

        DependencyAnalysisService::new(
            ast_provider,
            dependency_builder,
            cache_manager,
        )
    }

    #[tokio::test]
    async fn test_dependency_service_creation() {
        let service = create_test_dependency_service();
        let stats = service.get_dependency_stats().await;
        
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_edges, 0);
        assert_eq!(stats.cycles_detected, 0);
    }

    #[tokio::test]
    async fn test_cycle_severity_determination() {
        let service = create_test_dependency_service();
        
        assert_eq!(service.determine_cycle_severity(2), CycleSeverity::Low);
        assert_eq!(service.determine_cycle_severity(3), CycleSeverity::Medium);
        assert_eq!(service.determine_cycle_severity(4), CycleSeverity::Medium);
        assert_eq!(service.determine_cycle_severity(5), CycleSeverity::Medium);
        assert_eq!(service.determine_cycle_severity(6), CycleSeverity::High);
        assert_eq!(service.determine_cycle_severity(10), CycleSeverity::High);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let service = create_test_dependency_service();
        let path = Path::new("/test/path");
        let cache_key = service.generate_cache_key(path);
        
        assert!(cache_key.contains("dep_graph_"));
        assert!(cache_key.contains("/test/path"));
    }

    #[tokio::test]
    async fn test_stats_reset() {
        let service = create_test_dependency_service();
        
        // Modify stats manually for testing
        {
            let mut stats = service.stats.write().await;
            stats.total_nodes = 10;
            stats.total_edges = 15;
            stats.cycles_detected = 2;
        }
        
        // Reset stats
        service.reset_stats().await;
        
        // Verify reset
        let stats = service.get_dependency_stats().await;
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_edges, 0);
        assert_eq!(stats.cycles_detected, 0);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let service = create_test_dependency_service();
        
        // Initially empty cache
        let (cache_size, hits, misses) = service.get_cache_info().await;
        assert_eq!(cache_size, 0);
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        
        // Clear empty cache should work
        service.clear_cache().await;
        
        let (cache_size_after_clear, _, _) = service.get_cache_info().await;
        assert_eq!(cache_size_after_clear, 0);
    }
}