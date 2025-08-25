//! Doctor command implementation
//!
//! Provides comprehensive health checking and auto-fixing capabilities
//! for the Uveddi installation and environment.

use crate::health::{HealthChecker, HealthReport, HealthStatus};
use crate::core::UveddiError;
use clap::Args;
use std::io::{self, Write};

#[derive(Args, Debug)]
pub struct DoctorCommand {
    /// Automatically attempt to fix detected issues
    #[arg(long, help = "Automatically fix issues where possible")]
    pub fix: bool,

    /// Check only language parser health
    #[arg(long, help = "Check only language parser availability and functionality")]
    pub parsers: bool,

    /// Check only AI integration health
    #[arg(long, help = "Check only AI service connectivity and model availability")]
    pub ai: bool,

    /// Check only system health
    #[arg(long, help = "Check only system resources and file permissions")]
    pub system: bool,

    /// Output format for the health report
    #[arg(long, default_value = "human", help = "Output format: human, json, markdown")]
    pub output_format: String,

    /// Run extended tests on each component
    #[arg(long, help = "Run extended functionality tests (slower but more comprehensive)")]
    pub extended: bool,

    /// Suppress warnings in output (only show critical issues)
    #[arg(long, help = "Only report critical issues, suppress warnings")]
    pub quiet: bool,
}

impl DoctorCommand {
    pub async fn execute(&self) -> Result<(), UveddiError> {
        println!("🔍 Running Uveddi health diagnostics...\n");

        let health_checker = self.create_health_checker();
        let report = health_checker.run_checks().await?;

        self.display_report(&report).await?;

        if self.fix && report.auto_fixable_count() > 0 {
            println!("\n🔧 Attempting to fix detected issues...\n");
            self.apply_fixes(&health_checker, &report).await?;
        }

        self.print_summary(&report).await?;

        // Exit with error code if there are critical issues
        if report.critical_count() > 0 {
            std::process::exit(1);
        }

        Ok(())
    }

    fn create_health_checker(&self) -> HealthChecker {
        match (self.parsers, self.ai, self.system) {
            (true, false, false) => HealthChecker::parsers_only(),
            (false, true, false) => HealthChecker::ai_only(),
            (false, false, true) => HealthChecker {
                check_parsers: false,
                check_ai: false,
                check_system: true,
            },
            _ => HealthChecker::new(), // Default: check all
        }
    }

    async fn display_report(&self, report: &HealthReport) -> Result<(), UveddiError> {
        match self.output_format.as_str() {
            "json" => self.display_json_report(report).await,
            "markdown" => self.display_markdown_report(report).await,
            _ => self.display_human_report(report).await,
        }
    }

    async fn display_human_report(&self, report: &HealthReport) -> Result<(), UveddiError> {
        println!("📊 Health Check Results\n");
        println!("Overall Status: {}", report.overall_status);
        println!("Timestamp: {}\n", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));

        let mut healthy_checks = Vec::new();
        let mut warning_checks = Vec::new();
        let mut critical_checks = Vec::new();

        for check in &report.checks {
            match check.status {
                HealthStatus::Healthy => healthy_checks.push(check),
                HealthStatus::Warning => warning_checks.push(check),
                HealthStatus::Critical => critical_checks.push(check),
            }
        }

        // Display critical issues first
        if !critical_checks.is_empty() {
            println!("❌ Critical Issues ({}):", critical_checks.len());
            for check in critical_checks {
                self.display_check(check);
            }
            println!();
        }

        // Display warnings if not in quiet mode
        if !self.quiet && !warning_checks.is_empty() {
            println!("⚠️  Warnings ({}):", warning_checks.len());
            for check in warning_checks {
                self.display_check(check);
            }
            println!();
        }

