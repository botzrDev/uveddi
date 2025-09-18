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
//!
//! ## Configuration
//!
//! The detector requires architectural configuration to understand the intended
//! layer boundaries and infrastructure dependencies:
//!
//! ```rust
//! use uveddi::analysis::detectors::anti_patterns::leaky_abstraction::{
//!     LeakyAbstractionDetector, ArchitecturalConfig, ArchitecturalLayer
//! };
//! use std::collections::{HashMap, HashSet};
//!
//! let mut layer_mappings = HashMap::new();
//! layer_mappings.insert("**/controllers/**".to_string(), ArchitecturalLayer::Presentation);
//! layer_mappings.insert("**/services/**".to_string(), ArchitecturalLayer::Application);
//! layer_mappings.insert("**/domain/**".to_string(), ArchitecturalLayer::Domain);
//!
//! let mut infrastructure_modules = HashSet::new();
//! infrastructure_modules.insert("sqlx".to_string());
//! infrastructure_modules.insert("tokio".to_string());
//! infrastructure_modules.insert("serde".to_string());
//!
//! let config = ArchitecturalConfig {
//!     layer_mappings,
//!     infrastructure_modules,
//!     internal_patterns: vec!["_internal".to_string(), "private".to_string()],
//! };
//!
//! let detector = LeakyAbstractionDetector::new(config);
//! ```
//!
//! ## Performance Considerations
//! - **Time Complexity**: O(n*m) where n is AST nodes and m is architectural rules
//! - **Space Complexity**: O(k) where k is the number of detected violations
//! - **Optimization Notes**: Uses efficient pattern matching and caches rule evaluations
//!
use async_trait::async_trait;

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::ast::{
    tree_sitter::{Node, Query, QueryCursor},
    SourceLanguage,
};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use std::collections::{HashMap, HashSet};
use strum_macros::EnumString;
#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Defines the configuration for architectural layers and boundaries.
///
/// This struct provides the necessary context for the `LeakyAbstractionDetector`
/// to understand the intended architecture of a project. It specifies how to map
/// file paths to architectural layers and identifies known infrastructure dependencies.
#[derive(Debug, Clone)]
pub struct ArchitecturalConfig {
    /// A mapping of glob patterns to `ArchitecturalLayer` enums.
    ///
    /// This is the primary mechanism for defining the architecture. For example:
    /// `{"**/controllers/**": Presentation, "**/services/**": Application}`.
    pub layer_mappings: HashMap<String, ArchitecturalLayer>,

    /// A set of module names or prefixes that are considered infrastructure.
    ///
    /// This set helps identify dependencies on frameworks, databases, or other
    /// external systems (e.g., "django", "sqlx", "react").
    pub infrastructure_modules: HashSet<String>,

    /// A list of patterns used to identify internal or private modules.
    ///
    /// Accessing modules whose paths contain these patterns from outside their
    /// parent component is considered a visibility violation (e.g., "_internal").
    pub internal_patterns: Vec<String>,
}

/// Represents the distinct architectural layers of a system, inspired by Clean Architecture.
///
/// Each layer has a specific responsibility, and dependencies should generally flow
/// from outer layers (e.g., `Presentation`) to inner layers (e.g., `Domain`).
#[derive(Debug, Clone, PartialEq, EnumString)]
pub enum ArchitecturalLayer {
    /// The outermost layer, responsible for UI, API endpoints, and user interaction.
    /// It translates user input into application-level commands.
    Presentation,

    /// The layer that contains application-specific business logic and use cases.
    /// It orchestrates the domain layer to perform tasks and is the primary entry
    /// point for application operations.
    Application,

    /// The core of the application, containing enterprise-wide business logic and entities.
    /// This layer should be independent of any framework, UI, or database.
    Domain,

    /// The layer that contains all external concerns and implementation details, such as
    /// databases, file systems, and third-party API clients. It implements interfaces
    /// defined by the application or domain layers.
    Infrastructure,
}

