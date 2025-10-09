# Detector Tolerance Calibration - Implementation Guide

## Overview

This guide provides step-by-step instructions for implementing and deploying the new detector tolerance calibration system across the Uveddi codebase.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Architecture Overview](#architecture-overview)
3. [Phase 1: Module Integration](#phase-1-module-integration)
4. [Phase 2: Detector Migration](#phase-2-detector-migration)
5. [Phase 3: Testing & Validation](#phase-3-testing--validation)
6. [Phase 4: Deployment](#phase-4-deployment)
7. [API Reference](#api-reference)

## Quick Start

### Using the New Calibration System

```rust
use crate::analysis::detectors::calibration::{
    ConfidenceBand, SeverityScore, ContextAwareThresholds,
    ProfileConfig, DetectorProfile,
};

// 1. Create a profile configuration
let profile = ProfileConfig::balanced();

// 2. Create context-aware thresholds
let context = ContextAwareThresholds::default_config();

// 3. Use in detector
let base_threshold = 100;
let file_path = "src/main.rs";
let adjusted_threshold = context.apply_to_int_threshold(base_threshold, file_path);

// 4. Calculate severity with components
let severity = SeverityScore::from_components(vec![
    SeverityComponent::new("complexity", 30, 0.4),
    SeverityComponent::new("size", 25, 0.3),
    SeverityComponent::new("coupling", 20, 0.3),
]);

// 5. Check confidence band
let confidence = ConfidenceBand::from_score(0.75);
println!("Confidence: {}, Recommendation: {}",
    confidence.level, confidence.recommendation);
```

## Architecture Overview

### Module Structure

```
src/analysis/detectors/calibration/
├── mod.rs                      # Public API exports
├── confidence_bands.rs         # Standardized confidence levels (0.0-1.0)
├── severity_scoring.rs         # Standardized severity scoring (0-100)
├── context_multipliers.rs      # Context-aware threshold adjustments
├── profiles.rs                 # Pre-configured analysis profiles
├── testing.rs                  # Calibration testing infrastructure
└── feedback.rs                 # User feedback and auto-calibration
```

### Key Components

1. **Confidence Bands** - Standardized confidence levels across all detectors
2. **Severity Scoring** - Unified 0-100 severity scale
3. **Context Multipliers** - Automatic threshold adjustment based on code context
4. **Profiles** - Pre-configured settings for different use cases
5. **Testing Framework** - A/B testing and metric collection
6. **Feedback System** - User feedback collection and auto-calibration

## Phase 1: Module Integration

### Step 1.1: Update Module Declarations

Add to `src/analysis/detectors/mod.rs`:

```rust
pub mod calibration;

pub use calibration::{
    ConfidenceBand, ConfidenceLevel, ConfidenceThresholds,
    SeverityScore, StandardSeverity, SeverityThresholds,
    CodeContext, ContextAwareThresholds,
    DetectorProfile, ProfileConfig,
};
```

### Step 1.2: Update Cargo Dependencies (if needed)

The calibration system uses only standard library and serde. No additional dependencies required.

### Step 1.3: Create Configuration Integration

Create `src/analysis/detectors/calibrated_config.rs`:

```rust
use super::calibration::{ProfileConfig, ContextAwareThresholds};
use serde::{Deserialize, Serialize};

/// Global calibrated configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibratedConfig {
    /// Active profile
    pub profile: ProfileConfig,
    /// Context-aware thresholds
    pub context: ContextAwareThresholds,
    /// Enable auto-calibration
    pub enable_auto_calibration: bool,
}

impl CalibratedConfig {
    pub fn new(profile: ProfileConfig) -> Self {
        Self {
            profile,
            context: ContextAwareThresholds::default_config(),
            enable_auto_calibration: false,
        }
    }

    pub fn balanced() -> Self {
        Self::new(ProfileConfig::balanced())
    }

    pub fn thorough() -> Self {
        Self::new(ProfileConfig::thorough())
    }
}

impl Default for CalibratedConfig {
    fn default() -> Self {
        Self::balanced()
    }
}
```

## Phase 2: Detector Migration

### Step 2.1: Migrate Security Config

Update `src/analysis/detectors/security/config/config.rs`:

```rust
use crate::analysis::detectors::calibration::{
    ConfidenceThresholds, SeverityThresholds
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSecurityConfig {
    // ... existing fields ...

    /// Calibrated confidence thresholds (replaces single confidence_threshold)
    pub confidence_thresholds: ConfidenceThresholds,

    /// Calibrated severity thresholds
    pub severity_thresholds: SeverityThresholds,
}

impl Default for ConfigSecurityConfig {
    fn default() -> Self {
        Self {
            enable_credential_detection: true,
            enable_misconfiguration_detection: true,
            enable_permission_detection: true,
            enable_defaults_detection: true,
            confidence_thresholds: ConfidenceThresholds::calibrated(),
            severity_thresholds: SeverityThresholds::balanced(),
            max_issues_per_file: 50,
            enable_compliance_validation: true,
            enable_policy_validation: true,
        }
    }
}

impl ConfigSecurityConfig {
    pub fn production() -> Self {
        Self {
            confidence_thresholds: ConfidenceThresholds {
                development: 0.5,
                staging: 0.6,
                production: 0.7,
                ci_cd: 0.8,
            },
            max_issues_per_file: 100,
            ..Default::default()
        }
    }

    pub fn development() -> Self {
        Self {
            confidence_thresholds: ConfidenceThresholds {
                development: 0.5,
                staging: 0.5,
                production: 0.5,
                ci_cd: 0.6,
            },
            max_issues_per_file: 20,
            enable_compliance_validation: false,
            ..Default::default()
        }
    }
}
```

### Step 2.2: Migrate Long Methods Config

Update `src/analysis/detectors/anti_patterns/long_methods/config.rs`:

```rust
use crate::analysis::detectors::calibration::{
    ContextAwareThresholds, SeverityScore, SeverityComponent
};

impl LongMethodsConfig {
    /// Apply context-aware threshold adjustments
    pub fn get_adjusted_threshold(
        &self,
        language: SourceLanguage,
        file_path: &str,
    ) -> Option<LanguageThresholds> {
        let base_threshold = self.thresholds.get(&language)?;

        let context = ContextAwareThresholds::default_config();
        let multiplier = context.get_effective_multiplier(file_path);

        Some(LanguageThresholds {
            max_logical_loc: ((base_threshold.max_logical_loc as f64) * multiplier) as u32,
            max_statements: ((base_threshold.max_statements as f64) * multiplier) as u32,
            max_parameters: base_threshold.max_parameters,
            max_nesting_depth: base_threshold.max_nesting_depth,
            max_cyclomatic_complexity: ((base_threshold.max_cyclomatic_complexity as f64) * multiplier) as u32,
            max_cognitive_complexity: ((base_threshold.max_cognitive_complexity as f64) * multiplier) as u32,
        })
    }
}

// Update MethodMetrics to use new severity system
impl MethodMetrics {
    pub fn calculate_calibrated_severity(&self, thresholds: &LanguageThresholds) -> SeverityScore {
        let components = vec![
            SeverityComponent::with_description(
                "size",
                self.calculate_size_score(thresholds),
                0.4,
                "Lines of code and statement count"
            ),
            SeverityComponent::with_description(
                "complexity",
                self.calculate_complexity_score(thresholds),
                0.4,
                "Cyclomatic and cognitive complexity"
            ),
            SeverityComponent::with_description(
                "structure",
                self.calculate_structure_score(thresholds),
                0.2,
                "Nesting depth and parameter count"
            ),
        ];

        SeverityScore::from_components(components)
    }

    fn calculate_size_score(&self, thresholds: &LanguageThresholds) -> u32 {
        let loc_excess = self.logical_loc.saturating_sub(thresholds.max_logical_loc);
        let stmt_excess = self.statement_count.saturating_sub(thresholds.max_statements);
        ((loc_excess * 2 + stmt_excess) / 3).min(40)
    }

    fn calculate_complexity_score(&self, thresholds: &LanguageThresholds) -> u32 {
        let cyclo_excess = self.cyclomatic_complexity.saturating_sub(thresholds.max_cyclomatic_complexity);
        let cogn_excess = self.cognitive_complexity.saturating_sub(thresholds.max_cognitive_complexity);
        ((cyclo_excess * 3 + cogn_excess * 2) / 2).min(40)
    }

    fn calculate_structure_score(&self, thresholds: &LanguageThresholds) -> u32 {
        let nest_excess = self.max_nesting_depth.saturating_sub(thresholds.max_nesting_depth);
        let param_excess = self.parameter_count.saturating_sub(thresholds.max_parameters);
        ((nest_excess * 5 + param_excess * 3) / 2).min(20)
    }
}
```

### Step 2.3: Migrate God Object Config

Update `src/analysis/detectors/anti_patterns/god_object/config.rs`:

```rust
use crate::analysis::detectors::calibration::{
    ContextAwareThresholds, CodeContext
};

impl GodObjectConfig {
    /// Get context-adjusted method threshold
    pub fn get_adjusted_method_threshold(
        &self,
        language: SourceLanguage,
        file_path: &str,
    ) -> usize {
        let base = self.get_method_threshold(language);
        let context = ContextAwareThresholds::default_config();
        context.apply_to_int_threshold(base as u32, file_path) as usize
    }

    /// Get context-adjusted field threshold
    pub fn get_adjusted_field_threshold(
        &self,
        language: SourceLanguage,
        file_path: &str,
    ) -> usize {
        let base = self.get_field_threshold(language);
        let context = ContextAwareThresholds::default_config();
        context.apply_to_int_threshold(base as u32, file_path) as usize
    }
}
```

### Step 2.4: Update Recommended Threshold Values

Based on the audit, apply these calibrated values to `src/analysis/detectors/anti_patterns/long_methods/thresholds/language_thresholds.rs`:

```rust
/// Get thresholds for Python (UPDATED: reduced from 100 to 60 LOC)
fn python_thresholds() -> LanguageThresholds {
    LanguageThresholds {
        max_logical_loc: 60,  // Reduced from 100 (align with PEP 8)
        max_statements: 50,
        max_parameters: 5,
        max_nesting_depth: 5,
        max_cyclomatic_complexity: 10,
        max_cognitive_complexity: 15,
    }
}

/// Get thresholds for JavaScript (UPDATED: reduced from 80 to 70 LOC)
fn javascript_thresholds() -> LanguageThresholds {
    LanguageThresholds {
        max_logical_loc: 70,  // Reduced from 80
        max_statements: 40,
        max_parameters: 4,
        max_nesting_depth: 4,
        max_cyclomatic_complexity: 12,
        max_cognitive_complexity: 18,
    }
}

/// Get thresholds for Rust (UPDATED: slightly increased from 25 to 30 LOC)
fn rust_thresholds() -> LanguageThresholds {
    LanguageThresholds {
        max_logical_loc: 30,  // Increased from 25 (account for match expressions)
        max_statements: 30,
        max_parameters: 7,
        max_nesting_depth: 4,
        max_cyclomatic_complexity: 15,
        max_cognitive_complexity: 12,
    }
}
```

Update God Object thresholds in `src/analysis/detectors/anti_patterns/god_object/config.rs`:

```rust
impl Default for GodObjectConfig {
    fn default() -> Self {
        let mut method_thresholds = HashMap::new();
        method_thresholds.insert(SourceLanguage::Rust, 30);
        method_thresholds.insert(SourceLanguage::Python, 25);
        method_thresholds.insert(SourceLanguage::JavaScript, 20);
        method_thresholds.insert(SourceLanguage::TypeScript, 18); // UPDATED: increased from 15

        let mut field_thresholds = HashMap::new();
        field_thresholds.insert(SourceLanguage::Rust, 20);
        field_thresholds.insert(SourceLanguage::Python, 15);
        field_thresholds.insert(SourceLanguage::JavaScript, 12);
        field_thresholds.insert(SourceLanguage::TypeScript, 12); // UPDATED: increased from 10

        // ... rest of implementation
    }
}
```

Update Tight Coupling thresholds in `src/analysis/detectors/anti_patterns/tight_coupling/config.rs`:

```rust
impl Default for TightCouplingConfig {
    fn default() -> Self {
        Self {
            rust_thresholds: CouplingThresholds {
                fan_out_warning: 3,
                fan_out_critical: 7,
                cbo_warning: 6,
                cbo_critical: 10,
                rfc_warning: 15,
                rfc_critical: 25,
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            // UPDATED: Python thresholds reduced by 25%
            python_thresholds: CouplingThresholds {
                fan_out_warning: 8,   // Reduced from 10
                fan_out_critical: 11, // Reduced from 15
                cbo_warning: 6,       // Reduced from 8
                cbo_critical: 9,      // Reduced from 12
                rfc_warning: 14,      // Reduced from 18
                rfc_critical: 23,     // Reduced from 30
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            // UPDATED: JavaScript thresholds reduced by 20%
            javascript_thresholds: CouplingThresholds {
                fan_out_warning: 10,  // Reduced from 12
                fan_out_critical: 14, // Reduced from 18
                cbo_warning: 8,       // Reduced from 10
                cbo_critical: 12,     // Reduced from 15
                rfc_warning: 16,      // Reduced from 20
                rfc_critical: 28,     // Reduced from 35
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            // ... rest of implementation
        }
    }
}
```

## Phase 3: Testing & Validation

### Step 3.1: Create Test Corpus

```bash
# Create directory structure
mkdir -p test_corpus/{small,medium,large}/{rust,python,javascript}

# Collect sample files
# Small: < 1K LOC
# Medium: 1K-10K LOC
# Large: > 10K LOC
```

### Step 3.2: Run Baseline Tests

Create `scripts/baseline_calibration_test.sh`:

```bash
#!/bin/bash

echo "Running baseline calibration tests..."

# Run with current thresholds
cargo test --test calibration_baseline -- --nocapture

# Generate metrics
cargo run --bin analyze_calibration -- \
  --corpus test_corpus \
  --output baseline_metrics.json

echo "Baseline metrics saved to baseline_metrics.json"
```

### Step 3.3: Create Calibration Test Suite

Create `tests/calibration_tests.rs`:

```rust
use uveddi::analysis::detectors::calibration::*;

#[test]
fn test_confidence_band_consistency() {
    // Test that all detectors use consistent confidence levels
    let scores = vec![0.2, 0.4, 0.6, 0.8, 0.95];

    for score in scores {
        let band = ConfidenceBand::from_score(score);
        let level = ConfidenceLevel::from_score(score);

        assert_eq!(band.level, level);
        assert!(band.level.contains(score));
    }
}

#[test]
fn test_context_multipliers() {
    let adjuster = ContextAwareThresholds::default_config();

    // Production code: 1.0x
    assert_eq!(adjuster.get_effective_multiplier("src/main.rs"), 1.0);

    // Test code: 1.5x
    assert_eq!(adjuster.get_effective_multiplier("tests/test.rs"), 1.5);

    // Generated: should skip (infinite threshold)
    let threshold = adjuster.apply_to_threshold(100.0, "generated/foo.rs");
    assert!(threshold.is_infinite());
}

#[test]
fn test_profile_configurations() {
    let fast = ProfileConfig::fast();
    let balanced = ProfileConfig::balanced();
    let thorough = ProfileConfig::thorough();

    // Fast should have highest thresholds
    assert!(fast.confidence.ci_cd > balanced.confidence.ci_cd);
    assert!(balanced.confidence.ci_cd > thorough.confidence.ci_cd);

    // Thorough should analyze more files
    assert!(!thorough.skip_generated);
    assert!(!thorough.skip_tests);
    assert!(fast.skip_generated);
    assert!(fast.skip_tests);
}

#[test]
fn test_severity_calculation() {
    let components = vec![
        SeverityComponent::new("complexity", 30, 0.5),
        SeverityComponent::new("size", 40, 0.5),
    ];

    let severity = SeverityScore::from_components(components);
    assert_eq!(severity.score, 70); // 30 + 40
    assert_eq!(severity.level, StandardSeverity::Medium);
}
```

### Step 3.4: A/B Testing Framework

Create example A/B test:

```rust
use uveddi::analysis::detectors::calibration::testing::*;

#[test]
fn test_threshold_ab_comparison() {
    let ab_test = ABTest::new(
        "long_methods_threshold",
        0.7,  // Current threshold
        0.75, // Proposed threshold
    );

    // Run comparison (would analyze same corpus with both thresholds)
    let result = ab_test.run_comparison();

    // Check if variant is statistically better
    if result.statistical_significance {
        println!("Variant shows significant improvement!");
        println!("Recommendation: {}", result.recommendation);
    }
}
```

## Phase 4: Deployment

### Step 4.1: Gradual Rollout Plan

**Week 1-2: Internal Testing**
- Deploy to development environment
- Collect metrics from dogfooding
- Adjust based on internal feedback

**Week 3-4: Beta Testing**
- Release as opt-in feature flag
- Collect user feedback
- Monitor key metrics

**Week 5-6: Full Deployment**
- Make calibrated system default
- Provide migration guide
- Maintain legacy mode for backward compatibility

### Step 4.2: Feature Flag Implementation

Add to `Cargo.toml`:

```toml
[features]
default = ["calibrated-thresholds"]
calibrated-thresholds = []
legacy-thresholds = []
```

Use in code:

```rust
#[cfg(feature = "calibrated-thresholds")]
let config = CalibratedConfig::balanced();

#[cfg(feature = "legacy-thresholds")]
let config = LegacyConfig::default();
```

### Step 4.3: Configuration File Support

Create `.uveddi/calibration.toml`:

```toml
[profile]
type = "balanced"  # fast, balanced, thorough, paranoid, legacy

[confidence]
development = 0.5
staging = 0.6
production = 0.7
ci_cd = 0.8

[severity]
development = "Info"
staging = "Low"
production = "Medium"
ci_cd = "High"

[context]
skip_generated = true
skip_third_party = true
legacy_multiplier = 1.2

[feedback]
enable_auto_calibration = false
min_feedback_count = 100
max_fp_rate = 0.2
```

### Step 4.4: CLI Integration

Update `src/cli/commands/analyze.rs`:

```rust
#[derive(Parser)]
pub struct AnalyzeArgs {
    // ... existing args ...

    /// Analysis profile to use
    #[arg(long, default_value = "balanced")]
    profile: String,

    /// Enable auto-calibration based on feedback
    #[arg(long)]
    enable_auto_calibration: bool,
}

impl AnalyzeArgs {
    pub fn get_profile(&self) -> ProfileConfig {
        match self.profile.as_str() {
            "fast" => ProfileConfig::fast(),
            "balanced" => ProfileConfig::balanced(),
            "thorough" => ProfileConfig::thorough(),
            "paranoid" => ProfileConfig::paranoid(),
            "legacy" => ProfileConfig::legacy(),
            _ => ProfileConfig::balanced(),
        }
    }
}
```

## API Reference

### Core Types

#### ConfidenceLevel

```rust
pub enum ConfidenceLevel {
    Low,      // 0.0 - 0.3
    Medium,   // 0.3 - 0.5
    High,     // 0.5 - 0.7
    Critical, // 0.7 - 0.9
    Certain,  // 0.9 - 1.0
}
```

#### StandardSeverity

```rust
pub enum StandardSeverity {
    Info,     // 0-25
    Low,      // 25-50
    Medium,   // 50-75
    High,     // 75-90
    Critical, // 90-100
}
```

#### CodeContext

```rust
pub enum CodeContext {
    Production,    // 1.0x
    Test,         // 1.5x
    Generated,    // 3.0x or skip
    Framework,    // 2.0x
    Legacy,       // 1.2x (configurable)
    ThirdParty,   // 2.5x or skip
    Build,        // 2.0x
    Documentation,// 2.5x
}
```

#### DetectorProfile

```rust
pub enum DetectorProfile {
    Fast,      // Quick CI/CD analysis
    Balanced,  // Standard development
    Thorough,  // Deep pre-release analysis
    Paranoid,  // Maximum sensitivity
    Legacy,    // Legacy code analysis
    Custom,    // User-defined
}
```

### Usage Examples

See test files in each module for comprehensive examples.

## Migration Checklist

- [ ] Integrate calibration module into detectors/mod.rs
- [ ] Update SecurityConfig with ConfidenceThresholds
- [ ] Update LongMethodsConfig with context awareness
- [ ] Update GodObjectConfig with context awareness
- [ ] Update TightCouplingConfig thresholds
- [ ] Update CodeDuplicationConfig profiles
- [ ] Create test corpus
- [ ] Run baseline tests
- [ ] Implement A/B testing
- [ ] Add CLI profile support
- [ ] Create configuration file format
- [ ] Write user documentation
- [ ] Set up feedback collection
- [ ] Deploy feature flag
- [ ] Monitor metrics
- [ ] Adjust based on real-world feedback

## Success Metrics

Target metrics within 3 months of full deployment:

- **False Positive Rate**: < 15% (from ~25%)
- **Recall**: > 85% (from ~70%)
- **F1 Score**: > 0.80 (from ~0.65)
- **User Satisfaction**: > 4.0/5.0
- **Analysis Speed**: < 10% performance impact

## Support & Feedback

For questions or issues:
- GitHub Issues: https://github.com/yourusername/uveddi/issues
- Documentation: https://docs.uveddi.dev/calibration
- Email: support@uveddi.dev
