//! Application orchestration module for Uveddi
//!
//! This module coordinates the high-level workflow for codebase analysis, integrating the
//! database, analysis engine, AI engine, and report generation. It serves as the boundary
//! between the CLI and infrastructure layers.

use anyhow::Context;
use chrono::Utc;
use log::{error, info};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::analysis::AnalysisEngine;
use crate::database::crud::Database;
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::error::UveddiError;
use crate::report::ReportGenerator;

/// Application layer orchestrator for analysis workflows
///
/// This struct coordinates the analysis process by managing dependencies
/// and orchestrating the workflow between different system components.
/// It serves as the boundary between the CLI layer and infrastructure layers.
pub struct AnalysisOrchestrator {
    /// The database connection for storing and retrieving analysis results.
    database: Database,
    /// The core analysis engine that performs code parsing and issue detection.
    analysis_engine: AnalysisEngine,
}

/// Configuration for analysis operations.
pub struct AnalysisConfig {
    /// The path to the target directory or file to be analyzed.
    pub target_path: PathBuf,
    /// The desired output format for the analysis report (e.g., "json", "markdown").
    pub output_format: String,
    /// An optional path to a file where the report should be saved.
    pub output_file: Option<PathBuf>,
    /// A flag to enable or disable AI-powered analysis.
    pub enable_ai: bool,
    /// The URL of the Ollama API endpoint, if applicable.
    pub ollama_api_url: Option<String>,
    /// The name of the Ollama model to be used for analysis.
    pub ollama_model: Option<String>,
}

/// The result of an analysis operation, containing the report content and metadata.
pub struct AnalysisReport {
    /// The generated analysis report as a string.
    pub content: String,
    /// Metadata about the analysis operation.
    pub metadata: AnalysisMetadata,
}

/// Metadata about the analysis operation.
pub struct AnalysisMetadata {
    /// The number of files that were analyzed.
    pub files_analyzed: usize,
    /// The total number of issues that were found.
    pub issues_found: usize,
    /// The duration of the analysis.
    pub analysis_duration: std::time::Duration,
    /// A flag indicating whether AI enhancement was used.
    pub ai_enhanced: bool,
}

impl AnalysisOrchestrator {
    /// Creates a new `AnalysisOrchestrator` with a database at the specified path.
    pub fn with_db_path(db_path: &std::path::Path) -> Result<Self, UveddiError> {
        let database =
            Database::new(Some(db_path)).context("Failed to initialize database with path")?;
        let analysis_engine =
            AnalysisEngine::new().context("Failed to initialize analysis engine")?;

        Ok(Self {
            database,
            analysis_engine,
        })
    }

    /// Creates a new `AnalysisOrchestrator` with an in-memory database.
    /// Ideal for testing or environments where file system access is restricted.
    pub fn new() -> Result<Self, UveddiError> {
        let database = Database::new(None).context("Failed to initialize in-memory database")?;
        let analysis_engine =
            AnalysisEngine::new().context("Failed to initialize analysis engine")?;

        Ok(Self {
            database,
            analysis_engine,
        })
    }

    /// Execute a complete analysis workflow
    pub async fn execute_analysis(
        &mut self,
        config: AnalysisConfig,
    ) -> Result<AnalysisReport, UveddiError> {
        let start_time = std::time::Instant::now();

        // Validate input path
        if !config.target_path.exists() {
            return Err(UveddiError::PathNotFound(
                config.target_path.display().to_string(),
            ))
            .context("Input path validation failed")?;
        }

        info!("Starting analysis of: {}", config.target_path.display());

        // Initialize database schema
        self.initialize_database_schema().await?;

        // Create analysis run record
        let mut analysis_run = self
            .database
            .create_analysis_run(&config.target_path)
            .context("Failed to create analysis run")?;

        // Execute core analysis
        let (mut issues, _dependency_graph) = self
            .analysis_engine
            .analyze(&config.target_path)
            .await
            .context("Analysis failed")?;

        // Plugin system removed in community version
        info!("Plugin analysis skipped (not available in community version)");

        let ai_enhanced = false;
        info!("AI analysis disabled, skipping enhancement");

        // Update analysis run record
        info!("Starting analysis run finalization");
        let analysis_duration = start_time.elapsed();
        self.finalize_analysis_run(&mut analysis_run, &issues, analysis_duration)
            .await?;
        info!("Analysis run finalization completed");

        // Store results
        info!("Starting to store {} issues to database", issues.len());

        // Set the correct analysis_run_id for all issues
        let analysis_run_id = analysis_run.run_id.expect("Analysis run should have an ID");
        for issue in &mut issues {
            issue.analysis_run_id = analysis_run_id;
        }

        self.database
            .store_issues(&issues)
            .context("Failed to store analysis issues")?;
        info!("Issues stored to database successfully");

        // Generate report
        let report_generator = ReportGenerator::new();
        let report_content = self.generate_report(&config, &analysis_run, &issues, &report_generator)?;

        // Write output file if specified
        if let Some(output_path) = &config.output_file {
            std::fs::write(output_path, &report_content).context(format!(
                "Failed to write report to {}",
                output_path.display()
            ))?;
            info!("Report written to: {}", output_path.display());
        }

        Ok(AnalysisReport {
            content: report_content,
            metadata: AnalysisMetadata {
                files_analyzed: self.analysis_engine.get_files_analyzed() as usize,
                issues_found: issues.len(),
                analysis_duration,
                ai_enhanced,
            },
        })
    }

