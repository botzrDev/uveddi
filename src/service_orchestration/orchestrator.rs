//! Service Orchestrator for Uveddi
//!
//! This module manages the lifecycle of all Uveddi services including:
//! - REST API server for dashboard integration
//! - Rendering service for Mermaid diagrams
//! - Health monitoring
//! - Database connections
//!
//! When a user runs `uveddi`, this orchestrator ensures all necessary
//! services are started automatically in the background.

use crate::api::{CombinedApiServer, RestApiConfig};
use crate::database::Database;
use color_eyre::eyre::{Result, WrapErr};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Service orchestrator that manages all Uveddi background services
pub struct ServiceOrchestrator {
    api_server_handle: Option<tokio::task::JoinHandle<Result<()>>>,
    rendering_service_process: Option<Child>,
    frontend_process: Option<Child>,
    database: Option<Arc<Database>>,
    services_started: bool,
}

/// Configuration for the service orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Port for the main API server
    pub api_port: u16,
    /// Port for the rendering service
    pub rendering_port: u16,
    /// Port for the frontend development server
    pub frontend_port: u16,
    /// Whether to start services automatically
    pub auto_start_services: bool,
    /// Database path
    pub database_path: PathBuf,
    /// Frontend assets path (for production builds)
    pub frontend_assets_path: Option<PathBuf>,
    /// Whether we're in development mode
    pub development_mode: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            api_port: 8080,
            rendering_port: 3001,
            frontend_port: 3000,
            auto_start_services: true,
            database_path: PathBuf::from("./.uveddi/database.db"),
            frontend_assets_path: None,
            development_mode: false,
        }
    }
}

impl ServiceOrchestrator {
    /// Create a new service orchestrator
    pub fn new() -> Self {
        Self {
            api_server_handle: None,
            rendering_service_process: None,
            frontend_process: None,
            database: None,
            services_started: false,
        }
    }

    /// Start all required services for Uveddi operation
    pub async fn start_services(&mut self, config: OrchestratorConfig) -> Result<()> {
        if self.services_started {
            info!("Services are already running");
            return Ok(());
        }

        info!("🚀 Starting Uveddi services...");

        // Ensure database directory exists
        if let Some(parent) = config.database_path.parent() {
            std::fs::create_dir_all(parent).wrap_err("Failed to create database directory")?;
        }

        // Initialize database
        info!("📊 Initializing database at {:?}", config.database_path);
        let database = Arc::new(
            Database::new(Some(&config.database_path)).wrap_err("Failed to initialize database")?,
        );
        self.database = Some(database.clone());

        // Start API server
        info!("🌐 Starting API server on port {}", config.api_port);
        let api_ready_rx = self
            .start_api_server(
                database,
                config.api_port,
                config.frontend_assets_path.clone(),
            )
            .await?;

        // Start rendering service
        info!(
            "🎨 Starting rendering service on port {}",
            config.rendering_port
        );
        self.start_rendering_service(config.rendering_port)?;

        // Start frontend (development mode only)
        if config.development_mode {
            info!(
                "🖥️  Starting frontend development server on port {}",
                config.frontend_port
            );
            self.start_frontend_dev_server(config.frontend_port)?;
        }

        // Wait for API server to be ready
        info!("⏳ Waiting for API server to be ready...");
        match tokio::time::timeout(Duration::from_secs(10), api_ready_rx).await {
            Ok(Ok(Ok(()))) => {
                info!("✅ API server is ready and listening");
            }
            Ok(Ok(Err(e))) => {
                error!("❌ API server failed to start: {}", e);
                return Err(color_eyre::eyre::eyre!("API server startup failed: {}", e));
            }
            Ok(Err(_)) => {
                error!("❌ API server readiness signal was dropped");
                return Err(color_eyre::eyre::eyre!(
                    "API server readiness signal failed"
                ));
            }
            Err(_) => {
                error!("❌ API server startup timed out after 10 seconds");
                return Err(color_eyre::eyre::eyre!("API server startup timed out"));
            }
        }

        // Give rendering service a moment to start
        info!("⏳ Giving rendering service time to start...");
        sleep(Duration::from_millis(2000)).await;

        // Verify all services are running
        info!("🔍 Checking service health...");
        self.verify_services_health(&config).await?;

        self.services_started = true;
        info!("✅ All Uveddi services started successfully!");
        self.print_service_urls(&config);

        Ok(())
    }

