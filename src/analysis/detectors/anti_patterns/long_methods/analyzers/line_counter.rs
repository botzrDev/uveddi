//! Line counting functionality for methods

use crate::analysis::AnalysisError;
#[cfg(feature = "tree-sitter")]
use crate::ast::tree_sitter::Node;

/// Line counter for various line metrics
pub struct LineCounter;

impl LineCounter {
    /// Calculate logical lines of code (excluding comments and blanks)
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_logical_loc(node: &Node, source: &[u8]) -> u32 {
        let source_text = match node.utf8_text(source) {
            Ok(text) => text,
            Err(_) => return 0,
        };

        let mut logical_lines = 0;
        for line in source_text.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Skip comment lines
            if Self::is_comment_line(trimmed) {
                continue;
            }

            logical_lines += 1;
        }

        logical_lines
    }

    /// Count physical lines (including blanks and comments)
    pub fn count_physical_lines(source_text: &str) -> u32 {
        source_text.lines().count() as u32
    }

    /// Count statement nodes in an AST
    #[cfg(feature = "tree-sitter")]
    pub fn count_statements(node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        let mut count = 0;
        Self::count_statements_recursive(node, &mut count);
        Ok(count)
    }

    #[cfg(feature = "tree-sitter")]
    fn count_statements_recursive(node: &Node, count: &mut u32) {
        let node_kind = node.kind();

        // Check if this is a statement node
        if Self::is_statement_node(node_kind) {
            *count += 1;
        }

        // Recursively process children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                Self::count_statements_recursive(&child, count);
            }
        }
    }

    /// Check if a node kind represents a statement
    fn is_statement_node(node_kind: &str) -> bool {
        matches!(
            node_kind,
            "let_declaration"
                | "assignment_expression"
                | "expression_statement"
                | "return_expression"
                | "break_expression"
                | "continue_expression"
                | "for_expression"
                | "while_expression"
                | "loop_expression"
                | "if_expression"
                | "match_expression"
                | "call_expression"
                | "macro_invocation"
                | "await_expression"
                | "yield_expression"
                | "throw_statement"
                | "try_statement"
        )
    }

    /// Check if a line is a comment
    fn is_comment_line(line: &str) -> bool {
        let trimmed = line.trim();

        // Single-line comments
        if trimmed.starts_with("//")
            || trimmed.starts_with("#")
            || trimmed.starts_with("/*") && trimmed.ends_with("*/")
        {
            return true;
        }

        // Multi-line comment markers
        if trimmed == "/*" || trimmed == "*/" || trimmed.starts_with("*") {
            return true;
        }

        // Python docstrings (simplified check)
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            return true;
        }

        false
    }

    /// Count lines excluding documentation blocks
    pub fn count_code_lines(source_text: &str, language: &str) -> u32 {
        let mut in_doc_block = false;
        let mut code_lines = 0;

        let doc_start = match language {
            "rust" => "///",
            "python" => "\"\"\"",
            "javascript" | "typescript" => "/**",
            _ => "",
        };

        let doc_end = match language {
            "python" => "\"\"\"",
            "javascript" | "typescript" => "*/",
            _ => "",
        };

        for line in source_text.lines() {
            let trimmed = line.trim();

            if !doc_start.is_empty() && trimmed.starts_with(doc_start) {
                in_doc_block = true;
            }

            if !in_doc_block && !trimmed.is_empty() && !Self::is_comment_line(trimmed) {
                code_lines += 1;
            }

            if !doc_end.is_empty() && trimmed.ends_with(doc_end) {
                in_doc_block = false;
            }
        }

        code_lines
    }
}
