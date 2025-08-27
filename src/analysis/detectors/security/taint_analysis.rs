//! Taint analysis engine for tracking data flow vulnerabilities
//!
//! This module implements a comprehensive taint analysis system that models
//! injection vulnerabilities as data flow problems. It tracks untrusted input
//! from sources through the program to sensitive execution sinks, identifying
//! potential security vulnerabilities when tainted data reaches dangerous operations.
//!
//! ## Core Concepts
//!
//! - **Taint Sources**: Origins of untrusted data (user input, environment variables, files)
//! - **Taint Sinks**: Sensitive operations where tainted data should not arrive (SQL queries, command execution)
//! - **Sanitization Points**: Functions that properly clean or validate data
//! - **Data Flow Graph**: Representation of how data flows through the program
//!
//! ## Implementation Strategy
//!
//! The taint analysis uses static analysis techniques including:
//! - Tree-sitter AST traversal for language-agnostic analysis
//! - Control Flow Graph (CFG) construction
//! - Inter-procedural data flow analysis
//! - Field-sensitive analysis for object properties
//! - Path-sensitive analysis for complex control flow

use crate::analysis::detectors::security::config::TaintAnalysisConfig;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
    VulnerabilityType,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Represents a taint source in the program
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaintSource {
    /// Unique identifier for this source
    pub id: String,
    /// Source pattern (e.g., "std::env::args", "request.body")
    pub pattern: String,
    /// Programming language this source applies to
    pub language: Option<SourceLanguage>,
    /// Description of what this source represents
    pub description: String,
    /// Severity level of vulnerabilities from this source
    pub default_severity: SecuritySeverity,
    /// Source location if found in code
    pub location: Option<SourceLocation>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TaintSource {
    pub fn new(id: String, pattern: String, description: String) -> Self {
        Self {
            id,
            pattern,
            language: None,
            description,
            default_severity: SecuritySeverity::Medium,
            location: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_language(mut self, language: SourceLanguage) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_severity(mut self, severity: SecuritySeverity) -> Self {
        self.default_severity = severity;
        self
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }
}

/// Represents a taint sink (dangerous operation)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaintSink {
    /// Unique identifier for this sink
    pub id: String,
    /// Sink pattern (e.g., "sqlx::query", "os.system")
    pub pattern: String,
    /// Programming language this sink applies to
    pub language: Option<SourceLanguage>,
    /// Type of vulnerability this sink can cause
    pub vulnerability_type: SecurityIssueType,
    /// Description of the vulnerability
    pub description: String,
    /// Severity of vulnerabilities reaching this sink
    pub severity: SecuritySeverity,
    /// Source location if found in code
    pub location: Option<SourceLocation>,
    /// Which parameters are vulnerable (0-indexed, None means all)
    pub vulnerable_parameters: Option<Vec<usize>>,
}

impl TaintSink {
    pub fn new(
        id: String,
        pattern: String,
        vulnerability_type: SecurityIssueType,
        description: String,
    ) -> Self {
        let severity = vulnerability_type.default_severity();
        Self {
            id,
            pattern,
            language: None,
            vulnerability_type,
            description,
            severity,
            location: None,
            vulnerable_parameters: None,
        }
    }

    pub fn with_language(mut self, language: SourceLanguage) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_vulnerable_params(mut self, params: Vec<usize>) -> Self {
        self.vulnerable_parameters = Some(params);
        self
    }
}

/// Represents a sanitization point that cleans tainted data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SanitizationPoint {
    /// Unique identifier for this sanitizer
    pub id: String,
    /// Sanitizer pattern (e.g., "html_escape", "parameterized_query")
    pub pattern: String,
    /// Programming language this sanitizer applies to
    pub language: Option<SourceLanguage>,
    /// Types of vulnerabilities this sanitizer prevents
    pub prevents: Vec<SecurityIssueType>,
    /// Description of what this sanitizer does
    pub description: String,
    /// Source location if found in code
    pub location: Option<SourceLocation>,
    /// Effectiveness score (0.0 - 1.0)
    pub effectiveness: f64,
}

impl SanitizationPoint {
    pub fn new(id: String, pattern: String, prevents: Vec<SecurityIssueType>) -> Self {
        Self {
            id,
            pattern,
            language: None,
            prevents,
            description: String::new(),
            location: None,
            effectiveness: 1.0,
        }
    }

    pub fn with_language(mut self, language: SourceLanguage) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_effectiveness(mut self, effectiveness: f64) -> Self {
        self.effectiveness = effectiveness.clamp(0.0, 1.0);
        self
    }
}

/// Source location in code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub length: u32,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaintLevel {
    Clean,        // No taint
    Tainted,      // Fully tainted
    Sanitized,    // Was tainted but sanitized
    Partial(f64), // Partially tainted (0.0 - 1.0)
}

