//! Large Classes anti-pattern detector
//!
//! This detector implements a sophisticated multi-metric approach to identify Large Classes,
//! God Objects, and Blob anti-patterns as described in the Large Class Research document.
//!
//! ## Detection Strategy:
//! The detector uses a three-pillar analysis framework:
//! 1. **Size Metrics**: Lines of Code (LLOC), Number of Methods (NOM), Number of Fields (NOF)
//! 2. **Complexity Metrics**: Cyclomatic Complexity (CC), Cognitive Complexity
//! 3. **Structural Metrics**: Lack of Cohesion in Methods (LCOM), Coupling Between Objects (CBO)
//!
//! ## Severity Scoring:
//! - **Info (0-25)**: Slightly above thresholds, minor concern
//! - **Low (26-50)**: Moderate size, should be monitored
//! - **Medium (51-75)**: Clear anti-pattern, refactoring recommended
//! - **High (76-90)**: Significant design issues, refactoring needed
//! - **Critical (91-100)**: God Object, immediate attention required
//!
//! ## Language-Specific Thresholds:
//! Based on industry standards from Pylint, ESLint, and Clippy:
//! - **Rust**: Conservative thresholds due to systems programming nature
//! - **Python**: Standard object-oriented thresholds
//! - **JavaScript**: Framework-aware thresholds for React/Node.js patterns

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use log::{debug, info};
use std::collections::HashMap;
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Query, QueryCursor};

/// Holds the collected metrics for a single class or struct.
///
/// This struct aggregates various size, complexity, and structural metrics that are
/// used to evaluate whether a class has grown too large or is taking on too many
/// responsibilities.
#[derive(Debug, Clone)]
pub struct ClassMetrics {
    /// The name of the class or struct.
    pub name: String,
    /// The absolute path to the file where the class is defined.
    pub file_path: String,
    /// The line number where the class definition begins.
    pub start_line: u32,
    /// The line number where the class definition ends.
    pub end_line: u32,
    /// Logical Lines of Code, excluding comments and blank lines, within the class body.
    pub logical_loc: u32,
    /// The total number of methods or functions defined in the class.
    pub method_count: u32,
    /// The total number of fields, properties, or attributes in the class.
    pub field_count: u32,
    /// The sum of the cyclomatic complexity of all methods in the class.
    pub cyclomatic_complexity: u32,
    /// The sum of the cognitive complexity of all methods in the class.
    pub cognitive_complexity: u32,
    /// The Lack of Cohesion in Methods (LCOM) score. A higher score (closer to 1.0)
    /// indicates lower cohesion, meaning the class may have unrelated responsibilities.
    pub lcom_score: f64,
    /// The number of other classes this class depends on (Coupling Between Objects).
    pub coupling_count: u32,
    /// Indicates whether the class is public or exported.
    pub is_exported: bool,
    /// A snippet of the source code where the class is defined.
    pub code_snippet: String,
}

/// Defines language-specific thresholds for detecting large classes.
///
/// Because coding idioms and project structures vary by language, applying a single
/// set of thresholds is often ineffective. This struct allows for fine-tuning the
/// detection logic for Rust, Python, and JavaScript.
#[derive(Debug, Clone)]
pub struct LanguageThresholds {
    /// The maximum number of logical lines of code a class can have before being flagged.
    pub max_logical_loc: u32,
    /// The maximum number of methods allowed in a class.
    pub max_methods: u32,
    /// The maximum number of fields or attributes allowed in a class.
    pub max_fields: u32,
    /// The maximum total cyclomatic complexity for a class.
    pub max_cyclomatic_complexity: u32,
    /// The maximum total cognitive complexity for a class.
    pub max_cognitive_complexity: u32,
    /// The maximum LCOM score. Scores above this threshold suggest poor cohesion.
    pub max_lcom_score: f64,
    /// The maximum number of external dependencies (coupling) allowed.
    pub max_coupling: u32,
}

impl LanguageThresholds {
    /// Provides conservative thresholds suitable for Rust, which favors smaller,
    /// more focused structs and `impl` blocks.
    pub fn rust() -> Self {
        Self {
            max_logical_loc: 400,      // Conservative for systems code
            max_methods: 20,           // Rust encourages smaller impl blocks
            max_fields: 15,            // Structs should be focused
            max_cyclomatic_complexity: 50,
            max_cognitive_complexity: 40,
            max_lcom_score: 0.8,
            max_coupling: 12,
        }
    }

