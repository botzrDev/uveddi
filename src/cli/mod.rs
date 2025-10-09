//! Command-line interface (CLI) module for Uveddi
//!
//! This module defines the CLI commands and argument parsing for the Uveddi application.
//! Submodules implement each top-level command.
//!
//! NOTE: UV-112, UV-115 - Layer boundary compliance confirmed July 2025. This module only interacts with the Application layer per architecture.

pub mod commands {
    pub mod analyze;
    pub mod ci;
    pub mod config;
    pub mod doctor;
    pub mod help;
    pub mod hooks;
    pub mod init;
    pub mod migrate;
    #[cfg(feature = "wasm-plugins")]
    pub mod plugin;
}

pub mod enhanced_help;
pub mod tui_command;

// Re-export commonly used types
pub use commands::analyze::AnalyzeCommand;
pub use commands::ci::CiCommand;
pub use commands::config::ConfigCommand;
pub use commands::doctor::DoctorCommand;
pub use commands::help::HelpCommand;
pub use commands::hooks::HooksCommand;
pub use commands::init::InitCommand;
pub use commands::migrate::MigrateCommand;
#[cfg(feature = "wasm-plugins")]
pub use commands::plugin::PluginCommand;
