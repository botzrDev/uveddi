//! Core taint flow analysis logic

use crate::analysis::detectors::security::taint_analysis::propagation::{
    PathTracker, SanitizerDetector,
};
use crate::analysis::detectors::security::taint_analysis::types::{
    DataFlowGraph, DataFlowNodeType, TaintFlow, TaintLevel,
};
use crate::analysis::AnalysisError;
use std::collections::{HashSet, VecDeque};
use tracing::debug;

/// Main flow analyzer for taint propagation
pub struct FlowAnalyzer {
    max_depth: usize,
    max_paths: usize,
}

impl FlowAnalyzer {
    pub fn new() -> Self {
        Self {
            max_depth: 10,
            max_paths: 1000,
        }
    }

    pub fn with_limits(max_depth: usize, max_paths: usize) -> Self {
        Self {
            max_depth,
            max_paths,
        }
    }

    /// Trace taint flow from a specific source using BFS
    pub fn trace_taint_from_source(
        &self,
        graph: &DataFlowGraph,
        source_id: &str,
        path_tracker: &PathTracker,
        sanitizer_detector: &SanitizerDetector,
    ) -> Result<Vec<TaintFlow>, AnalysisError> {
        let mut flows = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let current_path = Vec::new();

        debug!("Tracing taint from source: {}", source_id);

        queue.push_back((
            source_id.to_string(),
            TaintLevel::Tainted,
            current_path.clone(),
        ));

        while let Some((node_id, taint_level, path)) = queue.pop_front() {
            // Prevent infinite loops
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            let mut new_path = path;
            new_path.push(node_id.clone());

            // Check path limits
            if new_path.len() > self.max_depth {
                continue;
            }

            if flows.len() >= self.max_paths {
                break;
            }

            if let Some(node) = graph.nodes.get(&node_id) {
                // Check if this is a sink
                if graph.taint_sinks.contains(&node_id) && taint_level.is_dangerous() {
                    let sanitizers_in_path = self.find_sanitizers_in_path(graph, &new_path);
                    let confidence = path_tracker.calculate_confidence(&new_path, &taint_level);

                    flows.push(TaintFlow {
                        source_id: source_id.to_string(),
                        sink_id: node_id.clone(),
                        path: new_path.clone(),
                        taint_level: taint_level.clone(),
                        sanitizers_passed: sanitizers_in_path,
                        confidence,
                    });
                }

                // Continue propagation to successors
                let new_taint_level =
                    sanitizer_detector.detect_sanitization(node, taint_level.clone());

                // Only continue if there's still meaningful taint
                if new_taint_level.score() > 0.1 {
                    for successor in graph.get_successors(&node_id) {
                        queue.push_back((
                            successor.to_string(),
                            new_taint_level.clone(),
                            new_path.clone(),
                        ));
                    }
                }
            }
        }

        debug!(
            "Found {} taint flows from source {}",
            flows.len(),
            source_id
        );
        Ok(flows)
    }

    /// Perform inter-procedural taint analysis
    pub fn analyze_interprocedural_flows(
        &self,
        graph: &DataFlowGraph,
        source_id: &str,
        path_tracker: &PathTracker,
        sanitizer_detector: &SanitizerDetector,
    ) -> Result<Vec<TaintFlow>, AnalysisError> {
        // This would implement more sophisticated inter-procedural analysis
        // For now, delegate to the basic intra-procedural analysis
        self.trace_taint_from_source(graph, source_id, path_tracker, sanitizer_detector)
    }

    /// Perform field-sensitive taint analysis
    pub fn analyze_field_sensitive_flows(
        &self,
        graph: &DataFlowGraph,
        source_id: &str,
        path_tracker: &PathTracker,
        sanitizer_detector: &SanitizerDetector,
    ) -> Result<Vec<TaintFlow>, AnalysisError> {
        // This would track taint at the field level for objects
        // For now, delegate to the basic analysis
        self.trace_taint_from_source(graph, source_id, path_tracker, sanitizer_detector)
    }

    /// Perform path-sensitive taint analysis
    pub fn analyze_path_sensitive_flows(
        &self,
        graph: &DataFlowGraph,
        source_id: &str,
        path_tracker: &PathTracker,
        sanitizer_detector: &SanitizerDetector,
    ) -> Result<Vec<TaintFlow>, AnalysisError> {
        // This would consider different execution paths and their conditions
        // For now, delegate to the basic analysis
        self.trace_taint_from_source(graph, source_id, path_tracker, sanitizer_detector)
    }

    /// Find sanitizers along a taint path
    fn find_sanitizers_in_path(&self, graph: &DataFlowGraph, path: &[String]) -> Vec<String> {
        let mut sanitizers = Vec::new();

        for node_id in path {
            if let Some(node) = graph.nodes.get(node_id) {
                if let DataFlowNodeType::Sanitizer(sanitizer_id) = &node.node_type {
                    sanitizers.push(sanitizer_id.clone());
                }
            }
        }

        sanitizers
    }

    /// Check if a path contains effective sanitization
    pub fn has_effective_sanitization(
        &self,
        graph: &DataFlowGraph,
        path: &[String],
        vulnerability_type: &str,
    ) -> bool {
        for node_id in path {
            if let Some(node) = graph.nodes.get(node_id) {
                if let DataFlowNodeType::Sanitizer(_) = &node.node_type {
                    // TODO: Check if sanitizer is effective for this vulnerability type
                    // This would involve checking sanitizer metadata
                    return true;
                }
            }
        }
        false
    }

    /// Calculate taint propagation through control flow
    pub fn propagate_through_control_flow(
        &self,
        graph: &DataFlowGraph,
        current_taint: &TaintLevel,
        control_node: &str,
    ) -> TaintLevel {
        // This would handle taint propagation through conditionals, loops, etc.
        // For now, just return the current taint level
        current_taint.clone()
    }

    /// Handle taint merging at join points
    pub fn merge_taint_at_join(&self, taint_levels: &[TaintLevel]) -> TaintLevel {
        if taint_levels.is_empty() {
            return TaintLevel::Clean;
        }

        let mut result = taint_levels[0].clone();
        for taint in &taint_levels[1..] {
            result = result.combine(taint);
        }

        result
    }
}

impl Default for FlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
