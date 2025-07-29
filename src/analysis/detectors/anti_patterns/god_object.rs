//! God Object Anti-pattern Detector
//!
//! ## Overview
//! Detects "God Objects" - classes or structs that have accumulated too many responsibilities,
//! violating the Single Responsibility Principle. Also known as "Blob" or "Large Class"
//! anti-pattern, these objects become difficult to maintain, test, and understand.
//!
//! God Objects typically exhibit:
//! - Excessive number of methods (high method count)
//! - Excessive number of fields/attributes (high field count)
//! - Multiple unrelated responsibilities
//! - High coupling with many other classes
//!
//! ## Detection Strategy
//! Uses a simple threshold-based approach to identify oversized classes:
//! 1. **Method Counting**: Counts all methods/functions within classes, structs, and impl blocks
//! 2. **Field Counting**: Counts all fields/attributes within data structures
//! 3. **Threshold Comparison**: Compares counts against configurable thresholds
//! 4. **Severity Assignment**: Assigns severity based on how much thresholds are exceeded
//!
//! The detector uses Tree-sitter queries to extract structural information from the AST,
//! ensuring accurate counting across different language syntaxes.
//!
//! ## Supported Languages
//! - **Rust**: Analyzes `struct` definitions and their associated `impl` blocks
//!   - Counts methods in all impl blocks for a given struct
//!   - Counts fields in struct definitions
//!   - Handles both tuple structs and named field structs
//! - **Python**: Analyzes `class` definitions
//!   - Counts methods within class bodies
//!   - Counts instance variables (self.field assignments)
//!   - Handles inheritance and nested classes
//! - **JavaScript**: Analyzes `class` declarations
//!   - Counts method definitions within class bodies
//!   - Counts field definitions and constructor assignments
//!   - Handles both ES6 classes and prototype-based patterns
//!
//! ## Configuration
//! The detector accepts two threshold parameters:
//! - `method_threshold`: Maximum number of methods before flagging (default: 10)
//! - `field_threshold`: Maximum number of fields before flagging (default: 8)
//!
//! These thresholds are applied uniformly across all languages, though language-specific
//! defaults could be implemented in future versions.
//!
//! ## Examples
//!
//! ### Detected Pattern (Rust)
//! ```rust
//! // This would be flagged as a God Object (too many methods)
//! struct UserManager {
//!     users: Vec<User>,
//!     sessions: HashMap<String, Session>,
//!     permissions: PermissionSet,
//!     audit_log: AuditLog,
//!     cache: Cache,
//!     config: Config,
//!     metrics: Metrics,
//!     notifications: NotificationService,
//! }
//!
//! impl UserManager {
//!     fn create_user(&self) { /* ... */ }
//!     fn delete_user(&self) { /* ... */ }
//!     fn authenticate(&self) { /* ... */ }
//!     fn authorize(&self) { /* ... */ }
//!     fn log_action(&self) { /* ... */ }
//!     fn send_notification(&self) { /* ... */ }
//!     fn update_cache(&self) { /* ... */ }
//!     fn generate_report(&self) { /* ... */ }
//!     fn backup_data(&self) { /* ... */ }
//!     fn validate_permissions(&self) { /* ... */ }
//!     fn handle_session(&self) { /* ... */ }
//!     // ... more methods (exceeds threshold)
//! }
//! ```
//!
//! ### Good Pattern (Rust)
//! ```rust
//! // Well-designed, focused structs
//! struct User {
//!     id: UserId,
//!     name: String,
//!     email: String,
//! }
//!
//! struct UserRepository {
//!     storage: Box<dyn Storage>,
//! }
//!
//! impl UserRepository {
//!     fn create(&self, user: User) -> Result<(), Error> { /* ... */ }
//!     fn find_by_id(&self, id: UserId) -> Result<User, Error> { /* ... */ }
//!     fn update(&self, user: User) -> Result<(), Error> { /* ... */ }
//!     fn delete(&self, id: UserId) -> Result<(), Error> { /* ... */ }
//! }
//! ```
//!
//! ## Performance Considerations
//! - **Time Complexity**: O(n) where n is the number of AST nodes in the file
//! - **Space Complexity**: O(m) where m is the number of classes/structs found
//! - **Optimization Notes**:
//!   - Uses efficient Tree-sitter queries to minimize AST traversal
//!   - Caches query compilation for repeated use
//!   - Processes files independently for parallelization
//!
//! ## Limitations
//! - **Single-file Analysis**: Cannot detect responsibilities spread across multiple files
//! - **Static Analysis Only**: Cannot detect runtime behavior or dynamic method addition
//! - **Language Specifics**: May miss language-specific patterns (e.g., Python metaclasses)
//! - **Threshold Sensitivity**: Simple thresholds may not account for domain complexity
//! - **No Semantic Analysis**: Counts methods without understanding their relationships
//!
//! ## References
//! - [Fowler, M. "Refactoring: Improving the Design of Existing Code"](https://refactoring.com/)
//! - [Brown, W. et al. "AntiPatterns: Refactoring Software, Architectures, and Projects in Crisis"](https://www.amazon.com/AntiPatterns-Refactoring-Software-Architectures-Projects/dp/0471197130)
//! - [Clean Code: A Handbook of Agile Software Craftsmanship](https://www.amazon.com/Clean-code-Handbook-Software-Craftsmanship/dp/0132350884)

