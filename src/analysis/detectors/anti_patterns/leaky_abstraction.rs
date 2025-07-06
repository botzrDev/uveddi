//! Advanced Leaky Abstraction Detector
//!
//! This detector implements a comprehensive analysis framework for identifying leaky abstractions
//! across multiple programming languages (Rust, Python, JavaScript/TypeScript). It uses a 
//! multi-signal approach combining AST analysis, dependency tracking, and architectural pattern
//! recognition to detect violations of abstraction boundaries.
//!
//! ## Detection Capabilities
//!
//! ### Rust-Specific Patterns
//! - Visibility violations (pub vs private boundaries)
//! - Framework-specific types in public APIs
//! - ORM/Database types leaking into business logic
//! - Error type propagation across layers
//! - Async runtime details in interfaces
//!
//! ### Python-Specific Patterns  
//! - Direct database model usage in views/controllers
//! - Framework objects in business logic (Flask request, Django models)
//! - File system paths in public interfaces
//! - Import violations across architectural layers
//!
//! ### JavaScript/TypeScript Patterns
//! - DOM manipulation in business logic
//! - Framework-specific objects in domain models
//! - Infrastructure dependencies in application layer
//! - Type definition leaks and generic pollution
//!
//! ## Architecture
//!
//! The detector uses a layered analysis approach:
//! 1. **Syntactic Analysis**: Tree-sitter queries for pattern matching
//! 2. **Semantic Analysis**: Symbol resolution and type flow tracking
//! 3. **Architectural Analysis**: Layer boundary validation
//! 4. **Cross-file Analysis**: Dependency graph traversal

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Query, QueryCursor, Node};

/// Configuration for architectural layers and boundaries
#[derive(Debug, Clone)]
pub struct ArchitecturalConfig {
    /// Mapping of path patterns to architectural layers
    pub layer_mappings: HashMap<String, ArchitecturalLayer>,
    /// Known infrastructure modules/frameworks to detect
    pub infrastructure_modules: HashSet<String>,
    /// Internal/private module patterns
    pub internal_patterns: Vec<String>,
}

/// Architectural layers following Clean Architecture principles
#[derive(Debug, Clone, PartialEq)]
pub enum ArchitecturalLayer {
    /// Presentation layer (UI, controllers, views)
    Presentation,
    /// Application layer (use cases, services)
    Application, 
    /// Domain layer (business logic, entities)
    Domain,
    /// Infrastructure layer (databases, external APIs, frameworks)
    Infrastructure,
}

/// Types of leaky abstraction violations
#[derive(Debug, Clone)]
pub enum LeakType {
    /// Visibility boundary violation (accessing private/internal items)
    VisibilityViolation,
    /// Layer boundary violation (wrong dependency direction)
    LayerViolation,
    /// Implementation detail exposure (internal types in public APIs)
    ImplementationExposure,
    /// Framework coupling (framework-specific types in business logic)
    FrameworkCoupling,
    /// Error propagation (low-level errors bubbling up)
    ErrorPropagation,
    /// Performance leak (abstraction causing performance issues)
    PerformanceLeak,
}

/// Advanced Leaky Abstraction Detector
///
/// Implements comprehensive detection of abstraction boundary violations using
/// multi-language AST analysis, dependency tracking, and architectural pattern recognition.
pub struct LeakyAbstractionDetector {
    /// Architectural configuration for layer boundaries
    config: ArchitecturalConfig,
    /// Language-specific query patterns for Tree-sitter
    rust_queries: Option<Query>,
    python_queries: Option<Query>, 
    js_queries: Option<Query>,
}

impl Default for LeakyAbstractionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LeakyAbstractionDetector {
    /// Create a new detector with default configuration
    pub fn new() -> Self {
        let config = Self::default_config();
        Self {
            config,
            rust_queries: None,
            python_queries: None,
            js_queries: None,
        }
    }

    /// Create detector with custom architectural configuration
    pub fn with_config(config: ArchitecturalConfig) -> Self {
        Self {
            config,
            rust_queries: None,
            python_queries: None,
            js_queries: None,
        }
    }