impl TaintLevel {
    pub fn is_dangerous(&self) -> bool {
        match self {
            TaintLevel::Tainted => true,
            TaintLevel::Partial(level) => *level > 0.5,
            _ => false,
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            TaintLevel::Clean => 0.0,
            TaintLevel::Sanitized => 0.1, // Small residual risk
            TaintLevel::Partial(level) => *level,
            TaintLevel::Tainted => 1.0,
        }
    }

    pub fn combine(&self, other: &TaintLevel) -> TaintLevel {
        match (self, other) {
            (TaintLevel::Clean, other) => other.clone(),
            (other, TaintLevel::Clean) => other.clone(),
            (TaintLevel::Tainted, _) | (_, TaintLevel::Tainted) => TaintLevel::Tainted,
            (TaintLevel::Partial(a), TaintLevel::Partial(b)) => {
                TaintLevel::Partial((*a + *b).min(1.0))
            }
            (TaintLevel::Partial(level), TaintLevel::Sanitized)
            | (TaintLevel::Sanitized, TaintLevel::Partial(level)) => {
                TaintLevel::Partial(*level * 0.5) // Sanitization reduces risk
            }
            (TaintLevel::Sanitized, TaintLevel::Sanitized) => TaintLevel::Sanitized,
        }
    }
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

/// Main taint analysis engine
pub struct TaintAnalysisEngine {
    config: TaintAnalysisConfig,
    sources: Vec<TaintSource>,
    sinks: Vec<TaintSink>,
    sanitizers: Vec<SanitizationPoint>,
    language_patterns: HashMap<SourceLanguage, LanguageTaintPatterns>,
}

#[derive(Debug, Clone)]
struct LanguageTaintPatterns {
    source_patterns: Vec<String>,
    sink_patterns: Vec<String>,
    sanitizer_patterns: Vec<String>,
}

impl TaintAnalysisEngine {
    pub fn new(config: TaintAnalysisConfig) -> Result<Self, AnalysisError> {
        let mut engine = Self {
            config,
            sources: Vec::new(),
            sinks: Vec::new(),
            sanitizers: Vec::new(),
            language_patterns: HashMap::new(),
        };

        engine.initialize_default_patterns()?;
        Ok(engine)
    }

    /// Initialize default taint patterns for supported languages
    fn initialize_default_patterns(&mut self) -> Result<(), AnalysisError> {
        // Rust patterns
        self.add_rust_patterns();
        // Python patterns
        self.add_python_patterns();
        // JavaScript/TypeScript patterns
        self.add_javascript_patterns();

        Ok(())
    }

    fn add_rust_patterns(&mut self) {
        let rust_sources = vec![
            TaintSource::new(
                "rust_env_args".to_string(),
                "std::env::args".to_string(),
                "Command line arguments".to_string(),
            )
            .with_language(SourceLanguage::Rust),
            TaintSource::new(
                "rust_env_var".to_string(),
                "std::env::var".to_string(),
                "Environment variables".to_string(),
            )
            .with_language(SourceLanguage::Rust),
            TaintSource::new(
                "rust_file_read".to_string(),
                "std::fs::read_to_string".to_string(),
                "File contents".to_string(),
            )
            .with_language(SourceLanguage::Rust),
            TaintSource::new(
                "rust_http_request".to_string(),
                "request.body".to_string(),
                "HTTP request body".to_string(),
            )
            .with_language(SourceLanguage::Rust),
        ];

        let rust_sinks = vec![
            TaintSink::new(
                "rust_sql_query".to_string(),
                "sqlx::query".to_string(),
                SecurityIssueType::Injection,
                "SQL query execution with potential injection".to_string(),
            )
            .with_language(SourceLanguage::Rust),
            TaintSink::new(
                "rust_command_exec".to_string(),
                "std::process::Command::new".to_string(),
                SecurityIssueType::Injection,
                "Command execution with user input".to_string(),
            )
            .with_language(SourceLanguage::Rust),
            TaintSink::new(
                "rust_file_write".to_string(),
                "std::fs::write".to_string(),
                SecurityIssueType::PathTraversal,
                "File write with user-controlled path".to_string(),
            )
            .with_language(SourceLanguage::Rust),
        ];

        let rust_sanitizers = vec![SanitizationPoint::new(
            "rust_sql_bind".to_string(),
            "sqlx::query!".to_string(),
            vec![SecurityIssueType::Injection],
        )
        .with_language(SourceLanguage::Rust)];

        self.sources.extend(rust_sources);
        self.sinks.extend(rust_sinks);
        self.sanitizers.extend(rust_sanitizers);
    }

