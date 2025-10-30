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
    analysis::AnalysisError, database::models::ArchitecturalIssue, engine::cache::AnalysisCache,
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
            .map(|issue| {
                let mut metadata_map = serde_json::Map::new();
                metadata_map.insert(
                    "issue_type".to_string(),
                    serde_json::Value::String(detector_name.to_string()),
                );
                if let Ok(existing_metadata) =
                    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(
                        &serde_json::to_string(&issue.metadata).unwrap_or("{}".to_string()),
                    )
                {
                    for (k, v) in existing_metadata {
                        metadata_map.insert(k, v);
                    }
                }

                ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: 0,      // TODO: Get from context
                    anti_pattern_type_id: 1, // TODO: Get from detector mapping
                    file_path: file_path.to_string_lossy().to_string(),
                    start_line: Some(issue.start_line as i32),
                    end_line: Some(issue.end_line as i32),
                    line_number: Some(issue.start_line as i32),
                    column_number: Some(issue.start_column as i32),
                    message: issue.description.clone(),
                    metadata: serde_json::to_string(&metadata_map).unwrap_or("{}".to_string()),
                    detector_name: detector_name.to_string(),
                    created_at: chrono::Utc::now(),
                    severity: match issue.severity {
                        super::base::types::Severity::Info => "info".to_string(),
                        super::base::types::Severity::Low => "low".to_string(),
                        super::base::types::Severity::Medium => "medium".to_string(),
                        super::base::types::Severity::High => "high".to_string(),
                        super::base::types::Severity::Critical => "critical".to_string(),
                    },
                    description: issue.description.clone(),
                    code_snippet: issue.suggestion.clone(),
                    ai_explanation: None,
                }
            })
            .collect()
    }

    /// Convert architectural issues back to detector issues
    fn convert_from_architectural_issues(&self, issues: &[ArchitecturalIssue]) -> Vec<Issue> {
        issues
            .iter()
            .map(|issue| {
                let metadata_map: serde_json::Map<String, serde_json::Value> =
                    serde_json::from_str(&issue.metadata).unwrap_or_default();
                let issue_type = metadata_map
                    .get("issue_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&issue.detector_name)
                    .to_string();

                Issue {
                    id: format!("cached-{}", issue.issue_id.unwrap_or(0)),
                    title: issue_type,
                    description: issue.message.clone(),
                    severity: match issue.severity.as_str() {
                        "critical" => super::base::types::Severity::Critical,
                        "high" => super::base::types::Severity::High,
                        "medium" => super::base::types::Severity::Medium,
                        "low" => super::base::types::Severity::Low,
                        _ => super::base::types::Severity::Info,
                    },
                    file_path: issue.file_path.clone().into(),
                    start_line: issue.line_number.unwrap_or(0) as u32,
                    end_line: issue.line_number.unwrap_or(0) as u32,
                    start_column: issue.column_number.unwrap_or(0) as u32,
                    end_column: issue.column_number.unwrap_or(0) as u32,
                    metadata: serde_json::from_str(&issue.metadata).unwrap_or_default(),
                    suggestion: issue.code_snippet.clone(),
                }
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
        let mut cached_results: Vec<Issue> = Vec::new();
        let mut files_to_analyze = Vec::new();

        for parsed_file in &context.files {
            let file_path = &parsed_file.path();

            // Compute file hash for cache key (simple hash based on path and detector version)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            file_path.hash(&mut hasher);
            self.detector_version.hash(&mut hasher);
            let file_hash = hasher.finish();

            // Check cache - disabled in CLI-only mode
            if let Ok(_cache) = context.analysis_cache.lock() {
                // Cache lookup disabled in minimal build
                if false {
                    debug!(
                        "Cache hit for {} on file: {}",
                        self.inner.name(),
                        file_path.display()
                    );

                    // Convert cached issues back to detector format
                    // For now, skip cached results as they're stored as Vec<String>
                    // This would need proper deserialization in production
                    // cached_results.extend(issues);
                    // continue;
                }
            }

            debug!(
                "Cache miss for {} on file: {}",
                self.inner.name(),
                file_path.display()
            );
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
        for parsed_file in &files_to_analyze {
            let file_path = &parsed_file.path();

            // Convert output to architectural issues for caching
            let issues =
                self.convert_to_architectural_issues(&result, file_path, self.inner.name());

            // Compute file hash for cache key
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            file_path.hash(&mut hasher);
            self.detector_version.hash(&mut hasher);
            let _file_hash = hasher.finish();

            // Store in cache - simplified for CLI-only release
            // Note: Full caching implementation removed in CLI-only mode
            if let Ok(_cache) = context.analysis_cache.lock() {
                // Cache storage disabled in minimal build
                debug!("Cache storage disabled in CLI-only mode");
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
