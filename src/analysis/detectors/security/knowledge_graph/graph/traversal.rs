//! Graph traversal algorithms for knowledge graph navigation
//!
//! This module provides efficient algorithms for traversing the knowledge graph,
//! including path finding, neighbor discovery, and security-focused traversal patterns.

use crate::analysis::detectors::security::knowledge_graph::types::{
    GraphEdge, GraphNode, StructuralSemanticGraph, CodeEntity,
};
use crate::analysis::AnalysisError;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::debug;

/// Graph traversal utilities for knowledge graph navigation
pub struct GraphTraversal<'a> {
    graph: &'a StructuralSemanticGraph,
    adjacency_list: HashMap<String, Vec<String>>,
    reverse_adjacency: HashMap<String, Vec<String>>,
}

impl<'a> GraphTraversal<'a> {
    pub fn new(graph: &'a StructuralSemanticGraph) -> Self {
        let mut adjacency_list = HashMap::new();
        let mut reverse_adjacency = HashMap::new();

        // Build adjacency lists for efficient traversal
        for edge in &graph.edges {
            adjacency_list.entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());

            reverse_adjacency.entry(edge.to.clone())
                .or_insert_with(Vec::new)
                .push(edge.from.clone());
        }

        Self {
            graph,
            adjacency_list,
            reverse_adjacency,
        }
    }

    /// Find shortest path between two nodes using BFS
    pub fn find_shortest_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        if start == end {
            return Some(vec![start.to_string()]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = self.adjacency_list.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());

                        if neighbor == end {
                            return Some(self.reconstruct_path(&parent, start, end));
                        }
                    }
                }
            }
        }

        None
    }

    /// Find all paths within a given radius from a starting node
    pub fn find_paths_within_radius(&self, start: &str, radius: usize) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        let mut current_path = vec![start.to_string()];

        self.dfs_radius_paths(start, radius, 0, &mut visited, &mut current_path, &mut paths);
        paths
    }

    /// Find all nodes reachable from a starting node
    pub fn find_reachable_nodes(&self, start: &str) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start.to_string());
        reachable.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = self.adjacency_list.get(&current) {
                for neighbor in neighbors {
                    if !reachable.contains(neighbor) {
                        reachable.insert(neighbor.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        reachable
    }

    /// Find nodes that can reach the target node (reverse reachability)
    pub fn find_nodes_reaching(&self, target: &str) -> HashSet<String> {
        let mut reaching = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(target.to_string());
        reaching.insert(target.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(predecessors) = self.reverse_adjacency.get(&current) {
                for predecessor in predecessors {
                    if !reaching.contains(predecessor) {
                        reaching.insert(predecessor.clone());
                        queue.push_back(predecessor.clone());
                    }
                }
            }
        }

        reaching
    }

    /// Get immediate neighbors of a node
    pub fn get_neighbors(&self, node_id: &str) -> Vec<&GraphNode> {
        let mut neighbors = Vec::new();

        // Add outgoing neighbors
        if let Some(outgoing) = self.adjacency_list.get(node_id) {
            for neighbor_id in outgoing {
                if let Some(node) = self.graph.nodes.get(neighbor_id) {
                    neighbors.push(node);
                }
            }
        }

        // Add incoming neighbors
        if let Some(incoming) = self.reverse_adjacency.get(node_id) {
            for neighbor_id in incoming {
                if let Some(node) = self.graph.nodes.get(neighbor_id) {
                    neighbors.push(node);
                }
            }
        }

        neighbors
    }

    /// Find nodes by type within a certain distance
    pub fn find_nodes_by_type_within_distance(
        &self,
        start: &str,
        node_type: crate::analysis::detectors::security::knowledge_graph::types::GraphNodeType,
        max_distance: usize,
    ) -> Vec<(String, usize)> {
        let mut result = Vec::new();
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();

        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string(), 0);

        while let Some((current, distance)) = queue.pop_front() {
            if distance <= max_distance {
                if let Some(node) = self.graph.nodes.get(&current) {
                    if std::mem::discriminant(&node.node_type) == std::mem::discriminant(&node_type) {
                        result.push((current.clone(), distance));
                    }
                }

                // Explore neighbors
                if let Some(neighbors) = self.adjacency_list.get(&current) {
                    for neighbor in neighbors {
                        let new_distance = distance + 1;
                        if new_distance <= max_distance {
                            if let Some(&existing_distance) = visited.get(neighbor) {
                                if new_distance < existing_distance {
                                    visited.insert(neighbor.clone(), new_distance);
                                    queue.push_back((neighbor.clone(), new_distance));
                                }
                            } else {
                                visited.insert(neighbor.clone(), new_distance);
                                queue.push_back((neighbor.clone(), new_distance));
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Find security-critical paths (paths through high-risk nodes)
    pub fn find_security_critical_paths(
        &self,
        start: &str,
        end: &str,
        high_risk_nodes: &HashSet<String>,
    ) -> Vec<(Vec<String>, f64)> {
        let mut critical_paths = Vec::new();
        let mut all_paths = Vec::new();
        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        self.dfs_find_all_paths(start, end, &mut visited, &mut current_path, &mut all_paths, 0, 15);

        for path in all_paths {
            let risk_score = self.calculate_path_risk_score(&path, high_risk_nodes);
            if risk_score > 0.5 {  // Threshold for critical paths
                critical_paths.push((path, risk_score));
            }
        }

        // Sort by risk score descending
        critical_paths.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        critical_paths
    }

    /// Calculate centrality-based importance of nodes
    pub fn calculate_closeness_centrality(&self, node_id: &str) -> f64 {
        let reachable = self.find_reachable_nodes(node_id);
        let total_distance: usize = reachable.iter()
            .filter_map(|target| self.find_shortest_path(node_id, target))
            .map(|path| path.len() - 1)  // Distance is path length - 1
            .sum();

        if total_distance > 0 && reachable.len() > 1 {
            (reachable.len() - 1) as f64 / total_distance as f64
        } else {
            0.0
        }
    }

    /// Find bottleneck nodes (nodes that appear in many shortest paths)
    pub fn find_bottleneck_nodes(&self, sample_pairs: &[(String, String)]) -> HashMap<String, usize> {
        let mut bottleneck_count = HashMap::new();

        for (start, end) in sample_pairs {
            if let Some(path) = self.find_shortest_path(start, end) {
                // Count intermediate nodes (exclude start and end)
                for node in path.iter().skip(1).take(path.len().saturating_sub(2)) {
                    *bottleneck_count.entry(node.clone()).or_insert(0) += 1;
                }
            }
        }

        bottleneck_count
    }

    /// Reconstruct path from parent map
    fn reconstruct_path(&self, parent: &HashMap<String, String>, start: &str, end: &str) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = end.to_string();

        while current != start {
            path.push(current.clone());
            if let Some(p) = parent.get(&current) {
                current = p.clone();
            } else {
                break;
            }
        }

        path.push(start.to_string());
        path.reverse();
        path
    }

    /// DFS for finding paths within radius
    fn dfs_radius_paths(
        &self,
        current: &str,
        max_radius: usize,
        current_radius: usize,
        visited: &mut HashSet<String>,
        current_path: &mut Vec<String>,
        all_paths: &mut Vec<Vec<String>>,
    ) {
        if current_radius > max_radius {
            return;
        }

        if current_radius > 0 {
            all_paths.push(current_path.clone());
        }

        visited.insert(current.to_string());

        if let Some(neighbors) = self.adjacency_list.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    current_path.push(neighbor.clone());
                    self.dfs_radius_paths(neighbor, max_radius, current_radius + 1, visited, current_path, all_paths);
                    current_path.pop();
                }
            }
        }

        visited.remove(current);
    }

    /// DFS to find all paths between two nodes
    fn dfs_find_all_paths(
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
        } else if let Some(neighbors) = self.adjacency_list.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_find_all_paths(neighbor, target, visited, current_path, all_paths, depth + 1, max_depth);
                }
            }
        }

        current_path.pop();
        visited.remove(current);
    }

    /// Calculate risk score for a path based on high-risk nodes
    fn calculate_path_risk_score(&self, path: &[String], high_risk_nodes: &HashSet<String>) -> f64 {
        let risk_nodes = path.iter().filter(|node| high_risk_nodes.contains(*node)).count();
        if path.is_empty() {
            0.0
        } else {
            risk_nodes as f64 / path.len() as f64
        }
    }
}

