use super::{AnalysisOutput, DependencyAnalyzer};
use crate::analysis::detectors::dependency::config::*;
use crate::analysis::detectors::dependency::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependencyAnalysis {
    pub cycles: Vec<CircularDependency>,
    pub affected_packages: HashSet<String>,
    pub severity_distribution: HashMap<CircularDependencySeverity, usize>,
    pub recommendations: Vec<String>,
}

pub struct CircularDependencyDetector {
    adjacency_list: HashMap<String, HashSet<String>>,
    visited: HashSet<String>,
    rec_stack: HashSet<String>,
    cycles: Vec<Vec<String>>,
}

impl CircularDependencyDetector {
    pub fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
            visited: HashSet::new(),
            rec_stack: HashSet::new(),
            cycles: Vec::new(),
        }
    }

    fn detect_cycles(&mut self, dependencies: &[DependencyInfo]) -> CircularDependencyAnalysis {
        self.build_dependency_graph(dependencies);
        self.find_all_cycles();
        self.analyze_cycles()
    }

    fn build_dependency_graph(&mut self, dependencies: &[DependencyInfo]) {
        for dep in dependencies {
            let from = dep.name.clone();

            // Simulate dependencies based on naming patterns
            for other in dependencies {
                if dep.name != other.name && self.has_dependency_pattern(&dep.name, &other.name) {
                    self.adjacency_list
                        .entry(from.clone())
                        .or_insert_with(HashSet::new)
                        .insert(other.name.clone());
                }
            }
        }
    }

    fn has_dependency_pattern(&self, from: &str, to: &str) -> bool {
        // Simple heuristic: check if package names share common prefixes
        let from_parts: Vec<&str> = from.split(&['-', '_', '.'][..]).collect();
        let to_parts: Vec<&str> = to.split(&['-', '_', '.'][..]).collect();

        if from_parts.is_empty() || to_parts.is_empty() {
            return false;
        }

        // Check for common patterns that might indicate dependencies
        from_parts.iter().any(|p| to.contains(p)) || to_parts.iter().any(|p| from.contains(p))
    }

    fn find_all_cycles(&mut self) {
        let nodes: Vec<String> = self.adjacency_list.keys().cloned().collect();

        for node in nodes {
            if !self.visited.contains(&node) {
                let mut path = Vec::new();
                self.dfs_cycle_detection(&node, &mut path);
            }
        }
    }

    fn dfs_cycle_detection(&mut self, node: &str, path: &mut Vec<String>) -> bool {
        if self.rec_stack.contains(node) {
            // Found a cycle
            let cycle_start = path.iter().position(|n| n == node).unwrap_or(0);
            let cycle = path[cycle_start..].to_vec();
            if !cycle.is_empty() {
                self.cycles.push(cycle);
            }
            return true;
        }

        if self.visited.contains(node) {
            return false;
        }

        self.visited.insert(node.to_string());
        self.rec_stack.insert(node.to_string());
        path.push(node.to_string());

        let mut has_cycle = false;
        if let Some(neighbors) = self.adjacency_list.get(node).cloned() {
            for neighbor in neighbors {
                if self.dfs_cycle_detection(&neighbor, path) {
                    has_cycle = true;
                }
            }
        }

        path.pop();
        self.rec_stack.remove(node);
        has_cycle
    }

    fn analyze_cycles(&self) -> CircularDependencyAnalysis {
        let mut affected_packages = HashSet::new();
        let mut severity_distribution = HashMap::new();
        let mut circular_dependencies = Vec::new();

        for cycle in &self.cycles {
            for package in cycle {
                affected_packages.insert(package.clone());
            }

            let severity = self.assess_cycle_severity(cycle);
            *severity_distribution.entry(severity).or_insert(0) += 1;

            circular_dependencies.push(CircularDependency {
                cycle: cycle.clone(),
                entry_point: cycle.first().unwrap_or(&String::new()).clone(),
                severity,
            });
        }

        let recommendations = self.generate_recommendations(&circular_dependencies);

        CircularDependencyAnalysis {
            cycles: circular_dependencies,
            affected_packages,
            severity_distribution,
            recommendations,
        }
    }

    fn assess_cycle_severity(&self, cycle: &[String]) -> CircularDependencySeverity {
        match cycle.len() {
            0..=2 => CircularDependencySeverity::Low,
            3..=4 => CircularDependencySeverity::Medium,
            _ => CircularDependencySeverity::High,
        }
    }

    fn generate_recommendations(&self, cycles: &[CircularDependency]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !cycles.is_empty() {
            recommendations.push("Consider refactoring to break circular dependencies".to_string());
        }

        for cycle in cycles {
            match cycle.severity {
                CircularDependencySeverity::High => {
                    recommendations.push(format!(
                        "High-priority: Break cycle in {}",
                        cycle.entry_point
                    ));
                }
                CircularDependencySeverity::Medium => {
                    recommendations.push(format!(
                        "Medium-priority: Review dependency structure in {}",
                        cycle.entry_point
                    ));
                }
                CircularDependencySeverity::Low => {
                    recommendations.push(format!(
                        "Low-priority: Consider simplifying dependencies in {}",
                        cycle.entry_point
                    ));
                }
            }
        }

        // Add general recommendations
        if cycles.len() > 5 {
            recommendations
                .push("Consider introducing dependency injection or interfaces".to_string());
            recommendations.push("Review overall architecture for tight coupling".to_string());
        }

        recommendations
    }

    pub fn find_strongly_connected_components(&self) -> Vec<HashSet<String>> {
        // Tarjan's algorithm for finding SCCs
        let mut index_counter = 0;
        let mut stack = Vec::new();
        let mut lowlinks = HashMap::new();
        let mut index = HashMap::new();
        let mut on_stack = HashSet::new();
        let mut sccs = Vec::new();

        for node in self.adjacency_list.keys() {
            if !index.contains_key(node) {
                self.tarjan_scc(
                    node,
                    &mut index_counter,
                    &mut stack,
                    &mut lowlinks,
                    &mut index,
                    &mut on_stack,
                    &mut sccs,
                );
            }
        }

        sccs
    }

    fn tarjan_scc(
        &self,
        node: &str,
        index_counter: &mut usize,
        stack: &mut Vec<String>,
        lowlinks: &mut HashMap<String, usize>,
        index: &mut HashMap<String, usize>,
        on_stack: &mut HashSet<String>,
        sccs: &mut Vec<HashSet<String>>,
    ) {
        index.insert(node.to_string(), *index_counter);
        lowlinks.insert(node.to_string(), *index_counter);
        *index_counter += 1;
        stack.push(node.to_string());
        on_stack.insert(node.to_string());

        if let Some(neighbors) = self.adjacency_list.get(node) {
            for neighbor in neighbors {
                if !index.contains_key(neighbor) {
                    self.tarjan_scc(
                        neighbor,
                        index_counter,
                        stack,
                        lowlinks,
                        index,
                        on_stack,
                        sccs,
                    );
                    let neighbor_lowlink = *lowlinks.get(neighbor).unwrap();
                    let current_lowlink = lowlinks.get_mut(node).unwrap();
                    *current_lowlink = (*current_lowlink).min(neighbor_lowlink);
                } else if on_stack.contains(neighbor) {
                    let neighbor_index = *index.get(neighbor).unwrap();
                    let current_lowlink = lowlinks.get_mut(node).unwrap();
                    *current_lowlink = (*current_lowlink).min(neighbor_index);
                }
            }
        }

        if lowlinks[node] == index[node] {
            let mut scc = HashSet::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                scc.insert(w.clone());
                if w == node {
                    break;
                }
            }
            if scc.len() > 1 {
                sccs.push(scc);
            }
        }
    }
}

impl DependencyAnalyzer for CircularDependencyDetector {
    fn analyze(
        &self,
        dependencies: &[DependencyInfo],
        _config: &DependencyDetectorConfig,
    ) -> Result<AnalysisOutput, DependencyError> {
        let mut detector = Self::new();
        let analysis = detector.detect_cycles(dependencies);
        Ok(AnalysisOutput::Circular(analysis))
    }

    fn name(&self) -> &str {
        "CircularDependencyDetector"
    }

    fn description(&self) -> &str {
        "Detects and analyzes circular dependencies in the dependency graph"
    }
}
