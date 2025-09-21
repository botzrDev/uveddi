use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::debug;

pub struct ContentFrequencyFilter {
    frequency_threshold: usize,
}

impl ContentFrequencyFilter {
    pub fn new(_: &ValidationConfig) -> Self {
        Self {
            frequency_threshold: 10,
        }
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(ContentFrequencyFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for ContentFrequencyFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut pattern_counts = HashMap::new();
        for issue in &issues {
            let key = format!("{}:{}", issue.issue_type.to_string(), issue.title);
            *pattern_counts.entry(key).or_insert(0) += 1;
        }

        let mut filtered = Vec::new();
        for mut issue in issues {
            let key = format!("{}:{}", issue.issue_type.to_string(), issue.title);
            if let Some(&count) = pattern_counts.get(&key) {
                if count > self.frequency_threshold {
                    issue.confidence_score *= 0.8;
                    debug!("Down-ranking frequent pattern {} (count: {})", key, count);
                }
            }

            if issue.confidence_score >= 0.2 {
                filtered.push(issue);
            }
        }

        Ok(filtered)
    }
}
