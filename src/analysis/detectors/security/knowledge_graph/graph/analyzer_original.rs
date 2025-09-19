//! Graph analysis functionality for knowledge graph
//!
//! This module provides analysis capabilities for the constructed knowledge graph,
//! including centrality measures, clustering, and security-relevant patterns.

use crate::analysis::detectors::security::knowledge_graph::types::{
    GraphEdge, GraphNode, StructuralSemanticGraph, AntiPatternInfo, CodeEntity,
};
use crate::analysis::AnalysisError;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Graph analyzer for knowledge graph analysis
pub struct GraphAnalyzer<'a> {
    graph: &'a StructuralSemanticGraph,
    centrality_cache: HashMap<String, f64>,
}

impl<'a> GraphAnalyzer<'a> {
    pub fn new(graph: &'a StructuralSemanticGraph) -> Self {
        Self {
            graph,
            centrality_cache: HashMap::new(),
        }
    }

    /// Analyze the graph for security-relevant patterns
    pub async fn analyze_security_patterns(&mut self) -> Result<Vec<AntiPatternInfo>, AnalysisError> {
        info!("Analyzing graph for security patterns");

        let mut patterns = Vec::new();

        // Detect high-risk central nodes
        patterns.extend(self.detect_high_centrality_risks().await?);

        // Detect tightly coupled components
        patterns.extend(self.detect_tight_coupling().await?);

        // Detect god objects/classes
        patterns.extend(self.detect_god_objects().await?);

        // Detect circular dependencies
        patterns.extend(self.detect_circular_dependencies().await?);

        info!("Found {} security-relevant patterns", patterns.len());
        Ok(patterns)
    }

    /// Calculate betweenness centrality for all nodes
    pub async fn calculate_betweenness_centrality(&mut self) -> Result<HashMap<String, f64>, AnalysisError> {
        debug!("Calculating betweenness centrality");

        let mut centrality = HashMap::new();
        let nodes: Vec<_> = self.graph.nodes.keys().cloned().collect();

        for node in &nodes {
            let mut betweenness = 0.0;

            // For each pair of nodes, find shortest paths and count how many go through this node
            for source in &nodes {
                for target in &nodes {
                    if source != target && source != node && target != node {
                        let paths = self.find_shortest_paths(source, target);
                        let paths_through_node = paths.iter()
                            .filter(|path| path.contains(node))
                            .count();

                        if !paths.is_empty() {
                            betweenness += paths_through_node as f64 / paths.len() as f64;
                        }
                    }
                }
            }

            centrality.insert(node.clone(), betweenness);
            self.centrality_cache.insert(node.clone(), betweenness);
        }

        Ok(centrality)
    }

