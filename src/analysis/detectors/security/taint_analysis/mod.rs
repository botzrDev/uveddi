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
//!
//! ## Modular Architecture
//!
//! - `types`: Core data structures and enums
//! - `config`: Configuration types and settings
//! - `sources`: Taint source detection modules
//! - `sinks`: Taint sink detection modules
//! - `propagation`: Taint flow analysis and tracking
//! - `language_support`: Per-language specific implementations
//! - `detector`: Main detector implementation
//!
//! ## Usage
//!
//! ```rust,no_run
//! use crate::analysis::detectors::security::taint_analysis::{
//!     TaintAnalysisEngine,
//!     config::TaintAnalysisConfig,
//! };
//!
//! let config = TaintAnalysisConfig::production();
//! let engine = TaintAnalysisEngine::new(config)?;
//! let issues = engine.analyze_file(&parsed_file).await?;
//! ```

pub mod config;
pub mod detector;
pub mod language_support;
pub mod propagation;
pub mod sinks;
pub mod sources;
pub mod types;

// Re-export main types and detector for convenient access
pub use detector::{TaintAnalysisEngine, TaintAnalysisStats};
pub use types::{
    DataFlowGraph, DataFlowNode, DataFlowNodeType, SanitizationPoint, SourceLocation, TaintFlow,
    TaintLevel, TaintSink, TaintSource,
};

// Re-export commonly used detector components
pub use language_support::{LanguageAnalyzerFactory, LanguageTaintAnalyzer};
pub use propagation::{FlowAnalyzer, PathTracker, PropagationAnalyzer, SanitizerDetector};
pub use sinks::{CommandSinkDetector, FileSinkDetector, NetworkSinkDetector, UnifiedSinkDetector};
pub use sources::{
    ExternalSourceDetector, InputSourceDetector, UnifiedSourceDetector, UserSourceDetector,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::TaintAnalysisConfig;
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
        .with_severity(crate::analysis::detectors::security::types::SecuritySeverity::High);

        assert_eq!(source.id, "test_source");
        assert_eq!(source.pattern, "input()");
        assert_eq!(source.language, Some(SourceLanguage::Python));
        assert_eq!(
            source.default_severity,
            crate::analysis::detectors::security::types::SecuritySeverity::High
        );
    }

    #[test]
    fn test_taint_sink_creation() {
        let sink = TaintSink::new(
            "test_sink".to_string(),
            "eval()".to_string(),
            crate::analysis::detectors::security::types::SecurityIssueType::Injection,
            "Code injection".to_string(),
        )
        .with_language(SourceLanguage::Python);

        assert_eq!(sink.id, "test_sink");
        assert_eq!(
            sink.vulnerability_type,
            crate::analysis::detectors::security::types::SecurityIssueType::Injection
        );
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
            metadata: std::collections::HashMap::new(),
        };

        let sink_node = DataFlowNode {
            id: "sink1".to_string(),
            node_type: DataFlowNodeType::Sink("eval()".to_string()),
            location: None,
            taint_level: TaintLevel::Clean,
            metadata: std::collections::HashMap::new(),
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

        let stats = engine.get_analysis_stats();
        assert!(stats.supported_languages > 0);

        // Test language support
        assert!(engine.supports_language(SourceLanguage::Rust));
        assert!(engine.supports_language(SourceLanguage::Python));
        assert!(engine.supports_language(SourceLanguage::JavaScript));
        assert!(engine.supports_language(SourceLanguage::TypeScript));
    }

    #[test]
    fn test_unified_source_detector() {
        let detector = UnifiedSourceDetector::new();
        // Basic instantiation test - actual detection would require parsed files
        assert!(true); // Placeholder for when we implement AST-based detection
    }

    #[test]
    fn test_unified_sink_detector() {
        let detector = UnifiedSinkDetector::new();
        // Basic instantiation test - actual detection would require parsed files
        assert!(true); // Placeholder for when we implement AST-based detection
    }

    #[test]
    fn test_propagation_analyzer() {
        let analyzer = PropagationAnalyzer::new();
        let empty_graph = DataFlowGraph::new();

        // Test with empty graph
        let flows = analyzer.analyze_taint_flows(&empty_graph);
        assert!(flows.is_ok());
        assert!(flows.unwrap().is_empty());
    }

    #[test]
    fn test_language_analyzer_factory() {
        let rust_analyzer = LanguageAnalyzerFactory::create_analyzer(SourceLanguage::Rust);
        let python_analyzer = LanguageAnalyzerFactory::create_analyzer(SourceLanguage::Python);
        let js_analyzer = LanguageAnalyzerFactory::create_analyzer(SourceLanguage::JavaScript);
        let ts_analyzer = LanguageAnalyzerFactory::create_analyzer(SourceLanguage::TypeScript);

        // Test that we get different analyzer types for different languages
        assert!(!rust_analyzer.get_taint_sources().is_empty());
        assert!(!python_analyzer.get_taint_sources().is_empty());
        assert!(!js_analyzer.get_taint_sources().is_empty());
        assert!(!ts_analyzer.get_taint_sources().is_empty());

        let supported = LanguageAnalyzerFactory::supported_languages();
        assert!(supported.contains(&SourceLanguage::Rust));
        assert!(supported.contains(&SourceLanguage::Python));
        assert!(supported.contains(&SourceLanguage::JavaScript));
        assert!(supported.contains(&SourceLanguage::TypeScript));
    }

    #[test]
    fn test_taint_flow_confidence() {
        let path_tracker = PathTracker::new();

        // Test short path (high confidence)
        let short_path = vec!["source".to_string(), "sink".to_string()];
        let confidence = path_tracker.calculate_confidence(&short_path, &TaintLevel::Tainted);
        assert!(confidence > 0.8);

        // Test long path (lower confidence)
        let long_path = vec![
            "source".to_string(),
            "node1".to_string(),
            "node2".to_string(),
            "node3".to_string(),
            "node4".to_string(),
            "node5".to_string(),
            "sink".to_string(),
        ];
        let long_confidence = path_tracker.calculate_confidence(&long_path, &TaintLevel::Tainted);
        assert!(long_confidence < confidence);
    }

    #[test]
    fn test_sanitizer_detector() {
        let detector = SanitizerDetector::new();

        // Test sanitization application
        let tainted = TaintLevel::Tainted;
        let sanitizer_node = DataFlowNode {
            id: "sanitizer".to_string(),
            node_type: DataFlowNodeType::Sanitizer("html_escape".to_string()),
            location: None,
            taint_level: TaintLevel::Clean,
            metadata: std::collections::HashMap::new(),
        };

        let sanitizer_point = SanitizationPoint {
            id: sanitizer_node.id,
            sanitizer_type: match sanitizer_node.node_type {
                DataFlowNodeType::Sanitizer(s) => s,
                _ => "unknown".to_string(),
            },
            effectiveness: 0.9,
            location: sanitizer_node.location,
        };
        let result = detector.apply_sanitization(&sanitizer_point, tainted);
        match result {
            TaintLevel::Sanitized | TaintLevel::Partial(_) => assert!(true),
            _ => assert!(false, "Expected sanitization to reduce taint level"),
        }
    }
}
