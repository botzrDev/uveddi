//! Profile-based configuration for different analysis scenarios

use super::confidence_bands::ConfidenceThresholds;
use super::context_multipliers::ContextAwareConfig;
use super::severity_scoring::SeverityThresholds;
use serde::{Deserialize, Serialize};

/// Named detector profile with clear use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectorProfile {
    /// Quick analysis for CI/CD - fast, high confidence only
    Fast,
    /// Standard development analysis - balanced approach
    Balanced,
    /// Deep analysis for releases - thorough, lower threshold
    Thorough,
    /// Security-critical analysis - paranoid mode, catch everything
    Paranoid,
    /// Legacy code analysis - more lenient
    Legacy,
    /// Custom user-defined profile
    Custom,
}

impl DetectorProfile {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Fast => "Quick analysis for CI/CD - high confidence issues only",
            Self::Balanced => "Standard development analysis with balanced thresholds",
            Self::Thorough => "Deep analysis for releases - thorough detection",
            Self::Paranoid => "Security-critical analysis - maximum sensitivity",
            Self::Legacy => "Legacy code analysis with more lenient thresholds",
            Self::Custom => "Custom user-defined profile",
        }
    }

    /// Get recommended use case
    pub fn use_case(&self) -> &'static str {
        match self {
            Self::Fast => "CI/CD pipelines, quick feedback loops",
            Self::Balanced => "Day-to-day development, code reviews",
            Self::Thorough => "Pre-release analysis, security audits",
            Self::Paranoid => "Critical security analysis, compliance checks",
            Self::Legacy => "Working with legacy codebases",
            Self::Custom => "Specific project requirements",
        }
    }
}

/// Complete profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Profile identifier
    pub profile: DetectorProfile,
    /// Confidence thresholds
    pub confidence: ConfidenceThresholds,
    /// Severity thresholds
    pub severity: SeverityThresholds,
    /// Context-aware configuration
    pub context: ContextAwareConfig,
    /// Maximum issues to report per file
    pub max_issues_per_file: Option<usize>,
    /// Whether to skip generated code
    pub skip_generated: bool,
    /// Whether to skip test files
    pub skip_tests: bool,
    /// Profile-specific settings
    pub settings: ProfileSettings,
}

/// Profile-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    /// Overall strictness multiplier (0.5 = more lenient, 2.0 = stricter)
    pub strictness_multiplier: f64,
    /// Enable all detectors
    pub enable_all_detectors: bool,
    /// Enable experimental detectors
    pub enable_experimental: bool,
    /// Parallel processing
    pub enable_parallel: bool,
    /// Maximum analysis time per file (seconds)
    pub max_analysis_time_secs: u64,
}

impl ProfileConfig {
    /// Create a Fast profile
    pub fn fast() -> Self {
        use super::confidence_bands::ConfidenceLevel;
        use super::severity_scoring::StandardSeverity;

        Self {
            profile: DetectorProfile::Fast,
            confidence: ConfidenceThresholds {
                development: ConfidenceLevel::Critical.min_threshold(),
                staging: ConfidenceLevel::Critical.min_threshold(),
                production: ConfidenceLevel::Certain.min_threshold(),
                ci_cd: ConfidenceLevel::Certain.min_threshold(),
            },
            severity: SeverityThresholds {
                development: StandardSeverity::High,
                staging: StandardSeverity::High,
                production: StandardSeverity::Critical,
                ci_cd: StandardSeverity::Critical,
            },
            context: ContextAwareConfig {
                skip_generated: true,
                skip_third_party: true,
                ..Default::default()
            },
            max_issues_per_file: Some(10),
            skip_generated: true,
            skip_tests: true,
            settings: ProfileSettings {
                strictness_multiplier: 1.5, // Only flag serious issues
                enable_all_detectors: false,
                enable_experimental: false,
                enable_parallel: true,
                max_analysis_time_secs: 10,
            },
        }
    }

