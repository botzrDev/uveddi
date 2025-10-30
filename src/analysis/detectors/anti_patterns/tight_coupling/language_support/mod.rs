pub mod python;
pub mod rust;
pub mod rust_extractor;
pub mod rust_queries;
pub mod typescript;
pub mod typescript_imports;
pub mod typescript_modules;

pub use python::PythonAnalyzer;
pub use rust::RustAnalyzer;
pub use rust_extractor::RustExtractor;
pub use rust_queries::RustQueries;
pub use typescript::TypeScriptAnalyzer;
pub use typescript_imports::TypeScriptImportAnalyzer;
pub use typescript_modules::TypeScriptModuleAnalyzer;

use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
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
