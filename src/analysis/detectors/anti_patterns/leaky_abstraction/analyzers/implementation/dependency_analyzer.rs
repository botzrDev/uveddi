//! Dependency coupling analysis for implementation leaks.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ImplementationExposure, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes dependency coupling for implementation leaks.
#[derive(Clone)]
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// Creates a new dependency analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes Python dependency coupling.
    pub fn analyze_python_dependencies(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationExposure>, AnalysisError> {
        let mut implementation_exposures = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect class inheritance from infrastructure classes
            (class_definition
              name: (identifier) @class_name
              superclasses: (argument_list
                (identifier) @parent_class*)) @class_def

            ; Detect import statements that may expose dependencies
            (import_statement
              name: (dotted_name
                (identifier) @module_name)) @import_stmt

            (import_from_statement
              module_name: (dotted_name
                (identifier) @from_module)
              name: (dotted_name
                (identifier) @import_name)) @from_import
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!(
                "Failed to create Python dependency query: {}",
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
                    "parent_class" => {
                        if let Ok(parent_class) = node.utf8_text(source_bytes) {
                            if self.is_infrastructure_class(parent_class) {
                                implementation_exposures.push(ImplementationExposure {
                                    description: format!(
                                        "Class inherits from infrastructure class '{}'",
                                        parent_class
                                    ),
                                    exposed_detail: parent_class.to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "high".to_string(),
                                });
                            }
                        }
                    }
                    "module_name" | "from_module" => {
                        if let Ok(module_name) = node.utf8_text(source_bytes) {
                            if self.is_infrastructure_module(module_name) {
                                implementation_exposures.push(ImplementationExposure {
                                    description: format!(
                                        "Direct dependency on infrastructure module '{}'",
                                        module_name
                                    ),
                                    exposed_detail: module_name.to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "medium".to_string(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(implementation_exposures)
    }

    /// Analyzes JavaScript/TypeScript dependency coupling.
    pub fn analyze_js_dependencies(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationExposure>, AnalysisError> {
        let mut implementation_exposures = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect import statements
            (import_statement
              source: (string) @import_source) @import_stmt

            ; Detect variable declarations with infrastructure objects
            (variable_declaration
              declarations: (variable_declarator
                name: (identifier) @var_name
                value: (_) @var_value)) @var_decl

            ; Detect class extensions
            (class_declaration
              name: (identifier) @class_name
              heritage: (class_heritage
                (extends_clause
                  value: (identifier) @extends_class))?) @class_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create JS dependency query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "import_source" => {
                        if let Ok(import_text) = node.utf8_text(source_bytes) {
                            let module_name = import_text.trim_matches('"').trim_matches('\'');
                            if self.is_infrastructure_module(module_name) {
                                implementation_exposures.push(ImplementationExposure {
                                    description: format!(
                                        "Direct dependency on infrastructure module '{}'",
                                        module_name
                                    ),
                                    exposed_detail: module_name.to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "medium".to_string(),
                                });
                            }
                        }
                    }
                    "extends_class" => {
                        if let Ok(extends_class) = node.utf8_text(source_bytes) {
                            if self.is_framework_class(extends_class) {
                                implementation_exposures.push(ImplementationExposure {
                                    description: format!(
                                        "Class extends framework class '{}'",
                                        extends_class
                                    ),
                                    exposed_detail: extends_class.to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "high".to_string(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(implementation_exposures)
    }

    /// Checks if a class name represents an infrastructure class.
    fn is_infrastructure_class(&self, class_name: &str) -> bool {
        let infrastructure_classes = [
            "Model",
            "View",
            "Serializer",
            "Form",
            "Request",
            "Response",
            "HttpRequest",
            "HttpResponse",
            "Connection",
            "Session",
            "Transaction",
            "Component",
            "Widget",
            "Handler",
        ];

        infrastructure_classes
            .iter()
            .any(|pattern| class_name.contains(pattern))
    }

    /// Checks if a module is considered infrastructure.
    fn is_infrastructure_module(&self, module_name: &str) -> bool {
        let infrastructure_modules = [
            "django",
            "flask",
            "fastapi",
            "tornado",
            "sqlalchemy",
            "peewee",
            "mongoengine",
            "requests",
            "urllib",
            "http",
            "express",
            "koa",
            "fastify",
            "nest",
            "mongoose",
            "sequelize",
            "typeorm",
            "prisma",
            "react",
            "vue",
            "angular",
            "next",
        ];

        infrastructure_modules
            .iter()
            .any(|pattern| module_name.starts_with(pattern))
    }

    /// Checks if a class represents a framework class.
    fn is_framework_class(&self, class_name: &str) -> bool {
        let framework_classes = [
            "Component",
            "Controller",
            "Service",
            "Repository",
            "Model",
            "Entity",
            "Document",
            "Schema",
            "Middleware",
            "Guard",
            "Interceptor",
            "Pipe",
        ];

        framework_classes
            .iter()
            .any(|pattern| class_name.contains(pattern))
    }

    /// Analyzes coupling strength between components.
    pub fn analyze_coupling_strength(&self, dependencies: &[String]) -> f64 {
        let infrastructure_deps = dependencies
            .iter()
            .filter(|dep| self.is_infrastructure_module(dep))
            .count();

        let total_deps = dependencies.len();
        if total_deps == 0 {
            return 0.0;
        }

        infrastructure_deps as f64 / total_deps as f64
    }

    /// Provides recommendations for reducing coupling.
    pub fn suggest_decoupling_strategies(
        &self,
        exposures: &[ImplementationExposure],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        for exposure in exposures {
            if exposure.exposed_detail.contains("Model")
                || exposure.exposed_detail.contains("Entity")
            {
                suggestions.push(
                    "Consider using Data Transfer Objects (DTOs) instead of direct model usage"
                        .to_string(),
                );
            }

            if exposure.exposed_detail.contains("Request")
                || exposure.exposed_detail.contains("Response")
            {
                suggestions
                    .push("Abstract HTTP concerns behind domain-specific interfaces".to_string());
            }

            if exposure.exposed_detail.contains("Connection")
                || exposure.exposed_detail.contains("Session")
            {
                suggestions
                    .push("Use repository pattern to abstract data access details".to_string());
            }
        }

        if suggestions.is_empty() {
            suggestions.push(
                "Consider using dependency injection to invert control dependencies".to_string(),
            );
        }

        suggestions
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
