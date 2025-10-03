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

mod server;

/// CLI structure for Uveddi
#[derive(Parser)]
#[command(name = "uveddi")]
#[command(version = "1.0.0")]
#[command(about = "A Rust-based code analysis and exploration tool", long_about = None)]
struct Cli {
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
    /// Launch the UI dashboard
    Ui(UiCommand),
    /// CI/CD integration command
    Ci(CiCommand),
    /// Launch the Terminal User Interface for interactive analysis
    Tui(TuiCommand),
    /// Manage WASM plugins
    #[cfg(feature = "wasm-plugins")]
    Plugin(PluginCommand),
    /// Start the web dashboard with all required services
    Serve {
        /// Port for the API server and dashboard
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Port for the rendering service
        #[arg(long, default_value = "3001")]
        rendering_port: u16,

        /// Port for frontend development server
        #[arg(long, default_value = "3000")]
        frontend_port: u16,

        /// Database path
        #[arg(long, default_value = "./.uveddi/database.db")]
        database_path: std::path::PathBuf,

        /// Enable development mode (starts frontend dev server)
        #[arg(long)]
        development: bool,

        /// Frontend build assets path (for production)
        #[arg(long)]
        frontend_assets: Option<std::path::PathBuf>,
    },
}

/// Main entry point for Uveddi. All errors are handled and logged consistently.
///
/// Uses #[tokio::main] pattern for proper async runtime management, eliminating
/// manual runtime creation and async/sync boundary violations per UV-294 guidelines.
#[tokio::main]
async fn main() -> Result<()> {
    // Set up color_eyre for better error reporting
    color_eyre::install()?;

    // Initialize enhanced logging system
    // Use RUST_LOG environment variable or default to "info"
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
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

    // Parse CLI arguments
    info!("Parsing CLI arguments");
    let cli = Cli::parse();

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
        Commands::Ui(command) => {
            info!("Executing UI command...");
            command
                .execute()
                .await
                .map_err(|e| UveddiError::config_error(&e.to_string(), "ui command"))
        }
        Commands::Ci(command) => {
            info!("Executing CI command...");
            command.execute().await
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

            // Auto-detect frontend assets path if not provided
            let frontend_assets_path = frontend_assets.or_else(|| {
                let default_path = std::path::PathBuf::from("frontend/dist");
                if default_path.exists() && default_path.join("index.html").exists() {
                    info!(
                        "📁 Auto-detected frontend assets at: {}",
                        default_path.display()
                    );
                    Some(default_path)
                } else {
                    info!(
                        "📁 No frontend assets found at default location: {}",
                        default_path.display()
                    );
                    None
                }
            });

            #[cfg(feature = "service-orchestration")]
            {
                use uveddi::service_orchestration::{OrchestratorConfig, ServiceOrchestrator};

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

                info!("🌐 Dashboard available at: http://localhost:{}", port);
                info!("🖼️  Rendering service at: http://localhost:{}", rendering_port);
                if development {
                    info!("🛠️  Frontend dev server at: http://localhost:{}", frontend_port);
                }
                info!("📁 Database path: {}", database_path.display());
                info!("\n✨ Press Ctrl+C to stop all services\n");

                // Wait for Ctrl+C
                tokio::signal::ctrl_c()
                    .await
                    .map_err(|e| UveddiError::from(anyhow::anyhow!("Failed to listen for ctrl-c: {}", e)))?;

                info!("Shutting down services...");
                Ok(())
            }

            #[cfg(not(feature = "service-orchestration"))]
            {
                error!("Service orchestration feature is not enabled. Rebuild with --features service-orchestration");
                Err(UveddiError::config_error(
                    "Service orchestration feature not enabled",
                    "serve command",
                ))
            }
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