    /// Create a Balanced profile
    pub fn balanced() -> Self {
        use super::confidence_bands::ConfidenceLevel;
        use super::severity_scoring::StandardSeverity;

        Self {
            profile: DetectorProfile::Balanced,
            confidence: ConfidenceThresholds::calibrated(),
            severity: SeverityThresholds::balanced(),
            context: ContextAwareConfig::default(),
            max_issues_per_file: Some(50),
            skip_generated: true,
            skip_tests: false,
            settings: ProfileSettings {
                strictness_multiplier: 1.0, // Standard thresholds
                enable_all_detectors: true,
                enable_experimental: false,
                enable_parallel: true,
                max_analysis_time_secs: 60,
            },
        }
    }

    /// Create a Thorough profile
    pub fn thorough() -> Self {
        use super::confidence_bands::ConfidenceLevel;
        use super::severity_scoring::StandardSeverity;

        Self {
            profile: DetectorProfile::Thorough,
            confidence: ConfidenceThresholds {
                development: ConfidenceLevel::Medium.min_threshold(),
                staging: ConfidenceLevel::Medium.min_threshold(),
                production: ConfidenceLevel::High.min_threshold(),
                ci_cd: ConfidenceLevel::High.min_threshold(),
            },
            severity: SeverityThresholds {
                development: StandardSeverity::Info,
                staging: StandardSeverity::Low,
                production: StandardSeverity::Low,
                ci_cd: StandardSeverity::Medium,
            },
            context: ContextAwareConfig {
                skip_generated: false,
                skip_third_party: false,
                ..Default::default()
            },
            max_issues_per_file: None, // Unlimited
            skip_generated: false,
            skip_tests: false,
            settings: ProfileSettings {
                strictness_multiplier: 0.7, // More sensitive
                enable_all_detectors: true,
                enable_experimental: true,
                enable_parallel: true,
                max_analysis_time_secs: 300,
            },
        }
    }

    /// Create a Paranoid profile
    pub fn paranoid() -> Self {
        use super::confidence_bands::ConfidenceLevel;
        use super::severity_scoring::StandardSeverity;

        Self {
            profile: DetectorProfile::Paranoid,
            confidence: ConfidenceThresholds {
                development: ConfidenceLevel::Low.min_threshold(),
                staging: ConfidenceLevel::Low.min_threshold(),
                production: ConfidenceLevel::Medium.min_threshold(),
                ci_cd: ConfidenceLevel::Medium.min_threshold(),
            },
            severity: SeverityThresholds {
                development: StandardSeverity::Info,
                staging: StandardSeverity::Info,
                production: StandardSeverity::Info,
                ci_cd: StandardSeverity::Low,
            },
            context: ContextAwareConfig {
                skip_generated: false,
                skip_third_party: false,
                ..Default::default()
            },
            max_issues_per_file: None,
            skip_generated: false,
            skip_tests: false,
            settings: ProfileSettings {
                strictness_multiplier: 0.5, // Very sensitive
                enable_all_detectors: true,
                enable_experimental: true,
                enable_parallel: true,
                max_analysis_time_secs: 600,
            },
        }
    }

    /// Create a Legacy profile
    pub fn legacy() -> Self {
        use super::confidence_bands::ConfidenceLevel;
        use super::severity_scoring::StandardSeverity;

        let mut context_config = ContextAwareConfig::default();
        context_config.legacy_multiplier = 2.0; // Much more lenient

        Self {
            profile: DetectorProfile::Legacy,
            confidence: ConfidenceThresholds {
                development: ConfidenceLevel::High.min_threshold(),
                staging: ConfidenceLevel::High.min_threshold(),
                production: ConfidenceLevel::Critical.min_threshold(),
                ci_cd: ConfidenceLevel::Critical.min_threshold(),
            },
            severity: SeverityThresholds::lenient(),
            context: context_config,
            max_issues_per_file: Some(25),
            skip_generated: true,
            skip_tests: true,
            settings: ProfileSettings {
                strictness_multiplier: 1.5, // More lenient
                enable_all_detectors: false,
                enable_experimental: false,
                enable_parallel: true,
                max_analysis_time_secs: 120,
            },
        }
    }