    /// Provides thresholds based on common Python style guides and tools like Pylint.
    pub fn python() -> Self {
        Self {
            max_logical_loc: 1000,     // Pylint default
            max_methods: 20,           // Pylint too-many-public-methods
            max_fields: 7,             // Pylint too-many-instance-attributes
            max_cyclomatic_complexity: 60,
            max_cognitive_complexity: 50,
            max_lcom_score: 0.8,
            max_coupling: 15,
        }
    }

    /// Provides thresholds that are more lenient to accommodate patterns common in
    /// JavaScript frameworks like React (e.g., components with state and event handlers).
    pub fn javascript() -> Self {
        Self {
            max_logical_loc: 800,      // Accommodates React components
            max_methods: 25,           // Higher for event handlers
            max_fields: 12,            // React state and props
            max_cyclomatic_complexity: 55,
            max_cognitive_complexity: 45,
            max_lcom_score: 0.8,
            max_coupling: 18,          // Higher for module imports
        }
    }
}

/// Configures the behavior of the `LargeClassDetector`.
///
/// This struct allows for customization of the detection process, including setting
/// language-specific thresholds, defining a minimum severity score for reporting,
/// and ignoring certain files.
#[derive(Debug, Clone)]
pub struct LargeClassConfig {
    /// A map of `SourceLanguage` to `LanguageThresholds` to be used during analysis.
    pub language_thresholds: HashMap<SourceLanguage, LanguageThresholds>,
    /// The minimum severity score (from 0 to 100) an issue must have to be reported.
    pub min_severity_score: u32,
    /// A list of string patterns to exclude files from analysis (e.g., test files).
    pub ignore_patterns: Vec<String>,
    /// If `true`, the generated issue description will include a detailed breakdown
    /// of the metrics that exceeded their thresholds.
    pub include_metrics_detail: bool,
}

impl Default for LargeClassConfig {
    /// Provides a default configuration for the `LargeClassDetector`.
    ///
    /// This includes standard thresholds for Rust, Python, and JavaScript, a minimum
    /// severity score of 25, and patterns to ignore common test directories.
    fn default() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(SourceLanguage::Rust, LanguageThresholds::rust());
        thresholds.insert(SourceLanguage::Python, LanguageThresholds::python());
        thresholds.insert(SourceLanguage::JavaScript, LanguageThresholds::javascript());

        Self {
            language_thresholds: thresholds,
            min_severity_score: 25, // Report Low severity and above
            ignore_patterns: vec![
                "test".to_string(),
                "spec".to_string(),
                "mock".to_string(),
                "fixture".to_string(),
                "generated".to_string(),
                "__pycache__".to_string(),
                "node_modules".to_string(),
            ],
            include_metrics_detail: true,
        }
    }
}

/// Detects "Large Class" anti-patterns using a multi-metric analysis approach.
///
/// This detector identifies classes, structs, or other primary data structures that have
/// grown too large, complex, or non-cohesive. It uses a combination of size, complexity,
/// and structural metrics to calculate a severity score, which helps distinguish between
/// moderately large classes and true "God Objects."
///
/// The analysis is language-aware, applying different thresholds for Rust, Python, and
/// JavaScript to account for their distinct coding idioms.
pub struct LargeClassesDetector {
    pub config: LargeClassConfig,
}

impl LargeClassesDetector {
    /// Creates a new `LargeClassesDetector` with the specified configuration.
    pub fn new(config: LargeClassConfig) -> Self {
        Self { config }
    }

    /// Creates a new `LargeClassesDetector` with a default configuration.
    pub fn with_default_config() -> Self {
        Self::new(LargeClassConfig::default())
    }

