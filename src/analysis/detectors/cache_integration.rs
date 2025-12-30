//! Enhanced detector cache integration
//!
//! This module provides comprehensive cache integration for detectors including:
//! - Detector-specific cache key generation
//! - Per-detector cache hit/miss tracking
//! - Graceful failure handling
//! - Invalidation strategies

use super::base::{
    AnalysisContext, DetectionMetrics, Detector, DetectorCategory, DetectorOutput, Issue, Severity,
};
use crate::analysis::cache::{EnhancedEngineCache, InvalidationStrategy};
use crate::analysis::AnalysisError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cache key generator for detectors
#[derive(Debug, Clone)]
pub struct DetectorCacheKey {
    /// Detector name
    pub detector_name: String,
    /// Detector version
    pub detector_version: String,
    /// File path being analyzed
    pub file_path: PathBuf,
    /// Content hash for the file
    pub content_hash: String,
    /// Configuration hash
    pub config_hash: String,
}

impl DetectorCacheKey {
    /// Generate a unique cache key for the detector and file
    pub fn generate(
        detector_name: &str,
        detector_version: &str,
        file_path: &Path,
        file_content: &str,
        config: &str,
    ) -> Self {
        use sha2::{Digest, Sha256};

        let mut content_hasher = Sha256::new();
        content_hasher.update(file_content);
        let content_hash = format!("{:x}", content_hasher.finalize());

        let mut config_hasher = Sha256::new();
        config_hasher.update(config);
        let config_hash = format!("{:x}", config_hasher.finalize());

        Self {
            detector_name: detector_name.to_string(),
            detector_version: detector_version.to_string(),
            file_path: file_path.to_path_buf(),
            content_hash,
            config_hash,
        }
    }

    /// Convert to a string key for caching
    pub fn to_cache_key(&self) -> String {
        format!(
            "{}_{}_{}_{}_{}",
            self.detector_name,
            self.detector_version,
            self.file_path.to_string_lossy(),
            &self.content_hash[..8], // Use first 8 chars of hash
            &self.config_hash[..8]
        )
    }
}

/// Per-detector cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorCacheStats {
    pub detector_name: String,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_errors: u64,
    pub average_hit_time_ms: f64,
    pub average_miss_time_ms: f64,
    pub memory_usage_bytes: usize,
}

impl DetectorCacheStats {
    pub fn new(detector_name: String) -> Self {
        Self {
            detector_name,
            cache_hits: 0,
            cache_misses: 0,
            cache_errors: 0,
            average_hit_time_ms: 0.0,
            average_miss_time_ms: 0.0,
            memory_usage_bytes: 0,
        }
    }

    /// Calculate hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Update with a cache hit
    pub fn record_hit(&mut self, duration_ms: f64) {
        self.cache_hits += 1;
        // Running average calculation
        let total_hits = self.cache_hits as f64;
        self.average_hit_time_ms =
            ((self.average_hit_time_ms * (total_hits - 1.0)) + duration_ms) / total_hits;
    }

    /// Update with a cache miss
    pub fn record_miss(&mut self, duration_ms: f64) {
        self.cache_misses += 1;
        // Running average calculation
        let total_misses = self.cache_misses as f64;
        self.average_miss_time_ms =
            ((self.average_miss_time_ms * (total_misses - 1.0)) + duration_ms) / total_misses;
    }

    /// Record a cache error
    pub fn record_error(&mut self) {
        self.cache_errors += 1;
    }
}

/// Cache manager for all detectors
pub struct DetectorCacheManager {
    /// The underlying cache implementation
    cache: Arc<EnhancedEngineCache>,
    /// Per-detector statistics
    stats: Arc<RwLock<HashMap<String, DetectorCacheStats>>>,
    /// Cache invalidation strategy
    invalidation_strategy: Box<dyn InvalidationStrategy>,
    /// Whether to fail gracefully on cache errors
    graceful_failures: bool,
}

impl DetectorCacheManager {
    /// Create a new cache manager
    pub async fn new(
        cache: Arc<EnhancedEngineCache>,
        invalidation_strategy: Box<dyn InvalidationStrategy>,
    ) -> Self {
        Self {
            cache,
            stats: Arc::new(RwLock::new(HashMap::new())),
            invalidation_strategy,
            graceful_failures: true,
        }
    }

