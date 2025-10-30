//! Rust-specific tree-sitter queries

/// Query for use declarations (imports)
pub const USE_QUERY: &str = r#"
(use_declaration path: (_) @path
"#;

/// Query for module declarations
pub const MODULE_DECLARATION: &str = r#"
(mod_item name: (identifier) @name) @mod
"#;

/// Query for function definitions
pub const FUNCTION_DEFINITION: &str = r#"
(function_item name: (identifier) @name) @function
"#;
