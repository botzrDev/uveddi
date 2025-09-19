//! Malicious pattern behavior detection and matching logic
//!
//! This module implements the pattern matching engine for malicious behavior detection,
//! including signature matching, threat assessment, and security issue generation.

use super::signatures::{load_default_patterns, MaliciousPattern, MaliciousPatternType, ThreatLevel};
use super::super::{PatternConfig, PatternMatch, PatternMatcher};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use std::collections::HashMap;

/// Database of malicious patterns with behavior analysis capabilities
pub struct MaliciousPatternDatabase {
    patterns: Vec<MaliciousPattern>,
    config: PatternConfig,
}

impl MaliciousPatternDatabase {
    pub fn new(config: PatternConfig) -> Self {
        let patterns = load_default_patterns();
        Self { patterns, config }
    }

    /// Match malicious patterns in context
    pub async fn match_malicious_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for pattern in &self.patterns {
            let matches = self.find_malicious_matches(pattern, context).await?;
            for pattern_match in matches {
                let issue = self.create_issue_from_match(pattern, &pattern_match, context);
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    async fn find_malicious_matches(
        &self,
        pattern: &MaliciousPattern,
        context: &SecurityContext,
    ) -> Result<Vec<PatternMatch>, AnalysisError> {
        let mut matches = Vec::new();
        let content = &context.content;

        // Search for signature matches
        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            // Check signatures (high confidence)
            for signature in &pattern.signatures {
                if line_lower.contains(&signature.to_lowercase()) {
                    let pattern_match = PatternMatch {
                        pattern_name: pattern.name.clone(),
                        pattern_type: format!("{:?}", pattern.pattern_type),
                        confidence_score: pattern.confidence_base,
                        line_number: line_num + 1,
                        column_start: 0,
                        column_end: line.len(),
                        matched_text: line.to_string(),
                    };
                    matches.push(pattern_match);
                    break; // Only one match per line per pattern
                }
            }

            // Check indicators (lower confidence)
            for indicator in &pattern.indicators {
                if line_lower.contains(&indicator.to_lowercase()) {
                    let pattern_match = PatternMatch {
                        pattern_name: pattern.name.clone(),
                        pattern_type: format!("{:?}", pattern.pattern_type),
                        confidence_score: pattern.confidence_base * 0.7, // Reduce confidence for indicators
                        line_number: line_num + 1,
                        column_start: 0,
                        column_end: line.len(),
                        matched_text: line.to_string(),
                    };
                    matches.push(pattern_match);
                    break; // Only one match per line per pattern
                }
            }
        }

        Ok(matches)
    }

    fn create_issue_from_match(
        &self,
        pattern: &MaliciousPattern,
        pattern_match: &PatternMatch,
        context: &SecurityContext,
    ) -> SecurityIssue {
        let location = SecurityLocation::new(
            context.file_path.clone(),
            pattern_match.line_number as i32,
            pattern_match.line_number as i32,
        ).with_columns(
            pattern_match.column_start as i32,
            pattern_match.column_end as i32,
        );

        SecurityIssue::new(
            SecurityIssueType::PotentialMaliciousAgent,
            VulnerabilityType::Static,
            format!("Malicious Pattern Detected: {}", pattern.name),
            format!(
                "{} (Threat Level: {:?})",
                pattern.description, pattern.threat_level
            ),
            location,
        )
        .with_severity(pattern.severity.clone())
        .with_confidence(pattern_match.confidence_score)
        .with_remediation(format!(
            "Immediately investigate and remove suspected {} code",
            pattern.pattern_type.to_string().to_lowercase()
        ))
        .with_detector("MaliciousPatternDatabase".to_string())
    }

    pub fn add_pattern(&mut self, pattern: MaliciousPattern) {
        self.patterns.push(pattern);
    }

    pub fn get_patterns(&self) -> &[MaliciousPattern] {
        &self.patterns
    }

    pub fn get_patterns_by_type(&self, pattern_type: &MaliciousPatternType) -> Vec<&MaliciousPattern> {
        self.patterns
            .iter()
            .filter(|p| std::mem::discriminant(&p.pattern_type) == std::mem::discriminant(pattern_type))
            .collect()
    }

    pub fn get_critical_patterns(&self) -> Vec<&MaliciousPattern> {
        self.patterns
            .iter()
            .filter(|p| matches!(p.threat_level, ThreatLevel::Critical))
            .collect()
    }
}

#[async_trait]
impl PatternMatcher for MaliciousPatternDatabase {
    async fn match_patterns(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError> {
        self.match_malicious_patterns(context).await
    }

    fn matcher_name(&self) -> &'static str {
        "MaliciousPatternDatabase"
    }

    fn can_match(&self, _context: &SecurityContext) -> bool {
        true // Can match all contexts
    }
}