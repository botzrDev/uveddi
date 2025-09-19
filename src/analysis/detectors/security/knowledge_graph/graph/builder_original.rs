//! Graph construction and building functionality
//!
//! This module handles the construction of knowledge graphs from codebase analysis results,
//! including entity extraction, relationship mapping, and graph structure optimization.

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, CodeRelationship, FileAnalysis, GraphEdge, GraphEdgeType, GraphNode,
    GraphNodeType, StructuralSemanticGraph,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use tracing::{debug, info};

/// Builder for constructing knowledge graphs from analysis results
pub struct GraphBuilder {
    graph: StructuralSemanticGraph,
    entity_index: HashMap<String, String>, // entity_name -> node_id
}

impl GraphBuilder {
    pub fn new(language: SourceLanguage) -> Self {
        Self {
            graph: StructuralSemanticGraph::new(language),
            entity_index: HashMap::new(),
        }
    }

    /// Build a graph from multiple file analyses
    pub async fn build_from_analyses(
        &mut self,
        analyses: Vec<FileAnalysis>,
    ) -> Result<(), AnalysisError> {
        info!("Building graph from {} file analyses", analyses.len());

        // First pass: Add all entities as nodes
        for analysis in &analyses {
            for entity in &analysis.entities {
                self.add_entity_node(entity).await?;
            }
        }

        // Second pass: Add relationships as edges
        for analysis in &analyses {
            self.add_file_relationships(&analysis.file_path, &analysis.entities)
                .await?;
        }

        // Third pass: Infer additional relationships
        self.infer_implicit_relationships().await?;

        info!("Graph construction completed with {} nodes and {} edges",
              self.graph.metadata.node_count, self.graph.metadata.edge_count);

        Ok(())
    }

    /// Add a code entity as a graph node
    async fn add_entity_node(&mut self, entity: &CodeEntity) -> Result<(), AnalysisError> {
        let node_id = format!("{}_{}", entity.entity_type_string(), entity.id);

        let mut properties = entity.metadata.clone();
        properties.insert("name".to_string(), entity.name.clone().into());
        properties.insert("file_path".to_string(),
                         entity.location.file_path.display().to_string().into());

        let node = GraphNode {
            id: node_id.clone(),
            node_type: self.convert_entity_to_node_type(&entity.entity_type),
            properties,
            location: Some(entity.location.clone()),
        };

        self.graph.add_node(node);
        self.entity_index.insert(entity.id.clone(), node_id);

        debug!("Added entity node: {} -> {}", entity.name, entity.id);
        Ok(())
    }

    /// Add relationships between entities in a file
    async fn add_file_relationships(
        &mut self,
        _file_path: &std::path::Path,
        entities: &[CodeEntity],
    ) -> Result<(), AnalysisError> {
        // Create containment relationships (modules contain functions, etc.)
        for entity in entities {
            if let Some(container) = self.find_container_for_entity(entity, entities) {
                self.add_relationship_edge(
                    &container.id,
                    &entity.id,
                    GraphEdgeType::Composition,
                    1.0,
                ).await?;
            }
        }

        // Add dependency relationships based on naming patterns
        self.add_dependency_relationships(entities).await?;

        Ok(())
    }

    /// Add a relationship edge between two entities
    async fn add_relationship_edge(
        &mut self,
        from_entity_id: &str,
        to_entity_id: &str,
        edge_type: GraphEdgeType,
        weight: f64,
    ) -> Result<(), AnalysisError> {
        if let (Some(from_node_id), Some(to_node_id)) = (
            self.entity_index.get(from_entity_id),
            self.entity_index.get(to_entity_id),
        ) {
            let edge = GraphEdge {
                from: from_node_id.clone(),
                to: to_node_id.clone(),
                edge_type,
                weight,
            };

            self.graph.add_edge(edge);
            debug!("Added edge: {} -> {} ({:?})", from_node_id, to_node_id, edge_type);
        }

        Ok(())
    }

