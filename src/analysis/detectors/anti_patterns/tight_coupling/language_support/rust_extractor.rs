use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::Tree;
use std::path::Path;

use super::super::types::{Dependency, DependencyStrength};
use super::rust_queries::RustQueries;

/// Rust-specific dependency extraction methods
pub struct RustExtractor;

impl RustExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract use declarations (imports)
    pub fn extract_use_declarations(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();
        let query = RustQueries::create_query(RustQueries::USE_QUERY)?;
        let matches = RustQueries::execute_query(&query, tree, source)?;

        for m in matches {
            for capture in m.captures {
                let node_text = RustQueries::get_node_text(&capture.node, source)?;

                let from_component = ComponentNode::Module {
                    path: file_path.to_string_lossy().to_string(),
                };

                let to_component = ComponentNode::Module { path: node_text };

                dependencies.push(Dependency {
                    from_component,
                    to_component,
                    dependency_type: LocalDependencyType::Import,
                    line_number: Some(RustQueries::get_line_number(&capture.node)),
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
        let query = RustQueries::create_query(RustQueries::CALL_QUERY)?;
        let matches = RustQueries::execute_query(&query, tree, source)?;

        for m in matches {
            if let Some(capture) = m.captures.first() {
                let node_text = RustQueries::get_node_text(&capture.node, source)?;

                let from_component = ComponentNode::Function {
                    name: "caller".to_string(), // Would need more context to get actual function name
                    file_path: file_path.to_string_lossy().to_string(),
                };

                let to_component = ComponentNode::Function {
                    name: node_text,
                    file_path: "external".to_string(),
                };

                dependencies.push(Dependency {
                    from_component,
                    to_component,
                    dependency_type: LocalDependencyType::Call,
                    line_number: Some(RustQueries::get_line_number(&capture.node)),
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
        let query = RustQueries::create_query(RustQueries::STRUCT_QUERY)?;
        let matches = RustQueries::execute_query(&query, tree, source)?;

        for m in matches {
            if let Some(capture) = m.captures.first() {
                let node_text = RustQueries::get_node_text(&capture.node, source)?;

                let from_component = ComponentNode::Function {
                    name: "instantiator".to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                };

                let to_component = ComponentNode::Class {
                    name: node_text,
                    file_path: "external".to_string(),
                };

                dependencies.push(Dependency {
                    from_component,
                    to_component,
                    dependency_type: LocalDependencyType::Call,
                    line_number: Some(RustQueries::get_line_number(&capture.node)),
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

        if let Ok(query) = RustQueries::create_query(RustQueries::TRAIT_IMPL_QUERY) {
            let matches = RustQueries::execute_query(&query, tree, source)?;

            for m in matches {
                if let Some(capture) = m.captures.first() {
                    let trait_name = RustQueries::get_node_text(&capture.node, source)
                        .unwrap_or_else(|_| "unknown_trait".to_string());

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Class {
                            name: "impl_struct".to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Class {
                            name: trait_name,
                            file_path: "trait_definition".to_string(),
                        },
                        dependency_type: LocalDependencyType::Implementation,
                        line_number: Some(RustQueries::get_line_number(&capture.node)),
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

        if let Ok(query) = RustQueries::create_query(RustQueries::MOD_QUERY) {
            let matches = RustQueries::execute_query(&query, tree, source)?;

            for m in matches {
                for capture in m.captures {
                    if capture.index
                        == query
                            .capture_index_for_name("module_name")
                            .unwrap_or(u32::MAX)
                    {
                        let module_name = RustQueries::get_node_text(&capture.node, source)
                            .unwrap_or_else(|_| "unknown_module".to_string());

                        dependencies.push(Dependency {
                            from_component: ComponentNode::Module {
                                path: file_path.to_string_lossy().to_string(),
                            },
                            to_component: ComponentNode::Module { path: module_name },
                            dependency_type: LocalDependencyType::Import,
                            line_number: Some(RustQueries::get_line_number(&capture.node)),
                            strength: DependencyStrength::Medium,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }
}

impl Default for RustExtractor {
    fn default() -> Self {
        Self::new()
    }
}
