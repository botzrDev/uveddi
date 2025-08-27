//! UI Command for Interactive Reports
//!
//! This module provides CLI commands for starting the interactive reporting server
//! and managing the React SPA frontend. It bridges the analysis pipeline with the
//! web-based visualization interface.

use crate::api::{CombinedApiServer, RestApiConfig};
use crate::database::Database;
use crate::report::{InteractiveReportConfig, InteractiveReportGenerator};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio;
use tracing::{info, error};

/// UI-related commands for interactive reporting
#[derive(Debug, Args)]
pub struct UiCommand {
    #[command(subcommand)]
    pub action: UiAction,
}

/// UI actions
#[derive(Debug, Subcommand)]
pub enum UiAction {
    /// Start the interactive reporting server
    Serve(ServeArgs),
    /// Generate and open a specific report
    Open(OpenArgs),
    /// Export a report as a portable bundle
    Export(ExportArgs),
}

/// Arguments for the serve command
#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Port to run the server on
    #[arg(short, long, default_value = "4000")]
    pub port: u16,

    /// Path to SPA static assets (if available)
    #[arg(long)]
    pub spa_assets: Option<PathBuf>,

    /// Path to reports storage directory
    #[arg(long, default_value = "./.uveddi/reports")]
    pub reports_dir: PathBuf,

    /// Enable development mode with CORS and detailed logging
    #[arg(long)]
    pub dev: bool,

    /// Database file path
    #[arg(long, default_value = ".uveddi/analysis.db")]
    pub database: PathBuf,
}

/// Arguments for the open command
#[derive(Debug, Args)]
pub struct OpenArgs {
    /// Report ID to open
    pub report_id: String,

    /// Port to run the server on
    #[arg(short, long, default_value = "4000")]
    pub port: u16,

    /// Open browser automatically
    #[arg(long, default_value = "true")]
    pub browser: bool,
}

/// Arguments for the export command
#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Report ID to export
    pub report_id: String,

    /// Output directory for the portable bundle
    #[arg(short, long, default_value = "./report-bundle")]
    pub output: PathBuf,

    /// Include all assets for offline viewing
    #[arg(long)]
    pub portable: bool,
}

