use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct HtmlSanitizerFilter;

impl HtmlSanitizerFilter {
    pub fn new() -> Self {
        Self
    }
}

pub fn build_filter(_: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(HtmlSanitizerFilter::new()))
}

#[async_trait]
impl SecurityIssueFilter for HtmlSanitizerFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut retained = Vec::with_capacity(issues.len());

        for mut issue in issues {
            let sanitized = issue
                .metadata
                .false_positive_indicators
                .iter()
                .any(|indicator| indicator.contains("html_sanitized"));

            if sanitized {
                debug!("Detected HTML sanitization marker; reducing confidence");
                issue.confidence_score *= 0.7;
            }

            if issue.confidence_score >= 0.2 {
                retained.push(issue);
            }
        }

        Ok(retained)
    }
}
