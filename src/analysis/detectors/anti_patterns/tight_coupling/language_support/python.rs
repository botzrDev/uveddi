use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use crate::ast::{ParsedFile, SourceLanguage};
use std::path::Path;
use streaming_iterator::StreamingIterator;

use super::super::types::{Dependency, DependencyStrength};
use super::LanguageAnalyzer;

/// Python language analyzer for dependency extraction
#[derive(Debug, Default, Clone)]
pub struct PythonAnalyzer;

impl PythonAnalyzer {
    const IMPORT_QUERY: &'static str = r#"
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

    const CALL_QUERY: &'static str = r#"
        (call
          function: (attribute
            object: (identifier) @object
            attribute: (identifier) @method)) @method_call

        (call
          function: (identifier) @function) @function_call
    "#;

    const INHERITANCE_QUERY: &'static str = r#"
        (class_definition
          name: (identifier) @class_name
          superclasses: (argument_list
            (identifier) @parent_class)) @class_def
    "#;

    /// Extract import statements
    pub fn extract_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(),
            Self::IMPORT_QUERY,
        ) {
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

    /// Extract function and method calls
    pub fn extract_calls(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(),
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

    /// Extract inheritance relationships
    pub fn extract_inheritance(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(),
            Self::INHERITANCE_QUERY,
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

    /// Extract decorator usage (can indicate framework dependencies)
    pub fn extract_decorator_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let decorator_query = r#"
            (decorator
              (identifier) @decorator_name) @decorator_usage
        "#;

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(),
            decorator_query,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index
                        == query
                            .capture_index_for_name("decorator_name")
                            .unwrap_or(u32::MAX)
                    {
                        let decorator_name = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("unknown_decorator");

                        dependencies.push(Dependency {
                            from_component: ComponentNode::Function {
                                name: "decorated_function".to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                            },
                            to_component: ComponentNode::Function {
                                name: decorator_name.to_string(),
                                file_path: "decorator_definition".to_string(),
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

    /// Extract global variable usage
    pub fn extract_global_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let global_query = r#"
            (global_statement
              (identifier) @global_var) @global_usage
        "#;

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(),
            global_query,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index
                        == query
                            .capture_index_for_name("global_var")
                            .unwrap_or(u32::MAX)
                    {
                        let global_var = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("unknown_global");

                        dependencies.push(Dependency {
                            from_component: ComponentNode::Function {
                                name: "function_using_global".to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                            },
                            to_component: ComponentNode::Module {
                                path: format!("global::{}", global_var),
                            },
                            dependency_type: LocalDependencyType::Call,
                            line_number: Some(capture.node.start_position().row as u32 + 1),
                            strength: DependencyStrength::Strong, // Global usage is tight coupling
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }
}

impl LanguageAnalyzer for PythonAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            dependencies.extend(self.extract_imports(file_path, tree, &parsed_file.source)?);
            dependencies.extend(self.extract_calls(file_path, tree, &parsed_file.source)?);
            dependencies.extend(self.extract_inheritance(file_path, tree, &parsed_file.source)?);
            dependencies.extend(self.extract_decorator_dependencies(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_global_dependencies(
                file_path,
                tree,
                &parsed_file.source,
            )?);
        }

        Ok(dependencies)
    }

    fn get_language(&self) -> SourceLanguage {
        SourceLanguage::Python
    }
}