        // Display healthy checks in summary form
        if !healthy_checks.is_empty() {
            if self.extended {
                println!("✅ Healthy Components ({}):", healthy_checks.len());
                for check in healthy_checks {
                    self.display_check(check);
                }
            } else {
                println!("✅ Healthy Components ({}): {}", 
                    healthy_checks.len(),
                    healthy_checks.iter()
                        .map(|c| c.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            println!();
        }

        Ok(())
    }

    fn display_check(&self, check: &crate::health::HealthCheck) {
        print!("  {} {}: {}", 
            match check.status {
                HealthStatus::Healthy => "✅",
                HealthStatus::Warning => "⚠️ ",
                HealthStatus::Critical => "❌",
            },
            check.name,
            check.message
        );

        if check.auto_fixable {
            print!(" (auto-fixable)");
        }

        println!();

        if let Some(details) = &check.details {
            println!("     💡 {}", details);
        }
    }

    async fn display_json_report(&self, report: &HealthReport) -> Result<(), UveddiError> {
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| UveddiError::Config(format!("Failed to serialize report to JSON: {}", e)))?;
        
        println!("{}", json);
        Ok(())
    }

    async fn display_markdown_report(&self, report: &HealthReport) -> Result<(), UveddiError> {
        println!("# Uveddi Health Report\n");
        println!("**Overall Status:** {}\n", report.overall_status);
        println!("**Timestamp:** {}\n", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));

        println!("## Summary\n");
        println!("- ✅ Healthy: {}", report.healthy_count());
        println!("- ⚠️  Warnings: {}", report.warning_count());
        println!("- ❌ Critical: {}", report.critical_count());
        println!("- 🔧 Auto-fixable: {}\n", report.auto_fixable_count());

        println!("## Detailed Results\n");

        for check in &report.checks {
            let status_icon = match check.status {
                HealthStatus::Healthy => "✅",
                HealthStatus::Warning => "⚠️",
                HealthStatus::Critical => "❌",
            };

            println!("### {} {}\n", status_icon, check.name);
            println!("**Status:** {}\n", check.status);
            println!("**Message:** {}\n", check.message);

            if let Some(details) = &check.details {
                println!("**Details:** {}\n", details);
            }

            if check.auto_fixable {
                println!("🔧 *This issue can be automatically fixed with --fix*\n");
            }

            println!("---\n");
        }

        Ok(())
    }

    async fn apply_fixes(&self, health_checker: &HealthChecker, report: &HealthReport) -> Result<(), UveddiError> {
        let fix_results = health_checker.run_fixes(report).await?;

        if fix_results.is_empty() {
            println!("No fixes were applied (no auto-fixable issues found).");
            return Ok(());
        }

        for result in fix_results {
            println!("{}", result);
        }

        println!("\n🔄 Re-running health checks to verify fixes...\n");

        let updated_report = health_checker.run_checks().await?;
        let improvements = report.critical_count() + report.warning_count() - 
                          updated_report.critical_count() - updated_report.warning_count();

        if improvements > 0 {
            println!("✅ Successfully resolved {} issue(s)!", improvements);
        } else {
            println!("⚠️  Some issues may require manual intervention.");
        }

        Ok(())
    }

    async fn print_summary(&self, report: &HealthReport) -> Result<(), UveddiError> {
        println!("📈 Summary:");
        println!("  Healthy:      {} components", report.healthy_count());
        
        if !self.quiet {
            println!("  Warnings:     {} components", report.warning_count());
        }
        
        println!("  Critical:     {} components", report.critical_count());
        
        if report.auto_fixable_count() > 0 && !self.fix {
            println!("  Auto-fixable: {} components (run with --fix to attempt automatic fixes)", 
                    report.auto_fixable_count());
        }

        match report.overall_status {
            HealthStatus::Healthy => {
                println!("\n🎉 All systems are operational!");
                println!("   Your Uveddi installation is ready for use.");
            },
            HealthStatus::Warning => {
                println!("\n🟡 Some issues detected, but core functionality should work.");
                if report.auto_fixable_count() > 0 && !self.fix {
                    println!("   Run `uveddi doctor --fix` to automatically resolve fixable issues.");
                }
            },
            HealthStatus::Critical => {
                println!("\n🔴 Critical issues detected that may prevent proper operation.");
                println!("   Please resolve these issues before using Uveddi for analysis.");
                if report.auto_fixable_count() > 0 && !self.fix {
                    println!("   Run `uveddi doctor --fix` to automatically resolve fixable issues.");
                }
            }
        }

        Ok(())
    }
}

/// Convenience function to run basic health check
pub async fn quick_health_check() -> Result<bool, UveddiError> {
    let health_checker = HealthChecker::new();
    let report = health_checker.run_checks().await?;
    
    Ok(report.overall_status != HealthStatus::Critical)
}