//! Rust interface analysis for leaky abstraction detection.

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, InterfaceAnalysisResult, ApiElement, VisibilityViolation
};

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes Rust public interfaces for potential abstraction leaks.
pub struct RustInterfaceAnalyzer;

impl RustInterfaceAnalyzer {
    /// Creates a new Rust interface analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes Rust interface for potential leaks.
    pub fn analyze_rust_interface(
        &self,
        parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
        let mut public_api = Vec::new();
        let mut visibility_violations = Vec::new();
        let contract_violations = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::DetectionError("No AST available".to_string())
        })?;
        let language = tree.language();

        let query_source = r#"
            ; Detect public struct fields (potential encapsulation violation)
            (struct_item
              (visibility_modifier) @pub_vis
              name: (type_identifier) @struct_name
              body: (field_declaration_list
                (field_declaration
                  (visibility_modifier) @field_vis
                  name: (field_identifier) @field_name
                  type: (_) @field_type))) @struct_decl

            ; Detect public functions
            (function_item
              (visibility_modifier) @fn_vis
              name: (identifier) @fn_name
              return_type: (_)? @return_type) @function_decl

            ; Detect public enums
            (enum_item
              (visibility_modifier)? @enum_vis
              name: (type_identifier) @enum_name) @enum_decl

            ; Detect public traits
            (trait_item
              (visibility_modifier)? @trait_vis
              name: (type_identifier) @trait_name) @trait_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Rust interface query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "field_vis" => {
                        if let Ok(vis_text) = node.utf8_text(source_bytes) {
                            if vis_text == "pub" {
                                if let Some(field_name_node) = self.find_sibling_capture(
                                    &match_.captures, &query, "field_name"
                                ) {
                                    if let Ok(field_name) = field_name_node.utf8_text(source_bytes) {
                                        public_api.push(ApiElement {
                                            name: field_name.to_string(),
                                            element_type: "field".to_string(),
                                            exposes_internals: true,
                                            line_number: node.start_position().row as u32 + 1,
                                        });

                                        visibility_violations.push(VisibilityViolation {
                                            description: format!(
                                                "Public field '{}' exposes internal structure",
                                                field_name
                                            ),
                                            accessed_element: field_name.to_string(),
                                            line_number: node.start_position().row as u32 + 1,
                                            severity: "medium".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    "fn_vis" => {
                        if let Ok(vis_text) = node.utf8_text(source_bytes) {
                            if vis_text == "pub" {
                                if let Some(fn_name_node) = self.find_sibling_capture(
                                    &match_.captures, &query, "fn_name"
                                ) {
                                    if let Ok(fn_name) = fn_name_node.utf8_text(source_bytes) {
                                        public_api.push(ApiElement {
                                            name: fn_name.to_string(),
                                            element_type: "function".to_string(),
                                            exposes_internals: false,
                                            line_number: node.start_position().row as u32 + 1,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    "struct_name" => {
                        if let Some(vis_node) = self.find_sibling_capture(
                            &match_.captures, &query, "pub_vis"
                        ) {
                            if let Ok(vis_text) = vis_node.utf8_text(source_bytes) {
                                if vis_text == "pub" {
                                    if let Ok(struct_name) = node.utf8_text(source_bytes) {
                                        public_api.push(ApiElement {
                                            name: struct_name.to_string(),
                                            element_type: "struct".to_string(),
                                            exposes_internals: false,
                                            line_number: node.start_position().row as u32 + 1,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(InterfaceAnalysisResult {
            public_api,
            visibility_violations,
            contract_violations,
        })
    }

    /// Helper function to find a sibling capture by name.
    fn find_sibling_capture<'a>(
        &self,
        captures: &'a [crate::ast::tree_sitter::QueryCapture],
        query: &Query,
        capture_name: &str,
    ) -> Option<Node<'a>> {
        captures.iter().find(|capture| {
            query.capture_names()[capture.index as usize] == capture_name
        }).map(|capture| capture.node)
    }
}

impl Default for RustInterfaceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}