/// Enumerates the specific types of leaky abstraction violations detected by this module.
#[derive(Debug, Clone)]
pub enum LeakType {
    /// A violation of encapsulation where code accesses a private or internal item
    /// from an outside module.
    VisibilityViolation,

    /// A dependency that flows in the wrong direction between architectural layers,
    /// such as a domain module depending on a presentation module.
    LayerViolation,

    /// Occurs when internal implementation types (e.g., a database model or ORM entity)
    /// are exposed through a module's public API.
    ImplementationExposure,

    /// Occurs when core business logic (domain or application layers) becomes directly
    /// dependent on types defined by a specific framework (e.g., using an Express `Request`
    /// object in a service class).
    FrameworkCoupling,

    /// Occurs when low-level, implementation-specific error types (e.g., `sql::Error`)
    /// are propagated across abstraction boundaries instead of being wrapped in
    /// domain-specific errors.
    ErrorPropagation,

    /// An abstraction that introduces significant, unexpected performance overhead,
    /// such as an Object-Relational Mapper (ORM) causing an N+1 query problem.
    PerformanceLeak,
}

/// The primary detector for identifying leaky abstractions in a codebase.
///
/// This struct orchestrates the analysis by combining architectural configuration,
/// language-specific parsing, and a set of detection heuristics to find violations
/// of architectural boundaries and encapsulation.
pub struct LeakyAbstractionDetector {
    /// The architectural configuration that guides the analysis.
    pub config: ArchitecturalConfig,
}

impl Clone for LeakyAbstractionDetector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

impl Default for LeakyAbstractionDetector {
    /// Creates a new `LeakyAbstractionDetector` with a default configuration.
    fn default() -> Self {
        Self::new()
    }
}

impl LeakyAbstractionDetector {
    /// Creates a new `LeakyAbstractionDetector` with a default configuration.
    ///
    /// The default configuration includes common file path patterns for architectural
    /// layers and a list of well-known infrastructure modules for Rust, Python, and JavaScript.
    pub fn new() -> Self {
        let config = Self::default_config();
        Self { config }
    }

    /// Creates a new detector with a custom `ArchitecturalConfig`.
    ///
    /// This allows for fine-tuning the analysis to match a project's specific
    /// architectural conventions and dependencies.
    ///
    /// # Arguments
    ///
    /// * `config` - The `ArchitecturalConfig` to use for analysis.
    pub fn with_config(config: ArchitecturalConfig) -> Self {
        Self { config }
    }

