//! Tree-sitter query system for structured code analysis

use tree_sitter::{Query, QueryCursor, Tree};
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use crate::analysis::components::ast_provider::ParsedFile;
use anyhow::Result;

/// Query execution engine
pub struct QueryEngine {
    rust_queries: HashMap<&'static str, Query>,
    python_queries: HashMap<&'static str, Query>,
    javascript_queries: HashMap<&'static str, Query>,
}

impl QueryEngine {
    /// Creates a new QueryEngine with compiled queries
    pub fn new() -> Result<Self> {
        let mut rust_queries = HashMap::new();
        rust_queries.insert("use_declaration", Query::new(
            tree_sitter_rust::language(),
            RUST_USE_QUERY
        )?);

        let mut python_queries = HashMap::new();
        python_queries.insert("import_statement", Query::new(
            tree_sitter_python::language(),
            PYTHON_IMPORT_QUERY
        )?);

        let mut javascript_queries = HashMap::new();
        javascript_queries.insert("import_statement", Query::new(
            tree_sitter_javascript::language(),
            JS_IMPORT_QUERY
        )?);

        Ok(Self {
            rust_queries,
            python_queries,
            javascript_queries,
        })
    }

    /// Executes a named query on a parsed file
    pub fn execute_query(&self, parsed_file: &ParsedFile, query_name: &str) -> Vec<QueryMatch> {
        let language = parsed_file.language;
        let tree = parsed_file.tree.as_ref().unwrap();
        let source = parsed_file.source.as_bytes();

        let queries = match language {
            SourceLanguage::Rust => &self.rust_queries,
            SourceLanguage::Python => &self.python_queries,
            SourceLanguage::JavaScript => &self.javascript_queries,
        };

        if let Some(query) = queries.get(query_name) {
            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(query, tree.root_node(), source);
            
            matches.map(|m| QueryMatch {
                pattern_index: m.pattern_index,
                captures: m.captures.iter().map(|c| {
                    Capture {
                        node: c.node,
                        index: c.index,
                        name: query.capture_names()[c.index as usize].clone()
                    }
                }).collect()
            }).collect()
        } else {
            Vec::new()
        }
    }
}

/// Represents a query match result
pub struct QueryMatch {
    pub pattern_index: u32,
    pub captures: Vec<Capture>,
}

/// Represents a captured node in a query match
pub struct Capture {
    pub node: tree_sitter::Node,
    pub index: u32,
    pub name: String,
}

// Rust dependency queries
const RUST_USE_QUERY: &str = "(use_declaration path: (_) @path)";

// Python import queries
const PYTHON_IMPORT_QUERY: &str = r#"
(import_statement name: (_) @path)
(import_from_statement module_name: (_) @path)
"#;

// JavaScript/TypeScript import queries
const JS_IMPORT_QUERY: &str = r#"
(import_statement source: (string) @path)
(call_expression
  function: (identifier) @func
  arguments: (arguments (string) @path)
  (#eq? @func "require"))
"#;
