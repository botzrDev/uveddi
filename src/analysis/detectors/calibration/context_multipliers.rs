//! Context-aware threshold multipliers for intelligent threshold adjustment

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Code context that affects threshold interpretation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodeContext {
    /// Production code (1.0x multiplier)
    Production,
    /// Test code (1.5x multiplier - more lenient)
    Test,
    /// Generated code (3.0x multiplier or skip entirely)
    Generated,
    /// Framework/library code (2.0x multiplier)
    Framework,
    /// Legacy code (user-configurable, default 1.2x)
    Legacy,
    /// Third-party code (2.5x multiplier or skip)
    ThirdParty,
    /// Build/tooling scripts (2.0x multiplier)
    Build,
    /// Documentation examples (2.5x multiplier)
    Documentation,
}

impl CodeContext {
    /// Get the default threshold multiplier for this context
    pub fn default_multiplier(&self) -> f64 {
        match self {
            Self::Production => 1.0,
            Self::Test => 1.5,
            Self::Generated => 3.0,
            Self::Framework => 2.0,
            Self::Legacy => 1.2,
            Self::ThirdParty => 2.5,
            Self::Build => 2.0,
            Self::Documentation => 2.5,
        }
    }

    /// Check if this context should typically be skipped in analysis
    pub fn should_skip_by_default(&self) -> bool {
        matches!(self, Self::Generated | Self::ThirdParty)
    }

    /// Detect context from file path
    pub fn detect_from_path(file_path: &str) -> Vec<Self> {
        let mut contexts = Vec::new();
        let path_lower = file_path.to_lowercase();

        // Test files
        if path_lower.contains("/test/")
            || path_lower.contains("/tests/")
            || path_lower.contains("_test.")
            || path_lower.contains(".test.")
            || path_lower.ends_with("_spec.rs")
            || path_lower.ends_with(".spec.js")
            || path_lower.ends_with(".spec.ts")
            || path_lower.contains("__tests__")
        {
            contexts.push(Self::Test);
        }

        // Generated files
        if path_lower.contains("/generated/")
            || path_lower.contains("/gen/")
            || path_lower.contains(".generated.")
            || path_lower.contains(".pb.")
            || path_lower.contains(".g.")
            || path_lower.contains("_pb2.py")
            || path_lower.ends_with(".pb.go")
        {
            contexts.push(Self::Generated);
        }

        // Third-party code
        if path_lower.contains("/node_modules/")
            || path_lower.contains("/vendor/")
            || path_lower.contains("/third_party/")
            || path_lower.contains("/external/")
            || path_lower.contains("/.cargo/")
        {
            contexts.push(Self::ThirdParty);
        }

        // Build scripts
        if path_lower.contains("/build/")
            || path_lower.contains("/scripts/")
            || path_lower.contains("build.rs")
            || path_lower.contains("makefile")
            || path_lower.contains("dockerfile")
            || path_lower.ends_with(".sh")
            || path_lower.ends_with(".bash")
        {
            contexts.push(Self::Build);
        }

        // Documentation
        if path_lower.contains("/docs/")
            || path_lower.contains("/examples/")
            || path_lower.ends_with(".md")
            || path_lower.ends_with("readme")
        {
            contexts.push(Self::Documentation);
        }

        // Legacy code (requires explicit marking, check for legacy markers)
        if path_lower.contains("/legacy/") || path_lower.contains("deprecated") {
            contexts.push(Self::Legacy);
        }

        // If no specific context detected, it's production code
        if contexts.is_empty() {
            contexts.push(Self::Production);
        }

        contexts
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Production => "Production code with standard thresholds",
            Self::Test => "Test code with more lenient thresholds",
            Self::Generated => "Generated code (typically skipped)",
            Self::Framework => "Framework/library code with relaxed thresholds",
            Self::Legacy => "Legacy code with slightly relaxed thresholds",
            Self::ThirdParty => "Third-party code (typically skipped)",
            Self::Build => "Build/tooling scripts with relaxed thresholds",
            Self::Documentation => "Documentation examples with relaxed thresholds",
        }
    }
}