    /// Get or create stats for a detector
    async fn get_or_create_stats(&self, detector_name: &str) -> DetectorCacheStats {
        let mut stats = self.stats.write().await;
        stats
            .entry(detector_name.to_string())
            .or_insert_with(|| DetectorCacheStats::new(detector_name.to_string()))
            .clone()
    }

    /// Update stats for a detector (creates the entry if it doesn't exist)
    async fn update_stats(
        &self,
        detector_name: &str,
        updater: impl FnOnce(&mut DetectorCacheStats),
    ) {
        let mut stats = self.stats.write().await;
        let detector_stats = stats
            .entry(detector_name.to_string())
            .or_insert_with(|| DetectorCacheStats::new(detector_name.to_string()));
        updater(detector_stats);
    }

    /// Try to get cached results for a detector
    pub async fn get_cached_result<O: DetectorOutput>(
        &self,
        cache_key: &DetectorCacheKey,
    ) -> Option<O> {
        let start_time = Instant::now();
        let detector_name = &cache_key.detector_name;

        // Check if the cache key should be invalidated
        let file_path_str = cache_key.file_path.to_string_lossy().to_string();
        match self
            .invalidation_strategy
            .should_invalidate(&file_path_str, &cache_key.content_hash)
        {
            Ok(should_invalidate) => {
                if should_invalidate {
                    debug!(
                        "Cache invalidation triggered for detector {} on file {}",
                        detector_name,
                        cache_key.file_path.display()
                    );
                    self.update_stats(detector_name, |s| {
                        s.record_miss(start_time.elapsed().as_millis() as f64)
                    })
                    .await;
                    return None;
                }
            }
            Err(e) => {
                warn!(
                    "Cache invalidation check failed for detector {} on file {}: {}",
                    detector_name,
                    cache_key.file_path.display(),
                    e
                );
                // On invalidation error, assume cache should be invalidated to be safe
                self.update_stats(detector_name, |s| {
                    s.record_miss(start_time.elapsed().as_millis() as f64)
                })
                .await;
                return None;
            }
        }

        // Try to get from cache
        match self.cache.get_cached_results(&cache_key.file_path).await {
            Some(issues) => {
                let duration_ms = start_time.elapsed().as_millis() as f64;
                self.update_stats(detector_name, |s| s.record_hit(duration_ms))
                    .await;

                info!(
                    "Cache hit for detector {} on file {} ({}ms)",
                    detector_name,
                    cache_key.file_path.display(),
                    duration_ms
                );

                // Note: This is a simplified conversion - you'd need proper serialization
                // Some(O::from_issues(issues)) // Would need to implement this trait method
                None // Placeholder
            }
            None => {
                let duration_ms = start_time.elapsed().as_millis() as f64;
                self.update_stats(detector_name, |s| s.record_miss(duration_ms))
                    .await;

                debug!(
                    "Cache miss for detector {} on file {}",
                    detector_name,
                    cache_key.file_path.display()
                );
                None
            }
        }
    }

    /// Cache detector results
    pub async fn cache_result(
        &self,
        cache_key: &DetectorCacheKey,
        output: impl DetectorOutput,
    ) -> Result<(), AnalysisError> {
        let start_time = Instant::now();

        // Convert output to architectural issues for caching
        let issues: Vec<crate::database::models::ArchitecturalIssue> = output
            .issues()
            .iter()
            .map(|issue| crate::database::models::ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0,
                anti_pattern_type_id: 1,
                file_path: cache_key.file_path.to_string_lossy().to_string(),
                start_line: Some(issue.start_line as i32),
                end_line: Some(issue.end_line as i32),
                line_number: Some(issue.start_line as i32),
                column_number: Some(issue.start_column as i32),
                message: issue.description.clone(),
                metadata: serde_json::to_string(&issue.metadata).unwrap_or_default(),
                detector_name: cache_key.detector_name.clone(),
                created_at: chrono::Utc::now(),
                severity: issue.severity.to_string(),
                description: issue.description.clone(),
                code_snippet: issue.suggestion.clone(),
                ai_explanation: None,
            })
            .collect();

