//! Core detection patterns for security analysis
//!
//! This module contains the main pattern detection logic and coordination
//! for identifying security issues through graph analysis.

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, StructuralSemanticGraph, AntiPatternInfo,
};
use crate::analysis::detectors::security::types::{SecurityIssue, SecurityIssueType, SecuritySeverity, VulnerabilityType};
use crate::analysis::AnalysisError;
use std::collections::HashMap;
use tracing::{debug, info};

/// Security pattern detector using graph analysis
pub struct SecurityPatternDetector {
    patterns: Vec<SecurityPattern>,
    detection_config: DetectionConfig,
}

impl SecurityPatternDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_security_patterns(),
            detection_config: DetectionConfig::default(),
        }
    }

    /// Detect security patterns in the knowledge graph
    pub async fn detect_patterns(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<SecurityPatternResult, AnalysisError> {
        info!("Detecting security patterns in graph with {} nodes", graph.nodes.len());

        let mut detected_issues = Vec::new();
        let mut pattern_matches = Vec::new();

        // Apply each security pattern
        for pattern in &self.patterns {
            let pattern_result = self.apply_security_pattern(pattern, graph, entities).await?;
            detected_issues.extend(pattern_result.security_issues);
            pattern_matches.extend(pattern_result.pattern_matches);
        }

        // Analyze pattern interactions
        let interaction_issues = self.analyze_pattern_interactions(&pattern_matches, graph).await?;
        detected_issues.extend(interaction_issues);

        let confidence_score = self.calculate_overall_confidence(&detected_issues);
        Ok(SecurityPatternResult {
            detected_issues,
            pattern_matches,
            confidence_score,
        })
    }

    /// Apply a specific security pattern to the graph
    async fn apply_security_pattern(
        &self,
        pattern: &SecurityPattern,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        match pattern {
            SecurityPattern::PrivilegeEscalation => {
                super::threat_patterns::detect_privilege_escalation(self, graph, entities).await
            }
            SecurityPattern::TrustBoundaryViolation => {
                super::threat_patterns::detect_trust_boundary_violations(self, graph, entities).await
            }
            SecurityPattern::UnsafeDataFlow => {
                super::behavioral_patterns::detect_unsafe_data_flows(self, graph, entities).await
            }
            SecurityPattern::OverPrivilegedComponents => {
                super::behavioral_patterns::detect_overprivileged_components(self, graph, entities).await
            }
            SecurityPattern::WeakAuthenticationPaths => {
                super::threat_patterns::detect_weak_authentication_paths(self, graph, entities).await
            }
            SecurityPattern::InsecureDefaults => {
                super::behavioral_patterns::detect_insecure_defaults(self, graph, entities).await
            }
            SecurityPattern::UnvalidatedInputPaths => {
                super::behavioral_patterns::detect_unvalidated_input_paths(self, graph, entities).await
            }
            SecurityPattern::ExposedInternalAPIs => {
                super::threat_patterns::detect_exposed_internal_apis(self, graph, entities).await
            }
        }
    }

    /// Analyze interactions between detected patterns
    async fn analyze_pattern_interactions(
        &self,
        pattern_matches: &[PatternMatch],
        _graph: &StructuralSemanticGraph,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut interaction_issues = Vec::new();

        // Group patterns by affected entities
        let mut entity_patterns: HashMap<String, Vec<&PatternMatch>> = HashMap::new();
        for pattern_match in pattern_matches {
            for entity_id in &pattern_match.affected_entities {
                entity_patterns.entry(entity_id.clone())
                    .or_default()
                    .push(pattern_match);
            }
        }

        // Look for entities with multiple security patterns (amplification)
        for (entity_id, patterns) in entity_patterns {
            if patterns.len() > 1 {
                let combined_severity = patterns.iter()
                    .map(|p| p.severity)
                    .fold(0.0, |acc, s| acc + s * 0.7); // Amplification factor

                if combined_severity > 1.0 {
                    interaction_issues.push(SecurityIssue {
                        id: Some(format!("pattern_amplification_{}", entity_id)),
                        issue_type: SecurityIssueType::Custom("MultipleVulnerabilities".to_string()),
                        vulnerability_type: VulnerabilityType::Static,
                        severity: SecuritySeverity::Critical,
                        confidence_score: combined_severity.min(1.0),
                        title: "Security Pattern Amplification".to_string(),
                        description: format!(
                            "Entity '{}' affected by multiple security patterns, amplifying risk",
                            entity_id
                        ),
                        location: self.default_location(),
                        language: None,
                        remediation: Some("Address all security patterns affecting this entity as a high priority".to_string()),
                        context: HashMap::new(),
                        metadata: Default::default(),
                        detected_by: vec!["knowledge_graph".to_string()],
                        correlation_id: None,
                    });
                }
            }
        }

        Ok(interaction_issues)
    }

    /// Calculate overall confidence score
    fn calculate_overall_confidence(&self, issues: &[SecurityIssue]) -> f64 {
        if issues.is_empty() {
            1.0
        } else {
            issues.iter().map(|i| i.confidence_score).sum::<f64>() / issues.len() as f64
        }
    }

    /// Default location for issues without specific location
    fn default_location(&self) -> crate::analysis::detectors::security::types::SecurityLocation {
        use std::path::PathBuf;
        crate::analysis::detectors::security::types::SecurityLocation::new(
            PathBuf::from("unknown"),
            0,
            0,
        )
    }

    /// Get default security patterns
    fn default_security_patterns() -> Vec<SecurityPattern> {
        vec![
            SecurityPattern::PrivilegeEscalation,
            SecurityPattern::TrustBoundaryViolation,
            SecurityPattern::UnsafeDataFlow,
            SecurityPattern::OverPrivilegedComponents,
            SecurityPattern::WeakAuthenticationPaths,
            SecurityPattern::InsecureDefaults,
            SecurityPattern::UnvalidatedInputPaths,
            SecurityPattern::ExposedInternalAPIs,
        ]
    }

    // Helper methods exposed for use by other pattern modules
    pub fn get_detection_config(&self) -> &DetectionConfig {
        &self.detection_config
    }
}

/// Security patterns to detect in the knowledge graph
#[derive(Debug, Clone)]
pub enum SecurityPattern {
    PrivilegeEscalation,
    TrustBoundaryViolation,
    UnsafeDataFlow,
    OverPrivilegedComponents,
    WeakAuthenticationPaths,
    InsecureDefaults,
    UnvalidatedInputPaths,
    ExposedInternalAPIs,
}

/// Configuration for pattern detection
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    pub escalation_threshold: f64,
    pub data_flow_threshold: f64,
    pub trust_boundary_threshold: f64,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            escalation_threshold: 0.6,
            data_flow_threshold: 0.7,
            trust_boundary_threshold: 0.8,
        }
    }
}

/// Result of security pattern detection
#[derive(Debug, Clone)]
pub struct SecurityPatternResult {
    pub detected_issues: Vec<SecurityIssue>,
    pub pattern_matches: Vec<PatternMatch>,
    pub confidence_score: f64,
}

/// Result of applying a specific pattern
#[derive(Debug, Clone)]
pub struct PatternApplicationResult {
    pub security_issues: Vec<SecurityIssue>,
    pub pattern_matches: Vec<PatternMatch>,
}

/// A detected pattern match
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_type: String,
    pub affected_entities: Vec<String>,
    pub severity: f64,
    pub description: String,
}