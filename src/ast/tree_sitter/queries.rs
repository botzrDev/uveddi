//! This module contains tree-sitter queries for extracting dependencies from various languages.

pub const RUST_IMPORTS_QUERY: &str = r#"
(use_declaration
  (use_wildcard
    (scoped_identifier) @path)
)
(use_declaration
  (use_list
    (scoped_identifier) @path)
)
(use_declaration
  (scoped_identifier) @path
)
(use_declaration
  (identifier) @path
)
(extern_crate_declaration
  name: (identifier) @path
)
(mod_item
  name: (identifier) @path
)
"#;

/// Placeholder documentation for public items
pub const PYTHON_IMPORTS_QUERY: &str = r#"
(import_statement
  name: (dotted_name) @path
)
(import_from_statement
  module_name: (dotted_name) @path
)
"#;

/// Placeholder documentation for public items
// This query is a bit more complex to handle various JS/TS import syntaxes
pub const JAVASCRIPT_IMPORTS_QUERY: &str = r#"
(import_statement
  source: (string) @path
)
(export_statement
  source: (string) @path
)
(call_expression
  function: (identifier) @func
  arguments: (arguments (string) @path)
  (#eq? @func "require")
)
"#;
