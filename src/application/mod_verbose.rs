//! Application orchestration module for Uveddi
//!
//! This module provides a simplified, modular interface for codebase analysis,
//! integrating configuration management, orchestration, services, and workflows.
//!
//! The module has been refactored to improve maintainability and provide clear
//! separation of concerns between different aspects of the analysis process.

// Re-export existing modules for backward compatibility
pub mod plugin_manager;
pub mod startup;

// New modular structure
pub mod configuration;
pub mod orchestrator;
pub mod services;
pub mod workflows;

// Re-exports for the new API
pub use configuration::{AnalysisConfig, AiConfig, ApplicationConfig, OutputConfig};
pub use orchestrator::{AnalysisMetadata, AnalysisOrchestrator, AnalysisResult};

// Legacy compatibility layer
use crate::core::logging::{debug, error, info, warn};
use crate::core::mocks::ai_mocks::AiInsight;
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::error::UveddiError;
use crate::report::{markdown_generator::MarkdownReportGenerator, ReportGenerator};
use anyhow::Context;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;

/// Legacy AnalysisConfig for backward compatibility
///
/// **DEPRECATED**: Use `configuration::AnalysisConfig` instead.
/// This struct will be removed in a future version.
#[deprecated(
    since = "0.9.0",
    note = "Use configuration::AnalysisConfig instead. This provides better modularity and configuration validation."
)]
pub struct LegacyAnalysisConfig {
    pub target_path: PathBuf,
    pub output_format: String,
    pub output_file: Option<PathBuf>,
    pub enable_ai: bool,
    pub ollama_api_url: Option<String>,
    pub ollama_model: Option<String>,
    pub dead_code_confidence: Option<f64>,
    pub dead_code_library_mode: bool,
    pub dead_code_ignore_patterns: Option<Vec<String>>,
    pub dead_code_keep_alive: Option<Vec<String>>,
    pub large_classes_max_loc: Option<u32>,
    pub large_classes_max_methods: Option<u32>,
    pub large_classes_max_fields: Option<u32>,
    pub large_classes_max_complexity: Option<u32>,
    pub large_classes_max_lcom: Option<f64>,
    pub large_classes_ignore_patterns: Option<Vec<String>>,
    pub large_classes_min_severity: Option<u32>,
    pub enable_memory_optimization: bool,
    pub memory_limit_gb: Option<f64>,
    pub memory_profile: Option<String>,
    pub timeout_seconds: u64,
    pub enable_resource_management: bool,
    pub resource_config: Option<crate::resource_management::ResourceConfig>,

    #[cfg(feature = "memory-optimization")]
    pub memory_optimization: Option<crate::analysis::memory::MemoryOptimizationConfig>,
}

/// Legacy AnalysisReport for backward compatibility
///
/// **DEPRECATED**: Use workflows and services directly for more control.
/// This struct will be removed in a future version.
#[deprecated(
    since = "0.9.0",
    note = "Use workflows::report_workflow::ReportWorkflowOutput for more detailed reporting capabilities."
)]
pub struct AnalysisReport {
    pub content: String,
    pub metadata: AnalysisMetadata,
}

