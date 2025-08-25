//! TUI command module for launching the Terminal User Interface
//!
//! This module provides the command-line interface for launching Uveddi's
//! interactive terminal interface. It handles arguments, validates the
//! environment, and bridges between the CLI and TUI modules.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use tracing::{info, warn, error, debug};

#[cfg(feature = "tui")]
use crate::tui::events::run_tui;
use tracing::{info, warn, error, debug};

/// Arguments for the TUI command
#[derive(Debug, Args)]
pub struct TuiCommand {
    /// Project path to analyze in TUI (defaults to current directory)
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Skip initial analysis and go straight to TUI interface
    #[arg(long, help = "Skip automatic analysis on startup")]
    pub skip_analysis: bool,

    /// Load existing analysis results from a previous run
    #[arg(long, value_name = "FILE", help = "Load results from JSON file")]
    pub load_results: Option<PathBuf>,

    /// Enable debug mode for TUI development and troubleshooting
    #[arg(long, help = "Enable debug logging and diagnostics")]
    pub debug: bool,

    /// Force TUI to run even in non-interactive environments (not recommended)
    #[arg(long, help = "Force TUI to run in non-interactive environment")]
    pub force: bool,
}

impl TuiCommand {
    /// Execute the TUI command
    pub async fn execute(&self) -> Result<()> {
        // Check if TUI feature is enabled
        #[cfg(not(feature = "tui"))]
        {
            return Err(anyhow::anyhow!(
                "TUI feature not enabled. Please build with: cargo build --features=tui\n\
                 Or use the production build: cargo build --features=production"
            ));
        }

        #[cfg(feature = "tui")]
        {
            // Validate environment before launching TUI
            self.validate_environment().await?;

            // Initialize debug logging if requested
            if self.debug {
                tracing::info!("🔧 Debug mode enabled for TUI");
                tracing::info!("📁 Target path: {}", self.path.display());
                if let Some(ref results_file) = self.load_results {
                    tracing::info!("📊 Loading results from: {}", results_file.display());
                }
            }

            // Launch the TUI
            tracing::info!("🚀 Starting Uveddi Terminal User Interface...");
            run_tui()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to run TUI application: {}", e))?;

            tracing::info!("👋 Thanks for using Uveddi TUI!");
            Ok(())
        }
    }

    /// Validate that the environment is suitable for TUI
    #[cfg(feature = "tui")]
    async fn validate_environment(&self) -> Result<()> {
        use is_terminal::IsTerminal;

        // Check if we're running in a terminal (unless forced)
        if !self.force && !std::io::stderr().is_terminal() {
            return Err(anyhow::anyhow!(
                "TUI requires an interactive terminal environment.\n\n\
                 Suggestions:\n\
                 • Run in a terminal/console application\n\
                 • Use 'uveddi analyze' for non-interactive analysis\n\
                 • Use 'uveddi serve' for web-based interface\n\
                 • Use --force flag to override (not recommended)\n\n\
                 Alternative commands:\n\
                 • uveddi analyze {} --output-format json\n\
                 • uveddi analyze {} --output-format html\n\
                 • uveddi serve --port 8080",
                self.path.display(),
                self.path.display()
            ));
        }

        // Validate target path exists
        if !self.path.exists() {
            return Err(anyhow::anyhow!(
                "Target path does not exist: {}\n\
                 Please provide a valid directory or file path.",
                self.path.display()
            ));
        }

        // Validate results file if specified
        if let Some(ref results_file) = self.load_results {
            if !results_file.exists() {
                return Err(anyhow::anyhow!(
                    "Results file does not exist: {}\n\
                     Please provide a valid JSON results file from a previous analysis.",
                    results_file.display()
                ));
            }

            // Verify it's a JSON file
            if results_file.extension().and_then(|s| s.to_str()) != Some("json") {
                tracing::warn!(
                    "Results file does not have .json extension: {}",
                    results_file.display()
                );
            }
        }

        // Check terminal capabilities
        self.check_terminal_capabilities().await?;

        Ok(())
    }

    /// Check if the terminal has adequate capabilities for TUI
    #[cfg(feature = "tui")]
    async fn check_terminal_capabilities(&self) -> Result<()> {
        use crossterm::terminal;

        // Get terminal size
        let (width, height) = terminal::size().context("Failed to get terminal size")?;

        // Check minimum size requirements
        if width < 80 || height < 24 {
            let message = format!(
                "Terminal size may be too small for optimal TUI experience.\n\
                 Current size: {}x{} (columns x rows)\n\
                 Recommended minimum: 80x24\n\n\
                 The TUI will still work but some content may be truncated.",
                width, height
            );

            if self.force {
                tracing::warn!("{}", message);
            } else {
                error!("⚠️  {}", message);
                error!("Use --force to continue anyway.");
                return Err(anyhow::anyhow!("Terminal size too small"));
            }
        }

        Ok(())
    }

    /// Fallback validation for non-TUI builds
    #[cfg(not(feature = "tui"))]
    async fn validate_environment(&self) -> Result<()> {
        Ok(())
    }

    /// Check terminal capabilities fallback
    #[cfg(not(feature = "tui"))]
    async fn check_terminal_capabilities(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for TuiCommand {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            skip_analysis: false,
            load_results: None,
            debug: false,
            force: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_tui_command_default() {
        let cmd = TuiCommand::default();
        assert_eq!(cmd.path, PathBuf::from("."));
        assert!(!cmd.skip_analysis);
        assert!(cmd.load_results.is_none());
        assert!(!cmd.debug);
        assert!(!cmd.force);
    }

    #[tokio::test]
    async fn test_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let valid_path = temp_dir.path().to_path_buf();
        let invalid_path = PathBuf::from("/nonexistent/path");

        // Valid path should pass validation
        let cmd = TuiCommand {
            path: valid_path,
            force: true, // Force to skip terminal checks in tests
            ..Default::default()
        };

        #[cfg(feature = "tui")]
        {
            assert!(cmd.validate_environment().await.is_ok());
        }

        // Invalid path should fail validation
        let cmd = TuiCommand {
            path: invalid_path,
            force: true,
            ..Default::default()
        };

        #[cfg(feature = "tui")]
        {
            assert!(cmd.validate_environment().await.is_err());
        }
    }

    #[tokio::test]
    async fn test_results_file_validation() {
        let temp_dir = TempDir::new().unwrap();
        let valid_path = temp_dir.path().to_path_buf();

        // Create a temporary JSON file
        let results_file = temp_dir.path().join("results.json");
        std::fs::write(&results_file, "{}").unwrap();

        let cmd = TuiCommand {
            path: valid_path,
            load_results: Some(results_file),
            force: true,
            ..Default::default()
        };

        #[cfg(feature = "tui")]
        {
            assert!(cmd.validate_environment().await.is_ok());
        }

        // Test with non-existent results file
        let cmd = TuiCommand {
            path: temp_dir.path().to_path_buf(),
            load_results: Some(PathBuf::from("/nonexistent/results.json")),
            force: true,
            ..Default::default()
        };

        #[cfg(feature = "tui")]
        {
            assert!(cmd.validate_environment().await.is_err());
        }
    }
}
