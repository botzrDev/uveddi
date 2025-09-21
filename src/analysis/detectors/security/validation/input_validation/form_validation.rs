use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct CommentSuppressionFilter {
    suppression_patterns: Vec<String>,
}

impl CommentSuppressionFilter {
    pub fn new(patterns: Vec<String>) -> Self {
        Self {
            suppression_patterns: patterns,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    if config.false_positive.suppression_comments.is_empty() {
        None
    } else {
        Some(Box::new(CommentSuppressionFilter::new(
            config.false_positive.suppression_comments.clone(),
        )))
    }
}

#[async_trait]
impl SecurityIssueFilter for CommentSuppressionFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut filtered = Vec::new();

        for issue in issues {
            let mut should_suppress = false;

            if let Ok(content) = std::fs::read_to_string(&issue.location.file_path) {
                let lines: Vec<&str> = content.lines().collect();
                let issue_line = (issue.location.start_line - 1) as usize;

                for idx in
                    issue_line.saturating_sub(3)..=issue_line.min(lines.len().saturating_sub(1))
                {
                    if let Some(line) = lines.get(idx) {
                        if self
                            .suppression_patterns
                            .iter()
                            .any(|pattern| line.contains(pattern))
                        {
                            should_suppress = true;
                            debug!("Suppressing issue near line {} via comment", idx + 1);
                            break;
                        }
                    }
                }
            }

            if !should_suppress {
                filtered.push(issue);
            }
        }

        debug!(
            "Comment suppression filter retained {} issues",
            filtered.len()
        );
        Ok(filtered)
    }
}
