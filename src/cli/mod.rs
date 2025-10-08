//! Command-line interface (CLI) module for Uveddi
//!
//! This module defines the CLI commands and argument parsing for the Uveddi application.
//! Submodules implement each top-level command.
//!
//! NOTE: UV-112, UV-115 - Layer boundary compliance confirmed July 2025. This module only interacts with the Application layer per architecture.

pub mod analyze_command;
pub mod ci_command;
pub mod config_command;
pub mod doctor_command;
pub mod enhanced_help;
pub mod help_command;
pub mod hooks_command;
pub mod init_command;
#[cfg(feature = "wasm-plugins")]
pub mod plugin_command;

// Re-export commonly used types
pub use analyze_command::AnalyzeCommand;
pub use ci_command::CiCommand;
pub use config_command::ConfigCommand;
pub use doctor_command::DoctorCommand;
pub use help_command::HelpCommand;
pub use hooks_command::HooksCommand;
pub use init_command::InitCommand;
#[cfg(feature = "wasm-plugins")]
pub use plugin_command::PluginCommand;
