//! Control flow validation for dead code detection

use std::collections::{HashSet, VecDeque};

use crate::analysis::detectors::anti_patterns::dead_code::types::Symbol;

/// Analyzes control flow to validate dead code detection
pub struct FlowAnalyzer;

impl FlowAnalyzer {
    /// Check if a symbol is reachable through control flow
    pub fn is_reachable(symbol: &Symbol, all_symbols: &[Symbol]) -> bool {
        // Build control flow graph
        let cfg = Self::build_control_flow_graph(all_symbols);

        // Find entry points
        let entry_points = Self::find_entry_points(all_symbols);

        // Perform reachability analysis
        Self::analyze_reachability(&cfg, &entry_points, &symbol.name)
    }

    /// Build a control flow graph from symbols
    fn build_control_flow_graph(symbols: &[Symbol]) -> ControlFlowGraph {
        let mut cfg = ControlFlowGraph::new();

        for symbol in symbols {
            cfg.add_node(symbol.name.clone());

            // Add edges based on code snippet analysis
            // This is simplified; real implementation would parse the AST
            for other in symbols {
                if symbol.code_snippet.contains(&other.name) {
                    cfg.add_edge(symbol.name.clone(), other.name.clone());
                }
            }
        }

        cfg
    }

    /// Find entry points in the code
    fn find_entry_points(symbols: &[Symbol]) -> Vec<String> {
        symbols
            .iter()
            .filter(|s| {
                s.name == "main"
                    || s.name.starts_with("test_")
                    || s.name == "__init__"
                    || s.is_exported
            })
            .map(|s| s.name.clone())
            .collect()
    }

    /// Analyze reachability from entry points
    fn analyze_reachability(cfg: &ControlFlowGraph, entry_points: &[String], target: &str) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from all entry points
        for entry in entry_points {
            queue.push_back(entry.clone());
        }

        // BFS to find target
        while let Some(current) = queue.pop_front() {
            if current == target {
                return true;
            }

            if visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());

            // Add all reachable nodes
            if let Some(edges) = cfg.edges.get(&current) {
                for next in edges {
                    if !visited.contains(next) {
                        queue.push_back(next.clone());
                    }
                }
            }
        }

        false
    }

    /// Analyze for unreachable branches
    pub fn find_unreachable_branches(cfg: &ControlFlowGraph) -> Vec<String> {
        let mut unreachable = Vec::new();

        for node in &cfg.nodes {
            if !cfg.has_incoming_edges(node) && node != "main" {
                unreachable.push(node.clone());
            }
        }

        unreachable
    }
}

/// Control flow graph representation
pub struct ControlFlowGraph {
    nodes: HashSet<String>,
    edges: std::collections::HashMap<String, HashSet<String>>,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: std::collections::HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: String) {
        self.nodes.insert(node);
    }

    pub fn add_edge(&mut self, from: String, to: String) {
        self.edges
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    pub fn has_incoming_edges(&self, node: &str) -> bool {
        self.edges.values().any(|edges| edges.contains(node))
    }

    pub fn get_successors(&self, node: &str) -> Option<&HashSet<String>> {
        self.edges.get(node)
    }
}
