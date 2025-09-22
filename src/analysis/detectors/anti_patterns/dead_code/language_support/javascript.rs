//! JavaScript/TypeScript-specific dead code analysis

use std::collections::HashSet;

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use streaming_iterator::StreamingIterator;

use crate::analysis::detectors::anti_patterns::dead_code::types::{Symbol, SymbolType};

use super::LanguageAnalyzer;

/// JavaScript/TypeScript-specific analyzer for dead code detection
pub struct JavaScriptAnalyzer;

impl JavaScriptAnalyzer {
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

    fn is_react_component(&self, name: &str) -> bool {
        name.chars().next().map_or(false, |c| c.is_uppercase())
    }
}

impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        // Extract functions
        let function_query = Query::new(&language, JAVASCRIPT_FUNCTION_QUERY)
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
                        let confidence =
                            self.calculate_javascript_confidence(name, parsed_file.path());

                        let symbol_type = if self.is_async_function(&name_node, source) {
                            SymbolType::AsyncFunction
                        } else {
                            SymbolType::Function
                        };

                        symbols.push(Symbol {
                            name: name.to_string(),
                            symbol_type,
                            path: (*parsed_file.file_path).clone(),
                            line_number: (name_node.start_position().row + 1) as u32,
                            is_exported,
                            is_live: false,
                            confidence,
                            code_snippet,
                        });
                    }
                }
            }
        }

        // Extract JavaScript-specific symbols
        self.extract_javascript_specific_symbols(parsed_file, &mut symbols)?;

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

        let call_query = Query::new(&language, JAVASCRIPT_CALL_QUERY)
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
            if let Ok(text) = parent.utf8_text(source) {
                if text.contains("export") || text.contains("module.exports") {
                    return true;
                }
            }
            current = parent.parent();
        }
        false
    }

    fn calculate_confidence(&self, symbol: &Symbol) -> f64 {
        self.calculate_javascript_confidence(&symbol.name, &symbol.path)
    }

    fn identify_entry_points(&self, symbols: &[Symbol]) -> Vec<String> {
        let mut entry_points = Vec::new();

        for symbol in symbols {
            if symbol.name == "main"
                || symbol.name == "index"
                || symbol.name == "App"
                || symbol.name.starts_with("test")
                || symbol.name.starts_with("describe")
                || symbol.name.starts_with("it")
            {
                entry_points.push(symbol.name.clone());
            }
        }

        entry_points
    }
}

impl JavaScriptAnalyzer {
    fn calculate_javascript_confidence(&self, name: &str, file_path: &std::path::Path) -> f64 {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.contains("test") || file_name.contains("spec") {
            return 0.3;
        }

        // Lower confidence for React components
        if self.is_react_component(name) {
            return 0.6;
        }

        0.8
    }

    fn is_async_function(&self, node: &Node, source: &[u8]) -> bool {
        if let Some(parent) = node.parent() {
            if let Ok(text) = parent.utf8_text(source) {
                return text.starts_with("async ");
            }
        }
        false
    }

    /// Extract JavaScript-specific symbols like closures, arrow functions, modules
    fn extract_javascript_specific_symbols(
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

        // Extract arrow functions
        let arrow_query = Query::new(&language, JAVASCRIPT_ARROW_QUERY)
            .map_err(|e| AnalysisError::DetectionError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        #[cfg(feature = "tree-sitter")]
        {
            use tree_sitter::StreamingIterator;
            let mut matches = cursor.matches(&arrow_query, tree.root_node(), source);
            while let Some(mat) = matches.next() {
                if let Some(name_capture) = mat.captures.first() {
                    let name_node = name_capture.node;
                    if let Ok(name) = name_node.utf8_text(source) {
                        let is_exported = self.is_exported(&name_node, source);
                        let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                        let confidence =
                            self.calculate_javascript_confidence(name, parsed_file.path());

                        symbols.push(Symbol {
                            name: name.to_string(),
                            symbol_type: SymbolType::Closure,
                            path: (*parsed_file.file_path).clone(),
                            line_number: (name_node.start_position().row + 1) as u32,
                            is_exported,
                            is_live: false,
                            confidence,
                            code_snippet,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

// Tree-sitter queries for JavaScript/TypeScript
const JAVASCRIPT_FUNCTION_QUERY: &str = r#"
(function_declaration
  name: (identifier) @name
)
(function_expression
  name: (identifier) @name
)
(method_definition
  name: (property_identifier) @name
)
"#;

const JAVASCRIPT_ARROW_QUERY: &str = r#"
(variable_declarator
  name: (identifier) @name
  value: (arrow_function)
)
"#;

const JAVASCRIPT_CALL_QUERY: &str = r#"
(call_expression
  function: (identifier) @name
)
(call_expression
  function: (member_expression
    property: (property_identifier) @name)
)
(call_expression
  function: (member_expression
    object: (identifier)
    property: (property_identifier) @name)
)
"#;
