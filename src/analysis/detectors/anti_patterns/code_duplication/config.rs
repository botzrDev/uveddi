//! Configuration for code duplication detection

use crate::analysis::detectors::base::{BaseConfig, DetectorConfig};
use crate::analysis::AnalysisError;
use serde::{Deserialize, Serialize};

/// Configuration for the code duplication detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationConfig {
    /// Base detector configuration
    pub base: BaseConfig,

    // Core detection parameters
    /// Minimum number of tokens a code block must have
    pub min_tokens: usize,
    /// Minimum number of lines a code block must have
    pub min_lines: usize,
    /// Similarity threshold for Type-3 (near-miss) clones
    pub similarity_threshold: f64,
    /// Length of token window for rolling hash fingerprints
    pub fingerprint_length: usize,

    // Normalization options
    /// Replace identifiers with placeholders for Type-2 detection
    pub ignore_identifiers: bool,
    /// Replace literals with placeholders for Type-2 detection
    pub ignore_literals: bool,
    /// Ignore whitespace and formatting differences
    pub ignore_whitespace: bool,
    /// Ignore comments when comparing blocks
    pub ignore_comments: bool,

    // Advanced analysis settings
    /// Enable Control Flow Graph analysis for semantic detection
    pub enable_cfg_analysis: bool,
    /// Enable semantic feature extraction for Type-4 detection
    pub enable_semantic_features: bool,
    /// Weight for CFG similarity in overall score
    pub cfg_similarity_weight: f64,
    /// Threshold for semantic similarity (Type-4 detection)
    pub semantic_similarity_threshold: f64,
    /// Number of iterations for Weisfeiler-Lehman kernel
    pub wl_kernel_iterations: usize,
    /// Maximum CFG nodes to process (performance limit)
    pub max_cfg_nodes: usize,

    // Performance settings
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Timeout per file in seconds
    pub timeout_per_file_secs: u64,

    // Filtering options
    /// Minimum clone pair similarity to report
    pub min_clone_similarity: f64,
    /// Maximum number of clone pairs to report
    pub max_clone_pairs: Option<usize>,
    /// Skip files matching these patterns
    pub skip_patterns: Vec<String>,
    /// Only analyze files matching these patterns
    pub include_patterns: Vec<String>,
}

impl DuplicationConfig {
    /// Creates a new configuration with sensible defaults
    pub fn new() -> Self {
        Self {
            base: BaseConfig::new(),
            min_tokens: 50,
            min_lines: 10,
            similarity_threshold: 0.8,
            fingerprint_length: 16,
            ignore_identifiers: true,
            ignore_literals: true,
            ignore_whitespace: true,
            ignore_comments: true,
            enable_cfg_analysis: false,
            enable_semantic_features: false,
            cfg_similarity_weight: 0.3,
            semantic_similarity_threshold: 0.7,
            wl_kernel_iterations: 3,
            max_cfg_nodes: 1000,
            enable_parallel: true,
            max_memory_mb: 512,
            timeout_per_file_secs: 30,
            min_clone_similarity: 0.8,
            max_clone_pairs: Some(100),
            skip_patterns: vec![
                "*.min.js".to_string(),
                "*.bundle.js".to_string(),
                "node_modules/**".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
            ],
            include_patterns: vec![],
        }
    }

    /// Creates a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            min_tokens: 100,
            min_lines: 20,
            similarity_threshold: 0.9,
            fingerprint_length: 12,
            enable_cfg_analysis: false,
            enable_semantic_features: false,
            max_memory_mb: 256,
            timeout_per_file_secs: 10,
            max_clone_pairs: Some(50),
            ..Self::new()
        }
    }

    /// Creates a configuration optimized for thoroughness
    pub fn thorough_analysis() -> Self {
        Self {
            min_tokens: 20,
            min_lines: 5,
            similarity_threshold: 0.6,
            fingerprint_length: 20,
            enable_cfg_analysis: true,
            enable_semantic_features: true,
            max_memory_mb: 1024,
            timeout_per_file_secs: 120,
            max_clone_pairs: None,
            ..Self::new()
        }
    }

    /// Sets the minimum token threshold
    pub fn with_min_tokens(mut self, min_tokens: usize) -> Self {
        self.min_tokens = min_tokens;
        self
    }

    /// Sets the similarity threshold
    pub fn with_similarity_threshold(mut self, threshold: f64) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// Enables or disables CFG analysis
    pub fn with_cfg_analysis(mut self, enabled: bool) -> Self {
        self.enable_cfg_analysis = enabled;
        self
    }

    /// Enables or disables semantic features
    pub fn with_semantic_features(mut self, enabled: bool) -> Self {
        self.enable_semantic_features = enabled;
        self
    }

    /// Sets the maximum clone pairs to report
    pub fn with_max_clone_pairs(mut self, max: Option<usize>) -> Self {
        self.max_clone_pairs = max;
        self
    }

    /// Adds a skip pattern
    pub fn with_skip_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.skip_patterns.push(pattern.into());
        self
    }

    /// Adds an include pattern
    pub fn with_include_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.include_patterns.push(pattern.into());
        self
    }

    /// Returns whether a file should be skipped based on patterns
    pub fn should_skip_file(&self, file_path: &str) -> bool {
        // Check skip patterns
        for pattern in &self.skip_patterns {
            if glob_match(pattern, file_path) {
                return true;
            }
        }

        // Check include patterns (if any)
        if !self.include_patterns.is_empty() {
            for pattern in &self.include_patterns {
                if glob_match(pattern, file_path) {
                    return false;
                }
            }
            return true; // Not in include list
        }

        false
    }
}

