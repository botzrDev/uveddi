use thiserror::Error;
use crate::ast::tree_sitter_impl::AstError;
use crate::analysis::detectors::dependency::ExtractionError;
use crate::analysis::component_extractor::ComponentExtractionError;
use crate::analysis::mermaid_generator::MermaidGenerationError;

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("AST error: {0}")]
    AstError(#[from] AstError),
    #[error("Dependency extraction error: {0}")]
    DependencyExtractionError(#[from] ExtractionError),
    #[error("Metric calculation error: {0}")]
    MetricCalculationError(String),
    #[error("Anti-pattern detection error: {0}")]
    AntiPatternDetectionError(String),
    #[error("Component extraction error: {0}")]
    ComponentExtractionError(#[from] ComponentExtractionError),
    #[error("Mermaid generation error: {0}")]
    MermaidGenerationError(#[from] MermaidGenerationError),
    #[error("Symbol resolution error: {0}")]
    SymbolResolutionError(String),
    #[error("Graph analysis error: {0}")]
    GraphAnalysisError(String),
    #[error("Tree-sitter query error: {0}")]
    QueryError(String),
}
