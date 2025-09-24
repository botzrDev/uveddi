//! # Analysis Pipeline
//!
//! Orchestrates detector execution with the new analysis context.
//! Provides clear separation between business logic and AST parsing.

use super::performance::{AnalysisInstrumentation, AnalysisMetrics};
use super::AnalysisContext;
use crate::database::models::ArchitecturalIssue;
use crate::engine::analysis::context::{
    DependencySource, FileInfo, ProjectContext, ProjectDependency,
};
use crate::engine::parsing::{AstBuilder, LanguageParser, ParseResult};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Analysis pipeline error types
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Context building failed: {0}")]
    ContextBuildError(String),

    #[error("Detector execution failed: {0}")]
    DetectorError(String),

    #[error("Result aggregation failed: {0}")]
    AggregationError(String),
}

/// Result of analysis pipeline execution
#[derive(Debug)]
pub struct AnalysisResult {
    pub issues: Vec<ArchitecturalIssue>,
    pub context: AnalysisContext,
    pub execution_time: std::time::Duration,
    pub performance_metrics: Option<AnalysisMetrics>,
}

/// Analysis pipeline that orchestrates detector execution
pub struct AnalysisPipeline {
    /// Registered detectors
    detectors: Vec<Box<dyn Detector>>,

    /// Context builder for preparing analysis context
    context_builder: Arc<ContextBuilder>,

    /// Performance instrumentation enabled
    performance_enabled: bool,
}

/// Trait for analysis detectors using the new context
pub trait Detector: Send + Sync {
    /// Detect issues using the analysis context
    fn detect(&self, context: &AnalysisContext) -> Result<Vec<ArchitecturalIssue>, PipelineError>;

    /// Get detector name
    fn name(&self) -> &str;

    /// Check if detector supports the given language
    fn supports_language(&self, language: &crate::ast::SourceLanguage) -> bool;
}

/// Builds analysis context from parse results
pub struct ContextBuilder {
    ast_builder: Arc<AstBuilder>,
}

impl ContextBuilder {
    /// Create new context builder
    pub fn new(ast_builder: Arc<AstBuilder>) -> Self {
        Self { ast_builder }
    }

    /// Build analysis context from file path
    pub fn build_context(
        &self,
        file_path: &Path,
        project_context: ProjectContext,
    ) -> Result<AnalysisContext, PipelineError> {
        // Parse file using AstBuilder
        let parse_result = self
            .ast_builder
            .parse_file(file_path)
            .map_err(|e| PipelineError::ContextBuildError(format!("Parse failed: {}", e)))?;

        // Build file info
        let file_info = FileInfo {
            path: file_path.to_path_buf(),
            language: parse_result.language.clone(),
            lines_of_code: parse_result.source.lines().count(),
            size_bytes: parse_result.source.len(),
            modified_at: std::time::SystemTime::now(),
        };

        // Create analysis context
        Ok(AnalysisContext::new(
            file_info,
            parse_result.tree,
            parse_result.source,
            parse_result.symbols,
            parse_result.relations,
            project_context,
        ))
    }

    /// Build context from existing parse result
    pub fn build_from_parse_result(
        &self,
        parse_result: ParseResult,
        project_context: ProjectContext,
    ) -> AnalysisContext {
        let file_info = FileInfo {
            path: parse_result.path.clone(),
            language: parse_result.language.clone(),
            lines_of_code: parse_result.source.lines().count(),
            size_bytes: parse_result.source.len(),
            modified_at: parse_result.modified_at,
        };

        AnalysisContext::new(
            file_info,
            parse_result.tree,
            parse_result.source,
            parse_result.symbols,
            parse_result.relations,
            project_context,
        )
    }
}

impl AnalysisPipeline {
    /// Create a new analysis pipeline
    pub fn new(ast_builder: Arc<AstBuilder>) -> Self {
        Self {
            detectors: Vec::new(),
            context_builder: Arc::new(ContextBuilder::new(ast_builder)),
            performance_enabled: false,
        }
    }

    /// Enable performance instrumentation
    pub fn with_performance_instrumentation(mut self, enabled: bool) -> Self {
        self.performance_enabled = enabled;
        self
    }

    /// Add a detector to the pipeline
    pub fn with_detector(mut self, detector: Box<dyn Detector>) -> Self {
        self.detectors.push(detector);
        self
    }

    /// Analyze a file using the pipeline
    pub fn analyze(&self, context: AnalysisContext) -> Result<AnalysisResult, PipelineError> {
        let start_time = std::time::Instant::now();
        let mut all_issues = Vec::new();
        let mut instrumentation = AnalysisInstrumentation::new(self.performance_enabled);

        // Start detection phase
        instrumentation.start_phase("detection");

        // Run all applicable detectors
        for detector in &self.detectors {
            if detector.supports_language(&context.file_info.language) {
                let detector_start = Instant::now();

                match detector.detect(&context) {
                    Ok(mut issues) => {
                        let issue_count = issues.len();
                        all_issues.append(&mut issues);
                        instrumentation.record_issues_found(issue_count);
                    }
                    Err(e) => {
                        eprintln!("Detector {} failed: {}", detector.name(), e);
                        // Continue with other detectors
                    }
                }

                let detector_duration = detector_start.elapsed();
                instrumentation.record_detector_time(detector.name(), detector_duration);
            }
        }

        instrumentation.record_file_processed();
        let metrics = if self.performance_enabled {
            Some(instrumentation.finalize())
        } else {
            None
        };

        Ok(AnalysisResult {
            issues: all_issues,
            context,
            execution_time: start_time.elapsed(),
            performance_metrics: metrics,
        })
    }

    /// Analyze a file by path, building context automatically
    pub fn analyze_file(
        &self,
        file_path: &Path,
        project_context: ProjectContext,
    ) -> Result<AnalysisResult, PipelineError> {
        let context = self
            .context_builder
            .build_context(file_path, project_context)?;
        self.analyze(context)
    }

    /// Get registered detector count
    pub fn detector_count(&self) -> usize {
        self.detectors.len()
    }
}
