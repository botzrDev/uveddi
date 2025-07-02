use clap::{Parser, Subcommand};
use color_eyre::eyre::{Result, WrapErr};
use log::{error, info};

use uveddi::cli::{
    analyze_command::AnalyzeCommand, config_command::ConfigCommand,
    init_local_ai_command::InitLocalAiCommand, plugin_command::PluginCommand,
};
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

fn main() -> Result<()> {
    // Set up color_eyre for better error reporting
    color_eyre::install()?;
    env_logger::init();

    let cli = Cli::parse();

    // Handle commands with context
    let result = match cli.command {
        Commands::Analyze(command) => {
            info!("Executing analyze command...");
            tokio::runtime::Runtime::new()?
                .block_on(command.execute())
                .context("Analyze command failed")
        }
        Commands::InitLocalAi(command) => {
            info!("Executing init-local-ai command...");
            let setup = uveddi::cli::init_local_ai_command::OllamaSetup;
            tokio::runtime::Runtime::new()?
                .block_on(command.execute(&setup))
                .context("Init-local-ai command failed")
        }
        Commands::Plugin(command) => {
            info!("Executing plugin command...");
            tokio::runtime::Runtime::new()?
                .block_on(command.execute())
                .context("Plugin command failed")
        }
        Commands::Config(command) => {
            info!("Executing config command...");
            command
                .execute()
                .map_err(UveddiError::Configuration)
                .context("Config command failed")
        }
    };

    // Handle errors with rich reporting
    if let Err(e) = result {
        error!("{:?}", e);
        return Err(e);
    }

    Ok(())
}
