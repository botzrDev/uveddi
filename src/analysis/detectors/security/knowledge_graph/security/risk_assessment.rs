//! Risk assessment and scoring for security analysis

use crate::analysis::detectors::security::knowledge_graph::types::{CodeEntity, StructuralSemanticGraph};
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Risk assessment engine for security analysis
pub struct RiskAssessmentEngine {
    risk_factors: Vec<RiskFactor>,
    scoring_weights: ScoringWeights,
}

impl RiskAssessmentEngine {
    pub fn new() -> Self {
        Self {
            risk_factors: vec![
                RiskFactor::Complexity,
                RiskFactor::Exposure,
                RiskFactor::Privilege,
                RiskFactor::DataSensitivity,
                RiskFactor::NetworkAccess,
            ],
            scoring_weights: ScoringWeights::default(),
        }
    }

    /// Assess risk for entities and vulnerabilities
    pub async fn assess_risk(
        &self,
        entities: &[CodeEntity],
        vulnerabilities: &[SecurityIssue],
        graph: &StructuralSemanticGraph,
    ) -> Result<RiskAssessmentResult, AnalysisError> {
        let mut entity_risks = HashMap::new();
        let mut vulnerability_risks = HashMap::new();

        // Assess entity risks
        for entity in entities {
            let risk_score = self.calculate_entity_risk(entity, graph).await?;
            entity_risks.insert(entity.id.clone(), risk_score);
        }

        // Assess vulnerability risks
        for vulnerability in vulnerabilities {
            let risk_score = self.calculate_vulnerability_risk(vulnerability, &entity_risks).await?;
            if let Some(vuln_id) = &vulnerability.id {
                vulnerability_risks.insert(vuln_id.clone(), risk_score);
            }
        }

        let overall_risk = self.calculate_overall_risk(&entity_risks, &vulnerability_risks);

        Ok(RiskAssessmentResult {
            entity_risks,
            vulnerability_risks,
            overall_risk,
            risk_distribution: self.calculate_risk_distribution(&entity_risks, &vulnerability_risks),
        })
    }

    async fn calculate_entity_risk(
        &self,
        entity: &CodeEntity,
        graph: &StructuralSemanticGraph,
    ) -> Result<EntityRiskScore, AnalysisError> {
        let mut risk_components = HashMap::new();

        // Calculate risk for each factor
        for factor in &self.risk_factors {
            let score = match factor {
                RiskFactor::Complexity => self.assess_complexity_risk(entity, graph),
                RiskFactor::Exposure => self.assess_exposure_risk(entity),
                RiskFactor::Privilege => self.assess_privilege_risk(entity),
                RiskFactor::DataSensitivity => self.assess_data_sensitivity_risk(entity),
                RiskFactor::NetworkAccess => self.assess_network_access_risk(entity),
            };
            risk_components.insert(format!("{:?}", factor), score);
        }

        // Calculate weighted total
        let total_score = risk_components.iter()
            .map(|(factor, &score)| {
                let weight = self.get_factor_weight(factor);
                score * weight
            })
            .sum::<f64>();

        Ok(EntityRiskScore {
            entity_id: entity.id.clone(),
            total_score,
            risk_components,
            risk_level: self.score_to_risk_level(total_score),
        })
    }

    async fn calculate_vulnerability_risk(
        &self,
        vulnerability: &SecurityIssue,
        entity_risks: &HashMap<String, EntityRiskScore>,
    ) -> Result<VulnerabilityRiskScore, AnalysisError> {
        let base_score = match vulnerability.severity {
            crate::analysis::detectors::security::types::SecuritySeverity::Critical => 0.9,
            crate::analysis::detectors::security::types::SecuritySeverity::High => 0.7,
            crate::analysis::detectors::security::types::SecuritySeverity::Medium => 0.5,
            crate::analysis::detectors::security::types::SecuritySeverity::Low => 0.3,
            crate::analysis::detectors::security::types::SecuritySeverity::Info => 0.1,
        };

        // Adjust based on context (entity risk, exploitability, etc.)
        let context_multiplier = 1.0; // Would be calculated based on affected entities
        let exploitability = self.assess_exploitability(vulnerability);
        let impact = self.assess_impact(vulnerability);

        let total_score = (base_score * context_multiplier * exploitability * impact).min(1.0);

        Ok(VulnerabilityRiskScore {
            vulnerability_id: vulnerability.id.clone().unwrap_or_default(),
            total_score,
            base_score,
            exploitability,
            impact,
            risk_level: self.score_to_risk_level(total_score),
        })
    }

