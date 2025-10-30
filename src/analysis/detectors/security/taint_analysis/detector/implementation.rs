//! Implementation helpers for taint analysis detector

use crate::analysis::detectors::security::taint_analysis::{
    config::TaintAnalysisConfig,
    language_support::{LanguageAnalyzerFactory, LanguageTaintAnalyzer},
    types::{
        DataFlowGraph, LanguageTaintPatterns, SanitizationPoint, TaintFlow, TaintSink, TaintSource,
    },
};
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
    VulnerabilityType,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;
use tracing::{debug, warn};

use super::TaintAnalysisEngine;

impl TaintAnalysisEngine {
    /// Initialize language-specific patterns
    pub(super) fn initialize_language_patterns(&mut self) -> Result<(), AnalysisError> {
        let supported_languages = vec![
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ];

        for language in supported_languages {
            let analyzer = LanguageAnalyzerFactory::create_analyzer(language);
            let patterns = analyzer.get_taint_patterns();
            self.language_patterns.insert(language, patterns);
        }

        Ok(())
    }

    /// Build data flow graph from parsed file
    pub(super) fn build_data_flow_graph(
        &self,
        file: &ParsedFile,
        language_analysis: &crate::analysis::detectors::security::taint_analysis::language_support::LanguageAnalysisResult,
    ) -> Result<DataFlowGraph, AnalysisError> {
        let mut graph = DataFlowGraph::new();

        debug!("Building data flow graph for {}", file.file_path.display());

        // Add nodes for detected sources, sinks, and sanitizers
        self.add_source_nodes(&mut graph, &language_analysis.sources_found)?;
        self.add_sink_nodes(&mut graph, &language_analysis.sinks_found)?;
        self.add_sanitizer_nodes(&mut graph, &language_analysis.sanitizers_found)?;

        // TODO: Implement tree-sitter AST traversal to build actual graph
        // This would involve:
        // 1. Walking the AST nodes
        // 2. Identifying variable assignments, function calls, etc.
        // 3. Building edges based on data dependencies
        // 4. Marking sources, sinks, and sanitizers based on patterns

        Ok(graph)
    }

    /// Add source nodes to the data flow graph
    pub(super) fn add_source_nodes(
        &self,
        graph: &mut DataFlowGraph,
        sources: &[TaintSource],
    ) -> Result<(), AnalysisError> {
        for source in sources {
            let node = crate::analysis::detectors::security::taint_analysis::types::DataFlowNode {
                id: source.id.clone(),
                node_type: crate::analysis::detectors::security::taint_analysis::types::DataFlowNodeType::Source(source.pattern.clone()),
                location: source.location.clone(),
                taint_level: crate::analysis::detectors::security::taint_analysis::types::TaintLevel::Tainted,
                metadata: source.metadata.clone(),
            };
            graph.add_node(node);
        }
        Ok(())
    }

    /// Add sink nodes to the data flow graph
    pub(super) fn add_sink_nodes(
        &self,
        graph: &mut DataFlowGraph,
        sinks: &[TaintSink],
    ) -> Result<(), AnalysisError> {
        for sink in sinks {
            let node = crate::analysis::detectors::security::taint_analysis::types::DataFlowNode {
                id: sink.id.clone(),
                node_type: crate::analysis::detectors::security::taint_analysis::types::DataFlowNodeType::Sink(sink.pattern.clone()),
                location: sink.location.clone(),
                taint_level: crate::analysis::detectors::security::taint_analysis::types::TaintLevel::Clean,
                metadata: HashMap::new(),
            };
            graph.add_node(node);
        }
        Ok(())
    }

    /// Add sanitizer nodes to the data flow graph
    pub(super) fn add_sanitizer_nodes(
        &self,
        graph: &mut DataFlowGraph,
        sanitizers: &[SanitizationPoint],
    ) -> Result<(), AnalysisError> {
        for sanitizer in sanitizers {
            let node = crate::analysis::detectors::security::taint_analysis::types::DataFlowNode {
                id: sanitizer.id.clone(),
                node_type: crate::analysis::detectors::security::taint_analysis::types::DataFlowNodeType::Sanitizer(sanitizer.pattern.clone()),
                location: sanitizer.location.clone(),
                taint_level: crate::analysis::detectors::security::taint_analysis::types::TaintLevel::Clean,
                metadata: HashMap::new(),
            };
            graph.add_node(node);
        }
        Ok(())
    }

