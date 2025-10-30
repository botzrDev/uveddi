//! JavaScript/TypeScript interface analysis for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ApiElement, InterfaceAnalysisResult,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::ParsedFile;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes JavaScript/TypeScript public interfaces for potential abstraction leaks.
#[derive(Clone)]
pub struct JsInterfaceAnalyzer;

impl JsInterfaceAnalyzer {
    /// Creates a new JavaScript/TypeScript interface analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes JavaScript/TypeScript interface for potential leaks.
    pub fn analyze_js_interface(
        &self,
        parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<InterfaceAnalysisResult, AnalysisError> {
        let mut public_api = Vec::new();
        let visibility_violations = Vec::new();
        let contract_violations = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect function declarations
            (function_declaration
              name: (identifier) @func_name) @func_decl

            ; Detect class declarations
            (class_declaration
              name: (identifier) @class_name) @class_decl

            ; Detect exported elements
            (export_statement
              declaration: (_) @exported_decl) @export_stmt
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create JS interface query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "func_name" => {
                        if let Ok(func_name) = node.utf8_text(source_bytes) {
                            public_api.push(ApiElement {
                                name: func_name.to_string(),
                                element_type: "function".to_string(),
                                exposes_internals: false,
                                line_number: node.start_position().row as u32 + 1,
                            });
                        }
                    }
                    "class_name" => {
                        if let Ok(class_name) = node.utf8_text(source_bytes) {
                            public_api.push(ApiElement {
                                name: class_name.to_string(),
                                element_type: "class".to_string(),
                                exposes_internals: false,
                                line_number: node.start_position().row as u32 + 1,
                            });
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

impl Default for JsInterfaceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
