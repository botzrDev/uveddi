//! Python interface analysis for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ApiElement, InterfaceAnalysisResult, VisibilityViolation,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes Python public interfaces for potential abstraction leaks.
pub struct PythonInterfaceAnalyzer;

impl PythonInterfaceAnalyzer {
    /// Creates a new Python interface analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes Python interface for potential leaks.
    pub fn analyze_python_interface(
        &self,
        parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
        let mut public_api = Vec::new();
        let mut visibility_violations = Vec::new();
        let contract_violations = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect class definitions
            (class_definition
              name: (identifier) @class_name) @class_def

            ; Detect function definitions
            (function_definition
              name: (identifier) @func_name) @func_def

            ; Detect attribute access with underscores (private access)
            (attribute
              object: (identifier) @obj_name
              attribute: (identifier) @attr_name) @attr_access
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Python interface query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "class_name" => {
                        if let Ok(class_name) = node.utf8_text(source_bytes) {
                            if !class_name.starts_with('_') {
                                public_api.push(ApiElement {
                                    name: class_name.to_string(),
                                    element_type: "class".to_string(),
                                    exposes_internals: false,
                                    line_number: node.start_position().row as u32 + 1,
                                });
                            }
                        }
                    }
                    "func_name" => {
                        if let Ok(func_name) = node.utf8_text(source_bytes) {
                            if !func_name.starts_with('_') {
                                public_api.push(ApiElement {
                                    name: func_name.to_string(),
                                    element_type: "function".to_string(),
                                    exposes_internals: false,
                                    line_number: node.start_position().row as u32 + 1,
                                });
                            }
                        }
                    }
                    "attr_name" => {
                        if let Ok(attr_name) = node.utf8_text(source_bytes) {
                            if attr_name.starts_with('_') && !attr_name.starts_with("__") {
                                visibility_violations.push(VisibilityViolation {
                                    description: format!(
                                        "Access to private attribute '{}'",
                                        attr_name
                                    ),
                                    accessed_element: attr_name.to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "medium".to_string(),
                                });
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
}

impl Default for PythonInterfaceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
