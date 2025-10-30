//! JavaScript/TypeScript-specific tree-sitter queries

/// Query for import statements
pub const IMPORT_QUERY: &str = r#"
(import_statement source: (string) @path)
"#;

/// Query for require statements
pub const REQUIRE_QUERY: &str = r#"
(call_expression
  function: (identifier) @func
  arguments: (arguments (string) @path)
  (#eq? @func "require"))
"#;

/// Query for function definitions
pub const FUNCTION_DEFINITION: &str = r#"
(function_declaration name: (identifier) @name) @function
"#;

/// Query for class definitions
pub const CLASS_DEFINITION: &str = r#"
(class_declaration name: (identifier) @name) @class
"#;
