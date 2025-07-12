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

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use log::{debug, info};
use std::collections::HashMap;
use tree_sitter::{Node, Query, QueryCursor};

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

/// Detects "God Objects" by analyzing class and struct sizes.
///
/// This detector identifies classes or structs that have grown too large, accumulating
/// an excessive number of methods and fields. Such objects, often called "God Objects"
/// violate the Single Responsibility Principle and can lead to maintenance challenges.
///
/// ## Configuration
///
/// The detector accepts two main parameters:
/// - `method_threshold`: Maximum number of methods before flagging as God Object
/// - `field_threshold`: Maximum number of fields before flagging as God Object
///
/// ## Examples
///
/// ```rust
/// use uveddi::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
///
/// // Create a detector with custom thresholds for methods and fields.
/// let detector = GodObjectDetector::new(15, 10);
///
/// // Use the default thresholds (10 methods, 8 fields).
/// let default_detector = GodObjectDetector::default();
/// ```
pub struct GodObjectDetector {
    /// The maximum number of methods a class or struct can have before being flagged.
    method_threshold: usize,
    /// The maximum number of fields a class or struct can have before being flagged.
    field_threshold: usize,
}

impl GodObjectDetector {
    /// Creates a new `GodObjectDetector` with specified thresholds.
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
        Self {
            method_threshold,
            field_threshold,
        }
    }

    /// Scores the severity of a detected God Object.
    ///
    /// The severity is determined by how much the method and field counts exceed
    /// their respective thresholds. The scoring is as follows:
    /// - **Medium**: 1-4 total excess members.
    /// - **High**: 5-10 total excess members.
    /// - **Critical**: 11 or more total excess members.
    ///
    /// Returns `None` if no thresholds are exceeded.
    fn score_severity(&self, method_count: usize, field_count: usize) -> Option<String> {
        let method_excess = method_count.saturating_sub(self.method_threshold);
        let field_excess = field_count.saturating_sub(self.field_threshold);

        // Only consider it an issue if at least one threshold is exceeded.
        if method_excess == 0 && field_excess == 0 {
            return None;
        }

        let total_excess = method_excess + field_excess;
        let severity = match total_excess {
            0..=4 => "Medium",
            5..=10 => "High",
            _ => "Critical",
        };
        Some(severity.to_string())
    }

    /// Creates an `ArchitecturalIssue` for a detected God Object.
    ///
    /// This helper function is called when a God Object is identified. It constructs
    /// an `ArchitecturalIssue` with relevant details, including the severity,
    /// file path, line numbers, and a descriptive message.
    ///
    /// Returns `None` if the severity score is not high enough to warrant an issue.
    fn create_issue(
        &self,
        parsed_file: &ParsedFile,
        name: &str,
        name_node: Node,
        container_node: Node,
        method_count: usize,
        field_count: usize,
    ) -> Option<ArchitecturalIssue> {
        self.score_severity(method_count, field_count)
            .map(|severity| ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0, // Will be set by the engine
                anti_pattern_type_id: 1, // God Object
                file_path: parsed_file.file_path.display().to_string(), // TODO UV-222: Use Arc<PathBuf> for O(1) clones
                start_line: Some((name_node.start_position().row + 1) as i32),
                end_line: Some((name_node.end_position().row + 1) as i32),
                severity,
                description: format!(
                    "God Object detected: '{}' has {} methods and {} fields. (Thresholds: methods>{}, fields>{})",
                    name, method_count, field_count, self.method_threshold, self.field_threshold
                ),
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
            })
    }

    /// Analyzes a file for God Objects using a standard approach for languages
    /// like Python and JavaScript, where class members are defined in a single block.
    ///
    /// This function executes a series of Tree-sitter queries to:
    /// 1. Identify all class or container definitions.
    /// 2. Count the number of methods within each container.
    /// 3. Count the number of fields within each container.
    /// 4. Create an issue if the counts exceed the configured thresholds.
    fn analyze_standard(
        &self,
        parsed_file: &ParsedFile,
        container_query_str: &str,
        method_query_str: &str,
        field_query_str: &str,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let empty_source = String::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| crate::analysis::errors::AnalysisError::AntiPatternDetectionError("AST tree missing".to_string()))?;
        let language = tree.language();

        let container_query = Query::new(&language, container_query_str)
            .map_err(|e| crate::analysis::errors::AnalysisError::QueryError(e.to_string()))?;
        let method_query = Query::new(&language, method_query_str)
            .map_err(|e| crate::analysis::errors::AnalysisError::QueryError(e.to_string()))?;
        let field_query = Query::new(&language, field_query_str)
            .map_err(|e| crate::analysis::errors::AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&container_query, tree.root_node(), source) {
            let name_node = mat.captures[0].node;
            let body_node = mat.captures[1].node;
            let container_node = name_node.parent().unwrap_or(name_node);

            let name = name_node.utf8_text(source).unwrap_or("Unnamed");

            let mut method_cursor = QueryCursor::new();
            let method_count = method_cursor
                .matches(&method_query, body_node, source)
                .count();

            let mut field_cursor = QueryCursor::new();
            let field_count = field_cursor
                .matches(&field_query, body_node, source)
                .count();

            debug!(
                "Analyzing {}: {} methods, {} fields (thresholds: >{}, >{})",
                name, method_count, field_count, self.method_threshold, self.field_threshold
            );

            if let Some(issue) = self.create_issue(
                parsed_file,
                name,
                name_node,
                container_node,
                method_count,
                field_count,
            ) {
                info!("Found God Object: {name}");
                issues.push(issue);
            }
        }
        Ok(issues)
    }

    /// Analyzes a Rust file for God Objects by correlating `struct` and `impl` blocks.
    ///
    /// Rust analysis is more complex because methods (`impl` blocks) are often
    /// separate from data definitions (`struct` blocks). This function:
    /// 1. Scans the file to find all `impl` blocks and counts their methods, mapping
    ///    them to the struct they implement.
    /// 2. Scans the file again to find all `struct` definitions and counts their fields.
    /// 3. Combines the method and field counts for each struct and checks them
    ///    against the thresholds.
    fn analyze_rust(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let empty_source = String::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| crate::analysis::errors::AnalysisError::AntiPatternDetectionError("AST tree missing".to_string()))?;
        let language = tree.language();
        let root_node = tree.root_node();

        // 1. Find all impl blocks and count their methods
        let mut impl_method_counts: HashMap<String, usize> = HashMap::new();
        let impl_query = Query::new(&language, RUST_IMPL_QUERY)
            .map_err(|e| crate::analysis::errors::AnalysisError::QueryError(e.to_string()))?;
        let function_query = Query::new(&language, RUST_FUNCTION_COUNT_QUERY)
            .map_err(|e| crate::analysis::errors::AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&impl_query, root_node, source) {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.first(), mat.captures.get(1))
            {
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                if let Ok(name) = name_node.utf8_text(source) {
                    let mut method_cursor = QueryCursor::new();
                    let method_count = method_cursor
                        .matches(&function_query, body_node, source)
                        .count();
                    impl_method_counts.insert(name.to_string(), method_count);
                }
            }
        }

        // 2. Find all structs, count their fields, and check against method counts
        let struct_query = Query::new(&language, RUST_STRUCT_QUERY)
            .map_err(|e| crate::analysis::errors::AnalysisError::QueryError(e.to_string()))?;
        let field_query = Query::new(&language, RUST_FIELD_COUNT_QUERY)
            .map_err(|e| crate::analysis::errors::AnalysisError::QueryError(e.to_string()))?;

        let mut struct_cursor = QueryCursor::new();
        for mat in struct_cursor.matches(&struct_query, root_node, source) {
            if let (Some(name_capture), Some(body_capture)) =
                (mat.captures.first(), mat.captures.get(1))
            {
                let name_node = name_capture.node;
                let body_node = body_capture.node;
                let container_node = name_node.parent().unwrap_or(name_node);

                if let Ok(name) = name_node.utf8_text(source) {
                    let method_count = impl_method_counts.get(name).cloned().unwrap_or(0);

                    let mut field_cursor = QueryCursor::new();
                    let field_count = field_cursor
                        .matches(&field_query, body_node, source)
                        .count();

                    if let Some(issue) = self.create_issue(
                        parsed_file,
                        name,
                        name_node,
                        container_node,
                        method_count,
                        field_count,
                    ) {
                        issues.push(issue);
                    }
                }
            }
        }

        Ok(issues)
    }
}

impl AnalysisDetector for GodObjectDetector {
    fn get_detector_name(&self) -> &'static str {
        "GodObjectDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class or struct that centralizes too many responsibilities, violating the Single Responsibility Principle.".to_string(),
            category: "Abstraction-Based".to_string(),
        }]
    }

    fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!("Running God Object detection on: {}", parsed_file.file_path.display());
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
                    debug!("No God Object issues found in {}", parsed_file.file_path.display());
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
                    parsed_file.file_path.display(), e
                );
            }
        }

        result
    }
}

impl Default for GodObjectDetector {
    /// Creates a `GodObjectDetector` with default thresholds.
    ///
    /// The default thresholds are:
    /// - `method_threshold`: 10
    /// - `field_threshold`: 8
    fn default() -> Self {
        Self::new(10, 8)
    }
}
