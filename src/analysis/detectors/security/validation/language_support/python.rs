use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;

pub struct PythonLanguageFilter;

impl PythonLanguageFilter {
    pub fn new(_: &ValidationConfig) -> Self {
        Self
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(PythonLanguageFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for PythonLanguageFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let adjusted = issues
            .into_iter()
            .map(|mut issue| {
                if matches!(issue.language, Some(SourceLanguage::Python)) {
                    issue.confidence_score = issue.confidence_score.max(0.4);
                }
                issue
            })
            .collect();

        Ok(adjusted)
    }
}
