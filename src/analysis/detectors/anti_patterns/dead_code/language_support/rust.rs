//! Rust-specific dead code analysis

use std::collections::HashSet;

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::ParsedFile;
use streaming_iterator::StreamingIterator;

use crate::analysis::detectors::anti_patterns::dead_code::types::{Symbol, SymbolType};

use super::LanguageAnalyzer;

/// Rust-specific analyzer for dead code detection
pub struct RustAnalyzer;

impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn extract_code_snippet(&self, node: &Node, source: &[u8], context_lines: usize) -> String {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();

        if let Ok(snippet) = std::str::from_utf8(&source[start_byte..end_byte]) {
            let lines: Vec<&str> = snippet.lines().take(context_lines).collect();
            lines.join("\n")
        } else {
            "Unable to extract snippet".to_string()
        }
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        // Extract functions
        let function_query = Query::new(&language, RUST_FUNCTION_QUERY)
            .map_err(|e| AnalysisError::DetectionError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        #[cfg(feature = "tree-sitter")]
        {
            use tree_sitter::StreamingIterator;
            let mut matches = cursor.matches(&function_query, tree.root_node(), source);
            while let Some(mat) = matches.next() {
                if let Some(name_capture) = mat.captures.first() {
                    let name_node = name_capture.node;
                    if let Ok(name) = name_node.utf8_text(source) {
                        let is_exported = self.is_exported(&name_node, source);
                        let code_snippet = self.extract_code_snippet(&name_node, source, 3);

                        symbols.push(Symbol {
                            name: name.to_string(),
                            symbol_type: SymbolType::Function,
                            path: (*parsed_file.file_path).clone(),
                            line_number: (name_node.start_position().row + 1) as u32,
                            is_exported,
                            is_live: false,
                            confidence: if is_exported { 0.6 } else { 0.9 },
                            code_snippet,
                        });
                    }
                }
            }
        }

        // Extract structs, enums, traits, etc.
        self.extract_rust_specific_symbols(parsed_file, &mut symbols)?;

        Ok(symbols)
    }

    fn extract_references(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError> {
        let mut references = HashSet::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        let call_query = Query::new(&language, RUST_CALL_QUERY)
            .map_err(|e| AnalysisError::DetectionError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        #[cfg(feature = "tree-sitter")]
        {
            use tree_sitter::StreamingIterator;
            let mut matches = cursor.matches(&call_query, tree.root_node(), source);
            while let Some(mat) = matches.next() {
                if let Some(name_capture) = mat.captures.first() {
                    let name_node = name_capture.node;
                    if let Ok(name) = name_node.utf8_text(source) {
                        references.insert(name.to_string());
                    }
                }
            }
        }

        Ok(references)
    }

    fn is_exported(&self, node: &Node, source: &[u8]) -> bool {
        let mut current = node.parent();
        while let Some(parent) = current {
            let kind = parent.kind();

            if kind == "function_item"
                || kind == "struct_item"
                || kind == "enum_item"
                || kind == "const_item"
            {
                for i in 0..parent.child_count() {
                    if let Some(child) = parent.child(i) {
                        if child.kind() == "visibility_modifier" {
                            if let Ok(text) = child.utf8_text(source) {
                                return text.contains("pub");
                            }
                        }
                    }
                }
                return false;
            }
            current = parent.parent();
        }
        false
    }

    fn calculate_confidence(&self, symbol: &Symbol) -> f64 {
        if symbol.name.starts_with("test_") || symbol.name.starts_with("bench_") {
            return 0.3;
        }

        if symbol.is_exported {
            0.6
        } else {
            0.9
        }
    }

    fn identify_entry_points(&self, symbols: &[Symbol]) -> Vec<String> {
        let mut entry_points = Vec::new();

        for symbol in symbols {
            if symbol.name == "main" || symbol.name.starts_with("test_") {
                entry_points.push(symbol.name.clone());
            }
        }

        entry_points
    }
}

impl RustAnalyzer {
    /// Extract Rust-specific symbols like traits, macros, lifetimes
    fn extract_rust_specific_symbols(
        &self,
        parsed_file: &ParsedFile,
        symbols: &mut Vec<Symbol>,
    ) -> Result<(), AnalysisError> {
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        // Extract structs
        let struct_query = Query::new(&language, RUST_STRUCT_QUERY)
            .map_err(|e| AnalysisError::DetectionError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        #[cfg(feature = "tree-sitter")]
        {
            use tree_sitter::StreamingIterator;
            let mut matches = cursor.matches(&struct_query, tree.root_node(), source);
            while let Some(mat) = matches.next() {
                if let Some(name_capture) = mat.captures.first() {
                    let name_node = name_capture.node;
                    if let Ok(name) = name_node.utf8_text(source) {
                        let is_exported = self.is_exported(&name_node, source);
                        let code_snippet = self.extract_code_snippet(&name_node, source, 3);

                        symbols.push(Symbol {
                            name: name.to_string(),
                            symbol_type: SymbolType::Struct,
                            path: (*parsed_file.file_path).clone(),
                            line_number: (name_node.start_position().row + 1) as u32,
                            is_exported,
                            is_live: false,
                            confidence: if is_exported { 0.6 } else { 0.9 },
                            code_snippet,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

// Tree-sitter queries for Rust
const RUST_FUNCTION_QUERY: &str = r#"
(function_item
  name: (identifier) @name
)
"#;

const RUST_STRUCT_QUERY: &str = r#"
(struct_item
  name: (type_identifier) @name
)
"#;

const RUST_CALL_QUERY: &str = r#"
(call_expression
  function: (identifier) @name
)
(call_expression
  function: (field_expression
    field: (field_identifier) @name)
)
(call_expression
  function: (scoped_identifier
    name: (identifier) @name)
)
(macro_invocation
  macro: (identifier) @name
)
"#;
