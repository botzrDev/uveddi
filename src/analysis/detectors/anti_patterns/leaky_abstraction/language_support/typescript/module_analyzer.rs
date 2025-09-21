//! TypeScript module boundary analysis for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes TypeScript module boundaries for abstraction leaks.
pub struct ModuleAnalyzer;

impl ModuleAnalyzer {
    /// Creates a new module analyzer.
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
              source: (string) @import_source) @import_stmt

            ; Detect import clause with specifiers
            (import_statement
              import: (import_clause
                (named_imports
                  (import_specifier
                    name: (identifier) @import_name)*))
              source: (string) @named_import_source) @named_import_stmt

            ; Detect dynamic imports
            (call_expression
              function: (identifier) @import_func
              arguments: (arguments
                (string) @dynamic_import_source)) @dynamic_import
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!(
                "Failed to create TypeScript import query: {}",
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
                    "import_source" | "named_import_source" | "dynamic_import_source" => {
                        if let Ok(import_source) = node.utf8_text(source_bytes) {
                            let module_name = import_source.trim_matches('"').trim_matches('\'');
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
                    "import_func" => {
                        if let Ok(func_name) = node.utf8_text(source_bytes) {
                            if func_name == "import" {
                                // Handle dynamic import analysis
                                // Additional logic could be added here
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(issues)
    }

    /// Analyzes export patterns for potential leaks.
    pub fn analyze_export_patterns(
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
            ; Detect export statements
            (export_statement
              declaration: (_) @exported_decl) @export_stmt

            ; Detect re-exports
            (export_statement
              source: (string) @reexport_source) @reexport_stmt

            ; Detect default exports
            (export_statement
              default: "default"
              declaration: (_) @default_export) @default_export_stmt
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!(
                "Failed to create TypeScript export query: {}",
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
                    "reexport_source" => {
                        if let Ok(export_source) = node.utf8_text(source_bytes) {
                            let module_name = export_source.trim_matches('"').trim_matches('\'');
                            if self.is_internal_module(module_name) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!("Re-export exposes internal module '{}'", module_name),
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

    /// Checks if a module is considered infrastructure.
    fn is_infrastructure_module(&self, module_name: &str) -> bool {
        let infrastructure_modules = [
            "express",
            "koa",
            "fastify",
            "nest",
            "mongoose",
            "sequelize",
            "typeorm",
            "prisma",
            "socket.io",
            "ws",
            "redis",
            "aws-sdk",
            "react",
            "vue",
            "angular",
            "next",
        ];

        infrastructure_modules
            .iter()
            .any(|pattern| module_name.starts_with(pattern) || module_name.contains(pattern))
    }

    /// Checks if a module is considered internal.
    fn is_internal_module(&self, module_name: &str) -> bool {
        module_name.contains("/internal/")
            || module_name.contains("\\internal\\")
            || module_name.contains("/impl/")
            || module_name.contains("\\impl\\")
            || module_name.starts_with("./internal")
            || module_name.starts_with("../internal")
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
            "TypeScriptModuleAnalyzer".to_string(),
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

impl Default for ModuleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
