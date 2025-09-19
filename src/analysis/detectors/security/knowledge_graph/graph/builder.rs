//! Graph construction for knowledge graph (Simplified)

use crate::analysis::detectors::security::knowledge_graph::types::{
    StructuralSemanticGraph, GraphNode, GraphEdge, GraphNodeType, GraphEdgeType,
    GraphMetadata, FileAnalysis, CodeEntity,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use tracing::debug;

/// Graph builder for constructing knowledge graphs
pub struct GraphBuilder {
    graph: StructuralSemanticGraph,
    entity_to_node: HashMap<String, String>,
}

impl GraphBuilder {
    pub fn new(language: SourceLanguage) -> Self {
        let metadata = GraphMetadata {
            creation_time: chrono::Utc::now(),
            language,
            version: "0.1.0".to_string(),
        };

        Self {
            graph: StructuralSemanticGraph::new(language),
            entity_to_node: HashMap::new(),
        }
    }

    /// Build graph from file analyses (simplified)
    pub async fn build_from_analyses(&mut self, analyses: Vec<FileAnalysis>) -> Result<(), AnalysisError> {
        debug!("Building graph from {} analyses", analyses.len());

        for analysis in analyses {
            self.process_file_analysis(analysis).await?;
        }

        Ok(())
    }

    /// Process a single file analysis (simplified)
    async fn process_file_analysis(&mut self, analysis: FileAnalysis) -> Result<(), AnalysisError> {
        // Add entities as nodes
        for entity in analysis.entities {
            self.add_entity_node(entity).await?;
        }

        Ok(())
    }

    /// Add entity as graph node (simplified)
    async fn add_entity_node(&mut self, entity: CodeEntity) -> Result<String, AnalysisError> {
        let node_id = format!("node_{}", entity.id);

        let node = GraphNode {
            id: node_id.clone(),
            node_type: GraphNodeType::CodeEntity,
            code_entity: Some(entity),
            properties: HashMap::new(),
        };

        self.graph.nodes.push(node);
        debug!("Added node: {}", node_id);

        Ok(node_id)
    }

    /// Add edge between nodes (simplified)
    pub fn add_edge(
        &mut self,
        from_node_id: String,
        to_node_id: String,
        edge_type: GraphEdgeType,
    ) -> Result<(), AnalysisError> {
        let edge = GraphEdge {
            id: format!("edge_{}_{}", from_node_id, to_node_id),
            from: from_node_id.clone(),
            to: to_node_id.clone(),
            edge_type: edge_type.clone(),
            weight: 1.0,
            properties: HashMap::new(),
        };

        self.graph.edges.push(edge);
        debug!("Added edge: {} -> {} ({:?})", from_node_id, to_node_id, edge_type.clone());

        Ok(())
    }

    /// Get the constructed graph
    pub fn into_graph(self) -> StructuralSemanticGraph {
        self.graph
    }

    /// Get reference to the graph
    pub fn graph(&self) -> &StructuralSemanticGraph {
        &self.graph
    }

    /// Find node by entity ID (simplified)
    pub fn find_node_by_entity_id(&self, entity_id: &str) -> Option<&GraphNode> {
        self.graph.nodes
            .iter()
            .find(|node| {
                if let Some(entity) = &node.code_entity {
                    entity.id == entity_id
                } else {
                    false
                }
            })
    }

    /// Get graph statistics
    pub fn get_statistics(&self) -> GraphStatistics {
        GraphStatistics {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            entity_count: self.entity_to_node.len(),
        }
    }
}

/// Graph construction statistics
#[derive(Debug, Clone)]
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub entity_count: usize,
}