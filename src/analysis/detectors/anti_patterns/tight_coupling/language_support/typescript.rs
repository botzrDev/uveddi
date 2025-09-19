use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use std::path::Path;
use streaming_iterator::StreamingIterator;

use super::super::types::{Dependency, DependencyStrength};
use super::LanguageAnalyzer;

/// TypeScript/JavaScript language analyzer for dependency extraction
#[derive(Debug, Default, Clone)]
pub struct TypeScriptAnalyzer;

impl TypeScriptAnalyzer {
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

    /// Extract ES6 imports and CommonJS requires
    pub fn extract_es6_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), Self::IMPORT_QUERY)
        {
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

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), require_query) {
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

    /// Extract function calls
    pub fn extract_function_calls(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), Self::CALL_QUERY) {
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

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), Self::CLASS_EXTENDS_QUERY) {
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

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), Self::INTERFACE_IMPLEMENTS_QUERY) {
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

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), type_annotation_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index == query.capture_index_for_name("type_name").unwrap_or(u32::MAX) {
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

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), generic_constraint_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index == query.capture_index_for_name("constraint_type").unwrap_or(u32::MAX) {
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

impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            dependencies.extend(self.extract_es6_imports(file_path, tree, &parsed_file.source)?);
            dependencies.extend(self.extract_commonjs_requires(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_function_calls(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_class_inheritance(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_interface_implementations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_type_dependencies(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_generic_constraints(
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