//! Dead Code anti-pattern detector
//!
//! This detector implements a simplified version of the mark-and-sweep algorithm
//! described in the Dead Code Research document. It identifies potentially unused
//! functions, variables, and other symbols across multiple programming languages.
//!
//! ## Detection Strategy:
//! 1. **Symbol Collection**: Extract all function/variable definitions using Tree-sitter queries
//! 2. **Usage Analysis**: Find all references and calls to these symbols
//! 3. **Entry Point Detection**: Identify main functions, exports, and public APIs
//! 4. **Reachability Analysis**: Mark symbols as live if they're reachable from entry points
//! 5. **Dead Code Reporting**: Report symbols that remain unmarked as potentially dead
//!
//! ## Confidence Scoring:
//! - **High (0.8-1.0)**: Private/internal symbols with no references
//! - **Medium (0.5-0.7)**: Exported symbols in applications with no apparent usage
//! - **Low (0.2-0.4)**: Symbols in files with dynamic features (eval, decorators, etc.)
//!
//! ## Language Support:
//! - **Rust**: Functions, structs, enums, constants, modules
//! - **Python**: Functions, classes, variables, imports
//! - **JavaScript**: Functions, classes, variables, exports

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use log::{debug, info};
use std::collections::HashSet;
use tree_sitter::{Query, QueryCursor};

/// Represents a symbol (function, variable, class, etc.) found in the code
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Name of the symbol
    pub name: String,
    /// Type of symbol (function, variable, class, etc.)
    pub symbol_type: SymbolType,
    /// File path where the symbol is defined
    pub file_path: String,
    /// Line number where the symbol is defined
    pub line_number: u32,
    /// Whether the symbol is exported/public
    pub is_exported: bool,
    /// Whether the symbol has been marked as live during reachability analysis
    pub is_live: bool,
    /// Confidence score for dead code detection (0.0 to 1.0)
    pub confidence: f64,
    /// Source code snippet of the symbol definition
    pub code_snippet: String,
}

/// Types of symbols that can be detected as dead code
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Variable,
    Class,
    Struct,
    Enum,
    Constant,
    Module,
    Import,
}

/// Configuration for the dead code detector
#[derive(Debug, Clone)]
pub struct DeadCodeConfig {
    /// Minimum confidence threshold for reporting (0.0 to 1.0)
    pub min_confidence: f64,
    /// Whether to analyze exported symbols in library mode
    pub library_mode: bool,
    /// Patterns to ignore (e.g., test files, generated code)
    pub ignore_patterns: Vec<String>,
    /// Symbols to always consider live
    pub keep_alive_patterns: Vec<String>,
}

impl Default for DeadCodeConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            library_mode: false,
            ignore_patterns: vec![
                "test".to_string(),
                "tests".to_string(),
                "spec".to_string(),
                "mock".to_string(),
            ],
            keep_alive_patterns: vec![
                "main".to_string(),
                "init".to_string(),
                "setup".to_string(),
                "teardown".to_string(),
            ],
        }
    }
}

/// Dead code detector that identifies unused symbols across multiple languages
pub struct DeadCodeDetector {
    config: DeadCodeConfig,
}

impl DeadCodeDetector {
    pub fn new(config: DeadCodeConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(DeadCodeConfig::default())
    }

