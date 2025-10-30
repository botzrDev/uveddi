use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct EncodingValidationFilter {
    boost_factor: f64,
}

impl EncodingValidationFilter {
    pub fn new(_: &ValidationConfig) -> Self {
        Self { boost_factor: 0.1 }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(EncodingValidationFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for EncodingValidationFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut adjusted = Vec::with_capacity(issues.len());

        for mut issue in issues {
            let has_encoding_tag = issue
                .metadata
                .tags
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case("encoding"));

            if has_encoding_tag {
                let before = issue.confidence_score;
                issue.confidence_score = (before + self.boost_factor).min(1.0);
                debug!(
                    "Boosted encoding issue confidence from {:.2} to {:.2}",
                    before, issue.confidence_score
                );
            }

            adjusted.push(issue);
        }

        Ok(adjusted)
    }
}
