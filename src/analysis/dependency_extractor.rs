use std::path::{Path, PathBuf};
use log::debug;
use tree_sitter::{Query, QueryCursor};

use crate::ast::tree_sitter::{
    queries::{JAVASCRIPT_IMPORTS_QUERY, PYTHON_IMPORTS_QUERY, RUST_IMPORTS_QUERY},
    AstParser, ParsedFile, SourceLanguage,
};

/// Represents a dependency relationship between modules
#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    pub from_file: PathBuf,
    pub to_module: String,
    pub dependency_type: DependencyType,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Use,
    Mod,
    External,
    Import, // Generic for Python/JS
}

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

        let query = Query::new(parsed_file.tree.language(), query_str)
            .map_err(|e| ExtractionError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, parsed_file.tree.root_node(), parsed_file.source.as_bytes());

        let mut dependencies = Vec::new();
        for mat in matches {
            for capture in mat.captures {
                let node = capture.node;
                let line_number = node.start_position().row + 1;
                let mut module_name = node
                    .utf8_text(parsed_file.source.as_bytes())
                    .unwrap_or("")
                    .to_string();

                // Clean up the module name (e.g., remove quotes from strings)
                if module_name.starts_with('"') && module_name.ends_with('"')
                    || module_name.starts_with('\'') && module_name.ends_with('\'')
                {
                    module_name = module_name[1..module_name.len() - 1].to_string();
                }

                dependencies.push(Dependency {
                    from_file: parsed_file.path.clone(),
                    to_module: module_name,
                    dependency_type: dependency_type.clone(),
                    line_number,
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