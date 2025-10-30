use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, SecurityIssueFilter,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

pub struct FileValidationFilter {
    exclude_test_files: bool,
    exclude_generated_files: bool,
    exclude_third_party: bool,
}

impl FileValidationFilter {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            exclude_test_files: config.false_positive.exclude_test_files,
            exclude_generated_files: config.false_positive.exclude_generated_files,
            exclude_third_party: config.false_positive.exclude_third_party,
        }
    }

    fn should_exclude(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();

        if self.exclude_test_files
            && (path_str.contains("/test/")
                || path_str.contains("/tests/")
                || path_str.contains("_test.")
                || path_str.contains(".test."))
        {
            return true;
        }

        if self.exclude_generated_files
            && (path_str.contains("generated")
                || path_str.contains(".gen.")
                || path_str.contains("__pycache__")
                || path_str.contains("/target/"))
        {
            return true;
        }

        if self.exclude_third_party
            && (path_str.contains("node_modules")
                || path_str.contains("vendor/")
                || path_str.contains("third_party")
                || path_str.contains(".cargo/"))
        {
            return true;
        }

        false
    }
}

pub fn build_filter(config: &ValidationConfig) -> Option<DynSecurityIssueFilter> {
    Some(Box::new(FileValidationFilter::new(config)))
}

#[async_trait]
impl SecurityIssueFilter for FileValidationFilter {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let filtered: Vec<SecurityIssue> = issues
            .into_iter()
            .filter(|issue| !self.should_exclude(&issue.location.file_path))
            .collect();

        debug!("File validation filter retained {} issues", filtered.len());
        Ok(filtered)
    }
}
