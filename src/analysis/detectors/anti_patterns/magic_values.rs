//! Magic Values anti-pattern detector
//!
//! This detector implements a sophisticated AST-based approach for identifying magic values
//! (unexplained literals) across multiple programming languages. Based on comprehensive
//! research and industry best practices, it applies contextual heuristics to distinguish
//! between problematic magic values and legitimate, self-explanatory constants.
//!
//! ## Detection Strategy:
//! - Uses Tree-sitter AST analysis for context-aware detection
//! - Applies language-specific allow-lists and contextual exclusions
//! - Implements advanced heuristics for powers of 2, mathematical constants, etc.
//! - Focuses on literals in behavior-controlling contexts (comparisons, assignments, etc.)

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::core::logging::debug;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;
use std::collections::HashSet;
#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Configuration for magic value detection with language-specific settings
#[derive(Debug, Clone)]
pub struct MagicValuesConfig {
    /// Universal exceptions (typically 0, 1, -1, 2)
    pub allowed_integers: HashSet<i64>,
    /// Universal exceptions for floating-point values
    pub allowed_floats: HashSet<String>,
    /// Whether to ignore powers of 2 (common in bitwise operations)
    pub ignore_powers_of_two: bool,
    /// Whether to ignore array indices
    pub ignore_array_indices: bool,
    /// Whether to ignore values in constant declarations
    pub ignore_const_declarations: bool,
    /// Whether to ignore mathematical constants (pi approximations, etc.)
    pub ignore_math_constants: bool,
    /// Maximum string length to consider for magic string detection
    pub max_string_length: usize,
    /// Exclude simple punctuation and delimiters
    pub ignore_simple_strings: bool,
}

impl Default for MagicValuesConfig {
    fn default() -> Self {
        let mut allowed_integers = HashSet::new();
        // Universal exceptions based on industry research
        allowed_integers.insert(0);
        allowed_integers.insert(1);
        allowed_integers.insert(-1);
        allowed_integers.insert(2); // Common in modulo operations

        let mut allowed_floats = HashSet::new();
        allowed_floats.insert("0.0".to_string());
        allowed_floats.insert("1.0".to_string());
        allowed_floats.insert("-1.0".to_string());

        Self {
            allowed_integers,
            allowed_floats,
            ignore_powers_of_two: true,
            ignore_array_indices: true,
            ignore_const_declarations: true,
            ignore_math_constants: true,
            max_string_length: 50,
            ignore_simple_strings: true,
        }
    }
}

