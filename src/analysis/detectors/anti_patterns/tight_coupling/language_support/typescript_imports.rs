use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use std::path::Path;
use streaming_iterator::StreamingIterator;

use super::super::types::{Dependency, DependencyStrength};

/// Handles TypeScript/JavaScript import analysis
pub struct TypeScriptImportAnalyzer;

impl TypeScriptImportAnalyzer {
    const IMPORT_QUERY: &'static str = r#"
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

    pub fn new() -> Self {
        Self
    }

    /// Extract ES6 imports and CommonJS requires
    pub fn extract_es6_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
            Self::IMPORT_QUERY,
        ) {
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

    /// Extract CommonJS require statements
    pub fn extract_commonjs_requires(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let require_query = r#"
            (call_expression
              function: (identifier) @require_func
              arguments: (arguments (string) @module)) @require_call
        "#;

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
            require_query,
        ) {
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

    /// Extract TypeScript type dependencies
    pub fn extract_type_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let type_annotation_query = r#"
            (type_annotation
              (type_identifier) @type_name) @type_usage
        "#;

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
            type_annotation_query,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index
                        == query
                            .capture_index_for_name("type_name")
                            .unwrap_or(u32::MAX)
                    {
                        let type_name = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("unknown_type");

                        dependencies.push(Dependency {
                            from_component: ComponentNode::Class {
                                name: "type_user".to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                            },
                            to_component: ComponentNode::Class {
                                name: type_name.to_string(),
                                file_path: "type_definition".to_string(),
                            },
                            dependency_type: LocalDependencyType::Call,
                            line_number: Some(capture.node.start_position().row as u32 + 1),
                            strength: DependencyStrength::Medium,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Extract generic type constraints
    pub fn extract_generic_constraints(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let generic_constraint_query = r#"
            (type_parameter
              (constraint
                (type_identifier) @constraint_type)) @generic_param
        "#;

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
            generic_constraint_query,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index
                        == query
                            .capture_index_for_name("constraint_type")
                            .unwrap_or(u32::MAX)
                    {
                        let constraint_type = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("unknown_constraint");

                        dependencies.push(Dependency {
                            from_component: ComponentNode::Class {
                                name: "generic_user".to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                            },
                            to_component: ComponentNode::Class {
                                name: constraint_type.to_string(),
                                file_path: "constraint_definition".to_string(),
                            },
                            dependency_type: LocalDependencyType::Implementation,
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

impl Default for TypeScriptImportAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
