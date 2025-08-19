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
            std::fs::create_dir_all(parent)
                .wrap_err("Failed to create database directory")?;
        }

        // Initialize database
        info!("📊 Initializing database at {:?}", config.database_path);
        let database = Arc::new(
            Database::new(Some(&config.database_path))
                .wrap_err("Failed to initialize database")?,
        );
        self.database = Some(database.clone());

        // Start API server
        info!("🌐 Starting API server on port {}", config.api_port);
        self.start_api_server(database, config.api_port, config.frontend_assets_path.clone())
            .await?;

        // Start rendering service
        info!("🎨 Starting rendering service on port {}", config.rendering_port);
        self.start_rendering_service(config.rendering_port)?;

        // Start frontend (development mode only)
        if config.development_mode {
            info!("🖥️  Starting frontend development server on port {}", config.frontend_port);
            self.start_frontend_dev_server(config.frontend_port)?;
        }

        // Wait a moment for services to initialize
        sleep(Duration::from_millis(1500)).await;

        // Verify services are running
        self.verify_services_health(&config).await?;

        self.services_started = true;
        info!("✅ All Uveddi services started successfully!");
        self.print_service_urls(&config);

        Ok(())
    }

    /// Start the REST API server
    async fn start_api_server(
        &mut self,
        database: Arc<Database>,
        port: u16,
        frontend_assets_path: Option<PathBuf>,
    ) -> Result<()> {
        let config = RestApiConfig {
            enable_cors: true,
            cors_origins: vec!["http://localhost:3000".to_string(), "http://localhost:3001".to_string()],
            spa_assets_path: frontend_assets_path,
            reports_storage_path: PathBuf::from("./.uveddi/reports"),
            serve_spa: true,
            enable_csp: true,
            cache_max_age: 3600,
        };

        let server = CombinedApiServer::new(config, port);
        
        // Start the server in a background task
        let server_handle = tokio::spawn(async move {
            server.start(database).await.map_err(|e| color_eyre::eyre::eyre!("API server failed to start: {}", e))
        });

        self.api_server_handle = Some(server_handle);
        Ok(())
    }

    /// Start the rendering service for Mermaid diagrams
    fn start_rendering_service(&mut self, port: u16) -> Result<()> {
        let rendering_service_path = self.find_rendering_service_path()?;
        
        let mut child = Command::new("node")
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
        
        let mut child = Command::new("npm")
            .arg("run")
            .arg("dev")
            .env("PORT", port.to_string())
            .env("VITE_API_URL", "http://localhost:8080")
            .current_dir(&frontend_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err("Failed to start frontend dev server. Run 'npm install' in the frontend directory.")?;

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
        let exe_dir = exe_path.parent().ok_or_else(|| color_eyre::eyre::eyre!("Invalid executable path"))?;
        
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
        let possible_paths = vec![
            "frontend",
            "./frontend",
            "../frontend",
        ];

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
        let client = reqwest::Client::new();
        let mut retries = 0;
        const MAX_RETRIES: u32 = 10;

        while retries < MAX_RETRIES {
            let mut all_healthy = true;

            // Check API server
            match client.get(&format!("http://localhost:{}/health", config.api_port)).send().await {
                Ok(response) if response.status().is_success() => {
                    info!("✅ API server is healthy");
                }
                _ => {
                    warn!("⚠️ API server not ready yet (attempt {}/{})", retries + 1, MAX_RETRIES);
                    all_healthy = false;
                }
            }

            // Check rendering service
            match client.get(&format!("http://localhost:{}/health", config.rendering_port)).send().await {
                Ok(response) if response.status().is_success() => {
                    info!("✅ Rendering service is healthy");
                }
                _ => {
                    warn!("⚠️ Rendering service not ready yet (attempt {}/{})", retries + 1, MAX_RETRIES);
                    all_healthy = false;
                }
            }

            if all_healthy {
                return Ok(());
            }

            retries += 1;
            sleep(Duration::from_millis(500)).await;
        }

        Err(color_eyre::eyre::eyre!(
            "Services failed to become healthy after {} retries", MAX_RETRIES
        ))
    }

    /// Print service URLs for user reference
    fn print_service_urls(&self, config: &OrchestratorConfig) {
        println!("\n🌟 Uveddi Services Running:");
        println!("   📊 Dashboard: http://localhost:{}", config.api_port);
        println!("   🔧 API Server: http://localhost:{}/api/v1", config.api_port);
        println!("   🎨 Rendering Service: http://localhost:{}", config.rendering_port);
        
        if config.development_mode {
            println!("   🖥️  Dev Frontend: http://localhost:{}", config.frontend_port);
        }
        
        println!("   📈 Health Check: http://localhost:{}/health", config.api_port);
        println!();
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
