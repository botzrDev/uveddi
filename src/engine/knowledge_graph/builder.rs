//! # Knowledge Graph Builder
//!
//! Incremental knowledge graph construction from parsed files and symbols.
//! Integrates with AnalysisContext and cached analysis pipeline.
//! Target: <400 lines total

use super::relations::{GraphRelation, RelationType};
use crate::engine::analysis::context::AnalysisContext;
use crate::engine::parsing::{Relation, Symbol};
use std::collections::{HashMap, HashSet};

// Cache-aware imports
#[cfg(feature = "analysis-cache")]
use crate::engine::cache::{AnalysisCache, CacheMetricsCollector};
#[cfg(feature = "analysis-cache")]
use std::sync::{Arc, Mutex};

/// Knowledge graph containing all code relationships
#[derive(Debug, Default, Clone)]
pub struct KnowledgeGraph {
    /// All nodes in the graph (symbols)
    nodes: HashMap<String, GraphNode>,

    /// All edges in the graph (relationships)
    edges: HashMap<String, Vec<GraphRelation>>,

    /// Index for efficient querying
    symbol_index: HashMap<String, HashSet<String>>,
}

/// Node in the knowledge graph
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub symbol: Symbol,
    pub file_path: String,
}

/// Builder for incrementally constructing knowledge graphs
pub struct GraphBuilder {
    graph: KnowledgeGraph,

    /// Cache manager for graph data (when caching enabled)
    #[cfg(feature = "analysis-cache")]
    cache_manager: Option<Arc<Mutex<GraphCacheManager>>>,
}

/// Cache manager for graph-specific data
#[cfg(feature = "analysis-cache")]
pub struct GraphCacheManager {
    /// Cache for computed graph relationships
    relation_cache: HashMap<String, Vec<GraphRelation>>,
    /// Cache for symbol resolutions
    symbol_resolution_cache: HashMap<String, Option<String>>,
    /// Cache for graph statistics
    stats_cache: HashMap<String, GraphStats>,
    /// Metrics collector
    metrics: CacheMetricsCollector,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            graph: KnowledgeGraph::default(),
            #[cfg(feature = "analysis-cache")]
            cache_manager: None,
        }
    }

    /// Create a new graph builder with cache support
    #[cfg(feature = "analysis-cache")]
    pub fn with_cache() -> Self {
        Self {
            graph: KnowledgeGraph::default(),
            cache_manager: Some(Arc::new(Mutex::new(GraphCacheManager::new()))),
        }
    }

    /// Build graph from analysis context (cache-aware)
    pub fn build_from_context(&mut self, context: &AnalysisContext) -> Result<(), GraphBuildError> {
        let file_path = context.file_info.path.to_string_lossy().to_string();

        // Add symbols from context
        self.add_symbols(&file_path, context.symbols.clone());

        // Add relations from context
        self.add_relations(&file_path, context.relations.clone());

        // If caching is enabled, try to use cached data
        #[cfg(feature = "analysis-cache")]
        if let Some(ref cache_manager) = self.cache_manager {
            self.build_with_cache_support(&file_path, context)?;
        }

        Ok(())
    }

    /// Add symbols from a file to the graph
    pub fn add_symbols(&mut self, file_path: &str, symbols: Vec<Symbol>) {
        for symbol in symbols {
            let symbol_name = symbol.name.clone();
            let node_id = self.generate_node_id(file_path, &symbol_name);
            let node = GraphNode {
                id: node_id.clone(),
                symbol,
                file_path: file_path.to_string(),
            };

            self.graph.nodes.insert(node_id.clone(), node);

            // Update symbol index
            self.graph
                .symbol_index
                .entry(symbol_name)
                .or_insert_with(HashSet::new)
                .insert(node_id);
        }
    }

    /// Add relations from a file to the graph
    pub fn add_relations(&mut self, file_path: &str, relations: Vec<Relation>) {
        for relation in relations {
            let from_id = self.generate_node_id(file_path, &relation.from);
            let to_id = self
                .resolve_symbol_reference(&relation.to)
                .unwrap_or_else(|| {
                    // Create external reference node if not found
                    self.generate_node_id("external", &relation.to)
                });

            let graph_relation = GraphRelation {
                from: from_id.clone(),
                to: to_id,
                relation_type: self.convert_relation_type(&relation.kind),
                file_path: file_path.to_string(),
            };

            self.graph
                .edges
                .entry(from_id)
                .or_insert_with(Vec::new)
                .push(graph_relation);
        }
    }

    /// Build the final knowledge graph
    pub fn build(self) -> KnowledgeGraph {
        self.graph
    }

    /// Get a reference to the current graph state without consuming the builder
    pub fn get_graph(&self) -> &KnowledgeGraph {
        &self.graph
    }

    /// Cache-aware graph building support
    #[cfg(feature = "analysis-cache")]
    fn build_with_cache_support(&mut self, file_path: &str, context: &AnalysisContext) -> Result<(), GraphBuildError> {
        if let Some(ref cache_manager) = self.cache_manager {
            let cache_key = self.generate_cache_key(file_path, &context.file_info.modified_at);

            // Try to get cached relations
            if let Ok(mut cache) = cache_manager.lock() {
                if let Some(cached_relations) = cache.get_cached_relations(&cache_key) {
                    // Use cached data if available
                    for relation in cached_relations {
                        self.graph.edges
                            .entry(relation.from.clone())
                            .or_insert_with(Vec::new)
                            .push(relation.clone());
                    }
                    cache.metrics.record_hit("graph_relations");
                    return Ok(());
                }
                cache.metrics.record_miss("graph_relations");
            }

            // Build fresh and cache the results
            self.build_and_cache_relations(file_path, &cache_key)?;
        }
        Ok(())
    }

    /// Generate cache key for graph data
    #[cfg(feature = "analysis-cache")]
    fn generate_cache_key(&self, file_path: &str, modified_at: &std::time::SystemTime) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        modified_at.hash(&mut hasher);
        format!("graph_{:x}", hasher.finish())
    }

    /// Build relations and cache them
    #[cfg(feature = "analysis-cache")]
    fn build_and_cache_relations(&mut self, file_path: &str, cache_key: &str) -> Result<(), GraphBuildError> {
        let relations_to_cache: Vec<GraphRelation> = self.graph.edges
            .values()
            .flat_map(|relations| relations.iter().cloned())
            .collect();

        if let Some(ref cache_manager) = self.cache_manager {
            if let Ok(mut cache) = cache_manager.lock() {
                cache.cache_relations(cache_key.to_string(), relations_to_cache);
            }
        }
        Ok(())
    }

    /// Generate a unique node ID
    fn generate_node_id(&self, file_path: &str, symbol_name: &str) -> String {
        format!("{}::{}", file_path, symbol_name)
    }

    /// Resolve a symbol reference to a node ID
    fn resolve_symbol_reference(&self, symbol_name: &str) -> Option<String> {
        self.graph
            .symbol_index
            .get(symbol_name)
            .and_then(|ids| ids.iter().next().cloned())
    }

    /// Convert relation kind to graph relation type
    fn convert_relation_type(&self, kind: &crate::engine::parsing::RelationKind) -> RelationType {
        use crate::engine::parsing::RelationKind;
        match kind {
            RelationKind::Imports => RelationType::Imports,
            RelationKind::Extends => RelationType::Extends,
            RelationKind::Implements => RelationType::Implements,
            RelationKind::Uses => RelationType::Uses,
            RelationKind::Calls => RelationType::Calls,
        }
    }
}

