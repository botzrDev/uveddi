use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

use super::super::types::{CouplingMetrics, Dependency, DependencyStrength};

/// Visualizes dependency relationships as a graph structure
#[derive(Debug, Clone)]
pub struct DependencyGraphVisualizer;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub metrics: Option<CouplingMetrics>,
    pub size: f64,     // Based on coupling metrics
    pub color: String, // Based on coupling severity
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub strength: DependencyStrength,
    pub weight: f64,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct DependencyGraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub clusters: HashMap<String, Vec<String>>, // Group nodes by module/package
    pub statistics: GraphStatistics,
}

#[derive(Debug, Clone, Default)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub max_coupling: usize,
    pub avg_coupling: f64,
    pub isolated_nodes: usize,
    pub strongly_connected_components: usize,
}

impl DependencyGraphVisualizer {
    pub fn new() -> Self {
        Self
    }

    /// Generate graph data for visualization
    pub fn generate_graph_data(
        &self,
        graph: &LocalDependencyGraph,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        dependencies: &[Dependency],
    ) -> DependencyGraphData {
        debug!("Generating dependency graph visualization data");

        let nodes = self.create_graph_nodes(graph, metrics);
        let edges = self.create_graph_edges(dependencies);
        let clusters = self.identify_clusters(&nodes);
        let statistics = self.calculate_graph_statistics(&nodes, &edges, metrics);

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
                dot.push_str(&format!("  subgraph cluster_{} {{\n", cluster_name.replace("::", "_")));
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
                    label.push_str(&format!(
                        "\\nCBO: {}, RFC: {}",
                        metrics.cbo, metrics.rfc
                    ));
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
                DependencyStrength::Weak => "dotted",
                DependencyStrength::Medium => "solid",
                DependencyStrength::Strong => "bold",
            };

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [style=\"{}\", color=\"{}\", penwidth={}];\n",
                edge.from,
                edge.to,
                style,
                edge.color,
                edge.weight
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

            mermaid.push_str(&format!(
                "  {} {} {}\n",
                edge.from, arrow, edge.to
            ));
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
        let mut nodes_with_metrics: Vec<_> = graph_data
            .nodes
            .iter()
            .filter(|node| node.metrics.is_some())
            .collect();

        nodes_with_metrics.sort_by(|a, b| {
            let a_cbo = a.metrics.as_ref().unwrap().cbo;
            let b_cbo = b.metrics.as_ref().unwrap().cbo;
            b_cbo.cmp(&a_cbo)
        });

        let hotspot_count = ((nodes_with_metrics.len() as f64) * (1.0 - threshold_percentile)).max(1.0) as usize;

        nodes_with_metrics
            .into_iter()
            .take(hotspot_count)
            .cloned()
            .collect()
    }

