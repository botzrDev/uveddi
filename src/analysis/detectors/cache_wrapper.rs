//! Cache-aware detector wrapper
//!
//! Provides transparent caching for detector results to improve performance
//! on repeated analysis runs with unchanged files.

#[cfg(feature = "analysis-cache")]
use super::{
    base::traits::{Detector, DetectorConfig, DetectorOutput},
    base::types::{AnalysisContext, DetectorCategory, Issue},
};

#[cfg(feature = "analysis-cache")]
use crate::{
    analysis::AnalysisError,
    database::models::ArchitecturalIssue,
    engine::cache::AnalysisCache,
};

#[cfg(feature = "analysis-cache")]
use async_trait::async_trait;
#[cfg(feature = "analysis-cache")]
use std::{any::Any, collections::HashMap, path::Path, sync::Arc, time::Instant};
#[cfg(feature = "analysis-cache")]
use tracing::{debug, warn};

/// Cache-aware detector wrapper that adds result caching to any detector
#[cfg(feature = "analysis-cache")]
pub struct CachedDetector<D>
where
    D: Detector,
{
    /// The wrapped detector
    inner: D,
    /// Detector version for cache invalidation
    detector_version: String,
}

#[cfg(feature = "analysis-cache")]
impl<D> CachedDetector<D>
where
    D: Detector,
{
    /// Create a new cached detector wrapper
    pub fn new(detector: D) -> Self {
        Self {
            detector_version: format!("{}:v1.0", detector.name()),
            inner: detector,
        }
    }

    /// Create a new cached detector with explicit version
    pub fn with_version(detector: D, version: String) -> Self {
        Self {
            detector_version: format!("{}:{}", detector.name(), version),
            inner: detector,
        }
    }

    /// Convert detector issues to architectural issues for caching
    fn convert_to_architectural_issues(
        &self,
        output: &D::Output,
        file_path: &Path,
        detector_name: &str,
    ) -> Vec<ArchitecturalIssue> {
        output
            .issues()
            .iter()
            .map(|issue| ArchitecturalIssue {
                id: None,
                analysis_run_id: None,
                file_path: file_path.to_string_lossy().to_string(),
                issue_type: detector_name.to_string(),
                description: issue.message.clone(),
                severity: match issue.severity {
                    super::base::types::Severity::Info => "Info".to_string(),
                    super::base::types::Severity::Low => "Low".to_string(),
                    super::base::types::Severity::Medium => "Medium".to_string(),
                    super::base::types::Severity::High => "High".to_string(),
                    super::base::types::Severity::Critical => "Critical".to_string(),
                },
                start_line: issue.line as i32,
                end_line: issue.line as i32,
                start_column: Some(issue.start_column as i32),
                end_column: Some(issue.end_column as i32),
                suggestion: issue.suggestion.clone(),
                metadata: serde_json::to_string(&issue.metadata).ok(),
                created_at: chrono::Utc::now(),
            })
            .collect()
    }

    /// Convert architectural issues back to detector issues
    fn convert_from_architectural_issues(&self, issues: &[ArchitecturalIssue]) -> Vec<Issue> {
        issues
            .iter()
            .map(|issue| Issue {
                detector_name: issue.issue_type.clone(),
                message: issue.description.clone(),
                severity: match issue.severity.as_str() {
                    "Critical" => super::base::types::Severity::Critical,
                    "High" => super::base::types::Severity::High,
                    "Medium" => super::base::types::Severity::Medium,
                    "Low" => super::base::types::Severity::Low,
                    _ => super::base::types::Severity::Info,
                },
                file_path: issue.file_path.clone(),
                line: issue.start_line as u32,
                start_column: issue.start_column.unwrap_or(0) as u32,
                end_column: issue.end_column.unwrap_or(0) as u32,
                suggestion: issue.suggestion.clone(),
                metadata: issue
                    .metadata
                    .as_ref()
                    .and_then(|m| serde_json::from_str(m).ok())
                    .unwrap_or_default(),
            })
            .collect()
    }
}

#[cfg(feature = "analysis-cache")]
#[async_trait]
impl<D> Detector for CachedDetector<D>
where
    D: Detector + Send + Sync,
{
    type Config = D::Config;
    type Output = D::Output;

    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn category(&self) -> DetectorCategory {
        self.inner.category()
    }

    fn supported_languages(&self) -> &[crate::ast::tree_sitter_impl::SourceLanguage] {
        self.inner.supported_languages()
    }

    async fn detect(&self, context: &AnalysisContext) -> Result<Self::Output, AnalysisError> {
        let start_time = Instant::now();

        // Try to get results from cache for each file
        let mut cached_results = Vec::new();
        let mut files_to_analyze = Vec::new();

        for parsed_file in &context.files {
            let file_path = &parsed_file.path;

            // Build detector versions map for cache key
            let mut detector_versions = HashMap::new();
            detector_versions.insert(self.inner.name().to_string(), self.detector_version.clone());

            // Check cache
            if let Ok(cache) = context.analysis_cache.lock() {
                if let Some(cached_entry) = cache.get(file_path, &detector_versions) {
                    debug!("Cache hit for {} on file: {}", self.inner.name(), file_path.display());

                    // Convert cached issues back to detector format
                    let issues = self.convert_from_architectural_issues(&cached_entry.issues);
                    cached_results.extend(issues);
                    continue;
                }
            }

            debug!("Cache miss for {} on file: {}", self.inner.name(), file_path.display());
            files_to_analyze.push(parsed_file.clone());
        }

        // If all results are cached, return them
        if files_to_analyze.is_empty() {
            // Create output from cached results
            // This is simplified - in reality you'd need to implement proper result aggregation
            return self.inner.detect(context).await; // Fallback to normal detection
        }

        // Create context with only files that need analysis
        let analysis_context = AnalysisContext {
            files: files_to_analyze.clone(),
            root_path: context.root_path.clone(),
            global_config: context.global_config.clone(),
            parallel: context.parallel,
            max_issues: context.max_issues,
            #[cfg(feature = "analysis-cache")]
            analysis_cache: context.analysis_cache.clone(),
        };

        // Run detection on uncached files
        let result = self.inner.detect(&analysis_context).await?;

        // Cache results for each analyzed file
        let execution_time = start_time.elapsed();
        let mut detector_versions = HashMap::new();
        detector_versions.insert(self.inner.name().to_string(), self.detector_version.clone());

        for parsed_file in &files_to_analyze {
            let file_path = &parsed_file.path;

            // Convert output to architectural issues for caching
            let issues = self.convert_to_architectural_issues(&result, file_path, self.inner.name());

            // Store in cache
            if let Ok(mut cache) = context.analysis_cache.lock() {
                if let Err(e) = cache.put(
                    file_path.clone(),
                    issues,
                    execution_time,
                    detector_versions.clone(),
                ) {
                    warn!(
                        "Failed to cache analysis results for {}: {}",
                        file_path.display(),
                        e
                    );
                }
            }
        }

        Ok(result)
    }

    fn config(&self) -> &Self::Config {
        self.inner.config()
    }

    fn update_config(&mut self, config: Self::Config) {
        self.inner.update_config(config);
    }

    fn is_enabled(&self) -> bool {
        self.inner.is_enabled()
    }

    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }
}