impl AnalysisOrchestrator {
    /// Legacy method for executing analysis with the old configuration format
    ///
    /// **DEPRECATED**: Use the new modular workflow approach instead.
    ///
    /// # Migration Guide
    ///
    /// Instead of:
    /// ```ignore
    /// let config = LegacyAnalysisConfig { ... };
    /// let result = orchestrator.execute_analysis(config).await?;
    /// ```
    ///
    /// Use:
    /// ```ignore
    /// let analysis_config = AnalysisConfig::new(target_path);
    /// let output_config = OutputConfig::new(format, output_file);
    /// let ai_config = AiConfig::with_ollama(api_url, model);
    ///
    /// let result = orchestrator.execute_core_analysis(&analysis_config).await?;
    /// // Then use workflows for report generation and AI analysis
    /// ```
    #[deprecated(
        since = "0.9.0",
        note = "Use execute_core_analysis with the new configuration modules and workflows"
    )]
    #[allow(deprecated)]
    pub async fn execute_analysis(
        &mut self,
        config: LegacyAnalysisConfig,
    ) -> Result<AnalysisReport, UveddiError> {
        warn!("Using deprecated execute_analysis method. Please migrate to the new modular API.");

        // Convert legacy config to new format
        let analysis_config = self.convert_legacy_config(config)?;

        // Execute core analysis
        let result = self.execute_core_analysis(&analysis_config).await?;

        // Generate report using legacy format
        let report_content = self.generate_legacy_report(&result, &analysis_config).await?;

        Ok(AnalysisReport {
            content: report_content,
            metadata: result.metadata,
        })
    }

    /// Convert legacy configuration to new modular format
    #[allow(deprecated)]
    fn convert_legacy_config(&self, legacy: LegacyAnalysisConfig) -> Result<configuration::AnalysisConfig, UveddiError> {
        let mut config = configuration::AnalysisConfig::new(legacy.target_path);

        // Convert basic settings
        config.timeout_seconds = legacy.timeout_seconds;
        config.enable_resource_management = legacy.enable_resource_management;
        config.resource_config = legacy.resource_config;
        config.enable_memory_optimization = legacy.enable_memory_optimization;
        config.memory_limit_gb = legacy.memory_limit_gb;
        config.memory_profile = legacy.memory_profile;

        #[cfg(feature = "memory-optimization")]
        {
            config.memory_optimization = legacy.memory_optimization;
        }

        // Convert dead code options
        config.dead_code.confidence = legacy.dead_code_confidence;
        config.dead_code.library_mode = legacy.dead_code_library_mode;
        config.dead_code.ignore_patterns = legacy.dead_code_ignore_patterns;
        config.dead_code.keep_alive = legacy.dead_code_keep_alive;

        // Convert large class options
        config.large_classes.max_loc = legacy.large_classes_max_loc;
        config.large_classes.max_methods = legacy.large_classes_max_methods;
        config.large_classes.max_fields = legacy.large_classes_max_fields;
        config.large_classes.max_complexity = legacy.large_classes_max_complexity;
        config.large_classes.max_lcom = legacy.large_classes_max_lcom;
        config.large_classes.ignore_patterns = legacy.large_classes_ignore_patterns;
        config.large_classes.min_severity = legacy.large_classes_min_severity;

        Ok(config)
    }

    /// Generate legacy format report
    async fn generate_legacy_report(
        &self,
        result: &AnalysisResult,
        analysis_config: &configuration::AnalysisConfig,
    ) -> Result<String, UveddiError> {
        // Create a simple output config for legacy compatibility
        let output_config = configuration::OutputConfig::new(
            "json".to_string(),
            None,
        );

        // Use the report workflow to generate content
        let mut report_workflow = workflows::ReportWorkflow::new()?;

        // Get anti-pattern types from database
        let anti_pattern_types = self.database()
            .get_all_anti_pattern_types()
            .map_err(|e| {
                error!("Failed to retrieve anti-pattern types: {}", e);
                UveddiError::from(crate::report::errors::ReportGenerationError::DataExtractionError(
                    e.to_string(),
                ))
            })?;

        let anti_pattern_map: HashMap<i64, crate::database::models::AntiPatternType> =
            anti_pattern_types
                .into_iter()
                .filter_map(|apt| apt.anti_pattern_type_id.map(|id| (id, apt)))
                .collect();

        let workflow_input = workflows::report_workflow::ReportWorkflowInput {
            analysis_result: result.clone(),
            output_config,
            anti_pattern_types: anti_pattern_map,
            ai_insights: None, // Legacy mode doesn't include AI insights
            codebase_path: Some(analysis_config.target_path.clone()),
        };

        let workflow_output = report_workflow.execute(workflow_input).await?;
        Ok(workflow_output.content)
    }
}