    /// Initialize database schema with anti-pattern types
    async fn initialize_database_schema(&mut self) -> Result<(), UveddiError> {
        for mut anti_pattern_type in self.analysis_engine.get_anti_pattern_types() {
            self.database
                .store_anti_pattern_type(&mut anti_pattern_type)
                .context("Failed to store anti-pattern type")?;
        }
        Ok(())
    }

    /// Finalize the analysis run record
    async fn finalize_analysis_run(
        &mut self,
        analysis_run: &mut AnalysisRun,
        issues: &[ArchitecturalIssue],
        _duration: std::time::Duration,
    ) -> Result<(), UveddiError> {
        analysis_run.total_files_analyzed = Some(self.analysis_engine.get_files_analyzed());
        analysis_run.total_issues_found = Some(issues.len() as i32);
        analysis_run.end_time = Some(Utc::now());
        analysis_run.status = "completed".to_string();

        self.database
            .update_analysis_run(analysis_run)
            .context("Failed to update analysis run")?;
        Ok(())
    }

    /// Generate the final report
    fn generate_report(
        &self,
        config: &AnalysisConfig,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        report_generator: &ReportGenerator,
    ) -> Result<String, UveddiError> {
        match config.output_format.as_str() {
            "json" => {
                let report = report_generator
                    .generate_json_report(analysis_run, issues, &HashMap::new(), None)
                    .map_err(|e| crate::error::UveddiError::ReportGeneration(e))?;
                Ok(report.to_string())
            }
            "markdown" => report_generator
                .generate_markdown_report(analysis_run, issues, &HashMap::new(), None)
                .map_err(|e| crate::error::UveddiError::ReportGeneration(e)),
            _ => Err(UveddiError::UnsupportedOutputFormat(
                config.output_format.clone(),
            ))
            .context("Unsupported output format specified")?,
        }
    }
}

impl Default for AnalysisOrchestrator {
    fn default() -> Self {
        Self::new().expect("Failed to create default AnalysisOrchestrator")
    }
}

/// Runs the main application orchestration logic, handling CLI commands and error context.
pub fn run_app() -> Result<(), UveddiError> {
    use crate::cli::{analyze_command::AnalyzeCommand, config_command::ConfigCommand};
    use clap::Parser;
    use log::{error, info};

    #[derive(Parser)]
    #[command(name = "uveddi")]
    #[command(about = "A Rust-based code analysis and exploration tool", long_about = None)]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(clap::Subcommand)]
    enum Commands {
        Analyze(AnalyzeCommand),
        Config(ConfigCommand),
    }

    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Analyze(command) => {
            info!("Executing analyze command...");
            tokio::runtime::Runtime::new()?
                .block_on(command.execute())
                .map_err(|e| UveddiError::Analysis(e.to_string()))
        }
        Commands::Config(command) => {
            info!("Executing config command...");
            command
                .execute()
                .map_err(|e| UveddiError::Configuration(e.to_string()))
        }
    };
    if let Err(e) = result {
        error!("Error: {e:?}");
        return Err(e);
    }
    Ok(())
}
