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

// Targeted warning suppressions - only for unused variables, not all warnings
#![allow(unused_variables, unused_imports)]

use clap::Parser;
use color_eyre::eyre::Result;
use tracing::{debug, error, info};
use uveddi::cli::*;
use uveddi::error::UveddiError;
// TODO: Re-enable when monitoring dependencies are properly configured
// use uveddi::monitoring::dashboard::MonitoringDashboard;
// use uveddi::config::monitoring::MonitoringConfig;

// Server module removed for CLI-only release
// mod server;

/// CLI structure for Uveddi
#[derive(Parser)]
#[command(name = "uveddi")]
#[command(version = "1.0.0")]
#[command(about = "A Rust-based code analysis and exploration tool", long_about = None)]
struct Cli {
    /// Enable verbose logging (info level) for debugging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands
#[derive(clap::Subcommand)]
enum Commands {
    /// Perform comprehensive code analysis
    #[command(alias = "a")]
    Analyze(AnalyzeCommand),
    /// Manage configuration settings
    #[command(alias = "cfg")]
    Config(ConfigCommand),
    /// Run health diagnostics and fix common issues
    #[command(alias = "dr")]
    Doctor(DoctorCommand),
    /// Show help information
    Help(HelpCommand),
    /// Git hooks management
    Hooks(HooksCommand),
    /// Initialize Uveddi configuration for a project
    Init(InitCommand),
    /// CI/CD integration command
    Ci(CiCommand),
    /// Database migration management
    Migrate(MigrateCommand),
    /// Manage WASM plugins
    #[cfg(feature = "wasm-plugins")]
    Plugin(PluginCommand),
}

/// Main entry point for Uveddi. All errors are handled and logged consistently.
///
/// Uses #[tokio::main] pattern for proper async runtime management, eliminating
/// manual runtime creation and async/sync boundary violations per UV-294 guidelines.
#[tokio::main]
async fn main() -> Result<()> {
    // Set up color_eyre for better error reporting
    color_eyre::install()?;

    // Parse CLI early to check for verbose flag
    let cli = Cli::parse();

    // Initialize enhanced logging system
    // Default to "error" for clean output, "info" if --verbose is set
    // RUST_LOG environment variable can still override
    let default_level = if cli.verbose { "info" } else { "error" };
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| default_level.to_string());
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "compact".to_string());

    uveddi::core::logging::unified::init_logging_with_config(&log_level, log_format == "json")
        .expect("Failed to initialize logging");

    info!("Starting Uveddi application");
    debug!("Log level: {}, Format: {}", log_level, log_format);

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
    //         error!("Health monitoring server error: {}", e);
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

    // Execute the appropriate command
    info!("Executing command");
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
        Commands::Ci(command) => {
            info!("Executing CI command...");
            command.execute().await
        }
        Commands::Migrate(command) => {
            info!("Executing migrate command...");
            command.execute().await
        }
        #[cfg(feature = "wasm-plugins")]
        Commands::Plugin(command) => {
            info!("Executing plugin command...");
            command.execute().await
        }
    };

    // Handle result
    match result {
        Ok(_) => {
            info!("Command completed successfully");
            Ok(())
        }
        Err(e) => {
            error!(error = ?e, "Command failed");
            Err(color_eyre::eyre::eyre!(e))
        }
    }
}
