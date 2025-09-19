pub mod rust;
pub mod python;
pub mod typescript;

pub use rust::RustAnalyzer;
pub use python::PythonAnalyzer;
pub use typescript::TypeScriptAnalyzer;

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use std::path::Path;

use super::types::Dependency;

/// Trait for language-specific dependency analysis
pub trait LanguageAnalyzer: Send + Sync {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError>;

    fn get_language(&self) -> SourceLanguage;
}