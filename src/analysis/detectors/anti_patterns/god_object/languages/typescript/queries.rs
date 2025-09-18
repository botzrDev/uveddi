//! TypeScript Tree-sitter queries

/// TypeScript class detection query
pub const TYPESCRIPT_CLASS_QUERY: &str = r#"
(class_declaration
  name: (type_identifier) @name
  body: (class_body) @body
)
(class_declaration
  name: (type_identifier) @name
  type_parameters: (type_parameters) @type_params
  body: (class_body) @body
)
(class_declaration
  name: (type_identifier) @name
  heritage: (class_heritage) @heritage
  body: (class_body) @body
)
"#;

/// TypeScript interface detection query
pub const TYPESCRIPT_INTERFACE_QUERY: &str = r#"
(interface_declaration
  name: (type_identifier) @name
  body: (object_type) @body
)
"#;

/// TypeScript namespace detection query
pub const TYPESCRIPT_NAMESPACE_QUERY: &str = r#"
[
  (module_declaration
    name: (identifier) @name
    body: (statement_block) @body
  )
  (namespace_declaration
    name: (identifier) @name
    body: (statement_block) @body
  )
]
"#;

/// TypeScript import detection query
pub const TYPESCRIPT_IMPORT_QUERY: &str = r#"
[
  (import_statement
    source: (string) @import_path
  )
  (import_statement
    (import_clause
      (named_imports
        (import_specifier) @import_name
      )
    )
    source: (string) @import_path
  )
  (import_statement
    (import_clause
      (namespace_import) @namespace_import
    )
    source: (string) @import_path
  )
  (call_expression
    function: (identifier) @func_name
    arguments: (arguments (string) @import_path)
    (#eq? @func_name "require")
  )
]
"#;