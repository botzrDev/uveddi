use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::path::Path;
use streaming_iterator::StreamingIterator;

/// Interface usage information
#[derive(Debug, Clone)]
pub struct InterfaceUsage {
    pub interface_name: String,
    pub implementing_types: Vec<String>,
    pub usage_locations: Vec<(String, u32)>, // (file_path, line_number)
}

/// Validates interface usage patterns and identifies anti-patterns
#[derive(Debug, Clone)]
pub struct InterfaceValidator;

impl InterfaceValidator {
    pub fn new() -> Self {
        Self
    }

    /// Analyze interface and trait usage patterns
    pub fn analyze_interface_usage(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
        language: SourceLanguage,
    ) -> Result<Vec<InterfaceUsage>, AnalysisError> {
        match language {
            SourceLanguage::Rust => self.analyze_rust_traits(file_path, tree, source),
            SourceLanguage::Python => self.analyze_python_protocols(file_path, tree, source),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                self.analyze_javascript_interfaces(file_path, tree, source)
            }
            _ => Ok(Vec::new()),
        }
    }

    fn analyze_rust_traits(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<InterfaceUsage>, AnalysisError> {
        let mut usages = Vec::new();

        // Trait definitions
        let trait_query = r#"
            (trait_item
              name: (type_identifier) @trait_name) @trait_def
        "#;

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(),
            trait_query,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let trait_name = capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_trait")
                        .to_string();

                    usages.push(InterfaceUsage {
                        interface_name: trait_name,
                        implementing_types: Vec::new(),
                        usage_locations: vec![(
                            file_path.to_string_lossy().to_string(),
                            capture.node.start_position().row as u32 + 1,
                        )],
                    });
                }
            }
        }

        Ok(usages)
    }

    fn analyze_python_protocols(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<InterfaceUsage>, AnalysisError> {
        let mut usages = Vec::new();

        // Python Protocol/ABC classes
        let protocol_query = r#"
            (class_definition
              name: (identifier) @class_name
              superclasses: (argument_list
                (identifier) @parent_class)) @class_def
        "#;

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_python::LANGUAGE.into(),
            protocol_query,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if m.captures.len() >= 2 {
                    let class_name = m.captures[0]
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_class")
                        .to_string();
                    let parent_name = m.captures[1]
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_parent");

                    if parent_name == "Protocol" || parent_name == "ABC" {
                        usages.push(InterfaceUsage {
                            interface_name: class_name,
                            implementing_types: Vec::new(),
                            usage_locations: vec![(
                                file_path.to_string_lossy().to_string(),
                                m.captures[0].node.start_position().row as u32 + 1,
                            )],
                        });
                    }
                }
            }
        }

        Ok(usages)
    }

    fn analyze_javascript_interfaces(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<InterfaceUsage>, AnalysisError> {
        let mut usages = Vec::new();

        // TypeScript interface definitions
        let interface_query = r#"
            (interface_declaration
              name: (type_identifier) @interface_name) @interface_def
        "#;

        if let Ok(query) = Query::new(
            &crate::ast::tree_sitter::tree_sitter_javascript::LANGUAGE.into(),
            interface_query,
        ) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let interface_name = capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_interface")
                        .to_string();

                    usages.push(InterfaceUsage {
                        interface_name,
                        implementing_types: Vec::new(),
                        usage_locations: vec![(
                            file_path.to_string_lossy().to_string(),
                            capture.node.start_position().row as u32 + 1,
                        )],
                    });
                }
            }
        }

        Ok(usages)
    }
}

impl Default for InterfaceValidator {
    fn default() -> Self {
        Self::new()
    }
}