/// Specialized traversal for security analysis
pub struct SecurityTraversal<'a> {
    traversal: GraphTraversal<'a>,
}

impl<'a> SecurityTraversal<'a> {
    pub fn new(graph: &'a StructuralSemanticGraph) -> Self {
        Self {
            traversal: GraphTraversal::new(graph),
        }
    }

    /// Find attack surfaces (entry points that can reach sensitive nodes)
    pub fn find_attack_surfaces(&self, sensitive_nodes: &HashSet<String>) -> Vec<String> {
        let mut attack_surfaces = Vec::new();

        for node_id in self.traversal.graph.nodes.keys() {
            // Check if this node can reach any sensitive node
            let reachable = self.traversal.find_reachable_nodes(node_id);
            if sensitive_nodes.iter().any(|sensitive| reachable.contains(sensitive)) {
                // Check if this is an "entry point" (fewer incoming edges)
                if self.is_potential_entry_point(node_id) {
                    attack_surfaces.push(node_id.clone());
                }
            }
        }

        attack_surfaces
    }

    /// Find privilege escalation paths
    pub fn find_privilege_escalation_paths(
        &self,
        low_privilege_nodes: &HashSet<String>,
        high_privilege_nodes: &HashSet<String>,
    ) -> Vec<Vec<String>> {
        let mut escalation_paths = Vec::new();

        for low_node in low_privilege_nodes {
            for high_node in high_privilege_nodes {
                if let Some(path) = self.traversal.find_shortest_path(low_node, high_node) {
                    escalation_paths.push(path);
                }
            }
        }

        escalation_paths
    }

    /// Check if a node is a potential entry point
    fn is_potential_entry_point(&self, node_id: &str) -> bool {
        // Entry points typically have fewer incoming edges
        self.traversal.reverse_adjacency.get(node_id)
            .map(|incoming| incoming.len())
            .unwrap_or(0) <= 2
    }
}