use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use tracing::debug;

pub struct RustLanguageFilter {
    active: bool,
}

impl RustLanguageFilter {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            active: config.false_positive.enable_heuristic_filtering,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(RustLanguageFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for RustLanguageFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        if !self.active {
            return Ok(issues);
        }

        let mut adjusted = Vec::with_capacity(issues.len());
        for mut issue in issues {
            if matches!(issue.language, Some(SourceLanguage::Rust)) {
                issue.confidence_score = issue.confidence_score.max(0.5);
            } else {
                debug!("Passing non-Rust issue through Rust language filter");
            }
            adjusted.push(issue);
        }
        Ok(adjusted)
    }
}
