use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::path::Path;
use tracing::debug;

use super::super::types::{Dependency, DependencyStrength};

/// Detects import statements and analyzes coupling patterns
#[derive(Debug, Clone)]
pub struct ImportDetector;

impl ImportDetector {
    pub fn new() -> Self {
        Self
    }

    /// Analyze imports in a file and detect potential coupling issues
    pub fn analyze_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        match language {
            SourceLanguage::Rust => self.analyze_rust_imports(file_path, tree, source),
            SourceLanguage::Python => self.analyze_python_imports(file_path, tree, source),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                self.analyze_javascript_imports(file_path, tree, source)
            }
        }
    }

    /// Detect unused imports that increase coupling without benefit
    pub fn detect_unused_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<String>, AnalysisError> {
        // This would require cross-referencing imports with usage
        // For now, return empty list - would need symbol table analysis
        debug!("Unused import detection not yet implemented for {}", file_path.display());
        Ok(Vec::new())
    }

    fn analyze_rust_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let use_query = r#"
            (use_declaration
              argument: (scoped_use_list
                list: (use_list
                  (scoped_identifier
                    path: (identifier) @module
                    name: (identifier) @item))) @use_decl)

            (use_declaration
              argument: (scoped_identifier
                path: (identifier) @module
                name: (identifier) @item)) @use_decl
        "#;

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(), use_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let node_text = capture.node.utf8_text(source.as_bytes()).map_err(|e| {
                        AnalysisError::QueryError(format!("Failed to get node text: {}", e))
                    })?;

                    let from_component = ComponentNode::Module {
                        path: file_path.to_string_lossy().to_string(),
                    };

                    let to_component = ComponentNode::Module {
                        path: node_text.to_string(),
                    };

                    dependencies.push(Dependency {
                        from_component,
                        to_component,
                        dependency_type: LocalDependencyType::Import,
                        line_number: Some(capture.node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Medium,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn analyze_python_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let import_query = r#"
            (import_statement
              name: (dotted_name) @module) @import

            (import_from_statement
              module_name: (dotted_name) @module
              name: (dotted_name) @item) @from_import

            (import_from_statement
              module_name: (dotted_name) @module
              name: (import_list
                (dotted_name) @item)) @from_import_list
        "#;

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(), import_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

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
                        strength: DependencyStrength::Medium,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn analyze_javascript_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let import_query = r#"
            (import_statement
              source: (string) @module) @import

            (import_statement
              (import_clause
                (named_imports
                  (import_specifier
                    name: (identifier) @item)))
              source: (string) @module) @named_import

            (variable_declaration
              (variable_declarator
                name: (identifier) @var
                value: (call_expression
                  function: (identifier) @require
                  arguments: (arguments (string) @module)))) @require_call
        "#;

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), import_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index == query.capture_index_for_name("module").unwrap_or(u32::MAX) {
                        let module_name = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("unknown_module")
                            .trim_matches('"')
                            .trim_matches('\'');

                        dependencies.push(Dependency {
                            from_component: ComponentNode::Module {
                                path: file_path.to_string_lossy().to_string(),
                            },
                            to_component: ComponentNode::Module {
                                path: module_name.to_string(),
                            },
                            dependency_type: LocalDependencyType::Import,
                            line_number: Some(capture.node.start_position().row as u32 + 1),
                            strength: DependencyStrength::Medium,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }
}

impl Default for ImportDetector {
    fn default() -> Self {
        Self::new()
    }
}