//! Standardized severity scoring system (0-100 scale)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standardized severity levels with consistent 0-100 scoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StandardSeverity {
    /// 0-25: Informational findings
    Info,
    /// 25-50: Low severity issues
    Low,
    /// 50-75: Medium severity issues
    Medium,
    /// 75-90: High severity issues
    High,
    /// 90-100: Critical severity issues
    Critical,
}

impl StandardSeverity {
    /// Convert a severity score (0-100) to a severity level
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=25 => Self::Info,
            26..=50 => Self::Low,
            51..=75 => Self::Medium,
            76..=90 => Self::High,
            _ => Self::Critical,
        }
    }

    /// Get the numeric range for this severity level
    pub fn range(&self) -> (u32, u32) {
        match self {
            Self::Info => (0, 25),
            Self::Low => (25, 50),
            Self::Medium => (50, 75),
            Self::High => (75, 90),
            Self::Critical => (90, 100),
        }
    }

    /// Get the minimum score for this severity level
    pub fn min_score(&self) -> u32 {
        self.range().0
    }

    /// Get the maximum score for this severity level
    pub fn max_score(&self) -> u32 {
        self.range().1
    }

    /// Get the midpoint score for this level
    pub fn midpoint(&self) -> u32 {
        let (min, max) = self.range();
        (min + max) / 2
    }

    /// Get numeric value for ordering (0 = Info, 4 = Critical)
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    /// Check if a score falls within this severity level
    pub fn contains(&self, score: u32) -> bool {
        let (min, max) = self.range();
        score >= min && score <= max
    }
}

impl fmt::Display for StandardSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Severity score with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityScore {
    /// The severity level
    pub level: StandardSeverity,
    /// Raw severity score (0-100)
    pub score: u32,
    /// Components contributing to the score
    pub components: Vec<SeverityComponent>,
    /// Explanation of the score
    pub explanation: String,
}

impl SeverityScore {
    /// Create a new severity score
    pub fn new(score: u32) -> Self {
        let level = StandardSeverity::from_score(score);
        Self {
            level,
            score: score.min(100),
            components: Vec::new(),
            explanation: format!("{} severity (score: {})", level, score),
        }
    }

    /// Create a severity score with components
    pub fn from_components(components: Vec<SeverityComponent>) -> Self {
        let score = components.iter().map(|c| c.score).sum::<u32>().min(100);
        let level = StandardSeverity::from_score(score);

        let explanation = if components.is_empty() {
            format!("{} severity (score: {})", level, score)
        } else {
            let comp_desc: Vec<String> = components
                .iter()
                .map(|c| format!("{} ({})", c.name, c.score))
                .collect();
            format!(
                "{} severity (score: {}) - Breakdown: {}",
                level,
                score,
                comp_desc.join(", ")
            )
        };

        Self {
            level,
            score,
            components,
            explanation,
        }
    }

    /// Add a severity component
    pub fn add_component(&mut self, component: SeverityComponent) {
        self.components.push(component);
        self.recalculate();
    }

    /// Recalculate score from components
    fn recalculate(&mut self) {
        self.score = self.components.iter().map(|c| c.score).sum::<u32>().min(100);
        self.level = StandardSeverity::from_score(self.score);
    }

    /// Apply a multiplier to the score (e.g., for context adjustments)
    pub fn apply_multiplier(&mut self, multiplier: f64) {
        self.score = ((self.score as f64 * multiplier) as u32).min(100);
        self.level = StandardSeverity::from_score(self.score);
    }

    /// Check if this severity should be reported given a minimum level
    pub fn should_report(&self, min_level: StandardSeverity) -> bool {
        self.level >= min_level
    }
}

/// Component contributing to overall severity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityComponent {
    /// Component name (e.g., "lines_of_code", "complexity", "coupling")
    pub name: String,
    /// Score contributed by this component (0-100)
    pub score: u32,
    /// Weight of this component in overall score (0.0-1.0)
    pub weight: f64,
    /// Description of what this component measures
    pub description: String,
}

impl SeverityComponent {
    /// Create a new severity component
    pub fn new(name: impl Into<String>, score: u32, weight: f64) -> Self {
        Self {
            name: name.into(),
            score: score.min(100),
            weight: weight.clamp(0.0, 1.0),
            description: String::new(),
        }
    }

