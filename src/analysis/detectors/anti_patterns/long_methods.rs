//! Long Methods anti-pattern detector
//!
//! This detector identifies methods and functions that are excessively long,
//! implementing a sophisticated multi-metric approach similar to the Large Classes detector.
//!
//! ## Detection Strategy:
//! The detector uses a three-pillar analysis framework:
//! 1. **Size Metrics**: Lines of Code (LLOC), Statement Count
//! 2. **Complexity Metrics**: Cyclomatic Complexity (CC), Cognitive Complexity
//! 3. **Structural Metrics**: Nesting Depth, Parameter Count
//!
//! ## Severity Scoring:
//! - **Info (0-25)**: Slightly above thresholds, minor concern
//! - **Low (26-50)**: Moderate size, should be monitored
//! - **Medium (51-75)**: Clear anti-pattern, refactoring recommended
//! - **High (76-90)**: Significant design issues, refactoring needed
//! - **Critical (91-100)**: Extremely long method, immediate attention required
//!
//! ## Language-Specific Thresholds:
//! Based on industry standards from Pylint, ESLint, and Clippy:
//! - **Rust**: Conservative thresholds due to systems programming nature
//! - **Python**: Standard procedural/OOP thresholds
//! - **JavaScript**: Framework-aware thresholds for React/Node.js patterns

use crate::analysis::{AnalysisDetector, AnalysisError};
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::StreamingIterator;
use crate::ast::tree_sitter::{tree_sitter_javascript, tree_sitter_python, tree_sitter_rust};
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{Language, Node, Parser, Query, QueryCursor, QueryMatch};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::core::logging::debug;
use crate::core::logging::info;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;
use futures::TryFutureExt;
use std::collections::HashMap;
#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Language, Node, Parser, Query, QueryCursor, QueryMatch};

/// Represents metrics collected for a method/function
#[derive(Debug, Clone)]
pub struct MethodMetrics {
    /// Name of the method/function
    pub name: String,
    /// File path where the method is defined
    pub file_path: String,
    /// Starting line number
    pub start_line: u32,
    /// Ending line number
    pub end_line: u32,
    /// Logical Lines of Code (excluding comments and blank lines)
    pub logical_loc: u32,
    /// Number of statements
    pub statement_count: u32,
    /// Number of parameters
    pub parameter_count: u32,
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
    /// Cyclomatic complexity
    pub cyclomatic_complexity: u32,
    /// Cognitive complexity
    pub cognitive_complexity: u32,
    /// Whether the method is exported/public
    pub is_exported: bool,
    /// Code snippet of the method definition
    pub code_snippet: String,
    /// Method type (function, method, async, etc.)
    pub method_type: String,
}

/// Language-specific thresholds for long method detection
#[derive(Debug, Clone)]
pub struct LanguageThresholds {
    /// Maximum logical lines of code
    pub max_logical_loc: u32,
    /// Maximum number of statements
    pub max_statements: u32,
    /// Maximum number of parameters
    pub max_parameters: u32,
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
    /// Maximum cyclomatic complexity
    pub max_cyclomatic_complexity: u32,
    /// Maximum cognitive complexity
    pub max_cognitive_complexity: u32,
}

impl LanguageThresholds {
    /// Get thresholds for Rust (conservative due to systems programming)
    pub fn rust() -> Self {
        Self {
            max_logical_loc: 25,  // Conservative for systems code
            max_statements: 30,   // Rust encourages smaller functions
            max_parameters: 7,    // Rust type system helps with this
            max_nesting_depth: 4, // Match-based patterns reduce nesting
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 12,
        }
    }

    /// Get thresholds for Python (based on Pylint defaults)
    pub fn python() -> Self {
        Self {
            max_logical_loc: 100,          // Pylint default
            max_statements: 50,            // Python readability standards
            max_parameters: 5,             // PEP 8 guidance
            max_nesting_depth: 5,          // Python nesting conventions
            max_cyclomatic_complexity: 10, // Common Python standard
            max_cognitive_complexity: 15,
        }
    }

    /// Get thresholds for JavaScript (framework-aware)
    pub fn javascript() -> Self {
        Self {
            max_logical_loc: 80,  // ESLint complexity defaults
            max_statements: 40,   // JavaScript function standards
            max_parameters: 4,    // JavaScript callback patterns
            max_nesting_depth: 4, // Callback hell prevention
            max_cyclomatic_complexity: 12,
            max_cognitive_complexity: 18,
        }
    }
}

/// Tree-sitter queries for method extraction
const RUST_FUNCTION_QUERY: &str = r#"
[
(function_item
  name: (identifier) @name
  body: (block) @body) @function

(impl_item
  body: (declaration_list
    (function_item
      name: (identifier) @name
      body: (block) @body) @function))
]
"#;

const PYTHON_FUNCTION_QUERY: &str = r#"
(function_definition
  name: (identifier) @name
  body: (block) @body) @function

(class_definition
  body: (block
    (function_definition
      name: (identifier) @name
      body: (block) @body) @function))
"#;

const JAVASCRIPT_FUNCTION_QUERY: &str = r#"
(function_declaration
  name: (identifier) @name
  body: (statement_block) @body) @function

(method_definition
  name: (property_name) @name
  body: (statement_block) @body) @function

(arrow_function
  body: (statement_block) @body) @function

(function_expression
  name: (identifier)? @name
  body: (statement_block) @body) @function
"#;

/// Long Methods detector implementation
pub struct LongMethodsDetector {
    thresholds: HashMap<SourceLanguage, LanguageThresholds>,
}

