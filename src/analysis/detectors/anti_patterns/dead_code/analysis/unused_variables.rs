//! Unused variable detection

use std::collections::HashSet;

use crate::analysis::detectors::anti_patterns::dead_code::types::{
    DeadCodeIssue, Severity, Symbol, SymbolType,
};
use crate::analysis::AnalysisError;

/// Detects unused variables in code
pub struct UnusedVariableAnalyzer;

impl UnusedVariableAnalyzer {
    /// Analyzes symbols for unused variables
    pub fn analyze(
        symbols: &[Symbol],
        references: &HashSet<String>,
    ) -> Result<Vec<DeadCodeIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for symbol in symbols {
            if symbol.symbol_type == SymbolType::Variable && !symbol.is_live {
                if !references.contains(&symbol.name) {
                    let severity = if symbol.is_exported {
                        Severity::Medium
                    } else {
                        Severity::High
                    };

                    issues.push(DeadCodeIssue {
                        symbol: symbol.clone(),
                        severity,
                        removal_safe: !symbol.is_exported,
                        related_symbols: Vec::new(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Checks if a variable is used in specific contexts
    pub fn is_variable_used_in_context(variable_name: &str, context_symbols: &[Symbol]) -> bool {
        for symbol in context_symbols {
            if symbol.code_snippet.contains(variable_name) {
                return true;
            }
        }
        false
    }

    /// Identifies variable declaration patterns
    pub fn identify_declaration_pattern(symbol: &Symbol) -> VariablePattern {
        if symbol.name.starts_with('_') {
            VariablePattern::IntentionallyUnused
        } else if symbol.is_exported {
            VariablePattern::Exported
        } else if symbol.name.chars().all(|c| c.is_uppercase() || c == '_') {
            VariablePattern::Constant
        } else {
            VariablePattern::Local
        }
    }
}

/// Patterns for variable declarations
#[derive(Debug, Clone, PartialEq)]
pub enum VariablePattern {
    Local,
    Exported,
    Constant,
    IntentionallyUnused,
}
