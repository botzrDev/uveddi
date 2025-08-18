use clap::{Args, Subcommand};
use crate::application::{AnalysisConfig, AnalysisOrchestrator};
use crate::error::UveddiError;
use crate::core::logging::{info, error};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct CiCommand {
    #[command(subcommand)]
    pub action: CiAction,
}

#[derive(Debug, Subcommand)]
pub enum CiAction {
    /// Run analysis and fail if thresholds are exceeded (for CI quality gates)
    Check(CiCheckArgs),
}

#[derive(Debug, Args)]
pub struct CiCheckArgs {
    /// Path to analyze
    pub path: PathBuf,

    /// Maximum allowed technical debt score (0-100). Fail if exceeded.
    #[arg(long, default_value = "50")]
    pub max_debt: u32,

    /// Maximum allowed critical issues. Fail if exceeded.
    #[arg(long, default_value = "0")]
    pub max_critical: u32,

    /// Output format for debug (optional)
    #[arg(long, default_value = "json")]
    pub output_format: String,
}

impl CiCommand {
    pub async fn execute(&self) -> Result<(), UveddiError> {
        match &self.action {
            CiAction::Check(args) => self.run_check(args).await,
        }
    }

    async fn run_check(&self, args: &CiCheckArgs) -> Result<(), UveddiError> {
        info!("CI check: analyzing {}", args.path.display());
        let mut orchestrator = AnalysisOrchestrator::new()?;
        let config = AnalysisConfig {
            target_path: args.path.clone(),
            output_format: args.output_format.clone(),
            output_file: None,
            enable_ai: false,
            ollama_api_url: None,
            ollama_model: None,
            dead_code_confidence: None,
            dead_code_library_mode: false,
            dead_code_ignore_patterns: None,
            dead_code_keep_alive: None,
            large_classes_max_loc: None,
            large_classes_max_methods: None,
            large_classes_max_fields: None,
            large_classes_max_complexity: None,
            large_classes_max_lcom: None,
            large_classes_ignore_patterns: None,
            large_classes_min_severity: None,
            #[cfg(feature = "memory-optimization")]
            memory_optimization: None,
            enable_memory_optimization: true,
            memory_limit_gb: None,
            memory_profile: None,
            timeout_seconds: 300,
        };

        let report = orchestrator.execute_analysis(config).await?;

        // Try parse JSON to extract summary (preferred)
        if args.output_format.to_lowercase() == "json" {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&report.content) {
                let debt = v.get("summary").and_then(|s| s.get("debtScore")).and_then(|n| n.as_u64()).unwrap_or(0) as u32;
                let critical = v.get("summary").and_then(|s| s.get("issuesBySeverity")).and_then(|m| m.get("critical")).and_then(|n| n.as_u64()).unwrap_or(0) as u32;
                if debt > args.max_debt || critical > args.max_critical {
                    error!("CI quality gate failed: debtScore={} (max {}), critical={} (max {})", debt, args.max_debt, critical, args.max_critical);
                    return Err(UveddiError::config_error("Quality gate failed", "ci-check"));
                }
                info!("CI quality gate passed: debtScore={}, critical={}", debt, critical);
                return Ok(());
            }
        }

        // Fallback: inspect summary via metadata
        let failed = report.metadata.issues_found > 0 && args.max_critical == 0;
        if failed {
            return Err(UveddiError::config_error("Quality gate failed (fallback)", "ci-check"));
        }
        Ok(())
    }
}
