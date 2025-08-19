//! Shared API types and configurations
//!
//! This module contains shared types and configurations used across the API
//! layer to prevent circular dependencies between different API components.

use std::path::PathBuf;
use tokio::sync::oneshot;

/// Configuration for the REST API server
#[derive(Debug, Clone)]
pub struct RestApiConfig {
    /// Enable CORS for browser clients
    pub enable_cors: bool,
    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,
    /// Path to SPA static assets
    pub spa_assets_path: Option<PathBuf>,
    /// Path to report storage directory
    pub reports_storage_path: PathBuf,
    /// Enable serving of static assets
    pub serve_spa: bool,
    /// Enable Content Security Policy headers
    pub enable_csp: bool,
    /// Cache max-age for static assets (seconds)
    pub cache_max_age: u32,
}

impl Default for RestApiConfig {
    fn default() -> Self {
        Self {
            enable_cors: false,
            cors_origins: vec![],
            spa_assets_path: None,
            reports_storage_path: PathBuf::from("./.uveddi/reports"),
            serve_spa: true,
            enable_csp: true,
            cache_max_age: 3600,
        }
    }
}

/// Server configuration interface
pub trait ApiServer {
    /// Start the API server with the given database
    fn start(
        self,
        database: std::sync::Arc<crate::database::Database>,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send;

    /// Start the API server with readiness notification
    fn start_with_readiness(
        self,
        database: std::sync::Arc<crate::database::Database>,
        ready_tx: oneshot::Sender<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send;
}