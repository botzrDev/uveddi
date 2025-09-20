//! Validation orchestration for dead code detection

pub mod dependency_check;
pub mod flow_analysis;

use crate::analysis::AnalysisError;
use std::collections::HashSet;

use super::types::{DeadCodeIssue, Symbol};

/// Main validation coordinator
pub struct ValidationOrchestrator;

impl ValidationOrchestrator {
    /// Validate dead code issues to reduce false positives
    pub async fn validate(
        issues: Vec<DeadCodeIssue>,
        symbols: &[Symbol],
    ) -> Result<Vec<DeadCodeIssue>, AnalysisError> {
        let mut validated = Vec::new();

        for issue in issues {
            if Self::validate_issue(&issue, symbols)? {
                validated.push(issue);
            }
        }

        Ok(validated)
    }

    /// Validate a single issue
    fn validate_issue(issue: &DeadCodeIssue, symbols: &[Symbol]) -> Result<bool, AnalysisError> {
        // Check dependency safety
        if !dependency_check::DependencyValidator::is_safe_to_remove(&issue.symbol, symbols) {
            return Ok(false);
        }

        // Check control flow
        if flow_analysis::FlowAnalyzer::is_reachable(&issue.symbol, symbols) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Check for false positives
    pub fn check_false_positives(
        issues: &[DeadCodeIssue],
        known_patterns: &HashSet<String>,
    ) -> Vec<DeadCodeIssue> {
        issues
            .iter()
            .filter(|issue| !known_patterns.contains(&issue.symbol.name))
            .cloned()
            .collect()
    }

    /// Validate removal safety across all issues
    pub fn validate_batch_removal(issues: &[DeadCodeIssue]) -> ValidationResult {
        let mut safe_to_remove = Vec::new();
        let mut unsafe_to_remove = Vec::new();

        for issue in issues {
            if issue.removal_safe {
                safe_to_remove.push(issue.clone());
            } else {
                unsafe_to_remove.push(issue.clone());
            }
        }

        ValidationResult {
            safe_to_remove,
            unsafe_to_remove,
        }
    }
}

/// Result of validation
pub struct ValidationResult {
    pub safe_to_remove: Vec<DeadCodeIssue>,
    pub unsafe_to_remove: Vec<DeadCodeIssue>,
}

/// Validation confidence levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationConfidence {
    High,    // Very confident the code is dead
    Medium,  // Moderately confident
    Low,     // Low confidence, needs manual review
}