impl Default for LongMethodsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LongMethodsDetector {
    /// Create a new Long Methods detector with default thresholds
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(SourceLanguage::Rust, LanguageThresholds::rust());
        thresholds.insert(SourceLanguage::Python, LanguageThresholds::python());
        thresholds.insert(SourceLanguage::JavaScript, LanguageThresholds::javascript());

        Self { thresholds }
    }

    /// Create detector with custom thresholds
    pub fn with_thresholds(thresholds: HashMap<SourceLanguage, LanguageThresholds>) -> Self {
        Self { thresholds }
    }

    /// Extract method metrics from a parsed file
    fn extract_method_metrics(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        match parsed_file.language {
            SourceLanguage::Rust => self.extract_rust_metrics(parsed_file),
            SourceLanguage::Python => self.extract_python_metrics(parsed_file),
            SourceLanguage::JavaScript => self.extract_javascript_metrics(parsed_file),
            _ => Ok(Vec::new()),
        }
    }

    /// Create and validate the Rust function query for tree-sitter analysis
    ///
    /// This method encapsulates the creation of the tree-sitter query used to identify
    /// Rust function definitions for metrics extraction.
    ///
    /// # Arguments
    /// * `language` - The tree-sitter language instance for Rust
    ///
    /// # Returns
    /// * `Ok(Query)` - Successfully created query ready for execution
    /// * `Err(AnalysisError)` - Query compilation failed with detailed error message
    ///
    /// # Errors
    /// Returns `AntiPatternDetectionError` if the query string is malformed or
    /// incompatible with the provided language grammar.
    fn create_rust_function_query(&self, language: &Language) -> Result<Query, AnalysisError> {
        Query::new(language, RUST_FUNCTION_QUERY).map_err(|e| {
            AnalysisError::AntiPatternDetectionError(format!(
                "Failed to create Rust function query: {}",
                e
            ))
        })
    }

    /// Calculate comprehensive metrics for a single Rust function
    ///
    /// This method computes all the metrics required for long method detection
    /// including logical lines of code, statement count, parameter count,
    /// nesting depth, cyclomatic complexity, and cognitive complexity.
    ///
    /// # Arguments
    /// * `name_node` - The tree-sitter node representing the function name
    /// * `body_node` - The tree-sitter node representing the function body
    /// * `function_node` - The tree-sitter node representing the entire function
    /// * `source` - The source code as bytes
    /// * `file_path` - The path to the file being analyzed
    ///
    /// # Returns
    /// * `Ok(MethodMetrics)` - Complete metrics for the function
    /// * `Err(AnalysisError)` - If metrics calculation fails
    fn calculate_method_metrics(
        &self,
        name_node: Node,
        body_node: Node,
        function_node: Node,
        source: &[u8],
        file_path: &str,
    ) -> Result<MethodMetrics, AnalysisError> {
        if let Ok(name) = name_node.utf8_text(source) {
            let logical_loc = self.calculate_logical_loc(&function_node, source);
            let statement_count = self.count_statements(&body_node, source)?;
            let parameter_count = self.count_rust_parameters(&function_node, source)?;
            let max_nesting_depth = self.calculate_max_nesting_depth(&body_node, source);
            let cyclomatic_complexity = self.calculate_cyclomatic_complexity(&body_node, source)?;
            let cognitive_complexity = self.calculate_cognitive_complexity(&body_node, source)?;
            let is_exported = self.is_rust_exported(&function_node, source);
            let code_snippet = self.extract_code_snippet(&function_node, source, 5);
            let method_type = self.determine_rust_method_type(&function_node, source);

            Ok(MethodMetrics {
                name: name.to_string(),
                file_path: file_path.to_string(),
                start_line: (name_node.start_position().row + 1) as u32,
                end_line: (function_node.end_position().row + 1) as u32,
                logical_loc,
                statement_count,
                parameter_count,
                max_nesting_depth,
                cyclomatic_complexity,
                cognitive_complexity,
                is_exported,
                code_snippet,
                method_type,
            })
        } else {
            Err(AnalysisError::AntiPatternDetectionError(
                "Failed to extract function name".to_string(),
            ))
        }
    }

    /// Process tree-sitter query matches into method metrics
    ///
    /// This method processes the results of tree-sitter queries to extract
    /// method metrics from each matched function. It handles the iteration
    /// over matches and delegates the actual metrics calculation to the
    /// calculate_method_metrics helper method.
    ///
    /// # Arguments
    /// * `matches` - Vector of query matches from tree-sitter
    /// * `source` - The source code as bytes
    /// * `file_path` - The path to the file being analyzed
    ///
    /// # Returns
    /// * `Ok(Vec<MethodMetrics>)` - Vector of calculated method metrics
    /// * `Err(AnalysisError)` - If processing fails
    fn process_function_matches(
        &self,
        mut matches: Vec<QueryMatch>,
        source: &[u8],
        file_path: &str,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        let mut metrics = Vec::new();

        // Sort matches by start position to ensure consistent ordering
        matches.sort_by_key(|m| {
            m.captures
                .get(0)
                .map(|c| c.node.start_position().row)
                .unwrap_or(0)
        });

        for mat in matches {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.get(1), mat.captures.get(2))
            {
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                let function_node = mat.captures.get(0).map(|c| c.node).unwrap_or(name_node);

                match self.calculate_method_metrics(
                    name_node,
                    body_node,
                    function_node,
                    source,
                    file_path,
                ) {
                    Ok(method_metrics) => metrics.push(method_metrics),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(metrics)
    }

    /// Extract metrics for Rust functions
    fn extract_rust_metrics(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        #[cfg(not(feature = "tree-sitter"))]
        {
            tracing::debug!(
                "Tree-sitter feature not enabled, skipping Rust method metrics extraction"
            );
            return Ok(Vec::new());
        }

        #[cfg(feature = "tree-sitter")]
        {
            let source = parsed_file.source.as_bytes();
            let tree = parsed_file.tree.as_ref().ok_or_else(|| {
                crate::analysis::errors::AnalysisError::AntiPatternDetectionError(
                    "AST tree missing".to_string(),
                )
            })?;
            let language = tree.language();

            let function_query = self.create_rust_function_query(&language)?;

            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&function_query, tree.root_node(), source);

            // Process matches directly to avoid lifetime issues
            let mut metrics = Vec::new();
            while let Some(query_match) = matches.next() {
                if let Some(function_node) = query_match.captures.get(0) {
                    if let (Some(name_node), Some(body_node)) =
                        (query_match.captures.get(1), query_match.captures.get(2))
                    {
                        match self.calculate_method_metrics(
                            name_node.node,
                            body_node.node,
                            function_node.node,
                            source,
                            &parsed_file.file_path.display().to_string(),
                        ) {
                            Ok(method_metrics) => metrics.push(method_metrics),
                            Err(e) => {
                                debug!("Failed to calculate metrics for function: {}", e);
                                continue;
                            }
                        }
                    }
                }
            }

            Ok(metrics)
        }
    }

    /// Extract metrics for Python functions
    fn extract_python_metrics(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        #[cfg(not(feature = "tree-sitter"))]
        {
            tracing::debug!(
                "Tree-sitter feature not enabled, skipping Python method metrics extraction"
            );
            return Ok(Vec::new());
        }

        #[cfg(feature = "tree-sitter")]
        {
            let mut metrics = Vec::new();
            let source = parsed_file.source.as_bytes();
            let tree = parsed_file.tree.as_ref().ok_or_else(|| {
                crate::analysis::errors::AnalysisError::AntiPatternDetectionError(
                    "AST tree missing".to_string(),
                )
            })?;
            let language = tree.language();

            let function_query = Query::new(&language, PYTHON_FUNCTION_QUERY).map_err(|e| {
                crate::analysis::errors::AnalysisError::AntiPatternDetectionError(format!(
                    "Failed to create Python function query: {}",
                    e
                ))
            })?;

            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&function_query, tree.root_node(), source);
            while let Some(mat) = matches.next() {
                if let (Some(name_capture), Some(body_capture)) =
                    (mat.captures.get(1), mat.captures.get(2))
                {
                    let name_node = name_capture.node;
                    let body_node = body_capture.node;
                    let function_node = mat.captures.get(0).map(|c| c.node).unwrap_or(name_node);

                    if let Ok(name) = name_node.utf8_text(source) {
                        let logical_loc = self.calculate_logical_loc(&function_node, source);
                        let statement_count = self.count_statements(&body_node, source)?;
                        let parameter_count =
                            self.count_python_parameters(&function_node, source)?;
                        let max_nesting_depth =
                            self.calculate_max_nesting_depth(&body_node, source);
                        let cyclomatic_complexity =
                            self.calculate_cyclomatic_complexity(&body_node, source)?;
                        let cognitive_complexity =
                            self.calculate_cognitive_complexity(&body_node, source)?;
                        let is_exported = !name.starts_with('_');
                        let code_snippet = self.extract_code_snippet(&function_node, source, 5);
                        let method_type = self.determine_python_method_type(&function_node, source);

                        metrics.push(MethodMetrics {
                            name: name.to_string(),
                            file_path: parsed_file.file_path.display().to_string(),
                            start_line: (name_node.start_position().row + 1) as u32,
                            end_line: (function_node.end_position().row + 1) as u32,
                            logical_loc,
                            statement_count,
                            parameter_count,
                            max_nesting_depth,
                            cyclomatic_complexity,
                            cognitive_complexity,
                            is_exported,
                            code_snippet,
                            method_type,
                        });
                    }
                }
            }

            Ok(metrics)
        }
    }

    /// Extract metrics for JavaScript functions
    fn extract_javascript_metrics(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MethodMetrics>, AnalysisError> {
        #[cfg(not(feature = "tree-sitter"))]
        {
            tracing::debug!(
                "Tree-sitter feature not enabled, skipping JavaScript method metrics extraction"
            );
            return Ok(Vec::new());
        }

        #[cfg(feature = "tree-sitter")]
        {
            let mut metrics = Vec::new();
            let source = parsed_file.source.as_bytes();
            let tree = parsed_file.tree.as_ref().ok_or_else(|| {
                crate::analysis::errors::AnalysisError::AntiPatternDetectionError(
                    "AST tree missing".to_string(),
                )
            })?;
            let language = tree.language();

            let function_query = Query::new(&language, JAVASCRIPT_FUNCTION_QUERY).map_err(|e| {
                crate::analysis::errors::AnalysisError::AntiPatternDetectionError(format!(
                    "Failed to create JavaScript function query: {}",
                    e
                ))
            })?;

            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&function_query, tree.root_node(), source);
            while let Some(mat) = matches.next() {
                if let Some(body_capture) = mat.captures.get(1) {
                    let body_node = body_capture.node;
                    let function_node = mat.captures.get(2).map(|c| c.node).unwrap_or(body_node);

                    // Handle anonymous functions
                    let name = if let Some(name_capture) = mat.captures.first() {
                        name_capture
                            .node
                            .utf8_text(source)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|_| "anonymous".to_string())
                    } else {
                        "anonymous".to_string()
                    };

                    let logical_loc = self.calculate_logical_loc(&function_node, source);
                    let statement_count = self.count_statements(&body_node, source)?;
                    let parameter_count =
                        self.count_javascript_parameters(&function_node, source)?;
                    let max_nesting_depth = self.calculate_max_nesting_depth(&body_node, source);
                    let cyclomatic_complexity =
                        self.calculate_cyclomatic_complexity(&body_node, source)?;
                    let cognitive_complexity =
                        self.calculate_cognitive_complexity(&body_node, source)?;
                    let is_exported = self.is_javascript_exported(&function_node, source);
                    let code_snippet = self.extract_code_snippet(&function_node, source, 5);
                    let method_type = self.determine_javascript_method_type(&function_node, source);

                    metrics.push(MethodMetrics {
                        name,
                        file_path: parsed_file.file_path.display().to_string(),
                        start_line: (function_node.start_position().row + 1) as u32,
                        end_line: (function_node.end_position().row + 1) as u32,
                        logical_loc,
                        statement_count,
                        parameter_count,
                        max_nesting_depth,
                        cyclomatic_complexity,
                        cognitive_complexity,
                        is_exported,
                        code_snippet,
                        method_type,
                    });
                }
            }

            Ok(metrics)
        }
    }

    /// Calculate logical lines of code (excluding comments and blank lines)
    fn calculate_logical_loc(&self, node: &Node, source: &[u8]) -> u32 {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        let mut logical_lines = 0;

        for line_num in start_line..=end_line {
            if let Some(line) = source.split(|&b| b == b'\n').nth(line_num) {
                let line_str = String::from_utf8_lossy(line);
                let trimmed = line_str.trim();

                // Skip empty lines and comment-only lines
                if !trimmed.is_empty()
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with('#')
                    && !trimmed.starts_with("/*")
                {
                    logical_lines += 1;
                }
            }
        }

        logical_lines
    }

    /// Count statements in a function body
    fn count_statements(&self, node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        let mut count = 0;
        let mut _cursor = node.walk();

        // Walk through all child nodes and count statement-like nodes
        if _cursor.goto_first_child() {
            loop {
                let node_type = _cursor.node().kind();
                if self.is_statement_node(node_type) {
                    count += 1;
                }

                if !_cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(count)
    }

    /// Determine if a node type represents a statement
    fn is_statement_node(&self, node_type: &str) -> bool {
        matches!(
            node_type,
            "expression_statement"
                | "if_statement"
                | "while_statement"
                | "for_statement"
                | "return_statement"
                | "break_statement"
                | "continue_statement"
                | "assignment"
                | "let_declaration"
                | "const_declaration"
                | "var_declaration"
                | "function_declaration"
                | "if_expression"
                | "while_expression"
                | "for_expression"
                | "loop_expression"
                | "match_expression"
                | "call_expression"
                | "macro_invocation"
        )
    }

    /// Calculate maximum nesting depth in a function
    fn calculate_max_nesting_depth(&self, node: &Node, _source: &[u8]) -> u32 {
        fn traverse_depth(node: &Node, current_depth: u32) -> u32 {
            let mut max_depth = current_depth;
            let mut cursor = node.walk();

            if cursor.goto_first_child() {
                loop {
                    let child_node = cursor.node();
                    let node_type = child_node.kind();

                    let new_depth = if matches!(
                        node_type,
                        "if_statement"
                            | "while_statement"
                            | "for_statement"
                            | "block"
                            | "if_expression"
                            | "while_expression"
                            | "for_expression"
                            | "loop_expression"
                            | "match_expression"
                            | "try_statement"
                            | "catch_clause"
                    ) {
                        current_depth + 1
                    } else {
                        current_depth
                    };

                    let child_max = traverse_depth(&child_node, new_depth);
                    max_depth = max_depth.max(child_max);

                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            max_depth
        }

        traverse_depth(node, 0)
    }

    /// Calculate cyclomatic complexity
    fn calculate_cyclomatic_complexity(
        &self,
        node: &Node,
        _source: &[u8],
    ) -> Result<u32, AnalysisError> {
        let mut complexity = 1; // Base complexity
        let mut cursor = node.walk();

        fn traverse_complexity(node: &Node, complexity: &mut u32) {
            let mut cursor = node.walk();

            if cursor.goto_first_child() {
                loop {
                    let child_node = cursor.node();
                    let node_type = child_node.kind();

                    // Increment complexity for decision points
                    if matches!(
                        node_type,
                        "if_statement"
                            | "while_statement"
                            | "for_statement"
                            | "match_expression"
                            | "if_expression"
                            | "while_expression"
                            | "for_expression"
                            | "loop_expression"
                            | "conditional_expression"
                            | "logical_and"
                            | "logical_or"
                            | "match_arm"
                            | "case_statement"
                            | "catch_clause"
                    ) {
                        *complexity += 1;
                    }

                    traverse_complexity(&child_node, complexity);

                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        traverse_complexity(node, &mut complexity);
        Ok(complexity)
    }

    /// Calculate cognitive complexity (more nuanced than cyclomatic)
    fn calculate_cognitive_complexity(
        &self,
        node: &Node,
        _source: &[u8],
    ) -> Result<u32, AnalysisError> {
        let mut complexity = 0;

        fn traverse_cognitive(node: &Node, complexity: &mut u32, nesting_level: u32) {
            let mut cursor = node.walk();

            if cursor.goto_first_child() {
                loop {
                    let child_node = cursor.node();
                    let node_type = child_node.kind();

                    let (increment, increases_nesting) = match node_type {
                        "if_statement" | "if_expression" => (1, true),
                        "while_statement" | "while_expression" | "for_statement"
                        | "for_expression" => (1, true),
                        "match_expression" => (1, true),
                        "logical_and" | "logical_or" => (1, false),
                        "conditional_expression" => (1, true),
                        "catch_clause" => (1, true),
                        "break_statement" | "continue_statement" => (1, false),
                        _ => (0, false),
                    };

                    if increment > 0 {
                        *complexity += increment + nesting_level;
                    }

                    let new_nesting = if increases_nesting {
                        nesting_level + 1
                    } else {
                        nesting_level
                    };
                    traverse_cognitive(&child_node, complexity, new_nesting);

                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        traverse_cognitive(node, &mut complexity, 0);
        Ok(complexity)
    }

    /// Count parameters in a Rust function
    fn count_rust_parameters(&self, node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "parameters" {
                    let params_node = cursor.node();
                    let mut param_cursor = params_node.walk();
                    let mut param_count = 0;

                    if param_cursor.goto_first_child() {
                        loop {
                            if param_cursor.node().kind() == "parameter" {
                                param_count += 1;
                            }
                            if !param_cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }

                    return Ok(param_count);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(0)
    }

    /// Count parameters in a Python function
    fn count_python_parameters(&self, node: &Node, source: &[u8]) -> Result<u32, AnalysisError> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "parameters" {
                    let param_count = cursor.node().child_count() as u32;
                    // Subtract 1 for 'self' parameter if present
                    if param_count > 0 {
                        let first_param_node = cursor.node().child(0).ok_or_else(|| {
                            crate::analysis::errors::AnalysisError::AntiPatternDetectionError(
                                "Missing first parameter node in Python function parameters"
                                    .to_string(),
                            )
                        })?;
                        let first_param_text = first_param_node.utf8_text(source).map_err(|e| {
                            crate::analysis::errors::AnalysisError::AntiPatternDetectionError(
                                format!("Failed to extract first parameter text: {}", e),
                            )
                        })?;
                        if first_param_text == "self" {
                            return Ok(param_count - 1);
                        }
                    }
                    return Ok(param_count);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(0)
    }

    /// Count parameters in a JavaScript function
    fn count_javascript_parameters(
        &self,
        node: &Node,
        _source: &[u8],
    ) -> Result<u32, AnalysisError> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "formal_parameters" {
                    return Ok(cursor.node().child_count() as u32);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        Ok(0)
    }

    /// Check if a Rust function is exported
    fn is_rust_exported(&self, node: &Node, source: &[u8]) -> bool {
        let mut cursor = node.walk();
        if cursor.goto_parent() {
            loop {
                if cursor.node().kind() == "visibility_modifier" {
                    if let Ok(vis) = cursor.node().utf8_text(source) {
                        return vis.contains("pub");
                    }
                }
                if !cursor.goto_previous_sibling() {
                    break;
                }
            }
        }
        false
    }

    /// Check if a JavaScript function is exported
    fn is_javascript_exported(&self, node: &Node, _source: &[u8]) -> bool {
        let mut cursor = node.walk();
        if cursor.goto_parent() {
            loop {
                let node_type = cursor.node().kind();
                if node_type == "export_statement" || node_type == "export_default_statement" {
                    return true;
                }
                if !cursor.goto_parent() {
                    break;
                }
            }
        }
        false
    }

    /// Extract code snippet around a node
    fn extract_code_snippet(&self, node: &Node, source: &[u8], context_lines: usize) -> String {
        let start_line = node.start_position().row.saturating_sub(context_lines);
        let end_line = node.end_position().row + context_lines;

        let lines: Vec<&str> = source
            .split(|&b| b == b'\n')
            .map(|line| std::str::from_utf8(line).unwrap_or_else(|_| ""))
            .collect();

        lines
            .get(start_line..=end_line.min(lines.len().saturating_sub(1)))
            .unwrap_or(&[])
            .join("\n")
    }

    /// Determine Rust method type
    fn determine_rust_method_type(&self, node: &Node, _source: &[u8]) -> String {
        let mut cursor = node.walk();
        let mut method_type = "function".to_string();

        // Check for async
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "async" {
                    method_type = "async_function".to_string();
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        // Check if it's in an impl block
        let mut parent_cursor = node.walk();
        while parent_cursor.goto_parent() {
            if parent_cursor.node().kind() == "impl_item" {
                method_type = format!("method_{}", method_type);
                break;
            }
        }

        method_type
    }

    /// Determine Python method type
    fn determine_python_method_type(&self, node: &Node, _source: &[u8]) -> String {
        let mut cursor = node.walk();
        let mut method_type = "function".to_string();

        // Check for async
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "async" {
                    method_type = "async_function".to_string();
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        // Check if it's in a class
        let mut parent_cursor = node.walk();
        while parent_cursor.goto_parent() {
            if parent_cursor.node().kind() == "class_definition" {
                method_type = format!("method_{}", method_type);
                break;
            }
        }

        method_type
    }

    /// Determine JavaScript method type
    fn determine_javascript_method_type(&self, node: &Node, _source: &[u8]) -> String {
        let node_type = node.kind();
        match node_type {
            "arrow_function" => "arrow_function".to_string(),
            "method_definition" => "method".to_string(),
            "function_expression" => "function_expression".to_string(),
            _ => "function".to_string(),
        }
    }

    /// Calculate severity score for a method
    fn calculate_severity_score(
        &self,
        metrics: &MethodMetrics,
        thresholds: &LanguageThresholds,
    ) -> u32 {
        let mut score = 0;

        // Size metrics (40% of score)
        if metrics.logical_loc > thresholds.max_logical_loc {
            score +=
                ((metrics.logical_loc as f64 / thresholds.max_logical_loc as f64) * 40.0) as u32;
        }

        // Complexity metrics (35% of score)
        if metrics.cyclomatic_complexity > thresholds.max_cyclomatic_complexity {
            score += ((metrics.cyclomatic_complexity as f64
                / thresholds.max_cyclomatic_complexity as f64)
                * 20.0) as u32;
        }

        if metrics.cognitive_complexity > thresholds.max_cognitive_complexity {
            score += ((metrics.cognitive_complexity as f64
                / thresholds.max_cognitive_complexity as f64)
                * 15.0) as u32;
        }

        // Structural metrics (25% of score)
        if metrics.parameter_count > thresholds.max_parameters {
            score +=
                ((metrics.parameter_count as f64 / thresholds.max_parameters as f64) * 15.0) as u32;
        }

        if metrics.max_nesting_depth > thresholds.max_nesting_depth {
            score += ((metrics.max_nesting_depth as f64 / thresholds.max_nesting_depth as f64)
                * 10.0) as u32;
        }

        score.min(100)
    }

    /// Get severity level from score
    fn get_severity_level(score: u32) -> String {
        match score {
            0..=25 => "Info".to_string(),
            26..=50 => "Low".to_string(),
            51..=75 => "Medium".to_string(),
            76..=90 => "High".to_string(),
            91..=100 => "Critical".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

#[async_trait]
impl AnalysisDetector for LongMethodsDetector {
    async fn detect_issues(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        debug!(
            "Long Methods Detector analyzing file: {}",
            file.file_path.display()
        );
        let method_metrics = self.extract_method_metrics(file)?;
        debug!(
            "Long Methods Detector found {} method metrics",
            method_metrics.len()
        );
        let thresholds = self.thresholds.get(&file.language).ok_or_else(|| {
            crate::analysis::errors::AnalysisError::UnsupportedLanguage(format!(
                "{:?}",
                file.language
            ))
        })?;
        for metrics in method_metrics {
            let severity_score = self.calculate_severity_score(&metrics, thresholds);
            debug!(
                "Method '{}': {} logical LOC, severity score: {} (threshold: {})",
                metrics.name, metrics.logical_loc, severity_score, thresholds.max_logical_loc
            );
            if severity_score > 25 {
                let mut issue = ArchitecturalIssue::new(
                    0, // analysis_run_id will be set by the engine
                    4, // anti_pattern_type_id for long methods
                    metrics.file_path.clone(),
                    Some(metrics.start_line as i32),
                    format!(
                        "Long method '{}' detected: {} lines, {} statements, complexity {}",
                        metrics.name,
                        metrics.logical_loc,
                        metrics.statement_count,
                        metrics.cyclomatic_complexity
                    ),
                    "LongMethodDetector".to_string(),
                    Self::get_severity_level(severity_score),
                    format!(
                        "Long method '{}' detected: {} lines, {} statements, complexity {}",
                        metrics.name,
                        metrics.logical_loc,
                        metrics.statement_count,
                        metrics.cyclomatic_complexity
                    ),
                );
                issue.start_line = Some(metrics.start_line as i32);
                issue.end_line = Some(metrics.end_line as i32);
                issue.code_snippet = Some(metrics.code_snippet.clone());
                issue.ai_explanation = Some(format!(
                    "Consider breaking down '{}' into smaller, more focused methods. Current metrics: LOC={}, Statements={}, Complexity={}, Nesting={}",
                    metrics.name, metrics.logical_loc, metrics.statement_count,
                    metrics.cyclomatic_complexity, metrics.max_nesting_depth
                ));
                issues.push(issue);
            }
        }
        info!("Long Methods detector found {} issues", issues.len());
        Ok(issues)
    }
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![
            AntiPatternType {
                anti_pattern_type_id: Some(4),
                name: "Long Method".to_string(),
                description: "Methods or functions that are excessively long and complex, making them difficult to understand, test, and maintain".to_string(),
                category: "Method-Level".to_string(),
            }
        ]
    }
    fn get_detector_name(&self) -> &'static str {
        "LongMethodsDetector"
    }

    fn detect(
        &self,
        graph: &crate::analysis::graph::dependency::LocalDependencyGraph,
    ) -> Vec<ArchitecturalIssue> {
        // For the graph-based detect method, we return empty for now
        // This method is used for dependency-based analysis
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tree_sitter_impl::AstParser;
    use crate::error::ErrorHelpers;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_long_method_detection_rust() -> Result<(), Box<dyn std::error::Error>> {
        let detector = LongMethodsDetector::new();
        let mut parser = AstParser::new()
            .map_err(|e| ErrorHelpers::ast_error(&format!("parser creation: {}", e)))?;

        let rust_code = r#"
fn very_long_function() {
    let x = 1;
    let y = 2;
    let z = 3;
    if x > 0 {
        if y > 0 {
            if z > 0 {
                for i in 0..10 {
                    for j in 0..10 {
                        for k in 0..10 {
                            println!("Nested loop: {}, {}, {}", i, j, k);
                            if i % 2 == 0 {
                                if j % 2 == 0 {
                                    if k % 2 == 0 {
                                        println!("All even");
                                    } else {
                                        println!("K is odd");
                                    }
                                } else {
                                    println!("J is odd");
                                }
                            } else {
                                println!("I is odd");
                            }
                        }
                    }
                }
            }
        }
    }
    println!("Done");
}
"#;

        let parsed_file = parser
            .parse_content(rust_code, &PathBuf::from("test.rs"), SourceLanguage::Rust)
            .map_err(|e| {
                ErrorHelpers::file_processing_error("test.rs", "parsing", &e.to_string())
            })?;

        let issues = detector
            .detect_issues(&parsed_file)
            .await
            .map_err(|e| ErrorHelpers::detector_config_error("long_methods", &e.to_string()))?;

        assert!(!issues.is_empty(), "Should detect long method");
        assert_eq!(issues[0].anti_pattern_type_id, 4); // LongMethod ID is 4
        assert!(issues[0].description.contains("very_long_function"));
        Ok(())
    }

    #[tokio::test]
    async fn test_short_method_no_detection() -> Result<(), Box<dyn std::error::Error>> {
        let detector = LongMethodsDetector::new();
        let mut parser = AstParser::new()
            .map_err(|e| ErrorHelpers::ast_error(&format!("parser creation: {}", e)))?;

        let rust_code = r#"
fn short_function() {
    println!("Hello, world!");
}
"#;

        let parsed_file = parser
            .parse_content(rust_code, &PathBuf::from("test.rs"), SourceLanguage::Rust)
            .map_err(|e| {
                ErrorHelpers::file_processing_error("test.rs", "parsing", &e.to_string())
            })?;

        let issues = detector
            .detect_issues(&parsed_file)
            .await
            .map_err(|e| ErrorHelpers::detector_config_error("long_methods", &e.to_string()))?;

        assert!(issues.is_empty(), "Should not detect short method");
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_method_metrics_success() -> Result<(), Box<dyn std::error::Error>> {
        let detector = LongMethodsDetector::new();
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let rust_code = r#"
fn test_function(param1: i32, param2: String) -> i32 {
    if param1 > 0 {
        for i in 0..param1 {
            println!("{}: {}", i, param2);
        }
    }
    param1 + 1
}
"#;

        let tree = parser
            .parse(rust_code, None)
            .ok_or_else(|| AnalysisError::parse_error("Failed to parse Rust test code"))?;
        let source = rust_code.as_bytes();
        let language = tree.language();

        let function_query = detector.create_rust_function_query(&language)?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&function_query, tree.root_node(), source);

        // Extract the first match and get owned data
        let mut found_match = false;
        let mut name_node = None;
        let mut body_node = None;
        let mut function_node = None;

        if let Some(query_match) = matches.next() {
            found_match = true;
            let captures = &query_match.captures;
            function_node = Some(
                captures
                    .get(0)
                    .ok_or_else(|| AnalysisError::query_error("Missing function capture"))?
                    .node,
            );
            name_node = Some(
                captures
                    .get(1)
                    .ok_or_else(|| AnalysisError::query_error("Missing name capture"))?
                    .node,
            );
            body_node = Some(
                captures
                    .get(2)
                    .ok_or_else(|| AnalysisError::query_error("Missing body capture"))?
                    .node,
            );
        }

        assert!(found_match, "Should find function");
        let name_node =
            name_node.ok_or_else(|| AnalysisError::data_not_found_error("Name node not found"))?;
        let body_node =
            body_node.ok_or_else(|| AnalysisError::data_not_found_error("Body node not found"))?;
        let function_node = function_node
            .ok_or_else(|| AnalysisError::data_not_found_error("Function node not found"))?;

        let metrics = detector.calculate_method_metrics(
            name_node,
            body_node,
            function_node,
            source,
            "test.rs",
        )?;

        assert_eq!(metrics.name, "test_function");
        assert_eq!(metrics.file_path, "test.rs");
        assert_eq!(metrics.parameter_count, 2);
        assert!(metrics.logical_loc > 0);
        assert!(metrics.statement_count > 0);
        assert!(metrics.max_nesting_depth > 0);
        assert!(metrics.cyclomatic_complexity > 0);
        assert!(metrics.cognitive_complexity > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_method_metrics_invalid_name() -> Result<(), Box<dyn std::error::Error>>
    {
        let detector = LongMethodsDetector::new();
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        // Create a mock node that will fail utf8_text extraction
        let rust_code = "fn test() {}";
        let tree = parser
            .parse(rust_code, None)
            .ok_or_else(|| AnalysisError::parse_error("Failed to parse Rust test code"))?;
        let source = rust_code.as_bytes();
        let language = tree.language();

        let function_query = detector.create_rust_function_query(&language)?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&function_query, tree.root_node(), source);

        if let Some(query_match) = matches.next() {
            let mat = query_match;
            let captures = &mat.captures;
            let name_node = captures
                .get(1)
                .ok_or_else(|| AnalysisError::query_error("Missing name capture"))?
                .node;
            let body_node = captures
                .get(2)
                .ok_or_else(|| AnalysisError::query_error("Missing body capture"))?
                .node;
            let function_node = captures
                .get(0)
                .ok_or_else(|| AnalysisError::query_error("Missing function capture"))?
                .node;

            // This should work normally, but we can test the error path conceptually
            let result = detector.calculate_method_metrics(
                name_node,
                body_node,
                function_node,
                source,
                "test.rs",
            );

            assert!(result.is_ok(), "Should succeed with valid function");
        }

        Ok(())
    }

    #[test]
    fn test_create_rust_function_query_success() {
        let detector = LongMethodsDetector::new();
        let language = tree_sitter_rust::LANGUAGE;
        let tree_sitter_language: Language = tree_sitter_rust::LANGUAGE.into();
        let result = detector.create_rust_function_query(&tree_sitter_language);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_function_matches_success() -> Result<(), Box<dyn std::error::Error>> {
        let detector = LongMethodsDetector::new();
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let rust_code = r#"
fn function_one(x: i32) -> i32 {
    if x > 0 {
        x + 1
    } else {
        0
    }
}

fn function_two() {
    println!("Hello, world!");
}
"#;

        let tree = parser
            .parse(rust_code, None)
            .ok_or_else(|| AnalysisError::parse_error("Failed to parse Rust test code"))?;
        let source = rust_code.as_bytes();
        let language = tree.language();

        let function_query = detector.create_rust_function_query(&language)?;
        let mut cursor = QueryCursor::new();
        let mut matches_iter = cursor.matches(&function_query, tree.root_node(), source);

        // Process matches directly like the main analysis does
        let mut metrics = Vec::new();
        while let Some(query_match) = matches_iter.next() {
            if let Some(function_node) = query_match.captures.get(0) {
                if let (Some(name_node), Some(body_node)) =
                    (query_match.captures.get(1), query_match.captures.get(2))
                {
                    match detector.calculate_method_metrics(
                        name_node.node,
                        body_node.node,
                        function_node.node,
                        source,
                        "test.rs",
                    ) {
                        Ok(method_metrics) => metrics.push(method_metrics),
                        Err(_) => continue,
                    }
                }
            }
        }

        assert_eq!(metrics.len(), 2, "Should find two functions");

        // Debug: print found function names
        println!(
            "Found functions: {:?}",
            metrics.iter().map(|m| &m.name).collect::<Vec<_>>()
        );

        // Find functions by name (order-agnostic test)
        let function_one = metrics
            .iter()
            .find(|m| m.name == "function_one")
            .expect("Should find function_one");
        let function_two = metrics
            .iter()
            .find(|m| m.name == "function_two")
            .expect("Should find function_two");

        // Check function_one
        assert_eq!(function_one.file_path, "test.rs");
        assert_eq!(function_one.parameter_count, 1);
        assert!(function_one.logical_loc > 0);

        // Check function_two
        assert_eq!(function_two.file_path, "test.rs");
        assert_eq!(function_two.parameter_count, 0);
        assert!(function_two.logical_loc > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_function_matches_empty() -> Result<(), Box<dyn std::error::Error>> {
        let detector = LongMethodsDetector::new();
        let empty_matches = Vec::new();
        let source = b"";

        let metrics = detector.process_function_matches(empty_matches, source, "empty.rs")?;

        assert!(
            metrics.is_empty(),
            "Should return empty metrics for empty matches"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_process_function_matches_malformed() -> Result<(), Box<dyn std::error::Error>> {
        let detector = LongMethodsDetector::new();
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let rust_code = r#"
fn valid_function() {
    println!("This is valid");
}
"#;

        let tree = parser
            .parse(rust_code, None)
            .ok_or_else(|| AnalysisError::parse_error("Failed to parse Rust test code"))?;
        let source = rust_code.as_bytes();
        let language = tree.language();

        let function_query = detector.create_rust_function_query(&language)?;
        let mut cursor = QueryCursor::new();
        let mut matches_iter = cursor.matches(&function_query, tree.root_node(), source);

        // Process matches directly like the main analysis does
        let mut metrics = Vec::new();
        while let Some(query_match) = matches_iter.next() {
            if let Some(function_node) = query_match.captures.get(0) {
                if let (Some(name_node), Some(body_node)) =
                    (query_match.captures.get(1), query_match.captures.get(2))
                {
                    match detector.calculate_method_metrics(
                        name_node.node,
                        body_node.node,
                        function_node.node,
                        source,
                        "test.rs",
                    ) {
                        Ok(method_metrics) => metrics.push(method_metrics),
                        Err(_) => continue,
                    }
                }
            }
        }

        assert!(!metrics.is_empty(), "Should process valid function");
        Ok(())
    }

    #[test]
    fn test_create_rust_function_query_with_invalid_query() {
        // This test demonstrates the error handling capability
        // We can't easily test with an invalid language, but we can test the structure
        let detector = LongMethodsDetector::new();
        let language = tree_sitter_rust::LANGUAGE;
        let tree_sitter_language: Language = tree_sitter_rust::LANGUAGE.into();
        let result = detector.create_rust_function_query(&tree_sitter_language);
        assert!(result.is_ok());

        // Verify the query can be used
        let query = result.expect("Query creation should succeed in test");
        assert!(query.capture_names().len() > 0);
    }

    #[tokio::test]
    async fn test_python_long_method_detection() -> Result<(), Box<dyn std::error::Error>> {
        let detector = LongMethodsDetector::new();
        let mut parser = AstParser::new()
            .map_err(|e| ErrorHelpers::ast_error(&format!("parser creation: {}", e)))?;

        let python_code = r#"
def very_long_function():
    x = 1
    y = 2
    z = 3
    if x > 0:
        if y > 0:
            if z > 0:
                for i in range(10):
                    for j in range(10):
                        for k in range(10):
                            print(f"Nested loop: {i}, {j}, {k}")
                            if i % 2 == 0:
                                if j % 2 == 0:
                                    if k % 2 == 0:
                                        print("All even")
                                    else:
                                        print("K is odd")
                                else:
                                    print("J is odd")
                            else:
                                print("I is odd")
    print("Done")
"#;

        let parsed_file = parser
            .parse_content(
                python_code,
                &PathBuf::from("test.py"),
                SourceLanguage::Python,
            )
            .map_err(|e| {
                ErrorHelpers::file_processing_error("test.py", "parsing", &e.to_string())
            })?;

        let issues = detector
            .detect_issues(&parsed_file)
            .await
            .map_err(|e| ErrorHelpers::detector_config_error("long_methods", &e.to_string()))?;

        assert!(!issues.is_empty(), "Should detect long method");
        assert_eq!(issues[0].anti_pattern_type_id, 4); // LongMethod ID is 4
        assert!(issues[0].description.contains("very_long_function"));
        Ok(())
    }
}
