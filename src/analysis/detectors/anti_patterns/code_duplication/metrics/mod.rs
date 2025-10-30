//! Metrics and similarity calculation for code duplication detection

pub mod similarity_calculator;
pub mod threshold_manager;

pub use similarity_calculator::SimilarityCalculator;
pub use threshold_manager::ThresholdManager;

use serde::{Deserialize, Serialize};

/// Combined metrics for code duplication analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationMetrics {
    /// Token-based similarity metrics
    pub token_metrics: TokenMetrics,
    /// AST-based similarity metrics
    pub ast_metrics: AstMetrics,
    /// Semantic similarity metrics (if available)
    pub semantic_metrics: Option<SemanticMetrics>,
    /// Overall combined score
    pub combined_score: f64,
    /// Confidence level of the detection
    pub confidence: f64,
}

/// Token-level similarity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    /// Jaccard similarity coefficient
    pub jaccard_similarity: f64,
    /// Cosine similarity
    pub cosine_similarity: f64,
    /// Edit distance normalized by length
    pub normalized_edit_distance: f64,
    /// Number of matching token sequences
    pub matching_sequences: usize,
    /// Total token count comparison
    pub token_ratio: f64,
}

/// AST-level similarity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstMetrics {
    /// Structural similarity score
    pub structural_similarity: f64,
    /// Tree edit distance
    pub tree_edit_distance: f64,
    /// Node type distribution similarity
    pub type_distribution_similarity: f64,
    /// Depth similarity
    pub depth_similarity: f64,
}

/// Semantic similarity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMetrics {
    /// Control flow similarity
    pub cfg_similarity: f64,
    /// Data flow similarity
    pub data_flow_similarity: f64,
    /// Semantic feature similarity
    pub feature_similarity: f64,
    /// Behavioral equivalence score
    pub behavioral_score: f64,
}
