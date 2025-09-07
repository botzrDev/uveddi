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

/// Python imports query for dependency analysis
pub const PYTHON_IMPORTS_QUERY: &str = r#"
(import_statement
  name: (dotted_name) @path
)
(import_from_statement
  module_name: (dotted_name) @path
)
"#;

/// JavaScript imports query for dependency analysis
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

/// TypeScript-specific imports query with enhanced support for TS constructs
pub const TYPESCRIPT_IMPORTS_QUERY: &str = r#"
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
(import_statement
  (import_clause
    (named_imports
      (import_specifier
        name: (identifier) @import_name
      )
    )
  )
  source: (string) @path
)
(import_statement
  (import_clause
    (namespace_import
      name: (identifier) @namespace_name
    )
  )
  source: (string) @path
)
(type_alias_declaration
  name: (type_identifier) @type_name
)
(interface_declaration
  name: (type_identifier) @interface_name
)
"#;

/// TypeScript interface detection query
pub const TYPESCRIPT_INTERFACE_QUERY: &str = r#"
(interface_declaration
  name: (type_identifier) @interface_name
  body: (object_type) @interface_body
)
"#;

/// TypeScript type alias detection query
pub const TYPESCRIPT_TYPE_ALIAS_QUERY: &str = r#"
(type_alias_declaration
  name: (type_identifier) @type_name
  value: (_) @type_value
)
"#;

/// TypeScript enum detection query
pub const TYPESCRIPT_ENUM_QUERY: &str = r#"
(enum_declaration
  name: (identifier) @enum_name
  body: (enum_body) @enum_body
)
"#;

/// TypeScript namespace detection query
pub const TYPESCRIPT_NAMESPACE_QUERY: &str = r#"
(module_declaration
  name: (identifier) @namespace_name
  body: (statement_block) @namespace_body
)
(namespace_declaration
  name: (identifier) @namespace_name
  body: (statement_block) @namespace_body
)
"#;

/// TypeScript generic detection query
pub const TYPESCRIPT_GENERIC_QUERY: &str = r#"
(interface_declaration
  name: (type_identifier) @interface_name
  type_parameters: (type_parameters) @type_params
)
(function_declaration
  name: (identifier) @function_name
  type_parameters: (type_parameters) @type_params
)
(class_declaration
  name: (type_identifier) @class_name
  type_parameters: (type_parameters) @type_params
)
(type_alias_declaration
  name: (type_identifier) @type_name
  type_parameters: (type_parameters) @type_params
)
"#;

/// TypeScript decorator detection query
pub const TYPESCRIPT_DECORATOR_QUERY: &str = r#"
(decorator
  (identifier) @decorator_name
)
(decorator
  (call_expression
    function: (identifier) @decorator_name
  )
)
"#;

/// TypeScript class with enhanced features query
pub const TYPESCRIPT_CLASS_QUERY: &str = r#"
(class_declaration
  name: (type_identifier) @class_name
  body: (class_body) @class_body
)
(class_declaration
  name: (type_identifier) @class_name
  type_parameters: (type_parameters) @type_params
  body: (class_body) @class_body
)
(class_declaration
  name: (type_identifier) @class_name
  superclass: (extends_clause) @extends
  body: (class_body) @class_body
)
"#;

/// TypeScript method detection query with visibility and static modifiers
pub const TYPESCRIPT_METHOD_QUERY: &str = r#"
(method_definition
  name: (property_name) @method_name
  value: (function_expression) @method_body
)
(method_definition
  accessibility_modifier: (accessibility_modifier) @visibility
  name: (property_name) @method_name
  value: (function_expression) @method_body
)
(method_definition
  "static"? @static_modifier
  name: (property_name) @method_name
  value: (function_expression) @method_body
)
"#;

/// TypeScript property detection query with types and modifiers
pub const TYPESCRIPT_PROPERTY_QUERY: &str = r#"
(property_signature
  name: (property_name) @property_name
  type: (type_annotation) @property_type
)
(public_field_definition
  name: (property_name) @property_name
  type: (type_annotation) @property_type
)
(public_field_definition
  accessibility_modifier: (accessibility_modifier) @visibility
  name: (property_name) @property_name
  type: (type_annotation) @property_type
)
"#;
