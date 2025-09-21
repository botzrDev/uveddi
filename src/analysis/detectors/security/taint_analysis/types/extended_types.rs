//! Extended types for taint analysis (data flow graph and related structures)

use super::core_types::{SourceLocation, TaintLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a data flow graph node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataFlowNode {
    pub id: String,
    pub node_type: DataFlowNodeType,
    pub location: Option<SourceLocation>,
    pub taint_level: TaintLevel,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataFlowNodeType {
    Source(String),       // Taint source
    Sink(String),         // Taint sink
    Sanitizer(String),    // Sanitization point
    Variable(String),     // Variable assignment
    FunctionCall(String), // Function call
    Parameter(String),    // Function parameter
    Return(String),       // Function return
    FieldAccess(String),  // Object field access
    ArrayAccess(String),  // Array element access
}

/// Data flow graph representing taint propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowGraph {
    pub nodes: HashMap<String, DataFlowNode>,
    pub edges: HashMap<String, Vec<String>>, // node_id -> list of successor node_ids
    pub reverse_edges: HashMap<String, Vec<String>>, // node_id -> list of predecessor node_ids
    pub taint_sources: Vec<String>,
    pub taint_sinks: Vec<String>,
    pub sanitizers: Vec<String>,
}

impl DataFlowGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
            taint_sources: Vec::new(),
            taint_sinks: Vec::new(),
            sanitizers: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: DataFlowNode) {
        let node_id = node.id.clone();

        match &node.node_type {
            DataFlowNodeType::Source(_) => self.taint_sources.push(node_id.clone()),
            DataFlowNodeType::Sink(_) => self.taint_sinks.push(node_id.clone()),
            DataFlowNodeType::Sanitizer(_) => self.sanitizers.push(node_id.clone()),
            _ => {}
        }

        self.nodes.insert(node_id.clone(), node);
        self.edges.entry(node_id.clone()).or_insert_with(Vec::new);
        self.reverse_edges.entry(node_id).or_insert_with(Vec::new);
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
        self.reverse_edges
            .entry(to.to_string())
            .or_default()
            .push(from.to_string());
    }

    pub fn get_successors(&self, node_id: &str) -> Vec<&str> {
        self.edges
            .get(node_id)
            .map(|successors| successors.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(Vec::new)
    }

    pub fn get_predecessors(&self, node_id: &str) -> Vec<&str> {
        self.reverse_edges
            .get(node_id)
            .map(|predecessors| predecessors.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(Vec::new)
    }
}

impl Default for DataFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a taint flow from source to sink
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintFlow {
    pub source_id: String,
    pub sink_id: String,
    pub path: Vec<String>,
    pub taint_level: TaintLevel,
    pub sanitizers_passed: Vec<String>,
    pub confidence: f64,
}

/// Analysis context for taint flow analysis
#[derive(Debug, Clone)]
pub struct TaintAnalysisContext {
    pub max_depth: usize,
    pub max_paths: usize,
    pub enable_interprocedural: bool,
    pub enable_field_sensitive: bool,
    pub enable_path_sensitive: bool,
}

impl TaintAnalysisContext {
    pub fn new() -> Self {
        Self {
            max_depth: 10,
            max_paths: 100,
            enable_interprocedural: false,
            enable_field_sensitive: false,
            enable_path_sensitive: false,
        }
    }

    pub fn with_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn with_paths(mut self, max_paths: usize) -> Self {
        self.max_paths = max_paths;
        self
    }

    pub fn enable_interprocedural(mut self) -> Self {
        self.enable_interprocedural = true;
        self
    }

    pub fn enable_field_sensitive(mut self) -> Self {
        self.enable_field_sensitive = true;
        self
    }

    pub fn enable_path_sensitive(mut self) -> Self {
        self.enable_path_sensitive = true;
        self
    }
}

impl Default for TaintAnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Confidence metrics for taint analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceMetrics {
    pub path_confidence: f64,
    pub taint_confidence: f64,
    pub sanitization_confidence: f64,
    pub overall_confidence: f64,
}

impl ConfidenceMetrics {
    pub fn new() -> Self {
        Self {
            path_confidence: 0.0,
            taint_confidence: 0.0,
            sanitization_confidence: 0.0,
            overall_confidence: 0.0,
        }
    }

    pub fn calculate_overall(&mut self) {
        self.overall_confidence =
            (self.path_confidence + self.taint_confidence + self.sanitization_confidence) / 3.0;
    }
}

impl Default for ConfidenceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Taint propagation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintPropagationRule {
    pub rule_id: String,
    pub source_pattern: String,
    pub sink_pattern: String,
    pub propagation_type: PropagationType,
    pub confidence_modifier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropagationType {
    Direct,         // Direct data flow
    Assignment,     // Variable assignment
    FunctionCall,   // Function parameter passing
    FunctionReturn, // Function return value
    FieldAccess,    // Object field access
    ArrayAccess,    // Array element access
    StringFormat,   // String formatting
    Concatenation,  // String concatenation
}

/// Sanitization effectiveness rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationEffectiveness {
    pub sanitizer_id: String,
    pub vulnerability_types: Vec<crate::analysis::detectors::security::types::SecurityIssueType>,
    pub effectiveness_score: f64,
    pub context_dependent: bool,
    pub false_positive_rate: f64,
}

impl SanitizationEffectiveness {
    pub fn new(sanitizer_id: String) -> Self {
        Self {
            sanitizer_id,
            vulnerability_types: Vec::new(),
            effectiveness_score: 0.8,
            context_dependent: false,
            false_positive_rate: 0.1,
        }
    }
}
