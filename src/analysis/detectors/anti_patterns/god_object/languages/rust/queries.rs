//! Rust Tree-sitter queries

/// Rust struct detection query
pub const RUST_STRUCT_QUERY: &str = r#"
(struct_item
  name: (type_identifier) @name
  body: (field_declaration_list) @body
)
"#;

/// Rust impl block detection query
pub const RUST_IMPL_QUERY: &str = r#"
(impl_item
  type: (type_identifier) @name
  body: (declaration_list) @body
)
"#;

/// Rust derive macro detection query
pub const RUST_DERIVE_QUERY: &str = r#"
(attribute_item
  (attribute
    (scoped_identifier) @attr_name
    arguments: (token_tree) @attr_args
  )
)
"#;

/// Rust use declaration detection query
pub const RUST_USE_QUERY: &str = r#"
(use_declaration
  argument: (scoped_identifier) @import_path
)
"#;