//! Python framework coupling and decorator analysis for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes Python framework coupling patterns and decorator usage for abstraction leaks.
pub struct FrameworkAnalyzer;

impl FrameworkAnalyzer {
    /// Creates a new framework analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes framework coupling patterns.
    pub fn analyze_framework_coupling(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect direct usage of framework objects
            (call
              function: (attribute
                object: (identifier) @framework_obj
                attribute: (identifier) @framework_method)) @framework_call

            ; Detect decorator usage
            (decorated_definition
              (decorator
                (identifier) @decorator_name)
              definition: (_) @decorated_item) @decorator_usage
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Python framework query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "framework_obj" => {
                        if let Ok(obj_name) = node.utf8_text(source_bytes) {
                            if self.is_framework_object(obj_name) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!("Direct usage of framework object '{}'", obj_name),
                                    node.start_position().row as u32 + 1,
                                    LeakType::FrameworkCoupling,
                                    "high",
                                ));
                            }
                        }
                    }
                    "decorator_name" => {
                        if let Ok(decorator_name) = node.utf8_text(source_bytes) {
                            if self.is_framework_decorator(decorator_name) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!(
                                        "Framework decorator '{}' used in business logic",
                                        decorator_name
                                    ),
                                    node.start_position().row as u32 + 1,
                                    LeakType::FrameworkCoupling,
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

    /// Checks if an object name represents a framework object.
    fn is_framework_object(&self, obj_name: &str) -> bool {
        let framework_objects = [
            "request",
            "response",
            "session",
            "connection",
            "db",
            "cursor",
            "query",
            "model",
            "app",
            "client",
            "server",
            "socket",
        ];

        framework_objects
            .iter()
            .any(|pattern| obj_name.contains(pattern))
    }

    /// Checks if a decorator name represents a framework decorator.
    fn is_framework_decorator(&self, decorator_name: &str) -> bool {
        let framework_decorators = [
            "app.route",
            "login_required",
            "csrf_exempt",
            "cache_page",
            "transaction",
            "atomic",
            "api_view",
            "permission_classes",
        ];

        framework_decorators
            .iter()
            .any(|pattern| decorator_name.contains(pattern))
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
            "PythonFrameworkAnalyzer".to_string(),
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

impl Default for FrameworkAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