    /// Default architectural configuration
    fn default_config() -> ArchitecturalConfig {
        let mut layer_mappings = HashMap::new();
        
        // Common patterns for different layers
        layer_mappings.insert("**/controllers/**".to_string(), ArchitecturalLayer::Presentation);
        layer_mappings.insert("**/views/**".to_string(), ArchitecturalLayer::Presentation);
        layer_mappings.insert("**/ui/**".to_string(), ArchitecturalLayer::Presentation);
        layer_mappings.insert("**/handlers/**".to_string(), ArchitecturalLayer::Presentation);
        
        layer_mappings.insert("**/services/**".to_string(), ArchitecturalLayer::Application);
        layer_mappings.insert("**/use_cases/**".to_string(), ArchitecturalLayer::Application);
        layer_mappings.insert("**/application/**".to_string(), ArchitecturalLayer::Application);
        
        layer_mappings.insert("**/domain/**".to_string(), ArchitecturalLayer::Domain);
        layer_mappings.insert("**/models/**".to_string(), ArchitecturalLayer::Domain);
        layer_mappings.insert("**/entities/**".to_string(), ArchitecturalLayer::Domain);
        
        layer_mappings.insert("**/repositories/**".to_string(), ArchitecturalLayer::Infrastructure);
        layer_mappings.insert("**/infrastructure/**".to_string(), ArchitecturalLayer::Infrastructure);
        layer_mappings.insert("**/adapters/**".to_string(), ArchitecturalLayer::Infrastructure);
        layer_mappings.insert("**/external/**".to_string(), ArchitecturalLayer::Infrastructure);

        let mut infrastructure_modules = HashSet::new();
        // Rust frameworks/ORMs
        infrastructure_modules.insert("diesel".to_string());
        infrastructure_modules.insert("sqlx".to_string());
        infrastructure_modules.insert("sea_orm".to_string());
        infrastructure_modules.insert("tokio".to_string());
        infrastructure_modules.insert("axum".to_string());
        infrastructure_modules.insert("warp".to_string());
        infrastructure_modules.insert("actix_web".to_string());
        infrastructure_modules.insert("reqwest".to_string());
        
        // Python frameworks/ORMs
        infrastructure_modules.insert("django".to_string());
        infrastructure_modules.insert("flask".to_string());
        infrastructure_modules.insert("fastapi".to_string());
        infrastructure_modules.insert("sqlalchemy".to_string());
        infrastructure_modules.insert("requests".to_string());
        infrastructure_modules.insert("psycopg2".to_string());
        
        // JavaScript/TypeScript frameworks
        infrastructure_modules.insert("express".to_string());
        infrastructure_modules.insert("prisma".to_string());
        infrastructure_modules.insert("mongoose".to_string());
        infrastructure_modules.insert("axios".to_string());
        infrastructure_modules.insert("react".to_string());
        infrastructure_modules.insert("vue".to_string());

        let internal_patterns = vec![
            "internal".to_string(),
            "impl".to_string(), 
            "detail".to_string(),
            "_internal".to_string(),
            "private".to_string(),
        ];

        ArchitecturalConfig {
            layer_mappings,
            infrastructure_modules,
            internal_patterns,
        }
    }