/// Legacy function for running the main application
///
/// **DEPRECATED**: This function is provided for backward compatibility only.
///
/// # Migration Guide
///
/// The application entry point has been moved and restructured. For new code,
/// consider using the modular components directly or the new service orchestration.
pub async fn run_app() -> Result<(), UveddiError> {
    warn!("Using deprecated run_app function. Consider migrating to the new modular API.");

    // Import the main CLI functionality
    use crate::cli::{
        analyze_command::AnalyzeCommand, ci_command::CiCommand, config_command::ConfigCommand,
        doctor_command::DoctorCommand, help_command::HelpCommand, hooks_command::HooksCommand,
        init_command::InitCommand, tui_command::TuiCommand,
    };
    use crate::core::logging::{error, info};
    use crate::service_orchestration::{OrchestratorConfig, ServiceOrchestrator};
    use clap::Parser;

    #[cfg(feature = "wasm-plugins")]
    use crate::cli::plugin_command::PluginCommand;

    #[derive(Parser)]
    #[command(name = "uveddi")]
    #[command(about = "A Rust-based code analysis and exploration tool", long_about = None)]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(clap::Subcommand)]
    enum Commands {
        #[command(alias = "a")]
        Analyze(AnalyzeCommand),
        #[command(alias = "cfg")]
        Config(ConfigCommand),
        #[command(alias = "dr")]
        Doctor(DoctorCommand),
        Help(HelpCommand),
        Hooks(HooksCommand),
        Init(InitCommand),
        Ui(crate::cli::ui_command::UiCommand),
        Ci(CiCommand),
        Tui(TuiCommand),
        #[cfg(feature = "wasm-plugins")]
        Plugin(PluginCommand),
        Serve {
            #[arg(short, long, default_value = "8080")]
            port: u16,
            #[arg(long, default_value = "3001")]
            rendering_port: u16,
            #[arg(long, default_value = "3000")]
            frontend_port: u16,
            #[arg(long, default_value = "./.uveddi/database.db")]
            database_path: std::path::PathBuf,
            #[arg(long)]
            development: bool,
            #[arg(long)]
            frontend_assets: Option<std::path::PathBuf>,
        },
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
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "config validation"))
        }
        Commands::Doctor(command) => {
            info!("Executing doctor command...");
            command.execute().await
        }
        Commands::Help(command) => {
            info!("Executing help command...");
            command.execute().await
        }
        Commands::Hooks(command) => {
            info!("Executing hooks command...");
            command.execute().await
        }
        Commands::Init(command) => {
            info!("Executing init command...");
            command.execute().await
        }
        Commands::Ui(command) => {
            info!("Executing UI command...");
            command
                .execute()
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "ui command"))
        }
        Commands::Ci(command) => {
            info!("Executing CI command...");
            command
                .execute()
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "ci command"))
        }
        Commands::Tui(command) => {
            info!("Launching TUI interface...");
            command
                .execute()
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "tui command"))
        }
        #[cfg(feature = "wasm-plugins")]
        Commands::Plugin(command) => {
            info!("Executing plugin command...");
            command.execute().await
        }
        Commands::Serve {
            port,
            rendering_port,
            frontend_port,
            database_path,
            development,
            frontend_assets,
        } => {
            info!("🚀 Starting Uveddi web services...");

            let frontend_assets_path = frontend_assets.or_else(|| {
                let default_path = std::path::PathBuf::from("frontend/dist");
                if default_path.exists() && default_path.join("index.html").exists() {
                    info!("📁 Auto-detected frontend assets at: {}", default_path.display());
                    Some(default_path)
                } else {
                    info!("📁 No frontend assets found at default location: {}", default_path.display());
                    None
                }
            });

            let config = OrchestratorConfig {
                api_port: port,
                rendering_port,
                frontend_port,
                auto_start_services: true,
                database_path: database_path.clone(),
                frontend_assets_path,
                development_mode: development,
            };

            let mut orchestrator = ServiceOrchestrator::new();
            orchestrator.start_services(config).await.map_err(|e| {
                UveddiError::from(anyhow::anyhow!("Service orchestration failed: {}", e))
            })?;

            info!("✅ All services are running. Press Ctrl+C to stop.");
            tokio::signal::ctrl_c()
                .await
                .map_err(|e| UveddiError::from(anyhow::anyhow!("Signal handling failed: {}", e)))?;
            info!("🛑 Stopping services...");

            Ok(())
        }
    };

    if let Err(e) = result {
        error!("Error: {e:?}");
        return Err(e);
    }
    Ok(())
}

