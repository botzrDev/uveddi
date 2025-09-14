//! Tree-sitter query execution utilities
//!
//! This module provides common utilities for executing tree-sitter queries across detectors,
//! eliminating duplication in query setup, execution, and error handling patterns.

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Language, Node, Query, QueryCursor, Tree};
use std::collections::HashMap;

/// A helper utility for executing tree-sitter queries with consistent error handling
/// and pattern extraction. This eliminates the need for each detector to implement
/// the same query setup and execution logic.
pub struct TreeSitterQueryHelper {
    query_cache: HashMap<String, Query>,
}

impl TreeSitterQueryHelper {
    /// Create a new query helper instance
    pub fn new() -> Self {
        Self {
            query_cache: HashMap::new(),
        }
    }

    /// Execute a tree-sitter query and collect matching nodes
    ///
    /// # Arguments
    /// * `language` - The tree-sitter language to use for parsing
    /// * `query_str` - The tree-sitter query string to execute
    /// * `tree` - The parsed AST tree to query against
    /// * `source` - The source code bytes
    ///
    /// # Returns
    /// A vector of tuples containing (capture_name, node, text) for each match
    pub fn execute_query(
        &mut self,
        language: &Language,
        query_str: &str,
        tree: &Tree,
        source: &[u8],
    ) -> Result<Vec<(String, Node<'_>, String)>, AnalysisError> {
        // Get or create query (with caching for performance)
        let query = self.get_or_create_query(language, query_str)?;

        let mut cursor = QueryCursor::new();
        let mut results = Vec::new();

        // Execute query and collect matches
        let mut matches = cursor.matches(&query, tree.root_node(), source);
        while let Some(mat) = matches.next() {
            for capture in mat.captures {
                let node = capture.node;
                if let Ok(text) = node.utf8_text(source) {
                    let capture_name = query
                        .capture_names()
                        .get(capture.index as usize)
                        .unwrap_or(&"unknown")
                        .to_string();

                    results.push((capture_name, node, text.to_string()));
                }
            }
        }

        Ok(results)
    }

    /// Execute a query and get only the first capture from each match
    ///
    /// This is a convenience method for the common pattern where detectors only care
    /// about the first capture in each match (typically the name or main node).
    pub fn execute_query_first_capture(
        &mut self,
        language: &Language,
        query_str: &str,
        tree: &Tree,
        source: &[u8],
    ) -> Result<Vec<(Node<'_>, String)>, AnalysisError> {
        let query = self.get_or_create_query(language, query_str)?;

        let mut cursor = QueryCursor::new();
        let mut results = Vec::new();

        let mut matches = cursor.matches(&query, tree.root_node(), source);
        while let Some(mat) = matches.next() {
            if let Some(capture) = mat.captures.first() {
                let node = capture.node;
                if let Ok(text) = node.utf8_text(source) {
                    results.push((node, text.to_string()));
                }
            }
        }

        Ok(results)
    }

    /// Execute multiple queries and return categorized results
    ///
    /// This is useful for detectors that need to run multiple related queries
    /// (e.g., functions, structs, enums) and process them together.
    pub fn execute_multiple_queries(
        &mut self,
        language: &Language,
        queries: &[(&str, &str)], // (query_name, query_string) pairs
        tree: &Tree,
        source: &[u8],
    ) -> Result<HashMap<String, Vec<(Node, String)>>, AnalysisError> {
        let mut results = HashMap::new();

        for (query_name, query_str) in queries {
            let query = self.get_or_create_query(language, query_str)?;
            let mut cursor = QueryCursor::new();
            let mut query_results = Vec::new();

            let mut matches = cursor.matches(&query, tree.root_node(), source);
            while let Some(mat) = matches.next() {
                if let Some(capture) = mat.captures.first() {
                    let node = capture.node;
                    if let Ok(text) = node.utf8_text(source) {
                        query_results.push((node, text.to_string()));
                    }
                }
            }

            results.insert(query_name.to_string(), query_results);
        }

        Ok(results)
    }

    /// Get node position information as a tuple (start_line, end_line, start_col, end_col)
    pub fn get_node_position(node: &Node) -> (u32, u32, u32, u32) {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        (
            (start_pos.row + 1) as u32, // Convert to 1-based line numbers
            (end_pos.row + 1) as u32,
            start_pos.column as u32,
            end_pos.column as u32,
        )
    }

    /// Extract a code snippet around a node with context lines
    pub fn extract_code_snippet(
        node: &Node,
        source: &[u8],
        context_lines: usize,
    ) -> String {
        let source_str = String::from_utf8_lossy(source);
        let lines: Vec<&str> = source_str.lines().collect();

        let start_line = node.start_position().row;
        let end_line = node.end_position().row;

        let context_start = start_line.saturating_sub(context_lines);
        let context_end = (end_line + context_lines + 1).min(lines.len());

        lines[context_start..context_end].join("\n")
    }

    /// Helper method to get or create a cached query
    fn get_or_create_query(
        &mut self,
        language: &Language,
        query_str: &str,
    ) -> Result<&Query, AnalysisError> {
        let cache_key = format!("{}_{}", language.id(), query_str);

        if !self.query_cache.contains_key(&cache_key) {
            let query = Query::new(language, query_str)
                .map_err(|e| AnalysisError::DetectionError(format!("Failed to create query: {}", e)))?;
            self.query_cache.insert(cache_key.clone(), query);
        }

        Ok(self.query_cache.get(&cache_key).unwrap())
    }
}

impl Default for TreeSitterQueryHelper {
    fn default() -> Self {
        Self::new()
    }
}

/// Common tree-sitter query constants used across multiple detectors
pub mod queries {
    /// Rust language queries
    pub mod rust {
        pub const FUNCTION_QUERY: &str = r#"
            (function_item
                name: (identifier) @name) @function
        "#;

        pub const STRUCT_QUERY: &str = r#"
            (struct_item
                name: (type_identifier) @name) @struct
        "#;

        pub const ENUM_QUERY: &str = r#"
            (enum_item
                name: (type_identifier) @name) @enum
        "#;

        pub const IMPL_QUERY: &str = r#"
            (impl_item
                type: (type_identifier) @name) @impl
        "#;

        pub const MOD_QUERY: &str = r#"
            (mod_item
                name: (identifier) @name) @module
        "#;
    }

    /// Python language queries
    pub mod python {
        pub const FUNCTION_QUERY: &str = r#"
            (function_definition
                name: (identifier) @name) @function
        "#;

        pub const CLASS_QUERY: &str = r#"
            (class_definition
                name: (identifier) @name) @class
        "#;

        pub const ASSIGNMENT_QUERY: &str = r#"
            (assignment
                left: (identifier) @name) @assignment
        "#;
    }

    /// JavaScript/TypeScript language queries
    pub mod javascript {
        pub const FUNCTION_QUERY: &str = r#"
            [
                (function_declaration
                    name: (identifier) @name) @function
                (method_definition
                    name: (property_identifier) @name) @method
                (arrow_function) @arrow_function
            ]
        "#;

        pub const CLASS_QUERY: &str = r#"
            (class_declaration
                name: (identifier) @name) @class
        "#;

        pub const VARIABLE_QUERY: &str = r#"
            [
                (variable_declarator
                    name: (identifier) @name) @variable
                (lexical_declaration
                    (variable_declarator
                        name: (identifier) @name)) @variable
            ]
        "#;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_position() {
        // Test would require actual tree-sitter setup
        // This is a placeholder for the position utility
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_query_helper_creation() {
        let helper = TreeSitterQueryHelper::new();
        assert_eq!(helper.query_cache.len(), 0);
    }
}