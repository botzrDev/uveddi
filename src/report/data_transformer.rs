//! Data Transformation Module
//!
//! This module provides transformation utilities to convert between different
//! data formats used throughout the Uveddi system, ensuring consistency
//! across the analysis engine, report generation, and API layers.

use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use crate::report::interactive_models::{
    AnalysisSummary, DependencyGraph, Finding, GraphMetadata, InteractiveReport, ProjectMetadata, ReportMetadata, REPORT_SCHEMA_VERSION,
};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Transform backend analysis data into the InteractiveReport format for API consumption
pub struct DataTransformer;

impl DataTransformer {
    /// Convert raw analysis data into an InteractiveReport
    pub fn transform_to_interactive_report(
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        project_name: String,
        project_path: String,
    ) -> InteractiveReport {
        // Transform issues to findings
        let findings = Self::transform_issues_to_findings(issues, anti_pattern_types);

        // Calculate summary statistics
        let summary = Self::calculate_summary(analysis_run, &findings);

        // Create project metadata
        let project = ProjectMetadata {
            id: format!("project-{}", analysis_run.project_id),
            name: project_name,
            commit: None,   // TODO: Extract from analysis config
            branch: None,   // TODO: Extract from analysis config
            repo_url: None, // TODO: Extract from analysis config
            path: project_path,
            languages: Self::detect_languages(issues),
        };

        // Create empty dependency graph (to be populated later)
        let dependency_graph = DependencyGraph {
            nodes: vec![],
            edges: vec![],
            metadata: GraphMetadata {
                node_count: 0,
                edge_count: 0,
                has_cycles: false,
                max_depth: 0,
                suggested_layout: crate::report::interactive_models::CytoscapeLayout::Dagre,
                layout_config: HashMap::new(),
                performance_config: crate::report::interactive_models::GraphPerformanceConfig {
                    enable_lod: true,
                    batch_size: 100,
                    texture_on_viewport: true,
                    hide_labels_on_viewport: true,
                    initial_viewport: None,
                    use_web_worker: false,
                },
                clustering_hints: vec![],
                cycles: vec![],
            },
        };

        // Create report metadata
        let metadata = ReportMetadata {
            generated_at: Utc::now(),
            uveddi_version: env!("CARGO_PKG_VERSION").to_string(),
            configuration: HashMap::new(),
            performance: Some(crate::report::interactive_models::GenerationPerformance {
                analysis_duration_ms: analysis_run
                    .end_time
                    .map(|end| (end - analysis_run.start_time).num_milliseconds() as u64)
                    .unwrap_or(0),
                generation_duration_ms: 0, // TODO: Track generation time
                peak_memory_bytes: None,
                files_per_second: analysis_run.total_files_analyzed.and_then(|files| {
                    analysis_run
                        .end_time
                        .map(|end| {
                            let duration_secs =
                                (end - analysis_run.start_time).num_seconds() as f64;
                            if duration_secs > 0.0 {
                                Some(files as f64 / duration_secs)
                            } else {
                                None
                            }
                        })
                        .flatten()
                }),
            }),
        };

        InteractiveReport {
            schema_version: REPORT_SCHEMA_VERSION.to_string(),
            project,
            summary,
            findings,
            dependency_graph,
            diagrams: vec![],
            chart_data: None,
            performance_metrics: None,
            ai_insights: None,
            #[cfg(feature = "security")]
            security_analysis: None,
            metadata,
        }
    }

