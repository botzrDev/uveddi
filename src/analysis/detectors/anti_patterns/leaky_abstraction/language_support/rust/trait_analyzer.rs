//! Rust trait and implementation analysis for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes Rust trait exposure patterns and implementation details for abstraction leaks.
#[derive(Clone)]
pub struct TraitAnalyzer;

impl TraitAnalyzer {
    /// Creates a new trait analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes trait exposure patterns.
    pub fn analyze_trait_exposure(
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
            ; Detect public trait implementations that expose internals
            (impl_item
              trait: (type_identifier) @trait_name
              type: (type_identifier) @impl_type
              body: (declaration_list
                (function_item
                  (visibility_modifier)? @method_vis
                  name: (identifier) @method_name))) @impl_block

            ; Detect trait object safety violations
            (trait_item
              (visibility_modifier) @trait_vis
              name: (type_identifier) @trait_name
              body: (declaration_list
                (function_item
                  name: (identifier) @trait_method))) @trait_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Rust trait query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                if capture_name == "trait_name" {
                    if let Ok(trait_name) = node.utf8_text(source_bytes) {
                        if self.is_infrastructure_trait(trait_name) {
                            issues.push(self.create_issue(
                                context,
                                &format!(
                                    "Infrastructure trait '{}' may expose implementation details",
                                    trait_name
                                ),
                                node.start_position().row as u32 + 1,
                                LeakType::ImplementationExposure,
                                "medium",
                            ));
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Analyzes implementation detail exposure.
    pub fn analyze_impl_details(
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
            ; Detect public struct fields
            (struct_item
              (visibility_modifier) @struct_vis
              name: (type_identifier) @struct_name
              body: (field_declaration_list
                (field_declaration
                  (visibility_modifier) @field_vis
                  name: (field_identifier) @field_name
                  type: (_) @field_type))) @struct_decl

            ; Detect public enum variants with data
            (enum_item
              (visibility_modifier) @enum_vis
              name: (type_identifier) @enum_name
              body: (enum_variant_list
                (enum_variant
                  name: (identifier) @variant_name
                  body: (_)? @variant_body))) @enum_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Rust impl query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                if capture_name == "field_vis" {
                    if let Ok(vis_text) = node.utf8_text(source_bytes) {
                        if vis_text == "pub" {
                            issues.push(self.create_issue(
                                context,
                                "Public field exposes internal structure - consider using getter methods",
                                node.start_position().row as u32 + 1,
                                LeakType::ImplementationExposure,
                                "medium",
                            ));
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Checks if a trait name represents an infrastructure trait.
    fn is_infrastructure_trait(&self, trait_name: &str) -> bool {
        let infrastructure_traits = [
            "Connection",
            "Transaction",
            "Session",
            "Query",
            "Serialize",
            "Deserialize",
            "FromRequest",
            "IntoResponse",
        ];

        infrastructure_traits
            .iter()
            .any(|pattern| trait_name.contains(pattern))
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
            "RustTraitAnalyzer".to_string(),
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

impl Default for TraitAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