    fn create_graph_nodes(
        &self,
        graph: &LocalDependencyGraph,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
    ) -> Vec<GraphNode> {
        let mut nodes = Vec::new();
        let petgraph = graph.get_petgraph();

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                let node_id = self.component_to_id(component);
                let label = self.component_to_label(component);
                let node_type = self.component_to_type(component);

                let component_metrics = metrics.get(component).cloned();
                let size = self.calculate_node_size(&component_metrics);
                let color = self.calculate_node_color(&component_metrics);

                nodes.push(GraphNode {
                    id: node_id,
                    label,
                    node_type,
                    metrics: component_metrics,
                    size,
                    color,
                });
            }
        }

        nodes
    }

    fn create_graph_edges(&self, dependencies: &[Dependency]) -> Vec<GraphEdge> {
        let mut edges = Vec::new();

        for dep in dependencies {
            let from = self.component_to_id(&dep.from_component);
            let to = self.component_to_id(&dep.to_component);
            let edge_type = format!("{:?}", dep.dependency_type);
            let weight = self.calculate_edge_weight(&dep.strength);
            let color = self.calculate_edge_color(&dep.strength);

            edges.push(GraphEdge {
                from,
                to,
                edge_type,
                strength: dep.strength.clone(),
                weight,
                color,
            });
        }

        edges
    }

    fn identify_clusters(&self, nodes: &[GraphNode]) -> HashMap<String, Vec<String>> {
        let mut clusters: HashMap<String, Vec<String>> = HashMap::new();

        for node in nodes {
            // Group by module/package based on node ID structure
            let cluster_name = if node.id.contains("::") {
                node.id.split("::").next().unwrap_or("default").to_string()
            } else if node.id.contains("/") {
                let parts: Vec<&str> = node.id.split("/").collect();
                if parts.len() > 1 {
                    parts[..parts.len() - 1].join("/")
                } else {
                    "default".to_string()
                }
            } else {
                "default".to_string()
            };

            clusters
                .entry(cluster_name)
                .or_insert_with(Vec::new)
                .push(node.id.clone());
        }

        clusters
    }

    fn calculate_graph_statistics(
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

        let isolated_nodes = nodes.iter().filter(|node| {
            !edges.iter().any(|edge| edge.from == node.id || edge.to == node.id)
        }).count();

        GraphStatistics {
            total_nodes,
            total_edges,
            max_coupling,
            avg_coupling,
            isolated_nodes,
            strongly_connected_components: 0, // Would need SCC algorithm
        }
    }

    fn component_to_id(&self, component: &ComponentNode) -> String {
        match component {
            ComponentNode::Class { name, file_path } => {
                format!("class_{}_{}", name, file_path.replace("/", "_").replace(".", "_"))
            }
            ComponentNode::Function { name, file_path } => {
                format!("fn_{}_{}", name, file_path.replace("/", "_").replace(".", "_"))
            }
            ComponentNode::Module { path } => {
                format!("mod_{}", path.replace("/", "_").replace(".", "_"))
            }
        }
    }

    fn component_to_label(&self, component: &ComponentNode) -> String {
        match component {
            ComponentNode::Class { name, .. } => name.clone(),
            ComponentNode::Function { name, .. } => name.clone(),
            ComponentNode::Module { path } => {
                path.split("/").last().unwrap_or(path).to_string()
            }
        }
    }

    fn component_to_type(&self, component: &ComponentNode) -> String {
        match component {
            ComponentNode::Class { .. } => "Class".to_string(),
            ComponentNode::Function { .. } => "Function".to_string(),
            ComponentNode::Module { .. } => "Module".to_string(),
        }
    }

    fn calculate_node_size(&self, metrics: &Option<CouplingMetrics>) -> f64 {
        if let Some(m) = metrics {
            // Size based on CBO (Coupling Between Objects)
            (m.cbo as f64 / 20.0).min(3.0).max(0.5)
        } else {
            1.0
        }
    }

    fn calculate_node_color(&self, metrics: &Option<CouplingMetrics>) -> String {
        if let Some(m) = metrics {
            if m.cbo >= 15 {
                "red".to_string()      // High coupling
            } else if m.cbo >= 8 {
                "orange".to_string()   // Medium coupling
            } else if m.cbo >= 3 {
                "yellow".to_string()   // Low coupling
            } else {
                "lightgreen".to_string() // Very low coupling
            }
        } else {
            "lightblue".to_string()
        }
    }

    fn calculate_edge_weight(&self, strength: &DependencyStrength) -> f64 {
        match strength {
            DependencyStrength::Weak => 1.0,
            DependencyStrength::Medium => 2.0,
            DependencyStrength::Strong => 3.0,
        }
    }

    fn calculate_edge_color(&self, strength: &DependencyStrength) -> String {
        match strength {
            DependencyStrength::Weak => "gray".to_string(),
            DependencyStrength::Medium => "blue".to_string(),
            DependencyStrength::Strong => "red".to_string(),
        }
    }
}

impl Default for DependencyGraphVisualizer {
    fn default() -> Self {
        Self::new()
    }
}