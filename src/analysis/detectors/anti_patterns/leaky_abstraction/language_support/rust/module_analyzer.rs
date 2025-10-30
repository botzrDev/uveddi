//! Rust module privacy and error propagation analysis for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes Rust error propagation and module privacy patterns for abstraction leaks.
#[derive(Clone)]
pub struct ModuleAnalyzer;

impl ModuleAnalyzer {
    /// Creates a new module analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes error propagation patterns.
    pub fn analyze_error_propagation(
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
            ; Detect public functions with Result return types
            (function_item
              (visibility_modifier) @fn_vis
              name: (identifier) @fn_name
              return_type: (type_identifier) @return_type) @function_decl

            ; Detect Result types in function signatures
            (function_item
              (visibility_modifier) @fn_vis
              return_type: (generic_type
                type: (type_identifier) @result_type
                type_arguments: (type_arguments
                  (_) @ok_type
                  (_) @err_type))) @result_function
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Rust error query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                if capture_name == "err_type" {
                    if let Ok(err_type_text) = node.utf8_text(source_bytes) {
                        if self.is_infrastructure_error_type(err_type_text) {
                            issues.push(self.create_issue(
                                context,
                                &format!(
                                    "Infrastructure error type '{}' propagated to public API",
                                    err_type_text
                                ),
                                node.start_position().row as u32 + 1,
                                LeakType::ErrorPropagation,
                                "high",
                            ));
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Analyzes module privacy violations.
    pub fn analyze_module_privacy(
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
            ; Detect use declarations
            (use_declaration
              argument: (scoped_identifier
                path: (identifier) @module_name
                name: (_) @item_name)) @use_stmt

            ; Detect mod declarations
            (mod_item
              (visibility_modifier)? @mod_vis
              name: (identifier) @mod_name) @mod_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Rust module query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                if capture_name == "module_name" {
                    if let Ok(module_name) = node.utf8_text(source_bytes) {
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
            }
        }

        Ok(issues)
    }

    /// Checks if a type represents an infrastructure error type.
    fn is_infrastructure_error_type(&self, type_text: &str) -> bool {
        let infrastructure_error_patterns = [
            "DieselError",
            "SqlxError",
            "SeaOrmError",
            "tokio::Error",
            "std::io::Error",
            "reqwest::Error",
            "serde_json::Error",
            "toml::de::Error",
            "rusqlite::Error",
            "postgres::Error",
        ];

        infrastructure_error_patterns.iter().any(|pattern| {
            type_text.contains(pattern)
                || (type_text.contains("Result<") && type_text.contains(pattern))
        })
    }

    /// Checks if a module is considered internal.
    fn is_internal_module(&self, module_name: &str) -> bool {
        module_name.contains("internal")
            || module_name.contains("impl")
            || module_name.contains("detail")
            || module_name.starts_with('_')
    }

    /// Extracts the module name from a use statement.
    pub fn extract_module_from_use_statement(&self, use_text: &str) -> Option<String> {
        if let Some(content) = use_text.strip_prefix("use ") {
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
            "RustModuleAnalyzer".to_string(),
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