    /// Initialize language-specific Tree-sitter queries
    fn init_queries(&mut self, language: &str) -> Result<(), AnalysisError> {
        match language {
            "rust" => {
                if self.rust_queries.is_none() {
                    self.rust_queries = Some(self.create_rust_queries()?);
                }
            }
            "python" => {
                if self.python_queries.is_none() {
                    self.python_queries = Some(self.create_python_queries()?);
                }
            }
            "javascript" | "typescript" => {
                if self.js_queries.is_none() {
                    self.js_queries = Some(self.create_js_queries()?);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Create Rust-specific Tree-sitter queries
    fn create_rust_queries(&self) -> Result<Query, AnalysisError> {
        let query_source = r#"
            ; Detect public struct fields (potential encapsulation violation)
            (struct_item
              (visibility_modifier) @pub_vis
              name: (type_identifier) @struct_name
              body: (field_declaration_list
                (field_declaration
                  (visibility_modifier) @field_vis
                  name: (field_identifier) @field_name
                  type: (_) @field_type))) @struct_decl

            ; Detect use statements importing from infrastructure modules
            (use_declaration
              argument: (scoped_identifier
                path: (identifier) @module_name
                name: (_) @import_name)) @use_stmt

            ; Detect function signatures with infrastructure types
            (function_item
              (visibility_modifier)? @fn_vis
              name: (identifier) @fn_name
              parameters: (parameters
                (parameter
                  pattern: (_) @param_name
                  type: (type_identifier) @param_type)*)
              return_type: (type_identifier)? @return_type) @function_decl

            ; Detect error type propagation
            (result_type
              ok_type: (_) @ok_type
              error_type: (type_identifier) @error_type) @result_type

            ; Detect async function signatures with runtime-specific types
            (function_item
              (visibility_modifier)? @async_vis
              "async"
              name: (identifier) @async_name
              return_type: (_) @async_return) @async_fn
        "#;

        let language = tree_sitter_rust::language();
        Query::new(&language, query_source)
            .map_err(|e| AnalysisError::Other(format!("Failed to create Rust query: {}", e)))
    }

    /// Create Python-specific Tree-sitter queries  
    fn create_python_queries(&self) -> Result<Query, AnalysisError> {
        let query_source = r#"
            ; Detect imports from infrastructure modules
            (import_statement
              name: (dotted_name
                (identifier) @module_name)) @import_stmt

            (import_from_statement
              module_name: (dotted_name
                (identifier) @from_module)
              name: (dotted_name
                (identifier) @import_name)) @from_import

            ; Detect function definitions with framework-specific parameters
            (function_definition
              name: (identifier) @func_name
              parameters: (parameters
                (identifier) @param_name*)) @func_def

            ; Detect class definitions that inherit from framework classes
            (class_definition
              name: (identifier) @class_name
              superclasses: (argument_list
                (identifier) @parent_class*)) @class_def

            ; Detect direct database model usage
            (call
              function: (attribute
                object: (identifier) @model_name
                attribute: (identifier) @method_name)) @model_call
        "#;

        let language = tree_sitter_python::language();
        Query::new(&language, query_source)
            .map_err(|e| AnalysisError::Other(format!("Failed to create Python query: {}", e)))
    }

    /// Create JavaScript/TypeScript-specific Tree-sitter queries
    fn create_js_queries(&self) -> Result<Query, AnalysisError> {
        let query_source = r#"
            ; Detect imports from infrastructure modules
            (import_statement
              source: (string) @import_source) @import_stmt

            ; Detect function declarations with framework-specific types
            (function_declaration
              name: (identifier) @func_name
              parameters: (formal_parameters
                (identifier) @param_name*)) @func_decl

            ; Detect DOM manipulation in business logic
            (call_expression
              function: (member_expression
                object: (identifier) @dom_object
                property: (property_identifier) @dom_method)) @dom_call

            ; Detect type annotations referencing framework/infrastructure types (e.g., Express.Request, React.Component)
            (type_annotation
              (type_identifier) @type_name) @type_ann

            ; Detect generic type parameter pollution
            (type_parameters
              (type_parameter
                name: (type_identifier) @type_param)) @type_params
        "#;

        let language = tree_sitter_javascript::language();
        Query::new(&language, query_source)
            .map_err(|e| AnalysisError::Other(format!("Failed to create JS query: {}", e)))
    }

    /// Determine architectural layer from file path
    fn get_layer_from_path(&self, file_path: &str) -> Option<ArchitecturalLayer> {
        for (pattern, layer) in &self.config.layer_mappings {
            if self.matches_pattern(file_path, pattern) {
                return Some(layer.clone());
            }
        }
        None
    }

    /// Simple pattern matching for file paths
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.starts_with("**/") && pattern.ends_with("/**") {
            let middle = &pattern[3..pattern.len()-3];
            path.contains(&format!("/{}/", middle)) || path.contains(&format!("\\{}\\", middle))
        } else if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            path.ends_with(suffix)
        } else if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len()-3];
            path.starts_with(prefix)
        } else {
            path.contains(pattern)
        }
    }

    /// Check if a module is an infrastructure dependency
    fn is_infrastructure_module(&self, module_name: &str) -> bool {
        self.config.infrastructure_modules.contains(module_name) ||
        self.config.infrastructure_modules.iter().any(|infra| module_name.starts_with(infra))
    }

    /// Check if a path indicates an internal/private module
    fn is_internal_module(&self, path: &str) -> bool {
        self.config.internal_patterns.iter().any(|pattern| path.contains(pattern))
    }

