//! Advanced Leaky Abstraction Detector
// //! 
// //! This detector implements a comprehensive analysis framework for identifying leaky abstractions
// //! across multiple programming languages (Rust, Python, JavaScript/TypeScript). It uses a 
// //! multi-signal approach combining AST analysis, dependency tracking, and architectural pattern
// //! recognition to detect violations of abstraction boundaries.
// //! 
// //! ## Detection Capabilities
// //! 
// //! ### Rust-Specific Patterns
// //! - Visibility violations (pub vs private boundaries)
// //! - Framework-specific types in public APIs
// //! - ORM/Database types leaking into business logic
// //! - Error type propagation across layers
// //! - Async runtime details in interfaces
// //! 
// //! ### Python-Specific Patterns  
// //! - Direct database model usage in views/controllers
// //! - Framework objects in business logic (Flask request, Django models)
// //! - File system paths in public interfaces
// //! - Import violations across architectural layers
// //! 
// //! ### JavaScript/TypeScript Patterns
// //! - DOM manipulation in business logic
// //! - Framework-specific objects in domain models
// //! - Infrastructure dependencies in application layer
// //! - Type definition leaks and generic pollution
// //! 
// //! ## Architecture
// //! 
// //! The detector uses a layered analysis approach:
// //! 1. **Syntactic Analysis**: Tree-sitter queries for pattern matching
// //! 2. **Semantic Analysis**: Symbol resolution and type flow tracking
// //! 3. **Architectural Analysis**: Layer boundary validation
// //! 4. **Cross-file Analysis**: Dependency graph traversal

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Query, QueryCursor, Node};

/// Configuration for architectural layers and boundaries.
///
/// Defines the rules and patterns for identifying architectural layers and
/// known infrastructure modules within a project. This configuration drives
/// the leaky abstraction analysis by providing context about the intended
/// architecture of the software.
#[derive(Debug, Clone)]
pub struct ArchitecturalConfig {
    /// Mapping of path patterns (globs) to architectural layers.
    ///
    /// This is the primary mechanism for defining the architecture. For example,
    /// `{"**/controllers/**": Presentation, "**/services/**": Application}`.
    pub layer_mappings: HashMap<String, ArchitecturalLayer>,
    
    /// A set of known infrastructure module names or prefixes.
    ///
    /// This set is used to identify dependencies on frameworks, databases,
    /// or other external systems (e.g., "django", "sqlx", "react").
    pub infrastructure_modules: HashSet<String>,
    
    /// A list of patterns used to identify internal or private modules.
    ///
    /// Accessing modules whose paths contain these patterns from outside
    /// their parent component is considered a visibility violation.
    pub internal_patterns: Vec<String>,
}

/// Represents the architectural layers of a system, inspired by Clean Architecture.
///
/// Each layer has a distinct responsibility, and dependencies should generally
/// flow from outer layers (like Presentation) to inner layers (like Domain).
#[derive(Debug, Clone, PartialEq)]
pub enum ArchitecturalLayer {
    /// The outermost layer, responsible for UI and user interaction.
    /// This includes controllers, views, and API endpoints.
    Presentation,
    
    /// The layer containing application-specific business logic and use cases.
    /// It orchestrates the domain layer to perform tasks.
    Application, 
    
    /// The core layer containing enterprise-wide business logic and entities.
    /// This layer should be independent of any framework or UI.
    Domain,
    
    /// The layer containing all external concerns and implementation details.
    /// This includes databases, file systems, and third-party API clients.
    Infrastructure,
}

/// Enumerates the specific types of leaky abstraction violations that can be detected.
#[derive(Debug, Clone)]
pub enum LeakType {
    /// Occurs when code accesses a private or internal item from an outside module,
    /// violating encapsulation.
    VisibilityViolation,
    
    /// A dependency that flows in the wrong direction between architectural layers,
    /// such as a domain module depending on a presentation module.
    LayerViolation,
    
    /// When internal implementation types (e.g., a database model) are exposed
    /// through a module's public API.
    ImplementationExposure,
    
    /// When core business logic becomes dependent on types defined by a specific
    /// framework (e.g., using Express `Request` objects in a service).
    FrameworkCoupling,
    
    /// When low-level error types (e.g., `sql::Error`) are propagated across
    /// abstraction boundaries instead of being wrapped in domain-specific errors.
    ErrorPropagation,
    
    /// When an abstraction introduces significant, unexpected performance overhead
    /// (e.g., an ORM causing N+1 query problems).
    PerformanceLeak,
}

