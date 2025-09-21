use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct ParameterValidationFilter {
    min_lines: usize,
    #[allow(dead_code)]
    min_tokens: usize,
}

impl ParameterValidationFilter {
    pub fn new(min_lines: usize, min_tokens: usize) -> Self {
        Self {
            min_lines,
            min_tokens,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(ParameterValidationFilter::new(
        config.false_positive.min_lines_threshold,
        config.false_positive.min_tokens_threshold,
    )))
}

#[async_trait]
impl SecurityIssueFilter for ParameterValidationFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let filtered: Vec<SecurityIssue> = issues
            .into_iter()
            .filter(|issue| {
                let line_count = (issue.location.end_line - issue.location.start_line + 1) as usize;
                line_count >= self.min_lines
            })
            .collect();

        debug!(
            "Parameter validation filter retained {} issues",
            filtered.len()
        );
        Ok(filtered)
    }
}
