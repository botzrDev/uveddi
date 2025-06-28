use clap::{Parser, Subcommand};
use log::info;

use codeatlas::database::DatabaseManager;


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
    Analyze {
        /// Path to the directory to analyze
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();
    match &cli.command {
        Commands::Analyze { path } => {
            info!("Analyzing directory: {}", path);
            // TODO: Implement analysis logic here
            println!("Analyzing directory: {}", path);
        }
    }

    // Initialize database
    let _db = DatabaseManager::new(None).await?;
    println!("Database initialized successfully!");

    // TODO: Implement core functionality
    // - Code parsing and analysis
    // - Project structure visualization
    // - Dependency mapping
    // - Code metrics calculation

    Ok(())
}

#[cfg(test)]
mod tests {
    // Integration and CLI tests should be in the `tests/` directory for clarity and maintainability.
    // Remove this test to avoid running the CLI main in a test context.
}
