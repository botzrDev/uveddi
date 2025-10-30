//! Unit tests for taint analysis engine

use uveddi::analysis::detectors::security::taint_analysis::*;
use uveddi::analysis::detectors::security::types::*;
use uveddi::analysis::detectors::security::config::TaintAnalysisConfig;
use uveddi::ast::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use std::collections::HashMap;

mod test_helpers;
use test_helpers::*;

#[test]
fn test_taint_variable_creation() {
    let var = TaintVariable::new("user_input".to_string(), TaintType::Source);
    
    assert_eq!(var.name, "user_input");
    assert_eq!(var.taint_type, TaintType::Source);
    assert!(var.id.len() > 0);
    assert!(var.propagation_path.is_empty());
}

#[test]
fn test_taint_variable_with_metadata() {
    let mut var = TaintVariable::new("sql_query".to_string(), TaintType::Sink);
    var.source_location = Some(SecurityLocation::new(PathBuf::from("test.py"), 10, 10));
    var.confidence = 0.8;
    
    assert!(var.source_location.is_some());
    assert_eq!(var.confidence, 0.8);
}

#[test]
fn test_data_flow_edge() {
    let source = TaintVariable::new("input".to_string(), TaintType::Source);
    let sink = TaintVariable::new("output".to_string(), TaintType::Sink);
    
    let edge = DataFlowEdge {
        from_variable: source.id.clone(),
        to_variable: sink.id.clone(),
        edge_type: FlowType::DirectAssignment,
        line_number: 42,
        confidence: 0.9,
        sanitized: false,
    };
    
    assert_eq!(edge.from_variable, source.id);
    assert_eq!(edge.to_variable, sink.id);
    assert_eq!(edge.edge_type, FlowType::DirectAssignment);
    assert_eq!(edge.line_number, 42);
    assert!(!edge.sanitized);
}

#[test]
fn test_data_flow_graph_creation() {
    let config = TaintAnalysisConfig::default();
    let graph = DataFlowGraph::new(config);
    
    assert!(graph.variables.is_empty());
    assert!(graph.edges.is_empty());
    assert!(graph.sources.is_empty());
    assert!(graph.sinks.is_empty());
    assert!(graph.sanitizers.is_empty());
}

#[test]
fn test_data_flow_graph_add_variable() {
    let config = TaintAnalysisConfig::default();
    let mut graph = DataFlowGraph::new(config);
    
    let var = TaintVariable::new("test_var".to_string(), TaintType::Source);
    let var_id = var.id.clone();
    graph.add_variable(var);
    
    assert_eq!(graph.variables.len(), 1);
    assert!(graph.variables.contains_key(&var_id));
    assert_eq!(graph.sources.len(), 1);
    assert!(graph.sources.contains(&var_id));
}