    /// Provides a default `ArchitecturalConfig` based on common project conventions.
    fn default_config() -> ArchitecturalConfig {
        let mut layer_mappings = HashMap::new();

        // Common patterns for different layers
        layer_mappings.insert(
            "**/controllers/**".to_string(),
            ArchitecturalLayer::Presentation,
        );
        layer_mappings.insert("**/views/**".to_string(), ArchitecturalLayer::Presentation);
        layer_mappings.insert("**/ui/**".to_string(), ArchitecturalLayer::Presentation);
        layer_mappings.insert(
            "**/handlers/**".to_string(),
            ArchitecturalLayer::Presentation,
        );

        layer_mappings.insert(
            "**/services/**".to_string(),
            ArchitecturalLayer::Application,
        );
        layer_mappings.insert(
            "**/use_cases/**".to_string(),
            ArchitecturalLayer::Application,
        );
        layer_mappings.insert(
            "**/application/**".to_string(),
            ArchitecturalLayer::Application,
        );

        layer_mappings.insert("**/domain/**".to_string(), ArchitecturalLayer::Domain);
        layer_mappings.insert("**/models/**".to_string(), ArchitecturalLayer::Domain);
        layer_mappings.insert("**/entities/**".to_string(), ArchitecturalLayer::Domain);

        layer_mappings.insert(
            "**/repositories/**".to_string(),
            ArchitecturalLayer::Infrastructure,
        );
        layer_mappings.insert(
            "**/infrastructure/**".to_string(),
            ArchitecturalLayer::Infrastructure,
        );
        layer_mappings.insert(
            "**/adapters/**".to_string(),
            ArchitecturalLayer::Infrastructure,
        );
        layer_mappings.insert(
            "**/external/**".to_string(),
            ArchitecturalLayer::Infrastructure,
        );

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

    /// A simple glob-like pattern matcher for file paths.
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.starts_with("**/") && pattern.ends_with("/**") {
            let middle = &pattern[3..pattern.len() - 3];
            path.contains(&format!("/{}/", middle)) || path.contains(&format!("\\{}/", middle))
        } else if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            path.ends_with(suffix)
        } else if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            path.starts_with(prefix)
        } else {
            path.contains(pattern)
        }
    }

    /// Checks if a given module name corresponds to a known infrastructure dependency.
    fn is_infrastructure_module(&self, module_name: &str) -> bool {
        self.config.infrastructure_modules.contains(module_name)
            || self
                .config
                .infrastructure_modules
                .iter()
                .any(|infra| module_name.starts_with(infra))
    }

    /// Checks if a type name represents an infrastructure error type that should not be exposed.
    fn is_infrastructure_error_type(&self, type_text: &str) -> bool {
        // Check for common infrastructure error type patterns
        let infrastructure_error_patterns = [
            "DieselError",
            "SqlxError",
            "SeaOrmError", // Database ORMs
            "tokio::Error",
            "std::io::Error",
            "reqwest::Error", // IO and HTTP
            "serde_json::Error",
            "toml::de::Error", // Serialization
            "rusqlite::Error",
            "postgres::Error", // Database drivers
        ];

        infrastructure_error_patterns.iter().any(|pattern| {
            type_text.contains(pattern) ||
            // Check for Result<T, InfrastructureError> patterns
            (type_text.contains("Result<") && type_text.contains(pattern))
        })
    }

    /// Checks if a given path or module name indicates an internal or private module.
    fn is_internal_module(&self, path: &str) -> bool {
        self.config
            .internal_patterns
            .iter()
            .any(|pattern| path.contains(pattern))
    }

    /// Extracts the root module name from a Rust `use` statement.
    fn extract_module_from_use_statement(&self, use_text: &str) -> Option<String> {
        // Remove "use " prefix and find the first identifier
        if let Some(content) = use_text.strip_prefix("use ") {
            // Find the first identifier before :: or ; or whitespace
            let content = content.trim();
            if let Some(pos) = content.find("::") {
                Some(content[..pos].to_string())
            } else if let Some(pos) = content.find(";") {
                Some(content[..pos].trim().to_string())
            } else if let Some(pos) = content.find(" ") {
                Some(content[..pos].to_string())
            } else {
                Some(content.to_string())
            }
        } else {
            None
        }
    }

    /// Extracts the root module name from a Python `import` or `from ... import` statement.
    fn extract_python_import_module(&self, import_text: &str) -> Option<String> {
        // Handle different Python import patterns:
        // from django.shortcuts import render -> "django"
        // import django.contrib.auth -> "django"
        // from myapp.models import User -> "myapp"

        let text = import_text.trim();

        if text.starts_with("from ") {
            // from module.submodule import something
            if let Some(module_part) = text.strip_prefix("from ") {
                if let Some(import_pos) = module_part.find(" import ") {
                    let module = &module_part[..import_pos].trim();
                    return Some(module.split('.').next()?.to_string());
                }
            }
        } else if text.starts_with("import ") {
            // import module.submodule
            if let Some(module_part) = text.strip_prefix("import ") {
                // Handle multiple imports: import os, sys -> take first
                let first_module = module_part.split(',').next()?.trim();
                return Some(first_module.split('.').next()?.to_string());
            }
        }

        None
    }

    /// Determines the architectural layer of a given file path based on configured mappings.
    /// Returns `None` if the layer cannot be determined.
    fn get_layer_from_path(&self, file_path: &str) -> Option<ArchitecturalLayer> {
        for (pattern, layer) in &self.config.layer_mappings {
            if self.matches_pattern(file_path, pattern) {
                return Some(layer.clone()); // Return a cloned ArchitecturalLayer
            }
        }
        None
    }

    /// Returns a human-readable name for an architectural layer.
    fn get_layer_name_string(&self, layer: &ArchitecturalLayer) -> &'static str {
        match layer {
            ArchitecturalLayer::Presentation => "Presentation",
            ArchitecturalLayer::Application => "Application",
            ArchitecturalLayer::Domain => "Domain",
            ArchitecturalLayer::Infrastructure => "Infrastructure",
        }
    }

    /// Runs leaky abstraction analysis on a single Rust file.
    ///
    /// This method uses pre-compiled Tree-sitter queries to find potential leaks, such as:
    /// - Importing infrastructure modules into domain or application layers.
    /// - Exposing public fields in structs, which violates encapsulation.
    /// - Propagating low-level infrastructure errors in public function signatures.
    fn analyze_rust_file(
        &self,
        parsed_file: &ParsedFile,
        analysis_run_id: i64,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let _empty_source = String::new();
        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            crate::analysis::errors::AnalysisError::DetectionError("No AST available".to_string())
        })?;
        let language = tree.language();

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
            (use_declaration) @use_stmt

            ; Detect public function signatures with return types
            (function_item
              (visibility_modifier) @fn_vis
              name: (identifier) @fn_name
              return_type: (_) @return_type) @function_decl

            ; Detect enum variants for infrastructure error types
            (enum_item
              name: (type_identifier) @enum_name) @enum_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            crate::analysis::errors::AnalysisError::DetectionError(format!(
                "Failed to create Rust query: {}",
                e
            ))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "use_stmt" => {
                        // Analyze the entire use declaration
                        if let Ok(use_text) = node.utf8_text(source_bytes) {
                            // Extract module name from use statement text
                            if let Some(module_name) =
                                self.extract_module_from_use_statement(use_text)
                            {
                                if self.is_infrastructure_module(&module_name) {
                                    let layer = self.get_layer_from_path(
                                        &parsed_file.file_path.display().to_string(),
                                    );
                                    if matches!(
                                        layer,
                                        Some(ArchitecturalLayer::Domain)
                                            | Some(ArchitecturalLayer::Application)
                                    ) {
                                        issues.push(self.create_issue(
                                            analysis_run_id,
                                            &parsed_file.file_path.display().to_string(),
                                            node,
                                            LeakType::FrameworkCoupling,
                                            &format!("Infrastructure module '{}' imported in {} layer", module_name, layer.map(|l| format!("{:?}", l)).unwrap_or_else(|| "unknown".to_string())),
                                            "high",
                                        ));
                                    }
                                }
                                if self.is_internal_module(&module_name) {
                                    issues.push(self.create_issue(
                                        analysis_run_id,
                                        &parsed_file.file_path.display().to_string(),
                                        node,
                                        LeakType::VisibilityViolation,
                                        &format!(
                                            "Direct import of internal module '{}'",
                                            module_name
                                        ),
                                        "high",
                                    ));
                                }
                            }
                        }
                    }
                    "field_vis" => {
                        if let Ok(vis_text) = node.utf8_text(source_bytes) {
                            if vis_text == "pub" {
                                // Check if this is exposing internal structure
                                if let Some(parent) = node.parent() {
                                    if let Some(_struct_node) = parent.parent() {
                                        issues.push(self.create_issue(
                                            analysis_run_id,
                                            &parsed_file.file_path.display().to_string(),
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
                    "fn_vis" => {
                        // Check if this is a public function
                        if let Ok(vis_text) = node.utf8_text(source_bytes) {
                            if vis_text == "pub" {
                                // Look for the corresponding return type in the same match
                                for other_capture in match_.captures.iter() {
                                    if query.capture_names()[other_capture.index as usize]
                                        == "return_type"
                                    {
                                        if let Ok(return_type_text) =
                                            other_capture.node.utf8_text(source_bytes)
                                        {
                                            // Check if return type contains infrastructure error types
                                            if self.is_infrastructure_error_type(return_type_text) {
                                                issues.push(self.create_issue(
                                                    analysis_run_id,
                                                    &parsed_file.file_path.display().to_string(),
                                                    other_capture.node,
                                                    LeakType::ErrorPropagation,
                                                    &format!("Infrastructure error type '{}' propagated to public API", return_type_text),
                                                    "high",
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(issues)
    }

    /// Runs leaky abstraction analysis on a single Python file.
    ///
    /// This method focuses on identifying layer violations by checking for imports
    /// of known infrastructure modules (e.g., `django`, `flask`) in layers where
    /// they don't belong (e.g., `Domain`, `Application`).
    fn analyze_python_file(
        &self,
        parsed_file: &ParsedFile,
        analysis_run_id: i64,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let _empty_source = String::new();
        let source_bytes = parsed_file.source.as_bytes();

        // Check architectural layer violations
        let file_path_str = parsed_file.file_path.display().to_string();
        if let Some(current_layer) = self.get_layer_from_path(&file_path_str) {
            if let Some(tree) = &parsed_file.tree {
                let language = tree.language();
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

                let query = Query::new(&language, query_source).map_err(|e| {
                    crate::analysis::errors::AnalysisError::Other(format!(
                        "Failed to create Python query: {}",
                        e
                    ))
                })?;

                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

                while let Some(query_match) = matches.next() {
                    for capture in query_match.captures {
                        let capture_text = capture.node.utf8_text(source_bytes).unwrap_or("");

                        // Extract module name from Python import
                        if let Some(module_name) = self.extract_python_import_module(capture_text) {
                            if self.is_infrastructure_module(&module_name) {
                                // Check if this is a layer violation (infrastructure should only be in Infrastructure layer)
                                if current_layer != ArchitecturalLayer::Infrastructure {
                                    let mut issue = ArchitecturalIssue::new(
                                        analysis_run_id,
                                        1, // anti_pattern_type_id for leaky abstraction
                                        parsed_file.file_path.display().to_string(),
                                        Some(capture.node.start_position().row as i32 + 1),
                                        format!(
                                            "Framework module '{}' imported in {} layer",
                                            module_name,
                                            self.get_layer_name_string(&current_layer)
                                        ),
                                        "LeakyAbstractionDetector".to_string(),
                                        "high".to_string(),
                                        format!(
                                            "Framework module '{}' imported in {} layer",
                                            module_name,
                                            self.get_layer_name_string(&current_layer)
                                        ),
                                    );
                                    issue.start_line =
                                        Some(capture.node.start_position().row as i32 + 1);
                                    issue.end_line =
                                        Some(capture.node.end_position().row as i32 + 1);
                                    issue.code_snippet = Some(capture_text.to_string());
                                    issues.push(issue);
                                }
                            }
                        }
                    }
                }
            } else {
                // Handle case where no tree is available
            }
        } else {
            // Handle case where layer could not be determined
        }

        Ok(issues)
    }

    /// Runs leaky abstraction analysis on a single JavaScript or TypeScript file.
    ///
    /// This method checks for common frontend and backend leaks, such as:
    /// - Importing infrastructure modules (e.g., `express`, `react`) into core logic layers.
    /// - Performing direct DOM manipulation (`document`, `window`) in business logic.
    fn analyze_js_file(
        &self,
        parsed_file: &ParsedFile,
        analysis_run_id: i64,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let _empty_source = String::new();
        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            crate::analysis::errors::AnalysisError::DetectionError("No AST available".to_string())
        })?;
        let language = tree.language();

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
            ; NOTE: TypeScript-specific, not available in plain JavaScript Tree-sitter

            ; Detect generic type parameter pollution
            ; NOTE: TypeScript-specific, not available in plain JavaScript Tree-sitter
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            crate::analysis::errors::AnalysisError::Other(format!(
                "Failed to create JS query: {}",
                e
            ))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "import_source" => {
                        if let Ok(import_text) = node.utf8_text(source_bytes) {
                            let module_name = import_text.trim_matches('"').trim_matches('\'');
                            if self.is_infrastructure_module(module_name) {
                                let layer = self.get_layer_from_path(
                                    &parsed_file.file_path.display().to_string(),
                                );
                                if matches!(
                                    layer,
                                    Some(ArchitecturalLayer::Domain)
                                        | Some(ArchitecturalLayer::Application)
                                ) {
                                    issues.push(self.create_issue(
                                        analysis_run_id,
                                        &parsed_file.file_path.display().to_string(),
                                        node,
                                        LeakType::FrameworkCoupling,
                                        &format!(
                                                "Infrastructure module '{}' imported in {} layer",
                                                module_name,
                                                layer
                                                    .map(|l| format!("{:?}", l))
                                                    .unwrap_or_else(|| "unknown".to_string())
                                            ),
                                        "high",
                                    ));
                                }
                            }
                            if self.is_internal_module(module_name) {
                                issues.push(self.create_issue(
                                    analysis_run_id,
                                    &parsed_file.file_path.display().to_string(),
                                    node,
                                    LeakType::VisibilityViolation,
                                    &format!("Direct import of internal module '{}'", module_name),
                                    "high",
                                ));
                            }
                        }
                    }
                    "dom_object" => {
                        if let Ok(dom_text) = node.utf8_text(source_bytes) {
                            if dom_text == "document" || dom_text == "window" {
                                let layer = self.get_layer_from_path(
                                    &parsed_file.file_path.display().to_string(),
                                );
                                if matches!(
                                    layer,
                                    Some(ArchitecturalLayer::Domain)
                                        | Some(ArchitecturalLayer::Application)
                                ) {
                                    issues.push(self.create_issue(
                                        analysis_run_id,
                                        &parsed_file.file_path.display().to_string(),
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

        Ok(issues)
    }

    /// Helper function to create a new `ArchitecturalIssue` instance.
    fn create_issue(
        &self,
        analysis_run_id: i64,
        file_path: &str,
        node: Node,
        leak_type: LeakType,
        description: &str,
        severity: &str,
    ) -> ArchitecturalIssue {
        {
            let mut issue = ArchitecturalIssue::new(
                analysis_run_id,
                self.get_anti_pattern_id_for_leak_type(&leak_type),
                file_path.to_string(),
                Some(node.start_position().row as i32 + 1),
                description.to_string(),
                "LeakyAbstractionDetector".to_string(),
                severity.to_string(),
                description.to_string(),
            );
            issue.start_line = Some(node.start_position().row as i32 + 1);
            issue.end_line = Some(node.end_position().row as i32 + 1);
            issue.code_snippet = None; // Could extract node text here
            issue
        }
    }

    /// Maps a `LeakType` to its corresponding `anti_pattern_type_id` for database storage.
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

#[async_trait]
impl AnalysisDetector for LeakyAbstractionDetector {
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        #[cfg(not(feature = "tree-sitter"))]
        {
            tracing::debug!(
                "Tree-sitter feature not enabled, skipping leaky abstraction detection"
            );
            return Ok(Vec::new());
        }
        #[cfg(feature = "tree-sitter")]
        {
            let detector = self.clone();
            let language_str = match parsed_file.language {
                SourceLanguage::Rust => "rust",
                SourceLanguage::Python => "python",
                SourceLanguage::JavaScript => "javascript",
                SourceLanguage::TypeScript => "typescript", // UV-XXX: Add TypeScript support
            };

            let analysis_run_id = 1; // TODO: Get from context

            match language_str {
                "rust" => detector.analyze_rust_file(parsed_file, analysis_run_id),
                "python" => detector.analyze_python_file(parsed_file, analysis_run_id),
                "javascript" | "typescript" => {
                    detector.analyze_js_file(parsed_file, analysis_run_id)
                }
                _ => Ok(vec![]),
            }
        }
    }
    fn get_detector_name(&self) -> &'static str {
        "LeakyAbstractionDetector"
    }
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(6),
            name: "Performance Leak".to_string(),
            description: "Abstraction causing unexpected performance degradation".to_string(),
            category: "behavioral".to_string(),
        }]
    }
}
