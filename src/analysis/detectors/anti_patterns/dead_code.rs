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
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::error::UveddiError;
use crate::analysis::memory::{DETECTOR_POOLS, PooledObject};
use log::{debug, info};
use std::collections::HashSet;

/// Represents a symbol (e.g., function, variable, class) identified in the source code.
///
/// A `Symbol` captures essential information about a code element, including its name,
/// type, location, and visibility. This information is used during dead code analysis
/// to determine if the symbol is ever used.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// The name of the symbol (e.g., `my_function`).
    pub name: String,
    /// The type of the symbol (e.g., `Function`, `Class`).
    pub symbol_type: SymbolType,
    /// The absolute path to the file where the symbol is defined.
    pub path: std::path::PathBuf, // Change to pub path: PathBuf
    /// The line number where the symbol's definition begins.
    pub line_number: u32,
    /// Indicates whether the symbol is public or exported, making it an entry point.
    pub is_exported: bool,
    /// A flag used during reachability analysis to mark the symbol as used.
    pub is_live: bool,
    /// A score from 0.0 to 1.0 indicating the confidence that this symbol is dead code.
    ///
    /// - **High (0.8-1.0)**: Private/internal symbols with no references.
    /// - **Medium (0.5-0.7)**: Exported symbols in applications with no apparent usage.
    /// - **Low (0.2-0.4)**: Symbols in files with dynamic features (e.g., reflection).
    pub confidence: f64,
    /// A snippet of the source code where the symbol is defined.
    pub code_snippet: String,
}

/// Enumerates the different types of symbols that can be analyzed for dead code.
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

/// Configures the behavior of the `DeadCodeDetector`.
///
/// This struct allows customization of the detection process, such as setting confidence
/// thresholds, ignoring specific files, and defining custom entry points.
#[derive(Debug, Clone)]
pub struct DeadCodeConfig {
    /// The minimum confidence score (0.0 to 1.0) a symbol must have to be reported as dead code.
    pub min_confidence: f64,
    /// If `true`, the detector treats all exported symbols as potential entry points,
    /// which is suitable for analyzing libraries. If `false`, it assumes an application
    /// context where unused exports might be dead code.
    pub library_mode: bool,
    /// A list of string patterns to exclude files from analysis.
    /// Useful for ignoring test directories, mocks, or generated code.
    pub ignore_patterns: Vec<String>,
    /// A list of symbol names to always consider "live," regardless of usage.
    /// Common examples include `main`, `init`, or framework-specific entry points.
    pub keep_alive_patterns: Vec<String>,
}

impl Default for DeadCodeConfig {
    /// Provides a default configuration for the `DeadCodeDetector`.
    ///
    /// - `min_confidence`: 0.5
    /// - `library_mode`: `false`
    /// - `ignore_patterns`: Includes common test and mock directories.
    /// - `keep_alive_patterns`: Includes `main`, `init`, `setup`, and `teardown`.
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

/// A detector for identifying unused (dead) code in a codebase.
///
/// This detector operates by performing a multi-language static analysis:
/// 1. **Symbol Collection**: It parses all source files to build a comprehensive list of every
///    function, class, variable, etc., defined in the project.
/// 2. **Usage Analysis**: It scans the codebase again to find all references to these symbols.
/// 3. **Reachability Analysis**: Starting from known entry points (like `main` or public APIs),
///    it traverses the call graph to mark all reachable symbols as "live."
/// 4. **Reporting**: Any symbol that is not marked as live is reported as potential dead code.
pub struct DeadCodeDetector {
    config: DeadCodeConfig,
}

impl DeadCodeDetector {
    /// Creates a new `DeadCodeDetector` with the given configuration.
    pub fn new(config: DeadCodeConfig) -> Self {
        Self { config }
    }

    /// Creates a new `DeadCodeDetector` with a pooled configuration.
    /// This reduces memory allocation overhead during analysis.
    pub fn with_pooled_config() -> (Self, PooledObject<DeadCodeConfig>) {
        let mut pooled_config = DETECTOR_POOLS.dead_code_configs.get();
        pooled_config.reset(); // Reset to default state
        let detector = Self::new(pooled_config.clone());
        (detector, pooled_config)
    }

    /// Creates a new `DeadCodeDetector` with a default configuration.
    pub fn with_default_config() -> Self {
        Self::new(DeadCodeConfig::default())
    }

