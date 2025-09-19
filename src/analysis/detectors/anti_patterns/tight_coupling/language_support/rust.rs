use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use std::path::Path;
use streaming_iterator::StreamingIterator;

use super::super::types::{Dependency, DependencyStrength};
use super::LanguageAnalyzer;

/// Rust language analyzer for dependency extraction
#[derive(Debug, Default, Clone)]
pub struct RustAnalyzer;

impl RustAnalyzer {
    const USE_QUERY: &'static str = r#"
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

    const CALL_QUERY: &'static str = r#"
        (call_expression
          function: (scoped_identifier
            path: (identifier) @module
            name: (identifier) @function)) @call

        (call_expression
          function: (field_expression
            value: (identifier) @object
            field: (field_identifier) @method)) @method_call
    "#;

    const STRUCT_QUERY: &'static str = r#"
        (struct_expression
          name: (scoped_type_identifier
            path: (identifier) @module
            name: (type_identifier) @struct)) @instantiation
    "#;

    const TRAIT_IMPL_QUERY: &'static str = r#"
        (impl_item
          trait: (type_identifier) @trait_name
          type: (type_identifier) @type_name) @impl_block
    "#;

    /// Extract use declarations (imports)
    pub fn extract_use_declarations(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();
        let query = Query::new(&crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(), Self::USE_QUERY)
            .map_err(|e| AnalysisError::QueryError(format!("Failed to create use query: {}", e)))?;

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
        let query =
            Query::new(&crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(), Self::CALL_QUERY).map_err(|e| {
                AnalysisError::QueryError(format!("Failed to create call query: {}", e))
            })?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        while let Some(m) = matches.next() {
            if let Some(capture) = m.captures.first() {
                let node_text = capture.node.utf8_text(source.as_bytes()).map_err(|e| {
                    AnalysisError::QueryError(format!("Failed to get node text: {}", e))
                })?;

                let from_component = ComponentNode::Function {
                    name: "caller".to_string(), // Would need more context to get actual function name
                    file_path: file_path.to_string_lossy().to_string(),
                };

                let to_component = ComponentNode::Function {
                    name: node_text.to_string(),
                    file_path: "external".to_string(),
                };

                dependencies.push(Dependency {
                    from_component,
                    to_component,
                    dependency_type: LocalDependencyType::Call,
                    line_number: Some(capture.node.start_position().row as u32 + 1),
                    strength: DependencyStrength::Strong,
                });
            }
        }

        Ok(dependencies)
    }

    /// Extract struct instantiations
    pub fn extract_struct_instantiations(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();
        let query =
            Query::new(&crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(), Self::STRUCT_QUERY).map_err(|e| {
                AnalysisError::QueryError(format!("Failed to create struct query: {}", e))
            })?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        while let Some(m) = matches.next() {
            if let Some(capture) = m.captures.first() {
                let node_text = capture.node.utf8_text(source.as_bytes()).map_err(|e| {
                    AnalysisError::QueryError(format!("Failed to get node text: {}", e))
                })?;

                let from_component = ComponentNode::Function {
                    name: "instantiator".to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                };

                let to_component = ComponentNode::Class {
                    name: node_text.to_string(),
                    file_path: "external".to_string(),
                };

                dependencies.push(Dependency {
                    from_component,
                    to_component,
                    dependency_type: LocalDependencyType::Call,
                    line_number: Some(capture.node.start_position().row as u32 + 1),
                    strength: DependencyStrength::Strong,
                });
            }
        }

        Ok(dependencies)
    }

    /// Extract trait implementations
    pub fn extract_trait_implementations(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(), Self::TRAIT_IMPL_QUERY) {
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
                        strength: DependencyStrength::Strong,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Extract module dependencies (mod declarations)
    pub fn extract_module_dependencies(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let mod_query = r#"
            (mod_item
              name: (identifier) @module_name) @mod_decl
        "#;

        if let Ok(query) = Query::new(&crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(), mod_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index == query.capture_index_for_name("module_name").unwrap_or(u32::MAX) {
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
        }

        Ok(dependencies)
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            dependencies.extend(self.extract_use_declarations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_function_calls(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_struct_instantiations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_trait_implementations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_module_dependencies(
                file_path,
                tree,
                &parsed_file.source,
            )?);
        }

        Ok(dependencies)
    }

    fn get_language(&self) -> SourceLanguage {
        SourceLanguage::Rust
    }
}