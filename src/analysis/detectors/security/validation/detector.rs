use crate::analysis::detectors::security::config::FalsePositiveConfig;
use crate::analysis::detectors::security::core::ConfidenceScore;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecuritySeverity, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use super::{
    config::ValidationConfig, input_validation, language_support, output_validation, sanitizers,
    types::ValidationStage,
};

/// Main validation engine that coordinates validation strategies.
pub struct ValidationEngine {
    config: ValidationConfig,
    false_positive_mitigator: FalsePositiveMitigator,
    confidence_calculator: ConfidenceCalculator,
    bayesian_optimizer: Option<BayesianOptimizer>,
}

impl ValidationEngine {
    pub fn new(config: FalsePositiveConfig) -> Result<Self, AnalysisError> {
        let validation_config = ValidationConfig::from_false_positive(config.clone());
        let false_positive_mitigator = FalsePositiveMitigator::new(validation_config.clone())?;
        let bayesian_optimizer = if validation_config.bayesian_enabled() {
            Some(BayesianOptimizer::new()?)
        } else {
            None
        };

        Ok(Self {
            config: validation_config,
            false_positive_mitigator,
            confidence_calculator: ConfidenceCalculator::new(),
            bayesian_optimizer,
        })
    }

    pub async fn validate_issues(
        &self,
        mut issues: Vec<SecurityIssue>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        info!("Validating {} security issues", issues.len());
        issues = self.false_positive_mitigator.filter_issues(issues).await?;
        info!("After false positive filtering: {} issues remain", issues.len());
        for issue in &mut issues {
            let new_confidence = self
                .confidence_calculator
                .calculate_confidence(issue)
                .await?;
            issue.confidence_score = new_confidence.final_score;
        }
        if let Some(optimizer) = &self.bayesian_optimizer {
            issues = optimizer.optimize_results(issues).await?;
        }
        issues.retain(|issue| issue.confidence_score >= self.config.min_confidence_threshold());
        info!("After validation: {} high-confidence issues remain", issues.len());
        Ok(issues)
    }

    pub async fn cross_validate(
        &self,
        detector_results: HashMap<String, Vec<SecurityIssue>>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        info!("Cross-validating results from {} detectors", detector_results.len());

        let mut validated_issues = Vec::new();
        let mut issue_agreements: HashMap<String, Vec<(String, SecurityIssue)>> = HashMap::new();

        for (detector_name, issues) in detector_results.iter() {
            for issue in issues {
                let issue_key = self.generate_issue_key(issue);
                issue_agreements
                    .entry(issue_key)
                    .or_insert_with(Vec::new)
                    .push((detector_name.clone(), issue.clone()));
            }
        }

        for (_issue_key, agreements) in issue_agreements {
            if agreements.len() >= self.config.agreement_threshold() {
                let mut representative_issue = agreements[0].1.clone();
                representative_issue.confidence_score = (representative_issue
                    .confidence_score
                    + (agreements.len() as f64 - 1.0) * 0.1)
                    .min(1.0);
                for (detector_name, _) in agreements {
                    representative_issue = representative_issue.with_detector(detector_name);
                }

                validated_issues.push(representative_issue);
            } else if agreements.len() == 1 && agreements[0].1.confidence_score >= 0.8 {
                validated_issues.push(agreements[0].1.clone());
            }
        }
        info!("Cross-validation completed: {} issues validated", validated_issues.len());
        Ok(validated_issues)
    }

    fn generate_issue_key(&self, issue: &SecurityIssue) -> String {
        format!(
            "{}:{}:{}:{}",
            issue.location.file_path.display(),
            issue.location.start_line,
            issue.issue_type.to_string(),
            issue.title
        )
    }
}

/// False positive mitigation system with modular stages.
pub struct FalsePositiveMitigator {
    config: ValidationConfig,
    input_stage: Option<ValidationStage>,
    output_stage: Option<ValidationStage>,
    sanitization_stage: Option<ValidationStage>,
    language_stage: Option<ValidationStage>,
}

