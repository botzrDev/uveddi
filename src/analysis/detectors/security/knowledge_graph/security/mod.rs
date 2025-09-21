//! Security analysis components for knowledge graph

pub mod patterns;
pub mod risk_assessment;
pub mod vulnerabilities;

pub use patterns::{detect_security_patterns, SecurityPatternDetector, SecurityPatternResult};
pub use risk_assessment::{RiskAssessmentEngine, RiskAssessmentResult};
pub use vulnerabilities::{VulnerabilityAnalysisResult, VulnerabilityAnalyzer};

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, StructuralSemanticGraph,
};
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;

/// Unified security analysis interface
pub struct SecurityAnalyzer {
    pattern_detector: SecurityPatternDetector,
    vulnerability_analyzer: VulnerabilityAnalyzer,
    risk_assessor: RiskAssessmentEngine,
}

impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self {
            pattern_detector: SecurityPatternDetector::new(),
            vulnerability_analyzer: VulnerabilityAnalyzer::new(),
            risk_assessor: RiskAssessmentEngine::new(),
        }
    }

    /// Perform comprehensive security analysis
    pub async fn analyze_security(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<SecurityAnalysisResult, AnalysisError> {
        // Detect security patterns
        let pattern_result = self
            .pattern_detector
            .detect_patterns(graph, entities)
            .await?;

        // Analyze vulnerabilities
        let vulnerability_result = self
            .vulnerability_analyzer
            .analyze_vulnerabilities(graph, entities)
            .await?;

        // Combine all security issues
        let mut all_issues = pattern_result.detected_issues;
        all_issues.extend(vulnerability_result.vulnerabilities);

        // Assess overall risk
        let risk_result = self
            .risk_assessor
            .assess_risk(entities, &all_issues, graph)
            .await?;

        Ok(SecurityAnalysisResult {
            security_issues: all_issues,
            pattern_matches: pattern_result.pattern_matches,
            risk_assessment: risk_result,
            confidence_score: (pattern_result.confidence_score + 0.8) / 2.0, // Average with vulnerability confidence
        })
    }
}

/// Complete security analysis result
#[derive(Debug, Clone)]
pub struct SecurityAnalysisResult {
    pub security_issues: Vec<SecurityIssue>,
    pub pattern_matches: Vec<patterns::PatternMatch>,
    pub risk_assessment: RiskAssessmentResult,
    pub confidence_score: f64,
}
