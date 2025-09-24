//! # Analysis Pipeline
//!
//! Orchestrates detector execution with the new analysis context.
//! Provides clear separation between business logic and AST parsing.

use super::AnalysisContext;
use crate::database::models::ArchitecturalIssue;
use std::sync::Arc;

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
}

/// Analysis pipeline that orchestrates detector execution
pub struct AnalysisPipeline {
    /// Registered detectors
    detectors: Vec<Box<dyn Detector>>,

    /// Context builder for preparing analysis context
    context_builder: Arc<ContextBuilder>,
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
    // TODO: Add necessary components for context building
}

impl AnalysisPipeline {
    /// Create a new analysis pipeline
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
            context_builder: Arc::new(ContextBuilder {}),
        }
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

        // Run all applicable detectors
        for detector in &self.detectors {
            if detector.supports_language(&context.file_info.language) {
                match detector.detect(&context) {
                    Ok(mut issues) => all_issues.append(&mut issues),
                    Err(e) => {
                        eprintln!("Detector {} failed: {}", detector.name(), e);
                        // Continue with other detectors
                    }
                }
            }
        }

        Ok(AnalysisResult {
            issues: all_issues,
            context,
            execution_time: start_time.elapsed(),
        })
    }
}