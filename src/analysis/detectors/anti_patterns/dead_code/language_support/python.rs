//! Python-specific dead code analysis

use std::collections::HashSet;

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use streaming_iterator::StreamingIterator;
use crate::ast::tree_sitter_impl::ParsedFile;

use crate::analysis::detectors::anti_patterns::dead_code::types::{Symbol, SymbolType};

use super::LanguageAnalyzer;

/// Python-specific analyzer for dead code detection
pub struct PythonAnalyzer;

impl PythonAnalyzer {
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

impl LanguageAnalyzer for PythonAnalyzer {
    fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        // Extract functions
        let function_query = Query::new(&language, PYTHON_FUNCTION_QUERY)
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
                        let is_exported = !name.starts_with('_');
                        let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                        let confidence = self.calculate_python_confidence(name, parsed_file);

                        symbols.push(Symbol {
                            name: name.to_string(),
                            symbol_type: SymbolType::Function,
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

        // Extract classes
        let class_query = Query::new(&language, PYTHON_CLASS_QUERY)
            .map_err(|e| AnalysisError::DetectionError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        #[cfg(feature = "tree-sitter")]
        {
            use tree_sitter::StreamingIterator;
            let mut matches = cursor.matches(&class_query, tree.root_node(), source);
            while let Some(mat) = matches.next() {
                if let Some(name_capture) = mat.captures.first() {
                    let name_node = name_capture.node;
                    if let Ok(name) = name_node.utf8_text(source) {
                        let is_exported = !name.starts_with('_');
                        let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                        let confidence = self.calculate_python_confidence(name, parsed_file);

                        symbols.push(Symbol {
                            name: name.to_string(),
                            symbol_type: SymbolType::Class,
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

        // Extract decorators and other Python-specific symbols
        self.extract_python_specific_symbols(parsed_file, &mut symbols)?;

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

        let call_query = Query::new(&language, PYTHON_CALL_QUERY)
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
        if let Ok(name) = node.utf8_text(source) {
            !name.starts_with('_')
        } else {
            false
        }
    }

    fn calculate_confidence(&self, symbol: &Symbol) -> f64 {
        self.calculate_python_confidence(&symbol.name, &symbol.path)
    }

    fn identify_entry_points(&self, symbols: &[Symbol]) -> Vec<String> {
        let mut entry_points = Vec::new();

        for symbol in symbols {
            if symbol.name == "main"
                || symbol.name == "__init__"
                || symbol.name == "__main__"
                || symbol.name.starts_with("test_")
            {
                entry_points.push(symbol.name.clone());
            }
        }

        entry_points
    }
}

impl PythonAnalyzer {
    fn calculate_python_confidence(&self, name: &str, file_path: &std::path::Path) -> f64 {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.contains("test") || file_name.contains("spec") {
            return 0.3;
        }

        if name.starts_with("__") && name.ends_with("__") {
            return 0.4;
        }

        if name.starts_with('_') {
            return 0.7;
        }

        0.9
    }

    /// Extract Python-specific symbols like decorators, comprehensions
    fn extract_python_specific_symbols(
        &self,
        _parsed_file: &ParsedFile,
        _symbols: &mut Vec<Symbol>,
    ) -> Result<(), AnalysisError> {
        // Additional Python-specific symbol extraction
        // - Decorators
        // - List/Dict comprehensions
        // - Lambda functions
        // - Global variables
        Ok(())
    }
}

// Tree-sitter queries for Python
const PYTHON_FUNCTION_QUERY: &str = r#"
(function_definition
  name: (identifier) @name
)
"#;

const PYTHON_CLASS_QUERY: &str = r#"
(class_definition
  name: (identifier) @name
)
"#;

const PYTHON_CALL_QUERY: &str = r#"
(call
  function: (identifier) @name
)
(call
  function: (attribute
    attribute: (identifier) @name)
)
"#;