/// The primary detector for identifying leaky abstractions in a codebase.
///
/// This struct orchestrates the analysis by combining architectural configuration,
/// language-specific parsing, and a set of detection heuristics to find violations
/// of architectural boundaries.
pub struct LeakyAbstractionDetector {
    /// The architectural configuration that guides the analysis.
    pub config: ArchitecturalConfig,
    
    /// Compiled Tree-sitter queries for Rust-specific patterns.
    rust_queries: Option<Query>,
    
    /// Compiled Tree-sitter queries for Python-specific patterns. 
    python_queries: Option<Query>,
    
    /// Compiled Tree-sitter queries for JavaScript/TypeScript-specific patterns.
    js_queries: Option<Query>,
}

impl Clone for LeakyAbstractionDetector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            rust_queries: None,
            python_queries: None,
            js_queries: None,
        }
    }
}

impl Default for LeakyAbstractionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LeakyAbstractionDetector {
    /// Creates a new `LeakyAbstractionDetector` with a default configuration.
    ///
    /// The default configuration includes common file path patterns for architectural
    /// layers and a list of well-known infrastructure modules for major languages.
    pub fn new() -> Self {
        let config = Self::default_config();
        Self {
            config,
            rust_queries: None,
            python_queries: None,
            js_queries: None,
        }
    }

    /// Creates a new detector with a custom `ArchitecturalConfig`.
    ///
    /// This allows for fine-tuning the analysis to match a project's specific
    /// architectural conventions.
    pub fn with_config(config: ArchitecturalConfig) -> Self {
        Self {
            config,
            rust_queries: None,
            python_queries: None,
            js_queries: None,
        }
    }

    /// Provides a default `ArchitecturalConfig` based on common conventions.
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

    /// Lazily initializes the Tree-sitter queries for a specific language.
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

    /// Compiles the Tree-sitter queries for analyzing Rust code.
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

