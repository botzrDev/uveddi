//! Health check system for Uveddi
//!
//! This module provides comprehensive health diagnostics for the Uveddi application,
//! including system checks, parser availability, AI integration status, and auto-fix
//! capabilities. It implements the `doctor` command functionality for troubleshooting
//! and maintaining a healthy development environment.

pub mod ai;
pub mod fixes;
pub mod parsers;
pub mod system;

use crate::error::UveddiError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Overall health status of the Uveddi installation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Minor issues that don't prevent core functionality
    Warning,
    /// Major issues that may prevent proper operation
    Critical,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "✅ Healthy"),
            HealthStatus::Warning => write!(f, "⚠️  Warning"),
            HealthStatus::Critical => write!(f, "❌ Critical"),
        }
    }
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<String>,
    pub auto_fixable: bool,
}

impl HealthCheck {
    pub fn new(name: impl Into<String>, status: HealthStatus, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status,
            message: message.into(),
            details: None,
            auto_fixable: false,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_auto_fix(mut self, fixable: bool) -> Self {
        self.auto_fixable = fixable;
        self
    }
}

/// Complete health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthReport {
    pub fn new(checks: Vec<HealthCheck>) -> Self {
        let overall_status = if checks.iter().any(|c| c.status == HealthStatus::Critical) {
            HealthStatus::Critical
        } else if checks.iter().any(|c| c.status == HealthStatus::Warning) {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        Self {
            overall_status,
            checks,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn healthy_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == HealthStatus::Healthy)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == HealthStatus::Warning)
            .count()
    }

    pub fn critical_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == HealthStatus::Critical)
            .count()
    }

    pub fn auto_fixable_count(&self) -> usize {
        self.checks.iter().filter(|c| c.auto_fixable).count()
    }
}

/// Main health check orchestrator
pub struct HealthChecker {
    pub check_parsers: bool,
    pub check_ai: bool,
    pub check_system: bool,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self {
            check_parsers: true,
            check_ai: true,
            check_system: true,
        }
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parsers_only() -> Self {
        Self {
            check_parsers: true,
            check_ai: false,
            check_system: false,
        }
    }

    pub fn ai_only() -> Self {
        Self {
            check_parsers: false,
            check_ai: true,
            check_system: false,
        }
    }

    pub async fn run_checks(&self) -> Result<HealthReport, UveddiError> {
        let mut checks = Vec::new();

        if self.check_system {
            checks.extend(system::check_system_health().await?);
        }

        if self.check_parsers {
            checks.extend(parsers::check_parser_health().await?);
        }

        if self.check_ai {
            checks.extend(ai::check_ai_health().await?);
        }

        Ok(HealthReport::new(checks))
    }

    pub async fn run_fixes(&self, report: &HealthReport) -> Result<Vec<String>, UveddiError> {
        let mut fix_results = Vec::new();

        for check in &report.checks {
            if check.auto_fixable && check.status != HealthStatus::Healthy {
                match fixes::apply_fix(&check.name).await {
                    Ok(result) => fix_results.push(format!("✅ Fixed: {}", result)),
                    Err(e) => fix_results.push(format!("❌ Failed to fix {}: {}", check.name, e)),
                }
            }
        }

        Ok(fix_results)
    }
}
