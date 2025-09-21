//! TypeScript class structure analysis for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;

#[cfg(feature = "tree-sitter")]
use tree_sitter::StreamingIterator;

/// Analyzes TypeScript class structures for abstraction leaks.
pub struct ClassAnalyzer;

impl ClassAnalyzer {
    /// Creates a new class analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyzes DOM coupling patterns in classes.
    pub fn analyze_dom_coupling(
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
            ; Detect DOM manipulation calls
            (call_expression
              function: (member_expression
                object: (identifier) @dom_object
                property: (property_identifier) @dom_method)) @dom_call

            ; Detect variable declarations with DOM types
            (variable_declaration
              declarations: (variable_declarator
                name: (identifier) @var_name
                type: (type_annotation
                  (_) @var_type)?)) @var_decl

            ; Detect class property definitions with DOM types
            (class_declaration
              body: (class_body
                (property_definition
                  name: (property_identifier) @prop_name
                  type: (type_annotation
                    (_) @prop_type)))) @class_prop_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create TypeScript DOM query: {}", e))
        })?;

        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&query, tree.root_node(), source_bytes);

        while let Some((match_, _)) = captures.next() {
            for capture in match_.captures.iter() {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];

                match capture_name {
                    "dom_object" => {
                        if let Ok(dom_object) = node.utf8_text(source_bytes) {
                            if self.is_dom_object(dom_object) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!(
                                        "DOM manipulation with '{}' in business logic",
                                        dom_object
                                    ),
                                    node.start_position().row as u32 + 1,
                                    LeakType::FrameworkCoupling,
                                    "high",
                                ));
                            }
                        }
                    }
                    "var_type" | "prop_type" => {
                        if let Ok(var_type) = node.utf8_text(source_bytes) {
                            if self.is_dom_type(var_type) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!("Variable declared with DOM type '{}'", var_type),
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

    /// Analyzes class inheritance and extension patterns.
    pub fn analyze_class_inheritance(
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
            ; Detect class declarations with extends clause
            (class_declaration
              name: (identifier) @class_name
              heritage: (class_heritage
                (extends_clause
                  value: (identifier) @parent_class))?) @class_decl

            ; Detect method definitions with framework-specific types
            (class_declaration
              body: (class_body
                (method_definition
                  name: (property_identifier) @method_name
                  parameters: (formal_parameters
                    (identifier) @param_name*)*))) @method_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to create TypeScript class query: {}", e))
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
                            if self.is_framework_class(parent_class) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!("Class extends framework class '{}'", parent_class),
                                    node.start_position().row as u32 + 1,
                                    LeakType::FrameworkCoupling,
                                    "high",
                                ));
                            }
                        }
                    }
                    "method_name" => {
                        if let Ok(method_name) = node.utf8_text(source_bytes) {
                            if self.is_framework_lifecycle_method(method_name) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!(
                                        "Framework lifecycle method '{}' in business logic",
                                        method_name
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

    /// Analyzes class property declarations for leaks.
    pub fn analyze_class_properties(
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
            ; Detect class property definitions
            (class_declaration
              body: (class_body
                (property_definition
                  accessibility_modifier? @access_mod
                  name: (property_identifier) @prop_name
                  type: (type_annotation
                    (_) @prop_type)?))) @prop_decl

            ; Detect constructor parameters with property declarations
            (class_declaration
              body: (class_body
                (method_definition
                  kind: "constructor"
                  parameters: (formal_parameters
                    (identifier) @constructor_param)*))) @constructor_decl
        "#;

        let query = Query::new(&language, query_source).map_err(|e| {
            AnalysisError::DetectionError(format!(
                "Failed to create TypeScript property query: {}",
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
                                        "Class property uses infrastructure type '{}'",
                                        prop_type
                                    ),
                                    node.start_position().row as u32 + 1,
                                    LeakType::ImplementationExposure,
                                    "medium",
                                ));
                            }
                        }
                    }
                    "prop_name" => {
                        if let Ok(prop_name) = node.utf8_text(source_bytes) {
                            if self.is_private_property_exposed(prop_name) {
                                issues.push(self.create_issue(
                                    context,
                                    &format!("Private property '{}' may be exposed", prop_name),
                                    node.start_position().row as u32 + 1,
                                    LeakType::VisibilityViolation,
                                    "low",
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

    /// Checks if an object represents a DOM object.
    fn is_dom_object(&self, obj_name: &str) -> bool {
        let dom_objects = [
            "document",
            "window",
            "navigator",
            "location",
            "console",
            "localStorage",
            "sessionStorage",
        ];

        dom_objects.iter().any(|pattern| obj_name == *pattern)
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
            "React.Component",
            "Vue.Component",
            "Angular.Component",
        ];

        framework_classes
            .iter()
            .any(|pattern| class_name.contains(pattern))
    }

    /// Checks if a method represents a framework lifecycle method.
    fn is_framework_lifecycle_method(&self, method_name: &str) -> bool {
        let lifecycle_methods = [
            "componentDidMount",
            "componentWillUnmount",
            "componentDidUpdate",
            "render",
            "ngOnInit",
            "ngOnDestroy",
            "ngOnChanges",
            "created",
            "mounted",
            "beforeDestroy",
            "destroyed",
        ];

        lifecycle_methods
            .iter()
            .any(|pattern| method_name == *pattern)
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

    /// Checks if a property name indicates private use but may be exposed.
    fn is_private_property_exposed(&self, prop_name: &str) -> bool {
        prop_name.starts_with('_') || prop_name.starts_with('#')
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
            "TypeScriptClassAnalyzer".to_string(),
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