    fn assess_complexity_risk(&self, entity: &CodeEntity, graph: &StructuralSemanticGraph) -> f64 {
        // Count connections as complexity indicator
        let connections = graph.edges.iter()
            .filter(|edge| edge.from == entity.id || edge.to == entity.id)
            .count();

        // Get cyclomatic complexity if available
        let cyclomatic = entity.metadata.get("cyclomatic_complexity")
            .and_then(|v| serde_json::from_value::<f64>(v.clone()).ok())
            .unwrap_or(1.0);

        ((connections as f64 / 20.0) + (cyclomatic / 10.0)).min(1.0)
    }

    fn assess_exposure_risk(&self, entity: &CodeEntity) -> f64 {
        let name_lower = entity.name.to_lowercase();
        let path_str = entity.location.file_path.display().to_string().to_toLowerCase();

        let mut exposure_score = 0.0;

        // Public API exposure
        if name_lower.contains("api") || name_lower.contains("endpoint") {
            exposure_score += 0.5;
        }

        // External visibility
        if path_str.contains("public") || path_str.contains("api") {
            exposure_score += 0.3;
        }

        // Network-related exposure
        if name_lower.contains("http") || name_lower.contains("web") {
            exposure_score += 0.4;
        }

        exposure_score.min(1.0)
    }

    fn assess_privilege_risk(&self, entity: &CodeEntity) -> f64 {
        let name_lower = entity.name.to_lowercase();

        if name_lower.contains("admin") || name_lower.contains("root") {
            0.9
        } else if name_lower.contains("auth") || name_lower.contains("secure") {
            0.7
        } else if name_lower.contains("user") {
            0.4
        } else {
            0.2
        }
    }

    fn assess_data_sensitivity_risk(&self, entity: &CodeEntity) -> f64 {
        let name_lower = entity.name.to_lowercase();

        let mut sensitivity_score = 0.0;

        if name_lower.contains("password") || name_lower.contains("secret") {
            sensitivity_score += 0.8;
        }
        if name_lower.contains("key") || name_lower.contains("token") {
            sensitivity_score += 0.7;
        }
        if name_lower.contains("personal") || name_lower.contains("private") {
            sensitivity_score += 0.6;
        }
        if name_lower.contains("financial") || name_lower.contains("payment") {
            sensitivity_score += 0.9;
        }

        sensitivity_score.min(1.0)
    }

    fn assess_network_access_risk(&self, entity: &CodeEntity) -> f64 {
        let name_lower = entity.name.to_lowercase();

        if name_lower.contains("socket") || name_lower.contains("network") {
            0.8
        } else if name_lower.contains("http") || name_lower.contains("tcp") {
            0.6
        } else if name_lower.contains("local") {
            0.2
        } else {
            0.0
        }
    }

    fn assess_exploitability(&self, vulnerability: &SecurityIssue) -> f64 {
        // Base exploitability on vulnerability type and confidence
        let type_exploitability = match vulnerability.issue_type {
            crate::analysis::detectors::security::types::SecurityIssueType::Injection => 0.9,
            crate::analysis::detectors::security::types::SecurityIssueType::BufferOverflow => 0.8,
            crate::analysis::detectors::security::types::SecurityIssueType::PrivilegeEscalation => 0.7,
            crate::analysis::detectors::security::types::SecurityIssueType::WeakAuthentication => 0.8,
            _ => 0.5,
        };

        (type_exploitability * vulnerability.confidence).min(1.0)
    }

    fn assess_impact(&self, vulnerability: &SecurityIssue) -> f64 {
        // Impact assessment based on vulnerability characteristics
        let base_impact = match vulnerability.issue_type {
            crate::analysis::detectors::security::types::SecurityIssueType::ExposureOfSensitiveInformation => 0.9,
            crate::analysis::detectors::security::types::SecurityIssueType::PrivilegeEscalation => 0.9,
            crate::analysis::detectors::security::types::SecurityIssueType::BufferOverflow => 0.8,
            crate::analysis::detectors::security::types::SecurityIssueType::Injection => 0.8,
            _ => 0.5,
        };

        // Adjust based on CWE severity if available
        if vulnerability.cwe_id.is_some() {
            base_impact * 1.1 // Boost for documented CVE patterns
        } else {
            base_impact
        }.min(1.0)
    }