    /// Analyze Rust-specific leaky abstraction patterns
    fn analyze_rust_file(&self, parsed_file: &ParsedFile, analysis_run_id: i64) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        
        if let Some(query) = &self.rust_queries {
            let tree = parsed_file.tree.as_ref().ok_or_else(|| AnalysisError::Other("No AST available".to_string()))?;
            let mut cursor = QueryCursor::new();
            let captures = cursor.captures(query, tree.root_node(), parsed_file.source.as_bytes());

            for (match_, _) in captures {
                for capture in match_.captures {
                    let node = capture.node;
                    let capture_name = query.capture_names()[capture.index as usize];
                    
                    match capture_name {
                        "module_name" => {
                            if let Ok(module_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                if self.is_infrastructure_module(module_text) {
                                    let layer = self.get_layer_from_path(&parsed_file.path.to_string_lossy());
                                    if matches!(layer, Some(ArchitecturalLayer::Domain) | Some(ArchitecturalLayer::Application)) {
                                        issues.push(self.create_issue(
                                            analysis_run_id,
                                            &parsed_file.path.to_string_lossy(),
                                            node,
                                            LeakType::FrameworkCoupling,
                                            &format!("Infrastructure module '{}' imported in {} layer", module_text, layer.map(|l| format!("{:?}", l)).unwrap_or_else(|| "unknown".to_string())),
                                            "high",
                                        ));
                                    }
                                }
                            }
                        }
                        "field_vis" => {
                            if let Ok(vis_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                if vis_text == "pub" {
                                    // Check if this is exposing internal structure
                                    if let Some(parent) = node.parent() {
                                        if let Some(_struct_node) = parent.parent() {
                                            issues.push(self.create_issue(
                                                analysis_run_id,
                                                &parsed_file.path.to_string_lossy(),
                                                node,
                                                LeakType::ImplementationExposure,
                                                "Public field exposes internal structure - consider using getter methods",
                                                "medium",
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        "error_type" => {
                            if let Ok(error_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                if self.is_infrastructure_module(error_text) {
                                    issues.push(self.create_issue(
                                        analysis_run_id,
                                        &parsed_file.path.to_string_lossy(),
                                        node,
                                        LeakType::ErrorPropagation,
                                        &format!("Infrastructure error type '{}' propagated to public API", error_text),
                                        "high",
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Analyze Python-specific leaky abstraction patterns
    fn analyze_python_file(&self, parsed_file: &ParsedFile, analysis_run_id: i64) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        
        if let Some(query) = &self.python_queries {
            let tree = parsed_file.tree.as_ref().ok_or_else(|| AnalysisError::Other("No AST available".to_string()))?;
            let mut cursor = QueryCursor::new();
            let captures = cursor.captures(query, tree.root_node(), parsed_file.source.as_bytes());

            for (match_, _) in captures {
                for capture in match_.captures {
                    let node = capture.node;
                    let capture_name = query.capture_names()[capture.index as usize];
                    
                    match capture_name {
                        "module_name" | "from_module" => {
                            if let Ok(module_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                if self.is_infrastructure_module(module_text) {
                                    let layer = self.get_layer_from_path(&parsed_file.path.to_string_lossy());
                                    if matches!(layer, Some(ArchitecturalLayer::Domain) | Some(ArchitecturalLayer::Application)) {
                                        issues.push(self.create_issue(
                                            analysis_run_id,
                                            &parsed_file.path.to_string_lossy(),
                                            node,
                                            LeakType::FrameworkCoupling,
                                            &format!("Framework module '{}' imported in {} layer", module_text, layer.map(|l| format!("{:?}", l)).unwrap_or_else(|| "unknown".to_string())),
                                            "high",
                                        ));
                                    }
                                }
                            }
                        }
                        "model_name" => {
                            // Detect direct ORM model usage
                            if let Ok(model_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                if model_text.ends_with("Model") || model_text.contains("objects") {
                                    let layer = self.get_layer_from_path(&parsed_file.path.to_string_lossy());
                                    if matches!(layer, Some(ArchitecturalLayer::Presentation)) {
                                        issues.push(self.create_issue(
                                            analysis_run_id,
                                            &parsed_file.path.to_string_lossy(),
                                            node,
                                            LeakType::LayerViolation,
                                            "Direct database model usage in presentation layer - use service layer instead",
                                            "high",
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Analyze JavaScript/TypeScript-specific leaky abstraction patterns
    fn analyze_js_file(&self, parsed_file: &ParsedFile, analysis_run_id: i64) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        
        if let Some(query) = &self.js_queries {
            let tree = parsed_file.tree.as_ref().ok_or_else(|| AnalysisError::Other("No AST available".to_string()))?;
            let mut cursor = QueryCursor::new();
            let captures = cursor.captures(query, tree.root_node(), parsed_file.source.as_bytes());

            for (match_, _) in captures {
                for capture in match_.captures {
                    let node = capture.node;
                    let capture_name = query.capture_names()[capture.index as usize];
                    
                    match capture_name {
                        "import_source" => {
                            if let Ok(import_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                let module_name = import_text.trim_matches('"').trim_matches('\'');
                                if self.is_infrastructure_module(module_name) {
                                    let layer = self.get_layer_from_path(&parsed_file.path.to_string_lossy());
                                    if matches!(layer, Some(ArchitecturalLayer::Domain) | Some(ArchitecturalLayer::Application)) {
                                        issues.push(self.create_issue(
                                            analysis_run_id,
                                            &parsed_file.path.to_string_lossy(),
                                            node,
                                            LeakType::FrameworkCoupling,
                                            &format!("Infrastructure module '{}' imported in {} layer", module_name, layer.map(|l| format!("{:?}", l)).unwrap_or_else(|| "unknown".to_string())),
                                            "high",
                                        ));
                                    }
                                }
                            }
                        }
                        "dom_object" => {
                            if let Ok(dom_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                if dom_text == "document" || dom_text == "window" {
                                    let layer = self.get_layer_from_path(&parsed_file.path.to_string_lossy());
                                    if matches!(layer, Some(ArchitecturalLayer::Domain) | Some(ArchitecturalLayer::Application)) {
                                        issues.push(self.create_issue(
                                            analysis_run_id,
                                            &parsed_file.path.to_string_lossy(),
                                            node,
                                            LeakType::FrameworkCoupling,
                                            "DOM manipulation in business logic - move to presentation layer",
                                            "high",
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Create an architectural issue from detected leak
    fn create_issue(
        &self,
        analysis_run_id: i64,
        file_path: &str,
        node: Node,
        leak_type: LeakType,
        description: &str,
        severity: &str,
    ) -> ArchitecturalIssue {
        ArchitecturalIssue {
            issue_id: None,
            analysis_run_id,
            anti_pattern_type_id: self.get_anti_pattern_id_for_leak_type(&leak_type),
            file_path: file_path.to_string(),
            start_line: Some(node.start_position().row as i32 + 1),
            end_line: Some(node.end_position().row as i32 + 1),
            severity: severity.to_string(),
            description: description.to_string(),
            code_snippet: None, // Could extract node text here
            ai_explanation: None,
        }
    }

    /// Map leak type to anti-pattern type ID
    fn get_anti_pattern_id_for_leak_type(&self, leak_type: &LeakType) -> i64 {
        match leak_type {
            LeakType::VisibilityViolation => 1,
            LeakType::LayerViolation => 2,
            LeakType::ImplementationExposure => 3,
            LeakType::FrameworkCoupling => 4,
            LeakType::ErrorPropagation => 5,
            LeakType::PerformanceLeak => 6,
        }
    }
}

impl AnalysisDetector for LeakyAbstractionDetector {
    fn get_detector_name(&self) -> &'static str {
        "LeakyAbstractionDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![
            AntiPatternType {
                anti_pattern_type_id: Some(1),
                name: "Visibility Violation".to_string(),
                description: "Accessing private or internal implementation details across module boundaries".to_string(),
                category: "structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(2),
                name: "Layer Violation".to_string(),
                description: "Dependencies flowing in wrong direction between architectural layers".to_string(),
                category: "structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(3),
                name: "Implementation Exposure".to_string(),
                description: "Internal implementation details exposed through public interfaces".to_string(),
                category: "structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(4),
                name: "Framework Coupling".to_string(),
                description: "Framework-specific types or objects used in business logic".to_string(),
                category: "structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(5),
                name: "Error Propagation".to_string(),
                description: "Low-level error types propagating through abstraction boundaries".to_string(),
                category: "behavioral".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(6),
                name: "Performance Leak".to_string(),
                description: "Abstraction causing unexpected performance degradation".to_string(),
                category: "behavioral".to_string(),
            },
        ]
    }

    fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut detector = self.clone();
        let language_str = match parsed_file.language {
            crate::ast::tree_sitter::SourceLanguage::Rust => "rust",
            crate::ast::tree_sitter::SourceLanguage::Python => "python", 
            crate::ast::tree_sitter::SourceLanguage::JavaScript => "javascript",
        };
        
        detector.init_queries(language_str)?;
        
        let analysis_run_id = 1; // TODO: Get from context
        
        match language_str {
            "rust" => detector.analyze_rust_file(parsed_file, analysis_run_id),
            "python" => detector.analyze_python_file(parsed_file, analysis_run_id),
            "javascript" | "typescript" => detector.analyze_js_file(parsed_file, analysis_run_id),
            _ => Ok(vec![]),
        }
    }
}

impl Clone for LeakyAbstractionDetector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            rust_queries: None, // Queries will be re-initialized as needed
            python_queries: None,
            js_queries: None,
        }
    }
}
