use std::path::{Path, PathBuf};
use log::debug;
use tree_sitter::{Query, QueryCursor};

use crate::ast::tree_sitter::{
    queries::{JAVASCRIPT_IMPORTS_QUERY, PYTHON_IMPORTS_QUERY, RUST_IMPORTS_QUERY},
    AstParser, ParsedFile, SourceLanguage,
};
pub use uveddi_plugin_api::models::{Dependency, DependencyType};

/// AST-based dependency extractor
pub struct DependencyExtractor {
    parser: AstParser,
}

impl DependencyExtractor {
    pub fn new() -> Result<Self, ExtractionError> {
        let parser = AstParser::new().map_err(ExtractionError::AstError)?;
        Ok(Self { parser })
    }

    /// Extract dependencies from a single file using AST parsing
    pub fn extract_from_file(&mut self, file_path: &Path) -> Result<Vec<Dependency>, ExtractionError> {
        let parsed_file = self
            .parser
            .parse_file(file_path)
            .map_err(ExtractionError::AstError)?;
        self.extract_from_ast(&parsed_file)
    }

    /// Extracts dependencies from a previously parsed file
    pub fn extract_from_ast(&self, parsed_file: &ParsedFile) -> Result<Vec<Dependency>, ExtractionError> {
        let (query_str, dependency_type) = match parsed_file.language {
            SourceLanguage::Rust => (RUST_IMPORTS_QUERY, DependencyType::Use),
            SourceLanguage::Python => (PYTHON_IMPORTS_QUERY, DependencyType::Import),
            SourceLanguage::JavaScript => (JAVASCRIPT_IMPORTS_QUERY, DependencyType::Import),
        };

        let query = Query::new(&parsed_file.tree.as_ref().expect("AST tree missing").language(), query_str)
            .map_err(|e| ExtractionError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, parsed_file.tree.as_ref().expect("AST tree missing").root_node(), parsed_file.source.as_bytes());

        let mut dependencies = Vec::new();
        for mat in matches {
            for capture in mat.captures {
                let node = capture.node;
                let line_number = node.start_position().row + 1;
                let mut module_name = node
                    .utf8_text(parsed_file.source.as_bytes())
                    .unwrap_or("")
                    .to_string();

                // For Rust, resolve `mod` statements to file paths
                if parsed_file.language == SourceLanguage::Rust && dependency_type == DependencyType::Use {
                    let mut potential_path = parsed_file.path.parent().unwrap().join(&module_name);
                    if !potential_path.exists() {
                        potential_path.set_extension("rs");
                        if !potential_path.exists() {
                             // Check for module/mod.rs
                            let mod_path = parsed_file.path.parent().unwrap().join(&module_name).join("mod.rs");
                            if mod_path.exists() {
                                potential_path = mod_path;
                            }
                        }
                    }
                    if potential_path.exists() {
                        module_name = potential_path.to_string_lossy().into_owned();
                    }
                }

                // Clean up the module name (e.g., remove quotes from strings)
                if module_name.starts_with('"') && module_name.ends_with('"')
                    || module_name.starts_with('\'') && module_name.ends_with('\'')
                {
                    module_name = module_name[1..module_name.len() - 1].to_string();
                }

                // Normalize JS/TS import paths to match file stem (e.g., './b.js' -> 'b')
                if let SourceLanguage::JavaScript = parsed_file.language {
                    if module_name.starts_with("./") {
                        let name = module_name.trim_start_matches("./");
                        if let Some(stripped) = name.strip_suffix(".js") {
                            module_name = stripped.to_string();
                        } else if let Some(stripped) = name.strip_suffix(".ts") {
                            module_name = stripped.to_string();
                        } else {
                            module_name = name.to_string();
                        }
                    }
                }

                if let SourceLanguage::Rust = parsed_file.language {
                    if let Some(parent) = parsed_file.path.parent() {
                        let mut path = parent.join(&module_name);
                        if !path.exists() {
                            path.set_extension("rs");
                        }
                        if path.exists() {
                            module_name = path.to_string_lossy().to_string();
                        }
                    }
                }

                dependencies.push(Dependency {
                    from_file: parsed_file.path.clone(),
                    to_module: module_name,
                    dependency_type: dependency_type.clone(),
                    line_number: Some(line_number as u32),
                });
            }
        }

        debug!(
            "Extracted {} dependencies from {}",
            dependencies.len(),
            parsed_file.path.display()
        );
        Ok(dependencies)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ExtractionError {
    #[error("AST error: {0}")]
    AstError(#[from] crate::ast::tree_sitter::AstError),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("IO error reading {0}: {1}")]
    IoError(PathBuf, std::io::Error),
}