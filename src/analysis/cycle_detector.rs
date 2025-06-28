use std::collections::{HashMap, HashSet};
use crate::analysis::dependency_graph::{DependencyGraph, Cycle, AnalysisResults};
use log::{debug, info};

/// Cycle detection using Depth-First Search with color coding
pub struct CycleDetector {
    visited: HashMap<String, VisitState>,
    current_path: Vec<String>,
    cycles: Vec<Cycle>,
}

#[derive(Debug, Clone, PartialEq)]
enum VisitState {
    Unvisited,
    Visiting,   // Gray - currently in DFS path
    Visited,    // Black - completely processed
}

impl CycleDetector {
    pub fn new() -> Self {
        Self {
            visited: HashMap::new(),
            current_path: Vec::new(),
            cycles: Vec::new(),
        }
    }

    /// Detect all cycles in the dependency graph
    pub fn detect_cycles(&mut self, graph: &DependencyGraph) -> AnalysisResults {
        let start_time = std::time::Instant::now();
        for module in graph.get_modules() {
            self.visited.insert(module.clone(), VisitState::Unvisited);
        }
        for module in graph.get_modules() {
            if self.visited.get(module) == Some(&VisitState::Unvisited) {
                self.dfs_visit(module, graph);
            }
        }
        let analysis_duration = start_time.elapsed();
        info!("Cycle detection completed: found {} cycles in {:?}", 
              self.cycles.len(), analysis_duration);
        AnalysisResults {
            cycles: self.cycles.clone(),
            total_modules: graph.get_modules().len(),
            total_dependencies: graph.get_all_dependencies().len(),
            analysis_duration,
        }
    }

    fn dfs_visit(&mut self, module: &str, graph: &DependencyGraph) {
        self.visited.insert(module.to_string(), VisitState::Visiting);
        self.current_path.push(module.to_string());
        if let Some(dependencies) = graph.get_dependencies(module) {
            for dep in dependencies {
                match self.visited.get(dep) {
                    Some(VisitState::Visiting) => {
                        self.extract_cycle(dep, graph);
                    }
                    Some(VisitState::Unvisited) | None => {
                        self.dfs_visit(dep, graph);
                    }
                    Some(VisitState::Visited) => {}
                }
            }
        }
        self.visited.insert(module.to_string(), VisitState::Visited);
        self.current_path.pop();
    }

    fn extract_cycle(&mut self, back_edge_target: &str, graph: &DependencyGraph) {
        if let Some(cycle_start) = self.current_path.iter().position(|m| m == back_edge_target) {
            let cycle_modules: Vec<String> = self.current_path[cycle_start..].to_vec();
            if !self.is_duplicate_cycle(&cycle_modules) {
                let cycle = Cycle::new(cycle_modules, graph);
                debug!("Found cycle: {:?}", cycle.modules);
                self.cycles.push(cycle);
            }
        }
    }

    fn is_duplicate_cycle(&self, new_cycle: &[String]) -> bool {
        self.cycles.iter().any(|existing| {
            let mut existing_sorted = existing.modules.clone();
            existing_sorted.sort();
            let mut new_sorted = new_cycle.to_vec();
            new_sorted.sort();
            existing_sorted == new_sorted
        })
    }
}
