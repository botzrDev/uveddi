//! Knowledge graph security detector module
//!
//! This module implements a sophisticated knowledge graph that serves as the central
//! "brain" for deep architectural analysis. It combines formal graph relationships,
//! vector-based semantic search, and keyword-based full-text search to provide
//! contextually rich information for security analysis.
//!
//! ## Architecture
//!
//! The knowledge graph contains:
//! - **Structural Facts**: ASTs, call graphs, dependency graphs
//! - **Semantic Enrichments**: LLM-generated summaries, conceptual links
//! - **Security Information**: Vulnerabilities, anti-patterns, correlations
//! - **Long-Term Memory**: Historical analysis results and patterns

pub mod config;
pub mod detector;
pub mod graph;
pub mod knowledge;
pub mod security;
pub mod types;
pub mod language_support;

// Re-export main public interfaces
pub use config::KnowledgeGraphConfig;
pub use detector::KnowledgeGraphDetector;
pub use types::{
    SecurityKnowledgeResult, SecurityQuery, ArchitecturalCorrelation,
    CodeEntity, FileAnalysis, StructuralSemanticGraph,
    StructuralQueryResult, SemanticQueryResult, RAGQueryResult,
};

use crate::analysis::detectors::security::types::{SecurityIssue, SecurityIssueType};
use crate::analysis::AnalysisError;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

/// Knowledge graph builder for constructing the graph from codebase analysis
pub struct KnowledgeGraphBuilder {
    detector: KnowledgeGraphDetector,
}

impl KnowledgeGraphBuilder {
    /// Create a new knowledge graph builder
    pub fn new() -> Result<Self, AnalysisError> {
        let config = KnowledgeGraphConfig::default();
        let detector = KnowledgeGraphDetector::new(config);

        Ok(Self { detector })
    }

    /// Create builder with custom configuration
    pub fn with_config(config: KnowledgeGraphConfig) -> Result<Self, AnalysisError> {
        let detector = KnowledgeGraphDetector::new(config);
        Ok(Self { detector })
    }

    /// Build knowledge graph from codebase analysis results
    pub async fn build_from_codebase(
        &mut self,
        file_paths: Vec<PathBuf>,
    ) -> Result<SecurityKnowledgeResult, AnalysisError> {
        info!("Building knowledge graph from {} files", file_paths.len());

        self.detector.analyze_files(&file_paths).await
    }

    /// Query the built knowledge graph (simplified)
    pub async fn query_security_context(
        &self,
        _query: SecurityQuery,
    ) -> Result<SecurityKnowledgeResult, AnalysisError> {
        Ok(SecurityKnowledgeResult {
            structural_facts: StructuralQueryResult::default(),
            semantic_insights: SemanticQueryResult::default(),
            contextual_information: RAGQueryResult::default(),
            historical_patterns: Vec::new(),
            confidence_score: 0.7,
        })
    }

    /// Correlate security issues with architectural anti-patterns (simplified)
    pub async fn correlate_with_architecture(
        &self,
        _security_issue: &SecurityIssue,
    ) -> Result<ArchitecturalCorrelation, AnalysisError> {
        Ok(ArchitecturalCorrelation {
            security_issue_id: "none".to_string(),
            architectural_patterns: Vec::new(),
            correlation_strength: 0.0,
            amplification_factors: HashMap::new(),
            recommendations: Vec::new(),
        })
    }

    /// Get the underlying detector for advanced operations
    pub fn detector(&self) -> &KnowledgeGraphDetector {
        &self.detector
    }
}

impl Default for KnowledgeGraphBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create default KnowledgeGraphBuilder")
    }
}

/// Main security knowledge graph that integrates structural and semantic information
pub struct SecurityKnowledgeGraph {
    builder: KnowledgeGraphBuilder,
    current_result: Option<SecurityKnowledgeResult>,
}

impl SecurityKnowledgeGraph {
    /// Create a new security knowledge graph
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            builder: KnowledgeGraphBuilder::new()?,
            current_result: None,
        })
    }

    /// Build the graph from file paths
    pub async fn build_from_files(&mut self, file_paths: Vec<PathBuf>) -> Result<(), AnalysisError> {
        let result = self.builder.build_from_codebase(file_paths).await?;
        self.current_result = Some(result);
        Ok(())
    }

    /// Query the knowledge graph for security-relevant context
    pub async fn query_security_context(
        &self,
        query: SecurityQuery,
    ) -> Result<SecurityKnowledgeResult, AnalysisError> {
        self.builder.query_security_context(query).await
    }

    /// Get current analysis results
    pub fn get_current_results(&self) -> Option<&SecurityKnowledgeResult> {
        self.current_result.as_ref()
    }

    /// Correlate security issue with architectural patterns
    pub async fn correlate_with_architecture(
        &self,
        security_issue: &SecurityIssue,
    ) -> Result<ArchitecturalCorrelation, AnalysisError> {
        self.builder.correlate_with_architecture(security_issue).await
    }

    /// Get summary of extracted security issues
    pub fn get_security_issues(&self) -> Vec<SecurityIssue> {
        if let Some(result) = &self.current_result {
            // Extract security issues from the knowledge graph results
            // This would be implemented based on how issues are stored in the result
            Vec::new() // Placeholder
        } else {
            Vec::new()
        }
    }

    /// Get confidence score for the analysis
    pub fn get_confidence_score(&self) -> f64 {
        self.current_result
            .as_ref()
            .map(|result| result.confidence_score)
            .unwrap_or(0.0)
    }
}

impl Default for SecurityKnowledgeGraph {
    fn default() -> Self {
        Self::new().expect("Failed to create default SecurityKnowledgeGraph")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_knowledge_graph_creation() {
        let graph = SecurityKnowledgeGraph::new();
        assert!(graph.is_ok());
    }

    #[tokio::test]
    async fn test_knowledge_graph_builder() {
        let mut builder = KnowledgeGraphBuilder::new().unwrap();

        let result = builder.build_from_codebase(vec![]).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_knowledge_graph_config() {
        let config = KnowledgeGraphConfig::default();
        assert!(!config.enable_semantic_enrichment); // Default should be false for performance

        let builder = KnowledgeGraphBuilder::with_config(config);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_security_knowledge_graph_confidence() {
        let graph = SecurityKnowledgeGraph::new().unwrap();
        let confidence = graph.get_confidence_score();
        assert_eq!(confidence, 0.0); // Should be 0.0 before any analysis
    }

    #[tokio::test]
    async fn test_query_empty_graph() {
        let graph = SecurityKnowledgeGraph::new().unwrap();

        let query = SecurityQuery::PatternSearch {
            pattern_type: "test_pattern".to_string(),
        };

        let result = graph.query_security_context(query).await;
        assert!(result.is_ok());
    }
}