impl FalsePositiveMitigator {
    pub fn new(config: ValidationConfig) -> Result<Self, AnalysisError> {
        let input_stage = stage_or_none(config.enable_input_validation, || {
            input_validation::build_stage(&config)
        });
        let output_stage = stage_or_none(config.enable_output_validation, || {
            output_validation::build_stage(&config)
        });
        let sanitization_stage = stage_or_none(config.enable_sanitization_validation, || {
            sanitizers::build_stage(&config)
        });
        let language_stage = stage_or_none(config.enable_language_support, || {
            language_support::build_stage(&config)
        });

        Ok(Self {
            config,
            input_stage,
            output_stage,
            sanitization_stage,
            language_stage,
        })
    }

    pub async fn filter_issues(
        &self,
        mut issues: Vec<SecurityIssue>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!(
            "Applying false positive mitigation to {} issues",
            issues.len()
        );

        for stage in [
            &self.input_stage,
            &self.output_stage,
            &self.sanitization_stage,
            &self.language_stage,
        ] {
            if let Some(stage) = stage {
                issues = stage.run(issues).await?;
            }
        }

        debug!(
            "False positive mitigation completed: {} issues remain",
            issues.len()
        );
        Ok(issues)
    }

    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }
}

fn stage_or_none<F>(enabled: bool, builder: F) -> Option<ValidationStage>
where
    F: FnOnce() -> ValidationStage,
{
    if enabled {
        let stage = builder();
        (!stage.is_empty()).then_some(stage)
    } else {
        None
    }
}

pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn calculate_confidence(
        &self,
        issue: &SecurityIssue,
    ) -> Result<ConfidenceScore, AnalysisError> {
        let mut confidence = ConfidenceScore::new();

        let detection_confidence = match issue.vulnerability_type {
            VulnerabilityType::Static => 0.8,
            VulnerabilityType::Dynamic => 0.9,
            VulnerabilityType::Dependency => 0.95,
            VulnerabilityType::Configuration => 0.85,
            VulnerabilityType::AiInferred => 0.6,
            VulnerabilityType::Hybrid => 0.75,
        };
        confidence = confidence.with_detection_method(detection_confidence);

        let evidence_strength = match issue.severity {
            SecuritySeverity::Critical => 0.9,
            SecuritySeverity::High => 0.8,
            SecuritySeverity::Medium => 0.6,
            SecuritySeverity::Low => 0.4,
            SecuritySeverity::Info => 0.2,
        };
        confidence = confidence.with_evidence_strength(evidence_strength);

        let architectural_score = if issue.correlation_id.is_some() {
            0.3
        } else {
            0.0
        };
        confidence = confidence.with_architectural_context(architectural_score);

        let cross_validation_score = if issue.detected_by.len() > 1 {
            (issue.detected_by.len() as f64 * 0.2).min(1.0)
        } else {
            0.0
        };
        confidence = confidence.with_cross_validation(cross_validation_score);

        Ok(confidence)
    }
}

pub struct BayesianOptimizer {
    optimization_history: Vec<OptimizationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OptimizationResult {
    parameters: HashMap<String, f64>,
    performance_score: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl BayesianOptimizer {
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            optimization_history: Vec::new(),
        })
    }

    pub async fn optimize_results(
        &self,
        issues: Vec<SecurityIssue>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!("Applying Bayesian optimization to {} issues", issues.len());
        Ok(issues)
    }

    pub async fn update_optimization(
        &mut self,
        parameters: HashMap<String, f64>,
        performance_score: f64,
    ) -> Result<(), AnalysisError> {
        let result = OptimizationResult {
            parameters,
            performance_score,
            timestamp: chrono::Utc::now(),
        };

        self.optimization_history.push(result);
        if self.optimization_history.len() > 1000 {
            self.optimization_history.drain(0..100);
        }

        Ok(())
    }
}
