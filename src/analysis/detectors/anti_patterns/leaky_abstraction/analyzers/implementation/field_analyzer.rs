//! Field and property analysis for implementation leaks.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ImplementationExposure, LeakType, TypeLeakage,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes field and property implementations for leaks.
#[derive(Clone)]
pub struct FieldAnalyzer;

impl FieldAnalyzer {
    /// Creates a new field analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes Rust field implementations.
    pub fn analyze_rust_fields(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<(Vec<ImplementationExposure>, Vec<TypeLeakage>), AnalysisError> {
        let mut implementation_exposures = Vec::new();
        let mut type_leakages = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect public struct fields with concrete infrastructure types
            (struct_item
              (visibility_modifier) @struct_vis
              name: (type_identifier) @struct_name
              body: (field_declaration_list
                (field_declaration
                  (visibility_modifier) @field_vis
                  name: (field_identifier) @field_name
                  type: (_) @field_type))) @struct_decl

            ; Detect type aliases that expose implementation details
            (type_item
              (visibility_modifier) @type_vis
              name: (type_identifier) @type_name
              type: (_) @type_def) @type_alias
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Rust field query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "field_type" => {
                        if let Ok(field_type_text) = node.utf8_text(source_bytes) {
                            if self.is_concrete_implementation_type(field_type_text) {
                                if let Some(field_vis_node) =
                                    self.find_sibling_capture(&match_.captures, &query, "field_vis")
                                {
                                    if let Ok(vis_text) = field_vis_node.utf8_text(source_bytes) {
                                        if vis_text == "pub" {
                                            implementation_exposures.push(ImplementationExposure {
                                                description: format!(
                                                    "Concrete implementation type '{}' exposed as public field",
                                                    field_type_text
                                                ),
                                                exposed_detail: field_type_text.to_string(),
                                                line_number: node.start_position().row as u32 + 1,
                                                severity: "medium".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "type_def" => {
                        if let Ok(type_def_text) = node.utf8_text(source_bytes) {
                            if self.is_infrastructure_type(type_def_text) {
                                if let Some(type_vis_node) =
                                    self.find_sibling_capture(&match_.captures, &query, "type_vis")
                                {
                                    if let Ok(vis_text) = type_vis_node.utf8_text(source_bytes) {
                                        if vis_text == "pub" {
                                            type_leakages.push(TypeLeakage {
                                                description: format!(
                                                    "Infrastructure type '{}' exposed through public type alias",
                                                    type_def_text
                                                ),
                                                leaked_type: type_def_text.to_string(),
                                                line_number: node.start_position().row as u32 + 1,
                                                severity: "high".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok((implementation_exposures, type_leakages))
    }

    /// Analyzes Python field implementations.
    pub fn analyze_python_fields(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<(Vec<ImplementationExposure>, Vec<TypeLeakage>), AnalysisError> {
        let mut implementation_exposures = Vec::new();
        let mut type_leakages = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect direct attribute assignments with infrastructure objects
            (assignment
              left: (attribute
                object: (identifier) @obj_name
                attribute: (identifier) @attr_name)
              right: (_) @assignment_value) @assignment_stmt

            ; Detect class attribute definitions
            (class_definition
              body: (block
                (expression_statement
                  (assignment
                    left: (identifier) @class_attr
                    right: (_) @attr_value))*)) @class_def
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Python field query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                if capture_name == "attr_name" {
                    if let Ok(attr_name) = node.utf8_text(source_bytes) {
                        if attr_name.starts_with('_') && !attr_name.starts_with("__") {
                            implementation_exposures.push(ImplementationExposure {
                                description: format!(
                                    "Private attribute '{}' may be exposed",
                                    attr_name
                                ),
                                exposed_detail: attr_name.to_string(),
                                line_number: node.start_position().row as u32 + 1,
                                severity: "low".to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok((implementation_exposures, type_leakages))
    }

    /// Checks if a type represents a concrete implementation type.
    fn is_concrete_implementation_type(&self, type_text: &str) -> bool {
        let concrete_patterns = [
            "Connection",
            "Pool",
            "Transaction",
            "Session",
            "Client",
            "Builder",
            "Config",
            "Context",
        ];

        concrete_patterns
            .iter()
            .any(|pattern| type_text.contains(pattern))
    }

    /// Checks if a type represents an infrastructure type.
    fn is_infrastructure_type(&self, type_text: &str) -> bool {
        self.is_concrete_implementation_type(type_text)
            || type_text.contains("Error")
                && (type_text.contains("Diesel")
                    || type_text.contains("Sqlx")
                    || type_text.contains("tokio"))
    }

    /// Helper function to find a sibling capture by name.
    fn find_sibling_capture<'a>(
        &self,
        captures: &'a [crate::ast::tree_sitter::QueryCapture],
        query: &Query,
        capture_name: &str,
    ) -> Option<Node<'a>> {
        captures
            .iter()
            .find(|capture| query.capture_names()[capture.index as usize] == capture_name)
            .map(|capture| capture.node)
    }
}

impl Default for FieldAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
