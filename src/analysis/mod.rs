pub mod analysis_engine;
pub mod anti_patterns;
pub mod cycle_detector;
pub mod dependency_extractor;
pub mod dependency_graph;

use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::ast::tree_sitter::ParsedFile;
use std::any::Any;

/// Core analysis trait for all detectors
pub trait AnalysisDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
    fn get_detector_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("AST error: {0}")]
    Ast(#[from] crate::ast::tree_sitter::AstError),
    #[error("Analysis error: {0}")]
    Generic(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Extraction error: {0}")]
    Extraction(#[from] crate::analysis::dependency_extractor::ExtractionError),
    #[error("Report error: {0}")]
    Report(#[from] crate::report::ReportError),
    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),
    #[error("Invalid input path: {0}")]
    InvalidInputPath(std::path::PathBuf),
}