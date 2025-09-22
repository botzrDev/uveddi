use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor, Tree};
use streaming_iterator::StreamingIterator;

/// Rust Tree-sitter query definitions and execution utilities
pub struct RustQueries;

impl RustQueries {
    pub const USE_QUERY: &'static str = r#"
        (use_declaration
          argument: (scoped_use_list
            list: (use_list
              (scoped_identifier
                path: (identifier) @module
                name: (identifier) @item))) @use_decl)

        (use_declaration
          argument: (scoped_identifier
            path: (identifier) @module
            name: (identifier) @item)) @use_decl
    "#;

    pub const CALL_QUERY: &'static str = r#"
        (call_expression
          function: (scoped_identifier
            path: (identifier) @module
            name: (identifier) @function)) @call

        (call_expression
          function: (field_expression
            value: (identifier) @object
            field: (field_identifier) @method)) @method_call
    "#;

    pub const STRUCT_QUERY: &'static str = r#"
        (struct_expression
          name: (scoped_type_identifier
            path: (identifier) @module
            name: (type_identifier) @struct)) @instantiation
    "#;

    pub const TRAIT_IMPL_QUERY: &'static str = r#"
        (impl_item
          trait: (type_identifier) @trait_name
          type: (type_identifier) @type_name) @impl_block
    "#;

    pub const MOD_QUERY: &'static str = r#"
        (mod_item
          name: (identifier) @module_name) @mod_decl
    "#;

    /// Create a query from query string and handle errors
    pub fn create_query(query_str: &str) -> Result<Query, AnalysisError> {
        Query::new(
            &crate::ast::tree_sitter::tree_sitter_rust::LANGUAGE.into(),
            query_str,
        )
        .map_err(|e| AnalysisError::QueryError(format!("Failed to create query: {}", e)))
    }

    /// Execute a query and collect all matches
    pub fn execute_query<'a>(
        query: &'a Query,
        tree: &'a Tree,
        source: &'a str,
    ) -> Result<Vec<crate::ast::tree_sitter::QueryMatch<'a, 'a>>, AnalysisError> {
        // TODO: Fix QueryMatches lifetime issue
        // QueryMatches contains borrowed data that can't be returned from this function
        // This needs to be redesigned to either:
        // 1. Take a closure to process matches immediately
        // 2. Return extracted data instead of QueryMatch objects
        // 3. Use a different approach that doesn't require storing QueryMatch
        Ok(Vec::new())
    }

    /// Get text content from a node safely
    pub fn get_node_text(
        node: &crate::ast::tree_sitter::Node,
        source: &str,
    ) -> Result<String, AnalysisError> {
        node.utf8_text(source.as_bytes())
            .map(|s| s.to_string())
            .map_err(|e| AnalysisError::QueryError(format!("Failed to get node text: {}", e)))
    }

    /// Get line number from node position
    pub fn get_line_number(node: &crate::ast::tree_sitter::Node) -> u32 {
        node.start_position().row as u32 + 1
    }
}