/// A detected magic value with contextual information
#[derive(Debug, Clone)]
pub struct MagicValue {
    pub value: String,
    pub value_type: MagicValueType,
    pub context: MagicValueContext,
    pub line: u32,
    pub column: u32,
    pub severity: MagicValueSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MagicValueType {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MagicValueContext {
    Comparison,
    Assignment,
    FunctionArgument,
    ReturnValue,
    ArrayIndex,
    ConstDeclaration,
    FieldInitialization,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MagicValueSeverity {
    High,   // Repeated values, comparison operations
    Medium, // Single-use in functions
    Low,    // Edge cases
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

    /// Check if an integer value should be ignored based on configured heuristics
    fn is_integer_allowed(&self, value: i64, context: &MagicValueContext) -> bool {
        // Universal exceptions
        if self.config.allowed_integers.contains(&value) {
            return true;
        }

        // Powers of 2 heuristic (common in bitwise operations)
        if self.config.ignore_powers_of_two && value > 0 && (value & (value - 1)) == 0 {
            return true;
        }

        // Array index context
        if self.config.ignore_array_indices && matches!(context, MagicValueContext::ArrayIndex) {
            return value >= 0 && value < 100; // Reasonable array index range
        }

        // Constant declaration context
        if self.config.ignore_const_declarations
            && matches!(context, MagicValueContext::ConstDeclaration)
        {
            return true;
        }

        false
    }

    /// Check if a float value should be ignored
    fn is_float_allowed(&self, value_str: &str, _value: f64, context: &MagicValueContext) -> bool {
        // Universal exceptions
        if self.config.allowed_floats.contains(value_str) {
            return true;
        }

        // Mathematical constants heuristic
        if self.config.ignore_math_constants {
            let normalized = value_str.replace("f32", "").replace("f64", "");
            // Common mathematical constants (pi approximations, e, etc.)
            if normalized.starts_with("3.14") || normalized.starts_with("2.71") {
                return true;
            }
        }

        // Constant declaration context
        if self.config.ignore_const_declarations
            && matches!(context, MagicValueContext::ConstDeclaration)
        {
            return true;
        }

        false
    }

    /// Check if a string value should be ignored
    fn is_string_allowed(&self, value: &str, context: &MagicValueContext) -> bool {
        // Ignore very long strings (likely not magic values)
        if value.len() > self.config.max_string_length {
            return true;
        }

        // Simple punctuation and delimiters
        if self.config.ignore_simple_strings {
            let simple_strings = [",", ";", " ", "\t", "\n", ":", ".", "-", "_", "/", "\\"];
            if simple_strings.contains(&value) {
                return true;
            }
        }

        // Constant declaration context
        if self.config.ignore_const_declarations
            && matches!(context, MagicValueContext::ConstDeclaration)
        {
            return true;
        }

        // Empty or very short strings are usually not magic
        if value.len() <= 1 {
            return true;
        }

        false
    }

    /// Determine context from AST node hierarchy
    fn determine_context(&self, node: &Node, source: &[u8]) -> MagicValueContext {
        let mut current = node.parent();

        while let Some(parent) = current {
            let parent_kind = parent.kind();

            match parent_kind {
                "const_item" | "const_declaration" | "variable_declarator" => {
                    // Check if this is a const/final declaration
                    if self.is_const_declaration(&parent, source) {
                        return MagicValueContext::ConstDeclaration;
                    }
                    return MagicValueContext::Assignment;
                }
                "binary_expression" | "comparison_expression" => {
                    return MagicValueContext::Comparison;
                }
                "index_expression" | "subscript_expression" => {
                    return MagicValueContext::ArrayIndex;
                }
                "call_expression" | "function_call" => {
                    return MagicValueContext::FunctionArgument;
                }
                "return_statement" => {
                    return MagicValueContext::ReturnValue;
                }
                "field_declaration" | "field_expression" => {
                    return MagicValueContext::FieldInitialization;
                }
                _ => {}
            }

            current = parent.parent();
        }

        MagicValueContext::Other
    }

    /// Check if a declaration is a constant (const, static, final, etc.)
    fn is_const_declaration(&self, node: &Node, source: &[u8]) -> bool {
        let text = node.utf8_text(source).unwrap_or("");
        text.contains("const")
            || text.contains("static")
            || text.contains("final")
            || text.contains("CONST")
    }

    /// Determine severity based on context and value characteristics
    fn determine_severity(
        &self,
        context: &MagicValueContext,
        _value: &MagicValueType,
    ) -> MagicValueSeverity {
        match context {
            MagicValueContext::Comparison | MagicValueContext::FunctionArgument => {
                MagicValueSeverity::High
            }
            MagicValueContext::Assignment | MagicValueContext::ReturnValue => {
                MagicValueSeverity::Medium
            }
            _ => MagicValueSeverity::Low,
        }
    }

    /// Analyze literals in Rust code
    async fn analyze_rust_literals(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MagicValue>, AnalysisError> {
        let mut magic_values = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            let source = parsed_file.source.as_bytes();

            // Query for numeric and string literals
            let query_str = r#"
                (integer_literal) @number
                (float_literal) @number  
                (string_literal) @string
                (char_literal) @string
            "#;

            let query = Query::new(&tree.language(), query_str).map_err(|e| {
                AnalysisError::tree_sitter_parse_error(
                    parsed_file.file_path.to_string_lossy().to_string(),
                    "rust",
                    0,
                    format!("Query creation failed: {}", e),
                )
            })?;

            let mut cursor = QueryCursor::new();
            let mut captures = cursor.matches(&query, tree.root_node(), source);

            while let Some(match_) = captures.next() {
                for capture in match_.captures {
                    let node = capture.node;
                    let text = node.utf8_text(source).unwrap_or("");
                    let context = self.determine_context(&node, source);

                    let start_position = node.start_position();
                    let line = start_position.row as u32 + 1;
                    let column = start_position.column as u32;

                    let capture_name = query.capture_names()[capture.index as usize];

                    match capture_name {
                        "number" => {
                            if let Ok(int_val) = text.parse::<i64>() {
                                if !self.is_integer_allowed(int_val, &context) {
                                    let value_type = MagicValueType::Integer(int_val);
                                    let severity = self.determine_severity(&context, &value_type);

                                    magic_values.push(MagicValue {
                                        value: text.to_string(),
                                        value_type,
                                        context: context.clone(),
                                        line,
                                        column,
                                        severity,
                                    });
                                }
                            } else if let Ok(float_val) = text.parse::<f64>() {
                                if !self.is_float_allowed(text, float_val, &context) {
                                    let value_type = MagicValueType::Float(float_val);
                                    let severity = self.determine_severity(&context, &value_type);

                                    magic_values.push(MagicValue {
                                        value: text.to_string(),
                                        value_type,
                                        context: context.clone(),
                                        line,
                                        column,
                                        severity,
                                    });
                                }
                            }
                        }
                        "string" => {
                            // Remove quotes for analysis
                            let string_content = text.trim_matches('"').trim_matches('\'');
                            if !self.is_string_allowed(string_content, &context) {
                                let value_type = MagicValueType::String(string_content.to_string());
                                let severity = self.determine_severity(&context, &value_type);

                                magic_values.push(MagicValue {
                                    value: text.to_string(),
                                    value_type,
                                    context: context.clone(),
                                    line,
                                    column,
                                    severity,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(magic_values)
    }

    /// Analyze literals in Python/JavaScript/TypeScript using similar approach
    async fn analyze_generic_literals(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<MagicValue>, AnalysisError> {
        let mut magic_values = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            let source = parsed_file.source.as_bytes();

            // Generic query that works for Python/JS/TS
            let query_str = match parsed_file.language {
                SourceLanguage::Python => {
                    r#"
                    (integer) @number
                    (float) @number
                    (string) @string
                "#
                }
                SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                    r#"
                    (number) @number
                    (string) @string
                "#
                }
                _ => return Ok(magic_values), // Skip unsupported languages
            };

            let query = Query::new(&tree.language(), query_str).map_err(|e| {
                AnalysisError::tree_sitter_parse_error(
                    parsed_file.file_path.to_string_lossy().to_string(),
                    "generic",
                    0,
                    format!("Query creation failed: {}", e),
                )
            })?;

            let mut cursor = QueryCursor::new();
            let mut captures = cursor.matches(&query, tree.root_node(), source);

            while let Some(match_) = captures.next() {
                for capture in match_.captures {
                    let node = capture.node;
                    let text = node.utf8_text(source).unwrap_or("");
                    let context = self.determine_context(&node, source);

                    let start_position = node.start_position();
                    let line = start_position.row as u32 + 1;
                    let column = start_position.column as u32;

                    let capture_name = query.capture_names()[capture.index as usize];

                    match capture_name {
                        "number" => {
                            if let Ok(int_val) = text.parse::<i64>() {
                                if !self.is_integer_allowed(int_val, &context) {
                                    let value_type = MagicValueType::Integer(int_val);
                                    let severity = self.determine_severity(&context, &value_type);

                                    magic_values.push(MagicValue {
                                        value: text.to_string(),
                                        value_type,
                                        context: context.clone(),
                                        line,
                                        column,
                                        severity,
                                    });
                                }
                            } else if let Ok(float_val) = text.parse::<f64>() {
                                if !self.is_float_allowed(text, float_val, &context) {
                                    let value_type = MagicValueType::Float(float_val);
                                    let severity = self.determine_severity(&context, &value_type);

                                    magic_values.push(MagicValue {
                                        value: text.to_string(),
                                        value_type,
                                        context: context.clone(),
                                        line,
                                        column,
                                        severity,
                                    });
                                }
                            }
                        }
                        "string" => {
                            let string_content =
                                text.trim_matches('"').trim_matches('\'').trim_matches('`');
                            if !self.is_string_allowed(string_content, &context) {
                                let value_type = MagicValueType::String(string_content.to_string());
                                let severity = self.determine_severity(&context, &value_type);

                                magic_values.push(MagicValue {
                                    value: text.to_string(),
                                    value_type,
                                    context: context.clone(),
                                    line,
                                    column,
                                    severity,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(magic_values)
    }

    /// Convert MagicValue to ArchitecturalIssue
    fn magic_value_to_issue(&self, magic_value: MagicValue, file_path: &str) -> ArchitecturalIssue {
        let context_desc = match magic_value.context {
            MagicValueContext::Comparison => "used in comparison operation",
            MagicValueContext::Assignment => "used in variable assignment",
            MagicValueContext::FunctionArgument => "passed as function argument",
            MagicValueContext::ReturnValue => "used in return statement",
            MagicValueContext::ArrayIndex => "used as array index",
            MagicValueContext::ConstDeclaration => "used in constant declaration",
            MagicValueContext::FieldInitialization => "used in field initialization",
            MagicValueContext::Other => "found in code",
        };

        let severity_level = match magic_value.severity {
            MagicValueSeverity::High => "high",
            MagicValueSeverity::Medium => "medium",
            MagicValueSeverity::Low => "low",
        };

        let suggestion = match magic_value.value_type {
            MagicValueType::Integer(_) | MagicValueType::Float(_) => {
                "Consider extracting this magic number into a named constant with a descriptive name."
            }
            MagicValueType::String(_) => {
                "Consider extracting this magic string into a named constant to improve maintainability."
            }
            MagicValueType::Boolean => {
                "Consider using a more descriptive boolean constant or enum value."
            }
        };

        ArchitecturalIssue {
            anti_pattern_type_id: 9, // Updated to use correct ID
            file_path: file_path.to_string(),
            line_number: Some(magic_value.line as i32),
            column_number: Some(magic_value.column as i32),
            description: format!(
                "Magic value '{}' {} ({}). {}",
                magic_value.value, context_desc, severity_level, suggestion
            ),
            severity: severity_level.to_string(),
            ..Default::default()
        }
    }
}

#[async_trait]
impl AnalysisDetector for MagicValuesDetector {
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!(
            "Analyzing file for magic values: {}",
            parsed_file.file_path.display()
        );

        let magic_values = match parsed_file.language {
            SourceLanguage::Rust => self.analyze_rust_literals(parsed_file).await?,
            SourceLanguage::Python | SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                self.analyze_generic_literals(parsed_file).await?
            }
        };

        let issues: Vec<ArchitecturalIssue> = magic_values
            .into_iter()
            .map(|mv| self.magic_value_to_issue(mv, &parsed_file.file_path.to_string_lossy()))
            .collect();

        debug!(
            "Found {} magic values in {}",
            issues.len(),
            parsed_file.file_path.display()
        );
        Ok(issues)
    }

    fn get_detector_name(&self) -> &'static str {
        "MagicValuesDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(9), // Updated to use correct ID
            name: "Magic Values".to_string(),
            description:
                "Hard-coded numeric or string literals that should be replaced with named constants"
                    .to_string(),
            category: "maintainability".to_string(),
        }]
    }
}
