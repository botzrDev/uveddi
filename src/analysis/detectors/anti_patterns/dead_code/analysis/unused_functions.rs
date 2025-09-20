//! Unused function detection

use std::collections::{HashMap, HashSet};

use crate::analysis::AnalysisError;
use crate::analysis::detectors::anti_patterns::dead_code::types::{
    DeadCodeIssue, Severity, Symbol, SymbolType,
};

/// Detects unused functions in code
pub struct UnusedFunctionAnalyzer;

impl UnusedFunctionAnalyzer {
    /// Analyzes symbols for unused functions
    pub fn analyze(
        symbols: &[Symbol],
        references: &HashSet<String>,
    ) -> Result<Vec<DeadCodeIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Build call graph
        let call_graph = Self::build_call_graph(symbols, references);

        for symbol in symbols {
            if matches!(
                symbol.symbol_type,
                SymbolType::Function | SymbolType::AsyncFunction
            ) && !symbol.is_live
            {
                if !references.contains(&symbol.name) && !Self::is_entry_point(symbol) {
                    let severity = Self::calculate_severity(symbol, &call_graph);

                    issues.push(DeadCodeIssue {
                        symbol: symbol.clone(),
                        severity,
                        removal_safe: Self::is_safe_to_remove(symbol, &call_graph),
                        related_symbols: Self::find_related_functions(symbol, &call_graph),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Builds a call graph from symbols and references
    fn build_call_graph(
        symbols: &[Symbol],
        references: &HashSet<String>,
    ) -> HashMap<String, Vec<String>> {
        let mut graph = HashMap::new();

        for symbol in symbols {
            let callers = symbols
                .iter()
                .filter(|s| s.code_snippet.contains(&symbol.name))
                .map(|s| s.name.clone())
                .collect();

            graph.insert(symbol.name.clone(), callers);
        }

        graph
    }

    /// Checks if a function is an entry point
    fn is_entry_point(symbol: &Symbol) -> bool {
        // Common entry point names
        matches!(
            symbol.name.as_str(),
            "main" | "init" | "setup" | "teardown" | "start" | "run"
        ) || symbol.name.starts_with("test_")
            || symbol.name.starts_with("bench_")
    }

    /// Calculates severity based on function characteristics
    fn calculate_severity(
        symbol: &Symbol,
        call_graph: &HashMap<String, Vec<String>>,
    ) -> Severity {
        if symbol.is_exported {
            Severity::Medium
        } else if call_graph.get(&symbol.name).map_or(0, |v| v.len()) > 0 {
            Severity::Low
        } else {
            Severity::High
        }
    }

    /// Checks if a function is safe to remove
    fn is_safe_to_remove(
        symbol: &Symbol,
        call_graph: &HashMap<String, Vec<String>>,
    ) -> bool {
        !symbol.is_exported && call_graph.get(&symbol.name).map_or(true, |v| v.is_empty())
    }

    /// Finds related functions that might also be dead code
    fn find_related_functions(
        symbol: &Symbol,
        call_graph: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        call_graph
            .get(&symbol.name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}