use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use std::path::Path;

use super::super::types::Dependency;
use super::{
    typescript_imports::TypeScriptImportAnalyzer,
    typescript_modules::TypeScriptModuleAnalyzer,
    LanguageAnalyzer,
};

/// TypeScript/JavaScript language analyzer for dependency extraction
#[derive(Debug, Default, Clone)]
pub struct TypeScriptAnalyzer {
    import_analyzer: TypeScriptImportAnalyzer,
    module_analyzer: TypeScriptModuleAnalyzer,
}

impl TypeScriptAnalyzer {
    pub fn new() -> Self {
        Self {
            import_analyzer: TypeScriptImportAnalyzer::new(),
            module_analyzer: TypeScriptModuleAnalyzer::new(),
        }
    }
}

impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            dependencies.extend(self.import_analyzer.extract_es6_imports(file_path, tree, &parsed_file.source)?);
            dependencies.extend(self.import_analyzer.extract_commonjs_requires(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.module_analyzer.extract_function_calls(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.module_analyzer.extract_class_inheritance(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.module_analyzer.extract_interface_implementations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.import_analyzer.extract_type_dependencies(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.import_analyzer.extract_generic_constraints(
                file_path,
                tree,
                &parsed_file.source,
            )?);
        }

        Ok(dependencies)
    }

    fn get_language(&self) -> SourceLanguage {
        SourceLanguage::JavaScript // Also handles TypeScript
    }
}