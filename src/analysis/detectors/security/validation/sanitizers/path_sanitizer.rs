use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct PathSanitizerFilter {
    strict_mode: bool,
}

impl PathSanitizerFilter {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            strict_mode: config.false_positive.enable_contextual_filtering,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(PathSanitizerFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for PathSanitizerFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut retained = Vec::with_capacity(issues.len());

        for issue in issues {
            let normalized = issue
                .metadata
                .false_positive_indicators
                .iter()
                .any(|indicator| indicator.contains("path_normalized"));

            if normalized && !self.strict_mode {
                debug!("Dropping normalized path issue during sanitization");
                continue;
            }

            retained.push(issue);
        }

        Ok(retained)
    }
}
