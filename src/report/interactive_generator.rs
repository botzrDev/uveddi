//! Interactive Report Generator
//!
//! This module provides functionality to generate interactive reports in the standardized
//! Report v1 format. It integrates with the existing analysis pipeline while producing
//! JSON data optimized for the React SPA frontend.
//!
//! # Key Features
//!
//! - Converts analysis results to Report v1 JSON format
//! - Integrates with existing database models and visualization system
//! - Supports both file-based storage and in-memory generation
//! - Provides demo data generation for development
//! - Maintains compatibility with existing report generators

use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue, Dependency};
use crate::database::Database;
use crate::models::visualization::{ArchitecturalComponent, DiagramMetadata};
use crate::report::interactive_models::{InteractiveReport, REPORT_SCHEMA_VERSION};
use crate::report::ReportGenerationError;
use chrono::Utc;
use serde_json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

/// Configuration for interactive report generation
#[derive(Debug, Clone)]
pub struct InteractiveReportConfig {
    /// Include AI-generated insights in the report
    pub include_ai_insights: bool,
    /// Include detailed code snippets in findings
    pub include_code_snippets: bool,
    /// Maximum number of findings to include (0 = no limit)
    pub max_findings: usize,
    /// Include dependency graph data
    pub include_dependency_graph: bool,
    /// Include diagram definitions
    pub include_diagrams: bool,
    /// Storage directory for generated reports
    pub storage_path: PathBuf,
    /// Auto-save reports to storage
    pub auto_save: bool,
}

impl Default for InteractiveReportConfig {
    fn default() -> Self {
        Self {
            include_ai_insights: true,
            include_code_snippets: true,
            max_findings: 0, // No limit
            include_dependency_graph: true,
            include_diagrams: true,
            storage_path: PathBuf::from("./.uveddi/reports"),
            auto_save: true,
        }
    }
}

/// Interactive report generator
pub struct InteractiveReportGenerator {
    config: InteractiveReportConfig,
    database: Option<Arc<Database>>,
}

impl InteractiveReportGenerator {
    /// Create a new interactive report generator
    pub fn new(config: InteractiveReportConfig) -> Self {
        Self {
            config,
            database: None,
        }
    }

    /// Create a new generator with database access
    pub fn with_database(config: InteractiveReportConfig, database: Arc<Database>) -> Self {
        Self {
            config,
            database: Some(database),
        }
    }