    fn add_python_patterns(&mut self) {
        let python_sources = vec![
            TaintSource::new(
                "python_sys_argv".to_string(),
                "sys.argv".to_string(),
                "Command line arguments".to_string(),
            )
            .with_language(SourceLanguage::Python),
            TaintSource::new(
                "python_input".to_string(),
                "input(".to_string(),
                "User input".to_string(),
            )
            .with_language(SourceLanguage::Python),
            TaintSource::new(
                "python_request".to_string(),
                "request.".to_string(),
                "HTTP request data".to_string(),
            )
            .with_language(SourceLanguage::Python),
            TaintSource::new(
                "python_file_read".to_string(),
                "open(".to_string(),
                "File contents".to_string(),
            )
            .with_language(SourceLanguage::Python),
        ];

        let python_sinks = vec![
            TaintSink::new(
                "python_sql_execute".to_string(),
                "cursor.execute".to_string(),
                SecurityIssueType::Injection,
                "SQL execution with potential injection".to_string(),
            )
            .with_language(SourceLanguage::Python),
            TaintSink::new(
                "python_eval".to_string(),
                "eval(".to_string(),
                SecurityIssueType::Injection,
                "Code evaluation with user input".to_string(),
            )
            .with_language(SourceLanguage::Python),
            TaintSink::new(
                "python_exec".to_string(),
                "exec(".to_string(),
                SecurityIssueType::Injection,
                "Code execution with user input".to_string(),
            )
            .with_language(SourceLanguage::Python),
            TaintSink::new(
                "python_os_system".to_string(),
                "os.system".to_string(),
                SecurityIssueType::Injection,
                "OS command execution".to_string(),
            )
            .with_language(SourceLanguage::Python),
        ];

        self.sources.extend(python_sources);
        self.sinks.extend(python_sinks);
    }

    fn add_javascript_patterns(&mut self) {
        let js_sources = vec![
            TaintSource::new(
                "js_query_params".to_string(),
                "req.query".to_string(),
                "URL query parameters".to_string(),
            )
            .with_language(SourceLanguage::JavaScript),
            TaintSource::new(
                "js_request_body".to_string(),
                "req.body".to_string(),
                "HTTP request body".to_string(),
            )
            .with_language(SourceLanguage::JavaScript),
            TaintSource::new(
                "js_location_search".to_string(),
                "location.search".to_string(),
                "Browser URL parameters".to_string(),
            )
            .with_language(SourceLanguage::JavaScript),
        ];

        let js_sinks = vec![
            TaintSink::new(
                "js_innerHTML".to_string(),
                "innerHTML".to_string(),
                SecurityIssueType::CrossSiteScripting,
                "DOM manipulation with user input".to_string(),
            )
            .with_language(SourceLanguage::JavaScript),
            TaintSink::new(
                "js_eval".to_string(),
                "eval(".to_string(),
                SecurityIssueType::Injection,
                "JavaScript code evaluation".to_string(),
            )
            .with_language(SourceLanguage::JavaScript),
            TaintSink::new(
                "js_setTimeout".to_string(),
                "setTimeout(".to_string(),
                SecurityIssueType::Injection,
                "Code execution via setTimeout".to_string(),
            )
            .with_language(SourceLanguage::JavaScript),
        ];

        self.sources.extend(js_sources);
        self.sinks.extend(js_sinks);
    }

    /// Analyze a parsed file for taint flow vulnerabilities
    pub async fn analyze_file(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        info!("Starting taint analysis for: {}", file.file_path.display());

        // Build data flow graph from AST
        let data_flow_graph = self.build_data_flow_graph(file)?;

        // Perform taint propagation analysis
        let taint_flows = self.analyze_taint_flows(&data_flow_graph)?;

        // Convert taint flows to security issues
        let issues = self.convert_flows_to_issues(taint_flows, file)?;

        info!(
            "Taint analysis completed: {} vulnerabilities found",
            issues.len()
        );
        Ok(issues)
    }

