use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Note: semver dependency would be needed for production use
// use semver::{Version, VersionReq};
use super::{AnalysisOutput, DependencyAnalyzer};
use crate::analysis::detectors::dependency::config::*;
use crate::analysis::detectors::dependency::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionAnalysis {
    pub version_conflicts: Vec<VersionConflict>,
    pub version_ranges: HashMap<String, VersionRange>,
    pub resolution_suggestions: Vec<ResolutionSuggestion>,
    pub semantic_version_stats: SemanticVersionStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConflict {
    pub package: String,
    pub conflicting_versions: Vec<String>,
    pub dependent_packages: Vec<String>,
    pub severity: ConflictSeverity,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRange {
    pub min: String,
    pub max: String,
    pub current: String,
    pub is_compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionSuggestion {
    pub package: String,
    pub current_version: String,
    pub suggested_version: String,
    pub reason: String,
    pub impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticVersionStats {
    pub total_packages: usize,
    pub with_semver: usize,
    pub major_versions: HashMap<String, usize>,
    pub prerelease_count: usize,
    pub exact_versions: usize,
    pub range_versions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    None,
    Minor,
    Moderate,
    Major,
}

pub struct VersionAnalyzer {
    version_map: HashMap<String, Vec<String>>,
}

impl VersionAnalyzer {
    pub fn new() -> Self {
        Self {
            version_map: HashMap::new(),
        }
    }

    fn analyze_versions(&mut self, dependencies: &[DependencyInfo]) -> VersionAnalysis {
        self.build_version_map(dependencies);

        let conflicts = self.detect_conflicts();
        let ranges = self.analyze_ranges(dependencies);
        let suggestions = self.generate_suggestions(&conflicts);
        let stats = self.calculate_stats(dependencies);

        VersionAnalysis {
            version_conflicts: conflicts,
            version_ranges: ranges,
            resolution_suggestions: suggestions,
            semantic_version_stats: stats,
        }
    }

    fn build_version_map(&mut self, dependencies: &[DependencyInfo]) {
        for dep in dependencies {
            let version = dep
                .version
                .as_ref()
                .unwrap_or(&"unknown".to_string())
                .clone();
            self.version_map
                .entry(dep.name.clone())
                .or_insert_with(Vec::new)
                .push(version);
        }
    }

    fn detect_conflicts(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();

        for (package, versions) in &self.version_map {
            if versions.len() > 1 {
                let unique_versions: Vec<_> = versions
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .cloned()
                    .collect();

                if unique_versions.len() > 1 {
                    let severity = self.assess_conflict_severity(&unique_versions);
                    let resolution = self.suggest_resolution(&unique_versions);

                    conflicts.push(VersionConflict {
                        package: package.clone(),
                        conflicting_versions: unique_versions,
                        dependent_packages: Vec::new(), // Would need dependency graph to fill this
                        severity,
                        resolution,
                    });
                }
            }
        }

        conflicts
    }

    fn assess_conflict_severity(&self, versions: &[String]) -> ConflictSeverity {
        // Simple heuristic without semver parsing
        let unique_versions: std::collections::HashSet<_> = versions.iter().collect();

        if unique_versions.len() <= 1 {
            return ConflictSeverity::Low;
        }

        // Check for major version differences (simple pattern matching)
        let has_major_diff = versions.iter().any(|v1| {
            versions.iter().any(|v2| {
                let v1_major = v1.split('.').next().unwrap_or("0");
                let v2_major = v2.split('.').next().unwrap_or("0");
                v1_major != v2_major
            })
        });

        if has_major_diff {
            ConflictSeverity::High
        } else if unique_versions.len() > 2 {
            ConflictSeverity::Medium
        } else {
            ConflictSeverity::Low
        }
    }

    fn suggest_resolution(&self, versions: &[String]) -> Option<String> {
        // Simple resolution: suggest the last version in the list
        versions.last().cloned()
    }

    fn analyze_ranges(&self, dependencies: &[DependencyInfo]) -> HashMap<String, VersionRange> {
        let mut ranges = HashMap::new();

        for (package, versions) in &self.version_map {
            if !versions.is_empty() {
                let min = versions.iter().min().unwrap();
                let max = versions.iter().max().unwrap();
                let current = versions.last().unwrap();

                ranges.insert(
                    package.clone(),
                    VersionRange {
                        min: min.clone(),
                        max: max.clone(),
                        current: current.clone(),
                        is_compatible: self.check_compatibility(min, max),
                    },
                );
            }
        }

        ranges
    }

    fn check_compatibility(&self, min: &str, max: &str) -> bool {
        // Simple compatibility check based on major version
        let min_major = min.split('.').next().unwrap_or("0");
        let max_major = max.split('.').next().unwrap_or("0");
        min_major == max_major
    }

    fn generate_suggestions(&self, conflicts: &[VersionConflict]) -> Vec<ResolutionSuggestion> {
        let mut suggestions = Vec::new();

        for conflict in conflicts {
            if let Some(resolution) = &conflict.resolution {
                let current = conflict
                    .conflicting_versions
                    .first()
                    .unwrap_or(&"unknown".to_string());

                suggestions.push(ResolutionSuggestion {
                    package: conflict.package.clone(),
                    current_version: current.clone(),
                    suggested_version: resolution.clone(),
                    reason: format!(
                        "Resolve version conflict (severity: {:?})",
                        conflict.severity
                    ),
                    impact: self.assess_impact(&conflict.severity),
                });
            }
        }

        suggestions
    }

    fn assess_impact(&self, severity: &ConflictSeverity) -> ImpactLevel {
        match severity {
            ConflictSeverity::Low => ImpactLevel::Minor,
            ConflictSeverity::Medium => ImpactLevel::Moderate,
            ConflictSeverity::High | ConflictSeverity::Critical => ImpactLevel::Major,
        }
    }

    fn calculate_stats(&self, dependencies: &[DependencyInfo]) -> SemanticVersionStats {
        let mut stats = SemanticVersionStats {
            total_packages: dependencies.len(),
            with_semver: 0,
            major_versions: HashMap::new(),
            prerelease_count: 0,
            exact_versions: 0,
            range_versions: 0,
        };

        for dep in dependencies {
            if let Some(version_str) = &dep.version {
                // Simple semver detection based on pattern matching
                if version_str.contains('.')
                    && version_str.chars().next().unwrap_or('a').is_ascii_digit()
                {
                    stats.with_semver += 1;
                    if let Some(major_str) = version_str.split('.').next() {
                        if let Ok(major) = major_str.parse::<usize>() {
                            stats.major_versions.insert(dep.name.clone(), major);
                        }
                    }

                    if version_str.contains('-') {
                        stats.prerelease_count += 1;
                    }
                }

                if version_str.contains('^')
                    || version_str.contains('~')
                    || version_str.contains('*')
                {
                    stats.range_versions += 1;
                } else {
                    stats.exact_versions += 1;
                }
            }
        }

        stats
    }
}

impl DependencyAnalyzer for VersionAnalyzer {
    fn analyze(
        &self,
        dependencies: &[DependencyInfo],
        _config: &DependencyDetectorConfig,
    ) -> Result<AnalysisOutput, DependencyError> {
        let mut analyzer = Self::new();
        let analysis = analyzer.analyze_versions(dependencies);
        Ok(AnalysisOutput::Version(analysis))
    }

    fn name(&self) -> &str {
        "VersionAnalyzer"
    }

    fn description(&self) -> &str {
        "Analyzes version conflicts, ranges, and compatibility"
    }
}
