use clap::{Parser, Subcommand};
use log::info;

use uveddi::cli::analyze_command::AnalyzeCommand;
use uveddi::cli::config_command::ConfigCommand;
use uveddi::cli::init_local_ai_command::InitLocalAiCommand;
use uveddi::cli::plugin_command::PluginCommand;
use uveddi::error::UveddiError;

/// Uveddi - A tool for code analysis and exploration
///
/// This is the main entry point for the Uveddi application.

#[derive(Parser)]
#[command(name = "uveddi")]
#[command(about = "A Rust-based code analysis and exploration tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a codebase at the given path
    Analyze(AnalyzeCommand),
    /// Initialize and set up local AI (Ollama)
    InitLocalAi(InitLocalAiCommand),
    /// Plugin management commands
    Plugin(PluginCommand),
    /// Configuration management commands
    Config(ConfigCommand),
}

#[tokio::main]
async fn main() -> Result<(), UveddiError> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze(command) => {
            info!("Executing analyze command...");
            command.execute().await?;
        }
        Commands::InitLocalAi(command) => {
            info!("Executing init-local-ai command...");
            let setup = uveddi::cli::init_local_ai_command::OllamaSetup;
            command.execute(&setup).await?;
        }
        Commands::Plugin(command) => {
            info!("Executing plugin command...");
            command.execute().await;
        }
        Commands::Config(command) => {
            info!("Executing config command...");
            // ConfigCommand is sync, so run in blocking
            command.execute().map_err(UveddiError::Configuration)?;
        }
    }

    Ok(())
}
