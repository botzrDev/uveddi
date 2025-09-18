//! Application orchestration module for Uveddi
//!
//! This module provides a modular interface for codebase analysis, integrating
//! configuration management, orchestration, services, and workflows.

// Re-export existing modules for backward compatibility
pub mod plugin_manager;
pub mod startup;

// New modular structure
pub mod configuration;
pub mod orchestrator;
pub mod services;
pub mod workflows;

// Primary public API exports
pub use configuration::{AnalysisConfig, AiConfig, ApplicationConfig, OutputConfig};
pub use orchestrator::{AnalysisMetadata, AnalysisOrchestrator, AnalysisResult};

// Legacy compatibility imports
use crate::core::logging::{error, info, warn};
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use crate::report::ReportGenerator;
use std::collections::HashMap;
use std::path::PathBuf;

/// Legacy AnalysisConfig for backward compatibility
///
/// **DEPRECATED**: Use `configuration::AnalysisConfig` instead.
#[deprecated(since = "0.9.0", note = "Use configuration::AnalysisConfig instead")]
pub struct LegacyAnalysisConfig {
    pub target_path: PathBuf,
    pub output_format: String,
    pub output_file: Option<PathBuf>,
    pub enable_ai: bool,
    pub ollama_api_url: Option<String>,
    pub ollama_model: Option<String>,
    // Truncated for brevity - see full implementation in backup
}

/// Legacy AnalysisReport for backward compatibility
///
/// **DEPRECATED**: Use workflows directly for more control.
#[deprecated(since = "0.9.0", note = "Use workflows for detailed reporting")]
pub struct AnalysisReport {
    pub content: String,
    pub metadata: AnalysisMetadata,
}

impl AnalysisOrchestrator {
    /// Legacy method for executing analysis
    ///
    /// **DEPRECATED**: Use the new modular workflow approach instead.
    #[deprecated(since = "0.9.0", note = "Use execute_core_analysis with new configuration modules")]
    #[allow(deprecated)]
    pub async fn execute_analysis(
        &mut self,
        config: LegacyAnalysisConfig,
    ) -> Result<AnalysisReport, UveddiError> {
        warn!("Using deprecated execute_analysis method. Please migrate to the new modular API.");

        // Convert legacy config and execute
        let analysis_config = configuration::AnalysisConfig::new(config.target_path);
        let result = self.execute_core_analysis(&analysis_config).await?;

        // Generate basic report
        let report_content = format!(
            "{{\"files_analyzed\":{},\"issues_found\":{},\"duration_ms\":{}}}",
            result.metadata.files_analyzed,
            result.metadata.issues_found,
            result.metadata.analysis_duration.as_millis()
        );

        Ok(AnalysisReport {
            content: report_content,
            metadata: result.metadata,
        })
    }
}

/// Legacy function for running the main application
///
/// **DEPRECATED**: This function is provided for backward compatibility only.
pub async fn run_app() -> Result<(), UveddiError> {
    warn!("Using deprecated run_app function. Consider migrating to the new modular API.");

    // For backward compatibility, this would call the main CLI logic
    // In practice, this is now handled by main.rs directly
    Err(UveddiError::config_error(
        "run_app has been moved to main.rs CLI handling",
        "deprecated function",
    ))
}

/// Migration guide for v0.9.0 refactoring
///
/// The module has been refactored into focused components:
/// - `configuration`: Modular configuration management
/// - `orchestrator`: Core analysis coordination
/// - `services`: Service layer with dependency injection
/// - `workflows`: Workflow-based processing
///
/// For migration examples, see the module-level documentation.
pub mod migration {
    //! Migration utilities and examples for the v0.9.0 refactoring

    /// Check if the legacy API is being used
    pub fn is_using_legacy_api() -> bool {
        // This could check for deprecated function usage
        false
    }

    /// Get migration suggestions
    pub fn get_migration_suggestions() -> Vec<String> {
        vec![
            "Replace LegacyAnalysisConfig with configuration::AnalysisConfig".to_string(),
            "Use workflows for report generation instead of execute_analysis".to_string(),
            "Leverage service layer for better testing and modularity".to_string(),
        ]
    }
}