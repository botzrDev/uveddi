//! Command-line interface (CLI) module for Uveddi
//!
//! This module defines the CLI commands and argument parsing for the Uveddi application.
//! Submodules implement each top-level command.
//!
//! NOTE: UV-112, UV-115 - Layer boundary compliance confirmed July 2025. This module only interacts with the Application layer per architecture.

pub mod analyze_command;
pub mod config_command;
pub mod plugin_command;
