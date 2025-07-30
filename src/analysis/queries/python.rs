//! Python-specific tree-sitter queries

/// Query for import statements
pub const IMPORT_QUERY: &str = r#"
(import_statement name: (_) @path
(import_from_statement module_name: (_) @path)
"#;

/// Query for function definitions
pub const FUNCTION_DEFINITION: &str = r#"
(function_definition name: (identifier) @name) @function
"#;

/// Query for class definitions
pub const CLASS_DEFINITION: &str = r#"
(class_definition name: (identifier) @name) @class
"#;
