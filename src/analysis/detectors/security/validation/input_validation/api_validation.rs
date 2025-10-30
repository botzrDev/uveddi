use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct ApiValidationFilter {
    enforce_min_confidence: f64,
}

impl ApiValidationFilter {
    pub fn new(min_confidence: f64) -> Self {
        Self {
            enforce_min_confidence: min_confidence.max(0.0),
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(ApiValidationFilter::new(
        config.false_positive.min_confidence_threshold,
    )))
}

#[async_trait]
impl SecurityIssueFilter for ApiValidationFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut adjusted = Vec::with_capacity(issues.len());

        for mut issue in issues {
            let is_api_issue = issue
                .metadata
                .tags
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case("api"));

            if is_api_issue && issue.confidence_score < self.enforce_min_confidence {
                debug!(
                    "Boosting API issue confidence from {:.2} to {:.2}",
                    issue.confidence_score, self.enforce_min_confidence
                );
                issue.confidence_score = self.enforce_min_confidence;
            }

            adjusted.push(issue);
        }

        Ok(adjusted)
    }
}
