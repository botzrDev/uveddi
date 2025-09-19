//! Taint propagation analysis modules

pub mod flow_analyzer;
pub mod path_tracker;
pub mod sanitizer_detector;
pub mod patterns;

pub use flow_analyzer::FlowAnalyzer;
pub use path_tracker::PathTracker;
pub use sanitizer_detector::SanitizerDetector;

use crate::analysis::detectors::security::taint_analysis::types::{
    DataFlowGraph, TaintFlow, TaintLevel, DataFlowNode
};
use crate::analysis::AnalysisError;

/// Main propagation analyzer that coordinates all propagation components
pub struct PropagationAnalyzer {
    flow_analyzer: FlowAnalyzer,
    path_tracker: PathTracker,
    sanitizer_detector: SanitizerDetector,
}

impl PropagationAnalyzer {
    pub fn new() -> Self {
        Self {
            flow_analyzer: FlowAnalyzer::new(),
            path_tracker: PathTracker::new(),
            sanitizer_detector: SanitizerDetector::new(),
        }
    }

    /// Analyze taint flows in the data flow graph
    pub fn analyze_taint_flows(&self, graph: &DataFlowGraph) -> Result<Vec<TaintFlow>, AnalysisError> {
        let mut flows = Vec::new();

        // For each taint source, perform forward data flow analysis
        for source_id in &graph.taint_sources {
            let source_flows = self.flow_analyzer.trace_taint_from_source(
                graph,
                source_id,
                &self.path_tracker,
                &self.sanitizer_detector
            )?;
            flows.extend(source_flows);
        }

        Ok(flows)
    }

    /// Propagate taint through a node, considering sanitizers
    pub fn propagate_taint(&self, current_taint: TaintLevel, node: &DataFlowNode) -> TaintLevel {
        self.sanitizer_detector.detect_sanitization(node, current_taint)
    }

    /// Calculate confidence score for a taint flow
    pub fn calculate_flow_confidence(&self, path: &[String], taint_level: &TaintLevel) -> f64 {
        self.path_tracker.calculate_confidence(path, taint_level)
    }
}

impl Default for PropagationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}