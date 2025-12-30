use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct HeaderValidationFilter {
    strict_headers: bool,
}

impl HeaderValidationFilter {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            strict_headers: config.false_positive.enable_contextual_filtering,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(HeaderValidationFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for HeaderValidationFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut retained = Vec::with_capacity(issues.len());

        for mut issue in issues {
            let header_related = issue
                .metadata
                .tags
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case("header"));

            // If strict_headers is enabled, boost header-related issues rather than filtering others
            // All issues pass through, but header-related ones get a confidence boost
            if header_related && self.strict_headers {
                let before = issue.confidence_score;
                issue.confidence_score = (before + 0.1).min(1.0);
                debug!(
                    "Boosted header-related issue confidence from {:.2} to {:.2}",
                    before, issue.confidence_score
                );
            }

            retained.push(issue);
        }

        Ok(retained)
    }
}
