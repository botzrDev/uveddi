//! Magic Values anti-pattern detector
//!
//! This detector implements static analysis for detecting "magic values" - unexplained
//! literals in source code that should be replaced with named constants for better
//! readability, maintainability, and reliability.
//!
//! ## Detection Strategy:
//! 1. **AST Traversal**: Use Tree-sitter queries to find all literal nodes
//! 2. **Contextual Analysis**: Examine parent nodes to determine if literal is in valid context
//! 3. **Value-Based Heuristics**: Apply allow-lists and pattern matching for common exceptions
//! 4. **Language-Specific Rules**: Handle language-specific idioms and patterns
//!
//! ## Supported Literals:
//! - **Numeric**: Integer and floating-point literals
//! - **String**: String literals used for behavior control
//! - **Boolean**: Boolean literals in unexpected contexts
//!
//! ## Exceptions (Not Flagged):
//! - Constants: 0, 1, -1, 2 (universal programming idioms)
//! - Array indices and loop bounds in obvious contexts
//! - Literals in constant declarations (const, static, final)
//! - Mathematical expressions in constant definitions
//! - Powers of 2 in bitwise operations
//! - Common time/percentage values (60, 100, 24, etc.) in self-documenting contexts

use async_trait::async_trait;
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::core::logging::debug;
use std::collections::HashSet;

#[cfg(feature = "tree-sitter")]
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};

/// Configuration for magic value detection
#[derive(Debug, Clone)]
pub struct MagicValuesConfig {
    /// Numbers that are universally acceptable and won't be flagged
    pub allowed_numbers: HashSet<i64>,
    /// Strings that are acceptable (punctuation, common delimiters)
    pub allowed_strings: HashSet<String>,
    /// Whether to ignore powers of 2 (common in bitwise operations)
    pub ignore_powers_of_two: bool,
    /// Whether to ignore common time/percentage values
    pub ignore_common_values: bool,
    /// Maximum string length to consider for magic string detection
    pub max_string_length: usize,
}

impl Default for MagicValuesConfig {
    fn default() -> Self {
        let mut allowed_numbers = HashSet::new();
        // Universal exceptions based on research document
        allowed_numbers.insert(-1); // Sentinel value for "not found"
        allowed_numbers.insert(0);  // Initialization, null representation
        allowed_numbers.insert(1);  // Increment, multiplicative identity
        allowed_numbers.insert(2);  // Even/odd checks, binary operations
        
        let mut allowed_strings = HashSet::new();
        // Common punctuation and delimiters
        allowed_strings.insert(",".to_string());
        allowed_strings.insert(";".to_string());
        allowed_strings.insert(" ".to_string());
        allowed_strings.insert("\n".to_string());
        allowed_strings.insert("\t".to_string());
        allowed_strings.insert("".to_string()); // Empty string
        
        Self {
            allowed_numbers,
            allowed_strings,
            ignore_powers_of_two: true,
            ignore_common_values: true,
            max_string_length: 50, // Don't flag very long strings (likely legitimate)
        }
    }
}

pub struct MagicValuesDetector {
    config: MagicValuesConfig,
}

impl Default for MagicValuesDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl MagicValuesDetector {
    pub fn new() -> Self {
        Self {
            config: MagicValuesConfig::default(),
        }
    }

    pub fn with_config(config: MagicValuesConfig) -> Self {
        Self { config }
    }

    /// Check if a number is a power of 2
    fn is_power_of_two(n: i64) -> bool {
        n > 0 && (n & (n - 1)) == 0
    }

    /// Check if a number is a common time/percentage value
    fn is_common_value(n: i64) -> bool {
        matches!(n, 10 | 24 | 60 | 100 | 360 | 365 | 1000 | 1024)
    }

    /// Check if a numeric literal should be ignored based on configuration
    fn should_ignore_number(&self, value: i64) -> bool {
        // Check allow-list
        if self.config.allowed_numbers.contains(&value) {
            return true;
        }

        // Check powers of 2
        if self.config.ignore_powers_of_two && Self::is_power_of_two(value.abs()) {
            return true;
        }

        // Check common values
        if self.config.ignore_common_values && Self::is_common_value(value.abs()) {
            return true;
        }

        false
    }

