//! Core types used across all detectors

use crate::ast::ParsedFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "analysis-cache")]
use crate::engine::cache::AnalysisCache;
#[cfg(feature = "analysis-cache")]
use std::sync::{Arc, Mutex};

/// Categories of detectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DetectorCategory {
    /// Anti-pattern detectors (code smells, design issues)
    AntiPattern,
    /// Security vulnerability detectors
    Security,
    /// Code complexity analyzers
    Complexity,
    /// General pattern detection
    Pattern,
    /// Dependency analysis
    Dependency,
}

/// Severity levels for detected issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational findings
    Info = 0,
    /// Low severity issues
    Low = 1,
    /// Medium severity issues
    Medium = 2,
    /// High severity issues
    High = 3,
    /// Critical issues requiring immediate attention
    Critical = 4,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "Info"),
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

/// Represents a detected issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique identifier for this issue type
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description of the issue
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// File path where the issue was found
    pub file_path: PathBuf,
    /// Starting line number (1-based)
    pub start_line: u32,
    /// Ending line number (1-based)
    pub end_line: u32,
    /// Starting column (0-based)
    pub start_column: u32,
    /// Ending column (0-based)
    pub end_column: u32,
    /// Additional metadata about the issue
    pub metadata: HashMap<String, serde_json::Value>,
    /// Suggested fix or remediation
    pub suggestion: Option<String>,
}

impl Issue {
    /// Creates a new issue
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        severity: Severity,
        file_path: impl Into<PathBuf>,
        start_line: u32,
        end_line: u32,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            severity,
            file_path: file_path.into(),
            start_line,
            end_line,
            start_column: 0,
            end_column: 0,
            metadata: HashMap::new(),
            suggestion: None,
        }
    }

    /// Adds metadata to the issue
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Adds a suggestion to the issue
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Sets column information
    pub fn with_columns(mut self, start_column: u32, end_column: u32) -> Self {
        self.start_column = start_column;
        self.end_column = end_column;
        self
    }
}

/// Metrics collected during detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionMetrics {
    /// Time taken for detection in milliseconds
    pub duration_ms: u64,
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Number of nodes processed
    pub nodes_processed: usize,
    /// Memory usage in bytes
    pub memory_usage_bytes: Option<usize>,
    /// Additional custom metrics
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

impl DetectionMetrics {
    /// Creates new empty metrics
    pub fn new() -> Self {
        Self {
            duration_ms: 0,
            files_analyzed: 0,
            nodes_processed: 0,
            memory_usage_bytes: None,
            custom_metrics: HashMap::new(),
        }
    }

    /// Adds a custom metric
    pub fn add_metric(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom_metrics.insert(key.into(), value);
        self
    }
}

impl Default for DetectionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Context provided to detectors during analysis
#[derive(Debug)]
pub struct AnalysisContext {
    /// The parsed files to analyze
    pub files: Vec<ParsedFile>,
    /// The root directory of the analysis
    pub root_path: PathBuf,
    /// Global configuration options
    pub global_config: HashMap<String, serde_json::Value>,
    /// Whether to enable parallel processing
    pub parallel: bool,
    /// Maximum number of issues to report
    pub max_issues: Option<usize>,
    /// Analysis result cache (if caching is enabled)
    #[cfg(feature = "analysis-cache")]
    pub analysis_cache: Arc<Mutex<AnalysisCache>>,
}

impl AnalysisContext {
    /// Creates a new analysis context
    pub fn new(files: Vec<ParsedFile>, root_path: PathBuf) -> Self {
        #[cfg(feature = "analysis-cache")]
        let analysis_cache = Arc::new(Mutex::new(AnalysisCache::new(5000))); // Default cache size

        Self {
            files,
            root_path,
            global_config: HashMap::new(),
            parallel: true,
            max_issues: None,
            #[cfg(feature = "analysis-cache")]
            analysis_cache,
        }
    }

    /// Adds a global configuration option
    pub fn with_config(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.global_config.insert(key.into(), value);
        self
    }

    /// Sets the parallel processing flag
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Sets the maximum number of issues to report
    pub fn with_max_issues(mut self, max_issues: usize) -> Self {
        self.max_issues = Some(max_issues);
        self
    }

    /// Gets a configuration value
    pub fn get_config<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.global_config
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Gets analysis cache statistics if caching is enabled
    #[cfg(feature = "analysis-cache")]
    pub fn cache_stats(&self) -> Option<crate::engine::cache::analysis_cache::AnalysisCacheStats> {
        if let Ok(cache) = self.analysis_cache.lock() {
            Some(cache.stats().clone())
        } else {
            None
        }
    }

    /// Clears the analysis cache if caching is enabled
    #[cfg(feature = "analysis-cache")]
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.analysis_cache.lock() {
            cache.clear();
        }
    }
}
