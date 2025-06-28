use clap::{Parser, Subcommand};
use log::info;

use codeatlas::cli::analyze_command::AnalyzeCommand;
use codeatlas::cli::init_local_ai_command::InitLocalAiCommand;

/// CodeAtlas - A tool for code analysis and exploration
///
/// This is the main entry point for the CodeAtlas application.

#[derive(Parser)]
#[command(name = "codeatlas")]
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze(command) => {
            info!("Executing analyze command...");
            command.execute().await?;
        }
        Commands::InitLocalAi(command) => {
            info!("Executing init-local-ai command...");
            command.execute().await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Integration and CLI tests should be in the `tests/` directory for clarity and maintainability.
    // Remove this test to avoid running the CLI main in a test context.
}