    /// Extract symbol definitions from a parsed file
    pub fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        match parsed_file.language {
            SourceLanguage::Rust => self.extract_rust_symbols(parsed_file),
            SourceLanguage::Python => self.extract_python_symbols(parsed_file),
            SourceLanguage::JavaScript => self.extract_javascript_symbols(parsed_file),
        }
    }

    /// Extract symbol references/calls from a parsed file
    pub fn extract_references(&self, parsed_file: &ParsedFile) -> Result<HashSet<String>, AnalysisError> {
        match parsed_file.language {
            SourceLanguage::Rust => self.extract_rust_references(parsed_file),
            SourceLanguage::Python => self.extract_python_references(parsed_file),
            SourceLanguage::JavaScript => self.extract_javascript_references(parsed_file),
        }
    }

    /// Extract Rust symbols (functions, structs, enums, constants)
    fn extract_rust_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        // Query for function definitions
        let function_query = Query::new(&language, RUST_FUNCTION_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&function_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let is_exported = self.is_rust_symbol_exported(&name_node, source);
                    let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                    
                    symbols.push(Symbol {
                        name: name.to_string(),
                        symbol_type: SymbolType::Function,
                        file_path: parsed_file.path.to_string_lossy().to_string(),
                        line_number: (name_node.start_position().row + 1) as u32,
                        is_exported,
                        is_live: false,
                        confidence: if is_exported { 0.6 } else { 0.9 },
                        code_snippet,
                    });
                }
            }
        }

        // Query for struct definitions
        let struct_query = Query::new(&language, RUST_STRUCT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&struct_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let is_exported = self.is_rust_symbol_exported(&name_node, source);
                    let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                    
                    symbols.push(Symbol {
                        name: name.to_string(),
                        symbol_type: SymbolType::Struct,
                        file_path: parsed_file.path.to_string_lossy().to_string(),
                        line_number: (name_node.start_position().row + 1) as u32,
                        is_exported,
                        is_live: false,
                        confidence: if is_exported { 0.6 } else { 0.9 },
                        code_snippet,
                    });
                }
            }
        }

        Ok(symbols)
    }

    /// Extract Python symbols (functions, classes, variables)
    fn extract_python_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        // Query for function definitions
        let function_query = Query::new(&language, PYTHON_FUNCTION_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&function_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let is_exported = !name.starts_with('_');
                    let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                    let confidence = self.calculate_python_confidence(name, &parsed_file.path);
                    
                    symbols.push(Symbol {
                        name: name.to_string(),
                        symbol_type: SymbolType::Function,
                        file_path: parsed_file.path.to_string_lossy().to_string(),
                        line_number: (name_node.start_position().row + 1) as u32,
                        is_exported,
                        is_live: false,
                        confidence,
                        code_snippet,
                    });
                }
            }
        }

        // Query for class definitions
        let class_query = Query::new(&language, PYTHON_CLASS_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&class_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let is_exported = !name.starts_with('_');
                    let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                    let confidence = self.calculate_python_confidence(name, &parsed_file.path);
                    
                    symbols.push(Symbol {
                        name: name.to_string(),
                        symbol_type: SymbolType::Class,
                        file_path: parsed_file.path.to_string_lossy().to_string(),
                        line_number: (name_node.start_position().row + 1) as u32,
                        is_exported,
                        is_live: false,
                        confidence,
                        code_snippet,
                    });
                }
            }
        }

        Ok(symbols)
    }

    /// Extract JavaScript symbols (functions, classes, variables)
    fn extract_javascript_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        // Query for function declarations
        let function_query = Query::new(&language, JAVASCRIPT_FUNCTION_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&function_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let is_exported = self.is_javascript_symbol_exported(&name_node, source);
                    let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                    let confidence = self.calculate_javascript_confidence(name, &parsed_file.path);
                    
                    symbols.push(Symbol {
                        name: name.to_string(),
                        symbol_type: SymbolType::Function,
                        file_path: parsed_file.path.to_string_lossy().to_string(),
                        line_number: (name_node.start_position().row + 1) as u32,
                        is_exported,
                        is_live: false,
                        confidence,
                        code_snippet,
                    });
                }
            }
        }

        Ok(symbols)
    }

    /// Extract Rust symbol references (function calls, variable usage)
    fn extract_rust_references(&self, parsed_file: &ParsedFile) -> Result<HashSet<String>, AnalysisError> {
        let mut references = HashSet::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        // Query for function calls
        let call_query = Query::new(&language, RUST_CALL_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&call_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    references.insert(name.to_string());
                }
            }
        }

        Ok(references)
    }

    /// Extract Python symbol references
    fn extract_python_references(&self, parsed_file: &ParsedFile) -> Result<HashSet<String>, AnalysisError> {
        let mut references = HashSet::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        // Query for function calls
        let call_query = Query::new(&language, PYTHON_CALL_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&call_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    references.insert(name.to_string());
                }
            }
        }

        Ok(references)
    }

    /// Extract JavaScript symbol references
    fn extract_javascript_references(&self, parsed_file: &ParsedFile) -> Result<HashSet<String>, AnalysisError> {
        let mut references = HashSet::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        // Query for function calls
        let call_query = Query::new(&language, JAVASCRIPT_CALL_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&call_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    references.insert(name.to_string());
                }
            }
        }

        Ok(references)
    }

    /// Check if a Rust symbol is exported (pub)
    fn is_rust_symbol_exported(&self, node: &tree_sitter::Node, source: &[u8]) -> bool {
        // Look for 'pub' keyword in the parent nodes
        let mut current = node.parent();
        while let Some(parent) = current {
            if let Ok(text) = parent.utf8_text(source) {
                if text.starts_with("pub ") {
                    return true;
                }
            }
            current = parent.parent();
        }
        false
    }

    /// Check if a JavaScript symbol is exported
    fn is_javascript_symbol_exported(&self, node: &tree_sitter::Node, source: &[u8]) -> bool {
        // Look for export keyword or module.exports
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

    /// Calculate confidence score for Python symbols
    fn calculate_python_confidence(&self, name: &str, file_path: &std::path::Path) -> f64 {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Lower confidence for test files or files with dynamic features
        if file_name.contains("test") || file_name.contains("spec") {
            return 0.3;
        }

        // Lower confidence for special methods
        if name.starts_with("__") && name.ends_with("__") {
            return 0.4;
        }

        // Lower confidence for private methods
        if name.starts_with('_') {
            return 0.7;
        }

        // Higher confidence for regular functions
        0.9
    }

    /// Calculate confidence score for JavaScript symbols
    fn calculate_javascript_confidence(&self, name: &str, file_path: &std::path::Path) -> f64 {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Lower confidence for test files
        if file_name.contains("test") || file_name.contains("spec") {
            return 0.3;
        }

        // Lower confidence for React components (might be used in JSX)
        if name.chars().next().map_or(false, |c| c.is_uppercase()) {
            return 0.6;
        }

        0.8
    }

    /// Extract a code snippet around a node
    fn extract_code_snippet(&self, node: &tree_sitter::Node, source: &[u8], context_lines: usize) -> String {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        
        if let Ok(snippet) = std::str::from_utf8(&source[start_byte..end_byte]) {
            // Take first few lines if the snippet is too long
            let lines: Vec<&str> = snippet.lines().take(context_lines).collect();
            lines.join("\n")
        } else {
            "Unable to extract snippet".to_string()
        }
    }

    /// Check if a symbol should be kept alive based on patterns
    fn should_keep_alive(&self, symbol: &Symbol) -> bool {
        for pattern in &self.config.keep_alive_patterns {
            if symbol.name.contains(pattern) {
                return true;
            }
        }
        false
    }
}

