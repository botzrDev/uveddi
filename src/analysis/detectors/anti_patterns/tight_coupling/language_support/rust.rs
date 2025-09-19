use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use std::path::Path;

use super::super::types::Dependency;
use super::{rust_extractor::RustExtractor, LanguageAnalyzer};

/// Rust language analyzer for dependency extraction
#[derive(Debug, Default, Clone)]
pub struct RustAnalyzer {
    extractor: RustExtractor,
}

impl RustAnalyzer {
    pub fn new() -> Self {
        Self {
            extractor: RustExtractor::new(),
        }
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            dependencies.extend(self.extractor.extract_use_declarations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extractor.extract_function_calls(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extractor.extract_struct_instantiations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extractor.extract_trait_implementations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extractor.extract_module_dependencies(
                file_path,
                tree,
                &parsed_file.source,
            )?);
        }

        Ok(dependencies)
    }

    fn get_language(&self) -> SourceLanguage {
        SourceLanguage::Rust
    }
}