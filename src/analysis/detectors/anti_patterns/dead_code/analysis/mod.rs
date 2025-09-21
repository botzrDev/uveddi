//! Analysis orchestration module for dead code detection

pub mod unreachable;
pub mod unused_functions;
pub mod unused_imports;
pub mod unused_variables;

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::Node;
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::collections::HashSet;

use super::types::{DeadCodeIssue, Symbol};

/// Main analysis coordinator for dead code detection
pub struct AnalysisOrchestrator;

impl AnalysisOrchestrator {
    /// Orchestrates dead code analysis across all analyzers
    pub async fn analyze(
        symbols: Vec<Symbol>,
        references: HashSet<String>,
        language: SourceLanguage,
    ) -> Result<Vec<DeadCodeIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Analyze unreachable code
        let unreachable_issues = unreachable::UnreachableCodeAnalyzer::analyze(&symbols)?;
        issues.extend(unreachable_issues);

        // Analyze unused variables
        let unused_var_issues =
            unused_variables::UnusedVariableAnalyzer::analyze(&symbols, &references)?;
        issues.extend(unused_var_issues);

        // Analyze unused functions
        let unused_func_issues =
            unused_functions::UnusedFunctionAnalyzer::analyze(&symbols, &references)?;
        issues.extend(unused_func_issues);

        // Analyze unused imports
        let unused_import_issues =
            unused_imports::UnusedImportAnalyzer::analyze(&symbols, &references)?;
        issues.extend(unused_import_issues);

        Ok(issues)
    }

    /// Performs reachability analysis from entry points
    pub fn mark_reachable_symbols(
        symbols: &mut Vec<Symbol>,
        references: &HashSet<String>,
        entry_points: Vec<String>,
    ) {
        // Mark entry points as live
        for symbol in symbols.iter_mut() {
            if entry_points.contains(&symbol.name) {
                symbol.is_live = true;
            }
        }

        // Iteratively mark symbols referenced by live symbols
        let mut changed = true;
        while changed {
            changed = false;
            for symbol in symbols.iter_mut() {
                if !symbol.is_live && references.contains(&symbol.name) {
                    // Check if any live symbol references this one
                    // This is simplified; real implementation would track who references whom
                    symbol.is_live = true;
                    changed = true;
                }
            }
        }
    }
}

/// Trait for language-specific analysis
pub trait LanguageAnalyzer: Send + Sync {
    fn analyze_node(&self, node: &Node, source: &[u8]) -> Vec<Symbol>;
    fn detect_entry_points(&self, symbols: &[Symbol]) -> Vec<String>;
}
