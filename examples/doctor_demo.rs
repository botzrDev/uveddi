#!/usr/bin/env rust-script
//! Demo script showing the doctor command functionality
//! 
//! This demonstrates the health check system implementation
//! without requiring the full Uveddi build to work.
//! 
//! Usage: cargo run --bin doctor_demo

use serde::{Deserialize, Serialize};
use std::fmt;

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
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
    pub timestamp: String,
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
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }

    pub fn healthy_count(&self) -> usize {
        self.checks.iter().filter(|c| c.status == HealthStatus::Healthy).count()
    }

    pub fn warning_count(&self) -> usize {
        self.checks.iter().filter(|c| c.status == HealthStatus::Warning).count()
    }

    pub fn critical_count(&self) -> usize {
        self.checks.iter().filter(|c| c.status == HealthStatus::Critical).count()
    }

    pub fn auto_fixable_count(&self) -> usize {
        self.checks.iter().filter(|c| c.auto_fixable).count()
    }
}

fn display_check(check: &HealthCheck) {
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

fn run_demo_health_checks() -> HealthReport {
    let mut checks = Vec::new();

    // System checks
    checks.push(
        HealthCheck::new(
            "binary_integrity",
            HealthStatus::Healthy,
            "Binary is accessible and valid"
        )
    );

    checks.push(
        HealthCheck::new(
            "file_permissions",
            HealthStatus::Healthy,
            "All required directories are accessible"
        )
    );

    checks.push(
        HealthCheck::new(
            "config_directories",
            HealthStatus::Warning,
            "Some configuration directories are missing"
        )
        .with_auto_fix(true)
        .with_details("Run with --fix to create missing directories")
    );

    // Parser checks
    #[cfg(feature = "tree-sitter")]
    checks.push(
        HealthCheck::new(
            "tree_sitter",
            HealthStatus::Healthy,
            "Tree-sitter parsing library is available"
        )
    );

    #[cfg(not(feature = "tree-sitter"))]
    checks.push(
        HealthCheck::new(
            "tree_sitter",
            HealthStatus::Warning,
            "Tree-sitter parsing library is not enabled"
        )
        .with_details("Compile with --features tree-sitter to enable AST parsing")
    );

    #[cfg(feature = "rust-lang")]
    checks.push(
        HealthCheck::new(
            "rust_parser",
            HealthStatus::Healthy,
            "Rust parser feature is enabled"
        )
    );

    #[cfg(not(feature = "rust-lang"))]
    checks.push(
        HealthCheck::new(
            "rust_parser",
            HealthStatus::Warning,
            "Rust parser is not enabled"
        )
        .with_details("Compile with --features rust-lang to enable Rust analysis")
    );

    // AI checks
    #[cfg(feature = "ai")]
    checks.push(
        HealthCheck::new(
            "ai_features",
            HealthStatus::Healthy,
            "AI features are available"
        )
    );

    #[cfg(not(feature = "ai"))]
    checks.push(
        HealthCheck::new(
            "ai_features",
            HealthStatus::Warning,
            "AI features are not enabled"
        )
        .with_details("Compile with --features ai to enable AI analysis")
    );

    checks.push(
        HealthCheck::new(
            "ollama_connectivity",
            HealthStatus::Critical,
            "Cannot connect to Ollama at http://localhost:11434"
        )
        .with_details("Ensure Ollama is installed and running with 'ollama serve'")
        .with_auto_fix(true)
    );

    HealthReport::new(checks)
}

fn display_report(report: &HealthReport) {
    println!("🔍 Uveddi Health Diagnostics Demo\n");
    println!("📊 Health Check Results\n");
    println!("Overall Status: {}", report.overall_status);
    println!("Timestamp: {}\n", report.timestamp);

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
            display_check(check);
        }
        println!();
    }

    // Display warnings
    if !warning_checks.is_empty() {
        println!("⚠️  Warnings ({}):", warning_checks.len());
        for check in warning_checks {
            display_check(check);
        }
        println!();
    }

    // Display healthy checks in summary
    if !healthy_checks.is_empty() {
        println!("✅ Healthy Components ({}): {}", 
            healthy_checks.len(),
            healthy_checks.iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!();
    }

    println!("📈 Summary:");
    println!("  Healthy:      {} components", report.healthy_count());
    println!("  Warnings:     {} components", report.warning_count());
    println!("  Critical:     {} components", report.critical_count());
    
    if report.auto_fixable_count() > 0 {
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
            if report.auto_fixable_count() > 0 {
                println!("   Run `uveddi doctor --fix` to automatically resolve fixable issues.");
            }
        },
        HealthStatus::Critical => {
            println!("\n🔴 Critical issues detected that may prevent proper operation.");
            println!("   Please resolve these issues before using Uveddi for analysis.");
            if report.auto_fixable_count() > 0 {
                println!("   Run `uveddi doctor --fix` to automatically resolve fixable issues.");
            }
        }
    }
}

fn main() {
    println!("🏥 Uveddi Doctor Command Demo");
    println!("=============================\n");
    
    println!("This demonstrates the new health check system that would be available via:");
    println!("  uveddi doctor                    # Run all health checks");
    println!("  uveddi doctor --fix              # Auto-fix issues where possible");
    println!("  uveddi doctor --parsers          # Check only parser health");
    println!("  uveddi doctor --ai               # Check only AI integration");
    println!("  uveddi doctor --system           # Check only system health");
    println!("  uveddi doctor --output-format json  # JSON output");
    println!();

    let report = run_demo_health_checks();
    display_report(&report);

    println!("\n💡 Demo Notes:");
    println!("   • Real implementation would check actual system state");
    println!("   • Auto-fixes would create directories, start services, etc.");
    println!("   • Parser checks would test actual tree-sitter functionality");
    println!("   • AI checks would ping Ollama and verify model availability");
    println!("   • Output formats include human-readable, JSON, and Markdown");
}