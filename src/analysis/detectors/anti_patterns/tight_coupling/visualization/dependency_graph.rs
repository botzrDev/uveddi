use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

use super::super::types::{CouplingMetrics, Dependency};
use super::{
    graph_analyzer::{GraphAnalyzer, GraphStatistics},
    graph_builder::{GraphBuilder, GraphEdge, GraphNode},
};

/// Visualizes dependency relationships as a graph structure
#[derive(Debug, Clone)]
pub struct DependencyGraphVisualizer {
    builder: GraphBuilder,
    analyzer: GraphAnalyzer,
}

#[derive(Debug, Clone)]
pub struct DependencyGraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub clusters: HashMap<String, Vec<String>>, // Group nodes by module/package
    pub statistics: GraphStatistics,
}

impl DependencyGraphVisualizer {
    pub fn new() -> Self {
        Self {
            builder: GraphBuilder::new(),
            analyzer: GraphAnalyzer::new(),
        }
    }

    /// Generate graph data for visualization
    pub fn generate_graph_data(
        &self,
        graph: &LocalDependencyGraph,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        dependencies: &[Dependency],
    ) -> DependencyGraphData {
        debug!("Generating dependency graph visualization data");

        let nodes = self.builder.create_graph_nodes(graph, metrics);
        let edges = self.builder.create_graph_edges(dependencies);
        let clusters = self.builder.identify_clusters(&nodes);
        let statistics = self
            .analyzer
            .calculate_graph_statistics(&nodes, &edges, metrics);

        DependencyGraphData {
            nodes,
            edges,
            clusters,
            statistics,
        }
    }

    /// Generate DOT format for Graphviz rendering
    pub fn generate_dot_format(
        &self,
        graph_data: &DependencyGraphData,
        include_metrics: bool,
    ) -> String {
        let mut dot = String::from("digraph DependencyGraph {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=filled];\n");
        dot.push_str("  edge [arrowhead=open];\n\n");

        // Add clusters (subgraphs)
        for (cluster_name, node_ids) in &graph_data.clusters {
            if node_ids.len() > 1 {
                dot.push_str(&format!(
                    "  subgraph cluster_{} {{\n",
                    cluster_name.replace("::", "_")
                ));
                dot.push_str(&format!("    label=\"{}\";\n", cluster_name));
                dot.push_str("    style=dashed;\n");

                for node_id in node_ids {
                    dot.push_str(&format!("    \"{}\";\n", node_id));
                }

                dot.push_str("  }\n\n");
            }
        }

        // Add nodes
        for node in &graph_data.nodes {
            let mut label = node.label.clone();
            if include_metrics {
                if let Some(metrics) = &node.metrics {
                    label.push_str(&format!("\\nCBO: {}, RFC: {}", metrics.cbo, metrics.rfc));
                }
            }

            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", fillcolor=\"{}\", fontsize={}];\n",
                node.id,
                label,
                node.color,
                (8.0 + node.size * 4.0).min(16.0)
            ));
        }

        dot.push_str("\n");

        // Add edges
        for edge in &graph_data.edges {
            let style = match edge.strength {
                super::super::types::DependencyStrength::Weak => "dotted",
                super::super::types::DependencyStrength::Medium => "solid",
                super::super::types::DependencyStrength::Strong => "bold",
            };

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [style=\"{}\", color=\"{}\", penwidth={}];\n",
                edge.from, edge.to, style, edge.color, edge.weight
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Generate Mermaid format for web-based visualization
    pub fn generate_mermaid_format(&self, graph_data: &DependencyGraphData) -> String {
        let mut mermaid = String::from("graph TD\n");

        // Add nodes with styling
        for node in &graph_data.nodes {
            let shape = match node.node_type.as_str() {
                "Class" => format!("{}[{}]", node.id, node.label),
                "Function" => format!("{}({})", node.id, node.label),
                "Module" => format!("{}[{}]", node.id, node.label),
                _ => format!("{}[{}]", node.id, node.label),
            };

            mermaid.push_str(&format!("  {}\n", shape));
        }

        mermaid.push_str("\n");

        // Add edges
        for edge in &graph_data.edges {
            let arrow = match edge.edge_type.as_str() {
                "Import" => "-->",
                "Call" => "-.->",
                "Inheritance" => "==>",
                "Implementation" => "o--o",
                _ => "-->",
            };

            mermaid.push_str(&format!("  {} {} {}\n", edge.from, arrow, edge.to));
        }

        // Add styling
        mermaid.push_str("\n");
        for node in &graph_data.nodes {
            let class_name = match node.color.as_str() {
                "red" => "high-coupling",
                "orange" => "medium-coupling",
                "yellow" => "low-coupling",
                _ => "default",
            };
            mermaid.push_str(&format!("  class {} {}\n", node.id, class_name));
        }

        mermaid
    }

    /// Identify coupling hotspots in the graph
    pub fn identify_hotspots(
        &self,
        graph_data: &DependencyGraphData,
        threshold_percentile: f64,
    ) -> Vec<GraphNode> {
        self.analyzer
            .identify_hotspots(&graph_data.nodes, threshold_percentile)
    }
}

impl Default for DependencyGraphVisualizer {
    fn default() -> Self {
        Self::new()
    }
}
