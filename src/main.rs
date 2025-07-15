//! Uveddi CLI Application Entry Point
//!
//! This is the main entry point for the Uveddi command-line interface. It handles
//! command-line argument parsing, initializes the application environment, and
//! delegates execution to the appropriate subcommands.
//!
//! # Application Overview
//!
//! Uveddi is a comprehensive code analysis tool that combines static analysis
//! with AI-powered insights. It provides:
//!
//! - **Multi-language AST analysis**: Parse and analyze code across different languages
//! - **Anti-pattern detection**: Identify code quality issues and architectural problems
//! - **AI-powered explanations**: Generate intelligent explanations for detected issues
//! - **Flexible reporting**: Output results in multiple formats (Markdown, JSON, etc.)
//!
//! # Available Commands
//!
//! ## `analyze`
//! Performs comprehensive analysis of a codebase:
//! ```bash
//! uveddi analyze ./src --output-format markdown --enable-ai
//! ```
//!
//! ## `config`
//! Manages application configuration:
//! ```bash
//! uveddi config show
//! uveddi config set ollama.model "deepseek-coder:6.7b"
//! ```
//!
//! # Environment Variables
//!
//! - `OLLAMA_API_URL`: Base URL for Ollama API (default: http://localhost:11434)
//! - `OLLAMA_MODEL`: Default model for AI analysis
//! - `RUST_LOG`: Logging level (error, warn, info, debug, trace)
//!
//! # Error Handling
//!
//! The application uses `color-eyre` for enhanced error reporting with:
//! - Colorized error messages
//! - Stack traces for debugging
//! - Contextual error information
//! - Suggestions for common issues

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use uveddi::cli::analyze_command::AnalyzeCommand;
use uveddi::cli::config_command::ConfigCommand;
// TODO: Re-enable when monitoring dependencies are properly configured
// use uveddi::monitoring::dashboard::MonitoringDashboard;
// use uveddi::config::monitoring::MonitoringConfig;

mod server;

/// Uveddi CLI application
///
/// A comprehensive code analysis tool that combines static analysis with
/// AI-powered insights to help developers understand and improve their codebases.
#[derive(Parser)]
#[command(name = "uveddi")]
#[command(about = "A Rust-based code analysis and exploration tool with AI integration")]
#[command(
    long_about = "Uveddi performs comprehensive code analysis using static analysis \
    techniques combined with AI-powered insights. It can detect anti-patterns, analyze \
    architectural issues, and provide intelligent explanations for code quality problems."
)]
#[command(version)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the Uveddi CLI
#[derive(Subcommand)]
enum Commands {
    /// Analyze a codebase for quality issues and architectural problems
    ///
    /// Performs comprehensive static analysis on the specified path, detecting
    /// anti-patterns, architectural issues, and code quality problems. Optionally
    /// integrates with AI providers for enhanced explanations and recommendations.
    Analyze(AnalyzeCommand),

    /// Manage Uveddi configuration settings
    ///
    /// View and modify configuration settings for AI providers, output preferences,
    /// and analysis parameters. Configuration can be stored per-user or per-project.
    Config(ConfigCommand),
}

/// Main entry point for Uveddi. All errors are handled and logged consistently.
fn main() -> Result<()> {
    // Set up color_eyre for better error reporting
    color_eyre::install()?;
    env_logger::init();

    // TEMPORARILY DISABLED: Health monitoring server to debug hanging issue
    // TODO: Re-enable after fixing hanging issue

    // Create health monitor instance
    // let health_monitor = Arc::new(Mutex::new(HealthMonitor::new()));

    // Start health monitoring server in a separate thread (non-blocking)
    // let health_monitor_clone = Arc::clone(&health_monitor);
    // std::thread::spawn(move || {
    //     let rt = tokio::runtime::Runtime::new()
    //         .expect("FATAL [UV-150]: Failed to initialize async runtime. This indicates a critical system resource issue. See error handling policy.");
    //
    //     // Start the server asynchronously - this will block the thread but not the main process
    //     if let Err(e) = rt.block_on(server::run_server(health_monitor_clone)) {
    //         eprintln!("Health monitoring server error: {}", e);
    //     }
    // });
    //
    // // Give the server a moment to start
    // std::thread::sleep(std::time::Duration::from_millis(100));

    // TODO: Re-enable when monitoring dependencies are properly configured
    // // Initialize monitoring system (UV-219)
    // let monitoring_config = MonitoringConfig::default();
    // let rt = tokio::runtime::Runtime::new()
    //     .expect("FATAL [UV-219]: Failed to initialize async runtime for monitoring.");
    // rt.block_on(async {
    //     let monitoring_dashboard = MonitoringDashboard::new(monitoring_config).await.expect("Failed to init monitoring dashboard");
    //     monitoring_dashboard.start().await.expect("Failed to start monitoring dashboard");
    // });

    // Run main application
    uveddi::application::run_app().map_err(|e| color_eyre::eyre::eyre!(e))
}