        // Store in cache
        self.cache.cache_results(&cache_key.file_path, issues).await;

        let duration_ms = start_time.elapsed().as_millis() as f64;
        debug!(
            "Cached results for detector {} on file {} ({}ms)",
            cache_key.detector_name,
            cache_key.file_path.display(),
            duration_ms
        );

        Ok(())
    }

    /// Handle cache failure gracefully
    pub async fn handle_cache_failure(
        &self,
        detector_name: &str,
        error: AnalysisError,
    ) -> Result<(), AnalysisError> {
        self.update_stats(detector_name, |s| s.record_error()).await;

        if self.graceful_failures {
            warn!(
                "Cache failure for detector {}: {}. Continuing without cache.",
                detector_name, error
            );
            Ok(())
        } else {
            Err(error)
        }
    }

    /// Get all detector cache statistics
    pub async fn get_all_stats(&self) -> HashMap<String, DetectorCacheStats> {
        self.stats.read().await.clone()
    }

    /// Get stats for a specific detector
    pub async fn get_detector_stats(&self, detector_name: &str) -> Option<DetectorCacheStats> {
        self.stats.read().await.get(detector_name).cloned()
    }

    /// Clear cache for a specific detector
    pub async fn clear_detector_cache(&self, detector_name: &str) {
        // This would require enhanced cache API to clear by pattern
        // For now, we can only clear all
        info!("Clearing cache for detector: {}", detector_name);
        self.cache.clear_all().await;
    }

    /// Generate a comprehensive cache report
    pub async fn generate_cache_report(&self) -> CacheReport {
        let stats = self.stats.read().await;

        let total_hits: u64 = stats.values().map(|s| s.cache_hits).sum();
        let total_misses: u64 = stats.values().map(|s| s.cache_misses).sum();
        let total_errors: u64 = stats.values().map(|s| s.cache_errors).sum();

        let overall_hit_rate = if total_hits + total_misses > 0 {
            total_hits as f64 / (total_hits + total_misses) as f64
        } else {
            0.0
        };

        let detector_reports: Vec<DetectorCacheReport> = stats
            .values()
            .map(|s| DetectorCacheReport {
                detector_name: s.detector_name.clone(),
                hit_rate: s.hit_rate(),
                total_requests: s.cache_hits + s.cache_misses,
                cache_hits: s.cache_hits,
                cache_misses: s.cache_misses,
                cache_errors: s.cache_errors,
                average_hit_time_ms: s.average_hit_time_ms,
                average_miss_time_ms: s.average_miss_time_ms,
                time_saved_ms: s.cache_hits as f64
                    * (s.average_miss_time_ms - s.average_hit_time_ms),
            })
            .collect();

        CacheReport {
            overall_hit_rate,
            total_cache_hits: total_hits,
            total_cache_misses: total_misses,
            total_cache_errors: total_errors,
            detector_reports,
            generated_at: chrono::Utc::now(),
        }
    }
}

/// Report for individual detector cache performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorCacheReport {
    pub detector_name: String,
    pub hit_rate: f64,
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_errors: u64,
    pub average_hit_time_ms: f64,
    pub average_miss_time_ms: f64,
    pub time_saved_ms: f64,
}

/// Overall cache performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheReport {
    pub overall_hit_rate: f64,
    pub total_cache_hits: u64,
    pub total_cache_misses: u64,
    pub total_cache_errors: u64,
    pub detector_reports: Vec<DetectorCacheReport>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl CacheReport {
    /// Export report as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export report as markdown
    pub fn to_markdown(&self) -> String {
        let mut markdown = String::new();

        markdown.push_str("# Cache Performance Report\n\n");
        markdown.push_str(&format!("Generated: {}\n\n", self.generated_at));

        markdown.push_str("## Overall Statistics\n\n");
        markdown.push_str(&format!(
            "- **Hit Rate**: {:.2}%\n",
            self.overall_hit_rate * 100.0
        ));
        markdown.push_str(&format!("- **Total Hits**: {}\n", self.total_cache_hits));
        markdown.push_str(&format!(
            "- **Total Misses**: {}\n",
            self.total_cache_misses
        ));
        markdown.push_str(&format!(
            "- **Total Errors**: {}\n\n",
            self.total_cache_errors
        ));

        markdown.push_str("## Per-Detector Performance\n\n");
        markdown.push_str("| Detector | Hit Rate | Hits | Misses | Avg Hit (ms) | Avg Miss (ms) | Time Saved (ms) |\n");
        markdown.push_str("|----------|----------|------|--------|--------------|---------------|------------------|\n");

        for report in &self.detector_reports {
            markdown.push_str(&format!(
                "| {} | {:.2}% | {} | {} | {:.2} | {:.2} | {:.2} |\n",
                report.detector_name,
                report.hit_rate * 100.0,
                report.cache_hits,
                report.cache_misses,
                report.average_hit_time_ms,
                report.average_miss_time_ms,
                report.time_saved_ms
            ));
        }

        markdown
    }
}