impl UiCommand {
    /// Execute the UI command
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &self.action {
            UiAction::Serve(args) => self.serve(args).await,
            UiAction::Open(args) => self.open(args).await,
            UiAction::Export(args) => self.export(args).await,
        }
    }

    /// Start the interactive reporting server
    async fn serve(
        &self,
        args: &ServeArgs,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🚀 Starting Uveddi Interactive Reports Server");
        println!();

        // Create reports directory if it doesn't exist
        tokio::fs::create_dir_all(&args.reports_dir).await?;

        // Initialize database
        let database = Arc::new(Database::new(Some(args.database.as_path()))?);
        info!("✅ Database initialized: {}", args.database.display());

        // Create server configuration
        let config = RestApiConfig {
            enable_cors: args.dev,
            cors_origins: if args.dev {
                vec![
                    "http://localhost:3000".to_string(),
                    "http://127.0.0.1:3000".to_string(),
                    "http://localhost:5173".to_string(), // Vite dev server
                ]
            } else {
                vec![]
            },
            spa_assets_path: args.spa_assets.clone(),
            reports_storage_path: args.reports_dir.clone(),
            serve_spa: true,
            enable_csp: !args.dev, // Relaxed CSP in dev mode
            cache_max_age: if args.dev { 0 } else { 3600 },
        };

        // Generate demo report if it doesn't exist
        self.ensure_demo_report(&args.reports_dir).await?;

        // Start server
        let server = CombinedApiServer::new(config, args.port);

        info!("✅ Server configuration complete");
        println!();
        info!("🌐 Server starting on http://localhost:{}", args.port);
        println!(
            "📊 Dashboard: http://localhost:{}/dashboard/demo",
            args.port
        );
        println!("🔧 API: http://localhost:{}/api/v1/reports", args.port);
        println!("❤️  Health: http://localhost:{}/health", args.port);
        println!();

        if args.dev {
            println!("🛠️  Development mode enabled");
            println!("   • CORS enabled for frontend dev servers");
            println!("   • Relaxed Content Security Policy");
            println!("   • No asset caching");
            println!();
        }

        if args.spa_assets.is_none() {
            println!("ℹ️  SPA assets not configured - serving API-only mode");
            println!("   Run 'npm run build' in the frontend directory and use --spa-assets flag");
            println!();
        }

        println!("Press Ctrl+C to stop the server");
        println!();

        server.start(database).await
    }

    /// Open a specific report in the browser
    async fn open(&self, args: &OpenArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔗 Opening report: {}", args.report_id);

        // Start server in background (this is a simplified version)
        let url = format!(
            "http://localhost:{}/dashboard/{}",
            args.port, args.report_id
        );

        if args.browser {
            // Try to open browser
            if let Err(e) = open_browser(&url) {
                error!("⚠️  Could not open browser automatically: {}", e);
                println!("   Please open: {}", url);
            } else {
                info!("✅ Opened in browser: {}", url);
            }
        } else {
            info!("🌐 Report available at: {}", url);
        }

        // TODO: Start server in background and keep it running
        println!("ℹ️  Use 'uveddi ui serve' to start the full server");

        Ok(())
    }

    /// Export a report as a portable bundle
    async fn export(
        &self,
        args: &ExportArgs,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("📦 Exporting report: {}", args.report_id);
        println!("📁 Output directory: {}", args.output.display());

        // Create output directory
        tokio::fs::create_dir_all(&args.output).await?;

        // TODO: Implement portable export
        // This would involve:
        // 1. Loading the report data
        // 2. Creating a self-contained HTML file with embedded assets
        // 3. Copying any necessary static resources
        // 4. Creating an index.html that works offline

        println!("⚠️  Export functionality not yet implemented");
        println!("   This will create a self-contained report bundle for offline viewing");

        Ok(())
    }

    /// Ensure demo report exists in storage
    async fn ensure_demo_report(
        &self,
        reports_dir: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let demo_path = reports_dir.join("demo.json");

        if !demo_path.exists() {
            println!("📝 Creating demo report...");

            let config = InteractiveReportConfig {
                storage_path: reports_dir.clone(),
                auto_save: false,
                ..Default::default()
            };

            let generator = InteractiveReportGenerator::new(config);
            let demo_report = InteractiveReportGenerator::generate_demo_report();

            generator.save_report(&demo_report).await?;
            info!("✅ Demo report created");
        }

        Ok(())
    }
}

/// Attempt to open URL in the default browser
fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", url])
            .spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_serve_args_defaults() {
        let args = ServeArgs {
            port: 4000,
            spa_assets: None,
            reports_dir: PathBuf::from("./.uveddi/reports"),
            dev: false,
            database: PathBuf::from(".uveddi/analysis.db"),
        };

        assert_eq!(args.port, 4000);
        assert!(!args.dev);
        assert!(args.spa_assets.is_none());
    }

    #[tokio::test]
    async fn test_demo_report_creation() {
        let temp_dir = tempdir().unwrap();
        let reports_dir = temp_dir.path().to_path_buf();

        let ui_command = UiCommand {
            action: UiAction::Serve(ServeArgs {
                port: 4000,
                spa_assets: None,
                reports_dir: reports_dir.clone(),
                dev: false,
                database: PathBuf::from(":memory:"),
            }),
        };

        // Should create demo report
        ui_command.ensure_demo_report(&reports_dir).await.unwrap();

        // Check that demo.json was created
        let demo_path = reports_dir.join("demo.json");
        assert!(demo_path.exists());

        // Should not recreate if it already exists
        let metadata_before = tokio::fs::metadata(&demo_path).await.unwrap();
        ui_command.ensure_demo_report(&reports_dir).await.unwrap();
        let metadata_after = tokio::fs::metadata(&demo_path).await.unwrap();

        assert_eq!(
            metadata_before.modified().unwrap(),
            metadata_after.modified().unwrap()
        );
    }

    #[test]
    fn test_open_args() {
        let args = OpenArgs {
            report_id: "demo".to_string(),
            port: 4000,
            browser: true,
        };

        assert_eq!(args.report_id, "demo");
        assert_eq!(args.port, 4000);
        assert!(args.browser);
    }

    #[test]
    fn test_export_args() {
        let args = ExportArgs {
            report_id: "test-report".to_string(),
            output: PathBuf::from("./output"),
            portable: true,
        };

        assert_eq!(args.report_id, "test-report");
        assert_eq!(args.output, PathBuf::from("./output"));
        assert!(args.portable);
    }
}
