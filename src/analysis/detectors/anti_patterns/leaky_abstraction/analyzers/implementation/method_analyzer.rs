//! Method implementation analysis for leaks.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ImplementationVisibilityIssue, LeakType, TypeLeakage,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes method implementations for leaks.
#[derive(Clone)]
pub struct MethodAnalyzer;

impl MethodAnalyzer {
    /// Creates a new method analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes Rust method implementations.
    pub fn analyze_rust_methods(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<(Vec<TypeLeakage>, Vec<ImplementationVisibilityIssue>), AnalysisError> {
        let mut type_leakages = Vec::new();
        let mut visibility_issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect public functions with infrastructure error types
            (function_item
              (visibility_modifier) @fn_vis
              name: (identifier) @fn_name
              return_type: (_) @return_type) @function_decl

            ; Detect impl blocks that expose internal methods
            (impl_item
              type: (type_identifier) @impl_type
              body: (declaration_list
                (function_item
                  (visibility_modifier) @method_vis
                  name: (identifier) @method_name))) @impl_block
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Rust method query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "return_type" => {
                        if let Ok(return_type_text) = node.utf8_text(source_bytes) {
                            if self.is_infrastructure_error_type(return_type_text) {
                                if let Some(fn_vis_node) =
                                    self.find_sibling_capture(&match_.captures, &query, "fn_vis")
                                {
                                    if let Ok(vis_text) = fn_vis_node.utf8_text(source_bytes) {
                                        if vis_text == "pub" {
                                            type_leakages.push(TypeLeakage {
                                                description: format!(
                                                    "Infrastructure error type '{}' exposed in public API",
                                                    return_type_text
                                                ),
                                                leaked_type: return_type_text.to_string(),
                                                line_number: node.start_position().row as u32 + 1,
                                                severity: "high".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "method_vis" => {
                        if let Ok(vis_text) = node.utf8_text(source_bytes) {
                            if vis_text == "pub" {
                                if let Some(method_name_node) = self.find_sibling_capture(
                                    &match_.captures,
                                    &query,
                                    "method_name",
                                ) {
                                    if let Ok(method_name) =
                                        method_name_node.utf8_text(source_bytes)
                                    {
                                        if method_name.contains("internal")
                                            || method_name.contains("impl")
                                        {
                                            visibility_issues.push(ImplementationVisibilityIssue {
                                                description: format!(
                                                    "Internal method '{}' exposed as public",
                                                    method_name
                                                ),
                                                visibility_problem: format!(
                                                    "Method name suggests internal use but has public visibility"
                                                ),
                                                line_number: node.start_position().row as u32 + 1,
                                                severity: "medium".to_string(),
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

        Ok((type_leakages, visibility_issues))
    }

    /// Analyzes Python method implementations.
    pub fn analyze_python_methods(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationVisibilityIssue>, AnalysisError> {
        let mut visibility_issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect function calls to private methods
            (call
              function: (attribute
                object: (identifier) @obj_name
                attribute: (identifier) @method_name)) @method_call

            ; Detect function definitions
            (function_definition
              name: (identifier) @func_name
              parameters: (parameters
                (identifier) @param_name*)) @func_def
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Python method query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                if capture_name == "method_name" {
                    if let Ok(method_name) = node.utf8_text(source_bytes) {
                        if self.is_private_method(method_name) {
                            visibility_issues.push(ImplementationVisibilityIssue {
                                description: format!("Call to private method '{}'", method_name),
                                visibility_problem: "Private method access".to_string(),
                                line_number: node.start_position().row as u32 + 1,
                                severity: "medium".to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(visibility_issues)
    }

    /// Analyzes JavaScript/TypeScript method implementations.
    pub fn analyze_js_methods(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationVisibilityIssue>, AnalysisError> {
        let mut visibility_issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect class declarations with potential implementation exposure
            (class_declaration
              name: (identifier) @class_name
              body: (class_body
                (method_definition
                  name: (property_identifier) @method_name)*)) @class_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create JS method query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                if capture_name == "method_name" {
                    if let Ok(method_name) = node.utf8_text(source_bytes) {
                        if method_name.starts_with('_') || method_name.contains("internal") {
                            visibility_issues.push(ImplementationVisibilityIssue {
                                description: format!(
                                    "Internal method '{}' may be exposed",
                                    method_name
                                ),
                                visibility_problem: "Method name suggests internal use".to_string(),
                                line_number: node.start_position().row as u32 + 1,
                                severity: "medium".to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(visibility_issues)
    }

    /// Checks if a type name represents an infrastructure error type.
    fn is_infrastructure_error_type(&self, type_text: &str) -> bool {
        let infrastructure_error_patterns = [
            "DieselError",
            "SqlxError",
            "SeaOrmError",
            "tokio::Error",
            "std::io::Error",
            "reqwest::Error",
            "serde_json::Error",
            "toml::de::Error",
            "rusqlite::Error",
            "postgres::Error",
        ];

        infrastructure_error_patterns.iter().any(|pattern| {
            type_text.contains(pattern)
                || (type_text.contains("Result<") && type_text.contains(pattern))
        })
    }

    /// Checks if a method name indicates a private method.
    fn is_private_method(&self, method_name: &str) -> bool {
        method_name.starts_with('_') && !method_name.starts_with("__")
    }

    /// Helper function to find a sibling capture by name.
    fn find_sibling_capture<'a>(
        &self,
        captures: &'a [tree_sitter::QueryCapture],
        query: &Query,
        capture_name: &str,
    ) -> Option<Node<'a>> {
        captures
            .iter()
            .find(|capture| query.capture_names()[capture.index as usize] == capture_name)
            .map(|capture| capture.node)
    }
}

impl Default for MethodAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
