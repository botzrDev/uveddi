//! Application orchestration module for Uveddi
//!
//! This module coordinates the high-level workflow for codebase analysis, integrating the
//! database, analysis engine, AI engine, and report generation. It serves as the boundary
//! between the CLI and infrastructure layers.

use anyhow::Context;
use chrono::Utc;
use crate::core::logging::{error, info};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::analysis::detectors::anti_patterns::dead_code::DeadCodeConfig;
use crate::analysis::detectors::anti_patterns::large_classes::LargeClassConfig;
use crate::analysis::AnalysisEngine;
use crate::database::crud::Database;
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::error::UveddiError;
use crate::report::{ReportGenerator, markdown_generator::MarkdownReportGenerator};
use crate::core::mocks::ai_mocks::AiInsight;

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

    /// Analysis timeout in seconds (0 = no timeout)
    pub timeout_seconds: u64,
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
                tracing::warn!("Failed to initialize memory optimization: {}", e);
            } else {
                tracing::info!("Memory optimization initialized successfully");
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
                tracing::warn!("Failed to initialize memory optimization: {}", e);
            } else {
                tracing::info!("Memory optimization initialized successfully");
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
                tracing::warn!("Failed to initialize memory optimization: {}", e);
            } else {
                tracing::info!("Memory optimization initialized with custom configuration");
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
        tracing::debug!(
            "🚀 Starting execute_analysis for path: {}",
            config.target_path.display()
        );

        // Recreate analysis engine with memory optimization if enabled
        if config.enable_memory_optimization {
            tracing::debug!("🔧 Memory optimization enabled, creating optimized analysis engine");
            self.analysis_engine = Self::create_analysis_engine_with_memory_optimization(&config)?;
            tracing::debug!("✅ Memory-optimized analysis engine created successfully");
        }
        let start_time = std::time::Instant::now();
        tracing::debug!("⏱️  Analysis timer started");

        // Validate input path
        tracing::debug!("🔍 Validating input path: {}", config.target_path.display());
        if !config.target_path.exists() {
            tracing::error!("❌ Path does not exist: {}", config.target_path.display());
            return Err(UveddiError::PathError {
                path: config.target_path.display().to_string(),
                reason: "Path does not exist".to_string(),
                suggestion: "Verify the path exists and is accessible".to_string(),
            })
            .context("Input path validation failed")?;
        }
        tracing::debug!("✅ Input path validated successfully");

        info!("Starting analysis of: {}", config.target_path.display());

        // Configure dead code detector if settings provided
        tracing::debug!("🔧 Configuring dead code detector");
        self.configure_dead_code_detector(&config)?;
        tracing::debug!("✅ Dead code detector configured");

        // Configure large classes detector if settings provided
        tracing::debug!("🔧 Configuring large classes detector");
        self.configure_large_classes_detector(&config)?;
        tracing::debug!("✅ Large classes detector configured");

        // Initialize database schema
        tracing::debug!("💾 Initializing database schema");
        self.initialize_database_schema().await?;
        tracing::debug!("✅ Database schema initialized");

        // Create analysis run record
        tracing::debug!("📝 Creating analysis run record");
        let mut analysis_run = self
            .database
            .create_analysis_run(&config.target_path)
            .context("Failed to create analysis run")?;
        tracing::debug!(
            "✅ Analysis run record created with ID: {:?}",
            analysis_run.run_id
        );

        // Execute core analysis
        tracing::debug!("🔍 Starting core analysis execution");
        let (mut issues, _dependency_graph) = self
            .analysis_engine
            .analyze(&config.target_path)
            .await
            .context("Analysis failed")?;
        tracing::debug!("✅ Core analysis completed - found {} issues", issues.len());

        // Plugin system removed in community version
        info!("Plugin analysis skipped (not available in community version)");

        // Generate AI insights if enabled
        let ai_insights = if config.enable_ai {
            info!("AI analysis enabled, generating insights");
            self.generate_ai_insights(&issues).await
        } else {
            info!("AI analysis disabled, skipping enhancement");
            None
        };

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
            .map_err(|e| {
                error!("Database storage failure - detailed error: {:#}", e);
                e
            })
            .context("Failed to store analysis issues")?;
        info!("Issues stored to database successfully");

        // Generate report
        let mut report_generator = ReportGenerator::new();
        let report_content =
            self.generate_report(&config, &analysis_run, &issues, &mut report_generator, ai_insights.as_deref()).await?;

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
                ai_enhanced: ai_insights.is_some(),
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
    async fn generate_report(
        &self,
        config: &AnalysisConfig,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        report_generator: &mut ReportGenerator,
        ai_insights: Option<&[AiInsight]>,
    ) -> Result<String, UveddiError> {
        // Retrieve anti-pattern types from database for proper report generation
        let anti_pattern_types = self.database.get_all_anti_pattern_types()
            .map_err(|e| {
                error!("Failed to retrieve anti-pattern types from database: {}", e);
                crate::error::UveddiError::from(
                    crate::report::errors::ReportGenerationError::DataExtractionError(
                        format!("Failed to retrieve anti-pattern types: {}", e),
                    ),
                )
            })?;

        // Build HashMap mapping anti-pattern type IDs to their definitions
        let anti_pattern_map: HashMap<i64, crate::database::models::AntiPatternType> = anti_pattern_types
            .into_iter()
            .filter_map(|apt| apt.anti_pattern_type_id.map(|id| (id, apt)))
            .collect();

        info!("Retrieved {} anti-pattern types for report generation", anti_pattern_map.len());

        match config.output_format.as_str() {
            "json" => {
                let codebase_path = config.target_path.to_str();
                let report = report_generator
                    .generate_json_report(analysis_run, issues, &anti_pattern_map, None, codebase_path)
                    .map_err(|e| {
                        crate::error::UveddiError::from(
                            crate::report::errors::ReportGenerationError::DataExtractionError(
                                e.to_string(),
                            ),
                        )
                    })?;
                Ok(report.to_string())
            }
            "markdown" => {
                let mut markdown_generator = MarkdownReportGenerator::new()
                    .map_err(|e| crate::error::UveddiError::from(
                        crate::report::errors::ReportGenerationError::DataExtractionError(
                            e.to_string(),
                        )
                    ))?;
                
                markdown_generator
                    .generate_markdown_report(analysis_run, issues, &anti_pattern_map, ai_insights, None)
                    .await
                    .map_err(|e| {
                        crate::error::UveddiError::from(
                            crate::report::errors::ReportGenerationError::DataExtractionError(
                                e.to_string(),
                            ),
                        )
                    })
            }
            "html" => {
                let codebase_path = config.target_path.to_str();
                report_generator
                    .generate_html_report(analysis_run, issues, &anti_pattern_map, config.output_file.as_deref(), codebase_path)
                    .await
                    .map_err(|e| {
                        crate::error::UveddiError::from(
                            crate::report::errors::ReportGenerationError::DataExtractionError(
                                e.to_string(),
                            ),
                        )
                    })
            }
            _ => Err(UveddiError::config_error(
                &format!(
                    "Unsupported output format specified: {}",
                    config.output_format
                ),
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
                    tracing::warn!(
                        "Memory optimization configuration validation failed: {}",
                        validation_error
                    );
                    tracing::warn!("Falling back to standard analysis mode");
                    return AnalysisEngine::new()
                        .context("Failed to initialize analysis engine")
                        .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"));
                }

                if let Some(memory_config) = config.memory_optimization.clone() {
                    // Validate provided memory config
                    if let Err(validation_error) = memory_config.validate() {
                        tracing::warn!(
                            "Invalid memory optimization configuration: {}",
                            validation_error
                        );
                        tracing::warn!("Falling back to standard analysis mode");
                        return AnalysisEngine::new()
                            .context("Failed to initialize analysis engine")
                            .map_err(|e| {
                                UveddiError::config_error(&e.to_string(), "analysis engine")
                            });
                    }

                    AnalysisEngine::new()
                        .context("Failed to initialize analysis engine with memory optimization")
                        .map_err(|e| {
                            tracing::warn!("Memory optimization initialization failed: {}", e);
                            tracing::warn!("Falling back to standard analysis mode");
                            // Graceful fallback to standard mode
                            AnalysisEngine::new()
                                .context("Failed to initialize analysis engine")
                                .map_err(|e| {
                                    UveddiError::config_error(&e.to_string(), "analysis engine")
                                })
                        })
                        .or_else(|_| {
                            AnalysisEngine::new()
                                .context("Failed to initialize analysis engine")
                                .map_err(|e| {
                                    UveddiError::config_error(&e.to_string(), "analysis engine")
                                })
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
                            tracing::warn!(
                                "Invalid memory limit: {}GB. Using default configuration.",
                                limit_gb
                            );
                        } else {
                            memory_config.target_max_memory_bytes =
                                (limit_gb * 1024.0 * 1024.0 * 1024.0) as usize;
                        }
                    }

                    // Validate the created config
                    if let Err(validation_error) = memory_config.validate() {
                        tracing::warn!(
                            "Generated memory optimization configuration is invalid: {}",
                            validation_error
                        );
                        tracing::warn!("Falling back to standard analysis mode");
                        return AnalysisEngine::new()
                            .context("Failed to initialize analysis engine")
                            .map_err(|e| {
                                UveddiError::config_error(&e.to_string(), "analysis engine")
                            });
                    }

                    AnalysisEngine::new()
                        .context("Failed to initialize analysis engine with memory optimization")
                        .map_err(|e| {
                            tracing::warn!("Memory optimization initialization failed: {}", e);
                            tracing::warn!("Falling back to standard analysis mode");
                            // Graceful fallback to standard mode
                            AnalysisEngine::new()
                                .context("Failed to initialize analysis engine")
                                .map_err(|e| {
                                    UveddiError::config_error(&e.to_string(), "analysis engine")
                                })
                        })
                        .or_else(|_| {
                            AnalysisEngine::new()
                                .context("Failed to initialize analysis engine")
                                .map_err(|e| {
                                    UveddiError::config_error(&e.to_string(), "analysis engine")
                                })
                        })
                }
            }
            #[cfg(not(feature = "memory-optimization"))]
            {
                tracing::warn!("Memory optimization requested but feature not enabled");
                tracing::warn!("Falling back to standard analysis mode");
                AnalysisEngine::new()
                    .context("Failed to initialize analysis engine")
                    .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
            }
        } else {
            AnalysisEngine::new()
                .context("Failed to initialize analysis engine")
                .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
        }
    }

    /// Generate AI insights for the analysis results
    async fn generate_ai_insights(&self, issues: &[ArchitecturalIssue]) -> Option<Vec<AiInsight>> {
        use crate::core::features::ai_config::AiFeatureConfig;
        
        if !AiFeatureConfig::is_enabled() {
            info!("AI features not enabled, using mock insights");
            return Some(self.generate_mock_ai_insights(issues));
        }
        
        // If AI features are enabled, we would integrate with the real AI service here
        info!("AI features enabled, generating real insights");
        Some(self.generate_mock_ai_insights(issues))
    }
    
    /// Generate mock AI insights for demonstration
    fn generate_mock_ai_insights(&self, issues: &[ArchitecturalIssue]) -> Vec<AiInsight> {
        let mut insights = Vec::new();
        
        // Prioritize issues by criticality for AI analysis
        let mut prioritized_issues: Vec<_> = issues.iter().enumerate().collect();
        prioritized_issues.sort_by(|(_, a), (_, b)| {
            let a_priority = self.get_issue_priority(&a.message);
            let b_priority = self.get_issue_priority(&b.message);
            b_priority.cmp(&a_priority) // Sort descending (highest priority first)
        });
        
        // Take top 10 issues for analysis
        for (original_index, issue) in prioritized_issues.iter().take(10) {
            let (confidence, suggestion) = self.generate_detailed_ai_suggestion(issue, insights.len());
            
            insights.push(AiInsight {
                issue_id: issue.issue_id.map(|id| id.to_string()).unwrap_or_else(|| format!("issue_{}", original_index)),
                confidence,
                suggestion,
                metadata: serde_json::json!({
                    "message": issue.message,
                    "file_path": issue.file_path,
                    "line_number": issue.line_number,
                    "issue_type": self.classify_issue_type(&issue.message),
                    "severity": self.classify_issue_severity(&issue.message),
                    "priority": self.get_issue_priority(&issue.message)
                }),
            });
        }
        
        insights
    }
    
    /// Get priority score for issue ordering (higher = more important)
    fn get_issue_priority(&self, message: &str) -> u32 {
        if message.contains("critical") || message.contains("Critical") {
            if message.contains("dependencies") {
                100 // Highest priority - critical dependency issues
            } else {
                90 // Other critical issues
            }
        } else if message.contains("God Object") {
            80 // God objects are important architectural issues
        } else if message.contains("Code duplication") && message.contains("40") {
            70 // Large duplications
        } else if message.contains("Code duplication") && message.contains("24") {
            65 // Medium duplications
        } else if message.contains("Code duplication") {
            60 // Other duplications
        } else if message.contains("dead code") {
            if message.contains("function") {
                50 // Dead functions
            } else {
                45 // Dead structs
            }
        } else {
            30 // Other issues
        }
    }

    /// Generate detailed AI suggestions based on issue analysis
    fn generate_detailed_ai_suggestion(&self, issue: &ArchitecturalIssue, index: usize) -> (f64, String) {
        // Determine confidence based on issue type and severity
        let base_confidence = if issue.message.contains("critical") || issue.message.contains("Critical") {
            0.95
        } else if issue.message.contains("dependencies") {
            0.90
        } else if issue.message.contains("God Object") {
            0.88
        } else if issue.message.contains("Code duplication") {
            0.85
        } else if issue.message.contains("dead code") {
            0.80
        } else {
            0.75
        };
        
        let confidence = (base_confidence - (index as f64 * 0.02)).max(0.60);
        
        let suggestion = if issue.message.contains("dependencies") && issue.message.contains("critical") {
            self.generate_dependency_analysis(issue)
        } else if issue.message.contains("God Object") {
            self.generate_god_object_analysis(issue)
        } else if issue.message.contains("Code duplication") {
            self.generate_duplication_analysis(issue)
        } else if issue.message.contains("dead code") {
            self.generate_dead_code_analysis(issue)
        } else if issue.message.contains("LongMethod") {
            self.generate_long_method_analysis(issue)
        } else if issue.message.contains("LargeClass") {
            self.generate_large_class_analysis(issue)
        } else if issue.message.contains("FeatureEnvy") {
            self.generate_feature_envy_analysis(issue)
        } else if issue.message.contains("ShotgunSurgery") {
            self.generate_shotgun_surgery_analysis(issue)
        } else {
            self.generate_generic_analysis(issue)
        };
        
        (confidence, suggestion)
    }
    
    /// Generate detailed dependency analysis
    fn generate_dependency_analysis(&self, issue: &ArchitecturalIssue) -> String {
        let file_name = std::path::Path::new(&issue.file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("component");
            
        format!(
            "🔗 **Critical Dependency Issue Detected**\n\n\
            **Analysis**: The component '{file_name}' has excessive dependencies, indicating potential architectural violations.\n\n\
            **Root Cause**: High coupling suggests this component is trying to do too many things or is serving as a central hub.\n\n\
            **Recommended Actions**:\n\
            1. **Apply Dependency Inversion Principle**: Extract interfaces for external dependencies\n\
            2. **Use Facade Pattern**: Create a simplified interface to complex subsystems\n\
            3. **Implement Service Locator**: Centralize dependency management\n\
            4. **Consider Module Restructuring**: Break large modules into focused, cohesive units\n\n\
            **Code Strategy**:\n\
            ```rust\n\
            // Instead of direct dependencies:\n\
            // struct Component {{ db: Database, cache: Cache, logger: Logger, ... }}\n\n\
            // Use dependency injection:\n\
            trait Dependencies {{\n\
                fn get_repository(&self) -> &dyn Repository;\n\
                fn get_cache(&self) -> &dyn Cache;\n\
            }}\n\
            ```\n\n\
            **Metrics**: Aim for <8 dependencies per component for maintainable architecture."
        )
    }
    
    /// Generate detailed God Object analysis
    fn generate_god_object_analysis(&self, issue: &ArchitecturalIssue) -> String {
        format!(
            "👑 **God Object Anti-Pattern Detected**\n\n\
            **Analysis**: This structure violates the Single Responsibility Principle by handling too many concerns.\n\n\
            **Symptoms Identified**:\n\
            - High field count indicates data management complexity\n\
            - Multiple responsibilities within single structure\n\
            - Low cohesion between methods and fields\n\n\
            **Refactoring Strategy**:\n\
            1. **Extract Specialized Classes**: Create focused entities for each responsibility\n\
            2. **Apply Command Pattern**: Separate operations into command objects\n\
            3. **Use Composition**: Break into smaller, composable components\n\
            4. **Implement Builder Pattern**: For complex object construction\n\n\
            **Implementation Approach**:\n\
            ```rust\n\
            // Split God Object into focused components:\n\
            struct UserManager {{ /* user operations */ }}\n\
            struct ValidationService {{ /* validation logic */ }}\n\
            struct NotificationService {{ /* notifications */ }}\n\
            \n\
            // Compose via dependency injection or service layer\n\
            ```\n\n\
            **Success Metrics**: Target <8 fields and <10 methods per struct for optimal maintainability."
        )
    }
    
    /// Generate detailed code duplication analysis
    fn generate_duplication_analysis(&self, issue: &ArchitecturalIssue) -> String {
        let lines = issue.message.chars().filter(|&c| c.is_ascii_digit()).collect::<String>()
            .parse::<u32>().unwrap_or(10);
            
        format!(
            "📋 **Code Duplication Analysis**\n\n\
            **Duplication Details**: Found {} similar lines indicating copy-paste programming.\n\n\
            **Impact Assessment**:\n\
            - Maintenance burden: Changes require multiple updates\n\
            - Bug propagation risk: Fixes may be missed in copies\n\
            - Increased codebase size and complexity\n\n\
            **Elimination Strategy**:\n\
            1. **Extract Common Functions**: Create reusable utility functions\n\
            2. **Apply Template Method Pattern**: Define algorithm skeleton with variable steps\n\
            3. **Use Generic Programming**: Parameterize common behavior\n\
            4. **Create Shared Modules**: Centralize common functionality\n\n\
            **Refactoring Pattern**:\n\
            ```rust\n\
            // Before: Duplicated validation logic\n\
            // fn validate_user() {{ /* validation code */ }}\n\
            // fn validate_admin() {{ /* same validation code */ }}\n\
            \n\
            // After: Extracted common validation\n\
            trait Validatable {{\n\
                fn validate(&self) -> Result<(), ValidationError>;\n\
            }}\n\
            \n\
            fn validate_entity<T: Validatable>(entity: &T) -> bool {{\n\
                entity.validate().is_ok()\n\
            }}\n\
            ```\n\n\
            **Quality Target**: Maintain DRY principle with <3% code duplication.",
            lines
        )
    }
    
    /// Generate detailed dead code analysis
    fn generate_dead_code_analysis(&self, issue: &ArchitecturalIssue) -> String {
        let element_type = if issue.message.contains("function") {
            "function"
        } else if issue.message.contains("struct") {
            "struct"
        } else {
            "element"
        };
        
        format!(
            "🗑️ **Dead Code Detection Analysis**\n\n\
            **Element Type**: Unused {element_type} identified with high confidence.\n\n\
            **Technical Debt Impact**:\n\
            - Increases codebase maintenance overhead\n\
            - Slows down compilation and analysis\n\
            - Creates confusion for new developers\n\
            - May contain security vulnerabilities\n\n\
            **Removal Strategy**:\n\
            1. **Verify Usage**: Confirm no dynamic/reflection-based calls\n\
            2. **Check Test Dependencies**: Ensure not used in test-only scenarios\n\
            3. **Review API Surface**: Consider if part of public interface\n\
            4. **Safe Removal**: Use deprecation warnings before deletion\n\n\
            **Verification Process**:\n\
            ```bash\n\
            # Search for dynamic references\n\
            rg -i \"function_name\" --type rust\n\
            \n\
            # Check for string-based invocation\n\
            rg \"\\\"function_name\\\"\" --type rust\n\
            ```\n\n\
            **Best Practice**: Run dead code analysis regularly as part of CI/CD pipeline.\n\n\
            **File**: `{}`",
            issue.file_path
        )
    }
    
    /// Generate long method analysis
    fn generate_long_method_analysis(&self, issue: &ArchitecturalIssue) -> String {
        format!(
            "📏 **Long Method Anti-Pattern**\n\n\
            **Cognitive Complexity**: Method exceeds recommended length thresholds.\n\n\
            **Refactoring Techniques**:\n\
            1. **Extract Method**: Break into smaller, focused functions\n\
            2. **Replace Temp with Query**: Eliminate temporary variables\n\
            3. **Introduce Parameter Object**: Group related parameters\n\
            4. **Replace Method with Method Object**: For complex algorithms\n\n\
            **Target Metrics**: <20 lines per function, <10 cyclomatic complexity."
        )
    }
    
    /// Generate large class analysis
    fn generate_large_class_analysis(&self, issue: &ArchitecturalIssue) -> String {
        format!(
            "🏢 **Large Class Anti-Pattern**\n\n\
            **SRP Violation**: Class has grown beyond single responsibility.\n\n\
            **Decomposition Strategy**:\n\
            1. **Extract Classes**: Separate distinct responsibilities\n\
            2. **Move Methods**: Relocate behavior closer to data\n\
            3. **Use Composition**: Build complex behavior from simpler parts\n\n\
            **Architecture**: Consider hexagonal or clean architecture patterns."
        )
    }
    
    /// Generate feature envy analysis
    fn generate_feature_envy_analysis(&self, issue: &ArchitecturalIssue) -> String {
        format!(
            "👀 **Feature Envy Anti-Pattern**\n\n\
            **Data-Behavior Misalignment**: Component accessing external data excessively.\n\n\
            **Solutions**:\n\
            1. **Move Method**: Relocate behavior to data owner\n\
            2. **Extract Method**: Create focused operations\n\
            3. **Introduce Foreign Method**: Add behavior to external class interface\n\n\
            **Principle**: Follow \"Tell, Don't Ask\" - encapsulate behavior with data."
        )
    }
    
    /// Generate shotgun surgery analysis
    fn generate_shotgun_surgery_analysis(&self, issue: &ArchitecturalIssue) -> String {
        format!(
            "🔫 **Shotgun Surgery Anti-Pattern**\n\n\
            **High Change Impact**: Modifications require touching many files.\n\n\
            **Consolidation Strategy**:\n\
            1. **Move Methods**: Centralize related functionality\n\
            2. **Inline Classes**: Combine overly distributed behavior\n\
            3. **Use Design Patterns**: Apply Observer, Strategy, or Template Method\n\n\
            **Goal**: Minimize change ripple effects through better encapsulation."
        )
    }
    
    /// Generate generic analysis for unclassified issues
    fn generate_generic_analysis(&self, issue: &ArchitecturalIssue) -> String {
        format!(
            "🔍 **Architectural Quality Assessment**\n\n\
            **Issue Context**: General code quality improvement opportunity identified.\n\n\
            **SOLID Principles Review**:\n\
            1. **Single Responsibility**: Does this component have one clear purpose?\n\
            2. **Open/Closed**: Can it be extended without modification?\n\
            3. **Liskov Substitution**: Are abstractions properly defined?\n\
            4. **Interface Segregation**: Are interfaces focused and minimal?\n\
            5. **Dependency Inversion**: Does it depend on abstractions?\n\n\
            **Recommended Actions**:\n\
            - Review component boundaries and responsibilities\n\
            - Consider applying relevant design patterns\n\
            - Evaluate coupling and cohesion metrics\n\
            - Add comprehensive unit tests\n\n\
            **Quality Metrics**: Aim for high cohesion, low coupling architecture."
        )
    }
    
    /// Classify issue type from message
    fn classify_issue_type(&self, message: &str) -> String {
        if message.contains("dependencies") {
            "High Coupling"
        } else if message.contains("God Object") {
            "God Object"
        } else if message.contains("duplication") {
            "Code Duplication"
        } else if message.contains("dead code") {
            "Dead Code"
        } else if message.contains("LongMethod") {
            "Long Method"
        } else if message.contains("LargeClass") {
            "Large Class"
        } else {
            "General Quality"
        }.to_string()
    }
    
    /// Classify issue severity from message
    fn classify_issue_severity(&self, message: &str) -> String {
        if message.contains("critical") || message.contains("Critical") {
            "Critical"
        } else if message.contains("high") || message.contains("High") {
            "High"
        } else if message.contains("medium") || message.contains("Medium") {
            "Medium"
        } else {
            "Low"
        }.to_string()
    }

    /// Validate memory optimization configuration
    #[cfg(feature = "memory-optimization")]
    fn validate_memory_optimization_config(config: &AnalysisConfig) -> Result<(), String> {
        if let Some(limit_gb) = config.memory_limit_gb {
            if limit_gb <= 0.0 {
                return Err(format!(
                    "Memory limit must be positive, got: {}GB",
                    limit_gb
                ));
            }
            if limit_gb > 1000.0 {
                return Err(format!(
                    "Memory limit too high ({}GB), maximum is 1000GB",
                    limit_gb
                ));
            }
        }

        if let Some(profile) = config.memory_profile.as_deref() {
            if !matches!(profile, "small" | "default" | "large") {
                return Err(format!(
                    "Invalid memory profile '{}', must be one of: small, default, large",
                    profile
                ));
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
/// and dispatches to the relevant handlers (e.g., `analyze`, `config`). It leverages
/// the existing async runtime context from #[tokio::main] for proper async operation
/// handling, following UV-294 async standardization guidelines.
pub async fn run_app() -> Result<(), UveddiError> {
    use crate::cli::{analyze_command::AnalyzeCommand, config_command::ConfigCommand};
    use clap::Parser;
    use crate::core::logging::{error, info};

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
        Ui(crate::cli::ui_command::UiCommand),
    }

    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Analyze(command) => {
            info!("Executing analyze command...");
            command.execute_validated().await
        }
        Commands::Config(command) => {
            info!("Executing config command...");
            command
                .execute()
                .map_err(|e| UveddiError::config_error(&e.to_string(), "config validation"))
        }
        Commands::Ui(command) => {
            info!("Executing UI command...");
            command
                .execute()
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "ui command"))
        }
    };
    if let Err(e) = result {
        error!("Error: {e:?}");
        return Err(e);
    }
    Ok(())
}
