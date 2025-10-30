//! Pattern detection API for dead code analysis

pub mod code_patterns;
pub mod usage_patterns;

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::Node;

use super::types::Symbol;

/// Main pattern detection coordinator
pub struct PatternDetector;

impl PatternDetector {
    /// Detect dead code patterns in symbols
    pub fn detect_patterns(symbols: &[Symbol]) -> Result<Vec<DeadCodePattern>, AnalysisError> {
        let mut patterns = Vec::new();

        // Detect code patterns
        let code_patterns = code_patterns::CodePatternMatcher::find_patterns(symbols)?;
        patterns.extend(code_patterns);

        // Detect usage patterns
        let usage_patterns = usage_patterns::UsagePatternAnalyzer::analyze(symbols)?;
        patterns.extend(usage_patterns);

        Ok(patterns)
    }

    /// Check if a pattern indicates dead code
    pub fn is_dead_code_pattern(pattern: &DeadCodePattern) -> bool {
        matches!(
            pattern.pattern_type,
            PatternType::UnusedPrivate
                | PatternType::UnreferencedExport
                | PatternType::OrphanedHelper
                | PatternType::DuplicateDefinition
        )
    }
}

/// Represents a detected dead code pattern
#[derive(Debug, Clone)]
pub struct DeadCodePattern {
    pub pattern_type: PatternType,
    pub symbols: Vec<Symbol>,
    pub confidence: f64,
    pub description: String,
}

/// Types of dead code patterns
#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    UnusedPrivate,
    UnreferencedExport,
    OrphanedHelper,
    DuplicateDefinition,
    TestOnlyUsage,
    ConditionallyDead,
}

/// Trait for pattern matchers
pub trait PatternMatcher {
    fn matches(&self, symbol: &Symbol) -> bool;
    fn confidence(&self) -> f64;
    fn pattern_type(&self) -> PatternType;
}
