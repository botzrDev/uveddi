//! Python class and method analysis for leaky abstraction detection.

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::database::models::ArchitecturalIssue;
use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType
};

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes Python class structures and method visibility for abstraction leaks.
pub struct ClassAnalyzer;

impl ClassAnalyzer {
    /// Creates a new class analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes class internal exposure patterns.
    pub fn analyze_class_internals(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::DetectionError("No AST available".to_string())
        })?;
        let language = tree.language();

        let query_source = r#"
            ; Detect class definitions with potential internal exposure
            (class_definition
              name: (identifier) @class_name
              superclasses: (argument_list
                (identifier) @parent_class*)?
              body: (block
                (function_definition
                  name: (identifier) @method_name)*
                (expression_statement
                  (assignment
                    left: (identifier) @attr_name
                    right: (_) @attr_value))*)) @class_def

            ; Detect attribute access patterns
            (attribute
              object: (identifier) @obj_name
              attribute: (identifier) @attr_name) @attr_access
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Python class query: {}", e))
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
                                issues.push(self.create_issue(
                                    context,
                                    &format!("Class inherits from infrastructure class '{}'", parent_class),
                                    node.start_position().row as u32 + 1,
                                    LeakType::FrameworkCoupling,
                                    "high",
                                ));
                            }
                        }
                    }
                    "attr_name" => {
                        if let Ok(attr_name) = node.utf8_text(source_bytes) {
                            if self.is_private_attribute_access(attr_name) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!("Access to private attribute '{}'", attr_name),
                                    node.start_position().row as u32 + 1,
                                    LeakType::VisibilityViolation,
                                    "medium",
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(issues)
    }

    /// Analyzes private method exposure patterns.
    pub fn analyze_private_methods(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::DetectionError("No AST available".to_string())
        })?;
        let language = tree.language();

        let query_source = r#"
            ; Detect function calls to private methods
            (call
              function: (attribute
                object: (identifier) @obj_name
                attribute: (identifier) @method_name)) @method_call

            ; Detect function definitions
            (function_definition
              name: (identifier) @func_name
              parameters: (parameters
                (identifier) @param_name*)) @func_def
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Python method query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                if capture_name == "method_name" {
                    if let Ok(method_name) = node.utf8_text(source_bytes) {
                        if self.is_private_method(method_name) {
                            issues.push(self.create_issue(
                                context,
                                &format!("Call to private method '{}'", method_name),
                                node.start_position().row as u32 + 1,
                                LeakType::VisibilityViolation,
                                "medium",
                            ));
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Checks if an attribute access is to a private attribute.
    fn is_private_attribute_access(&self, attr_name: &str) -> bool {
        attr_name.starts_with('_') && !attr_name.starts_with("__")
    }

    /// Checks if a method name indicates a private method.
    fn is_private_method(&self, method_name: &str) -> bool {
        method_name.starts_with('_') && !method_name.starts_with("__")
    }

    /// Checks if a class name represents an infrastructure class.
    fn is_infrastructure_class(&self, class_name: &str) -> bool {
        let infrastructure_classes = [
            "Model", "View", "Serializer", "Form",
            "Request", "Response", "HttpRequest", "HttpResponse",
            "Connection", "Session", "Transaction",
            "Component", "Widget", "Handler",
        ];

        infrastructure_classes.iter().any(|pattern| class_name.contains(pattern))
    }

    /// Helper function to create an architectural issue.
    fn create_issue(
        &self,
        context: &AnalysisContext,
        description: &str,
        line_number: u32,
        leak_type: LeakType,
        severity: &str,
    ) -> ArchitecturalIssue {
        let mut issue = ArchitecturalIssue::new(
            context.analysis_run_id,
            self.get_anti_pattern_id_for_leak_type(&leak_type),
            context.file_path.clone(),
            Some(line_number as i32),
            description.to_string(),
            "PythonClassAnalyzer".to_string(),
            severity.to_string(),
            description.to_string(),
        );
        issue.start_line = Some(line_number as i32);
        issue.end_line = Some(line_number as i32);
        issue
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

impl Default for ClassAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}