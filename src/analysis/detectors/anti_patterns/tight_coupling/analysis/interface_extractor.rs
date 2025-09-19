use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::path::Path;
use streaming_iterator::StreamingIterator;

use super::super::types::{Dependency, DependencyStrength};

/// Extracts interface and trait dependencies from code
pub struct InterfaceExtractor;

impl InterfaceExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract trait/interface implementations and their dependencies
    pub fn extract_interface_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        match language {
            SourceLanguage::Rust => self.extract_rust_trait_dependencies(file_path, tree, source),
            SourceLanguage::Python => self.extract_python_protocol_dependencies(file_path, tree, source),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                self.extract_javascript_interface_dependencies(file_path, tree, source)
            }
        }
    }

    /// Detect direct field access which indicates tight coupling
    pub fn detect_field_access_coupling(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut field_access_deps = Vec::new();

        match language {
            SourceLanguage::Rust => {
                // Rust: struct.field access
                if let Ok(query) = Query::new(
                    &crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(),
                    r#"(field_expression value: (identifier) @object field: (field_identifier) @field) @access"#,
                ) {
                    field_access_deps.extend(self.extract_field_access_dependencies(
                        file_path, &query, tree, source,
                    )?);
                }
            }
            SourceLanguage::Python => {
                // Python: object.attribute access
                if let Ok(query) = Query::new(
                    &crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(),
                    r#"(attribute object: (identifier) @object attribute: (identifier) @field) @access"#,
                ) {
                    field_access_deps.extend(self.extract_field_access_dependencies(
                        file_path, &query, tree, source,
                    )?);
                }
            }
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                // JavaScript: object.property access
                if let Ok(query) = Query::new(
                    &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
                    r#"(member_expression object: (identifier) @object property: (property_identifier) @field) @access"#,
                ) {
                    field_access_deps.extend(self.extract_field_access_dependencies(
                        file_path, &query, tree, source,
                    )?);
                }
            }
        }

        Ok(field_access_deps)
    }

    fn extract_rust_trait_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let impl_query = r#"
            (impl_item
              trait: (type_identifier) @trait_name
              type: (type_identifier) @type_name) @impl_block
        "#;

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(), impl_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if let Some(capture) = m.captures.first() {
                    let trait_name = capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_trait");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Class {
                            name: "impl_struct".to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Class {
                            name: trait_name.to_string(),
                            file_path: "trait_definition".to_string(),
                        },
                        dependency_type: LocalDependencyType::Implementation,
                        line_number: Some(capture.node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Medium,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn extract_python_protocol_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let inheritance_query = r#"
            (class_definition
              name: (identifier) @class_name
              superclasses: (argument_list
                (identifier) @parent_class)) @class_def
        "#;

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(), inheritance_query) {
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

    fn extract_javascript_interface_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let implements_query = r#"
            (class_declaration
              name: (type_identifier) @class_name
              heritage: (class_heritage
                (implements_clause
                  (type_identifier) @interface_name))) @class_def
        "#;

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(), implements_query) {
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

    fn extract_field_access_dependencies(
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
            if m.captures.len() >= 2 {
                let object_name = m.captures[0]
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("unknown_object");
                let field_name = m.captures[1]
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("unknown_field");

                dependencies.push(Dependency {
                    from_component: ComponentNode::Function {
                        name: "accessor".to_string(),
                        file_path: file_path.to_string_lossy().to_string(),
                    },
                    to_component: ComponentNode::Class {
                        name: format!("{}::{}", object_name, field_name),
                        file_path: "external".to_string(),
                    },
                    dependency_type: LocalDependencyType::Call,
                    line_number: Some(m.captures[0].node.start_position().row as u32 + 1),
                    strength: DependencyStrength::Strong, // Direct field access is tight coupling
                });
            }
        }

        Ok(dependencies)
    }
}

impl Default for InterfaceExtractor {
    fn default() -> Self {
        Self::new()
    }
}