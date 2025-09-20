//! Configuration types for dead code detection

/// Configures the behavior of the `DeadCodeDetector`.
///
/// This struct allows customization of the detection process, such as setting confidence
/// thresholds, ignoring specific files, and defining custom entry points.
#[derive(Debug, Clone)]
pub struct DeadCodeConfig {
    /// The minimum confidence score (0.0 to 1.0) a symbol must have to be reported as dead code.
    pub min_confidence: f64,
    /// If `true`, the detector treats all exported symbols as potential entry points,
    /// which is suitable for analyzing libraries. If `false`, it assumes an application
    /// context where unused exports might be dead code.
    pub library_mode: bool,
    /// A list of string patterns to exclude files from analysis.
    /// Useful for ignoring test directories, mocks, or generated code.
    pub ignore_patterns: Vec<String>,
    /// A list of symbol names to always consider "live," regardless of usage.
    /// Common examples include `main`, `init`, or framework-specific entry points.
    pub keep_alive_patterns: Vec<String>,
}

impl Default for DeadCodeConfig {
    /// Provides a default configuration for the `DeadCodeDetector`.
    ///
    /// - `min_confidence`: 0.5
    /// - `library_mode`: `false`
    /// - `ignore_patterns`: Includes common test and mock directories.
    /// - `keep_alive_patterns`: Includes `main`, `init`, `setup`, and `teardown`.
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            library_mode: false,
            ignore_patterns: vec![
                "test".to_string(),
                "tests".to_string(),
                "spec".to_string(),
                "mock".to_string(),
            ],
            keep_alive_patterns: vec![
                "main".to_string(),
                "init".to_string(),
                "setup".to_string(),
                "teardown".to_string(),
            ],
        }
    }
}

impl DeadCodeConfig {
    /// Reset configuration to default values
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}