    /// Extracts `ClassMetrics` from a parsed file by dispatching to the appropriate
    /// language-specific extraction method.
    fn extract_class_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        match parsed_file.language {
            SourceLanguage::Rust => self.extract_rust_metrics(parsed_file),
            SourceLanguage::Python => self.extract_python_metrics(parsed_file),
            SourceLanguage::JavaScript => self.extract_javascript_metrics(parsed_file),
        }
    }

    /// Extracts metrics for Rust structs and their associated `impl` blocks.
    ///
    /// This function first identifies all `struct` definitions in the file. For each struct,
    /// it then searches for corresponding `impl` blocks to calculate method-related metrics
    /// like method count, complexity, and cohesion.
    fn extract_rust_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        let mut metrics = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        // First, collect all structs
        let struct_query = Query::new(&language, RUST_STRUCT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&struct_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                let struct_node = name_node.parent().unwrap_or(name_node);
                
                if let Ok(name) = name_node.utf8_text(source) {
                    let field_count = self.count_rust_struct_fields(&struct_node, source)?;
                    let logical_loc = self.calculate_logical_loc(&struct_node, source);
                    let is_exported = self.is_rust_exported(&struct_node, source);
                    let code_snippet = self.extract_code_snippet(&struct_node, source, 5);

                    // Look for corresponding impl blocks
                    let (method_count, complexity) = self.find_rust_impl_metrics(name, tree, source)?;
                    let lcom_score = self.calculate_rust_lcom(name, tree, source)?;
                    let coupling_count = self.count_rust_coupling(&struct_node, source)?;

                    metrics.push(ClassMetrics {
                        name: name.to_string(),
                        file_path: parsed_file.path.to_string_lossy().to_string(),
                        start_line: (name_node.start_position().row + 1) as u32,
                        end_line: (struct_node.end_position().row + 1) as u32,
                        logical_loc,
                        method_count,
                        field_count,
                        cyclomatic_complexity: complexity,
                        cognitive_complexity: complexity, // Simplified for now
                        lcom_score,
                        coupling_count,
                        is_exported,
                        code_snippet,
                    });
                }
            }
        }

        Ok(metrics)
    }

    /// Extracts metrics for Python classes.
    ///
    /// It uses `tree-sitter` queries to find class definitions and then analyzes their
    /// bodies to count methods, fields, and calculate complexity and cohesion scores.
    fn extract_python_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        let mut metrics = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        let class_query = Query::new(&language, PYTHON_CLASS_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&class_query, tree.root_node(), source) {
            if let (Some(name_capture), Some(body_capture)) = 
                (mat.captures.first(), mat.captures.get(1)) {
                
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                let class_node = name_node.parent().unwrap_or(name_node);
                
                if let Ok(name) = name_node.utf8_text(source) {
                    let method_count = self.count_python_methods(&body_node, source)?;
                    let field_count = self.count_python_fields(&body_node, source)?;
                    let logical_loc = self.calculate_logical_loc(&class_node, source);
                    let complexity = self.calculate_python_complexity(&body_node, source)?;
                    let lcom_score = self.calculate_python_lcom(&body_node, source)?;
                    let coupling_count = self.count_python_coupling(&class_node, source)?;
                    let is_exported = !name.starts_with('_');
                    let code_snippet = self.extract_code_snippet(&class_node, source, 5);

                    metrics.push(ClassMetrics {
                        name: name.to_string(),
                        file_path: parsed_file.path.to_string_lossy().to_string(),
                        start_line: (name_node.start_position().row + 1) as u32,
                        end_line: (class_node.end_position().row + 1) as u32,
                        logical_loc,
                        method_count,
                        field_count,
                        cyclomatic_complexity: complexity,
                        cognitive_complexity: complexity,
                        lcom_score,
                        coupling_count,
                        is_exported,
                        code_snippet,
                    });
                }
            }
        }

        Ok(metrics)
    }

    /// Extracts metrics for JavaScript classes.
    ///
    /// This function is designed to handle modern JavaScript (ES6 classes) and can be
    /// extended to recognize other patterns (e.g., constructor functions). It gathers
    /// metrics on methods, properties, and dependencies.
    fn extract_javascript_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        let mut metrics = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        let class_query = Query::new(&language, JAVASCRIPT_CLASS_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&class_query, tree.root_node(), source) {
            if let (Some(name_capture), Some(body_capture)) = 
                (mat.captures.first(), mat.captures.get(1)) {
                
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                let class_node = name_node.parent().unwrap_or(name_node);
                
                if let Ok(name) = name_node.utf8_text(source) {
                    let method_count = self.count_javascript_methods(&body_node, source)?;
                    let field_count = self.count_javascript_fields(&body_node, source)?;
                    let logical_loc = self.calculate_logical_loc(&class_node, source);
                    let complexity = self.calculate_javascript_complexity(&body_node, source)?;
                    let lcom_score = self.calculate_javascript_lcom(&body_node, source)?;
                    let coupling_count = self.count_javascript_coupling(&class_node, source)?;
                    let is_exported = self.is_javascript_exported(&class_node, source);
                    let code_snippet = self.extract_code_snippet(&class_node, source, 5);

                    metrics.push(ClassMetrics {
                        name: name.to_string(),
                        file_path: parsed_file.path.to_string_lossy().to_string(),
                        start_line: (name_node.start_position().row + 1) as u32,
                        end_line: (class_node.end_position().row + 1) as u32,
                        logical_loc,
                        method_count,
                        field_count,
                        cyclomatic_complexity: complexity,
                        cognitive_complexity: complexity,
                        lcom_score,
                        coupling_count,
                        is_exported,
                        code_snippet,
                    });
                }
            }
        }

        Ok(metrics)
    }

    /// Calculates a severity score (0-100) based on how much a class exceeds its thresholds.
    ///
    /// This method weighs different categories of metrics (size, complexity, structure)
    /// to produce a holistic score. A higher score indicates a more severe anti-pattern.
    /// It also applies a bonus penalty if multiple thresholds are violated, which is a
    /// strong indicator of a "God Object."
    ///
    /// # Arguments
    ///
    /// * `metrics` - The collected metrics for the class.
    /// * `language` - The source language, used to select the correct thresholds.
    ///
    /// # Returns
    ///
    /// A severity score from 0 to 100.
    pub fn calculate_severity_score(&self, metrics: &ClassMetrics, language: SourceLanguage) -> u32 {
        let default_thresholds = LanguageThresholds::rust();
        let thresholds = self.config.language_thresholds.get(&language)
            .unwrap_or(&default_thresholds);

        let mut score = 0u32;
        let mut violations = 0u32;

        // Size metrics (40% weight)
        if metrics.logical_loc > thresholds.max_logical_loc {
            let excess = (metrics.logical_loc as f64 / thresholds.max_logical_loc as f64 - 1.0) * 100.0;
            score += (excess.min(25.0) as u32).min(15);
            violations += 1;
        }

        if metrics.method_count > thresholds.max_methods {
            let excess = (metrics.method_count as f64 / thresholds.max_methods as f64 - 1.0) * 100.0;
            score += (excess.min(25.0) as u32).min(15);
            violations += 1;
        }

        if metrics.field_count > thresholds.max_fields {
            let excess = (metrics.field_count as f64 / thresholds.max_fields as f64 - 1.0) * 100.0;
            score += (excess.min(25.0) as u32).min(10);
            violations += 1;
        }

        // Complexity metrics (35% weight)
        if metrics.cyclomatic_complexity > thresholds.max_cyclomatic_complexity {
            let excess = (metrics.cyclomatic_complexity as f64 / thresholds.max_cyclomatic_complexity as f64 - 1.0) * 100.0;
            score += (excess.min(30.0) as u32).min(20);
            violations += 1;
        }

        // Structural metrics (25% weight)
        if metrics.lcom_score > thresholds.max_lcom_score {
            let excess = (metrics.lcom_score / thresholds.max_lcom_score - 1.0) * 100.0;
            score += (excess.min(20.0) as u32).min(15);
            violations += 1;
        }

        if metrics.coupling_count > thresholds.max_coupling {
            let excess = (metrics.coupling_count as f64 / thresholds.max_coupling as f64 - 1.0) * 100.0;
            score += (excess.min(15.0) as u32).min(10);
            violations += 1;
        }

        // Bonus for multiple violations (indicates God Object)
        if violations >= 3 {
            score += 10;
        }
        if violations >= 4 {
            score += 15;
        }

        score.min(100)
    }

    /// Converts a numeric severity score into a human-readable label.
    fn score_to_severity(&self, score: u32) -> &'static str {
        match score {
            0..=25 => "Info",
            26..=50 => "Low", 
            51..=75 => "Medium",
            76..=90 => "High",
            91..=100 => "Critical",
            _ => "Critical",
        }
    }

    /// Generates a detailed, human-readable description for a detected issue.
    ///
    /// If `include_metrics_detail` is enabled in the configuration, this function provides
    /// a full breakdown of the metrics, their values, and the thresholds they exceeded.
    /// It also includes targeted refactoring suggestions based on the specific violations.
    pub fn generate_description(&self, metrics: &ClassMetrics, score: u32, language: SourceLanguage) -> String {
        let default_thresholds = LanguageThresholds::rust();
        let thresholds = self.config.language_thresholds.get(&language)
            .unwrap_or(&default_thresholds);

        let mut description = format!(
            "Large class detected: '{}' has grown beyond recommended thresholds (severity: {}%)",
            metrics.name, score
        );

        if self.config.include_metrics_detail {
            description.push_str(&format!(
                "\n\nMetrics breakdown:\n• Lines of code: {} (threshold: {})\n• Methods: {} (threshold: {})\n• Fields: {} (threshold: {})\n• Cyclomatic complexity: {} (threshold: {})\n• LCOM score: {:.2} (threshold: {:.2})\n• Coupling: {} (threshold: {})",
                metrics.logical_loc, thresholds.max_logical_loc,
                metrics.method_count, thresholds.max_methods,
                metrics.field_count, thresholds.max_fields,
                metrics.cyclomatic_complexity, thresholds.max_cyclomatic_complexity,
                metrics.lcom_score, thresholds.max_lcom_score,
                metrics.coupling_count, thresholds.max_coupling
            ));

            // Add refactoring suggestions based on primary violations
            description.push_str("\n\nSuggested refactoring:");
            if metrics.lcom_score > thresholds.max_lcom_score {
                description.push_str("\n• Extract Class: Low cohesion suggests multiple responsibilities");
            }
            if metrics.method_count > thresholds.max_methods {
                description.push_str("\n• Extract Superclass: Consider inheritance hierarchy");
            }
            if metrics.coupling_count > thresholds.max_coupling {
                description.push_str("\n• Dependency Injection: Reduce tight coupling");
            }
        }

        description
    }

    /// Checks if a file should be ignored based on the configured patterns.
    fn should_ignore_file(&self, file_path: &str) -> bool {
        self.config.ignore_patterns.iter().any(|pattern| {
            file_path.contains(pattern)
        })
    }

    // Helper methods for metric calculations

    /// Calculates the Logical Lines of Code (LLOC) for a given AST node.
    ///
    /// This method counts non-empty lines that are not comments, providing a more
    /// accurate measure of code size than a simple line count.
    pub fn calculate_logical_loc(&self, node: &tree_sitter::Node, source: &[u8]) -> u32 {
        if let Ok(text) = node.utf8_text(source) {
            text.lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && 
                    !trimmed.starts_with("//") && 
                    !trimmed.starts_with('#') &&
                    !trimmed.starts_with("/*") &&
                    !trimmed.starts_with('*') &&
                    !trimmed.starts_with("*/")
                })
                .count() as u32
        } else {
            0
        }
    }

    /// Extracts a code snippet from a given AST node.
    ///
    /// This is used to provide context in the final architectural issue report. It limits
    /// the snippet to a maximum number of lines to keep the report concise.
    pub fn extract_code_snippet(&self, node: &tree_sitter::Node, source: &[u8], max_lines: usize) -> String {
        if let Ok(text) = node.utf8_text(source) {
            let lines: Vec<&str> = text.lines().take(max_lines).collect();
            let mut result = lines.join("\n");
            if text.lines().count() > max_lines {
                result.push_str("\n...");
            }
            result
        } else {
            "Unable to extract snippet".to_string()
        }
    }

    // Rust-specific helper methods
    /// Counts the number of fields in a Rust struct definition.
    fn count_rust_struct_fields(&self, struct_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        let language = struct_node.language();
        let field_query = Query::new(&language, RUST_FIELD_COUNT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        let field_count = cursor.matches(&field_query, *struct_node, source).count() as u32;
        
        Ok(field_count)
    }

    /// Finds all `impl` blocks for a given struct and aggregates method-related metrics.
    fn find_rust_impl_metrics(&self, struct_name: &str, tree: &tree_sitter::Tree, source: &[u8]) -> Result<(u32, u32), AnalysisError> {
        let language = tree.language();
        let impl_query = Query::new(&language, RUST_IMPL_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let function_query = Query::new(&language, RUST_FUNCTION_COUNT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut total_methods = 0u32;
        let mut total_complexity = 0u32;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&impl_query, tree.root_node(), source) {
            if let (Some(type_capture), Some(body_capture)) = 
                (mat.captures.first(), mat.captures.get(1)) {
                
                let type_node = type_capture.node;
                if let Ok(type_name) = type_node.utf8_text(source) {
                    if type_name == struct_name {
                        let body_node = body_capture.node;
                        
                        // Count methods in this impl block
                        let mut method_cursor = QueryCursor::new();
                        let method_count = method_cursor
                            .matches(&function_query, body_node, source)
                            .count() as u32;
                        
                        total_methods += method_count;
                        
                        // Calculate complexity for each method
                        for method_match in method_cursor.matches(&function_query, body_node, source) {
                            if let Some(method_node) = method_match.captures.first() {
                                total_complexity += self.calculate_method_complexity(&method_node.node, source)?;
                            }
                        }
                    }
                }
            }
        }
        
        Ok((total_methods, total_complexity))
    }

    /// Calculates a simplified Lack of Cohesion in Methods (LCOM) score for a Rust struct.
    ///
    /// This implementation approximates LCOM by measuring the average number of fields
    /// accessed by each method. A lower average suggests higher cohesion.
    fn calculate_rust_lcom(&self, struct_name: &str, tree: &tree_sitter::Tree, source: &[u8]) -> Result<f64, AnalysisError> {
        // Simplified LCOM calculation based on method-field relationships
        let language = tree.language();
        let impl_query = Query::new(&language, RUST_IMPL_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let function_query = Query::new(&language, RUST_FUNCTION_COUNT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut method_count = 0u32;
        let mut field_access_count = 0u32;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&impl_query, tree.root_node(), source) {
            if let (Some(type_capture), Some(body_capture)) = 
                (mat.captures.first(), mat.captures.get(1)) {
                
                let type_node = type_capture.node;
                if let Ok(type_name) = type_node.utf8_text(source) {
                    if type_name == struct_name {
                        let body_node = body_capture.node;
                        
                        let mut method_cursor = QueryCursor::new();
                        for method_match in method_cursor.matches(&function_query, body_node, source) {
                            if let Some(method_node) = method_match.captures.first() {
                                method_count += 1;
                                // Count field accesses in this method (simplified)
                                field_access_count += self.count_field_accesses(&method_node.node, source)?;
                            }
                        }
                    }
                }
            }
        }
        
        // Simplified LCOM calculation: higher values indicate lower cohesion
        if method_count == 0 {
            return Ok(0.0);
        }
        
        let avg_field_access = field_access_count as f64 / method_count as f64;
        // Normalize to 0-1 range where 1 = low cohesion
        let lcom_score = if avg_field_access > 3.0 { 
            0.2 // High cohesion
        } else if avg_field_access > 1.0 {
            0.5 // Medium cohesion
        } else {
            0.9 // Low cohesion
        };
        
        Ok(lcom_score)
    }

    /// Counts the number of `use` statements to approximate coupling for a Rust struct.
    fn count_rust_coupling(&self, node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        let language = node.language();
        let use_query = Query::new(&language, RUST_USE_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        let coupling_count = cursor.matches(&use_query, *node, source).count() as u32;
        
        Ok(coupling_count)
    }

    /// Checks if a Rust struct is marked as `pub`.
    fn is_rust_exported(&self, node: &tree_sitter::Node, source: &[u8]) -> bool {
        // Check for 'pub' keyword
        if let Some(parent) = node.parent() {
            if let Ok(text) = parent.utf8_text(source) {
                return text.starts_with("pub ");
            }
        }
        false
    }

    // Python-specific helper methods
    /// Counts the number of methods in a Python class.
    fn count_python_methods(&self, body_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        let language = body_node.language();
        let method_query = Query::new(&language, PYTHON_METHOD_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        let method_count = cursor.matches(&method_query, *body_node, source).count() as u32;
        
        Ok(method_count)
    }

    /// Counts the number of instance attributes (`self.field`) in a Python class.
    fn count_python_fields(&self, body_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        let language = body_node.language();
        let field_query = Query::new(&language, PYTHON_FIELD_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        let field_count = cursor.matches(&field_query, *body_node, source).count() as u32;
        
        Ok(field_count)
    }

    /// Calculates the cyclomatic complexity for a Python class body.
    fn calculate_python_complexity(&self, body_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        let language = body_node.language();
        let complexity_query = Query::new(&language, PYTHON_COMPLEXITY_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        // Count decision points (if, while, for, try, etc.) + 1 for base complexity
        let decision_points = cursor.matches(&complexity_query, *body_node, source).count() as u32;
        
        Ok(decision_points + 1)
    }

    /// Calculates a simplified LCOM score for a Python class.
    fn calculate_python_lcom(&self, body_node: &tree_sitter::Node, source: &[u8]) -> Result<f64, AnalysisError> {
        let language = body_node.language();
        let method_query = Query::new(&language, PYTHON_METHOD_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut method_count = 0u32;
        let mut field_access_count = 0u32;
        
        let mut cursor = QueryCursor::new();
        for method_match in cursor.matches(&method_query, *body_node, source) {
            if let Some(method_node) = method_match.captures.first() {
                method_count += 1;
                field_access_count += self.count_field_accesses(&method_node.node, source)?;
            }
        }
        
        // Simplified LCOM calculation
        if method_count == 0 {
            return Ok(0.0);
        }
        
        let avg_field_access = field_access_count as f64 / method_count as f64;
        let lcom_score = if avg_field_access > 2.0 { 
            0.3 // Good cohesion
        } else if avg_field_access > 1.0 {
            0.6 // Medium cohesion
        } else {
            0.9 // Poor cohesion
        };
        
        Ok(lcom_score)
    }

    /// Placeholder for counting dependencies in a Python class.
    fn count_python_coupling(&self, class_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        // Implementation for counting imports and external references
        Ok(0) // Placeholder
    }

    // JavaScript-specific helper methods
    /// Placeholder for counting methods in a JavaScript class.
    fn count_javascript_methods(&self, body_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        // Implementation for counting JavaScript methods
        Ok(0) // Placeholder
    }

    /// Placeholder for counting properties in a JavaScript class.
    fn count_javascript_fields(&self, body_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        // Implementation for counting JavaScript properties
        Ok(0) // Placeholder
    }

    /// Placeholder for calculating complexity in a JavaScript class.
    fn calculate_javascript_complexity(&self, body_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        // Implementation for calculating complexity
        Ok(0) // Placeholder
    }

    /// Placeholder for calculating LCOM in a JavaScript class.
    fn calculate_javascript_lcom(&self, body_node: &tree_sitter::Node, source: &[u8]) -> Result<f64, AnalysisError> {
        // Implementation for calculating LCOM score
        Ok(0.0) // Placeholder
    }

    /// Placeholder for counting dependencies in a JavaScript class.
    fn count_javascript_coupling(&self, class_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        // Implementation for counting imports and dependencies
        Ok(0) // Placeholder
    }

    /// Checks if a JavaScript class is exported.
    fn is_javascript_exported(&self, class_node: &tree_sitter::Node, source: &[u8]) -> bool {
        // Check for export keyword or module.exports
        if let Some(parent) = class_node.parent() {
            if let Ok(text) = parent.utf8_text(source) {
                return text.contains("export") || text.contains("module.exports");
            }
        }
        false
    }

    // Helper methods for complexity and field access calculations
    /// Calculates the cyclomatic complexity for a single method node.
    ///
    /// This helper function abstracts the complexity calculation by trying language-specific
    /// `tree-sitter` queries. It provides a fallback if no specific query matches.
    fn calculate_method_complexity(&self, method_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        // Simplified complexity calculation - count control flow statements
        let language = method_node.language();
        
        // Try each language's complexity query
        let queries = [
            RUST_COMPLEXITY_QUERY,
            PYTHON_COMPLEXITY_QUERY, 
            JAVASCRIPT_COMPLEXITY_QUERY,
        ];
        
        for query_str in &queries {
            if let Ok(complexity_query) = Query::new(&language, query_str) {
                let mut cursor = QueryCursor::new();
                let decision_points = cursor.matches(&complexity_query, *method_node, source).count() as u32;
                return Ok(decision_points + 1); // Base complexity of 1
            }
        }
        
        // Fallback: simple line count as complexity approximation
        if let Ok(text) = method_node.utf8_text(source) {
            let line_count = text.lines().count() as u32;
            Ok((line_count / 10).max(1)) // Rough complexity estimate
        } else {
            Ok(1)
        }
    }

    /// Counts field accesses (`self.` or `this.`) within a method.
    ///
    /// This is a key component of the simplified LCOM calculation. It uses `tree-sitter`
    /// queries for accuracy and falls back to simple text matching if needed.
    fn count_field_accesses(&self, method_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        // Simplified field access counting
        let language = method_node.language();
        
        // Try each language's field access query
        let queries = [
            RUST_FIELD_ACCESS_QUERY,
            PYTHON_FIELD_ACCESS_QUERY,
            JAVASCRIPT_FIELD_ACCESS_QUERY,
        ];
        
        for query_str in &queries {
            if let Ok(field_access_query) = Query::new(&language, query_str) {
                let mut cursor = QueryCursor::new();
                let field_accesses = cursor.matches(&field_access_query, *method_node, source).count() as u32;
                return Ok(field_accesses);
            }
        }
        
        // Fallback: simple text-based counting
        if let Ok(text) = method_node.utf8_text(source) {
            let self_count = text.matches("self.").count() as u32;
            let this_count = text.matches("this.").count() as u32;
            Ok(self_count + this_count)
        } else {
            Ok(0)
        }
    }
}

impl AnalysisDetector for LargeClassesDetector {
    fn get_detector_name(&self) -> &'static str {
        "LargeClassesDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(3),
            name: "Large Class".to_string(),
            description: "A class that has grown too large and violates the Single Responsibility Principle, including God Objects and Blob anti-patterns.".to_string(),
            category: "Object-Oriented Design".to_string(),
        }]
    }

    fn detect_issues(&self, parsed_file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!(
            "Running Large Classes detection on: {}",
            parsed_file.path.display()
        );

        // Check if file should be ignored
        let file_path = parsed_file.path.to_string_lossy();
        if self.should_ignore_file(&file_path) {
            debug!("Ignoring file based on patterns: {}", file_path);
            return Ok(vec![]);
        }

        let class_metrics = self.extract_class_metrics(parsed_file)?;
        let mut issues = Vec::new();

        for metrics in class_metrics {
            let severity_score = self.calculate_severity_score(&metrics, parsed_file.language);
            
            if severity_score >= self.config.min_severity_score {
                let severity = self.score_to_severity(severity_score);
                let description = self.generate_description(&metrics, severity_score, parsed_file.language);

                issues.push(ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: 0,
                    anti_pattern_type_id: 3, // Large Class
                    file_path: metrics.file_path,
                    start_line: Some(metrics.start_line as i32),
                    end_line: Some(metrics.end_line as i32),
                    severity: severity.to_string(),
                    description,
                    code_snippet: Some(metrics.code_snippet),
                    ai_explanation: None,
                });
            }
        }

        if issues.is_empty() {
            debug!(
                "No large class issues found in {}",
                parsed_file.path.display()
            );
        } else {
            info!(
                "Found {} large class issues in {}",
                issues.len(),
                parsed_file.path.display()
            );
        }

        Ok(issues)
    }
}

// Tree-sitter queries for different languages

const RUST_STRUCT_QUERY: &str = r#"
(struct_item
  name: (type_identifier) @name
  body: (field_declaration_list) @body
)
"#;

const PYTHON_CLASS_QUERY: &str = r#"
(class_definition
  name: (identifier) @name
  body: (block) @body
)
"#;

const JAVASCRIPT_CLASS_QUERY: &str = r#"
(class_declaration
  name: (identifier) @name
  body: (class_body) @body
)
"#;

// Additional Tree-sitter queries for metric calculations

const RUST_FIELD_COUNT_QUERY: &str = r#"
(field_declaration
  name: (field_identifier) @field
)
"#;

const RUST_FUNCTION_COUNT_QUERY: &str = r#"
(function_item
  name: (identifier) @name
)
"#;

const RUST_IMPL_QUERY: &str = r#"
(impl_item
  type: (type_identifier) @type
  body: (declaration_list) @body
)
"#;

const RUST_USE_QUERY: &str = r#"
(use_declaration) @use
"#;

const PYTHON_METHOD_QUERY: &str = r#"
(function_definition
  name: (identifier) @name
)
"#;

const PYTHON_FIELD_QUERY: &str = r#"
(expression_statement
  (assignment
    left: (attribute
      object: (identifier) @self
      attribute: (identifier) @field)
    (#eq? @self "self")
  )
)
"#;

const PYTHON_COMPLEXITY_QUERY: &str = r#"
[
  (if_statement)
  (while_statement) 
  (for_statement)
  (try_statement)
  (with_statement)
  (match_statement)
  (except_clause)
] @decision_point
"#;

const JAVASCRIPT_METHOD_QUERY: &str = r#"
(method_definition
  name: (property_identifier) @name
)
"#;

const JAVASCRIPT_FIELD_QUERY: &str = r#"
(field_definition
  property: (property_identifier) @field
)
"#;

const JAVASCRIPT_COMPLEXITY_QUERY: &str = r#"
[
  (if_statement)
  (while_statement)
  (for_statement)
  (for_in_statement)
  (for_of_statement)
  (try_statement)
  (catch_clause)
  (switch_statement)
  (conditional_expression)
] @decision_point
"#;

const RUST_COMPLEXITY_QUERY: &str = r#"
[
  (if_expression)
  (while_expression)
  (for_expression)
  (loop_expression)
  (match_expression)
  (match_arm)
] @decision_point
"#;

const RUST_FIELD_ACCESS_QUERY: &str = r#"
(field_expression
  base: (identifier) @base
  field: (field_identifier) @field
  (#eq? @base "self")
)
"#;

const PYTHON_FIELD_ACCESS_QUERY: &str = r#"
(attribute
  object: (identifier) @self
  attribute: (identifier) @field
  (#eq? @self "self")
)
"#;

const JAVASCRIPT_FIELD_ACCESS_QUERY: &str = r#"
(member_expression
  object: (this) @this
  property: (property_identifier) @field
)
"#;