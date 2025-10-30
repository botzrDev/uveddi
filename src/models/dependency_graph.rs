//! Dependency graph model for representing code dependencies.
//!
//! This module provides a simple dependency graph structure for tracking relationships between code entities.
//!
//! Used for analysis, visualization, and cycle detection in Uveddi.
use petgraph::Directed;
use petgraph::Graph;
use serde::{Deserialize, Serialize};

/// Represents a dependency graph with nodes and edges.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DependencyGraph {
    /// List of node names (e.g., files, modules, components).
    pub nodes: Vec<String>,
    /// List of directed edges as (from, to) pairs.
    pub edges: Vec<(String, String)>,
}

impl DependencyGraph {
    /// Creates a new, empty `DependencyGraph`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the dependency graph if it doesn't already exist.
    pub fn add_node(&mut self, node: String) {
        if !self.nodes.contains(&node) {
            self.nodes.push(node);
        }
    }

    /// Adds an edge between two nodes in the dependency graph.
    pub fn add_edge(&mut self, from: String, to: String) {
        self.add_node(from.clone());
        self.add_node(to.clone());
        self.edges.push((from, to));
    }

    /// Converts the `DependencyGraph` into a `petgraph::Graph` for cycle detection.
    pub fn get_petgraph(&self) -> Graph<&str, (), Directed> {
        let mut graph = Graph::<&str, (), Directed>::new();
        let node_indices: Vec<_> = self
            .nodes
            .iter()
            .map(|node| graph.add_node(node.as_str()))
            .collect();
        for (from, to) in &self.edges {
            if let (Some(from_idx), Some(to_idx)) = (
                self.nodes.iter().position(|n| n == from),
                self.nodes.iter().position(|n| n == to),
            ) {
                graph.add_edge(node_indices[from_idx], node_indices[to_idx], ());
            }
        }
        graph
    }

    /// Retrieves the node string based on its index in the graph.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the node in the internal node list.
    ///
    /// # Returns
    ///
    /// * `Some(&String)` - Reference to the node string if the index is valid.
    /// * `None` - If the index is out of bounds.
    ///
    /// # Example
    /// ```rust
    /// use uveddi::models::dependency_graph::DependencyGraph;
    /// let graph = DependencyGraph::default();
    /// let node = graph.get_node_from_index(0);
    /// ```
    pub fn get_node_from_index(&self, index: usize) -> Option<&String> {
        self.nodes.get(index)
    }
}
