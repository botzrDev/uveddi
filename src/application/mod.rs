//! Application orchestration module for Uveddi
//!
//! This module coordinates the high-level workflow for codebase analysis, integrating the
//! database, analysis engine, AI engine, and report generation. It serves as the boundary
//! between the CLI and infrastructure layers.

use anyhow::Context;
use chrono::Utc;
use log::{error, info};
use std::path::PathBuf;

use crate::ai::AiAnalysisEngine;
use crate::analysis::AnalysisEngine;
use crate::analysis::LocalDependencyGraph;
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
    database: Database,
    analysis_engine: AnalysisEngine,
    ai_engine: AiAnalysisEngine,
    report_generator: ReportGenerator,
}

/// Configuration for analysis operations
pub struct AnalysisConfig {
    pub target_path: PathBuf,
    pub output_format: String,
    pub output_file: Option<PathBuf>,
    pub enable_ai: bool,
    pub openai_api_key: Option<String>,
    pub ollama_api_url: Option<String>,
    pub ollama_model: Option<String>,
}

/// Result of an analysis operation
pub struct AnalysisReport {
    pub content: String,
    pub metadata: AnalysisMetadata,
}

/// Metadata about the analysis operation
pub struct AnalysisMetadata {
    pub files_analyzed: usize,
    pub issues_found: usize,
    pub analysis_duration: std::time::Duration,
    pub ai_enhanced: bool,
}

impl AnalysisOrchestrator {
    ///
    pub fn new() -> Result<Self, UveddiError> {
        let database = Database::new().context("Failed to initialize database")?;
        let analysis_engine =
            AnalysisEngine::new().context("Failed to initialize analysis engine")?;
        let ai_engine = AiAnalysisEngine::new();
        let report_generator = ReportGenerator::new();

        Ok(Self {
            database,
            analysis_engine,
            ai_engine,
            report_generator,
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

        // Configure AI if enabled
        self.configure_ai(&config)?;

        // Initialize database schema
        self.initialize_database_schema().await?;

        // Create analysis run record
        let mut analysis_run = self
            .database
            .create_analysis_run(&config.target_path)
            .context("Failed to create analysis run")?;

        // Execute core analysis
        let (mut issues, dependency_graph) = self
            .analysis_engine
            .analyze(&config.target_path)
            .await
            .context("Analysis failed")?;

        // Run plugin analysis
        let plugin_issues = self.run_plugin_analysis(&dependency_graph).await?;
        issues.extend(plugin_issues.into_iter());
        info!("Plugin analysis completed successfully");

        // Enhance with AI analysis if enabled
        let ai_enhanced = if config.enable_ai {
            info!("Starting AI analysis enhancement");
            self.enhance_with_ai_analysis(&mut issues).await?;
            info!("AI analysis enhancement completed");
            true
        } else {
            info!("AI analysis disabled, skipping enhancement");
            false
        };
        info!("AI analysis phase completed");

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
        let report_content = self.generate_report(&config, &analysis_run, &issues)?;

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

    /// Configure AI providers based on the provided configuration
    fn configure_ai(&mut self, config: &AnalysisConfig) -> Result<(), UveddiError> {
        if config.enable_ai {
            if let Some(_api_key) = &config.openai_api_key {
                // self.ai_engine = self.ai_engine.clone().with_openai_api(api_key.clone());
                // Placeholder: set OpenAI API key if needed
                info!("AI analysis enabled with OpenAI");
            } else {
                let ollama_api_url = config
                    .ollama_api_url
                    .clone()
                    .or_else(|| std::env::var("OLLAMA_API_URL").ok())
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                let ollama_model = config
                    .ollama_model
                    .clone()
                    .or_else(|| std::env::var("OLLAMA_MODEL").ok())
                    .unwrap_or_else(|| "deepseek-coder:6.7b-instruct-q4_0".to_string());
                // self.ai_engine = self.ai_engine.clone().with_ollama(&ollama_model, &ollama_api_url);
                // Placeholder: set Ollama model and API URL if needed
                info!(
                    "AI analysis enabled with local Ollama model: {} at {}",
                    ollama_model, ollama_api_url
                );
            }
        }
        Ok(())
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

    /// Run plugin-based analysis
    async fn run_plugin_analysis(
        &self,
        _local_graph: &LocalDependencyGraph,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        info!("Running analysis plugins...");

        // Plugin system is temporarily disabled to prevent WASM runtime errors
        // TODO: Re-enable once WASM plugin integration is stabilized
        info!("Plugin system disabled - native analysis only");
        Ok(Vec::new())
    }

    /// Enhance analysis results with AI insights
    async fn enhance_with_ai_analysis(
        &mut self,
        issues: &mut Vec<ArchitecturalIssue>,
    ) -> Result<(), UveddiError> {
        info!("Enhancing issues with AI analysis...");
        for issue in issues.iter_mut() {
            self.ai_engine
                .analyze_issue(issue)
                .await
                .context("Failed to enhance issue with AI analysis")?;
            info!("AI analysis placeholder for issue in {}", issue.file_path);
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
        analysis_run.total_files_analyzed = Some(self.analysis_engine.get_files_analyzed() as i32);
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
    ) -> Result<String, UveddiError> {
        match config.output_format.as_str() {
            "json" => {
                let report = self
                    .report_generator
                    .generate_json_report(analysis_run, issues)?;
                Ok(report.to_string())
            }
            "markdown" => self
                .report_generator
                .generate_markdown_report(analysis_run, issues),
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
    use crate::cli::{
        analyze_command::AnalyzeCommand, config_command::ConfigCommand,
        init_local_ai_command::InitLocalAiCommand, plugin_command::PluginCommand,
    };
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
        InitLocalAi(InitLocalAiCommand),
        Plugin(PluginCommand),
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
        Commands::InitLocalAi(command) => {
            info!("Executing init-local-ai command...");
            let setup = crate::cli::init_local_ai_command::OllamaSetup;
            tokio::runtime::Runtime::new()?
                .block_on(command.execute(&setup))
                .map_err(|e| UveddiError::Analysis(e.to_string()))
        }
        Commands::Plugin(command) => {
            info!("Executing plugin command...");
            tokio::runtime::Runtime::new()?
                .block_on(command.execute())
                .map_err(|e| UveddiError::Analysis(e.to_string()))
        }
        Commands::Config(command) => {
            info!("Executing config command...");
            command.execute().map_err(|e| UveddiError::Configuration(e.to_string()))
        }
    };
    if let Err(e) = result {
        error!("Error: {e:?}");
        return Err(e);
    }
    Ok(())
}