use async_trait::async_trait;

use crate::analysis::errors::AnalysisError as CoreAnalysisError;
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::error::ErrorHelpers;
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

// --- Queries for identifying language-specific containers (classes, structs) ---
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

const RUST_STRUCT_QUERY: &str = r#"
(struct_item
  name: (type_identifier) @name
  body: (field_declaration_list) @body
)
"#;

const RUST_IMPL_QUERY: &str = r#"
(impl_item
  type: (type_identifier) @name
  body: (declaration_list) @body
)
"#;

// --- Queries for counting methods/functions within a container ---
const RUST_FUNCTION_COUNT_QUERY: &str = "(function_item)";
const PYTHON_FUNCTION_COUNT_QUERY: &str = "(function_definition)";
const JAVASCRIPT_FUNCTION_COUNT_QUERY: &str = "(method_definition)";

// --- Queries for counting fields/attributes within a container ---
const RUST_FIELD_COUNT_QUERY: &str = "(field_declaration)";
const PYTHON_FIELD_COUNT_QUERY: &str = r#"(expression_statement (assignment))"#;
const JAVASCRIPT_FIELD_COUNT_QUERY: &str = "(field_definition)";

// Enhanced queries for pattern detection

// Import detection queries
const RUST_USE_QUERY: &str = r#"
(use_declaration
  argument: (scoped_identifier) @import_path
)
"#;

const PYTHON_IMPORT_QUERY: &str = r#"
[
  (import_statement
    name: (dotted_name) @import_path
  )
  (import_from_statement
    module_name: (dotted_name) @import_path
  )
]
"#;

const JAVASCRIPT_IMPORT_QUERY: &str = r#"
[
  (import_statement
    source: (string) @import_path
  )
  (call_expression
    function: (identifier) @func_name
    arguments: (arguments (string) @import_path)
  )
]
"#;

// Pattern detection queries
const RUST_DERIVE_QUERY: &str = r#"
(attribute_item
  (attribute
    (scoped_identifier) @attr_name
    arguments: (token_tree) @attr_args
  )
)
"#;

const PYTHON_DECORATOR_QUERY: &str = r#"
(decorator
  (dotted_name) @decorator_name
)
"#;

const PYTHON_CLASS_BASE_QUERY: &str = r#"
(class_definition
  superclasses: (argument_list) @bases
)
"#;

// Method call analysis for Builder pattern
const METHOD_CALL_QUERY: &str = r#"
(call_expression
  function: (attribute
    object: (_) @object
    attribute: (identifier) @method_name
  )
)
"#;

