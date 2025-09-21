//! Usage pattern analysis for dead code detection

use std::collections::{HashMap, HashSet};

use crate::analysis::detectors::anti_patterns::dead_code::types::{Symbol, SymbolType};
use crate::analysis::AnalysisError;

use super::{DeadCodePattern, PatternType};

/// Analyzes usage patterns to identify dead code
pub struct UsagePatternAnalyzer;

impl UsagePatternAnalyzer {
    /// Analyze usage patterns in symbols
    pub fn analyze(symbols: &[Symbol]) -> Result<Vec<DeadCodePattern>, AnalysisError> {
        let mut patterns = Vec::new();

        // Find unreferenced exports
        patterns.extend(Self::find_unreferenced_exports(symbols));

        // Find test-only usage
        patterns.extend(Self::find_test_only_usage(symbols));

        // Find conditionally dead code
        patterns.extend(Self::find_conditionally_dead(symbols));

        Ok(patterns)
    }

    /// Find exported symbols that are never referenced
    fn find_unreferenced_exports(symbols: &[Symbol]) -> Vec<DeadCodePattern> {
        let mut patterns = Vec::new();

        for symbol in symbols {
            if symbol.is_exported && !symbol.is_live {
                patterns.push(DeadCodePattern {
                    pattern_type: PatternType::UnreferencedExport,
                    symbols: vec![symbol.clone()],
                    confidence: 0.7,
                    description: format!(
                        "Exported {} '{}' is not referenced",
                        symbol.symbol_type, symbol.name
                    ),
                });
            }
        }

        patterns
    }

    /// Find symbols only used in tests
    fn find_test_only_usage(symbols: &[Symbol]) -> Vec<DeadCodePattern> {
        let mut patterns = Vec::new();

        for symbol in symbols {
            if Self::is_test_only(symbol) {
                patterns.push(DeadCodePattern {
                    pattern_type: PatternType::TestOnlyUsage,
                    symbols: vec![symbol.clone()],
                    confidence: 0.6,
                    description: format!(
                        "{} '{}' appears to be used only in tests",
                        symbol.symbol_type, symbol.name
                    ),
                });
            }
        }

        patterns
    }

    /// Find conditionally dead code
    fn find_conditionally_dead(symbols: &[Symbol]) -> Vec<DeadCodePattern> {
        let mut patterns = Vec::new();

        for symbol in symbols {
            if Self::is_conditionally_dead(symbol) {
                patterns.push(DeadCodePattern {
                    pattern_type: PatternType::ConditionallyDead,
                    symbols: vec![symbol.clone()],
                    confidence: 0.5,
                    description: format!(
                        "{} '{}' may be conditionally dead",
                        symbol.symbol_type, symbol.name
                    ),
                });
            }
        }

        patterns
    }

    /// Check if a symbol is only used in tests
    fn is_test_only(symbol: &Symbol) -> bool {
        let path_str = symbol.path.to_string_lossy();
        path_str.contains("test") || path_str.contains("spec") || symbol.name.starts_with("test_")
    }

    /// Check if code is conditionally dead
    fn is_conditionally_dead(symbol: &Symbol) -> bool {
        symbol.code_snippet.contains("#[cfg(")
            || symbol.code_snippet.contains("if cfg!")
            || symbol.code_snippet.contains("#ifdef")
    }

    /// Build a usage graph for cross-reference analysis
    pub fn build_usage_graph(symbols: &[Symbol]) -> UsageGraph {
        let mut graph = UsageGraph::new();

        for symbol in symbols {
            graph.add_symbol(symbol.clone());
        }

        graph
    }

    /// Validate usage patterns for false positives
    pub fn validate_usage(symbol: &Symbol, usage_graph: &UsageGraph) -> bool {
        // Check if symbol is referenced indirectly
        usage_graph.is_indirectly_referenced(&symbol.name)
    }
}

/// Usage graph for tracking symbol dependencies
pub struct UsageGraph {
    nodes: HashMap<String, Symbol>,
    edges: HashMap<String, HashSet<String>>,
}

impl UsageGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.nodes.insert(symbol.name.clone(), symbol);
    }

    pub fn add_edge(&mut self, from: String, to: String) {
        self.edges
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    pub fn is_indirectly_referenced(&self, symbol_name: &str) -> bool {
        // Simplified: check if any node references this symbol
        self.edges.values().any(|refs| refs.contains(symbol_name))
    }
}
