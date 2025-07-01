use thiserror::Error;

#[derive(Debug, Error)]
pub enum UveddiError {
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Analysis engine error: {0}")]
    AnalysisEngine(#[from] crate::analysis::AnalysisError),
    #[error("AI error: {0}")]
    Ai(#[from] crate::ai::AiError),
    #[error("Report error: {0}")]
    Report(#[from] crate::report::ReportError),
    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("AST error: {0}")]
    Ast(#[from] crate::ast::tree_sitter::AstError),
    #[error("Generic error: {0}")]
    Generic(String),
}