    /// Transform ArchitecturalIssues into Findings
    fn transform_issues_to_findings(
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> Vec<Finding> {
        issues
            .iter()
            .map(|issue| {
                let anti_pattern = anti_pattern_types.get(&issue.anti_pattern_type_id);

                Finding {
                    id: format!("issue-{}", Uuid::new_v4()),
                    finding_type: anti_pattern
                        .map(|apt| apt.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    severity: issue.severity.to_lowercase(), // Ensure lowercase
                    title: Self::generate_title_from_issue(issue, anti_pattern),
                    message: issue.description.clone(),
                    file: issue.file_path.clone(),
                    start_line: issue.start_line.map(|l| l as u32),
                    end_line: issue.end_line.map(|l| l as u32),
                    column: issue.column_number.map(|c| c as u32),
                    code_snippet: issue.code_snippet.clone(),
                    tags: Self::generate_tags(issue, anti_pattern),
                    detector: issue.detector_name.clone(),
                    confidence: Self::calculate_confidence(issue),
                    ai_explanation: issue.ai_explanation.clone(),
                    recommendation: Self::generate_recommendation(issue, anti_pattern),
                    related_findings: vec![],
                    #[cfg(feature = "security")]
                    security_metadata: None,
                }
            })
            .collect::<Vec<Finding>>() // Explicitly materialize transformed findings list
    }

    /// Calculate summary statistics
    fn calculate_summary(analysis_run: &AnalysisRun, findings: &[Finding]) -> AnalysisSummary {
        let mut issues_by_severity = HashMap::new();
        let mut issues_by_category = HashMap::new();

        // Count by severity
        for finding in findings {
            *issues_by_severity
                .entry(finding.severity.clone())
                .or_insert(0) += 1;
        }

        // Ensure all severity levels are present
        for severity in &["critical", "high", "medium", "low"] {
            issues_by_severity.entry(severity.to_string()).or_insert(0);
        }

        // Count by category (derived from tags)
        for finding in findings {
            for tag in &finding.tags {
                if tag == "anti-pattern" || tag == "code-quality" || tag == "security" {
                    *issues_by_category.entry(tag.clone()).or_insert(0) += 1;
                }
            }
        }

        AnalysisSummary {
            coverage: Self::calculate_coverage(analysis_run, findings),
            issues_total: findings.len() as u32,
            issues_by_severity,
            issues_by_category,
            files_analyzed: analysis_run.total_files_analyzed.unwrap_or(0) as u32,
            components_analyzed: 0, // TODO: Track components
            analysis_duration_ms: analysis_run
                .end_time
                .map(|end| (end - analysis_run.start_time).num_milliseconds() as u64)
                .unwrap_or(0),
            time_generated: Utc::now(),
        }
    }

    /// Generate a title from an issue
    fn generate_title_from_issue(
        issue: &ArchitecturalIssue,
        anti_pattern: Option<&AntiPatternType>,
    ) -> String {
        if let Some(apt) = anti_pattern {
            format!("{} detected", apt.name)
        } else {
            issue.message.chars().take(100).collect::<String>() // Explicitly collect first 100 chars into String
        }
    }

    /// Generate tags for a finding
    fn generate_tags(
        issue: &ArchitecturalIssue,
        anti_pattern: Option<&AntiPatternType>,
    ) -> Vec<String> {
        let mut tags = vec![];

        if let Some(apt) = anti_pattern {
            match apt.category.as_str() {
                "structural" => tags.push("anti-pattern".to_string()),
                "behavioral" => tags.push("code-quality".to_string()),
                "security" => tags.push("security".to_string()),
                _ => tags.push("misc".to_string()),
            }
        }

        // Add severity as tag
        tags.push(issue.severity.to_lowercase());

        // Add detector-specific tags
        if issue.detector_name.contains("DeadCode") {
            tags.push("cleanup".to_string());
        }
        if issue.detector_name.contains("Coupling") {
            tags.push("testability".to_string());
        }
        if issue.detector_name.contains("Long") || issue.detector_name.contains("Large") {
            tags.push("complexity".to_string());
        }

        tags
    }

    /// Calculate confidence score (0.0 - 1.0)
    fn calculate_confidence(_issue: &ArchitecturalIssue) -> f64 {
        // TODO: Extract from metadata or calculate based on detector
        0.85
    }

    /// Generate recommendation for an issue
    fn generate_recommendation(
        issue: &ArchitecturalIssue,
        anti_pattern: Option<&AntiPatternType>,
    ) -> Option<String> {
        if let Some(apt) = anti_pattern {
            match apt.name.as_str() {
                "God Object" => Some("Consider breaking this class into smaller, more focused components following the Single Responsibility Principle.".to_string()),
                "Dead Code" => Some("Remove unused code to improve maintainability and reduce complexity.".to_string()),
                "Tight Coupling" => Some("Use dependency injection or interfaces to reduce coupling between components.".to_string()),
                "Long Methods" => Some("Extract smaller methods with single responsibilities to improve readability and testability.".to_string()),
                "Magic Values" => Some("Replace magic values with named constants or configuration values.".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Calculate code quality coverage score
    fn calculate_coverage(analysis_run: &AnalysisRun, findings: &[Finding]) -> f64 {
        let files = analysis_run.total_files_analyzed.unwrap_or(0) as f64;
        let issues = findings.len() as f64;

        if files <= 0.0 {
            return 0.0;
        }

        // Simple formula: 100 - (issues per file * 10), clamped to 0-100
        let issues_per_file = issues / files;
        let score = 100.0 - (issues_per_file * 10.0);
        score.max(0.0).min(100.0)
    }

    /// Detect languages from file extensions
    fn detect_languages(issues: &[ArchitecturalIssue]) -> Vec<String> {
        let mut languages = std::collections::HashSet::new();

        for issue in issues {
            if issue.file_path.ends_with(".rs") {
                languages.insert("rust");
            } else if issue.file_path.ends_with(".py") {
                languages.insert("python");
            } else if issue.file_path.ends_with(".js") {
                languages.insert("javascript");
            } else if issue.file_path.ends_with(".ts") || issue.file_path.ends_with(".tsx") {
                languages.insert("typescript");
            }
        }

        languages.into_iter().map(|s| s.to_string()).collect::<Vec<String>>() // Explicitly collect language identifiers
    }
}