/// Cache-aware detector wrapper with enhanced functionality
pub struct EnhancedCachedDetector<D: Detector> {
    inner: D,
    cache_manager: Arc<DetectorCacheManager>,
    detector_version: String,
}

impl<D: Detector> EnhancedCachedDetector<D> {
    /// Create new enhanced cached detector
    pub fn new(detector: D, cache_manager: Arc<DetectorCacheManager>) -> Self {
        Self {
            detector_version: format!("{}:v2.0", detector.name()),
            inner: detector,
            cache_manager,
        }
    }

    /// Detect with cache support
    pub async fn detect_with_cache(
        &self,
        context: &AnalysisContext,
    ) -> Result<D::Output, AnalysisError> {
        // This would integrate with the actual detector implementation
        // For now, just delegate to inner detector
        self.inner.detect(context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let key = DetectorCacheKey::generate(
            "test_detector",
            "1.0.0",
            Path::new("/test/file.rs"),
            "fn main() {}",
            "{ \"threshold\": 10 }",
        );

        assert_eq!(key.detector_name, "test_detector");
        assert_eq!(key.detector_version, "1.0.0");
        assert!(!key.content_hash.is_empty());
        assert!(!key.config_hash.is_empty());

        let cache_key_str = key.to_cache_key();
        assert!(cache_key_str.contains("test_detector"));
        assert!(cache_key_str.contains("1.0.0"));
    }

    #[test]
    fn test_detector_cache_stats() {
        let mut stats = DetectorCacheStats::new("test_detector".to_string());

        stats.record_hit(10.0);
        stats.record_hit(20.0);
        stats.record_miss(100.0);

        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.average_hit_time_ms, 15.0);
        assert_eq!(stats.average_miss_time_ms, 100.0);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }

    #[tokio::test]
    async fn test_cache_report_generation() {
        use crate::analysis::cache::{ContentHashInvalidator, EnhancedCacheConfig};

        #[cfg(feature = "prometheus")]
        let cache = Arc::new(
            EnhancedEngineCache::new_with_config(
                EnhancedCacheConfig::default(),
                Arc::new(crate::analysis::cache::metrics::CacheMetrics::new()),
            )
            .await
            .unwrap(),
        );

        #[cfg(not(feature = "prometheus"))]
        let cache = Arc::new(
            EnhancedEngineCache::new_with_config(EnhancedCacheConfig::default())
                .await
                .unwrap(),
        );

        let invalidator = Box::new(ContentHashInvalidator::new());
        let manager = DetectorCacheManager::new(cache, invalidator).await;

        // Simulate some cache operations
        manager
            .update_stats("detector1", |s| {
                s.record_hit(5.0);
                s.record_hit(7.0);
                s.record_miss(50.0);
            })
            .await;

        manager
            .update_stats("detector2", |s| {
                s.record_miss(100.0);
                s.record_miss(120.0);
                s.record_error();
            })
            .await;

        let report = manager.generate_cache_report().await;

        assert_eq!(report.total_cache_hits, 2);
        assert_eq!(report.total_cache_misses, 3);
        assert_eq!(report.total_cache_errors, 1);
        assert_eq!(report.detector_reports.len(), 2);

        // Test markdown generation
        let markdown = report.to_markdown();
        assert!(markdown.contains("Cache Performance Report"));
        assert!(markdown.contains("detector1"));
        assert!(markdown.contains("detector2"));
    }
}