    fn calculate_overall_risk(
        &self,
        entity_risks: &HashMap<String, EntityRiskScore>,
        vulnerability_risks: &HashMap<String, VulnerabilityRiskScore>,
    ) -> f64 {
        let entity_risk_avg = if entity_risks.is_empty() {
            0.0
        } else {
            entity_risks.values().map(|r| r.total_score).sum::<f64>() / entity_risks.len() as f64
        };

        let vuln_risk_avg = if vulnerability_risks.is_empty() {
            0.0
        } else {
            vulnerability_risks.values().map(|r| r.total_score).sum::<f64>() / vulnerability_risks.len() as f64
        };

        // Weighted combination
        (entity_risk_avg * 0.4 + vuln_risk_avg * 0.6).min(1.0)
    }

    fn calculate_risk_distribution(
        &self,
        entity_risks: &HashMap<String, EntityRiskScore>,
        vulnerability_risks: &HashMap<String, VulnerabilityRiskScore>,
    ) -> RiskDistribution {
        let total_items = entity_risks.len() + vulnerability_risks.len();

        if total_items == 0 {
            return RiskDistribution::default();
        }

        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for risk in entity_risks.values() {
            match risk.risk_level {
                RiskLevel::Critical => critical += 1,
                RiskLevel::High => high += 1,
                RiskLevel::Medium => medium += 1,
                RiskLevel::Low => low += 1,
            }
        }

        for risk in vulnerability_risks.values() {
            match risk.risk_level {
                RiskLevel::Critical => critical += 1,
                RiskLevel::High => high += 1,
                RiskLevel::Medium => medium += 1,
                RiskLevel::Low => low += 1,
            }
        }

        RiskDistribution {
            critical: critical as f64 / total_items as f64,
            high: high as f64 / total_items as f64,
            medium: medium as f64 / total_items as f64,
            low: low as f64 / total_items as f64,
        }
    }

    fn get_factor_weight(&self, factor: &str) -> f64 {
        match factor {
            "Complexity" => self.scoring_weights.complexity,
            "Exposure" => self.scoring_weights.exposure,
            "Privilege" => self.scoring_weights.privilege,
            "DataSensitivity" => self.scoring_weights.data_sensitivity,
            "NetworkAccess" => self.scoring_weights.network_access,
            _ => 0.2,
        }
    }

    fn score_to_risk_level(&self, score: f64) -> RiskLevel {
        if score >= 0.8 {
            RiskLevel::Critical
        } else if score >= 0.6 {
            RiskLevel::High
        } else if score >= 0.4 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }
}

/// Risk factors for assessment
#[derive(Debug, Clone)]
pub enum RiskFactor {
    Complexity,
    Exposure,
    Privilege,
    DataSensitivity,
    NetworkAccess,
}

/// Scoring weights for risk factors
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub complexity: f64,
    pub exposure: f64,
    pub privilege: f64,
    pub data_sensitivity: f64,
    pub network_access: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            complexity: 0.2,
            exposure: 0.3,
            privilege: 0.25,
            data_sensitivity: 0.2,
            network_access: 0.05,
        }
    }
}

/// Risk assessment result
#[derive(Debug, Clone)]
pub struct RiskAssessmentResult {
    pub entity_risks: HashMap<String, EntityRiskScore>,
    pub vulnerability_risks: HashMap<String, VulnerabilityRiskScore>,
    pub overall_risk: f64,
    pub risk_distribution: RiskDistribution,
}

/// Risk score for individual entities
#[derive(Debug, Clone)]
pub struct EntityRiskScore {
    pub entity_id: String,
    pub total_score: f64,
    pub risk_components: HashMap<String, f64>,
    pub risk_level: RiskLevel,
}

/// Risk score for vulnerabilities
#[derive(Debug, Clone)]
pub struct VulnerabilityRiskScore {
    pub vulnerability_id: String,
    pub total_score: f64,
    pub base_score: f64,
    pub exploitability: f64,
    pub impact: f64,
    pub risk_level: RiskLevel,
}

/// Risk level classification
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// Distribution of risk levels
#[derive(Debug, Clone)]
pub struct RiskDistribution {
    pub critical: f64,
    pub high: f64,
    pub medium: f64,
    pub low: f64,
}

impl Default for RiskDistribution {
    fn default() -> Self {
        Self {
            critical: 0.0,
            high: 0.0,
            medium: 0.0,
            low: 0.0,
        }
    }
}