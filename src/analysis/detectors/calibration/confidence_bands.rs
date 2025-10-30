//! Standardized confidence level bands for consistent reporting across detectors

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standardized confidence levels with consistent semantics across all detectors
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// 0.0 - 0.3: Experimental/Noise - May be false positive
    Low,
    /// 0.3 - 0.5: Possible issues - Requires human review
    Medium,
    /// 0.5 - 0.7: Likely issues - Probable problems
    High,
    /// 0.7 - 0.9: Definite issues - High certainty
    Critical,
    /// 0.9 - 1.0: Confirmed issues - Virtually certain
    Certain,
}

impl ConfidenceLevel {
    /// Convert a raw confidence score (0.0-1.0) to a confidence level
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.3 => Self::Low,
            s if s < 0.5 => Self::Medium,
            s if s < 0.7 => Self::High,
            s if s < 0.9 => Self::Critical,
            _ => Self::Certain,
        }
    }

    /// Get the numeric range for this confidence level
    pub fn range(&self) -> (f64, f64) {
        match self {
            Self::Low => (0.0, 0.3),
            Self::Medium => (0.3, 0.5),
            Self::High => (0.5, 0.7),
            Self::Critical => (0.7, 0.9),
            Self::Certain => (0.9, 1.0),
        }
    }

    /// Get the minimum threshold for this level
    pub fn min_threshold(&self) -> f64 {
        self.range().0
    }

    /// Get the maximum threshold for this level
    pub fn max_threshold(&self) -> f64 {
        self.range().1
    }

    /// Get the midpoint score for this level
    pub fn midpoint(&self) -> f64 {
        let (min, max) = self.range();
        (min + max) / 2.0
    }

    /// Check if a score falls within this confidence level
    pub fn contains(&self, score: f64) -> bool {
        let (min, max) = self.range();
        score >= min && score < max
    }
}

impl fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
            Self::Certain => write!(f, "Certain"),
        }
    }
}

/// Confidence band with associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceBand {
    /// The confidence level
    pub level: ConfidenceLevel,
    /// Raw confidence score (0.0-1.0)
    pub score: f64,
    /// Human-readable description
    pub description: String,
    /// Recommended action
    pub recommendation: String,
}

impl ConfidenceBand {
    /// Create a new confidence band from a raw score
    pub fn from_score(score: f64) -> Self {
        let level = ConfidenceLevel::from_score(score);
        let (description, recommendation) = match level {
            ConfidenceLevel::Low => (
                "Experimental finding with high uncertainty".to_string(),
                "Review manually, likely noise".to_string(),
            ),
            ConfidenceLevel::Medium => (
                "Possible issue requiring investigation".to_string(),
                "Schedule for code review".to_string(),
            ),
            ConfidenceLevel::High => (
                "Likely issue with good evidence".to_string(),
                "Prioritize for refactoring".to_string(),
            ),
            ConfidenceLevel::Critical => (
                "Definite issue with strong evidence".to_string(),
                "Address in current sprint".to_string(),
            ),
            ConfidenceLevel::Certain => (
                "Confirmed issue requiring immediate attention".to_string(),
                "Fix immediately".to_string(),
            ),
        };

        Self {
            level,
            score,
            description,
            recommendation,
        }
    }

    /// Adjust confidence score based on additional factors
    pub fn adjust(&mut self, factor: f64) {
        self.score = (self.score * factor).clamp(0.0, 1.0);
        self.level = ConfidenceLevel::from_score(self.score);
    }

    /// Check if this confidence level should be reported given a minimum threshold
    pub fn should_report(&self, min_threshold: f64) -> bool {
        self.score >= min_threshold
    }
}

/// Configuration for confidence thresholds across different environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceThresholds {
    /// Development environment (lower threshold for early detection)
    pub development: f64,
    /// Staging environment (balanced threshold)
    pub staging: f64,
    /// Production environment (higher threshold, fewer false positives)
    pub production: f64,
    /// CI/CD environment (highest threshold, block only definite issues)
    pub ci_cd: f64,
}

impl ConfidenceThresholds {
    /// Create calibrated thresholds with gradual progression
    pub fn calibrated() -> Self {
        Self {
            development: 0.5,  // High confidence - catch likely issues in dev
            staging: 0.6,      // Between High and Critical - 20% increase
            production: 0.7,   // Critical confidence - 40% total increase from dev
            ci_cd: 0.8,        // Near-Certain - 60% total increase, block only serious issues
        }
    }

    /// Create legacy thresholds (current inconsistent values)
    pub fn legacy() -> Self {
        Self {
            development: 0.5,
            staging: 0.65,
            production: 0.7,
            ci_cd: 0.8,
        }
    }

    /// Get threshold for a specific environment
    pub fn get(&self, environment: &str) -> f64 {
        match environment.to_lowercase().as_str() {
            "dev" | "development" => self.development,
            "stage" | "staging" => self.staging,
            "prod" | "production" => self.production,
            "ci" | "cicd" | "ci_cd" | "ci-cd" => self.ci_cd,
            _ => self.production, // Default to production threshold
        }
    }
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self::calibrated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_level_from_score() {
        assert_eq!(ConfidenceLevel::from_score(0.2), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.4), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.6), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.8), ConfidenceLevel::Critical);
        assert_eq!(ConfidenceLevel::from_score(0.95), ConfidenceLevel::Certain);
    }

    #[test]
    fn test_confidence_level_ranges() {
        assert!(ConfidenceLevel::Low.contains(0.2));
        assert!(!ConfidenceLevel::Low.contains(0.4));
        assert_eq!(ConfidenceLevel::Medium.midpoint(), 0.4);
    }

    #[test]
    fn test_confidence_band_adjustment() {
        let mut band = ConfidenceBand::from_score(0.6);
        assert_eq!(band.level, ConfidenceLevel::High);

        band.adjust(1.2); // Increase confidence
        assert_eq!(band.level, ConfidenceLevel::Critical);

        band.adjust(0.5); // Decrease confidence
        assert_eq!(band.level, ConfidenceLevel::Medium);
    }

    #[test]
    fn test_calibrated_thresholds() {
        let thresholds = ConfidenceThresholds::calibrated();

        // Verify gradual progression
        assert!(thresholds.development < thresholds.staging);
        assert!(thresholds.staging < thresholds.production);
        assert!(thresholds.production < thresholds.ci_cd);

        // Verify no more than 20% increase between levels
        let dev_to_stage = (thresholds.staging - thresholds.development) / thresholds.development;
        assert!(dev_to_stage <= 0.2, "Stage to dev increase too large: {}", dev_to_stage);
    }

    #[test]
    fn test_confidence_threshold_lookup() {
        let thresholds = ConfidenceThresholds::calibrated();
        assert_eq!(thresholds.get("development"), 0.5);
        assert_eq!(thresholds.get("prod"), 0.7);
        assert_eq!(thresholds.get("ci-cd"), 0.8);
        assert_eq!(thresholds.get("unknown"), 0.7); // Defaults to production
    }
}