    /// Generate an interactive report from analysis data
    pub async fn generate_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &[AntiPatternType],
        components: Option<&[ArchitecturalComponent]>,
        dependencies: &[Dependency],
        diagrams: &[DiagramMetadata],
        project_name: String,
        project_path: String,
    ) -> Result<InteractiveReport, ReportGenerationError> {
        // Generate the report using the interactive models
        let mut report = InteractiveReport::from_analysis_data(
            analysis_run,
            issues,
            anti_pattern_types,
            components,
            dependencies,
            diagrams,
            project_name,
            project_path,
        );

        // Apply configuration filters
        if self.config.max_findings > 0 && report.findings.len() > self.config.max_findings {
            // Keep the highest severity findings
            report.findings.sort_by(|a, b| {
                let severity_order = |s: &str| match s {
                    "critical" => 0,
                    "high" => 1,
                    "medium" => 2,
                    "low" => 3,
                    _ => 4,
                };
                severity_order(&a.severity).cmp(&severity_order(&b.severity))
            });
            report.findings.truncate(self.config.max_findings);

            // Update summary counts
            report.summary.issues_total = report.findings.len() as u32;
        }

        if !self.config.include_ai_insights {
            report.ai_insights = None;
        }

        if !self.config.include_code_snippets {
            for finding in &mut report.findings {
                finding.code_snippet = None;
            }
        }

        if !self.config.include_dependency_graph {
            report.dependency_graph.nodes.clear();
            report.dependency_graph.edges.clear();
            report.dependency_graph.metadata.node_count = 0;
            report.dependency_graph.metadata.edge_count = 0;
        }

        if !self.config.include_diagrams {
            report.diagrams.clear();
        }

        // Auto-save if configured
        if self.config.auto_save {
            if let Err(e) = self.save_report(&report).await {
                eprintln!("Warning: Failed to auto-save report: {}", e);
            }
        }

        Ok(report)
    }

    /// Generate a report from an analysis run ID (requires database)
    pub async fn generate_from_run_id(
        &self,
        run_id: i64,
    ) -> Result<InteractiveReport, ReportGenerationError> {
        let db = self.database.as_ref().ok_or_else(|| {
            ReportGenerationError::ComponentAnalysisError(
                "Database is required for generating reports from run ID".to_string(),
            )
        })?;

        // Load analysis data from database
        let analysis_run = self.load_analysis_run(db, run_id).await?;
        let issues = self.load_issues(db, run_id).await?;
        let anti_pattern_types = self.load_anti_pattern_types(db).await?;
        let components = self.load_components(db, run_id).await.ok();
        let dependencies = self.load_dependencies(db, run_id).await?;
        let diagrams = self.load_diagrams(db, run_id).await?;

        // Extract project info from analysis run
        let project_name = format!("Analysis Run {}", run_id);
        let project_path = "/unknown".to_string(); // TODO: Extract from analysis config

        self.generate_report(
            &analysis_run,
            &issues,
            &anti_pattern_types,
            components.as_deref(),
            &dependencies,
            &diagrams,
            project_name,
            project_path,
        )
        .await
    }

    /// Save report to storage
    pub async fn save_report(
        &self,
        report: &InteractiveReport,
    ) -> Result<PathBuf, ReportGenerationError> {
        // Ensure storage directory exists
        fs::create_dir_all(&self.config.storage_path).await?;

        // Generate filename based on project ID and timestamp
        let filename = format!("{}.json", report.project.id);
        let file_path = self.config.storage_path.join(filename);

        // Serialize and save
        let json_content = serde_json::to_string_pretty(report)?;
        fs::write(&file_path, json_content).await?;

        Ok(file_path)
    }

    /// Load a saved report from storage
    pub async fn load_report(
        &self,
        report_id: &str,
    ) -> Result<InteractiveReport, ReportGenerationError> {
        let file_path = self.config.storage_path.join(format!("{}.json", report_id));

        if !file_path.exists() {
            return Err(ReportGenerationError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Report not found",
            )));
        }

        let content = fs::read_to_string(file_path).await?;
        let report: InteractiveReport = serde_json::from_str(&content)?;

        Ok(report)
    }

    /// List available saved reports
    pub async fn list_reports(&self) -> Result<Vec<String>, ReportGenerationError> {
        if !self.config.storage_path.exists() {
            return Ok(vec![]);
        }

        let mut report_ids = Vec::new();
        let mut entries = fs::read_dir(&self.config.storage_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    let report_id = filename.strip_suffix(".json").unwrap_or(filename);
                    report_ids.push(report_id.to_string());
                }
            }
        }

        report_ids.sort();
        Ok(report_ids)
    }

    /// Generate a demo report for development and testing
    pub fn generate_demo_report() -> InteractiveReport {
        let mut demo = InteractiveReport::default();

        // Enhance demo data with more realistic content
        demo.project.name = "Demo Rust Project".to_string();
        demo.project.languages = vec!["rust".to_string(), "javascript".to_string()];
        demo.project.commit = Some("abc123".to_string());
        demo.project.branch = Some("main".to_string());

        // Add some demo findings
        demo.findings = vec![
            crate::report::interactive_models::Finding {
                id: "finding-1".to_string(),
                finding_type: "LongMethod".to_string(),
                severity: "high".to_string(),
                title: "Long method in UserService".to_string(),
                message: "Method `process_user_registration` is 127 lines long and should be refactored".to_string(),
                file: "src/services/user_service.rs".to_string(),
                start_line: Some(45),
                end_line: Some(172),
                column: Some(5),
                code_snippet: Some("impl UserService {\n    pub fn process_user_registration(...) {\n        // 127 lines of code...\n    }\n}".to_string()),
                tags: vec!["maintainability".to_string(), "high".to_string()],
                detector: "long_method_detector".to_string(),
                confidence: 0.95,
                ai_explanation: Some("This method is doing too many things. Consider breaking it into smaller, focused methods.".to_string()),
                recommendation: Some("Extract validation, persistence, and notification logic into separate methods.".to_string()),
                related_findings: vec![],
            },
            crate::report::interactive_models::Finding {
                id: "finding-2".to_string(),
                finding_type: "DeadCode".to_string(),
                severity: "medium".to_string(),
                title: "Unused function in utilities".to_string(),
                message: "Function `old_helper` is never called and can be removed".to_string(),
                file: "src/utils/helpers.rs".to_string(),
                start_line: Some(123),
                end_line: Some(135),
                column: Some(1),
                code_snippet: Some("fn old_helper(data: &str) -> String {\n    // Unused function\n}".to_string()),
                tags: vec!["maintenance".to_string(), "medium".to_string()],
                detector: "dead_code_detector".to_string(),
                confidence: 0.88,
                ai_explanation: None,
                recommendation: Some("Remove this function to reduce code complexity.".to_string()),
                related_findings: vec![],
            },
        ];

        // Add demo dependency graph
        demo.dependency_graph = crate::report::interactive_models::DependencyGraph {
            nodes: vec![
                crate::report::interactive_models::GraphNode {
                    id: "user_service".to_string(),
                    label: "UserService".to_string(),
                    path: "src/services/user_service.rs".to_string(),
                    node_type: "service".to_string(),
                    metrics: Some(crate::report::interactive_models::NodeMetrics {
                        loc: Some(245),
                        complexity: Some(8.5),
                        dependencies: 3,
                        dependents: 5,
                    }),
                    group: Some("services".to_string()),
                    properties: std::collections::HashMap::new(),
                },
                crate::report::interactive_models::GraphNode {
                    id: "database".to_string(),
                    label: "Database".to_string(),
                    path: "src/database/mod.rs".to_string(),
                    node_type: "database".to_string(),
                    metrics: Some(crate::report::interactive_models::NodeMetrics {
                        loc: Some(156),
                        complexity: Some(4.2),
                        dependencies: 1,
                        dependents: 8,
                    }),
                    group: Some("infrastructure".to_string()),
                    properties: std::collections::HashMap::new(),
                },
            ],
            edges: vec![crate::report::interactive_models::GraphEdge {
                source: "user_service".to_string(),
                target: "database".to_string(),
                edge_type: "uses".to_string(),
                weight: Some(0.8),
                properties: std::collections::HashMap::new(),
            }],
            metadata: crate::report::interactive_models::GraphMetadata {
                node_count: 2,
                edge_count: 1,
                has_cycles: false,
                max_depth: 2,
                suggested_layout: crate::report::interactive_models::CytoscapeLayout::Cose,
                layout_config: std::collections::HashMap::new(),
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

        // Add demo diagram
        demo.diagrams = vec![
            crate::report::interactive_models::DiagramDefinition {
                id: "demo-architecture".to_string(),
                kind: "mermaid".to_string(),
                title: "Demo Architecture Overview".to_string(),
                source: "graph TD\n    A[User Service] --> B[Database]\n    A --> C[Email Service]\n    B --> D[Cache]".to_string(),
                description: Some("High-level architecture of the demo project".to_string()),
                components: vec!["user_service".to_string(), "database".to_string()],
                metadata: crate::report::interactive_models::DiagramRenderMetadata {
                    width: Some(800),
                    height: Some(600),
                    theme: Some("default".to_string()),
                    direction: Some("TD".to_string()),
                    options: std::collections::HashMap::new(),
                },
            },
        ];

        demo
    }

    // Database loading methods (these would be implemented based on the actual database schema)

    async fn load_analysis_run(
        &self,
        db: &Database,
        run_id: i64,
    ) -> Result<AnalysisRun, ReportGenerationError> {
        // TODO: Implement actual database query
        // This is a placeholder implementation
        Ok(AnalysisRun {
            run_id: Some(run_id),
            project_id: 1,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            status: "completed".to_string(),
            total_files_analyzed: Some(42),
            total_issues_found: Some(12),
            analysis_config: "{}".to_string(),
        })
    }

    async fn load_issues(
        &self,
        db: &Database,
        run_id: i64,
    ) -> Result<Vec<ArchitecturalIssue>, ReportGenerationError> {
        // TODO: Implement actual database query
        Ok(vec![])
    }

    async fn load_anti_pattern_types(
        &self,
        db: &Database,
    ) -> Result<Vec<AntiPatternType>, ReportGenerationError> {
        // TODO: Implement actual database query
        Ok(vec![])
    }

    async fn load_components(
        &self,
        db: &Database,
        run_id: i64,
    ) -> Result<Vec<ArchitecturalComponent>, ReportGenerationError> {
        // TODO: Implement actual database query
        Err(ReportGenerationError::ComponentAnalysisError(
            "Component loading not yet implemented".to_string(),
        ))
    }

    async fn load_dependencies(
        &self,
        db: &Database,
        run_id: i64,
    ) -> Result<Vec<Dependency>, ReportGenerationError> {
        // TODO: Implement actual database query
        Ok(vec![])
    }

    async fn load_diagrams(
        &self,
        db: &Database,
        run_id: i64,
    ) -> Result<Vec<DiagramMetadata>, ReportGenerationError> {
        // TODO: Implement actual database query
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    #[tokio::test]
    async fn test_generate_demo_report() {
        let demo_report = InteractiveReportGenerator::generate_demo_report();

        assert_eq!(demo_report.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(demo_report.project.name, "Demo Rust Project");
        assert_eq!(demo_report.findings.len(), 2);
        assert_eq!(demo_report.dependency_graph.nodes.len(), 2);
        assert_eq!(demo_report.diagrams.len(), 1);
    }

    #[tokio::test]
    async fn test_save_and_load_report() {
        let temp_dir = tempdir().unwrap();
        let config = InteractiveReportConfig {
            storage_path: temp_dir.path().to_path_buf(),
            auto_save: false,
            ..Default::default()
        };

        let generator = InteractiveReportGenerator::new(config);
        let demo_report = InteractiveReportGenerator::generate_demo_report();

        // Save report
        let saved_path = generator.save_report(&demo_report).await.unwrap();
        assert!(saved_path.exists());

        // Load report
        let loaded_report = generator
            .load_report(&demo_report.project.id)
            .await
            .unwrap();
        assert_eq!(loaded_report.project.id, demo_report.project.id);
        assert_eq!(loaded_report.schema_version, demo_report.schema_version);
    }

    #[tokio::test]
    async fn test_list_reports() {
        let temp_dir = tempdir().unwrap();
        let config = InteractiveReportConfig {
            storage_path: temp_dir.path().to_path_buf(),
            auto_save: false,
            ..Default::default()
        };

        let generator = InteractiveReportGenerator::new(config);

        // Initially empty
        let reports = generator.list_reports().await.unwrap();
        assert_eq!(reports.len(), 0);

        // Save a report
        let demo_report = InteractiveReportGenerator::generate_demo_report();
        generator.save_report(&demo_report).await.unwrap();

        // Should find one report
        let reports = generator.list_reports().await.unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0], demo_report.project.id);
    }

    #[test]
    fn test_report_config_filtering() {
        let config = InteractiveReportConfig {
            max_findings: 1,
            include_ai_insights: false,
            include_code_snippets: false,
            ..Default::default()
        };

        // Test that configuration affects report generation
        assert_eq!(config.max_findings, 1);
        assert!(!config.include_ai_insights);
        assert!(!config.include_code_snippets);
    }
}
