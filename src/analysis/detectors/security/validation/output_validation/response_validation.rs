use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;

pub struct ResponseValidationFilter {
    min_confidence: f64,
}

impl ResponseValidationFilter {
    pub fn new(min_confidence: f64) -> Self {
        Self { min_confidence }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(ResponseValidationFilter::new(
        config.false_positive.min_confidence_threshold,
    )))
}

#[async_trait]
impl SecurityIssueFilter for ResponseValidationFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        Ok(issues
            .into_iter()
            .filter(|issue| issue.confidence_score >= self.min_confidence * 0.5)
            .collect())
    }
}
