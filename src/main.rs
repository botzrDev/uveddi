//! Main entry point for the Uveddi CLI application
//!
//! This binary parses command-line arguments, initializes logging and error reporting,
//! and delegates to the application logic in `uveddi::application`.

//! Uveddi - A tool for code analysis and exploration
//!
//! This is the main entry point for the Uveddi application.
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

/// Main entry point for Uveddi. All errors are handled and logged consistently.
fn main() -> Result<()> {
    // Set up color_eyre for better error reporting
    color_eyre::install()?;
    env_logger::init();
    uveddi::application::run_app().map_err(|e| color_eyre::eyre::eyre!(e))
}
