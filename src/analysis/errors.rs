use crate::analysis::component_extractor::ComponentExtractionError;
use crate::analysis::detectors::dependency::ExtractionError;
use crate::analysis::mermaid_generator::MermaidGenerationError;
use crate::ast::tree_sitter_impl::AstError;
use crate::plugins::errors::PluginError;
use thiserror::Error;

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
    #[error("Detection error: {0}")]
    DetectionError(String),
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
    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),
    #[error("Other analysis error: {0}")]
    Other(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}