    /// Check if a string literal should be ignored
    fn should_ignore_string(&self, value: &str) -> bool {
        // Check allow-list
        if self.config.allowed_strings.contains(value) {
            return true;
        }

        // Ignore very long strings (likely legitimate data)
        if value.len() > self.config.max_string_length {
            return true;
        }

        // Ignore strings that look like log messages (contain spaces and common words)
        if value.contains(' ') && (value.to_lowercase().contains("error") || 
                                   value.to_lowercase().contains("info") ||
                                   value.to_lowercase().contains("debug") ||
                                   value.to_lowercase().contains("warning")) {
            return true;
        }

        false
    }

    /// Check if a literal is in a valid context (constant declaration, etc.)
    #[cfg(feature = "tree-sitter")]
    fn is_in_valid_context(&self, node: Node, source: &str) -> bool {
        let mut current = node.parent();
        let mut depth = 0;
        const MAX_DEPTH: usize = 5; // Prevent infinite loops

        while let Some(parent) = current {
            if depth > MAX_DEPTH {
                break;
            }

            let kind = parent.kind();
            debug!("Checking parent context: {}", kind);

            match kind {
                // Constant declarations (language-specific patterns)
                "const_item" | "static_item" => return true, // Rust
                "const_declaration" | "variable_declaration" => {
                    // Check if it's a const in JS/TS
                    if let Ok(text) = parent.utf8_text(source.as_bytes()) {
                        if text.trim_start().starts_with("const ") {
                            return true;
                        }
                    }
                    return false;
                }
                "field_declaration" => {
                    // Check for final/static modifiers in Java
                    if let Ok(text) = parent.utf8_text(source.as_bytes()) {
                        if text.contains("final") || text.contains("static") {
                            return true;
                        }
                    }
                    return false;
                }
                // Array indices and subscript expressions
                "index_expression" | "subscript_expression" => return true,
                // Loop constructs
                "for_statement" | "for_in_statement" | "while_statement" => return true,
                // Enum member definitions
                "enum_variant" | "enumerator" => return true,
                // Mathematical expressions in constant context
                "binary_expression" | "unary_expression" => {
                    // Continue checking parent for constant context
                }
                _ => {}
            }

            current = parent.parent();
            depth += 1;
        }

        false
    }

    #[cfg(feature = "tree-sitter")]
    async fn detect_magic_values_tree_sitter(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| AnalysisError::DetectionError("AST tree missing".to_string()))?;

        // Create queries for different types of literals based on language
        let queries = match parsed_file.language {
            SourceLanguage::Rust => vec![
                "(integer_literal) @number",
                "(float_literal) @number", 
                "(string_literal) @string",
                "(boolean_literal) @boolean",
            ],
            SourceLanguage::Python => vec![
                "(integer) @number",
                "(float) @number",
                "(string) @string",
                "(true) @boolean",
                "(false) @boolean",
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                "(number) @number",
                "(string) @string",
                "(true) @boolean", 
                "(false) @boolean",
            ],
        };

        for query_str in queries {
            let language = tree.language();
            let query = Query::new(&language, query_str)
                .map_err(|e| AnalysisError::QueryError(format!("Query error: {}", e)))?;

            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source);

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let node = capture.node;
                    let text = node.utf8_text(source)
                        .map_err(|e| AnalysisError::QueryError(format!("UTF-8 error: {}", e)))?;

                    debug!("Found literal: '{}' of kind: {}", text, node.kind());

                    // Skip if in valid context
                    let source_str = std::str::from_utf8(source).unwrap_or("");
                    if self.is_in_valid_context(node, source_str) {
                        debug!("Skipping literal '{}' - in valid context", text);
                        continue;
                    }

                    let should_flag = match capture.index {
                        idx if query.capture_names()[idx as usize] == "number" => {
                            // Parse and check numeric literals
                            if let Ok(int_val) = text.parse::<i64>() {
                                !self.should_ignore_number(int_val)
                            } else if let Ok(float_val) = text.parse::<f64>() {
                                // For floats, check if they're close to common mathematical constants
                                let abs_val = float_val.abs();
                                !(abs_val < 0.0001 || // Very small numbers
                                  (abs_val - std::f64::consts::PI).abs() < 0.01 || // π
                                  (abs_val - std::f64::consts::E).abs() < 0.01 ||  // e
                                  abs_val == 1.0 || abs_val == 0.5 || abs_val == 0.1) // Common fractions
                            } else {
                                true // Unknown format, flag it
                            }
                        }
                        idx if query.capture_names()[idx as usize] == "string" => {
                            // Remove quotes for analysis
                            let string_content = text.trim_matches('"').trim_matches('\'');
                            !self.should_ignore_string(string_content)
                        }
                        idx if query.capture_names()[idx as usize] == "boolean" => {
                            // Boolean literals are usually fine in most contexts
                            // Only flag if they appear in suspicious places
                            false // For now, don't flag boolean literals
                        }
                        _ => false,
                    };

