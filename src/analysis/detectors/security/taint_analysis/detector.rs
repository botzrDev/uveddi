//! Main taint analysis detector implementation

use crate::analysis::detectors::security::taint_analysis::{
    config::TaintAnalysisConfig,
    language_support::LanguageAnalyzerFactory,
    propagation::PropagationAnalyzer,
    sinks::UnifiedSinkDetector,
    sources::UnifiedSourceDetector,
    types::{LanguageTaintPatterns, SanitizationPoint, TaintSink, TaintSource},
};
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;
use tracing::info;

mod implementation;

/// Main taint analysis engine
pub struct TaintAnalysisEngine {
    config: TaintAnalysisConfig,
    source_detector: UnifiedSourceDetector,
    sink_detector: UnifiedSinkDetector,
    propagation_analyzer: PropagationAnalyzer,
    language_patterns: HashMap<SourceLanguage, LanguageTaintPatterns>,
    custom_sources: Vec<TaintSource>,
    custom_sinks: Vec<TaintSink>,
    custom_sanitizers: Vec<SanitizationPoint>,
}

impl TaintAnalysisEngine {
    /// Create a new taint analysis engine with the given configuration
    pub fn new(config: TaintAnalysisConfig) -> Result<Self, AnalysisError> {
        let mut engine = Self {
            config,
            source_detector: UnifiedSourceDetector::new(),
            sink_detector: UnifiedSinkDetector::new(),
            propagation_analyzer: PropagationAnalyzer::new(),
            language_patterns: HashMap::new(),
            custom_sources: Vec::new(),
            custom_sinks: Vec::new(),
            custom_sanitizers: Vec::new(),
        };

        engine.initialize_language_patterns()?;
        Ok(engine)
    }

    /// Analyze a parsed file for taint flow vulnerabilities
    pub async fn analyze_file(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        info!("Starting taint analysis for: {}", file.file_path.display());

        // Get language-specific analyzer
        let language_analyzer = LanguageAnalyzerFactory::create_analyzer(file.language);

        // Perform language-specific analysis
        let language_analysis = language_analyzer.analyze_language_constructs(file)?;

        // Build data flow graph from AST
        let data_flow_graph = self.build_data_flow_graph(file, &language_analysis)?;

        // Perform taint propagation analysis
        let taint_flows = self.analyze_taint_flows(&data_flow_graph)?;

        // Convert taint flows to security issues
        let mut issues = self.convert_flows_to_issues(taint_flows, file)?;

        // Add language-specific issues
        issues.extend(self.convert_language_issues_to_security_issues(
            language_analysis.language_specific_issues,
            file,
        )?);

        info!(
            "Taint analysis completed: {} vulnerabilities found",
            issues.len()
        );
        Ok(issues)
    }

    /// Add custom taint source
    pub fn add_custom_source(&mut self, source: TaintSource) {
        self.custom_sources.push(source);
    }

    /// Add custom taint sink
    pub fn add_custom_sink(&mut self, sink: TaintSink) {
        self.custom_sinks.push(sink);
    }

    /// Add custom sanitizer
    pub fn add_custom_sanitizer(&mut self, sanitizer: SanitizationPoint) {
        self.custom_sanitizers.push(sanitizer);
    }

    /// Get configuration
    pub fn config(&self) -> &TaintAnalysisConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TaintAnalysisConfig) {
        self.config = config;
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<SourceLanguage> {
        self.language_patterns.keys().cloned().collect()
    }

    /// Check if a language is supported
    pub fn supports_language(&self, language: SourceLanguage) -> bool {
        self.language_patterns.contains_key(&language)
    }

    /// Get statistics about the taint analysis
    pub fn get_analysis_stats(&self) -> TaintAnalysisStats {
        TaintAnalysisStats {
            supported_languages: self.supported_languages().len(),
            custom_sources: self.custom_sources.len(),
            custom_sinks: self.custom_sinks.len(),
            custom_sanitizers: self.custom_sanitizers.len(),
        }
    }
}

/// Statistics about the taint analysis configuration
#[derive(Debug, Clone)]
pub struct TaintAnalysisStats {
    pub supported_languages: usize,
    pub custom_sources: usize,
    pub custom_sinks: usize,
    pub custom_sanitizers: usize,
}