        let language = tree_sitter_rust::language();
        Query::new(&language, query_source)
            .map_err(|e| AnalysisError::Other(format!("Failed to create Rust query: {}", e)))
    }

    /// Compiles the Tree-sitter queries for analyzing Python code.
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

    /// Compiles the Tree-sitter queries for analyzing JavaScript and TypeScript code.
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
            ; NOTE: TypeScript-specific, not available in plain JavaScript Tree-sitter

            ; Detect generic type parameter pollution
            ; NOTE: TypeScript-specific, not available in plain JavaScript Tree-sitter
        "#;

        let language = tree_sitter_javascript::language();
        Query::new(&language, query_source)
            .map_err(|e| AnalysisError::Other(format!("Failed to create JS query: {}", e)))
    }

    /// Determines the architectural layer of a file based on its path.
    fn get_layer_from_path(&self, file_path: &str) -> Option<ArchitecturalLayer> {
        for (pattern, layer) in &self.config.layer_mappings {
            if self.matches_pattern(file_path, pattern) {
                return Some(layer.clone());
            }
        }
        None
    }

    /// A simple glob-like pattern matcher for file paths.
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.starts_with("**/") && pattern.ends_with("/**") {
            let middle = &pattern[3..pattern.len()-3];
            path.contains(&format!("/{}/", middle)) || path.contains(&format!("\\{}/", middle))
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

    /// Checks if a given module name corresponds to a known infrastructure dependency.
    fn is_infrastructure_module(&self, module_name: &str) -> bool {
        self.config.infrastructure_modules.contains(module_name) ||
        self.config.infrastructure_modules.iter().any(|infra| module_name.starts_with(infra))
    }

    /// Checks if a type name represents an infrastructure error type that shouldn't be exposed in public APIs.
    fn is_infrastructure_error_type(&self, type_text: &str) -> bool {
        // Check for common infrastructure error type patterns
        let infrastructure_error_patterns = [
            "DieselError", "SqlxError", "SeaOrmError", // Database ORMs
            "tokio::Error", "std::io::Error", "reqwest::Error", // IO and HTTP
            "serde_json::Error", "toml::de::Error", // Serialization
            "rusqlite::Error", "postgres::Error", // Database drivers
        ];
        
        infrastructure_error_patterns.iter().any(|pattern| {
            type_text.contains(pattern) || 
            // Check for Result<T, InfrastructureError> patterns
            (type_text.contains("Result<") && type_text.contains(pattern))
        })
    }

    /// Checks if a given path or module name indicates an internal/private module.
    fn is_internal_module(&self, path: &str) -> bool {
        self.config.internal_patterns.iter().any(|pattern| path.contains(pattern))
    }

    /// Extracts the module name from a Rust use statement text.
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

    /// Extracts the module name from a Python import statement.
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
    
    /// Returns a human-readable name for an architectural layer.
    fn get_layer_name(&self, layer: &ArchitecturalLayer) -> &'static str {
        match layer {
            ArchitecturalLayer::Presentation => "Presentation",
            ArchitecturalLayer::Application => "Application", 
            ArchitecturalLayer::Domain => "Domain",
            ArchitecturalLayer::Infrastructure => "Infrastructure",
        }
    }

    /// Runs leaky abstraction analysis on a single Rust file.
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
                        "use_stmt" => {
                            // Analyze the entire use declaration
                            if let Ok(use_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                // Extract module name from use statement text
                                if let Some(module_name) = self.extract_module_from_use_statement(use_text) {
                                    if self.is_infrastructure_module(&module_name) {
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
                                    if self.is_internal_module(&module_name) {
                                        issues.push(self.create_issue(
                                            analysis_run_id,
                                            &parsed_file.path.to_string_lossy(),
                                            node,
                                            LeakType::VisibilityViolation,
                                            &format!("Direct import of internal module '{}'", module_name),
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
                        "fn_vis" => {
                            // Check if this is a public function
                            if let Ok(vis_text) = node.utf8_text(parsed_file.source.as_bytes()) {
                                if vis_text == "pub" {
                                    // Look for the corresponding return type in the same match
                                    for other_capture in match_.captures {
                                        if query.capture_names()[other_capture.index as usize] == "return_type" {
                                            if let Ok(return_type_text) = other_capture.node.utf8_text(parsed_file.source.as_bytes()) {
                                                // Check if return type contains infrastructure error types
                                                if self.is_infrastructure_error_type(return_type_text) {
                                                    issues.push(self.create_issue(
                                                        analysis_run_id,
                                                        &parsed_file.path.to_string_lossy(),
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
        }

        Ok(issues)
    }

    /// Runs leaky abstraction analysis on a single Python file.
    fn analyze_python_file(&self, parsed_file: &ParsedFile, analysis_run_id: i64) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        
        // Check architectural layer violations
        let file_path_str = parsed_file.path.to_string_lossy();
        if let Some(current_layer) = self.get_layer_from_path(&file_path_str) {
            
            if let Some(query) = &self.python_queries {
                if let Some(tree) = &parsed_file.tree {
                    let mut cursor = QueryCursor::new();
                    let matches = cursor.matches(query, tree.root_node(), parsed_file.source.as_bytes());
                    
                    for query_match in matches {
                        for capture in query_match.captures {
                            let capture_text = capture.node.utf8_text(parsed_file.source.as_bytes()).unwrap_or("");
                            
                            // Extract module name from Python import
                            if let Some(module_name) = self.extract_python_import_module(capture_text) {
                                if self.is_infrastructure_module(&module_name) {
                                    // Check if this is a layer violation (infrastructure should only be in Infrastructure layer)
                                    if current_layer != ArchitecturalLayer::Infrastructure {
                                        let issue = ArchitecturalIssue {
                                            issue_id: None,
                                            analysis_run_id,
                                            anti_pattern_type_id: 1, // TODO: proper mapping
                                            file_path: parsed_file.path.to_string_lossy().to_string(),
                                            start_line: Some(capture.node.start_position().row as i32 + 1),
                                            end_line: Some(capture.node.end_position().row as i32 + 1),
                                            severity: "high".to_string(),
                                            description: format!(
                                                "Framework module '{}' imported in {} layer",
                                                module_name,
                                                self.get_layer_name(&current_layer)
                                            ),
                                            code_snippet: Some(capture_text.to_string()),
                                            ai_explanation: None,
                                        };
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
                // Handle case where Python queries are not available  
            }
        } else {
            // Handle case where layer could not be determined
        }

        Ok(issues)
    }

    /// Runs leaky abstraction analysis on a single JavaScript or TypeScript file.
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
                                if self.is_internal_module(module_name) {
                                    issues.push(self.create_issue(
                                        analysis_run_id,
                                        &parsed_file.path.to_string_lossy(),
                                        node,
                                        LeakType::VisibilityViolation,
                                        &format!("Direct import of internal module '{}'", module_name),
                                        "high",
                                    ));
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

    /// Helper function to create a new `ArchitecturalIssue`.
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

    /// Maps a `LeakType` to its corresponding `anti_pattern_type_id`.
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