impl KnowledgeGraph {
    /// Get all nodes in the graph
    pub fn nodes(&self) -> &HashMap<String, GraphNode> {
        &self.nodes
    }

    /// Get all edges from a node
    pub fn edges_from(&self, node_id: &str) -> Option<&Vec<GraphRelation>> {
        self.edges.get(node_id)
    }

    /// Find nodes by symbol name
    pub fn find_by_symbol(&self, symbol_name: &str) -> Vec<&GraphNode> {
        if let Some(node_ids) = self.symbol_index.get(symbol_name) {
            node_ids
                .iter()
                .filter_map(|id| self.nodes.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.values().map(|v| v.len()).sum(),
            symbol_count: self.symbol_index.len(),
        }
    }
}

/// Knowledge graph statistics
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub symbol_count: usize,
}

/// Error type for graph building operations
#[derive(Debug)]
pub enum GraphBuildError {
    /// Cache operation failed
    CacheError(String),
    /// Symbol resolution failed
    SymbolResolutionError(String),
    /// Invalid graph structure
    InvalidGraph(String),
}

impl std::fmt::Display for GraphBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphBuildError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            GraphBuildError::SymbolResolutionError(msg) => write!(f, "Symbol resolution error: {}", msg),
            GraphBuildError::InvalidGraph(msg) => write!(f, "Invalid graph: {}", msg),
        }
    }
}

impl std::error::Error for GraphBuildError {}

#[cfg(feature = "analysis-cache")]
impl GraphCacheManager {
    /// Create a new graph cache manager
    pub fn new() -> Self {
        Self {
            relation_cache: HashMap::new(),
            symbol_resolution_cache: HashMap::new(),
            stats_cache: HashMap::new(),
            metrics: CacheMetricsCollector::new(),
        }
    }

    /// Get cached relations for a file
    pub fn get_cached_relations(&self, cache_key: &str) -> Option<&Vec<GraphRelation>> {
        self.relation_cache.get(cache_key)
    }

    /// Cache relations for a file
    pub fn cache_relations(&mut self, cache_key: String, relations: Vec<GraphRelation>) {
        self.relation_cache.insert(cache_key, relations);
    }

    /// Get cached symbol resolution
    pub fn get_cached_symbol_resolution(&self, symbol: &str) -> Option<&Option<String>> {
        self.symbol_resolution_cache.get(symbol)
    }

    /// Cache symbol resolution
    pub fn cache_symbol_resolution(&mut self, symbol: String, resolution: Option<String>) {
        self.symbol_resolution_cache.insert(symbol, resolution);
    }

    /// Get cached statistics
    pub fn get_cached_stats(&self, cache_key: &str) -> Option<&GraphStats> {
        self.stats_cache.get(cache_key)
    }

    /// Cache statistics
    pub fn cache_stats(&mut self, cache_key: String, stats: GraphStats) {
        self.stats_cache.insert(cache_key, stats);
    }

    /// Clear all caches
    pub fn clear(&mut self) {
        self.relation_cache.clear();
        self.symbol_resolution_cache.clear();
        self.stats_cache.clear();
    }

    /// Get cache metrics
    pub fn get_metrics(&self) -> &CacheMetricsCollector {
        &self.metrics
    }
}
