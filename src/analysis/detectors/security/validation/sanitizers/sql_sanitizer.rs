use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct SqlSanitizerFilter {
    require_parameterization: bool,
}

impl SqlSanitizerFilter {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            require_parameterization: config.false_positive.enable_ml_filtering,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(SqlSanitizerFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for SqlSanitizerFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut retained = Vec::with_capacity(issues.len());

        for mut issue in issues {
            let parameterized = issue
                .metadata
                .false_positive_indicators
                .iter()
                .any(|indicator| indicator.contains("sql_parameterized"));

            if parameterized {
                debug!("Detected SQL parameterization; lowering confidence");
                issue.confidence_score *= 0.6;
            }

            if !self.require_parameterization || parameterized || issue.confidence_score >= 0.3 {
                retained.push(issue);
            }
        }

        Ok(retained)
    }
}
