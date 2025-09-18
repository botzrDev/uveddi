//! Analysis configuration for target paths, language filters, and detector settings

use crate::analysis::detectors::anti_patterns::dead_code::DeadCodeConfig;
use crate::analysis::detectors::anti_patterns::large_classes::LargeClassConfig;
use crate::error::UveddiError;
use crate::resource_management::ResourceConfig;
use std::path::PathBuf;

#[cfg(feature = "memory-optimization")]
use crate::analysis::memory::MemoryOptimizationConfig;

/// Configuration for analysis target and execution parameters
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// The path to the target directory or file to be analyzed
    pub target_path: PathBuf,

    /// Analysis timeout in seconds (0 = no timeout)
    pub timeout_seconds: u64,

    /// Enable resource management and monitoring
    pub enable_resource_management: bool,

    /// Resource management configuration
    pub resource_config: Option<ResourceConfig>,

    /// Enable memory optimization features
    pub enable_memory_optimization: bool,

    /// Memory limit in gigabytes
    pub memory_limit_gb: Option<f64>,

    /// Memory profile selection (small/default/large)
    pub memory_profile: Option<String>,

    /// Memory optimization configuration
    #[cfg(feature = "memory-optimization")]
    pub memory_optimization: Option<MemoryOptimizationConfig>,

    /// Dead code detector configuration
    pub dead_code: DeadCodeOptions,

    /// Large classes detector configuration
    pub large_classes: LargeClassOptions,
}

/// Dead code detection configuration options
#[derive(Debug, Clone)]
pub struct DeadCodeOptions {
    /// Confidence threshold for dead code detection (0.0 to 1.0)
    pub confidence: Option<f64>,
    /// Enable library mode for dead code detection
    pub library_mode: bool,
    /// Patterns to ignore during dead code detection
    pub ignore_patterns: Option<Vec<String>>,
    /// Symbols to always keep alive during dead code detection
    pub keep_alive: Option<Vec<String>>,
}

/// Large classes detection configuration options
#[derive(Debug, Clone)]
pub struct LargeClassOptions {
    /// Maximum logical lines of code threshold for large classes
    pub max_loc: Option<u32>,
    /// Maximum number of methods threshold for large classes
    pub max_methods: Option<u32>,
    /// Maximum number of fields threshold for large classes
    pub max_fields: Option<u32>,
    /// Maximum cyclomatic complexity threshold for large classes
    pub max_complexity: Option<u32>,
    /// Maximum LCOM score threshold for large classes
    pub max_lcom: Option<f64>,
    /// Patterns to ignore during large classes detection
    pub ignore_patterns: Option<Vec<String>>,
    /// Minimum severity score for large classes reporting
    pub min_severity: Option<u32>,
}

impl AnalysisConfig {
    /// Create a new analysis configuration with default settings
    pub fn new(target_path: PathBuf) -> Self {
        Self {
            target_path,
            timeout_seconds: 0,
            enable_resource_management: false,
            resource_config: None,
            enable_memory_optimization: false,
            memory_limit_gb: None,
            memory_profile: None,
            #[cfg(feature = "memory-optimization")]
            memory_optimization: None,
            dead_code: DeadCodeOptions::default(),
            large_classes: LargeClassOptions::default(),
        }
    }

    /// Validate the analysis configuration
    pub fn validate(&self) -> Result<(), UveddiError> {
        // Validate target path exists
        if !self.target_path.exists() {
            return Err(UveddiError::PathError {
                path: self.target_path.display().to_string(),
                reason: "Path does not exist".to_string(),
                suggestion: "Verify the path exists and is accessible".to_string(),
            });
        }

        // Validate dead code confidence
        if let Some(confidence) = self.dead_code.confidence {
            if confidence < 0.0 || confidence > 1.0 {
                return Err(UveddiError::config_error(
                    "Dead code confidence threshold must be between 0.0 and 1.0",
                    "config validation",
                ));
            }
        }

        // Validate large classes LCOM score
        if let Some(max_lcom) = self.large_classes.max_lcom {
            if max_lcom < 0.0 || max_lcom > 1.0 {
                return Err(UveddiError::config_error(
                    "Large classes LCOM score must be between 0.0 and 1.0",
                    "config validation",
                ));
            }
        }

        // Validate memory configuration if enabled
        if self.enable_memory_optimization {
            self.validate_memory_config()?;
        }

        Ok(())
    }

    /// Merge with another analysis configuration
    pub fn merge_with(mut self, other: AnalysisConfig) -> Self {
        // Only override if other has non-default values
        if other.timeout_seconds > 0 {
            self.timeout_seconds = other.timeout_seconds;
        }
        if other.enable_resource_management {
            self.enable_resource_management = other.enable_resource_management;
            self.resource_config = other.resource_config.or(self.resource_config);
        }
        if other.enable_memory_optimization {
            self.enable_memory_optimization = other.enable_memory_optimization;
            self.memory_limit_gb = other.memory_limit_gb.or(self.memory_limit_gb);
            self.memory_profile = other.memory_profile.or(self.memory_profile);
            #[cfg(feature = "memory-optimization")]
            {
                self.memory_optimization = other.memory_optimization.or(self.memory_optimization);
            }
        }

        self.dead_code = self.dead_code.merge_with(other.dead_code);
        self.large_classes = self.large_classes.merge_with(other.large_classes);

        self
    }