/// Module-level documentation and migration guide
///
/// # Migration Guide for v0.9.0 Refactoring
///
/// This module has been significantly refactored to improve maintainability and provide
/// better separation of concerns. Here's how to migrate your code:
///
/// ## Configuration
///
/// **Before (v0.8.x):**
/// ```ignore
/// let config = AnalysisConfig {
///     target_path: PathBuf::from("./src"),
///     output_format: "json".to_string(),
///     enable_ai: true,
///     // ... many other fields mixed together
/// };
/// ```
///
/// **After (v0.9.0+):**
/// ```ignore
/// use uveddi::application::configuration::{AnalysisConfig, OutputConfig, AiConfig, ApplicationConfig};
///
/// let analysis_config = AnalysisConfig::new(PathBuf::from("./src"));
/// let output_config = OutputConfig::new("json".to_string(), None);
/// let ai_config = AiConfig::with_ollama("http://localhost:11434".to_string(), "model".to_string());
///
/// let app_config = ApplicationConfig {
///     analysis: analysis_config,
///     output: output_config,
///     ai: ai_config,
/// };
/// ```
///
/// ## Analysis Execution
///
/// **Before (v0.8.x):**
/// ```ignore
/// let mut orchestrator = AnalysisOrchestrator::new()?;
/// let report = orchestrator.execute_analysis(config).await?;
/// ```
///
/// **After (v0.9.0+):**
/// ```ignore
/// use uveddi::application::{orchestrator::AnalysisOrchestrator, workflows::*};
///
/// let mut orchestrator = AnalysisOrchestrator::new()?;
///
/// // Step 1: Execute core analysis
/// let analysis_result = orchestrator.execute_core_analysis(&app_config.analysis).await?;
///
/// // Step 2: Generate AI insights (optional)
/// let mut ai_workflow = AiWorkflow::new(AiWorkflowConfig::default());
/// let ai_input = AiWorkflowInput { issues: analysis_result.issues.clone(), context: None };
/// let ai_output = ai_workflow.execute(ai_input).await?;
///
/// // Step 3: Generate report
/// let mut report_workflow = ReportWorkflow::new()?;
/// let report_input = ReportWorkflowInput {
///     analysis_result,
///     output_config: app_config.output,
///     anti_pattern_types: HashMap::new(), // Retrieved from database
///     ai_insights: Some(ai_output.insights),
///     codebase_path: Some(app_config.analysis.target_path.clone()),
/// };
/// let report_output = report_workflow.execute(report_input).await?;
/// ```
///
/// ## Key Benefits of the New Structure
///
/// 1. **Modular Configuration**: Separate concerns (analysis, output, AI) into focused config structs
/// 2. **Workflow-based Processing**: Clear separation between analysis, AI processing, and reporting
/// 3. **Service Architecture**: Dependency injection and service lifecycle management
/// 4. **Better Testability**: Each component can be tested in isolation
/// 5. **Extensibility**: Easy to add new workflows and services
///
/// ## Breaking Changes
///
/// - `AnalysisConfig` struct has been split into multiple configuration types
/// - `execute_analysis()` method is deprecated in favor of workflow-based approach
/// - Some configuration field names have changed for clarity
/// - Report generation is now handled by dedicated workflows
///
/// ## Backward Compatibility
///
/// The old API is still available but marked as deprecated. It will be removed in v1.0.0.
/// Please migrate to the new API to take advantage of improved functionality and performance.

// End of module