//! TypeScript interface and type parsing for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Parses TypeScript interfaces and types for abstraction leaks.
pub struct InterfaceParser;

impl InterfaceParser {
    /// Creates a new interface parser.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes interface violation patterns.
    pub fn analyze_interface_violations(
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
            ; Detect interface declarations
            (interface_declaration
              name: (type_identifier) @interface_name
              body: (object_type
                (property_signature
                  name: (property_identifier) @prop_name
                  type: (_) @prop_type)*)) @interface_decl

            ; Detect class implementations
            (class_declaration
              name: (type_identifier) @class_name
              heritage: (class_heritage
                (implements_clause
                  (type_identifier) @implemented_interface)*)?
              body: (class_body
                (property_definition
                  name: (property_identifier) @class_prop
                  type: (_)? @class_prop_type)*)) @class_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!(
                "Failed to create TypeScript interface query: {}",
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
                    "prop_type" => {
                        if let Ok(prop_type) = node.utf8_text(source_bytes) {
                            if self.is_infrastructure_type(prop_type) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!(
                                        "Interface property exposes infrastructure type '{}'",
                                        prop_type
                                    ),
                                    node.start_position().row as u32 + 1,
                                    LeakType::ImplementationExposure,
                                    "high",
                                ));
                            }
                        }
                    }
                    "class_prop_type" => {
                        if let Ok(prop_type) = node.utf8_text(source_bytes) {
                            if self.is_framework_type(prop_type) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!(
                                        "Class property exposes framework type '{}'",
                                        prop_type
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

    /// Analyzes type exposure patterns.
    pub fn analyze_type_exposure(
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
            ; Detect type alias declarations
            (type_alias_declaration
              name: (type_identifier) @type_name
              value: (_) @type_value) @type_alias

            ; Detect function declarations with type annotations
            (function_declaration
              name: (identifier) @func_name
              return_type: (type_annotation
                (_) @return_type)?) @func_decl

            ; Detect export statements
            (export_statement
              declaration: (_) @exported_decl) @export_stmt
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create TypeScript type query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "type_value" => {
                        if let Ok(type_value) = node.utf8_text(source_bytes) {
                            if self.is_infrastructure_type(type_value) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!(
                                        "Type alias exposes infrastructure type '{}'",
                                        type_value
                                    ),
                                    node.start_position().row as u32 + 1,
                                    LeakType::ImplementationExposure,
                                    "high",
                                ));
                            }
                        }
                    }
                    "return_type" => {
                        if let Ok(return_type) = node.utf8_text(source_bytes) {
                            if self.is_dom_type(return_type) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!(
                                        "Function returns DOM type '{}' in business logic",
                                        return_type
                                    ),
                                    node.start_position().row as u32 + 1,
                                    LeakType::FrameworkCoupling,
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

    /// Checks if a type represents an infrastructure type.
    fn is_infrastructure_type(&self, type_name: &str) -> bool {
        let infrastructure_types = [
            "Express.Request",
            "Express.Response",
            "Koa.Context",
            "Mongoose.Document",
            "Sequelize.Model",
            "TypeORM.Entity",
            "Socket.IO.Socket",
            "Redis.Client",
            "AWS.S3",
        ];

        infrastructure_types
            .iter()
            .any(|pattern| type_name.contains(pattern))
    }

    /// Checks if a type represents a framework type.
    fn is_framework_type(&self, type_name: &str) -> bool {
        let framework_types = [
            "React.Component",
            "React.FC",
            "Vue.Component",
            "Angular.Component",
            "Component",
            "Props",
            "State",
            "NextApiRequest",
            "NextApiResponse",
        ];

        framework_types
            .iter()
            .any(|pattern| type_name.contains(pattern))
    }

    /// Checks if a type represents a DOM type.
    fn is_dom_type(&self, type_name: &str) -> bool {
        let dom_types = [
            "HTMLElement",
            "Element",
            "Node",
            "Document",
            "HTMLInputElement",
            "HTMLButtonElement",
            "HTMLDivElement",
            "Event",
            "MouseEvent",
            "KeyboardEvent",
        ];

        dom_types.iter().any(|pattern| type_name.contains(pattern))
    }

    /// Checks if a generic type parameter indicates a leak.
    pub fn is_leaky_generic(&self, generic_def: &str) -> bool {
        generic_def.contains("<any>")
            || generic_def.contains("<unknown>")
            || generic_def.contains("<object>")
    }

    /// Analyzes method signatures for potential type leaks.
    pub fn analyze_method_signature(&self, signature: &str) -> Vec<String> {
        let mut issues = Vec::new();

        if self.is_dom_type(signature) {
            issues.push("Method signature contains DOM types".to_string());
        }

        if self.is_infrastructure_type(signature) {
            issues.push("Method signature contains infrastructure types".to_string());
        }

        if self.is_leaky_generic(signature) {
            issues.push("Method uses overly broad generic types".to_string());
        }

        issues
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
            "TypeScriptInterfaceParser".to_string(),
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

impl Default for InterfaceParser {
    fn default() -> Self {
        Self::new()
    }
}
