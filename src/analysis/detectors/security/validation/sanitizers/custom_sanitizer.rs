use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct CustomSanitizerFilter {
    max_similar_issues: usize,
}

impl CustomSanitizerFilter {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            max_similar_issues: config.false_positive.max_similar_issues,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(CustomSanitizerFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for CustomSanitizerFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut retained = Vec::with_capacity(issues.len());

        for issue in issues {
            let similar_count = issue
                .metadata
                .false_positive_indicators
                .iter()
                .filter(|indicator| indicator.contains("custom_pattern"))
                .count();

            if similar_count > self.max_similar_issues {
                debug!("Dropping issue due to custom sanitization pattern overload");
                continue;
            }

            retained.push(issue);
        }

        Ok(retained)
    }
}
