use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct DataTypeValidationFilter {
    allow_low_confidence: bool,
}

impl DataTypeValidationFilter {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            allow_low_confidence: config.false_positive.enable_ml_filtering,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(DataTypeValidationFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for DataTypeValidationFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut retained = Vec::with_capacity(issues.len());

        for mut issue in issues {
            let has_type_tag = issue
                .metadata
                .tags
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case("type"));

            if has_type_tag {
                issue.confidence_score = issue.confidence_score.max(0.4);
            } else if !self.allow_low_confidence && issue.confidence_score < 0.2 {
                debug!("Filtering data-type issue with low confidence");
                continue;
            }

            retained.push(issue);
        }

        Ok(retained)
    }
}
