//! # Knowledge Graph Builder
//!
//! Incremental knowledge graph construction from parsed files and symbols.
//! Target: <400 lines total

use super::relations::{GraphRelation, RelationType};
use crate::engine::parsing::{Relation, Symbol};
use std::collections::{HashMap, HashSet};

/// Knowledge graph containing all code relationships
#[derive(Debug, Default)]
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
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            graph: KnowledgeGraph::default(),
        }
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
#[derive(Debug)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub symbol_count: usize,
}
