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

use crate::analysis::detectors::anti_patterns::dead_code::DeadCodeConfig;
use crate::analysis::detectors::anti_patterns::large_classes::LargeClassConfig;
use crate::analysis::AnalysisEngine;
use crate::database::crud::Database;
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::error::UveddiError;
use crate::report::ReportGenerator;

#[cfg(feature = "memory-optimization")]
use crate::analysis::memory::MemoryOptimizationConfig;

/// Application layer orchestrator for analysis workflows.
///
/// This struct coordinates the analysis process by managing dependencies
/// and orchestrating the workflow between different system components, such as
/// the `AnalysisEngine` and `Database`. It serves as the primary boundary between the
/// command-line interface (CLI) and the core infrastructure layers of the application.
pub struct AnalysisOrchestrator {
    /// The database connection for storing and retrieving analysis results.
    database: Database,
    /// The core analysis engine that performs code parsing and issue detection.
    analysis_engine: AnalysisEngine,
}

/// Configuration for an analysis operation.
///
/// This struct holds all the settings required to perform a codebase analysis,
/// including target paths, output formats, and detector-specific configurations.
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
    /// Confidence threshold for dead code detection (0.0 to 1.0).
    pub dead_code_confidence: Option<f64>,
    /// Enable library mode for dead code detection.
    pub dead_code_library_mode: bool,
    /// Patterns to ignore during dead code detection.
    pub dead_code_ignore_patterns: Option<Vec<String>>,
    /// Symbols to always keep alive during dead code detection.
    pub dead_code_keep_alive: Option<Vec<String>>,
    /// Maximum logical lines of code threshold for large classes.
    pub large_classes_max_loc: Option<u32>,
    /// Maximum number of methods threshold for large classes.
    pub large_classes_max_methods: Option<u32>,
    /// Maximum number of fields threshold for large classes.
    pub large_classes_max_fields: Option<u32>,
    /// Maximum cyclomatic complexity threshold for large classes.
    pub large_classes_max_complexity: Option<u32>,
    /// Maximum LCOM score threshold for large classes.
    pub large_classes_max_lcom: Option<f64>,
    /// Patterns to ignore during large classes detection.
    pub large_classes_ignore_patterns: Option<Vec<String>>,
    /// Minimum severity score for large classes reporting.
    pub large_classes_min_severity: Option<u32>,
    
    /// Memory optimization configuration
    #[cfg(feature = "memory-optimization")]
    pub memory_optimization: Option<crate::analysis::memory::MemoryOptimizationConfig>,
    
    /// Enable memory optimization features
    pub enable_memory_optimization: bool,
    
    /// Memory limit in gigabytes
    pub memory_limit_gb: Option<f64>,
    
    /// Memory profile selection (small/default/large)
    pub memory_profile: Option<String>,
}

/// Represents the result of a completed analysis operation.
///
/// This struct contains the generated report content along with metadata
/// about the analysis process, such as the number of files analyzed and
/// the time taken.
pub struct AnalysisReport {
    /// The generated analysis report as a string.
    pub content: String,
    /// Metadata about the analysis operation.
    pub metadata: AnalysisMetadata,
}