                    if should_flag {
                        let start_pos = node.start_position();
                        let issue = ArchitecturalIssue {
                            issue_id: None,
                            analysis_run_id: 0, // Will be set by the analysis engine
                            anti_pattern_type_id: 0, // Will be set by the analysis engine
                            file_path: parsed_file.file_path.to_string_lossy().to_string(),
                            start_line: Some(start_pos.row as i32 + 1),
                            end_line: Some(node.end_position().row as i32 + 1),
                            line_number: Some(start_pos.row as i32 + 1),
                            column_number: Some(start_pos.column as i32),
                            message: format!(
                                "Magic {} '{}' found at line {}. Consider replacing with a named constant.",
                                if query.capture_names()[capture.index as usize] == "string" { "string" } else { "number" },
                                text,
                                start_pos.row + 1
                            ),
                            metadata: "{}".to_string(),
                            detector_name: self.get_detector_name().to_string(),
                            created_at: chrono::Utc::now(),
                            severity: "Medium".to_string(),
                            description: format!(
                                "Magic {} '{}' found. Consider replacing with a named constant for better readability and maintainability.",
                                if query.capture_names()[capture.index as usize] == "string" { "string" } else { "number" },
                                text
                            ),
                            code_snippet: Some(text.to_string()),
                            ai_explanation: None,
                        };
                        issues.push(issue);
                    }
                }
            }
        }

        Ok(issues)
    }

    #[cfg(not(feature = "tree-sitter"))]
    async fn detect_magic_values_fallback(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Simple regex-based fallback for when tree-sitter is not available
        use regex::Regex;
        
        let mut issues = Vec::new();
        let content = parsed_file.source.as_str();
        
        // Simple patterns for obvious magic numbers (not in const context)
        let number_pattern = Regex::new(r"(?m)^(?!.*\b(?:const|final|static)\b).*?\b(\d+(?:\.\d+)?)\b")
            .map_err(|e| AnalysisError::DetectionError(format!("Regex compilation error: {}", e)))?;
            
        for (line_num, line) in content.lines().enumerate() {
            for cap in number_pattern.captures_iter(line) {
                if let Some(number_match) = cap.get(1) {
                    let text = number_match.as_str();
                    
                    // Basic filtering
                    if let Ok(int_val) = text.parse::<i64>() {
                        if self.should_ignore_number(int_val) {
                            continue;
                        }
                    }
                    
                    let issue = ArchitecturalIssue {
                        issue_id: None,
                        analysis_run_id: 0,
                        anti_pattern_type_id: 0,
                        file_path: parsed_file.file_path.to_string_lossy().to_string(),
                        start_line: Some(line_num as i32 + 1),
                        end_line: Some(line_num as i32 + 1),
                        line_number: Some(line_num as i32 + 1),
                        column_number: None,
                        message: format!("Potential magic number '{}' found at line {}", text, line_num + 1),
                        metadata: "{}".to_string(),
                        detector_name: self.get_detector_name().to_string(),
                        created_at: chrono::Utc::now(),
                        severity: "Low".to_string(),
                        description: format!("Potential magic number '{}' found. Consider using a named constant.", text),
                        code_snippet: Some(line.trim().to_string()),
                        ai_explanation: None,
                    };
                    issues.push(issue);
                }
            }
        }
        
        Ok(issues)
    }
}

#[async_trait]
impl AnalysisDetector for MagicValuesDetector {
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!("Running Magic Values detection on: {:?}", parsed_file.file_path);

        #[cfg(feature = "tree-sitter")]
        {
            self.detect_magic_values_tree_sitter(parsed_file).await
        }

        #[cfg(not(feature = "tree-sitter"))]
        {
            self.detect_magic_values_fallback(parsed_file).await
        }
    }

    fn get_detector_name(&self) -> &'static str {
        "MagicValuesDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: None,
            name: "Magic Values".to_string(),
            description: "Unexplained numeric or string literals that should be replaced with named constants for better code readability and maintainability".to_string(),
            category: "maintainability".to_string(),
        }]
    }
}
