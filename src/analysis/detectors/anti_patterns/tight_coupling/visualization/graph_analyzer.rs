use std::collections::HashMap;

use super::super::types::CouplingMetrics;
use super::graph_builder::{GraphEdge, GraphNode};
use crate::analysis::graph::dependency::ComponentNode;

/// Analyzes graph structure and generates statistics
pub struct GraphAnalyzer;

#[derive(Debug, Clone, Default)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub max_coupling: usize,
    pub avg_coupling: f64,
    pub isolated_nodes: usize,
    pub strongly_connected_components: usize,
}

impl GraphAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Calculate comprehensive graph statistics
    pub fn calculate_graph_statistics(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
    ) -> GraphStatistics {
        let total_nodes = nodes.len();
        let total_edges = edges.len();

        let couplings: Vec<usize> = metrics.values().map(|m| m.cbo).collect();
        let max_coupling = couplings.iter().max().cloned().unwrap_or(0);
        let avg_coupling = if couplings.is_empty() {
            0.0
        } else {
            couplings.iter().sum::<usize>() as f64 / couplings.len() as f64
        };

        let isolated_nodes = nodes
            .iter()
            .filter(|node| {
                !edges
                    .iter()
                    .any(|edge| edge.from == node.id || edge.to == node.id)
            })
            .count();

        GraphStatistics {
            total_nodes,
            total_edges,
            max_coupling,
            avg_coupling,
            isolated_nodes,
            strongly_connected_components: 0, // Would need SCC algorithm
        }
    }

    /// Identify coupling hotspots in the graph
    pub fn identify_hotspots(
        &self,
        nodes: &[GraphNode],
        threshold_percentile: f64,
    ) -> Vec<GraphNode> {
        let mut nodes_with_metrics: Vec<_> =
            nodes.iter().filter(|node| node.metrics.is_some()).collect();

        nodes_with_metrics.sort_by(|a, b| {
            let a_cbo = a.metrics.as_ref().unwrap().cbo;
            let b_cbo = b.metrics.as_ref().unwrap().cbo;
            b_cbo.cmp(&a_cbo)
        });

        let hotspot_count =
            ((nodes_with_metrics.len() as f64) * (1.0 - threshold_percentile)).max(1.0) as usize;

        nodes_with_metrics
            .into_iter()
            .take(hotspot_count)
            .cloned()
            .collect()
    }

    /// Analyze node connectivity patterns
    pub fn analyze_connectivity_patterns(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
    ) -> ConnectivityAnalysis {
        let total_nodes = nodes.len();
        let total_edges = edges.len();

        // Count nodes by type
        let mut node_types = HashMap::new();
        for node in nodes {
            *node_types.entry(node.node_type.clone()).or_insert(0) += 1;
        }

        // Count edges by type
        let mut edge_types = HashMap::new();
        for edge in edges {
            *edge_types.entry(edge.edge_type.clone()).or_insert(0) += 1;
        }

        // Calculate density
        let max_possible_edges = if total_nodes > 1 {
            total_nodes * (total_nodes - 1)
        } else {
            0
        };
        let density = if max_possible_edges > 0 {
            total_edges as f64 / max_possible_edges as f64
        } else {
            0.0
        };

        ConnectivityAnalysis {
            total_nodes,
            total_edges,
            density,
            node_types,
            edge_types,
        }
    }

    /// Find the most connected nodes (highest degree)
    pub fn find_most_connected_nodes(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
        top_n: usize,
    ) -> Vec<(String, usize)> {
        let mut node_degrees: HashMap<String, usize> = HashMap::new();

        // Count incoming and outgoing edges for each node
        for edge in edges {
            *node_degrees.entry(edge.from.clone()).or_insert(0) += 1;
            *node_degrees.entry(edge.to.clone()).or_insert(0) += 1;
        }

        let mut sorted_nodes: Vec<_> = node_degrees.into_iter().collect();
        sorted_nodes.sort_by(|a, b| b.1.cmp(&a.1));

        sorted_nodes.into_iter().take(top_n).collect()
    }
}

/// Analysis of graph connectivity patterns
#[derive(Debug, Clone)]
pub struct ConnectivityAnalysis {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub density: f64,
    pub node_types: HashMap<String, usize>,
    pub edge_types: HashMap<String, usize>,
}

impl Default for GraphAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
