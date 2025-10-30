use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;
use tracing::debug;

pub struct JavaScriptLanguageFilter {
    confidence_penalty: f64,
}

impl JavaScriptLanguageFilter {
    pub fn new(_: &ValidationConfig) -> Self {
        Self {
            confidence_penalty: 0.1,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(JavaScriptLanguageFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for JavaScriptLanguageFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut adjusted = Vec::with_capacity(issues.len());

        for mut issue in issues {
            if matches!(
                issue.language,
                Some(SourceLanguage::JavaScript | SourceLanguage::TypeScript)
            ) {
                issue.confidence_score =
                    (issue.confidence_score - self.confidence_penalty).max(0.0);
            } else {
                debug!("Passing non-JS issue through JS language filter");
            }
            adjusted.push(issue);
        }

        Ok(adjusted)
    }
}