    /// Start the REST API server with readiness notification
    async fn start_api_server(
        &mut self,
        database: Arc<Database>,
        port: u16,
        frontend_assets_path: Option<PathBuf>,
    ) -> Result<oneshot::Receiver<Result<(), Box<dyn std::error::Error + Send + Sync>>>> {
        let config = RestApiConfig {
            enable_cors: true,
            cors_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:3001".to_string(),
                "http://localhost:8082".to_string(),
                "http://localhost:8081".to_string(),
                "http://localhost:8080".to_string(),
            ],
            spa_assets_path: frontend_assets_path,
            reports_storage_path: PathBuf::from("./.uveddi/reports"),
            serve_spa: true,
            enable_csp: true,
            cache_max_age: 3600,
        };

        let server = CombinedApiServer::new(config, port);

        // Create a channel for readiness notification
        let (ready_tx, ready_rx) = oneshot::channel();

        // Start the server in a background task
        let server_handle = tokio::spawn(async move {
            server
                .start_with_readiness(database, ready_tx)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("API server failed to start: {}", e))
        });

        self.api_server_handle = Some(server_handle);
        Ok(ready_rx)
    }

    /// Start the rendering service for Mermaid diagrams
    fn start_rendering_service(&mut self, port: u16) -> Result<()> {
        let rendering_service_path = self.find_rendering_service_path()?;

        let child = Command::new("node")
            .arg(&rendering_service_path)
            .env("PORT", port.to_string())
            .env("NODE_ENV", "production")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err("Failed to start rendering service. Ensure Node.js is installed.")?;

        // Store the child process so we can manage its lifecycle
        self.rendering_service_process = Some(child);
        Ok(())
    }

    /// Start the frontend development server (development mode only)
    fn start_frontend_dev_server(&mut self, port: u16) -> Result<()> {
        let frontend_path = self.find_frontend_path()?;

        let child = Command::new("npm")
            .arg("run")
            .arg("dev")
            .env("PORT", port.to_string())
            .env("VITE_API_URL", "http://localhost:8080")
            .current_dir(&frontend_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err(
                "Failed to start frontend dev server. Run 'npm install' in the frontend directory.",
            )?;

        self.frontend_process = Some(child);
        Ok(())
    }

    /// Find the rendering service executable
    fn find_rendering_service_path(&self) -> Result<PathBuf> {
        let possible_paths = vec![
            "rendering-service/src/server.js",
            "./rendering-service/src/server.js",
            "../rendering-service/src/server.js",
        ];

        for path in possible_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return Ok(path_buf);
            }
        }

        // If not found, try to find it relative to the binary
        let exe_path = std::env::current_exe().wrap_err("Failed to get current executable path")?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid executable path"))?;

        let service_path = exe_dir.join("../rendering-service/src/server.js");
        if service_path.exists() {
            return Ok(service_path);
        }

        Err(color_eyre::eyre::eyre!(
            "Rendering service not found. Please ensure rendering-service/src/server.js exists."
        ))
    }

    /// Find the frontend directory
    fn find_frontend_path(&self) -> Result<PathBuf> {
        let possible_paths = vec!["frontend", "./frontend", "../frontend"];

        for path in possible_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.join("package.json").exists() {
                return Ok(path_buf);
            }
        }

        Err(color_eyre::eyre::eyre!(
            "Frontend directory not found. Please ensure frontend/package.json exists."
        ))
    }

    /// Verify that all services are healthy
    async fn verify_services_health(&self, config: &OrchestratorConfig) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .wrap_err("Failed to create HTTP client")?;

        let mut retries = 0;
        const MAX_RETRIES: u32 = 15;
        let mut last_errors = Vec::new();

        while retries < MAX_RETRIES {
            let mut all_healthy = true;
            let mut errors = Vec::new();

            // Check API server
            let api_url = format!("http://localhost:{}/health", config.api_port);
            match client.get(&api_url).send().await {
                Ok(response) if response.status().is_success() => {
                    info!("✅ API server is healthy at {}", api_url);
                }
                Ok(response) => {
                    let error_msg = format!("API server returned status: {}", response.status());
                    warn!("⚠️ {} (attempt {}/{})", error_msg, retries + 1, MAX_RETRIES);
                    errors.push(error_msg);
                    all_healthy = false;
                }
                Err(e) => {
                    let error_msg = format!("API server connection failed: {}", e);
                    warn!("⚠️ {} (attempt {}/{})", error_msg, retries + 1, MAX_RETRIES);
                    errors.push(error_msg);
                    all_healthy = false;
                }
            }

            // Check rendering service
            let rendering_url = format!("http://localhost:{}/health", config.rendering_port);
            match client.get(&rendering_url).send().await {
                Ok(response) if response.status().is_success() => {
                    info!("✅ Rendering service is healthy at {}", rendering_url);
                }
                Ok(response) => {
                    let error_msg =
                        format!("Rendering service returned status: {}", response.status());
                    warn!("⚠️ {} (attempt {}/{})", error_msg, retries + 1, MAX_RETRIES);
                    errors.push(error_msg);
                    all_healthy = false;
                }
                Err(e) => {
                    let error_msg = format!("Rendering service connection failed: {}", e);
                    warn!("⚠️ {} (attempt {}/{})", error_msg, retries + 1, MAX_RETRIES);
                    errors.push(error_msg);
                    all_healthy = false;
                }
            }

            if all_healthy {
                info!("🎉 All services are healthy and ready!");
                return Ok(());
            }

            retries += 1;
            last_errors = errors; // Store the errors from this attempt

            // Exponential backoff: 500ms, 1s, 2s, 2s, 2s...
            let wait_time = std::cmp::min(500 * (1 << std::cmp::min(retries, 2)), 2000);
            info!(
                "⏳ Retrying health checks in {}ms... (attempt {}/{})",
                wait_time,
                retries + 1,
                MAX_RETRIES
            );
            sleep(Duration::from_millis(wait_time as u64)).await;
        }

        Err(color_eyre::eyre::eyre!(
            "Services failed to become healthy after {} retries. Last errors: {:?}",
            MAX_RETRIES,
            last_errors.join(", ")
        ))
    }

    /// Print service URLs for user reference
    fn print_service_urls(&self, config: &OrchestratorConfig) {
        info!("\n🌟 Uveddi Services Running:");
        info!("   📊 Dashboard: http://localhost:{}", config.api_port);
        info!(
            "   🔧 API Server: http://localhost:{}/api/v1",
            config.api_port
        );
        info!(
            "   🎨 Rendering Service: http://localhost:{}",
            config.rendering_port
        );

        if config.development_mode {
            info!(
                "   🖥️  Dev Frontend: http://localhost:{}",
                config.frontend_port
            );
        }

        info!(
            "   📈 Health Check: http://localhost:{}/health",
            config.api_port
        );
        info!("");
    }

    /// Stop all running services
    pub async fn stop_services(&mut self) -> Result<()> {
        if !self.services_started {
            return Ok(());
        }

        info!("🛑 Stopping Uveddi services...");

        // Stop API server
        if let Some(handle) = self.api_server_handle.take() {
            handle.abort();
            info!("✅ API server stopped");
        }

        // Stop rendering service
        if let Some(mut child) = self.rendering_service_process.take() {
            let _ = child.kill();
            let _ = child.wait();
            info!("✅ Rendering service stopped");
        }

        // Stop frontend dev server
        if let Some(mut child) = self.frontend_process.take() {
            let _ = child.kill();
            let _ = child.wait();
            info!("✅ Frontend dev server stopped");
        }

        self.services_started = false;
        info!("✅ All services stopped successfully");

        Ok(())
    }

    /// Check if services are currently running
    pub fn are_services_running(&self) -> bool {
        self.services_started
    }

    /// Get the database instance
    pub fn database(&self) -> Option<Arc<Database>> {
        self.database.clone()
    }
}

impl Drop for ServiceOrchestrator {
    fn drop(&mut self) {
        // Ensure we clean up processes when the orchestrator is dropped
        if let Some(mut child) = self.rendering_service_process.take() {
            let _ = child.kill();
        }

        if let Some(mut child) = self.frontend_process.take() {
            let _ = child.kill();
        }
    }
}

/// Helper function to start services with default configuration
pub async fn start_default_services() -> Result<ServiceOrchestrator> {
    let mut orchestrator = ServiceOrchestrator::new();
    let config = OrchestratorConfig::default();
    orchestrator.start_services(config).await?;
    Ok(orchestrator)
}

/// Helper function to start services in development mode
pub async fn start_development_services() -> Result<ServiceOrchestrator> {
    let mut orchestrator = ServiceOrchestrator::new();
    let config = OrchestratorConfig {
        development_mode: true,
        ..Default::default()
    };
    orchestrator.start_services(config).await?;
    Ok(orchestrator)
}
