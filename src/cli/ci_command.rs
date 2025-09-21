use crate::application::{LegacyAnalysisConfig, AnalysisOrchestrator};
use crate::core::logging::{error, info};
use crate::error::UveddiError;
use clap::{Args, Subcommand};
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

        // Use persistent database for consistency with dashboard
        let database_path = std::path::Path::new("./.uveddi/database.db");
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                UveddiError::config_error(
                    &format!("Failed to create database directory: {}", e),
                    "database setup",
                )
            })?;
        }

        let mut orchestrator = AnalysisOrchestrator::with_db_path(database_path)?;
        #[allow(deprecated)]
        let config = LegacyAnalysisConfig {
            target_path: args.path.clone(),
            output_format: args.output_format.clone(),
            output_file: None,
            enable_ai: false,
            ollama_api_url: None,
            ollama_model: None,
        };

        let report = orchestrator.execute_analysis(config).await?;

        // Try parse JSON to extract summary (preferred)
        if args.output_format.to_lowercase() == "json" {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&report.content) {
                let debt = v
                    .get("summary")
                    .and_then(|s| s.get("debtScore"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0) as u32;
                let critical = v
                    .get("summary")
                    .and_then(|s| s.get("issuesBySeverity"))
                    .and_then(|m| m.get("critical"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0) as u32;
                if debt > args.max_debt || critical > args.max_critical {
                    error!(
                        "CI quality gate failed: debtScore={} (max {}), critical={} (max {})",
                        debt, args.max_debt, critical, args.max_critical
                    );
                    return Err(UveddiError::config_error("Quality gate failed", "ci-check"));
                }
                info!(
                    "CI quality gate passed: debtScore={}, critical={}",
                    debt, critical
                );
                return Ok(());
            }
        }

        // Fallback: inspect summary via metadata
        let failed = report.metadata.issues_found > 0 && args.max_critical == 0;
        if failed {
            return Err(UveddiError::config_error(
                "Quality gate failed (fallback)",
                "ci-check",
            ));
        }
        Ok(())
    }
}

/// Helper function to parse JSON and extract CI metrics for testing
pub fn parse_ci_metrics_from_json(json_content: &str) -> Result<(u32, u32), String> {
    let v: serde_json::Value =
        serde_json::from_str(json_content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let debt = v
        .get("summary")
        .and_then(|s| s.get("debtScore"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0) as u32;

    let critical = v
        .get("summary")
        .and_then(|s| s.get("issuesBySeverity"))
        .and_then(|m| m.get("critical"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0) as u32;

    Ok((debt, critical))
}

/// Evaluate if CI gate should pass given metrics and thresholds
pub fn evaluate_ci_gate(
    debt_score: u32,
    critical_count: u32,
    max_debt: u32,
    max_critical: u32,
) -> bool {
    debt_score <= max_debt && critical_count <= max_critical
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ci_metrics_from_valid_json() {
        let json = r#"{
            "summary": {
                "issuesTotal": 23,
                "issuesBySeverity": {
                    "critical": 2,
                    "high": 5,
                    "medium": 8,
                    "low": 8
                },
                "filesAnalyzed": 42,
                "debtScore": 65.5
            },
            "issues": []
        }"#;

        let (debt, critical) = parse_ci_metrics_from_json(json).unwrap();
        assert_eq!(debt, 65);
        assert_eq!(critical, 2);
    }

    #[test]
    fn test_parse_ci_metrics_missing_summary() {
        let json = r#"{
            "issues": [],
            "timing": {}
        }"#;

        let (debt, critical) = parse_ci_metrics_from_json(json).unwrap();
        assert_eq!(debt, 0);
        assert_eq!(critical, 0);
    }

    #[test]
    fn test_parse_ci_metrics_partial_summary() {
        let json = r#"{
            "summary": {
                "issuesTotal": 5
            },
            "issues": []
        }"#;

        let (debt, critical) = parse_ci_metrics_from_json(json).unwrap();
        assert_eq!(debt, 0);
        assert_eq!(critical, 0);
    }

    #[test]
    fn test_parse_ci_metrics_invalid_json() {
        let json = r#"{ invalid json"#;

        let result = parse_ci_metrics_from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse JSON"));
    }

    #[test]
    fn test_evaluate_ci_gate_pass() {
        assert!(evaluate_ci_gate(30, 0, 50, 1)); // Both under threshold
        assert!(evaluate_ci_gate(50, 1, 50, 1)); // At threshold
        assert!(evaluate_ci_gate(0, 0, 50, 0)); // Zero values
    }

    #[test]
    fn test_evaluate_ci_gate_fail() {
        assert!(!evaluate_ci_gate(60, 0, 50, 1)); // Debt exceeds
        assert!(!evaluate_ci_gate(30, 2, 50, 1)); // Critical exceeds
        assert!(!evaluate_ci_gate(60, 2, 50, 1)); // Both exceed
    }

    #[test]
    fn test_ci_check_args_defaults() {
        let args = CiCheckArgs {
            path: PathBuf::from("/test"),
            max_debt: 50,
            max_critical: 0,
            output_format: "json".to_string(),
        };

        assert_eq!(args.max_debt, 50);
        assert_eq!(args.max_critical, 0);
        assert_eq!(args.output_format, "json");
    }

    #[test]
    fn test_parse_ci_metrics_edge_cases() {
        // Test with non-integer debt score (should be truncated)
        let json = r#"{
            "summary": {
                "debtScore": 42.7,
                "issuesBySeverity": {
                    "critical": 3
                }
            }
        }"#;

        let (debt, critical) = parse_ci_metrics_from_json(json).unwrap();
        assert_eq!(debt, 42);
        assert_eq!(critical, 3);
    }

    #[test]
    fn test_parse_ci_metrics_string_values() {
        // Test that string values are handled gracefully (should default to 0)
        let json = r#"{
            "summary": {
                "debtScore": "not-a-number",
                "issuesBySeverity": {
                    "critical": "also-not-a-number"
                }
            }
        }"#;

        let (debt, critical) = parse_ci_metrics_from_json(json).unwrap();
        assert_eq!(debt, 0);
        assert_eq!(critical, 0);
    }
}