    /// Create dead code detector configuration
    pub fn create_dead_code_config(&self) -> Option<DeadCodeConfig> {
        if self.has_dead_code_config() {
            let mut config = DeadCodeConfig::default();

            if let Some(confidence) = self.dead_code.confidence {
                config.min_confidence = confidence;
            }
            config.library_mode = self.dead_code.library_mode;

            if let Some(ref patterns) = self.dead_code.ignore_patterns {
                config.ignore_patterns = patterns.clone();
            }

            if let Some(ref patterns) = self.dead_code.keep_alive {
                config.keep_alive_patterns = patterns.clone();
            }

            Some(config)
        } else {
            None
        }
    }

    /// Create large classes detector configuration
    pub fn create_large_class_config(&self) -> Option<LargeClassConfig> {
        if self.has_large_class_config() {
            let mut config = LargeClassConfig::default();

            // Apply custom thresholds if provided
            if let Some(max_loc) = self.large_classes.max_loc {
                config.rust_thresholds.max_logical_loc = max_loc;
                config.python_thresholds.max_logical_loc = max_loc;
                config.javascript_thresholds.max_logical_loc = max_loc;
            }

            if let Some(max_methods) = self.large_classes.max_methods {
                config.rust_thresholds.max_methods = max_methods;
                config.python_thresholds.max_methods = max_methods;
                config.javascript_thresholds.max_methods = max_methods;
            }

            if let Some(max_fields) = self.large_classes.max_fields {
                config.rust_thresholds.max_fields = max_fields;
                config.python_thresholds.max_fields = max_fields;
                config.javascript_thresholds.max_fields = max_fields;
            }

            if let Some(max_complexity) = self.large_classes.max_complexity {
                config.rust_thresholds.max_cyclomatic_complexity = max_complexity;
                config.python_thresholds.max_cyclomatic_complexity = max_complexity;
                config.javascript_thresholds.max_cyclomatic_complexity = max_complexity;
                config.rust_thresholds.max_cognitive_complexity = max_complexity;
                config.python_thresholds.max_cognitive_complexity = max_complexity;
                config.javascript_thresholds.max_cognitive_complexity = max_complexity;
            }

            if let Some(max_lcom) = self.large_classes.max_lcom {
                config.rust_thresholds.max_lcom_score = max_lcom;
                config.python_thresholds.max_lcom_score = max_lcom;
                config.javascript_thresholds.max_lcom_score = max_lcom;
            }

            Some(config)
        } else {
            None
        }
    }

    /// Check if any dead code configuration is provided
    fn has_dead_code_config(&self) -> bool {
        self.dead_code.confidence.is_some()
            || self.dead_code.library_mode
            || self.dead_code.ignore_patterns.is_some()
            || self.dead_code.keep_alive.is_some()
    }

    /// Check if any large class configuration is provided
    fn has_large_class_config(&self) -> bool {
        self.large_classes.max_loc.is_some()
            || self.large_classes.max_methods.is_some()
            || self.large_classes.max_fields.is_some()
            || self.large_classes.max_complexity.is_some()
            || self.large_classes.max_lcom.is_some()
            || self.large_classes.ignore_patterns.is_some()
            || self.large_classes.min_severity.is_some()
    }

    /// Validate memory optimization configuration
    fn validate_memory_config(&self) -> Result<(), UveddiError> {
        if let Some(limit_gb) = self.memory_limit_gb {
            if limit_gb <= 0.0 {
                return Err(UveddiError::config_error(
                    &format!("Memory limit must be positive, got: {}GB", limit_gb),
                    "memory configuration",
                ));
            }
            if limit_gb > 1000.0 {
                return Err(UveddiError::config_error(
                    &format!("Memory limit too high ({}GB), maximum is 1000GB", limit_gb),
                    "memory configuration",
                ));
            }
        }

        if let Some(profile) = self.memory_profile.as_deref() {
            if !matches!(profile, "small" | "default" | "large") {
                return Err(UveddiError::config_error(
                    &format!("Invalid memory profile '{}', must be one of: small, default, large", profile),
                    "memory configuration",
                ));
            }
        }

        Ok(())
    }
}

impl Default for DeadCodeOptions {
    fn default() -> Self {
        Self {
            confidence: None,
            library_mode: false,
            ignore_patterns: None,
            keep_alive: None,
        }
    }
}

impl DeadCodeOptions {
    /// Merge with another dead code options configuration
    fn merge_with(mut self, other: DeadCodeOptions) -> Self {
        self.confidence = other.confidence.or(self.confidence);
        if other.library_mode {
            self.library_mode = other.library_mode;
        }
        self.ignore_patterns = other.ignore_patterns.or(self.ignore_patterns);
        self.keep_alive = other.keep_alive.or(self.keep_alive);
        self
    }
}

impl Default for LargeClassOptions {
    fn default() -> Self {
        Self {
            max_loc: None,
            max_methods: None,
            max_fields: None,
            max_complexity: None,
            max_lcom: None,
            ignore_patterns: None,
            min_severity: None,
        }
    }
}

impl LargeClassOptions {
    /// Merge with another large class options configuration
    fn merge_with(mut self, other: LargeClassOptions) -> Self {
        self.max_loc = other.max_loc.or(self.max_loc);
        self.max_methods = other.max_methods.or(self.max_methods);
        self.max_fields = other.max_fields.or(self.max_fields);
        self.max_complexity = other.max_complexity.or(self.max_complexity);
        self.max_lcom = other.max_lcom.or(self.max_lcom);
        self.ignore_patterns = other.ignore_patterns.or(self.ignore_patterns);
        self.min_severity = other.min_severity.or(self.min_severity);
        self
    }
}