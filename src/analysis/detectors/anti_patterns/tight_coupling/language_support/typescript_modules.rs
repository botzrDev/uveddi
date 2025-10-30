use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use std::path::Path;
use streaming_iterator::StreamingIterator;

use super::super::types::{Dependency, DependencyStrength};

/// Handles TypeScript/JavaScript module and class analysis
#[derive(Debug, Clone)]
pub struct TypeScriptModuleAnalyzer;

impl TypeScriptModuleAnalyzer {
    const CALL_QUERY: &'static str = r#"
        (call_expression
          function: (member_expression
            object: (identifier) @object
            property: (property_identifier) @method)) @method_call

        (call_expression
          function: (identifier) @function) @function_call
    "#;

    const CLASS_EXTENDS_QUERY: &'static str = r#"
        (class_declaration
          name: (type_identifier) @class_name
          heritage: (class_heritage
            (extends_clause
              (identifier) @parent_class))) @class_def
    "#;

    const INTERFACE_IMPLEMENTS_QUERY: &'static str = r#"
        (class_declaration
          name: (type_identifier) @class_name
          heritage: (class_heritage
            (implements_clause
              (type_identifier) @interface_name))) @class_def
    "#;

    pub fn new() -> Self {
        Self
    }

    /// Extract function calls
    pub fn extract_function_calls(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
            Self::CALL_QUERY,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if let Some(capture) = m.captures.first() {
                    let function_name = capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_function");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Function {
                            name: "caller".to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Function {
                            name: function_name.to_string(),
                            file_path: "external".to_string(),
                        },
                        dependency_type: LocalDependencyType::Call,
                        line_number: Some(capture.node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Strong,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Extract class inheritance relationships
    pub fn extract_class_inheritance(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
            Self::CLASS_EXTENDS_QUERY,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if m.captures.len() >= 2 {
                    let class_name = m.captures[0]
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_class");
                    let parent_name = m.captures[1]
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_parent");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Class {
                            name: class_name.to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Class {
                            name: parent_name.to_string(),
                            file_path: "parent_definition".to_string(),
                        },
                        dependency_type: LocalDependencyType::Inheritance,
                        line_number: Some(m.captures[0].node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Strong,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Extract interface implementations
    pub fn extract_interface_implementations(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
            Self::INTERFACE_IMPLEMENTS_QUERY,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if m.captures.len() >= 2 {
                    let class_name = m.captures[0]
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_class");
                    let interface_name = m.captures[1]
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_interface");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Class {
                            name: class_name.to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Class {
                            name: interface_name.to_string(),
                            file_path: "interface_definition".to_string(),
                        },
                        dependency_type: LocalDependencyType::Implementation,
                        line_number: Some(m.captures[0].node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Medium,
                    });
                }
            }
        }

        Ok(dependencies)
    }
}

impl Default for TypeScriptModuleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
