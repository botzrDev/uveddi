use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::path::Path;
use streaming_iterator::StreamingIterator;

use super::super::types::{Dependency, DependencyStrength};

/// Analyzes wildcard imports which create tight coupling
#[derive(Debug, Clone)]
pub struct WildcardAnalyzer;

impl WildcardAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Detect wildcard imports which can create tight coupling
    pub fn detect_wildcard_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut wildcard_deps = Vec::new();

        match language {
            SourceLanguage::Rust => {
                // Rust: use module::*;
                if let Ok(query) = Query::new(
                    &crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(),
                    r#"(use_declaration (scoped_use_list path: (identifier) @module "*")) @wildcard"#,
                ) {
                    wildcard_deps.extend(
                        self.extract_wildcard_dependencies(file_path, &query, tree, source)?,
                    );
                }
            }
            SourceLanguage::Python => {
                // Python: from module import *
                if let Ok(query) = Query::new(
                    &crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(),
                    r#"(import_from_statement module_name: (dotted_name) @module "*") @wildcard"#,
                ) {
                    wildcard_deps.extend(
                        self.extract_wildcard_dependencies(file_path, &query, tree, source)?,
                    );
                }
            }
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                // JavaScript: import * as name from 'module'
                if let Ok(query) = Query::new(
                    &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
                    r#"(import_statement (import_clause (namespace_import)) source: (string) @module) @wildcard"#,
                ) {
                    wildcard_deps.extend(
                        self.extract_wildcard_dependencies(file_path, &query, tree, source)?,
                    );
                }
            }
            _ => {
                // Not supported for other languages yet
            }
        }

        Ok(wildcard_deps)
    }

    /// Extract dependencies from wildcard import matches
    fn extract_wildcard_dependencies(
        &self,
        file_path: &Path,
        query: &Query,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let module_name = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("unknown_module");

                dependencies.push(Dependency {
                    from_component: ComponentNode::Module {
                        path: file_path.to_string_lossy().to_string(),
                    },
                    to_component: ComponentNode::Module {
                        path: module_name.to_string(),
                    },
                    dependency_type: LocalDependencyType::Import,
                    line_number: Some(capture.node.start_position().row as u32 + 1),
                    strength: DependencyStrength::Strong, // Wildcard imports are stronger coupling
                });
            }
        }

        Ok(dependencies)
    }
}

impl Default for WildcardAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