    /// Find the container entity for a given entity (e.g., class for method)
    fn find_container_for_entity<'a>(
        &self,
        entity: &'a CodeEntity,
        all_entities: &'a [CodeEntity],
    ) -> Option<&'a CodeEntity> {
        all_entities.iter().find(|container| {
            container.location.file_path == entity.location.file_path
                && container.location.start_line <= entity.location.start_line
                && container.location.end_line >= entity.location.end_line
                && container.id != entity.id
                && self.is_container_type(&container.entity_type)
        })
    }

    /// Check if an entity type can contain other entities
    fn is_container_type(&self, entity_type: &crate::analysis::detectors::security::knowledge_graph::types::EntityType) -> bool {
        matches!(entity_type,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Class |
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Module |
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Struct |
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Interface
        )
    }

    /// Add dependency relationships based on naming and usage patterns
    async fn add_dependency_relationships(&mut self, entities: &[CodeEntity]) -> Result<(), AnalysisError> {
        for entity in entities {
            // Simple heuristic: if entity name appears in another entity's metadata,
            // assume there's a dependency
            for other_entity in entities {
                if entity.id != other_entity.id {
                    if self.entities_have_dependency(entity, other_entity) {
                        self.add_relationship_edge(
                            &other_entity.id,
                            &entity.id,
                            GraphEdgeType::Dependency,
                            0.7,
                        ).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Heuristic to determine if two entities have a dependency relationship
    fn entities_have_dependency(&self, entity: &CodeEntity, other_entity: &CodeEntity) -> bool {
        // Check if entity name appears in other entity's metadata
        other_entity.metadata.values().any(|value| {
            if let Ok(str_value) = serde_json::from_value::<String>(value.clone()) {
                str_value.contains(&entity.name)
            } else {
                false
            }
        })
    }

    /// Infer implicit relationships not captured in the first pass
    async fn infer_implicit_relationships(&mut self) -> Result<(), AnalysisError> {
        // Add call relationships based on function references
        self.infer_call_relationships().await?;

        // Add inheritance relationships where applicable
        self.infer_inheritance_relationships().await?;

        Ok(())
    }

    /// Infer function call relationships
    async fn infer_call_relationships(&mut self) -> Result<(), AnalysisError> {
        let function_nodes: Vec<_> = self.graph.nodes.values()
            .filter(|node| matches!(node.node_type, GraphNodeType::Function))
            .cloned()
            .collect();

        for caller in &function_nodes {
            for callee in &function_nodes {
                if caller.id != callee.id {
                    if self.nodes_have_call_relationship(caller, callee) {
                        let edge = GraphEdge {
                            from: caller.id.clone(),
                            to: callee.id.clone(),
                            edge_type: GraphEdgeType::CallsTo,
                            weight: 0.8,
                        };
                        self.graph.add_edge(edge);
                    }
                }
            }
        }

        Ok(())
    }

    /// Infer inheritance relationships
    async fn infer_inheritance_relationships(&mut self) -> Result<(), AnalysisError> {
        // Look for inheritance patterns in class/struct metadata
        let class_nodes: Vec<_> = self.graph.nodes.values()
            .filter(|node| matches!(node.node_type, GraphNodeType::Class))
            .cloned()
            .collect();

        for child in &class_nodes {
            for parent in &class_nodes {
                if child.id != parent.id {
                    if self.nodes_have_inheritance_relationship(child, parent) {
                        let edge = GraphEdge {
                            from: child.id.clone(),
                            to: parent.id.clone(),
                            edge_type: GraphEdgeType::Inheritance,
                            weight: 0.9,
                        };
                        self.graph.add_edge(edge);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if two nodes have a call relationship
    fn nodes_have_call_relationship(&self, caller: &GraphNode, callee: &GraphNode) -> bool {
        if let Some(callee_name) = callee.properties.get("name") {
            if let Ok(callee_name_str) = serde_json::from_value::<String>(callee_name.clone()) {
                return caller.properties.values().any(|value| {
                    if let Ok(str_value) = serde_json::from_value::<String>(value.clone()) {
                        str_value.contains(&callee_name_str)
                    } else {
                        false
                    }
                });
            }
        }
        false
    }

    /// Check if two nodes have an inheritance relationship
    fn nodes_have_inheritance_relationship(&self, child: &GraphNode, parent: &GraphNode) -> bool {
        // Look for inheritance indicators in metadata
        if let Some(parent_name) = parent.properties.get("name") {
            if let Ok(parent_name_str) = serde_json::from_value::<String>(parent_name.clone()) {
                return child.properties.get("extends")
                    .or_else(|| child.properties.get("implements"))
                    .or_else(|| child.properties.get("inherits"))
                    .map(|value| {
                        if let Ok(str_value) = serde_json::from_value::<String>(value.clone()) {
                            str_value.contains(&parent_name_str)
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false);
            }
        }
        false
    }

    /// Convert entity type to graph node type
    fn convert_entity_to_node_type(
        &self,
        entity_type: &crate::analysis::detectors::security::knowledge_graph::types::EntityType,
    ) -> GraphNodeType {
        match entity_type {
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Function => GraphNodeType::Function,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Class => GraphNodeType::Class,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Module => GraphNodeType::Module,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Variable => GraphNodeType::Variable,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Constant => GraphNodeType::Variable,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Interface => GraphNodeType::Class,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Struct => GraphNodeType::Class,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Enum => GraphNodeType::Class,
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Trait => GraphNodeType::Class,
        }
    }

    /// Get the constructed graph
    pub fn into_graph(self) -> StructuralSemanticGraph {
        self.graph
    }
}

impl CodeEntity {
    /// Get string representation of entity type
    fn entity_type_string(&self) -> &'static str {
        match self.entity_type {
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Function => "func",
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Class => "class",
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Module => "mod",
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Variable => "var",
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Constant => "const",
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Interface => "iface",
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Struct => "struct",
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Enum => "enum",
            crate::analysis::detectors::security::knowledge_graph::types::EntityType::Trait => "trait",
        }
    }
}