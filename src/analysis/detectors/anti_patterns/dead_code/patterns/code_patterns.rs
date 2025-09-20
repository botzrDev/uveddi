//! Dead code pattern matching

use crate::analysis::AnalysisError;
use crate::analysis::detectors::anti_patterns::dead_code::types::Symbol;

use super::{DeadCodePattern, PatternType};

/// Matches code patterns that indicate dead code
pub struct CodePatternMatcher;

impl CodePatternMatcher {
    /// Find dead code patterns in symbols
    pub fn find_patterns(symbols: &[Symbol]) -> Result<Vec<DeadCodePattern>, AnalysisError> {
        let mut patterns = Vec::new();

        // Check for unused private symbols
        patterns.extend(Self::find_unused_private(symbols));

        // Check for orphaned helper functions
        patterns.extend(Self::find_orphaned_helpers(symbols));

        // Check for duplicate definitions
        patterns.extend(Self::find_duplicate_definitions(symbols));

        Ok(patterns)
    }

    /// Find unused private symbols
    fn find_unused_private(symbols: &[Symbol]) -> Vec<DeadCodePattern> {
        let mut patterns = Vec::new();

        for symbol in symbols {
            if !symbol.is_exported && !symbol.is_live {
                patterns.push(DeadCodePattern {
                    pattern_type: PatternType::UnusedPrivate,
                    symbols: vec![symbol.clone()],
                    confidence: 0.9,
                    description: format!("Private {} '{}' is never used", symbol.symbol_type, symbol.name),
                });
            }
        }

        patterns
    }

    /// Find orphaned helper functions
    fn find_orphaned_helpers(symbols: &[Symbol]) -> Vec<DeadCodePattern> {
        let mut patterns = Vec::new();

        for symbol in symbols {
            if Self::is_helper_pattern(&symbol.name) && !symbol.is_live {
                patterns.push(DeadCodePattern {
                    pattern_type: PatternType::OrphanedHelper,
                    symbols: vec![symbol.clone()],
                    confidence: 0.8,
                    description: format!("Helper {} '{}' appears to be orphaned", symbol.symbol_type, symbol.name),
                });
            }
        }

        patterns
    }

    /// Find duplicate definitions
    fn find_duplicate_definitions(symbols: &[Symbol]) -> Vec<DeadCodePattern> {
        let mut patterns = Vec::new();
        let mut seen_names = std::collections::HashMap::new();

        for symbol in symbols {
            seen_names
                .entry(&symbol.name)
                .or_insert_with(Vec::new)
                .push(symbol.clone());
        }

        for (name, duplicates) in seen_names {
            if duplicates.len() > 1 {
                patterns.push(DeadCodePattern {
                    pattern_type: PatternType::DuplicateDefinition,
                    symbols: duplicates,
                    confidence: 0.7,
                    description: format!("Multiple definitions of '{}'", name),
                });
            }
        }

        patterns
    }

    /// Check if a name follows helper function patterns
    fn is_helper_pattern(name: &str) -> bool {
        name.starts_with("_helper")
            || name.starts_with("helper_")
            || name.ends_with("_helper")
            || name.contains("_internal_")
            || name.starts_with("do_")
            || name.starts_with("impl_")
    }

    /// Match AST patterns for dead code
    pub fn match_ast_pattern(symbol: &Symbol) -> Option<AstPattern> {
        // Check for common dead code AST patterns
        if symbol.code_snippet.contains("// TODO") || symbol.code_snippet.contains("// FIXME") {
            return Some(AstPattern::TodoCode);
        }

        if symbol.code_snippet.contains("#[cfg(never)]") || symbol.code_snippet.contains("if false") {
            return Some(AstPattern::ConditionallyDead);
        }

        if symbol.code_snippet.contains("unreachable!") || symbol.code_snippet.contains("panic!") {
            return Some(AstPattern::Unreachable);
        }

        None
    }
}

/// AST patterns that indicate dead code
#[derive(Debug, Clone, PartialEq)]
pub enum AstPattern {
    TodoCode,
    ConditionallyDead,
    Unreachable,
    Deprecated,
    TestOnly,
}