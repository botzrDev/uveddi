use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct CommandSanitizerFilter {
    treat_whitelist_as_safe: bool,
}

impl CommandSanitizerFilter {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            treat_whitelist_as_safe: config.false_positive.enable_heuristic_filtering,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(CommandSanitizerFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for CommandSanitizerFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut retained = Vec::with_capacity(issues.len());

        for issue in issues {
            let whitelisted = issue
                .metadata
                .false_positive_indicators
                .iter()
                .any(|indicator| indicator.contains("command_whitelisted"));

            if whitelisted && self.treat_whitelist_as_safe {
                debug!("Suppressing whitelisted command execution issue");
                continue;
            }

            retained.push(issue);
        }

        Ok(retained)
    }
}