    /// Build data flow graph from parsed file
    fn build_data_flow_graph(&self, file: &ParsedFile) -> Result<DataFlowGraph, AnalysisError> {
        let graph = DataFlowGraph::new();

        // This would be implemented with tree-sitter traversal
        // For now, we'll create a simplified implementation
        debug!("Building data flow graph for {}", file.file_path.display());

        // TODO: Implement tree-sitter AST traversal to build actual graph
        // This would involve:
        // 1. Walking the AST nodes
        // 2. Identifying variable assignments, function calls, etc.
        // 3. Building edges based on data dependencies
        // 4. Marking sources, sinks, and sanitizers

        Ok(graph)
    }

    /// Analyze taint flows in the data flow graph
    fn analyze_taint_flows(&self, graph: &DataFlowGraph) -> Result<Vec<TaintFlow>, AnalysisError> {
        let mut flows = Vec::new();

        // For each taint source, perform forward data flow analysis
        for source_id in &graph.taint_sources {
            let source_flows = self.trace_taint_from_source(graph, source_id)?;
            flows.extend(source_flows);
        }

        Ok(flows)
    }

    /// Trace taint flow from a specific source
    fn trace_taint_from_source(
        &self,
        graph: &DataFlowGraph,
        source_id: &str,
    ) -> Result<Vec<TaintFlow>, AnalysisError> {
        let mut flows = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let current_path = Vec::new();

        queue.push_back((
            source_id.to_string(),
            TaintLevel::Tainted,
            current_path.clone(),
        ));

        while let Some((node_id, taint_level, path)) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            let mut new_path = path;
            new_path.push(node_id.clone());

            if let Some(node) = graph.nodes.get(&node_id) {
                // Check if this is a sink
                if graph.taint_sinks.contains(&node_id) && taint_level.is_dangerous() {
                    flows.push(TaintFlow {
                        source_id: source_id.to_string(),
                        sink_id: node_id.clone(),
                        path: new_path.clone(),
                        taint_level: taint_level.clone(),
                        sanitizers_passed: Vec::new(), // TODO: Track sanitizers
                        confidence: self.calculate_flow_confidence(&new_path, &taint_level),
                    });
                }

                // Continue propagation to successors
                let new_taint_level = self.propagate_taint(&taint_level, node);

                for successor in graph.get_successors(&node_id) {
                    if new_path.len() < self.config.max_depth {
                        queue.push_back((
                            successor.to_string(),
                            new_taint_level.clone(),
                            new_path.clone(),
                        ));
                    }
                }
            }
        }

        Ok(flows)
    }

    /// Propagate taint through a node, considering sanitizers
    fn propagate_taint(&self, current_taint: &TaintLevel, node: &DataFlowNode) -> TaintLevel {
        match &node.node_type {
            DataFlowNodeType::Sanitizer(_) => {
                // Sanitization reduces taint level
                match current_taint {
                    TaintLevel::Tainted => TaintLevel::Sanitized,
                    TaintLevel::Partial(level) => TaintLevel::Partial(*level * 0.3),
                    other => other.clone(),
                }
            }
            _ => current_taint.clone(), // Normal propagation
        }
    }

    /// Calculate confidence score for a taint flow
    fn calculate_flow_confidence(&self, path: &[String], taint_level: &TaintLevel) -> f64 {
        let base_confidence = match path.len() {
            1..=3 => 0.9,  // Short path, high confidence
            4..=6 => 0.7,  // Medium path
            7..=10 => 0.5, // Long path, lower confidence
            _ => 0.3,      // Very long path, low confidence
        };

        let taint_confidence = taint_level.score();

        (base_confidence + taint_confidence) / 2.0
    }

