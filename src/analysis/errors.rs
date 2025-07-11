use thiserror::Error;
use crate::ast::tree_sitter_impl::AstError;

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("AST error: {0}")]
    AstError(#[from] AstError),
    #[error("Dependency extraction error: {0}")]
    DependencyExtractionError(String),
    #[error("Metric calculation error: {0}")]
    MetricCalculationError(String),
    #[error("Anti-pattern detection error: {0}")]
    AntiPatternDetectionError(String),
    // Add other analysis errors as needed
}