/// Configuration for context-aware threshold adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareConfig {
    /// Custom multipliers for each context
    pub context_multipliers: HashMap<CodeContext, f64>,
    /// Whether to skip generated code entirely
    pub skip_generated: bool,
    /// Whether to skip third-party code entirely
    pub skip_third_party: bool,
    /// Legacy code multiplier (user-configurable)
    pub legacy_multiplier: f64,
    /// Additional file patterns to exclude
    pub custom_skip_patterns: Vec<String>,
}

impl ContextAwareConfig {
    /// Create default configuration
    pub fn new() -> Self {
        let mut multipliers = HashMap::new();
        for context in [
            CodeContext::Production,
            CodeContext::Test,
            CodeContext::Generated,
            CodeContext::Framework,
            CodeContext::Legacy,
            CodeContext::ThirdParty,
            CodeContext::Build,
            CodeContext::Documentation,
        ] {
            multipliers.insert(context, context.default_multiplier());
        }

        Self {
            context_multipliers: multipliers,
            skip_generated: true,
            skip_third_party: true,
            legacy_multiplier: 1.2,
            custom_skip_patterns: Vec::new(),
        }
    }

    /// Get multiplier for a specific context
    pub fn get_multiplier(&self, context: CodeContext) -> f64 {
        if context == CodeContext::Legacy {
            return self.legacy_multiplier;
        }
        self.context_multipliers
            .get(&context)
            .copied()
            .unwrap_or(context.default_multiplier())
    }

    /// Set custom multiplier for a context
    pub fn set_multiplier(&mut self, context: CodeContext, multiplier: f64) {
        self.context_multipliers.insert(context, multiplier);
    }

    /// Check if a file should be skipped based on context
    pub fn should_skip_file(&self, file_path: &str) -> bool {
        let contexts = CodeContext::detect_from_path(file_path);

        for context in contexts {
            if (context == CodeContext::Generated && self.skip_generated)
                || (context == CodeContext::ThirdParty && self.skip_third_party)
            {
                return true;
            }
        }

        // Check custom patterns
        for pattern in &self.custom_skip_patterns {
            if file_path.contains(pattern) {
                return true;
            }
        }

        false
    }
}

impl Default for ContextAwareConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Threshold adjuster that applies context-aware multipliers
#[derive(Debug, Clone)]
pub struct ContextAwareThresholds {
    /// Configuration
    pub config: ContextAwareConfig,
}

impl ContextAwareThresholds {
    /// Create a new context-aware threshold adjuster
    pub fn new(config: ContextAwareConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(ContextAwareConfig::default())
    }

    /// Apply context multiplier to a threshold value
    pub fn apply_to_threshold(&self, base_threshold: f64, file_path: &str) -> f64 {
        let contexts = CodeContext::detect_from_path(file_path);

        // If file should be skipped, return infinity to effectively disable
        if self.config.should_skip_file(file_path) {
            return f64::INFINITY;
        }

        // Use the most lenient multiplier from detected contexts
        let max_multiplier = contexts
            .iter()
            .map(|ctx| self.config.get_multiplier(*ctx))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        base_threshold * max_multiplier
    }

    /// Apply context multiplier to an integer threshold value
    pub fn apply_to_int_threshold(&self, base_threshold: u32, file_path: &str) -> u32 {
        let contexts = CodeContext::detect_from_path(file_path);

        // If file should be skipped, return max value
        if self.config.should_skip_file(file_path) {
            return u32::MAX;
        }

        // Use the most lenient multiplier from detected contexts
        let max_multiplier = contexts
            .iter()
            .map(|ctx| self.config.get_multiplier(*ctx))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        ((base_threshold as f64) * max_multiplier) as u32
    }

    /// Get all contexts for a file
    pub fn get_contexts(&self, file_path: &str) -> Vec<CodeContext> {
        CodeContext::detect_from_path(file_path)
    }

