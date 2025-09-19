//! Pattern modules for agent detection
//!
//! This module contains pattern definitions and matching logic for detecting
//! various types of agent behaviors and malicious patterns.

pub mod agent_patterns;
pub mod malicious_patterns;

pub use agent_patterns::{AgentPattern, AgentPatternDatabase};
pub use malicious_patterns::{MaliciousPattern, MaliciousPatternDatabase};

use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;

/// Trait for pattern matching
pub trait PatternMatcher: Send + Sync {
    /// Match patterns in the given context
    async fn match_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError>;

    /// Get the name of this pattern matcher
    fn matcher_name(&self) -> &'static str;

    /// Check if this matcher can analyze the given context
    fn can_match(&self, context: &SecurityContext) -> bool;
}

/// Pattern matching configuration
#[derive(Debug, Clone)]
pub struct PatternConfig {
    pub enable_fuzzy_matching: bool,
    pub confidence_threshold: f64,
    pub max_pattern_length: usize,
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            enable_fuzzy_matching: true,
            confidence_threshold: 0.5,
            max_pattern_length: 1000,
        }
    }
}

/// Pattern match result
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub pattern_type: String,
    pub confidence_score: f64,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub matched_text: String,
}