impl AnalysisDetector for DeadCodeDetector {
    fn get_detector_name(&self) -> &'static str {
        "DeadCodeDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(2),
            name: "Dead Code".to_string(),
            description: "Code that is defined but never used, including unused functions, variables, classes, and modules.".to_string(),
            category: "Maintainability".to_string(),
        }]
    }

    fn detect_issues(&self, parsed_file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!(
            "Running Dead Code detection on: {}",
            parsed_file.path.display()
        );

        // For single-file analysis, we can only detect obvious cases
        // Full dead code detection requires cross-file analysis
        let symbols = self.extract_symbols(parsed_file)?;
        let references = self.extract_references(parsed_file)?;

        let mut issues = Vec::new();

        for symbol in symbols {
            // Simple heuristic: if a symbol is not referenced in the same file
            // and it's not exported, it might be dead
            if !references.contains(&symbol.name) && 
               !symbol.is_exported && 
               !self.should_keep_alive(&symbol) &&
               symbol.confidence >= self.config.min_confidence {
                
                let severity = match symbol.confidence {
                    c if c >= 0.8 => "High",
                    c if c >= 0.6 => "Medium",
                    _ => "Low",
                };

                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: 0,
                    anti_pattern_type_id: 2, // Dead Code
                    file_path: symbol.file_path,
                    start_line: Some(symbol.line_number as i32),
                    end_line: Some(symbol.line_number as i32),
                    severity: severity.to_string(),
                    description: format!(
                        "Potentially dead code: {} '{}' is not used in this file (confidence: {:.1}%)",
                        format!("{:?}", symbol.symbol_type).to_lowercase(),
                        symbol.name,
                        symbol.confidence * 100.0
                    ),
                    code_snippet: Some(symbol.code_snippet),
                    ai_explanation: None,
                });
            }
        }

        if issues.is_empty() {
            debug!(
                "No dead code issues found in {}",
                parsed_file.path.display()
            );
        } else {
            info!(
                "Found {} potential dead code issues in {}",
                issues.len(),
                parsed_file.path.display()
            );
        }

        Ok(issues)
    }
}

// Tree-sitter queries for different languages

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

const JAVASCRIPT_CALL_QUERY: &str = r#"
(call_expression
  function: (identifier) @name
)
(call_expression
  function: (member_expression
    property: (property_identifier) @name)
)
"#;