    /// Create a component with description
    pub fn with_description(
        name: impl Into<String>,
        score: u32,
        weight: f64,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            score: score.min(100),
            weight: weight.clamp(0.0, 1.0),
            description: description.into(),
        }
    }

    /// Get the weighted score
    pub fn weighted_score(&self) -> u32 {
        (self.score as f64 * self.weight) as u32
    }
}

/// Severity threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityThresholds {
    /// Minimum severity to report in development
    pub development: StandardSeverity,
    /// Minimum severity to report in staging
    pub staging: StandardSeverity,
    /// Minimum severity to report in production
    pub production: StandardSeverity,
    /// Minimum severity to block in CI/CD
    pub ci_cd: StandardSeverity,
}

impl SeverityThresholds {
    /// Create balanced thresholds
    pub fn balanced() -> Self {
        Self {
            development: StandardSeverity::Info,   // Report everything in dev
            staging: StandardSeverity::Low,        // Filter out info in staging
            production: StandardSeverity::Medium,  // Only real issues in prod
            ci_cd: StandardSeverity::High,         // Block only serious issues
        }
    }

    /// Create strict thresholds
    pub fn strict() -> Self {
        Self {
            development: StandardSeverity::Info,
            staging: StandardSeverity::Info,
            production: StandardSeverity::Low,
            ci_cd: StandardSeverity::Medium,
        }
    }

    /// Create lenient thresholds
    pub fn lenient() -> Self {
        Self {
            development: StandardSeverity::Low,
            staging: StandardSeverity::Medium,
            production: StandardSeverity::High,
            ci_cd: StandardSeverity::Critical,
        }
    }

    /// Get threshold for environment
    pub fn get(&self, environment: &str) -> StandardSeverity {
        match environment.to_lowercase().as_str() {
            "dev" | "development" => self.development,
            "stage" | "staging" => self.staging,
            "prod" | "production" => self.production,
            "ci" | "cicd" | "ci_cd" | "ci-cd" => self.ci_cd,
            _ => self.production,
        }
    }
}

impl Default for SeverityThresholds {
    fn default() -> Self {
        Self::balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_from_score() {
        assert_eq!(StandardSeverity::from_score(10), StandardSeverity::Info);
        assert_eq!(StandardSeverity::from_score(35), StandardSeverity::Low);
        assert_eq!(StandardSeverity::from_score(60), StandardSeverity::Medium);
        assert_eq!(StandardSeverity::from_score(80), StandardSeverity::High);
        assert_eq!(StandardSeverity::from_score(95), StandardSeverity::Critical);
    }

    #[test]
    fn test_severity_ranges() {
        assert!(StandardSeverity::Low.contains(30));
        assert!(!StandardSeverity::Low.contains(60));
        assert_eq!(StandardSeverity::Medium.midpoint(), 62);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(StandardSeverity::Info < StandardSeverity::Low);
        assert!(StandardSeverity::Medium < StandardSeverity::High);
        assert!(StandardSeverity::High < StandardSeverity::Critical);
    }

    #[test]
    fn test_severity_score_from_components() {
        let components = vec![
            SeverityComponent::new("complexity", 30, 0.4),
            SeverityComponent::new("size", 20, 0.3),
            SeverityComponent::new("coupling", 15, 0.3),
        ];

        let severity = SeverityScore::from_components(components);
        assert_eq!(severity.score, 65); // 30 + 20 + 15
        assert_eq!(severity.level, StandardSeverity::Medium);
    }

    #[test]
    fn test_severity_multiplier() {
        let mut severity = SeverityScore::new(50);
        assert_eq!(severity.level, StandardSeverity::Medium);

        severity.apply_multiplier(1.5); // Test code multiplier
        assert_eq!(severity.score, 75);
        assert_eq!(severity.level, StandardSeverity::High);
    }

    #[test]
    fn test_severity_thresholds() {
        let thresholds = SeverityThresholds::balanced();
        assert_eq!(thresholds.get("dev"), StandardSeverity::Info);
        assert_eq!(thresholds.get("production"), StandardSeverity::Medium);
        assert_eq!(thresholds.get("ci-cd"), StandardSeverity::High);
    }
}