    /// Analyze taint flows in the data flow graph
    pub(super) fn analyze_taint_flows(
        &self,
        graph: &DataFlowGraph,
    ) -> Result<Vec<TaintFlow>, AnalysisError> {
        self.propagation_analyzer.analyze_taint_flows(graph)
    }

    /// Convert taint flows to security issues
    pub(super) fn convert_flows_to_issues(
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
    pub(super) fn create_security_issue_from_flow(
        &self,
        flow: &TaintFlow,
        file: &ParsedFile,
    ) -> Result<Option<SecurityIssue>, AnalysisError> {
        // Get language-specific analyzer for detailed information
        let language_analyzer = LanguageAnalyzerFactory::create_analyzer(file.language);
        let sources = language_analyzer.get_taint_sources();
        let sinks = language_analyzer.get_taint_sinks();

        // Find the source and sink information
        let source = sources.iter().find(|s| s.id == flow.source_id);
        let sink = sinks.iter().find(|s| s.id == flow.sink_id);

        if let (Some(source), Some(sink)) = (source, sink) {
            let location = SecurityLocation::new(
                file.file_path.as_ref().clone(),
                1, // TODO: Get actual line numbers from flow path
                1,
            );

            let mut issue = SecurityIssue::new(
                sink.vulnerability_type.clone(),
                VulnerabilityType::Static,
                format!(
                    "{} via {}",
                    sink.vulnerability_type.to_string(),
                    source.description
                ),
                format!(
                    "Tainted data from {} reaches {} without proper sanitization. Flow path: {}",
                    source.description,
                    sink.description,
                    flow.path.join(" -> ")
                ),
                location,
            )
            .with_language(file.language)
            .with_confidence(flow.confidence)
            .with_severity(sink.severity)
            .with_detector("TaintAnalysisEngine".to_string());

            // Add flow-specific metadata
            let mut metadata = VulnerabilityMetadata::new().with_tags(vec![
                "taint-analysis".to_string(),
                "data-flow".to_string(),
                format!("source:{}", source.pattern),
                format!("sink:{}", sink.pattern),
                format!("flow-length:{}", flow.path.len()),
                format!("taint-level:{:?}", flow.taint_level),
            ]);

            if !flow.sanitizers_passed.is_empty() {
                let mut new_tags = metadata.tags.clone();
                new_tags.push(format!("sanitizers:{}", flow.sanitizers_passed.join(",")));
                metadata = metadata.with_tags(new_tags);
            }

            issue = issue.with_metadata(metadata);

            Ok(Some(issue))
        } else {
            warn!("Could not find source or sink for flow: {:?}", flow);
            Ok(None)
        }
    }

    /// Convert language-specific issues to security issues
    pub(super) fn convert_language_issues_to_security_issues(
        &self,
        language_issues: Vec<crate::analysis::detectors::security::taint_analysis::language_support::LanguageSpecificIssue>,
        file: &ParsedFile,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut security_issues = Vec::new();

        for lang_issue in language_issues {
            let location = lang_issue
                .location
                .map(|loc| SecurityLocation::new(loc.file_path, loc.line as i32, loc.column as i32))
                .unwrap_or_else(|| SecurityLocation::new(file.file_path.as_ref().clone(), 1, 1));

            let severity = match lang_issue.severity {
                s if s >= 0.8 => SecuritySeverity::Critical,
                s if s >= 0.6 => SecuritySeverity::High,
                s if s >= 0.4 => SecuritySeverity::Medium,
                _ => SecuritySeverity::Low,
            };

            let issue = SecurityIssue::new(
                SecurityIssueType::Injection, // Default, could be more specific
                VulnerabilityType::Static,
                lang_issue.issue_type.clone(),
                lang_issue.description,
                location,
            )
            .with_language(file.language)
            .with_severity(severity)
            .with_confidence(lang_issue.severity)
            .with_detector("TaintAnalysisEngine".to_string())
            .with_metadata(VulnerabilityMetadata::new().with_tags(vec![
                "language-specific".to_string(),
                format!("{:?}", lang_issue.language).to_lowercase(),
            ]));

            security_issues.push(issue);
        }

        Ok(security_issues)
    }
}