#[test]
fn test_data_flow_graph_add_edge() {
    let config = TaintAnalysisConfig::default();
    let mut graph = DataFlowGraph::new(config);
    
    let source = TaintVariable::new("source".to_string(), TaintType::Source);
    let sink = TaintVariable::new("sink".to_string(), TaintType::Sink);
    
    let source_id = source.id.clone();
    let sink_id = sink.id.clone();
    
    graph.add_variable(source);
    graph.add_variable(sink);
    
    let edge = DataFlowEdge {
        from_variable: source_id.clone(),
        to_variable: sink_id.clone(),
        edge_type: FlowType::DirectAssignment,
        line_number: 10,
        confidence: 0.8,
        sanitized: false,
    };
    
    graph.add_edge(edge);
    
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_taint_analysis_engine_creation() {
    let config = TaintAnalysisConfig::default();
    let engine = TaintAnalysisEngine::new(config);
    
    // Check that language patterns were initialized
    assert!(engine.rust_patterns.sources.len() > 0);
    assert!(engine.python_patterns.sources.len() > 0);
    assert!(engine.javascript_patterns.sources.len() > 0);
}

#[test]
fn test_taint_patterns_rust() {
    let patterns = TaintPatterns::rust();
    
    assert!(patterns.sources.contains(&"std::env::args".to_string()));
    assert!(patterns.sources.contains(&"std::env::var".to_string()));
    assert!(patterns.sinks.contains(&"std::process::Command::new".to_string()));
    assert!(patterns.sinks.contains(&"sqlx::query".to_string()));
    assert!(patterns.sanitizers.contains(&"html_escape::encode".to_string()));
}

#[test]
fn test_taint_patterns_python() {
    let patterns = TaintPatterns::python();
    
    assert!(patterns.sources.contains(&"input(".to_string()));
    assert!(patterns.sources.contains(&"sys.argv".to_string()));
    assert!(patterns.sources.contains(&"request.args".to_string()));
    assert!(patterns.sinks.contains(&"eval(".to_string()));
    assert!(patterns.sinks.contains(&"os.system".to_string()));
    assert!(patterns.sinks.contains(&"cursor.execute".to_string()));
    assert!(patterns.sanitizers.contains(&"html.escape".to_string()));
}

#[test]
fn test_taint_patterns_javascript() {
    let patterns = TaintPatterns::javascript();
    
    assert!(patterns.sources.contains(&"req.query".to_string()));
    assert!(patterns.sources.contains(&"req.body".to_string()));
    assert!(patterns.sources.contains(&"location.search".to_string()));
    assert!(patterns.sinks.contains(&"eval(".to_string()));
    assert!(patterns.sinks.contains(&"document.write".to_string()));
    assert!(patterns.sinks.contains(&"innerHTML".to_string()));
    assert!(patterns.sanitizers.contains(&"DOMPurify.sanitize".to_string()));
}

#[tokio::test]
async fn test_taint_analysis_engine_analyze_empty_file() {
    let config = TaintAnalysisConfig::default();
    let engine = TaintAnalysisEngine::new(config);
    
    // Create a test file that doesn't exist (will cause IO error)
    let file = create_test_parsed_file("/non/existent/file.py", SourceLanguage::Python);
    
    let result = engine.analyze(&file).await;
    assert!(result.is_err()); // Should fail due to missing file
}

#[test]
fn test_vulnerability_path_creation() {
    let path = VulnerabilityPath {
        source_variable: "user_input".to_string(),
        sink_variable: "sql_query".to_string(),
        path_variables: vec!["intermediate1".to_string(), "intermediate2".to_string()],
        path_length: 3,
        confidence: 0.85,
        is_sanitized: false,
    };
    
    assert_eq!(path.source_variable, "user_input");
    assert_eq!(path.sink_variable, "sql_query");
    assert_eq!(path.path_variables.len(), 2);
    assert_eq!(path.path_length, 3);
    assert!(!path.is_sanitized);
}

#[test]
fn test_flow_type_variants() {
    let direct = FlowType::DirectAssignment;
    let function = FlowType::FunctionCall;
    let field = FlowType::FieldAccess;
    let array = FlowType::ArrayAccess;
    let conditional = FlowType::ConditionalFlow;
    let loop_flow = FlowType::LoopFlow;
    
    // Test that all variants can be created
    assert!(matches!(direct, FlowType::DirectAssignment));
    assert!(matches!(function, FlowType::FunctionCall));
    assert!(matches!(field, FlowType::FieldAccess));
    assert!(matches!(array, FlowType::ArrayAccess));
    assert!(matches!(conditional, FlowType::ConditionalFlow));
    assert!(matches!(loop_flow, FlowType::LoopFlow));
}

#[test]
fn test_taint_type_variants() {
    let source = TaintType::Source;
    let sink = TaintType::Sink;
    let sanitizer = TaintType::Sanitizer;
    let intermediate = TaintType::Intermediate;
    
    assert!(matches!(source, TaintType::Source));
    assert!(matches!(sink, TaintType::Sink));
    assert!(matches!(sanitizer, TaintType::Sanitizer));
    assert!(matches!(intermediate, TaintType::Intermediate));
}

#[test]
fn test_taint_analysis_config_context_sensitivity() {
    let mut config = TaintAnalysisConfig::default();
    
    assert!(config.context_sensitive);
    assert!(config.field_sensitive);
    assert!(config.track_implicit_flows);
    
    // Test modifying config
    config.context_sensitive = false;
    config.field_sensitive = false;
    
    assert!(!config.context_sensitive);
    assert!(!config.field_sensitive);
}

#[test]
fn test_data_flow_graph_find_paths() {
    let config = TaintAnalysisConfig::default();
    let mut graph = DataFlowGraph::new(config);
    
    // Create a simple flow: source -> intermediate -> sink
    let source = TaintVariable::new("source".to_string(), TaintType::Source);
    let intermediate = TaintVariable::new("intermediate".to_string(), TaintType::Intermediate);
    let sink = TaintVariable::new("sink".to_string(), TaintType::Sink);
    
    let source_id = source.id.clone();
    let intermediate_id = intermediate.id.clone();
    let sink_id = sink.id.clone();
    
    graph.add_variable(source);
    graph.add_variable(intermediate);
    graph.add_variable(sink);
    
    // Add edges
    graph.add_edge(DataFlowEdge {
        from_variable: source_id.clone(),
        to_variable: intermediate_id.clone(),
        edge_type: FlowType::DirectAssignment,
        line_number: 1,
        confidence: 0.9,
        sanitized: false,
    });
    
    graph.add_edge(DataFlowEdge {
        from_variable: intermediate_id.clone(),
        to_variable: sink_id.clone(),
        edge_type: FlowType::DirectAssignment,
        line_number: 2,
        confidence: 0.9,
        sanitized: false,
    });
    
    // Find paths should identify the vulnerability path
    let paths = graph.find_vulnerability_paths();
    assert!(paths.len() > 0);
    
    let path = &paths[0];
    assert_eq!(path.source_variable, source_id);
    assert_eq!(path.sink_variable, sink_id);
    assert!(path.path_variables.contains(&intermediate_id));
}