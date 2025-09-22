use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use crate::analysis::detectors::dependency::types::*;
use crate::analysis::detectors::dependency::config::*;
use super::{DependencyAnalyzer, AnalysisOutput};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysisResult {
    pub graph: DependencyGraph,
    pub statistics: GraphStatistics,
    pub critical_paths: Vec<Vec<String>>,
    pub dependency_clusters: Vec<DependencyCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub max_depth: usize,
    pub avg_dependencies_per_node: f64,
    pub density: f64,
    pub connected_components: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCluster {
    pub id: String,
    pub members: HashSet<String>,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub cohesion: f64,
}

pub struct DependencyGraphAnalyzer {
    adjacency_list: HashMap<String, HashSet<String>>,
    reverse_adjacency: HashMap<String, HashSet<String>>,
}

impl DependencyGraphAnalyzer {
    pub fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
            reverse_adjacency: HashMap::new(),
        }
    }

    pub fn build_graph(&mut self, dependencies: &[DependencyInfo]) -> DependencyGraph {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        let mut roots = HashSet::new();

        for (idx, dep) in dependencies.iter().enumerate() {
            let node_id = format!("{}@{}", dep.name, dep.version.as_ref().unwrap_or(&"latest".to_string()));
            
            nodes.insert(node_id.clone(), DependencyNode {
                id: node_id.clone(),
                info: dep.clone(),
                depth: 0,
                is_direct: idx < 10, // Simple heuristic, first 10 are direct
            });

            if idx < 10 {
                roots.insert(node_id.clone());
            }

            self.adjacency_list.entry(node_id.clone())
                .or_insert_with(HashSet::new);
        }

        for i in 0..dependencies.len() {
            for j in i+1..dependencies.len() {
                if self.has_dependency_relation(&dependencies[i], &dependencies[j]) {
                    let from = format!("{}@{}", 
                        dependencies[i].name, 
                        dependencies[i].version.as_ref().unwrap_or(&"latest".to_string()));
                    let to = format!("{}@{}", 
                        dependencies[j].name, 
                        dependencies[j].version.as_ref().unwrap_or(&"latest".to_string()));
                    
                    edges.push(DependencyEdge {
                        from: from.clone(),
                        to: to.clone(),
                        edge_type: EdgeType::Direct,
                    });

                    self.adjacency_list.entry(from.clone())
                        .or_insert_with(HashSet::new)
                        .insert(to.clone());
                    self.reverse_adjacency.entry(to)
                        .or_insert_with(HashSet::new)
                        .insert(from);
                }
            }
        }

        self.calculate_depths(&mut nodes, &roots);

        DependencyGraph { nodes, edges, roots }
    }

    fn has_dependency_relation(&self, dep1: &DependencyInfo, dep2: &DependencyInfo) -> bool {
        dep1.name.contains(&dep2.name[..dep2.name.len().min(3)])
    }

    fn calculate_depths(&self, nodes: &mut HashMap<String, DependencyNode>, roots: &HashSet<String>) {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        for root in roots {
            queue.push_back((root.clone(), 0));
        }

        while let Some((node_id, depth)) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            if let Some(node) = nodes.get_mut(&node_id) {
                node.depth = depth;
            }

            if let Some(neighbors) = self.adjacency_list.get(&node_id) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }
    }

    pub fn find_critical_paths(&self, graph: &DependencyGraph) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        
        for root in &graph.roots {
            let mut path = Vec::new();
            let mut visited = HashSet::new();
            self.dfs_longest_path(root, &mut path, &mut visited, &mut paths);
        }

        paths.sort_by_key(|p| std::cmp::Reverse(p.len()));
        paths.truncate(5);
        paths
    }

    fn dfs_longest_path(
        &self,
        node: &str,
        current_path: &mut Vec<String>,
        visited: &mut HashSet<String>,
        all_paths: &mut Vec<Vec<String>>,
    ) {
        if visited.contains(node) {
            return;
        }

        visited.insert(node.to_string());
        current_path.push(node.to_string());

        if let Some(neighbors) = self.adjacency_list.get(node) {
            if neighbors.is_empty() {
                all_paths.push(current_path.clone());
            } else {
                for neighbor in neighbors {
                    self.dfs_longest_path(neighbor, current_path, visited, all_paths);
                }
            }
        } else {
            all_paths.push(current_path.clone());
        }

        current_path.pop();
        visited.remove(node);
    }

    pub fn detect_clusters(&self, graph: &DependencyGraph) -> Vec<DependencyCluster> {
        let mut clusters = Vec::new();
        let mut visited = HashSet::new();
        let mut cluster_id = 0;

        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                let mut cluster_members = HashSet::new();
                self.bfs_cluster(node_id, &mut cluster_members, &mut visited);
                
                if cluster_members.len() > 1 {
                    let (internal, external) = self.count_cluster_edges(&cluster_members);
                    let cohesion = internal as f64 / (internal + external).max(1) as f64;
                    
                    clusters.push(DependencyCluster {
                        id: format!("cluster_{}", cluster_id),
                        members: cluster_members,
                        internal_edges: internal,
                        external_edges: external,
                        cohesion,
                    });
                    cluster_id += 1;
                }
            }
        }

        clusters
    }

    fn bfs_cluster(&self, start: &str, cluster: &mut HashSet<String>, visited: &mut HashSet<String>) {
        let mut queue = VecDeque::new();
        queue.push_back(start.to_string());

        while let Some(node) = queue.pop_front() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());
            cluster.insert(node.clone());

            if let Some(neighbors) = self.adjacency_list.get(&node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    fn count_cluster_edges(&self, cluster: &HashSet<String>) -> (usize, usize) {
        let mut internal = 0;
        let mut external = 0;

        for node in cluster {
            if let Some(neighbors) = self.adjacency_list.get(node) {
                for neighbor in neighbors {
                    if cluster.contains(neighbor) {
                        internal += 1;
                    } else {
                        external += 1;
                    }
                }
            }
        }

        (internal, external)
    }

    fn calculate_statistics(&self, graph: &DependencyGraph) -> GraphStatistics {
        let total_nodes = graph.nodes.len();
        let total_edges = graph.edges.len();
        let max_depth = graph.nodes.values().map(|n| n.depth).max().unwrap_or(0);
        let avg_dependencies = if total_nodes > 0 {
            total_edges as f64 / total_nodes as f64
        } else {
            0.0
        };
        let max_possible_edges = total_nodes * (total_nodes - 1) / 2;
        let density = if max_possible_edges > 0 {
            total_edges as f64 / max_possible_edges as f64
        } else {
            0.0
        };

        GraphStatistics {
            total_nodes,
            total_edges,
            max_depth,
            avg_dependencies_per_node: avg_dependencies,
            density,
            connected_components: self.count_connected_components(graph),
        }
    }

    fn count_connected_components(&self, graph: &DependencyGraph) -> usize {
        let mut visited = HashSet::new();
        let mut components = 0;

        for node in graph.nodes.keys() {
            if !visited.contains(node) {
                components += 1;
                let mut component = HashSet::new();
                self.bfs_cluster(node, &mut component, &mut visited);
            }
        }

        components
    }
}

impl DependencyAnalyzer for DependencyGraphAnalyzer {
    fn analyze(
        &self,
        dependencies: &[DependencyInfo],
        _config: &DependencyDetectorConfig,
    ) -> Result<AnalysisOutput, DependencyError> {
        let mut analyzer = Self::new();
        let analysis = analyzer.analyze_graph(dependencies);
        Ok(AnalysisOutput::Graph(analysis))
    }

    fn name(&self) -> &str {
        "DependencyGraphAnalyzer"
    }

    fn description(&self) -> &str {
        "Analyzes dependency graph structure, statistics, and clustering patterns"
    }
}