/// Contains metadata about a completed analysis operation.
///
/// This provides key metrics about the analysis run, such as performance
/// statistics and the scope of the analysis.
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
        
        // Initialize memory optimization with default configuration
        #[cfg(feature = "memory-optimization")]
        {
            let memory_config = crate::analysis::memory::MemoryOptimizationConfig::default();
            if let Err(e) = crate::analysis::memory::initialize_memory_optimization(memory_config) {
                log::warn!("Failed to initialize memory optimization: {}", e);
            } else {
                log::info!("Memory optimization initialized successfully");
            }
        }
        
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
        
        // Initialize memory optimization with default configuration
        #[cfg(feature = "memory-optimization")]
        {
            let memory_config = crate::analysis::memory::MemoryOptimizationConfig::default();
            if let Err(e) = crate::analysis::memory::initialize_memory_optimization(memory_config) {
                log::warn!("Failed to initialize memory optimization: {}", e);
            } else {
                log::info!("Memory optimization initialized successfully");
            }
        }
        
        let analysis_engine =
            AnalysisEngine::new().context("Failed to initialize analysis engine")?;

        Ok(Self {
            database,
            analysis_engine,
        })
    }

    /// Creates a new `AnalysisOrchestrator` with custom memory optimization configuration.
    pub fn with_memory_config(
        db_path: Option<&std::path::Path>,
        memory_config: Option<crate::analysis::memory::MemoryOptimizationConfig>,
    ) -> Result<Self, UveddiError> {
        let database = Database::new(db_path).context("Failed to initialize database")?;
        
        // Initialize memory optimization with custom configuration
        #[cfg(feature = "memory-optimization")]
        if let Some(config) = memory_config {
            if let Err(e) = crate::analysis::memory::initialize_memory_optimization(config) {
                log::warn!("Failed to initialize memory optimization: {}", e);
            } else {
                log::info!("Memory optimization initialized with custom configuration");
            }
        }
        
        let analysis_engine =
            AnalysisEngine::new().context("Failed to initialize analysis engine")?;

        Ok(Self {
            database,
            analysis_engine,
        })
    }

    /// Executes a complete analysis workflow based on the provided configuration.
    ///
    /// This is the main entry point for running an analysis. The process includes:
    /// 1. Validating the input configuration and paths.
    /// 2. Initializing the database schema.
    /// 3. Creating a new analysis run record in the database.
    /// 4. Invoking the `AnalysisEngine` to perform code analysis.
    /// 5. Storing the detected issues in the database.
    /// 6. Generating a report in the specified format.
    /// 7. Writing the report to a file if requested.
    ///
    /// # Arguments
    ///
    /// * `config` - An `AnalysisConfig` struct containing all settings for the run.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `AnalysisReport` on success, or an `UveddiError` on failure.
    pub async fn execute_analysis(
        &mut self,
        config: AnalysisConfig,
    ) -> Result<AnalysisReport, UveddiError> {
        log::debug!("🚀 Starting execute_analysis for path: {}", config.target_path.display());
        
        // Recreate analysis engine with memory optimization if enabled
        if config.enable_memory_optimization {
            log::debug!("🔧 Memory optimization enabled, creating optimized analysis engine");
            self.analysis_engine = Self::create_analysis_engine_with_memory_optimization(&config)?;
            log::debug!("✅ Memory-optimized analysis engine created successfully");
        }
        let start_time = std::time::Instant::now();
        log::debug!("⏱️  Analysis timer started");

        // Validate input path
        log::debug!("🔍 Validating input path: {}", config.target_path.display());
        if !config.target_path.exists() {
            log::error!("❌ Path does not exist: {}", config.target_path.display());
            return Err(UveddiError::PathError {
                path: config.target_path.display().to_string(),
                reason: "Path does not exist".to_string(),
                suggestion: "Verify the path exists and is accessible".to_string(),
            })
                .context("Input path validation failed")?;
        }
        log::debug!("✅ Input path validated successfully");

        info!("Starting analysis of: {}", config.target_path.display());

        // Configure dead code detector if settings provided
        log::debug!("🔧 Configuring dead code detector");
        self.configure_dead_code_detector(&config)?;
        log::debug!("✅ Dead code detector configured");

        // Configure large classes detector if settings provided
        log::debug!("🔧 Configuring large classes detector");
        self.configure_large_classes_detector(&config)?;
        log::debug!("✅ Large classes detector configured");

        // Initialize database schema
        log::debug!("💾 Initializing database schema");
        self.initialize_database_schema().await?;
        log::debug!("✅ Database schema initialized");

        // Create analysis run record
        log::debug!("📝 Creating analysis run record");
        let mut analysis_run = self
            .database
            .create_analysis_run(&config.target_path)
            .context("Failed to create analysis run")?;
        log::debug!("✅ Analysis run record created with ID: {:?}", analysis_run.run_id);

        // Execute core analysis
        log::debug!("🔍 Starting core analysis execution");
        let (mut issues, _dependency_graph) = self
            .analysis_engine
            .analyze(&config.target_path)
            .await
            .context("Analysis failed")?;
        log::debug!("✅ Core analysis completed - found {} issues", issues.len());

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
        let report_content =
            self.generate_report(&config, &analysis_run, &issues, &report_generator)?;

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
                    .map_err(|e| {
                        crate::error::UveddiError::from(
                            crate::report::errors::ReportGenerationError::DataExtractionError(
                                e.to_string(),
                            )
                        )
                    })?;
                Ok(report.to_string())
            }
            "markdown" => report_generator
                .generate_markdown_report(analysis_run, issues, &HashMap::new(), None)
                .map_err(|e| {
                    crate::error::UveddiError::from(
                        crate::report::errors::ReportGenerationError::DataExtractionError(
                            e.to_string(),
                        )
                    )
                }),
            _ => Err(UveddiError::config_error(
                &format!("Unsupported output format specified: {}", config.output_format),
                "output format",
            ))
            .context("Unsupported output format specified")?,
        }
    }

    /// Configure the dead code detector based on analysis config
    fn configure_dead_code_detector(&mut self, config: &AnalysisConfig) -> Result<(), UveddiError> {
        // Check if any dead code configuration is provided
        if config.dead_code_confidence.is_some()
            || config.dead_code_library_mode
            || config.dead_code_ignore_patterns.is_some()
            || config.dead_code_keep_alive.is_some()
        {
            let mut dead_code_config = DeadCodeConfig::default();

            if let Some(confidence) = config.dead_code_confidence {
                if confidence < 0.0 || confidence > 1.0 {
                    return Err(UveddiError::config_error(
                        "Dead code confidence threshold must be between 0.0 and 1.0",
                        "config validation",
                    ));
                }
                dead_code_config.min_confidence = confidence;
            }

            dead_code_config.library_mode = config.dead_code_library_mode;

            if let Some(ref patterns) = config.dead_code_ignore_patterns {
                dead_code_config.ignore_patterns = patterns.clone();
            }

            if let Some(ref patterns) = config.dead_code_keep_alive {
                dead_code_config.keep_alive_patterns = patterns.clone();
            }

            self.analysis_engine
                .configure_dead_code_detector(dead_code_config);
            info!("Dead code detector configured with custom settings");
        }

        Ok(())
    }

    /// Configure the large classes detector based on analysis config
    fn configure_large_classes_detector(
        &mut self,
        config: &AnalysisConfig,
    ) -> Result<(), UveddiError> {
        // Check if any large classes configuration is provided
        if config.large_classes_max_loc.is_some()
            || config.large_classes_max_methods.is_some()
            || config.large_classes_max_fields.is_some()
            || config.large_classes_max_complexity.is_some()
            || config.large_classes_max_lcom.is_some()
            || config.large_classes_ignore_patterns.is_some()
            || config.large_classes_min_severity.is_some()
        {
            let mut large_classes_config = LargeClassConfig::default();

            // Apply custom thresholds if provided
            if let Some(max_loc) = config.large_classes_max_loc {
                // Update all language thresholds with the custom value
                large_classes_config.rust_thresholds.max_logical_loc = max_loc;
                large_classes_config.python_thresholds.max_logical_loc = max_loc;
                large_classes_config.javascript_thresholds.max_logical_loc = max_loc;
            }

            if let Some(max_methods) = config.large_classes_max_methods {
                large_classes_config.rust_thresholds.max_methods = max_methods;
                large_classes_config.python_thresholds.max_methods = max_methods;
                large_classes_config.javascript_thresholds.max_methods = max_methods;
            }

            if let Some(max_fields) = config.large_classes_max_fields {
                large_classes_config.rust_thresholds.max_fields = max_fields;
                large_classes_config.python_thresholds.max_fields = max_fields;
                large_classes_config.javascript_thresholds.max_fields = max_fields;
            }

            if let Some(max_complexity) = config.large_classes_max_complexity {
                large_classes_config
                    .rust_thresholds
                    .max_cyclomatic_complexity = max_complexity;
                large_classes_config
                    .python_thresholds
                    .max_cyclomatic_complexity = max_complexity;
                large_classes_config
                    .javascript_thresholds
                    .max_cyclomatic_complexity = max_complexity;
                large_classes_config
                    .rust_thresholds
                    .max_cognitive_complexity = max_complexity;
                large_classes_config
                    .python_thresholds
                    .max_cognitive_complexity = max_complexity;
                large_classes_config
                    .javascript_thresholds
                    .max_cognitive_complexity = max_complexity;
            }

            if let Some(max_lcom) = config.large_classes_max_lcom {
                if max_lcom < 0.0 || max_lcom > 1.0 {
                    return Err(UveddiError::config_error(
                        "Large classes LCOM score must be between 0.0 and 1.0",
                        "config validation",
                    ));
                }
                large_classes_config.rust_thresholds.max_lcom_score = max_lcom;
                large_classes_config.python_thresholds.max_lcom_score = max_lcom;
                large_classes_config.javascript_thresholds.max_lcom_score = max_lcom;
            }

            // TODO: Add ignore_patterns and min_severity_score fields to LargeClassConfig
            // if let Some(ref patterns) = config.large_classes_ignore_patterns {
            //     large_classes_config.ignore_patterns = patterns.clone();
            // }

            // if let Some(min_severity) = config.large_classes_min_severity {
            //     if min_severity > 100 {
            //         return Err(UveddiError::Configuration(
            //             "Large classes minimum severity score must be between 0 and 100".to_string()
            //         ));
            //     }
            //     large_classes_config.min_severity_score = min_severity;
            // }

            self.analysis_engine
                .configure_large_classes_detector(large_classes_config);
            info!("Large classes detector configured with custom settings");
        }

        Ok(())
    }

    /// Create analysis engine with memory optimization support
    fn create_analysis_engine_with_memory_optimization(
        config: &AnalysisConfig,
    ) -> Result<AnalysisEngine, UveddiError> {
        if config.enable_memory_optimization {
            #[cfg(feature = "memory-optimization")]
            {
                // Validate memory optimization configuration
                if let Err(validation_error) = Self::validate_memory_optimization_config(config) {
                    log::warn!("Memory optimization configuration validation failed: {}", validation_error);
                    log::warn!("Falling back to standard analysis mode");
                    return AnalysisEngine::new().context("Failed to initialize analysis engine")
                        .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"));
                }

                if let Some(memory_config) = config.memory_optimization.clone() {
                    // Validate provided memory config
                    if let Err(validation_error) = memory_config.validate() {
                        log::warn!("Invalid memory optimization configuration: {}", validation_error);
                        log::warn!("Falling back to standard analysis mode");
                        return AnalysisEngine::new().context("Failed to initialize analysis engine")
                            .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"));
                    }

                    AnalysisEngine::with_memory_optimization(Some(memory_config), None)
                        .context("Failed to initialize analysis engine with memory optimization")
                        .map_err(|e| {
                            log::warn!("Memory optimization initialization failed: {}", e);
                            log::warn!("Falling back to standard analysis mode");
                            // Graceful fallback to standard mode
                            AnalysisEngine::new().context("Failed to initialize analysis engine")
                                .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
                        })
                        .or_else(|_| {
                            AnalysisEngine::new().context("Failed to initialize analysis engine")
                                .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
                        })
                } else {
                    // Create memory optimization config based on profile and limits
                    let mut memory_config = match config.memory_profile.as_deref() {
                        Some("small") => MemoryOptimizationConfig::small_project(),
                        Some("large") => MemoryOptimizationConfig::large_codebase(),
                        _ => MemoryOptimizationConfig::default(),
                    };
                    
                    // Apply memory limit if specified
                    if let Some(limit_gb) = config.memory_limit_gb {
                        if limit_gb <= 0.0 {
                            log::warn!("Invalid memory limit: {}GB. Using default configuration.", limit_gb);
                        } else {
                            memory_config.target_max_memory_bytes = (limit_gb * 1024.0 * 1024.0 * 1024.0) as usize;
                        }
                    }
                    
                    // Validate the created config
                    if let Err(validation_error) = memory_config.validate() {
                        log::warn!("Generated memory optimization configuration is invalid: {}", validation_error);
                        log::warn!("Falling back to standard analysis mode");
                        return AnalysisEngine::new().context("Failed to initialize analysis engine")
                            .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"));
                    }

                    AnalysisEngine::with_memory_optimization(Some(memory_config), None)
                        .context("Failed to initialize analysis engine with memory optimization")
                        .map_err(|e| {
                            log::warn!("Memory optimization initialization failed: {}", e);
                            log::warn!("Falling back to standard analysis mode");
                            // Graceful fallback to standard mode
                            AnalysisEngine::new().context("Failed to initialize analysis engine")
                                .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
                        })
                        .or_else(|_| {
                            AnalysisEngine::new().context("Failed to initialize analysis engine")
                                .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
                        })
                }
            }
            #[cfg(not(feature = "memory-optimization"))]
            {
                log::warn!("Memory optimization requested but feature not enabled");
                log::warn!("Falling back to standard analysis mode");
                AnalysisEngine::new().context("Failed to initialize analysis engine")
                    .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
            }
        } else {
            AnalysisEngine::new().context("Failed to initialize analysis engine")
                .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
        }
    }

    /// Validate memory optimization configuration
    #[cfg(feature = "memory-optimization")]
    fn validate_memory_optimization_config(config: &AnalysisConfig) -> Result<(), String> {
        if let Some(limit_gb) = config.memory_limit_gb {
            if limit_gb <= 0.0 {
                return Err(format!("Memory limit must be positive, got: {}GB", limit_gb));
            }
            if limit_gb > 1000.0 {
                return Err(format!("Memory limit too high ({}GB), maximum is 1000GB", limit_gb));
            }
        }

        if let Some(profile) = config.memory_profile.as_deref() {
            if !matches!(profile, "small" | "default" | "large") {
                return Err(format!("Invalid memory profile '{}', must be one of: small, default, large", profile));
            }
        }

        Ok(())
    }

    /// Validate memory optimization configuration (fallback)
    #[cfg(not(feature = "memory-optimization"))]
    fn validate_memory_optimization_config(_config: &AnalysisConfig) -> Result<(), String> {
        Ok(())
    }
}

impl Default for AnalysisOrchestrator {
    fn default() -> Self {
        Self::new().expect("Failed to create default AnalysisOrchestrator")
    }
}

/// Runs the main application logic, parsing command-line arguments and executing the
/// appropriate commands.
///
/// This function initializes the command-line interface, parses the user's input,
/// and dispatches to the relevant handlers (e.g., `analyze`, `config`). It also
/// sets up the Tokio runtime for asynchronous operations and handles top-level
/// error reporting.
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
                .map_err(|e| {
                    UveddiError::analysis_error("unknown", 0, &e.to_string(), "analyze command")
                })
        }
        Commands::Config(command) => {
            info!("Executing config command...");
            command
                .execute()
.map_err(|e| UveddiError::config_error(&e.to_string(), "config validation"))
        }
    };
    if let Err(e) = result {
        error!("Error: {e:?}");
        return Err(e);
    }
    Ok(())
}