    /// Get the effective multiplier for a file
    pub fn get_effective_multiplier(&self, file_path: &str) -> f64 {
        let contexts = self.get_contexts(file_path);
        contexts
            .iter()
            .map(|ctx| self.config.get_multiplier(*ctx))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_detection_test_files() {
        let contexts = CodeContext::detect_from_path("src/tests/my_test.rs");
        assert!(contexts.contains(&CodeContext::Test));

        let contexts = CodeContext::detect_from_path("src/foo.spec.ts");
        assert!(contexts.contains(&CodeContext::Test));
    }

    #[test]
    fn test_context_detection_generated() {
        let contexts = CodeContext::detect_from_path("src/generated/foo.rs");
        assert!(contexts.contains(&CodeContext::Generated));

        let contexts = CodeContext::detect_from_path("src/foo_pb2.py");
        assert!(contexts.contains(&CodeContext::Generated));
    }

    #[test]
    fn test_context_detection_third_party() {
        let contexts = CodeContext::detect_from_path("node_modules/foo/bar.js");
        assert!(contexts.contains(&CodeContext::ThirdParty));

        let contexts = CodeContext::detect_from_path("vendor/lib/foo.rs");
        assert!(contexts.contains(&CodeContext::ThirdParty));
    }

    #[test]
    fn test_context_detection_production() {
        let contexts = CodeContext::detect_from_path("src/main.rs");
        assert!(contexts.contains(&CodeContext::Production));

        let contexts = CodeContext::detect_from_path("lib/utils.py");
        assert!(contexts.contains(&CodeContext::Production));
    }

    #[test]
    fn test_default_multipliers() {
        assert_eq!(CodeContext::Production.default_multiplier(), 1.0);
        assert_eq!(CodeContext::Test.default_multiplier(), 1.5);
        assert_eq!(CodeContext::Generated.default_multiplier(), 3.0);
        assert_eq!(CodeContext::Framework.default_multiplier(), 2.0);
    }

    #[test]
    fn test_threshold_adjustment() {
        let adjuster = ContextAwareThresholds::default_config();

        // Production code: no adjustment
        let adjusted = adjuster.apply_to_threshold(100.0, "src/main.rs");
        assert_eq!(adjusted, 100.0);

        // Test code: 1.5x multiplier
        let adjusted = adjuster.apply_to_threshold(100.0, "tests/test_foo.rs");
        assert_eq!(adjusted, 150.0);

        // Generated code: should skip (returns infinity)
        let adjusted = adjuster.apply_to_threshold(100.0, "gen/foo.rs");
        assert!(adjusted.is_infinite());
    }

    #[test]
    fn test_int_threshold_adjustment() {
        let adjuster = ContextAwareThresholds::default_config();

        // Production code
        let adjusted = adjuster.apply_to_int_threshold(100, "src/main.rs");
        assert_eq!(adjusted, 100);

        // Test code
        let adjusted = adjuster.apply_to_int_threshold(100, "tests/test_foo.rs");
        assert_eq!(adjusted, 150);
    }

    #[test]
    fn test_should_skip_file() {
        let config = ContextAwareConfig::default();

        assert!(!config.should_skip_file("src/main.rs"));
        assert!(config.should_skip_file("generated/foo.rs"));
        assert!(config.should_skip_file("node_modules/lib/bar.js"));

        let mut config = ContextAwareConfig::default();
        config.skip_generated = false;
        assert!(!config.should_skip_file("generated/foo.rs"));
    }

    #[test]
    fn test_custom_multiplier() {
        let mut config = ContextAwareConfig::default();
        config.set_multiplier(CodeContext::Test, 2.0);

        let adjuster = ContextAwareThresholds::new(config);
        let adjusted = adjuster.apply_to_threshold(100.0, "tests/test_foo.rs");
        assert_eq!(adjusted, 200.0);
    }

    #[test]
    fn test_legacy_multiplier_configuration() {
        let mut config = ContextAwareConfig::default();
        config.legacy_multiplier = 1.8;

        let adjuster = ContextAwareThresholds::new(config);
        let adjusted = adjuster.apply_to_threshold(100.0, "legacy/old_code.rs");
        assert_eq!(adjusted, 180.0);
    }
}