    /// Create profile by name
    pub fn from_profile(profile: DetectorProfile) -> Self {
        match profile {
            DetectorProfile::Fast => Self::fast(),
            DetectorProfile::Balanced => Self::balanced(),
            DetectorProfile::Thorough => Self::thorough(),
            DetectorProfile::Paranoid => Self::paranoid(),
            DetectorProfile::Legacy => Self::legacy(),
            DetectorProfile::Custom => Self::balanced(), // Default to balanced for custom
        }
    }

    /// Apply strictness multiplier to a threshold
    pub fn apply_strictness(&self, base_threshold: f64) -> f64 {
        base_threshold * self.settings.strictness_multiplier
    }

    /// Apply strictness multiplier to an integer threshold
    pub fn apply_strictness_int(&self, base_threshold: u32) -> u32 {
        ((base_threshold as f64) * self.settings.strictness_multiplier) as u32
    }

    /// Check if a file should be analyzed based on profile settings
    pub fn should_analyze_file(&self, file_path: &str) -> bool {
        // Check if we should skip tests
        if self.skip_tests {
            let path_lower = file_path.to_lowercase();
            if path_lower.contains("/test/")
                || path_lower.contains("/tests/")
                || path_lower.contains(".test.")
                || path_lower.ends_with("_test.rs")
            {
                return false;
            }
        }

        // Check context-based skipping
        !self.context.should_skip_file(file_path)
    }
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let fast = ProfileConfig::fast();
        assert_eq!(fast.profile, DetectorProfile::Fast);
        assert!(fast.skip_tests);
        assert_eq!(fast.max_issues_per_file, Some(10));

        let balanced = ProfileConfig::balanced();
        assert_eq!(balanced.profile, DetectorProfile::Balanced);
        assert!(!balanced.skip_tests);

        let thorough = ProfileConfig::thorough();
        assert_eq!(thorough.profile, DetectorProfile::Thorough);
        assert!(thorough.max_issues_per_file.is_none());
    }

    #[test]
    fn test_strictness_multiplier() {
        let fast = ProfileConfig::fast();
        assert_eq!(fast.apply_strictness(100.0), 150.0); // 1.5x

        let thorough = ProfileConfig::thorough();
        assert_eq!(thorough.apply_strictness(100.0), 70.0); // 0.7x

        let paranoid = ProfileConfig::paranoid();
        assert_eq!(paranoid.apply_strictness(100.0), 50.0); // 0.5x
    }

    #[test]
    fn test_profile_confidence_thresholds() {
        let fast = ProfileConfig::fast();
        assert!(fast.confidence.development >= 0.7); // Critical or higher

        let paranoid = ProfileConfig::paranoid();
        assert!(paranoid.confidence.development <= 0.3); // Low confidence
    }

    #[test]
    fn test_should_analyze_file() {
        let fast = ProfileConfig::fast();
        assert!(fast.should_analyze_file("src/main.rs"));
        assert!(!fast.should_analyze_file("tests/test_main.rs")); // Skip tests

        let thorough = ProfileConfig::thorough();
        assert!(thorough.should_analyze_file("tests/test_main.rs")); // Include tests
        assert!(thorough.should_analyze_file("generated/foo.rs")); // Include generated
    }

    #[test]
    fn test_profile_from_enum() {
        let profile = ProfileConfig::from_profile(DetectorProfile::Fast);
        assert_eq!(profile.profile, DetectorProfile::Fast);

        let profile = ProfileConfig::from_profile(DetectorProfile::Paranoid);
        assert_eq!(profile.profile, DetectorProfile::Paranoid);
    }

    #[test]
    fn test_legacy_profile_multipliers() {
        let legacy = ProfileConfig::legacy();
        assert_eq!(legacy.context.legacy_multiplier, 2.0);
        assert_eq!(legacy.settings.strictness_multiplier, 1.5);
    }
}