impl Default for DuplicationConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectorConfig for DuplicationConfig {
    fn validate(&self) -> Result<(), AnalysisError> {
        fn config_error(field: &str, value: impl Into<String>, reason: impl Into<String>) -> AnalysisError {
            AnalysisError::ConfigurationError {
                field: field.to_string(),
                value: value.into(),
                reason: reason.into(),
            }
        }

        // Validate base configuration
        self.base.validate()?;

        // Validate specific parameters
        if self.min_tokens == 0 {
            return Err(config_error(
                "min_tokens",
                self.min_tokens.to_string(),
                "min_tokens must be greater than 0",
            ));
        }

        if self.min_lines == 0 {
            return Err(config_error(
                "min_lines",
                self.min_lines.to_string(),
                "min_lines must be greater than 0",
            ));
        }

        if !(0.0..=1.0).contains(&self.similarity_threshold) {
            return Err(config_error(
                "similarity_threshold",
                self.similarity_threshold.to_string(),
                "similarity_threshold must be between 0.0 and 1.0",
            ));
        }

        if !(0.0..=1.0).contains(&self.semantic_similarity_threshold) {
            return Err(config_error(
                "semantic_similarity_threshold",
                self.semantic_similarity_threshold.to_string(),
                "semantic_similarity_threshold must be between 0.0 and 1.0",
            ));
        }

        if !(0.0..=1.0).contains(&self.cfg_similarity_weight) {
            return Err(config_error(
                "cfg_similarity_weight",
                self.cfg_similarity_weight.to_string(),
                "cfg_similarity_weight must be between 0.0 and 1.0",
            ));
        }

        if self.fingerprint_length == 0 {
            return Err(config_error(
                "fingerprint_length",
                self.fingerprint_length.to_string(),
                "fingerprint_length must be greater than 0",
            ));
        }

        if self.max_memory_mb == 0 {
            return Err(config_error(
                "max_memory_mb",
                self.max_memory_mb.to_string(),
                "max_memory_mb must be greater than 0",
            ));
        }

        Ok(())
    }

    fn merge(&mut self, other: Self) {
        self.base.merge(other.base);

        // Take more restrictive values for thresholds
        self.min_tokens = self.min_tokens.max(other.min_tokens);
        self.min_lines = self.min_lines.max(other.min_lines);
        self.similarity_threshold = self.similarity_threshold.max(other.similarity_threshold);
        self.min_clone_similarity = self.min_clone_similarity.max(other.min_clone_similarity);

        // Take more restrictive memory/performance limits
        self.max_memory_mb = self.max_memory_mb.min(other.max_memory_mb);
        self.timeout_per_file_secs = self.timeout_per_file_secs.min(other.timeout_per_file_secs);

        // Merge patterns
        self.skip_patterns.extend(other.skip_patterns);
        self.include_patterns.extend(other.include_patterns);

        // Take more conservative analysis settings
        self.enable_cfg_analysis = self.enable_cfg_analysis || other.enable_cfg_analysis;
        self.enable_semantic_features =
            self.enable_semantic_features || other.enable_semantic_features;
    }

    fn default() -> Self {
        Self::new()
    }
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    // Simple implementation - in a real system you'd use a proper glob library
    if pattern.contains('*') {
        if pattern == "*" {
            return true;
        }
        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            return text.starts_with(prefix);
        }
        if pattern.starts_with("*.") {
            let suffix = &pattern[1..];
            return text.ends_with(suffix);
        }
    }
    pattern == text
}
