//! Unreachable code detection

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};

use crate::analysis::detectors::anti_patterns::dead_code::types::{
    DeadCodeIssue, Severity, Symbol, SymbolType,
};

/// Detects unreachable code patterns
pub struct UnreachableCodeAnalyzer;

impl UnreachableCodeAnalyzer {
    /// Analyzes symbols for unreachable code patterns
    pub fn analyze(symbols: &[Symbol]) -> Result<Vec<DeadCodeIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for symbol in symbols {
            if Self::is_after_return(symbol) {
                issues.push(DeadCodeIssue {
                    symbol: symbol.clone(),
                    severity: Severity::High,
                    removal_safe: true,
                    related_symbols: Vec::new(),
                });
            }
        }

        Ok(issues)
    }

    /// Checks if code appears after a return statement
    fn is_after_return(symbol: &Symbol) -> bool {
        // Check if the code snippet contains patterns indicating unreachable code
        let snippet = &symbol.code_snippet;

        // Simple heuristic: check for common unreachable patterns
        if snippet.contains("return") && symbol.symbol_type == SymbolType::Function {
            // This is a simplified check
            // Real implementation would use AST analysis
            return false;
        }

        false
    }

    /// Analyzes control flow for unreachable branches
    pub fn analyze_control_flow(node: &Node, source: &[u8]) -> Vec<UnreachablePattern> {
        let mut patterns = Vec::new();

        // Check for code after return statements
        if node.kind() == "return_statement" {
            if let Some(next_sibling) = node.next_sibling() {
                patterns.push(UnreachablePattern {
                    pattern_type: UnreachableType::AfterReturn,
                    start_line: next_sibling.start_position().row as u32,
                    end_line: next_sibling.end_position().row as u32,
                });
            }
        }

        // Check for unreachable catch blocks
        if node.kind() == "catch_clause" {
            // Simplified: would need more sophisticated analysis
            // to determine if catch block is actually unreachable
        }

        // Recursively analyze children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            patterns.extend(Self::analyze_control_flow(&child, source));
        }

        patterns
    }
}

/// Types of unreachable code patterns
#[derive(Debug, Clone)]
pub struct UnreachablePattern {
    pub pattern_type: UnreachableType,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone)]
pub enum UnreachableType {
    AfterReturn,
    AfterThrow,
    UnreachableCatch,
    UnreachableBranch,
}