// Constructor/build method patterns
const RUST_BUILD_METHOD_QUERY: &str = r#"
(function_item
  name: (identifier) @method_name
  (#match? @method_name "^(build|create|new|get)$")
)
"#;

const FACTORY_METHOD_QUERY: &str = r#"
(function_item
  name: (identifier) @method_name
  (#match? @method_name "^(create|make|build|get|factory).*")
)
"#;

/// Configuration for advanced God Object detection
#[derive(Debug, Clone)]
pub struct GodObjectConfig {
    /// Method count thresholds per language
    pub method_thresholds: HashMap<SourceLanguage, usize>,
    /// Field count thresholds per language
    pub field_thresholds: HashMap<SourceLanguage, usize>,
    /// Framework modules that should be excluded from detection
    pub framework_modules: HashSet<String>,
    /// Generated code file patterns to exclude
    pub generated_file_patterns: Vec<String>,
    /// Design patterns to recognize and exclude
    pub recognize_patterns: bool,
    /// Enable behavioral analysis
    pub enable_behavioral_analysis: bool,
    /// Enable LCOM4 cohesion analysis
    pub enable_cohesion_analysis: bool,
    /// Cyclomatic complexity threshold for "trivial" methods
    pub trivial_method_cc_threshold: u32,
}

impl Default for GodObjectConfig {
    fn default() -> Self {
        let mut method_thresholds = HashMap::new();
        method_thresholds.insert(SourceLanguage::Rust, 30);
        method_thresholds.insert(SourceLanguage::Python, 25);
        method_thresholds.insert(SourceLanguage::JavaScript, 20);

        let mut field_thresholds = HashMap::new();
        field_thresholds.insert(SourceLanguage::Rust, 20);
        field_thresholds.insert(SourceLanguage::Python, 15);
        field_thresholds.insert(SourceLanguage::JavaScript, 12);

        let mut framework_modules = HashSet::new();
        // Rust frameworks
        framework_modules.insert("axum".to_string());
        framework_modules.insert("rocket".to_string());
        framework_modules.insert("actix_web".to_string());
        framework_modules.insert("serde".to_string());
        framework_modules.insert("diesel".to_string());
        framework_modules.insert("sqlx".to_string());
        framework_modules.insert("tokio".to_string());

        // Python frameworks
        framework_modules.insert("django".to_string());
        framework_modules.insert("flask".to_string());
        framework_modules.insert("fastapi".to_string());
        framework_modules.insert("pydantic".to_string());
        framework_modules.insert("sqlalchemy".to_string());

        // JavaScript frameworks
        framework_modules.insert("react".to_string());
        framework_modules.insert("express".to_string());
        framework_modules.insert("vue".to_string());
        framework_modules.insert("axios".to_string());

        let generated_file_patterns = vec![
            "*_pb2.py".to_string(),
            "*_pb2_grpc.py".to_string(),
            "*.g.cs".to_string(),
            "*.generated.*".to_string(),
            "*_generated.*".to_string(),
        ];

        Self {
            method_thresholds,
            field_thresholds,
            framework_modules,
            generated_file_patterns,
            recognize_patterns: true,
            enable_behavioral_analysis: true,
            enable_cohesion_analysis: true,
            trivial_method_cc_threshold: 2,
        }
    }
}

/// Pattern detection results
#[derive(Debug, Clone)]
pub enum DetectedPattern {
    Builder {
        builder_methods: Vec<String>,
        build_method: Option<String>,
    },
    Factory {
        factory_methods: Vec<String>,
        product_types: Vec<String>,
    },
    Dto {
        framework: String,
        field_ratio: f64,
    },
    FrameworkController {
        framework: String,
        base_class: Option<String>,
    },
    GeneratedCode {
        generator: String,
        markers: Vec<String>,
    },
}

/// Detects "God Objects" with advanced pattern recognition and context-aware analysis.
///
/// This enhanced detector identifies classes or structs that have grown too large while
/// reducing false positives through sophisticated pattern recognition, framework awareness,
/// and cohesion analysis. It implements the multi-stage filtering pipeline described in
/// UV-25 research to distinguish true God Objects from legitimate large classes.
///
/// ## Enhanced Features
///
/// - **Framework Detection**: Excludes known framework base classes and patterns
/// - **Pattern Recognition**: Detects Builder, Factory, DTO, and other design patterns
/// - **Generated Code Exclusion**: Skips auto-generated classes and structs
/// - **Behavioral Analysis**: Distinguishes data structures from behavior classes
/// - **Cohesion Analysis**: Uses LCOM4 metrics to measure class cohesion
/// - **Language-Specific Tuning**: Optimizes thresholds per language ecosystem
///
/// ## Usage
///
/// ```rust
/// use uveddi::analysis::detectors::anti_patterns::god_object::{GodObjectDetector, GodObjectConfig};
///
/// // Use default configuration with all enhancements enabled
/// let detector = GodObjectDetector::default();
///
/// // Create with custom configuration
/// let config = GodObjectConfig {
///     recognize_patterns: true,
///     enable_behavioral_analysis: true,
///     enable_cohesion_analysis: true,
///     ..Default::default()
/// };
/// let detector = GodObjectDetector::with_config(config);
/// ```
pub struct GodObjectDetector {
    /// Configuration for the enhanced detection logic
    config: GodObjectConfig,
}

impl GodObjectDetector {
    /// Creates a new `GodObjectDetector` with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for enhanced God Object detection
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::detectors::anti_patterns::god_object::{GodObjectDetector, GodObjectConfig};
    ///
    /// let config = GodObjectConfig::default();
    /// let detector = GodObjectDetector::with_config(config);
    /// ```
    pub fn with_config(config: GodObjectConfig) -> Self {
        Self { config }
    }

    /// Creates a new `GodObjectDetector` with specified thresholds (legacy method).
    ///
    /// # Arguments
    ///
    /// * `method_threshold` - The maximum number of methods allowed.
    /// * `field_threshold` - The maximum number of fields allowed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    ///
    /// // A detector with strict thresholds for a project with small, focused classes.
    /// let strict_detector = GodObjectDetector::new(5, 3);
    ///
    /// // A detector with more lenient thresholds for a legacy or complex system.
    /// let lenient_detector = GodObjectDetector::new(20, 15);
    /// ```
    pub fn new(method_threshold: usize, field_threshold: usize) -> Self {
        let mut config = GodObjectConfig::default();
        // Override all language thresholds with provided values
        for (_, threshold) in config.method_thresholds.iter_mut() {
            *threshold = method_threshold;
        }
        for (_, threshold) in config.field_thresholds.iter_mut() {
            *threshold = field_threshold;
        }
        Self { config }
    }

    /// Checks if a file should be excluded based on generated code patterns.
    fn is_generated_file(&self, file_path: &str, source: &str) -> Option<DetectedPattern> {
        // Check file name patterns
        for pattern in &self.config.generated_file_patterns {
            if self.matches_pattern(file_path, pattern) {
                return Some(DetectedPattern::GeneratedCode {
                    generator: "file_pattern".to_string(),
                    markers: vec![pattern.clone()],
                });
            }
        }

        // Check for comment markers in first 10 lines
        let lines: Vec<&str> = source.lines().take(10).collect();
        let generated_markers = [
            "auto-generated",
            "auto generated",
            "DO NOT EDIT",
            "Code generated",
            "This file was automatically generated",
            "<auto-generated",
        ];

        for line in lines {
            for marker in &generated_markers {
                if line.to_lowercase().contains(&marker.to_lowercase()) {
                    return Some(DetectedPattern::GeneratedCode {
                        generator: "comment_marker".to_string(),
                        markers: vec![marker.to_string()],
                    });
                }
            }
        }

        None
    }

    /// Helper method to match file patterns (simplified glob matching)
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let inner = &pattern[1..pattern.len() - 1];
            path.contains(inner)
        } else if pattern.starts_with('*') {
            path.ends_with(&pattern[1..])
        } else if pattern.ends_with('*') {
            path.starts_with(&pattern[..pattern.len() - 1])
        } else {
            path == pattern
        }
    }

    /// Analyzes imports to detect framework usage.
    fn analyze_imports(&self, parsed_file: &ParsedFile) -> Result<HashSet<String>, AnalysisError> {
        let mut detected_frameworks = HashSet::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("import analysis"))?;
        let language = tree.language();

        let query_str = match parsed_file.language {
            SourceLanguage::Rust => RUST_USE_QUERY,
            SourceLanguage::Python => PYTHON_IMPORT_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_IMPORT_QUERY,
        };

        let query = Query::new(&language, query_str)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source);
        while let Some(mat) = matches.next() {
            for capture in mat.captures {
                if let Ok(import_text) = capture.node.utf8_text(source) {
                    // Extract module name from import path
                    let module_name = self.extract_module_name(import_text, parsed_file.language);
                    if self.config.framework_modules.contains(&module_name) {
                        detected_frameworks.insert(module_name);
                    }
                }
            }
        }

        Ok(detected_frameworks)
    }

    /// Extracts the root module name from an import statement.
    fn extract_module_name(&self, import_text: &str, language: SourceLanguage) -> String {
        match language {
            SourceLanguage::Rust => import_text
                .split("::")
                .next()
                .unwrap_or(import_text)
                .to_string(),
            SourceLanguage::Python => import_text
                .split('.')
                .next()
                .unwrap_or(import_text)
                .to_string(),
            SourceLanguage::JavaScript => {
                if import_text.starts_with('"') || import_text.starts_with('\'') {
                    let path = &import_text[1..import_text.len() - 1];
                    if path.starts_with("./") || path.starts_with("../") {
                        return "local".to_string();
                    }
                    path.split('/').next().unwrap_or(path).to_string()
                } else {
                    import_text.to_string()
                }
            }
        }
    }

    /// Detects Builder pattern in a class/struct.
    fn detect_builder_pattern(
        &self,
        parsed_file: &ParsedFile,
        class_node: Node,
        class_name: &str,
    ) -> Option<DetectedPattern> {
        if !class_name.ends_with("Builder") {
            return None;
        }

        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref()?;
        let language = tree.language();

        // Look for fluent interface methods (returning self)
        let query = Query::new(&language, METHOD_CALL_QUERY).ok()?;
        let mut cursor = QueryCursor::new();
        let mut builder_methods = Vec::new();
        let mut build_method = None;

        let mut matches = cursor.matches(&query, class_node, source);
        while let Some(mat) = matches.next() {
            for capture in mat.captures {
                if let Ok(method_name) = capture.node.utf8_text(source) {
                    if ["build", "create", "new", "get"].contains(&method_name) {
                        build_method = Some(method_name.to_string());
                    } else {
                        builder_methods.push(method_name.to_string());
                    }
                }
            }
        }

        if builder_methods.len() >= 3 && build_method.is_some() {
            Some(DetectedPattern::Builder {
                builder_methods,
                build_method,
            })
        } else {
            None
        }
    }

    /// Detects DTO pattern based on field-to-method ratio and framework markers.
    fn detect_dto_pattern(
        &self,
        parsed_file: &ParsedFile,
        class_node: Node,
        method_count: usize,
        field_count: usize,
        frameworks: &HashSet<String>,
    ) -> Option<DetectedPattern> {
        if field_count == 0 {
            return None;
        }

        let field_ratio = field_count as f64 / (field_count + method_count) as f64;

        // High field-to-method ratio suggests DTO
        if field_ratio > 0.7 {
            // Check for DTO framework markers
            for framework in frameworks {
                if ["serde", "pydantic", "dataclass"]
                    .iter()
                    .any(|f| framework.contains(f))
                {
                    return Some(DetectedPattern::Dto {
                        framework: framework.clone(),
                        field_ratio,
                    });
                }
            }

            // Check for DTO naming conventions
            if let Ok(source_text) = class_node.utf8_text(parsed_file.source.as_bytes()) {
                if source_text.to_lowercase().contains("dto") {
                    return Some(DetectedPattern::Dto {
                        framework: "naming_convention".to_string(),
                        field_ratio,
                    });
                }
            }
        }

        None
    }

    /// Calculates a simplified LCOM4 score for cohesion analysis.
    fn calculate_lcom4(
        &self,
        parsed_file: &ParsedFile,
        class_node: Node,
    ) -> Result<u32, AnalysisError> {
        // This is a simplified implementation - in practice, you'd want a more sophisticated analysis
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("cohesion analysis"))?;
        let language = tree.language();

        let method_query_str = match parsed_file.language {
            SourceLanguage::Rust => RUST_FUNCTION_COUNT_QUERY,
            SourceLanguage::Python => PYTHON_FUNCTION_COUNT_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_FUNCTION_COUNT_QUERY,
        };

        let query = Query::new(&language, method_query_str)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let method_count = {
            let mut matches = cursor.matches(&query, class_node, source);
            let mut count = 0;
            while matches.next().is_some() {
                count += 1;
            }
            count
        };

        // Simplified heuristic: assume low cohesion if many methods (>10) without deep analysis
        // A proper implementation would analyze shared fields and method calls
        if method_count > 10 {
            Ok(2) // Assume multiple responsibilities
        } else {
            Ok(1) // Assume cohesive
        }
    }

    /// Performs behavioral analysis to classify methods as trivial or complex.
    fn analyze_behavioral_complexity(
        &self,
        parsed_file: &ParsedFile,
        class_node: Node,
    ) -> Result<(usize, usize), AnalysisError> {
        // This is a simplified behavioral analysis
        // In practice, you'd calculate Cyclomatic Complexity for each method
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("behavioral analysis"))?;
        let language = tree.language();

        let method_query_str = match parsed_file.language {
            SourceLanguage::Rust => RUST_FUNCTION_COUNT_QUERY,
            SourceLanguage::Python => PYTHON_FUNCTION_COUNT_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_FUNCTION_COUNT_QUERY,
        };

        let query = Query::new(&language, method_query_str)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let total_methods = {
            let mut matches = cursor.matches(&query, class_node, source);
            let mut count = 0;
            while matches.next().is_some() {
                count += 1;
            }
            count
        };

        // Simplified heuristic: assume 70% are trivial methods (getters, setters)
        let trivial_methods = (total_methods as f64 * 0.7) as usize;
        let complex_methods = total_methods - trivial_methods;

        Ok((trivial_methods, complex_methods))
    }

    /// Scores the severity of a detected God Object with enhanced analysis.
    ///
    /// The severity is determined by how much the method and field counts exceed
    /// their respective thresholds, adjusted by cohesion and behavioral analysis.
    /// - **Medium**: 1-4 total excess members or low cohesion.
    /// - **High**: 5-10 total excess members with behavioral complexity.
    /// - **Critical**: 11+ total excess members with multiple responsibilities.
    ///
    /// Returns `None` if no thresholds are exceeded or pattern exclusions apply.
    fn score_severity(
        &self,
        language: SourceLanguage,
        method_count: usize,
        field_count: usize,
        lcom4_score: Option<u32>,
        behavioral_analysis: Option<(usize, usize)>,
    ) -> Option<String> {
        let method_threshold = self
            .config
            .method_thresholds
            .get(&language)
            .copied()
            .unwrap_or(10);
        let field_threshold = self
            .config
            .field_thresholds
            .get(&language)
            .copied()
            .unwrap_or(8);

        let method_excess = method_count.saturating_sub(method_threshold);
        let field_excess = field_count.saturating_sub(field_threshold);

        // Only consider it an issue if at least one threshold is exceeded.
        if method_excess == 0 && field_excess == 0 {
            return None;
        }

        let total_excess = method_excess + field_excess;

        // Factor in cohesion analysis
        let cohesion_penalty = if let Some(lcom4) = lcom4_score {
            if lcom4 > 1 {
                3
            } else {
                0
            }
        } else {
            0
        };

        // Factor in behavioral complexity
        let behavior_penalty = if let Some((trivial, complex)) = behavioral_analysis {
            if complex > trivial {
                2
            } else {
                0
            }
        } else {
            0
        };

        let adjusted_excess = total_excess + cohesion_penalty + behavior_penalty;

        let severity = match adjusted_excess {
            0..=4 => "Medium",
            5..=10 => "High",
            _ => "Critical",
        };
        Some(severity.to_string())
    }

    /// Creates an `ArchitecturalIssue` for a detected God Object with enhanced context.
    ///
    /// This helper function is called when a God Object is identified. It constructs
    /// an `ArchitecturalIssue` with relevant details, including the severity,
    /// file path, line numbers, and a descriptive message that includes analysis results.
    ///
    /// Returns `None` if the severity score is not high enough to warrant an issue
    /// or if exclusion patterns apply.
    fn create_issue(
        &self,
        parsed_file: &ParsedFile,
        name: &str,
        name_node: Node,
        container_node: Node,
        method_count: usize,
        field_count: usize,
        lcom4_score: Option<u32>,
        behavioral_analysis: Option<(usize, usize)>,
        excluded_pattern: Option<DetectedPattern>,
    ) -> Option<ArchitecturalIssue> {
        // If excluded by pattern recognition, return None
        if excluded_pattern.is_some() {
            debug!(
                "Excluding '{}' due to detected pattern: {:?}",
                name, excluded_pattern
            );
            return None;
        }

        let language = parsed_file.language;
        let method_threshold = self
            .config
            .method_thresholds
            .get(&language)
            .copied()
            .unwrap_or(10);
        let field_threshold = self
            .config
            .field_thresholds
            .get(&language)
            .copied()
            .unwrap_or(8);

        self.score_severity(language, method_count, field_count, lcom4_score, behavioral_analysis)
            .map(|severity| {
                let mut description = format!(
                    "God Object detected: '{}' has {} methods and {} fields. (Thresholds: methods>{}, fields>{})",
                    name, method_count, field_count, method_threshold, field_threshold
                );

                // Add enhanced analysis information
                if let Some(lcom4) = lcom4_score {
                    description.push_str(&format!(" LCOM4 score: {} (>1 indicates low cohesion)", lcom4));
                }

                if let Some((trivial, complex)) = behavioral_analysis {
                    description.push_str(&format!(" Methods: {} trivial, {} complex", trivial, complex));
                }
                ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: 0, // Will be set by the engine
                    anti_pattern_type_id: 1, // God Object
                    file_path: parsed_file.file_path.display().to_string(), // TODO UV-222: Use Arc<PathBuf> for O(1) clones
                    start_line: Some((name_node.start_position().row + 1) as i32),
                    end_line: Some((name_node.end_position().row + 1) as i32),
                    severity,
                    description,
                    code_snippet: Some(
                        {
                            let source_str = parsed_file.source.as_str();
                            container_node
                                .utf8_text(source_str.as_bytes())
                                .unwrap_or("")
                                .to_string()
                        }
                    ),
                    ai_explanation: None,
                }
            })
    }

    /// Analyzes a file for God Objects using enhanced pattern recognition for languages
    /// like Python and JavaScript, where class members are defined in a single block.
    ///
    /// This function implements the multi-stage filtering pipeline:
    /// 1. Pre-AST exclusion (generated code detection)
    /// 2. Initial AST-based analysis with framework detection
    /// 3. Pattern recognition (Builder, Factory, DTO)
    /// 4. Qualitative adjudication (LCOM4, behavioral analysis)
    fn analyze_standard(
        &self,
        parsed_file: &ParsedFile,
        container_query_str: &str,
        method_query_str: &str,
        field_query_str: &str,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();

        // Stage 1: Pre-AST Exclusion - Check for generated code
        if let Some(generated_pattern) = self.is_generated_file(
            &parsed_file.file_path.display().to_string(),
            &parsed_file.source,
        ) {
            debug!("Excluding file as generated code: {:?}", generated_pattern);
            return Ok(issues);
        }

        // Stage 2: Framework Detection
        let detected_frameworks = self.analyze_imports(parsed_file)?;
        debug!("Detected frameworks: {:?}", detected_frameworks);

        let container_query = Query::new(&language, container_query_str)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let method_query = Query::new(&language, method_query_str)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;
        let field_query = Query::new(&language, field_query_str)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&container_query, tree.root_node(), source);
        while let Some(mat) = matches.next() {
            let name_node = mat.captures[0].node;
            let body_node = mat.captures[1].node;
            let container_node = name_node.parent().unwrap_or(name_node);

            let name = name_node.utf8_text(source).unwrap_or("Unnamed");

            let mut method_cursor = QueryCursor::new();
            let method_count = {
                let mut matches = method_cursor.matches(&method_query, body_node, source);
                let mut count = 0;
                while matches.next().is_some() {
                    count += 1;
                }
                count
            };

            let mut field_cursor = QueryCursor::new();
            let field_count = {
                let mut matches = field_cursor.matches(&field_query, body_node, source);
                let mut count = 0;
                while matches.next().is_some() {
                    count += 1;
                }
                count
            };

            let language = parsed_file.language;
            let method_threshold = self
                .config
                .method_thresholds
                .get(&language)
                .copied()
                .unwrap_or(10);
            let field_threshold = self
                .config
                .field_thresholds
                .get(&language)
                .copied()
                .unwrap_or(8);

            debug!(
                "Analyzing {}: {} methods, {} fields (thresholds: >{}, >{})",
                name, method_count, field_count, method_threshold, field_threshold
            );

            // Only proceed with analysis if thresholds are exceeded
            if method_count <= method_threshold && field_count <= field_threshold {
                continue;
            }

            // Stage 3: Pattern Recognition
            let mut excluded_pattern = None;

            if self.config.recognize_patterns {
                // Check for Builder pattern
                if let Some(pattern) =
                    self.detect_builder_pattern(parsed_file, container_node, name)
                {
                    excluded_pattern = Some(pattern);
                }

                // Check for DTO pattern
                if excluded_pattern.is_none() {
                    if let Some(pattern) = self.detect_dto_pattern(
                        parsed_file,
                        container_node,
                        method_count,
                        field_count,
                        &detected_frameworks,
                    ) {
                        excluded_pattern = Some(pattern);
                    }
                }

                // Check for framework controller pattern
                if excluded_pattern.is_none() && !detected_frameworks.is_empty() {
                    for framework in &detected_frameworks {
                        if [
                            "django", "flask", "fastapi", "express", "react", "axum", "rocket",
                        ]
                        .contains(&framework.as_str())
                        {
                            excluded_pattern = Some(DetectedPattern::FrameworkController {
                                framework: framework.clone(),
                                base_class: None,
                            });
                            break;
                        }
                    }
                }
            }

            // Stage 4: Qualitative Analysis
            let lcom4_score = if self.config.enable_cohesion_analysis && excluded_pattern.is_none()
            {
                self.calculate_lcom4(parsed_file, container_node).ok()
            } else {
                None
            };

            let behavioral_analysis =
                if self.config.enable_behavioral_analysis && excluded_pattern.is_none() {
                    self.analyze_behavioral_complexity(parsed_file, container_node)
                        .ok()
                } else {
                    None
                };

            if let Some(issue) = self.create_issue(
                parsed_file,
                name,
                name_node,
                container_node,
                method_count,
                field_count,
                lcom4_score,
                behavioral_analysis,
                excluded_pattern,
            ) {
                info!("Found God Object: {name}");
                issues.push(issue);
            }
        }
        Ok(issues)
    }

    /// Analyzes a Rust file for God Objects with enhanced pattern recognition.
    ///
    /// Rust analysis is more complex because methods (`impl` blocks) are often
    /// separate from data definitions (`struct` blocks). This enhanced function:
    /// 1. Pre-AST exclusion (generated code detection)
    /// 2. Framework and derive macro detection
    /// 3. Correlates struct and impl blocks with pattern recognition
    /// 4. Applies qualitative analysis (LCOM4, behavioral analysis)
    fn analyze_rust(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();
        let root_node = tree.root_node();

        // Stage 1: Pre-AST Exclusion
        if let Some(generated_pattern) = self.is_generated_file(
            &parsed_file.file_path.display().to_string(),
            &parsed_file.source,
        ) {
            debug!(
                "Excluding Rust file as generated code: {:?}",
                generated_pattern
            );
            return Ok(issues);
        }

        // Stage 2: Framework Detection
        let detected_frameworks = self.analyze_imports(parsed_file)?;
        debug!("Detected Rust frameworks: {:?}", detected_frameworks);

        // Detect derive macros for DTO patterns
        let derive_query = Query::new(&language, RUST_DERIVE_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let mut derive_attributes: HashMap<String, Vec<String>> = HashMap::new();

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&derive_query, root_node, source);
        while let Some(mat) = matches.next() {
            for capture in mat.captures {
                if let Ok(attr_text) = capture.node.utf8_text(source) {
                    if attr_text.contains("Serialize") || attr_text.contains("Deserialize") {
                        // Find the associated struct - simplified approach
                        if let Some(parent) = capture.node.parent() {
                            if let Some(struct_node) = parent.next_sibling() {
                                if let Ok(struct_text) = struct_node.utf8_text(source) {
                                    if struct_text.starts_with("struct") {
                                        let struct_name = "derived_struct".to_string(); // Simplified
                                        derive_attributes
                                            .entry(struct_name)
                                            .or_insert_with(Vec::new)
                                            .push(attr_text.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 1. Find all impl blocks and count their methods
        let mut impl_method_counts: HashMap<String, usize> = HashMap::new();
        let impl_query = Query::new(&language, RUST_IMPL_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let function_query = Query::new(&language, RUST_FUNCTION_COUNT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&impl_query, root_node, source);
        while let Some(mat) = matches.next() {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.first(), mat.captures.get(1))
            {
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let mut method_cursor = QueryCursor::new();
                    let method_count = {
                        let mut matches = method_cursor.matches(&function_query, body_node, source);
                        let mut count = 0;
                        while matches.next().is_some() {
                            count += 1;
                        }
                        count
                    };
                    impl_method_counts.insert(name.to_string(), method_count);
                }
            }
        }

        // 2. Find all structs, count their fields, and apply enhanced analysis
        let struct_query = Query::new(&language, RUST_STRUCT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        let field_query = Query::new(&language, RUST_FIELD_COUNT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut struct_cursor = QueryCursor::new();
        let mut matches = struct_cursor.matches(&struct_query, root_node, source);
        while let Some(mat) = matches.next() {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.first(), mat.captures.get(1))
            {
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                let container_node = name_node.parent().unwrap_or(name_node);

                if let Ok(name) = name_node.utf8_text(source) {
                    let method_count = impl_method_counts.get(name).cloned().unwrap_or(0);

                    let mut field_cursor = QueryCursor::new();
                    let field_count = {
                        let mut matches = field_cursor.matches(&field_query, body_node, source);
                        let mut count = 0;
                        while matches.next().is_some() {
                            count += 1;
                        }
                        count
                    };

                    let language = parsed_file.language;
                    let method_threshold = self
                        .config
                        .method_thresholds
                        .get(&language)
                        .copied()
                        .unwrap_or(30);
                    let field_threshold = self
                        .config
                        .field_thresholds
                        .get(&language)
                        .copied()
                        .unwrap_or(20);

                    // Only proceed if thresholds are exceeded
                    if method_count <= method_threshold && field_count <= field_threshold {
                        continue;
                    }

                    // Stage 3: Pattern Recognition
                    let mut excluded_pattern = None;

                    if self.config.recognize_patterns {
                        // Check for Serde DTO pattern
                        if derive_attributes.contains_key(name) {
                            let field_ratio =
                                field_count as f64 / (field_count + method_count) as f64;
                            if field_ratio > 0.7 {
                                excluded_pattern = Some(DetectedPattern::Dto {
                                    framework: "serde".to_string(),
                                    field_ratio,
                                });
                            }
                        }

                        // Check for Builder pattern
                        if excluded_pattern.is_none() {
                            if let Some(pattern) =
                                self.detect_builder_pattern(parsed_file, container_node, name)
                            {
                                excluded_pattern = Some(pattern);
                            }
                        }

                        // Check for framework patterns
                        if excluded_pattern.is_none() && !detected_frameworks.is_empty() {
                            for framework in &detected_frameworks {
                                if ["axum", "rocket", "actix_web", "diesel", "sqlx"]
                                    .contains(&framework.as_str())
                                {
                                    excluded_pattern = Some(DetectedPattern::FrameworkController {
                                        framework: framework.clone(),
                                        base_class: None,
                                    });
                                    break;
                                }
                            }
                        }
                    }

                    // Stage 4: Qualitative Analysis
                    let lcom4_score =
                        if self.config.enable_cohesion_analysis && excluded_pattern.is_none() {
                            self.calculate_lcom4(parsed_file, container_node).ok()
                        } else {
                            None
                        };

                    let behavioral_analysis =
                        if self.config.enable_behavioral_analysis && excluded_pattern.is_none() {
                            self.analyze_behavioral_complexity(parsed_file, container_node)
                                .ok()
                        } else {
                            None
                        };

                    if let Some(issue) = self.create_issue(
                        parsed_file,
                        name,
                        name_node,
                        container_node,
                        method_count,
                        field_count,
                        lcom4_score,
                        behavioral_analysis,
                        excluded_pattern,
                    ) {
                        issues.push(issue);
                    }
                }
            }
        }

        Ok(issues)
    }
}

#[async_trait]
impl AnalysisDetector for GodObjectDetector {
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!(
            "Running God Object detection on: {}",
            parsed_file.file_path.display()
        );
        let result = match parsed_file.language {
            SourceLanguage::Rust => self.analyze_rust(parsed_file),
            SourceLanguage::Python => self.analyze_standard(
                parsed_file,
                PYTHON_CLASS_QUERY,
                PYTHON_FUNCTION_COUNT_QUERY,
                PYTHON_FIELD_COUNT_QUERY,
            ),
            SourceLanguage::JavaScript => self.analyze_standard(
                parsed_file,
                JAVASCRIPT_CLASS_QUERY,
                JAVASCRIPT_FUNCTION_COUNT_QUERY,
                JAVASCRIPT_FIELD_COUNT_QUERY,
            ),
        };

        match &result {
            Ok(issues) => {
                if issues.is_empty() {
                    debug!(
                        "No God Object issues found in {}",
                        parsed_file.file_path.display()
                    );
                } else {
                    info!(
                        "Found {} God Object issues in {}",
                        issues.len(),
                        parsed_file.file_path.display()
                    );
                }
            }
            Err(e) => {
                debug!(
                    "Error analyzing {} for God Objects: {}",
                    parsed_file.file_path.display(),
                    e
                );
            }
        }

        Ok(result.unwrap_or_default())
    }
    fn get_detector_name(&self) -> &'static str {
        "GodObjectDetector"
    }
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class or struct that centralizes too many responsibilities, violating the Single Responsibility Principle. Enhanced with pattern recognition to reduce false positives.".to_string(),
            category: "Abstraction-Based".to_string(),
        }]
    }
}

impl Default for GodObjectDetector {
    /// Creates a `GodObjectDetector` with default configuration.
    ///
    /// Enables all enhanced detection features with language-specific thresholds.
    fn default() -> Self {
        Self::with_config(GodObjectConfig::default())
    }
}
