//! Language-specific symbol extraction utilities
//!
//! This module provides a unified interface for extracting symbols from different
//! programming languages, eliminating the need for each detector to implement
//! language-specific extraction logic separately.

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use super::query_helper::{TreeSitterQueryHelper, queries};
use std::path::PathBuf;

/// Represents a symbol extracted from source code
#[derive(Debug, Clone)]
pub struct ExtractedSymbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub line_number: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub is_exported: bool,
    pub code_snippet: String,
    pub file_path: PathBuf,
}

/// The type of symbol found in source code
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Variable,
    Constant,
    Module,
    Interface,
    Unknown,
}

/// A unified interface for extracting symbols from different programming languages
pub struct LanguageSpecificExtractor {
    query_helper: TreeSitterQueryHelper,
}

impl LanguageSpecificExtractor {
    /// Create a new language-specific extractor
    pub fn new() -> Self {
        Self {
            query_helper: TreeSitterQueryHelper::new(),
        }
    }

    /// Extract all symbols from a parsed file based on its language
    pub fn extract_symbols(&mut self, parsed_file: &ParsedFile) -> Result<Vec<ExtractedSymbol>, AnalysisError> {
        match parsed_file.language {
            SourceLanguage::Rust => self.extract_rust_symbols(parsed_file),
            SourceLanguage::Python => self.extract_python_symbols(parsed_file),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => self.extract_javascript_symbols(parsed_file),
        }
    }

    /// Extract functions specifically from a parsed file
    pub fn extract_functions(&mut self, parsed_file: &ParsedFile) -> Result<Vec<ExtractedSymbol>, AnalysisError> {
        let all_symbols = self.extract_symbols(parsed_file)?;
        Ok(all_symbols.into_iter()
            .filter(|s| matches!(s.symbol_type, SymbolType::Function | SymbolType::Method))
            .collect())
    }

    /// Extract classes/structs specifically from a parsed file
    pub fn extract_types(&mut self, parsed_file: &ParsedFile) -> Result<Vec<ExtractedSymbol>, AnalysisError> {
        let all_symbols = self.extract_symbols(parsed_file)?;
        Ok(all_symbols.into_iter()
            .filter(|s| matches!(s.symbol_type, SymbolType::Class | SymbolType::Struct | SymbolType::Enum | SymbolType::Interface))
            .collect())
    }

    /// Extract Rust symbols (functions, structs, enums, impls, modules)
    fn extract_rust_symbols(&mut self, parsed_file: &ParsedFile) -> Result<Vec<ExtractedSymbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        // Define queries to run
        let queries = [
            ("functions", queries::rust::FUNCTION_QUERY),
            ("structs", queries::rust::STRUCT_QUERY),
            ("enums", queries::rust::ENUM_QUERY),
            ("modules", queries::rust::MOD_QUERY),
        ];

        let results = self.query_helper.execute_multiple_queries(&language, &queries, tree, source)?;

        // Process functions
        if let Some(functions) = results.get("functions") {
            for (node, name) in functions {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_rust_symbol_exported(node, source);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 2);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Function,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        // Process structs
        if let Some(structs) = results.get("structs") {
            for (node, name) in structs {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_rust_symbol_exported(node, source);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 2);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Struct,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        // Process enums
        if let Some(enums) = results.get("enums") {
            for (node, name) in enums {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_rust_symbol_exported(node, source);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 2);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Enum,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        // Process modules
        if let Some(modules) = results.get("modules") {
            for (node, name) in modules {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_rust_symbol_exported(node, source);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 2);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Module,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        Ok(symbols)
    }

    /// Extract Python symbols (functions, classes, variables)
    fn extract_python_symbols(&mut self, parsed_file: &ParsedFile) -> Result<Vec<ExtractedSymbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        let queries = [
            ("functions", queries::python::FUNCTION_QUERY),
            ("classes", queries::python::CLASS_QUERY),
            ("assignments", queries::python::ASSIGNMENT_QUERY),
        ];

        let results = self.query_helper.execute_multiple_queries(&language, &queries, tree, source)?;

        // Process functions
        if let Some(functions) = results.get("functions") {
            for (node, name) in functions {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_python_symbol_exported(&name);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 2);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Function,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        // Process classes
        if let Some(classes) = results.get("classes") {
            for (node, name) in classes {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_python_symbol_exported(&name);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 2);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Class,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        // Process variable assignments
        if let Some(assignments) = results.get("assignments") {
            for (node, name) in assignments {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_python_symbol_exported(&name);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 1);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Variable,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        Ok(symbols)
    }

    /// Extract JavaScript/TypeScript symbols (functions, classes, variables)
    fn extract_javascript_symbols(&mut self, parsed_file: &ParsedFile) -> Result<Vec<ExtractedSymbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        let queries = [
            ("functions", queries::javascript::FUNCTION_QUERY),
            ("classes", queries::javascript::CLASS_QUERY),
            ("variables", queries::javascript::VARIABLE_QUERY),
        ];

        let results = self.query_helper.execute_multiple_queries(&language, &queries, tree, source)?;

        // Process functions
        if let Some(functions) = results.get("functions") {
            for (node, name) in functions {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_javascript_symbol_exported(node, source);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 2);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: if node.kind() == "method_definition" { SymbolType::Method } else { SymbolType::Function },
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        // Process classes
        if let Some(classes) = results.get("classes") {
            for (node, name) in classes {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_javascript_symbol_exported(node, source);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 2);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Class,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        // Process variables
        if let Some(variables) = results.get("variables") {
            for (node, name) in variables {
                let (start_line, end_line, start_col, end_col) = TreeSitterQueryHelper::get_node_position(node);
                let is_exported = self.is_javascript_symbol_exported(node, source);
                let code_snippet = TreeSitterQueryHelper::extract_code_snippet(node, source, 1);

                symbols.push(ExtractedSymbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Variable,
                    line_number: start_line,
                    start_column: start_col,
                    end_line,
                    end_column: end_col,
                    is_exported,
                    code_snippet,
                    file_path: parsed_file.file_path.clone(),
                });
            }
        }

        Ok(symbols)
    }

    /// Check if a Rust symbol is exported (public)
    fn is_rust_symbol_exported(&self, node: &crate::ast::tree_sitter::Node, source: &[u8]) -> bool {
        // Look for "pub" keyword in the parent or current node
        let mut current = Some(*node);
        while let Some(n) = current {
            if let Ok(text) = n.utf8_text(source) {
                if text.trim_start().starts_with("pub ") {
                    return true;
                }
            }
            current = n.parent();
        }
        false
    }

    /// Check if a Python symbol is exported (doesn't start with underscore)
    fn is_python_symbol_exported(&self, name: &str) -> bool {
        !name.starts_with('_')
    }

    /// Check if a JavaScript symbol is exported
    fn is_javascript_symbol_exported(&self, node: &crate::ast::tree_sitter::Node, source: &[u8]) -> bool {
        // Look for export keyword or module.exports
        let mut current = Some(*node);
        while let Some(n) = current {
            if let Ok(text) = n.utf8_text(source) {
                if text.contains("export ") || text.contains("module.exports") {
                    return true;
                }
            }
            current = n.parent();
        }
        false
    }
}

impl Default for LanguageSpecificExtractor {
    fn default() -> Self {
        Self::new()
    }
}