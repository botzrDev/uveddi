use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;

use super::super::types::{CouplingMetrics, Dependency, DependencyStrength};

/// Builds graph nodes and edges for visualization
pub struct GraphBuilder;

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

impl GraphBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Create graph nodes from components and metrics
    pub fn create_graph_nodes(
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

    /// Create graph edges from dependencies
    pub fn create_graph_edges(&self, dependencies: &[Dependency]) -> Vec<GraphEdge> {
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

    /// Identify clusters (groups of related nodes)
    pub fn identify_clusters(&self, nodes: &[GraphNode]) -> HashMap<String, Vec<String>> {
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

    fn component_to_id(&self, component: &ComponentNode) -> String {
        match component {
            ComponentNode::Class { name, file_path } => {
                format!(
                    "class_{}_{}",
                    name,
                    file_path.replace("/", "_").replace(".", "_")
                )
            }
            ComponentNode::Function { name, file_path } => {
                format!(
                    "fn_{}_{}",
                    name,
                    file_path.replace("/", "_").replace(".", "_")
                )
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
            ComponentNode::Module { path } => path.split("/").last().unwrap_or(path).to_string(),
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
                "red".to_string() // High coupling
            } else if m.cbo >= 8 {
                "orange".to_string() // Medium coupling
            } else if m.cbo >= 3 {
                "yellow".to_string() // Low coupling
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

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
