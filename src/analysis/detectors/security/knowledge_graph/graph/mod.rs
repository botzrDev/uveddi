//! Graph construction and analysis components
//!
//! This module provides the core graph functionality for the knowledge graph system,
//! including construction, analysis, and traversal capabilities.

pub mod analyzer;
pub mod builder;
pub mod traversal;

pub use analyzer::GraphAnalyzer;
pub use builder::GraphBuilder;
pub use traversal::{GraphTraversal, SecurityTraversal};

use crate::analysis::detectors::security::knowledge_graph::types::{
    AntiPatternInfo, FileAnalysis, StructuralSemanticGraph,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashSet;
use tracing::info;

/// Main graph management interface
pub struct GraphManager {
    graph: StructuralSemanticGraph,
}

impl GraphManager {
    /// Create a new graph manager
    pub fn new(language: SourceLanguage) -> Self {
        Self {
            graph: StructuralSemanticGraph::new(language),
        }
    }

    /// Build graph from file analyses
    pub async fn build_from_analyses(
        &mut self,
        analyses: Vec<FileAnalysis>,
    ) -> Result<(), AnalysisError> {
        info!("Building graph from {} analyses", analyses.len());

        let mut builder = GraphBuilder::new(self.graph.metadata.language.clone());
        builder.build_from_analyses(analyses).await?;
        self.graph = builder.into_graph();

        Ok(())
    }

    /// Analyze the graph for security patterns
    pub async fn analyze_security_patterns(&self) -> Result<Vec<AntiPatternInfo>, AnalysisError> {
        let mut analyzer = GraphAnalyzer::new(&self.graph);
        analyzer.analyze_security_patterns().await
    }

    /// Get graph traversal interface
    pub fn traversal(&self) -> GraphTraversal {
        GraphTraversal::new(&self.graph)
    }

    /// Get security-focused traversal interface
    pub fn security_traversal(&self) -> SecurityTraversal {
        SecurityTraversal::new(&self.graph)
    }

    /// Get reference to the underlying graph
    pub fn graph(&self) -> &StructuralSemanticGraph {
        &self.graph
    }

    /// Get graph statistics
    pub fn get_statistics(&self) -> GraphStatistics {
        let node_types = self
            .graph
            .nodes
            .values()
            .map(|node| std::mem::discriminant(&node.node_type))
            .collect::<HashSet<_>>();

        let edge_types = self
            .graph
            .edges
            .iter()
            .map(|edge| std::mem::discriminant(&edge.edge_type))
            .collect::<HashSet<_>>();

        GraphStatistics {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            node_type_count: node_types.len(),
            edge_type_count: edge_types.len(),
            creation_time: self.graph.metadata.creation_time,
            language: self.graph.metadata.language.clone(),
        }
    }
}

/// Statistics about the graph structure
#[derive(Debug, Clone)]
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub node_type_count: usize,
    pub edge_type_count: usize,
    pub creation_time: chrono::DateTime<chrono::Utc>,
    pub language: SourceLanguage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::knowledge_graph::types::{
        CodeEntity, CodeLocation, EntityType,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_graph_manager_creation() {
        let manager = GraphManager::new(SourceLanguage::Rust);
        let stats = manager.get_statistics();
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.edge_count, 0);
    }

    #[tokio::test]
    async fn test_graph_building() {
        let mut manager = GraphManager::new(SourceLanguage::Rust);

        let file_analysis = FileAnalysis {
            file_path: PathBuf::from("test.rs"),
            entities: vec![CodeEntity {
                id: "test_func".to_string(),
                name: "test_function".to_string(),
                entity_type: EntityType::Function,
                location: CodeLocation {
                    file_path: PathBuf::from("test.rs"),
                    start_line: 1,
                    end_line: 10,
                    start_column: 0,
                    end_column: 0,
                },
                metadata: HashMap::new(),
                language: SourceLanguage::Rust,
            }],
            security_issues: Vec::new(),
            anti_patterns: Vec::new(),
        };

        let result = manager.build_from_analyses(vec![file_analysis]).await;
        assert!(result.is_ok());

        let stats = manager.get_statistics();
        assert_eq!(stats.node_count, 1);
    }
}