    /// Extracts all symbol definitions from a single parsed file.
    ///
    /// This method dispatches to a language-specific extraction function based on the
    /// `ParsedFile`'s language.
    ///
    /// # Arguments
    ///
    /// * `parsed_file` - The file to analyze.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<Symbol>` or an `AnalysisError`.
    pub fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        // Use pooled vector for collecting symbols to reduce allocations
        let mut pooled_symbols = DETECTOR_POOLS.string_vectors.get();
        pooled_symbols.clear(); // Ensure clean state
        
        let symbols = match parsed_file.language {
            SourceLanguage::Rust => self.extract_rust_symbols(parsed_file)?,
            SourceLanguage::Python => self.extract_python_symbols(parsed_file)?,
            SourceLanguage::JavaScript => self.extract_javascript_symbols(parsed_file)?,
        };
        
        Ok(symbols)
    }

    /// Extracts all symbol references (calls, usages) from a single parsed file.
    ///
    /// This method is used to build the call graph for reachability analysis. It dispatches
    /// to a language-specific reference extraction function.
    ///
    /// # Arguments
    ///
    /// * `parsed_file` - The file to analyze.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashSet<String>` of referenced symbol names or an `AnalysisError`.
    pub fn extract_references(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError> {
        // Use pooled vector for collecting references to reduce allocations
        let mut pooled_refs = DETECTOR_POOLS.string_vectors.get();
        pooled_refs.clear(); // Ensure clean state
        
        let references = match parsed_file.language {
            SourceLanguage::Rust => self.extract_rust_references(parsed_file)?,
            SourceLanguage::Python => self.extract_python_references(parsed_file)?,
            SourceLanguage::JavaScript => self.extract_javascript_references(parsed_file)?,
        };
        
        Ok(references)
    }

    /// Extracts symbols from a Rust source file.
    ///
    /// Uses `tree-sitter` queries to find functions, structs, enums, and constants.
    fn extract_rust_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            crate::analysis::errors::AnalysisError::DetectionError(
                "AST tree missing".to_string(),
            )
        })?;
        let language = tree.language();

        // Query for function definitions
        let function_query = Query::new(&language, RUST_FUNCTION_QUERY).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(
                e.to_string(),
            )
        })?;

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

        // Query for struct definitions
        let struct_query = Query::new(&language, RUST_STRUCT_QUERY).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(
                e.to_string(),
            )
        })?;

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

        Ok(symbols)
    }

    /// Extracts symbols from a Python source file.
    ///
    /// Uses `tree-sitter` queries to find function and class definitions.
    fn extract_python_symbols(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let default_source = String::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            crate::analysis::errors::AnalysisError::DetectionError(
                "AST tree missing".to_string(),
            )
        })?;
        let language = tree.language();

        // Query for function definitions
        let function_query = Query::new(&language, PYTHON_FUNCTION_QUERY).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(
                e.to_string(),
            )
        })?;

        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&function_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let is_exported = !name.starts_with('_');
                    let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                    let confidence =
                        self.calculate_python_confidence(name, &(*parsed_file.file_path));

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

        // Query for class definitions
        let class_query = Query::new(&language, PYTHON_CLASS_QUERY).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(
                e.to_string(),
            )
        })?;

        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&class_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let is_exported = !name.starts_with('_');
                    let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                    let confidence =
                        self.calculate_python_confidence(name, &(*parsed_file.file_path));

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

        Ok(symbols)
    }

    /// Extracts symbols from a JavaScript source file.
    ///
    /// Uses `tree-sitter` queries to find function and class definitions.
    fn extract_javascript_symbols(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Symbol>, AnalysisError> {
        let mut symbols = Vec::new();
        let default_source = String::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            crate::analysis::errors::AnalysisError::DetectionError(
                "AST tree missing".to_string(),
            )
        })?;
        let language = tree.language();

        // Query for function declarations
        let function_query = Query::new(&language, JAVASCRIPT_FUNCTION_QUERY).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(
                e.to_string(),
            )
        })?;

        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&function_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let is_exported = self.is_javascript_symbol_exported(&name_node, source);
                    let code_snippet = self.extract_code_snippet(&name_node, source, 3);
                    let confidence =
                        self.calculate_javascript_confidence(name, &(*parsed_file.file_path));

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

        Ok(symbols)
    }

    /// Extracts references (calls, usages) to symbols in a Rust source file.
    ///
    /// Uses `tree-sitter` queries to find all function and variable usages.
    fn extract_rust_references(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError> {
        let mut references = HashSet::new();
        let default_source = String::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            crate::analysis::errors::AnalysisError::DetectionError(
                "AST tree missing".to_string(),
            )
        })?;
        let language = tree.language();

        // Query for function calls
        let call_query = Query::new(&language, RUST_CALL_QUERY).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(
                e.to_string(),
            )
        })?;

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

    /// Extracts references (calls, usages) to symbols in a Python source file.
    fn extract_python_references(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError> {
        let mut references = HashSet::new();
        let default_source = String::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            crate::analysis::errors::AnalysisError::DetectionError(
                "AST tree missing".to_string(),
            )
        })?;
        let language = tree.language();

        // Query for function calls
        let call_query = Query::new(&language, PYTHON_CALL_QUERY).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(
                e.to_string(),
            )
        })?;

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

    /// Extracts references (calls, usages) to symbols in a JavaScript source file.
    fn extract_javascript_references(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError> {
        let mut references = HashSet::new();
        let default_source = String::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            crate::analysis::errors::AnalysisError::DetectionError(
                "AST tree missing".to_string(),
            )
        })?;
        let language = tree.language();

        // Query for function calls
        let call_query = Query::new(&language, JAVASCRIPT_CALL_QUERY).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(
                e.to_string(),
            )
        })?;

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
    fn is_rust_symbol_exported(&self, node: &Node, source: &[u8]) -> bool {
        // The node we get is the identifier, we need to check the function_item parent
        let mut current = node.parent();
        while let Some(parent) = current {
            let kind = parent.kind();

            // Check if this is a function_item, struct_item, etc.
            if kind == "function_item"
                || kind == "struct_item"
                || kind == "enum_item"
                || kind == "const_item"
            {
                // Look for a visibility_modifier child that contains "pub"
                for i in 0..parent.child_count() {
                    if let Some(child) = parent.child(i) {
                        if child.kind() == "visibility_modifier" {
                            if let Ok(text) = child.utf8_text(source) {
                                return text.contains("pub");
                            }
                        }
                    }
                }
                // If we found the declaration node and no pub modifier, it's private
                return false;
            }

            current = parent.parent();
        }

        // Default to false if we can't determine visibility
        false
    }

    /// Check if a JavaScript symbol is exported
    fn is_javascript_symbol_exported(&self, node: &Node, source: &[u8]) -> bool {
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
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

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
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

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
    fn extract_code_snippet(&self, node: &Node, source: &[u8], context_lines: usize) -> String {
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

    fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!(
            "Running Dead Code detection on: {}",
            parsed_file.file_path.display()
        );

        // For single-file analysis, we can only detect obvious cases
        // Full dead code detection requires cross-file analysis
        let symbols = self.extract_symbols(parsed_file)?;
        let references = self.extract_references(parsed_file)?;

        // Use pooled vector for collecting issues to reduce allocations
        let mut pooled_issues = DETECTOR_POOLS.issue_vectors.get();
        pooled_issues.clear(); // Ensure clean state
        let mut issues = Vec::new();

        for symbol in symbols {
            // Simple heuristic: if a symbol is not referenced in the same file
            // and it's not exported, it might be dead
            let is_referenced = references.contains(&symbol.name);
            let should_keep = self.should_keep_alive(&symbol);

            // Debug logging for test troubleshooting
            debug!(
                "Symbol: {}, referenced: {}, exported: {}, keep_alive: {}",
                symbol.name, is_referenced, symbol.is_exported, should_keep
            );

            // In library mode, only report non-exported symbols
            // In application mode, report both exported and non-exported unused symbols
            let should_report = if self.config.library_mode {
                !symbol.is_exported // Library mode: only report non-exported symbols
            } else {
                true // Application mode: report all unused symbols
            };

            if !is_referenced
                && should_report
                && !should_keep
                && symbol.confidence >= self.config.min_confidence
            {
                let severity = match symbol.confidence {
                    c if c >= 0.8 => "High",
                    c if c >= 0.6 => "Medium",
                    _ => "Low",
                };

                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: 0,
                    anti_pattern_type_id: 2, // Dead Code
                    file_path: symbol.path.display().to_string(), // Use symbol.path.display()
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
                parsed_file.file_path.display()
            );
        } else {
            info!(
                "Found {} potential dead code issues in {}",
                issues.len(),
                parsed_file.file_path.display()
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
(call_expression
  function: (member_expression
    object: (identifier)
    property: (property_identifier) @name)
)
"#;
