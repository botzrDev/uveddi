//! Python import and module boundary analysis for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes Python import patterns and module boundaries for abstraction leaks.
pub struct ImportAnalyzer;

impl ImportAnalyzer {
    /// Creates a new import analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes import violation patterns.
    pub fn analyze_import_violations(
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
            ; Detect import statements
            (import_statement
              name: (dotted_name
                (identifier) @module_name)) @import_stmt

            ; Detect from...import statements
            (import_from_statement
              module_name: (dotted_name
                (identifier) @from_module)
              name: (dotted_name
                (identifier) @import_name)) @from_import
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Python import query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "module_name" | "from_module" => {
                        if let Ok(module_name) = node.utf8_text(source_bytes) {
                            if self.is_infrastructure_module(module_name) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!(
                                        "Infrastructure module '{}' imported in business layer",
                                        module_name
                                    ),
                                    node.start_position().row as u32 + 1,
                                    LeakType::FrameworkCoupling,
                                    "high",
                                ));
                            }
                            if self.is_internal_module(module_name) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!("Direct import of internal module '{}'", module_name),
                                    node.start_position().row as u32 + 1,
                                    LeakType::VisibilityViolation,
                                    "high",
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
            "tkinter",
            "wx",
            "qt",
        ];

        infrastructure_modules
            .iter()
            .any(|pattern| module_name.starts_with(pattern))
    }

    /// Checks if a module is considered internal.
    fn is_internal_module(&self, module_name: &str) -> bool {
        module_name.contains("_internal")
            || module_name.contains(".internal")
            || module_name.contains("_impl")
            || module_name.contains(".impl")
    }

    /// Extracts the module name from a Python import statement.
    pub fn extract_python_import_module(&self, import_text: &str) -> Option<String> {
        let text = import_text.trim();

        if text.starts_with("from ") {
            if let Some(module_part) = text.strip_prefix("from ") {
                if let Some(import_pos) = module_part.find(" import ") {
                    let module = &module_part[..import_pos].trim();
                    return Some(module.split('.').next()?.to_string());
                }
            }
        } else if text.starts_with("import ") {
            if let Some(module_part) = text.strip_prefix("import ") {
                let first_module = module_part.split(',').next()?.trim();
                return Some(first_module.split('.').next()?.to_string());
            }
        }

        None
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
            "PythonImportAnalyzer".to_string(),
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

impl Default for ImportAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