    /// Find the strongest connected components
    pub async fn find_strongly_connected_components(&self) -> Result<Vec<Vec<String>>, AnalysisError> {
        debug!("Finding strongly connected components");

        let mut components = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        // First DFS to fill the stack
        for node_id in self.graph.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_fill_stack(node_id, &mut visited, &mut stack);
            }
        }

        // Create transpose graph
        let transpose_edges = self.create_transpose_edges();

        // Second DFS on transpose graph
        visited.clear();
        while let Some(node_id) = stack.pop() {
            if !visited.contains(&node_id) {
                let mut component = Vec::new();
                self.dfs_collect_component(&node_id, &transpose_edges, &mut visited, &mut component);
                if component.len() > 1 {
                    components.push(component);
                }
            }
        }

        Ok(components)
    }

    /// Detect nodes with high centrality that could be security risks
    async fn detect_high_centrality_risks(&mut self) -> Result<Vec<AntiPatternInfo>, AnalysisError> {
        let centrality = self.calculate_betweenness_centrality().await?;
        let mut patterns = Vec::new();

        let max_centrality = centrality.values().fold(0.0, |a, &b| a.max(b));
        let threshold = max_centrality * 0.7; // Top 30% of central nodes

        for (node_id, &centrality_score) in &centrality {
            if centrality_score >= threshold {
                if let Some(node) = self.graph.nodes.get(node_id) {
                    patterns.push(AntiPatternInfo {
                        pattern_type: "High Centrality Risk".to_string(),
                        description: format!(
                            "Node '{}' has high betweenness centrality ({:.2}), making it a critical point of failure",
                            node.properties.get("name")
                                .and_then(|v| serde_json::from_value::<String>(v.clone()).ok())
                                .unwrap_or_else(|| node_id.clone()),
                            centrality_score
                        ),
                        severity: (centrality_score / max_centrality).min(1.0),
                        affected_entities: vec![node_id.clone()],
                    });
                }
            }
        }

        Ok(patterns)
    }

    /// Detect tightly coupled components
    async fn detect_tight_coupling(&self) -> Result<Vec<AntiPatternInfo>, AnalysisError> {
        let components = self.find_strongly_connected_components().await?;
        let mut patterns = Vec::new();

        for component in components {
            if component.len() > 5 {  // Arbitrary threshold for "too large"
                let coupling_strength = self.calculate_component_coupling(&component);

                patterns.push(AntiPatternInfo {
                    pattern_type: "Tight Coupling".to_string(),
                    description: format!(
                        "Strongly connected component with {} nodes indicates tight coupling",
                        component.len()
                    ),
                    severity: (component.len() as f64 / 20.0).min(1.0),
                    affected_entities: component,
                });
            }
        }

        Ok(patterns)
    }

    /// Detect god objects/classes with too many responsibilities
    async fn detect_god_objects(&self) -> Result<Vec<AntiPatternInfo>, AnalysisError> {
        let mut patterns = Vec::new();

        for (node_id, node) in &self.graph.nodes {
            if matches!(node.node_type, crate::analysis::detectors::security::knowledge_graph::types::GraphNodeType::Class) {
                let connections = self.count_node_connections(node_id);
                let outgoing_edges = self.count_outgoing_edges(node_id);

                // Heuristic: God object has many connections and responsibilities
                if connections > 20 || outgoing_edges > 15 {
                    patterns.push(AntiPatternInfo {
                        pattern_type: "God Object".to_string(),
                        description: format!(
                            "Class '{}' has {} connections and {} outgoing dependencies, indicating too many responsibilities",
                            node.properties.get("name")
                                .and_then(|v| serde_json::from_value::<String>(v.clone()).ok())
                                .unwrap_or_else(|| node_id.clone()),
                            connections,
                            outgoing_edges
                        ),
                        severity: ((connections + outgoing_edges) as f64 / 50.0).min(1.0),
                        affected_entities: vec![node_id.clone()],
                    });
                }
            }
        }

        Ok(patterns)
    }

    /// Detect circular dependencies
    async fn detect_circular_dependencies(&self) -> Result<Vec<AntiPatternInfo>, AnalysisError> {
        let components = self.find_strongly_connected_components().await?;
        let mut patterns = Vec::new();

        for component in components {
            if component.len() >= 2 {
                // Check if this is a true circular dependency (not just bidirectional calls)
                if self.is_circular_dependency(&component) {
                    patterns.push(AntiPatternInfo {
                        pattern_type: "Circular Dependency".to_string(),
                        description: format!(
                            "Circular dependency detected between {} components",
                            component.len()
                        ),
                        severity: (component.len() as f64 / 10.0).min(1.0),
                        affected_entities: component,
                    });
                }
            }
        }

        Ok(patterns)
    }

    /// Find shortest paths between two nodes
    fn find_shortest_paths(&self, source: &str, target: &str) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        self.dfs_find_paths(source, target, &mut visited, &mut current_path, &mut paths, 0, 10);
        paths
    }

    /// DFS to find all paths (limited depth)
    fn dfs_find_paths(
        &self,
        current: &str,
        target: &str,
        visited: &mut HashSet<String>,
        current_path: &mut Vec<String>,
        all_paths: &mut Vec<Vec<String>>,
        depth: usize,
        max_depth: usize,
    ) {
        if depth > max_depth {
            return;
        }

        visited.insert(current.to_string());
        current_path.push(current.to_string());

        if current == target {
            all_paths.push(current_path.clone());
        } else {
            for edge in &self.graph.edges {
                if edge.from == current && !visited.contains(&edge.to) {
                    self.dfs_find_paths(&edge.to, target, visited, current_path, all_paths, depth + 1, max_depth);
                }
            }
        }

        current_path.pop();
        visited.remove(current);
    }

    /// DFS to fill stack for strongly connected components
    fn dfs_fill_stack(&self, node_id: &str, visited: &mut HashSet<String>, stack: &mut Vec<String>) {
        visited.insert(node_id.to_string());

        for edge in &self.graph.edges {
            if edge.from == node_id && !visited.contains(&edge.to) {
                self.dfs_fill_stack(&edge.to, visited, stack);
            }
        }

        stack.push(node_id.to_string());
    }

    /// Create transpose edges for strongly connected components algorithm
    fn create_transpose_edges(&self) -> HashMap<String, Vec<String>> {
        let mut transpose = HashMap::new();

        for edge in &self.graph.edges {
            transpose.entry(edge.to.clone())
                .or_insert_with(Vec::new)
                .push(edge.from.clone());
        }

        transpose
    }

    /// DFS to collect component nodes
    fn dfs_collect_component(
        &self,
        node_id: &str,
        transpose_edges: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        component: &mut Vec<String>,
    ) {
        visited.insert(node_id.to_string());
        component.push(node_id.to_string());

        if let Some(neighbors) = transpose_edges.get(node_id) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_collect_component(neighbor, transpose_edges, visited, component);
                }
            }
        }
    }

    /// Calculate coupling strength of a component
    fn calculate_component_coupling(&self, component: &[String]) -> f64 {
        let component_set: HashSet<_> = component.iter().collect();
        let internal_edges = self.graph.edges.iter()
            .filter(|edge| component_set.contains(&edge.from) && component_set.contains(&edge.to))
            .count();

        let total_possible = component.len() * (component.len() - 1);
        if total_possible > 0 {
            internal_edges as f64 / total_possible as f64
        } else {
            0.0
        }
    }

    /// Count total connections for a node
    fn count_node_connections(&self, node_id: &str) -> usize {
        self.graph.edges.iter()
            .filter(|edge| edge.from == node_id || edge.to == node_id)
            .count()
    }

    /// Count outgoing edges for a node
    fn count_outgoing_edges(&self, node_id: &str) -> usize {
        self.graph.edges.iter()
            .filter(|edge| edge.from == node_id)
            .count()
    }

    /// Check if a component represents a true circular dependency
    fn is_circular_dependency(&self, component: &[String]) -> bool {
        let component_set: HashSet<_> = component.iter().collect();

        // Look for dependency edges (not just call edges) within the component
        self.graph.edges.iter()
            .any(|edge| {
                component_set.contains(&edge.from) &&
                component_set.contains(&edge.to) &&
                matches!(edge.edge_type, crate::analysis::detectors::security::knowledge_graph::types::GraphEdgeType::Dependency)
            })
    }
}