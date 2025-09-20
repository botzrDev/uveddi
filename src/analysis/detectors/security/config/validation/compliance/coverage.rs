//! Compliance coverage tracking
//!
//! This module tracks compliance coverage for security issues.

use super::super::super::types::ConfigIssue;
use super::types::{ComplianceStandard, ComplianceRequirement};
use std::collections::HashMap;

/// Compliance coverage statistics
#[derive(Debug, Clone)]
pub struct ComplianceCoverage {
    pub standard_name: String,
    pub version: String,
    pub total_requirements: usize,
    pub covered_requirements: usize,
    pub coverage_percentage: f64,
}

impl ComplianceCoverage {
    /// Create new compliance coverage
    pub fn new(standard_name: String, version: String) -> Self {
        Self {
            standard_name,
            version,
            total_requirements: 0,
            covered_requirements: 0,
            coverage_percentage: 0.0,
        }
    }

    /// Calculate coverage from requirements
    pub fn from_requirements(
        standard: &ComplianceStandard,
        covered_requirements: usize,
    ) -> Self {
        let total = standard.requirements.len();
        let coverage_percentage = if total > 0 {
            (covered_requirements as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            standard_name: standard.name.clone(),
            version: standard.version.clone(),
            total_requirements: total,
            covered_requirements,
            coverage_percentage,
        }
    }

    /// Get coverage level description
    pub fn get_coverage_level(&self) -> CoverageLevel {
        match self.coverage_percentage {
            p if p >= 90.0 => CoverageLevel::Excellent,
            p if p >= 75.0 => CoverageLevel::Good,
            p if p >= 50.0 => CoverageLevel::Fair,
            p if p >= 25.0 => CoverageLevel::Poor,
            _ => CoverageLevel::Minimal,
        }
    }

    /// Check if coverage meets minimum threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.coverage_percentage >= threshold
    }
}

/// Coverage level classification
#[derive(Debug, Clone, PartialEq)]
pub enum CoverageLevel {
    Excellent, // 90%+
    Good,      // 75-89%
    Fair,      // 50-74%
    Poor,      // 25-49%
    Minimal,   // <25%
}

impl CoverageLevel {
    /// Get description of the coverage level
    pub fn description(&self) -> &'static str {
        match self {
            CoverageLevel::Excellent => "Excellent compliance coverage",
            CoverageLevel::Good => "Good compliance coverage",
            CoverageLevel::Fair => "Fair compliance coverage",
            CoverageLevel::Poor => "Poor compliance coverage",
            CoverageLevel::Minimal => "Minimal compliance coverage",
        }
    }

    /// Get recommended actions for this coverage level
    pub fn recommendations(&self) -> Vec<&'static str> {
        match self {
            CoverageLevel::Excellent => vec!["Maintain current security practices"],
            CoverageLevel::Good => vec!["Minor improvements recommended", "Consider additional controls"],
            CoverageLevel::Fair => vec!["Significant improvements needed", "Prioritize high-risk areas"],
            CoverageLevel::Poor => vec!["Major security gaps detected", "Immediate action required"],
            CoverageLevel::Minimal => vec!["Critical security deficiencies", "Comprehensive security review needed"],
        }
    }
}

/// Coverage calculator utility
pub struct CoverageCalculator;

impl CoverageCalculator {
    /// Calculate compliance coverage for multiple standards
    pub fn calculate_coverage(
        standards: &[ComplianceStandard],
        issues: &[ConfigIssue],
    ) -> HashMap<String, ComplianceCoverage> {
        let mut coverage = HashMap::new();

        for standard in standards {
            if !standard.enabled {
                continue;
            }

            let covered_count = Self::count_covered_requirements(standard, issues);
            let standard_coverage = ComplianceCoverage::from_requirements(standard, covered_count);
            coverage.insert(standard.name.clone(), standard_coverage);
        }

        coverage
    }

    /// Count requirements covered by the given issues
    fn count_covered_requirements(standard: &ComplianceStandard, issues: &[ConfigIssue]) -> usize {
        standard.requirements.iter()
            .filter(|requirement| Self::requirement_is_covered(requirement, issues))
            .count()
    }

    /// Check if a requirement is covered by any of the issues
    fn requirement_is_covered(requirement: &ComplianceRequirement, issues: &[ConfigIssue]) -> bool {
        issues.iter().any(|issue| {
            // Check if requirement applies to this issue
            let applies_by_cwe = issue.cwe_id.map_or(false, |cwe| requirement.applies_to_cwe(cwe));
            let applies_by_tags = requirement.applies_to_tags(&issue.tags);

            applies_by_cwe || applies_by_tags
        })
    }

    /// Generate coverage summary
    pub fn generate_summary(coverage: &HashMap<String, ComplianceCoverage>) -> ComplianceSummary {
        let total_standards = coverage.len();
        let total_requirements: usize = coverage.values().map(|c| c.total_requirements).sum();
        let total_covered: usize = coverage.values().map(|c| c.covered_requirements).sum();

        let overall_percentage = if total_requirements > 0 {
            (total_covered as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        ComplianceSummary {
            total_standards,
            total_requirements,
            total_covered,
            overall_percentage,
            coverage_level: Self::get_coverage_level_for_percentage(overall_percentage),
        }
    }

    fn get_coverage_level_for_percentage(percentage: f64) -> CoverageLevel {
        match percentage {
            p if p >= 90.0 => CoverageLevel::Excellent,
            p if p >= 75.0 => CoverageLevel::Good,
            p if p >= 50.0 => CoverageLevel::Fair,
            p if p >= 25.0 => CoverageLevel::Poor,
            _ => CoverageLevel::Minimal,
        }
    }
}

/// Overall compliance summary
#[derive(Debug, Clone)]
pub struct ComplianceSummary {
    pub total_standards: usize,
    pub total_requirements: usize,
    pub total_covered: usize,
    pub overall_percentage: f64,
    pub coverage_level: CoverageLevel,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ComplianceRequirement;
    use super::super::super::super::types::ConfigSeverity;

    #[test]
    fn test_coverage_calculation() {
        let coverage = ComplianceCoverage::new("Test Standard".to_string(), "1.0".to_string());
        assert_eq!(coverage.coverage_percentage, 0.0);
        assert_eq!(coverage.get_coverage_level(), CoverageLevel::Minimal);
    }

    #[test]
    fn test_coverage_levels() {
        let excellent = ComplianceCoverage {
            standard_name: "Test".to_string(),
            version: "1.0".to_string(),
            total_requirements: 10,
            covered_requirements: 9,
            coverage_percentage: 90.0,
        };

        assert_eq!(excellent.get_coverage_level(), CoverageLevel::Excellent);
        assert!(excellent.meets_threshold(85.0));
    }

    #[test]
    fn test_coverage_level_descriptions() {
        assert_eq!(CoverageLevel::Excellent.description(), "Excellent compliance coverage");
        assert!(!CoverageLevel::Poor.recommendations().is_empty());
    }
}