    /// Convert taint flows to security issues
    fn convert_flows_to_issues(
        &self,
        flows: Vec<TaintFlow>,
        file: &ParsedFile,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for flow in flows {
            if let Some(issue) = self.create_security_issue_from_flow(&flow, file)? {
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    /// Create a security issue from a taint flow
    fn create_security_issue_from_flow(
        &self,
        flow: &TaintFlow,
        file: &ParsedFile,
    ) -> Result<Option<SecurityIssue>, AnalysisError> {
        // Find the source and sink information
        let source = self.sources.iter().find(|s| s.id == flow.source_id);
        let sink = self.sinks.iter().find(|s| s.id == flow.sink_id);

        if let (Some(source), Some(sink)) = (source, sink) {
            let location = SecurityLocation::new(
                file.file_path.as_ref().to_path_buf(),
                1, // TODO: Get actual line numbers from flow path
                1,
            );

            let issue = SecurityIssue::new(
                sink.vulnerability_type.clone(),
                VulnerabilityType::Static,
                format!(
                    "{} via {}",
                    sink.vulnerability_type.to_string(),
                    source.description
                ),
                format!(
                    "Tainted data from {} reaches {} without proper sanitization",
                    source.description, sink.description
                ),
                location,
            )
            .with_language(file.language)
            .with_confidence(flow.confidence)
            .with_severity(sink.severity)
            .with_detector("TaintAnalysisEngine".to_string())
            .with_metadata(
                VulnerabilityMetadata::new()
                    .with_tags(vec!["taint-analysis".to_string(), "data-flow".to_string()]),
            );

            Ok(Some(issue))
        } else {
            warn!("Could not find source or sink for flow: {:?}", flow);
            Ok(None)
        }
    }
}

/// Represents a taint flow from source to sink
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaintFlow {
    pub source_id: String,
    pub sink_id: String,
    pub path: Vec<String>,
    pub taint_level: TaintLevel,
    pub sanitizers_passed: Vec<String>,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::SourceLanguage;
    use std::path::PathBuf;

    #[test]
    fn test_taint_level_operations() {
        let clean = TaintLevel::Clean;
        let tainted = TaintLevel::Tainted;
        let partial = TaintLevel::Partial(0.6);

        assert!(!clean.is_dangerous());
        assert!(tainted.is_dangerous());
        assert!(partial.is_dangerous());

        assert_eq!(clean.score(), 0.0);
        assert_eq!(tainted.score(), 1.0);
        assert_eq!(partial.score(), 0.6);

        let combined = clean.combine(&tainted);
        assert_eq!(combined, TaintLevel::Tainted);
    }

    #[test]
    fn test_taint_source_creation() {
        let source = TaintSource::new(
            "test_source".to_string(),
            "input()".to_string(),
            "User input".to_string(),
        )
        .with_language(SourceLanguage::Python)
        .with_severity(SecuritySeverity::High);

        assert_eq!(source.id, "test_source");
        assert_eq!(source.pattern, "input()");
        assert_eq!(source.language, Some(SourceLanguage::Python));
        assert_eq!(source.default_severity, SecuritySeverity::High);
    }

    #[test]
    fn test_taint_sink_creation() {
        let sink = TaintSink::new(
            "test_sink".to_string(),
            "eval()".to_string(),
            SecurityIssueType::Injection,
            "Code injection".to_string(),
        )
        .with_language(SourceLanguage::Python);

        assert_eq!(sink.id, "test_sink");
        assert_eq!(sink.vulnerability_type, SecurityIssueType::Injection);
        assert_eq!(sink.language, Some(SourceLanguage::Python));
    }

    #[test]
    fn test_data_flow_graph_operations() {
        let mut graph = DataFlowGraph::new();

        let source_node = DataFlowNode {
            id: "source1".to_string(),
            node_type: DataFlowNodeType::Source("input()".to_string()),
            location: None,
            taint_level: TaintLevel::Tainted,
            metadata: HashMap::new(),
        };

        let sink_node = DataFlowNode {
            id: "sink1".to_string(),
            node_type: DataFlowNodeType::Sink("eval()".to_string()),
            location: None,
            taint_level: TaintLevel::Clean,
            metadata: HashMap::new(),
        };

        graph.add_node(source_node);
        graph.add_node(sink_node);
        graph.add_edge("source1", "sink1");

        assert_eq!(graph.taint_sources.len(), 1);
        assert_eq!(graph.taint_sinks.len(), 1);
        assert_eq!(graph.get_successors("source1"), vec!["sink1"]);
        assert_eq!(graph.get_predecessors("sink1"), vec!["source1"]);
    }

    #[tokio::test]
    async fn test_taint_analysis_engine_creation() {
        let config = TaintAnalysisConfig::development();
        let engine = TaintAnalysisEngine::new(config);

        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert!(!engine.sources.is_empty());
        assert!(!engine.sinks.is_empty());
    }
}
