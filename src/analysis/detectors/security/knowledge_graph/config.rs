//! Configuration types for the knowledge graph system

use serde::{Deserialize, Serialize};

/// Configuration for the knowledge graph system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphConfig {
    /// Enable semantic enrichment via AI
    pub enable_semantic_enrichment: bool,
    /// Enable RAG-based contextual queries
    pub enable_rag: bool,
    /// Enable long-term memory persistence
    pub enable_memory: bool,
    /// Maximum entities to process in parallel
    pub max_parallel_entities: usize,
    /// Confidence threshold for results
    pub confidence_threshold: f64,
    /// Maximum context radius for queries
    pub max_context_radius: usize,
}

impl Default for KnowledgeGraphConfig {
    fn default() -> Self {
        Self {
            enable_semantic_enrichment: false,
            enable_rag: false,
            enable_memory: true,
            max_parallel_entities: 100,
            confidence_threshold: 0.5,
            max_context_radius: 10,
        }
    }
}

/// Graph construction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Include control flow edges
    pub include_control_flow: bool,
    /// Include data flow edges
    pub include_data_flow: bool,
    /// Maximum graph depth
    pub max_depth: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            include_control_flow: true,
            include_data_flow: true,
            max_depth: 50,
        }
    }
}
