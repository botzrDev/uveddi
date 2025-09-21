//! Visibility and access pattern analysis for implementation leaks.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ImplementationVisibilityIssue, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes visibility and access patterns for implementation leaks.
pub struct VisibilityAnalyzer;

impl VisibilityAnalyzer {
    /// Creates a new visibility analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes Rust visibility patterns.
    pub fn analyze_rust_visibility(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationVisibilityIssue>, AnalysisError> {
        let mut visibility_issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect module declarations with visibility
            (mod_item
              (visibility_modifier)? @mod_vis
              name: (identifier) @mod_name) @mod_decl

            ; Detect struct fields with public visibility
            (struct_item
              name: (type_identifier) @struct_name
              body: (field_declaration_list
                (field_declaration
                  (visibility_modifier) @field_vis
                  name: (field_identifier) @field_name))) @struct_decl

            ; Detect use declarations for internal modules
            (use_declaration
              argument: (scoped_identifier
                path: (identifier) @use_module)) @use_stmt
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create Rust visibility query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "field_vis" => {
                        if let Ok(vis_text) = node.utf8_text(source_bytes) {
                            if vis_text == "pub" {
                                if let Some(field_name_node) = self.find_sibling_capture(
                                    &match_.captures,
                                    &query,
                                    "field_name",
                                ) {
                                    if let Ok(field_name) = field_name_node.utf8_text(source_bytes)
                                    {
                                        if self.is_internal_field_name(field_name) {
                                            visibility_issues.push(ImplementationVisibilityIssue {
                                                description: format!(
                                                    "Internal field '{}' exposed as public",
                                                    field_name
                                                ),
                                                visibility_problem: "Field name suggests internal use but has public visibility".to_string(),
                                                line_number: node.start_position().row as u32 + 1,
                                                severity: "medium".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "use_module" => {
                        if let Ok(module_name) = node.utf8_text(source_bytes) {
                            if self.is_internal_module(module_name) {
                                visibility_issues.push(ImplementationVisibilityIssue {
                                    description: format!(
                                        "Import of internal module '{}'",
                                        module_name
                                    ),
                                    visibility_problem: "Accessing internal module from outside"
                                        .to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "high".to_string(),
                                });
                            }
                        }
                    }
                    "mod_vis" => {
                        if let Ok(vis_text) = node.utf8_text(source_bytes) {
                            if vis_text == "pub" {
                                if let Some(mod_name_node) =
                                    self.find_sibling_capture(&match_.captures, &query, "mod_name")
                                {
                                    if let Ok(mod_name) = mod_name_node.utf8_text(source_bytes) {
                                        if self.is_internal_module(mod_name) {
                                            visibility_issues.push(ImplementationVisibilityIssue {
                                                description: format!(
                                                    "Internal module '{}' exposed as public",
                                                    mod_name
                                                ),
                                                visibility_problem: "Module name suggests internal use but has public visibility".to_string(),
                                                line_number: node.start_position().row as u32 + 1,
                                                severity: "high".to_string(),
                                            });
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

        Ok(visibility_issues)
    }

    /// Analyzes Python visibility patterns.
    pub fn analyze_python_visibility(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationVisibilityIssue>, AnalysisError> {
        let mut visibility_issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect attribute access with underscores (private access)
            (attribute
              object: (identifier) @obj_name
              attribute: (identifier) @attr_name) @attr_access

            ; Detect function definitions with underscore naming
            (function_definition
              name: (identifier) @func_name) @func_def

            ; Detect class definitions with underscore naming
            (class_definition
              name: (identifier) @class_name) @class_def
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!(
                "Failed to create Python visibility query: {}",
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
                    "attr_name" => {
                        if let Ok(attr_name) = node.utf8_text(source_bytes) {
                            if self.is_private_attribute_access(attr_name) {
                                visibility_issues.push(ImplementationVisibilityIssue {
                                    description: format!(
                                        "Access to private attribute '{}'",
                                        attr_name
                                    ),
                                    visibility_problem: "Accessing private attribute from outside class".to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "medium".to_string(),
                                });
                            }
                        }
                    }
                    "func_name" => {
                        if let Ok(func_name) = node.utf8_text(source_bytes) {
                            if self.is_private_function(func_name)
                                && self.is_likely_exported(func_name)
                            {
                                visibility_issues.push(ImplementationVisibilityIssue {
                                    description: format!(
                                        "Private function '{}' may be exposed",
                                        func_name
                                    ),
                                    visibility_problem: "Function name suggests private use but appears to be exposed".to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "low".to_string(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(visibility_issues)
    }

    /// Analyzes JavaScript/TypeScript visibility patterns.
    pub fn analyze_js_visibility(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ImplementationVisibilityIssue>, AnalysisError> {
        let mut visibility_issues = Vec::new();

        let source_bytes = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::DetectionError("No AST available".to_string()))?;
        let language = tree.language();

        let query_source = r#"
            ; Detect property access on objects
            (member_expression
              object: (identifier) @obj_name
              property: (property_identifier) @prop_name) @member_access

            ; Detect export statements
            (export_statement
              declaration: (_) @exported_decl) @export_stmt

            ; Detect class method definitions
            (class_declaration
              body: (class_body
                (method_definition
                  name: (property_identifier) @method_name)*)) @class_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create JS visibility query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "prop_name" => {
                        if let Ok(prop_name) = node.utf8_text(source_bytes) {
                            if self.is_private_property(prop_name) {
                                visibility_issues.push(ImplementationVisibilityIssue {
                                    description: format!(
                                        "Access to private property '{}'",
                                        prop_name
                                    ),
                                    visibility_problem: "Accessing private property from outside"
                                        .to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "medium".to_string(),
                                });
                            }
                        }
                    }
                    "method_name" => {
                        if let Ok(method_name) = node.utf8_text(source_bytes) {
                            if self.is_private_method_js(method_name) {
                                visibility_issues.push(ImplementationVisibilityIssue {
                                    description: format!(
                                        "Private method '{}' may be exposed",
                                        method_name
                                    ),
                                    visibility_problem: "Method name suggests private use"
                                        .to_string(),
                                    line_number: node.start_position().row as u32 + 1,
                                    severity: "low".to_string(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(visibility_issues)
    }

    /// Checks if a field name indicates internal use.
    fn is_internal_field_name(&self, field_name: &str) -> bool {
        field_name.starts_with('_')
            || field_name.contains("internal")
            || field_name.contains("impl")
            || field_name.contains("private")
    }

    /// Checks if a module name indicates internal use.
    fn is_internal_module(&self, module_name: &str) -> bool {
        module_name.contains("internal")
            || module_name.contains("impl")
            || module_name.contains("detail")
            || module_name.starts_with('_')
    }

    /// Checks if an attribute access is to a private attribute.
    fn is_private_attribute_access(&self, attr_name: &str) -> bool {
        attr_name.starts_with('_') && !attr_name.starts_with("__")
    }

    /// Checks if a function name indicates private use.
    fn is_private_function(&self, func_name: &str) -> bool {
        func_name.starts_with('_') && !func_name.starts_with("__")
    }

    /// Checks if a function is likely to be exported.
    fn is_likely_exported(&self, func_name: &str) -> bool {
        // Simple heuristic - if it doesn't start with __ it might be exported
        !func_name.starts_with("__")
    }

    /// Checks if a property name indicates private use.
    fn is_private_property(&self, prop_name: &str) -> bool {
        prop_name.starts_with('_') || prop_name.starts_with('#') // Private fields in modern JS
    }

    /// Checks if a method name indicates private use in JS/TS.
    fn is_private_method_js(&self, method_name: &str) -> bool {
        method_name.starts_with('_')
            || method_name.starts_with('#')
            || method_name.contains("internal")
    }

    /// Helper function to find a sibling capture by name.
    fn find_sibling_capture<'a>(
        &self,
        captures: &'a [crate::ast::tree_sitter::QueryCapture],
        query: &Query,
        capture_name: &str,
    ) -> Option<Node<'a>> {
        captures
            .iter()
            .find(|capture| query.capture_names()[capture.index as usize] == capture_name)
            .map(|capture| capture.node)
    }
}

impl Default for VisibilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
