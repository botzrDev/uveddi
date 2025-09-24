//! Application startup and initialization module
//!
//! This module handles the initialization of all Uveddi components including
//! the plugin system, database connections, analysis engine, and other core services.
//! It provides a centralized place for application bootstrap logic.

use crate::analysis::{plugin_detector_adapter::PluginDetectorManager, AnalysisEngine};
use crate::application::plugin_manager::{ApplicationPluginManager, PluginManagerConfig};
use crate::database::{
    create_repository_factory, Database, DatabaseConfig, DatabaseType, RepositoryManager,
};
use crate::error::UveddiError;
use crate::plugins::{PluginRuntime, RuntimeFactory};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Application startup manager that handles initialization of all components
pub struct StartupManager {
    /// Database connection
    database: Option<Arc<Database>>,
    /// Repository manager for data access
    repository_manager: Option<Arc<RepositoryManager>>,
    /// Analysis engine
    analysis_engine: Option<Arc<RwLock<AnalysisEngine>>>,
    /// Plugin manager
    plugin_manager: Option<ApplicationPluginManager>,
    /// Plugin detector manager
    plugin_detector_manager: Option<PluginDetectorManager>,
    /// Plugin runtime
    plugin_runtime: Option<Arc<RwLock<PluginRuntime>>>,
    /// Startup configuration
    config: StartupConfig,
}

/// Configuration for application startup
#[derive(Debug, Clone)]
pub struct StartupConfig {
    /// Database path (None for in-memory)
    pub database_path: Option<PathBuf>,
    /// Enable plugin system
    pub enable_plugins: bool,
    /// Plugin directory
    pub plugin_directory: PathBuf,
    /// Auto-discover and load plugins on startup
    pub auto_load_plugins: bool,
    /// Maximum plugin load timeout in seconds
    pub plugin_load_timeout_seconds: u64,
    /// Development mode (enables additional debugging)
    pub development_mode: bool,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            database_path: Some(PathBuf::from("./.uveddi/database.db")),
            enable_plugins: true,
            plugin_directory: PathBuf::from("./plugins"),
            auto_load_plugins: true,
            plugin_load_timeout_seconds: 30,
            development_mode: cfg!(debug_assertions),
        }
    }
}

impl StartupManager {
    /// Create a new startup manager with default configuration
    pub fn new() -> Self {
        Self {
            database: None,
            repository_manager: None,
            analysis_engine: None,
            plugin_manager: None,
            plugin_detector_manager: None,
            plugin_runtime: None,
            config: StartupConfig::default(),
        }
    }

    /// Create startup manager with custom configuration
    pub fn with_config(config: StartupConfig) -> Self {
        Self {
            database: None,
            repository_manager: None,
            analysis_engine: None,
            plugin_manager: None,
            plugin_detector_manager: None,
            plugin_runtime: None,
            config,
        }
    }

    /// Initialize all application components
    pub async fn initialize(&mut self) -> Result<(), UveddiError> {
        info!("Starting Uveddi application initialization");

        // Initialize database
        self.initialize_database().await?;
        info!("✓ Database initialized");

        // Initialize analysis engine
        self.initialize_analysis_engine().await?;
        info!("✓ Analysis engine initialized");

        // Initialize plugin system if enabled
        if self.config.enable_plugins {
            self.initialize_plugin_system().await?;
            info!("✓ Plugin system initialized");

            // Auto-load plugins if configured
            if self.config.auto_load_plugins {
                self.auto_load_plugins().await?;
                info!("✓ Plugins auto-loaded");
            }
        } else {
            info!("Plugin system disabled in configuration");
        }

        info!("Uveddi application initialization complete");
        Ok(())
    }

    /// Initialize the database connection
    async fn initialize_database(&mut self) -> Result<(), UveddiError> {
        debug!("Initializing database connection");

        // Create configuration for repository-based database
        let db_config = if let Some(db_path) = &self.config.database_path {
            // Ensure directory exists
            if let Some(parent) = db_path.parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    UveddiError::io_error("create database directory", &parent.to_string_lossy(), e)
                })?;
            }
            DatabaseConfig {
                database_type: DatabaseType::SQLite,
                connection_string: db_path.to_string_lossy().to_string(),
                ..Default::default()
            }
        } else {
            DatabaseConfig {
                database_type: DatabaseType::SQLite,
                connection_string: ":memory:".to_string(),
                ..Default::default()
            }
        };

        // Create database with repositories
        let database = Database::new_with_repositories(Some(db_config.clone()))
            .await
            .map_err(|e| {
                UveddiError::database_error_msg(&format!("Failed to initialize database: {}", e))
            })?;

        // Create repository factory and manager
        let repository_factory = create_repository_factory(&db_config).await.map_err(|e| {
            UveddiError::database_error_msg(&format!("Failed to create repository factory: {}", e))
        })?;
        let repository_manager = RepositoryManager::new(repository_factory);

        self.database = Some(Arc::new(database));
        self.repository_manager = Some(Arc::new(repository_manager));
        Ok(())
    }

    /// Initialize the analysis engine
    async fn initialize_analysis_engine(&mut self) -> Result<(), UveddiError> {
        debug!("Initializing analysis engine");

        let engine = AnalysisEngine::new().map_err(|e| {
            UveddiError::config_error(
                &format!("Failed to create analysis engine: {}", e),
                "analysis engine",
            )
        })?;

        self.analysis_engine = Some(Arc::new(RwLock::new(engine)));
        Ok(())
    }

    /// Initialize the plugin system
    async fn initialize_plugin_system(&mut self) -> Result<(), UveddiError> {
        debug!("Initializing plugin system");

        // Get required dependencies
        let database = self
            .database
            .as_ref()
            .ok_or_else(|| UveddiError::config_error("Database not initialized", "plugin system"))?
            .clone();

        let analysis_engine = self
            .analysis_engine
            .as_ref()
            .ok_or_else(|| {
                UveddiError::config_error("Analysis engine not initialized", "plugin system")
            })?
            .clone();

        // Create plugin manager configuration
        let plugin_config = PluginManagerConfig {
            plugins_directory: self.config.plugin_directory.clone(),
            default_security_policy: crate::plugins::SecurityPolicy::default(),
            max_plugins: if self.config.development_mode { 10 } else { 50 },
            enable_hot_reload: self.config.development_mode,
            execution_timeout_seconds: self.config.plugin_load_timeout_seconds,
        };

        // Initialize plugin manager
        let plugin_manager = ApplicationPluginManager::new(
            database.clone(),
            analysis_engine.clone(),
            Some(plugin_config),
        )
        .await?;

        self.plugin_manager = Some(plugin_manager);

        // Initialize plugin runtime
        let plugin_runtime = if self.config.development_mode {
            RuntimeFactory::development_runtime()
        } else {
            RuntimeFactory::production_runtime()
        };

        self.plugin_runtime = Some(Arc::new(RwLock::new(plugin_runtime)));

        // Initialize plugin detector manager
        let plugin_detector_manager = PluginDetectorManager::new().await.map_err(|e| {
            warn!("Failed to initialize plugin detector manager: {}", e);
            e
        })?;

        self.plugin_detector_manager = Some(plugin_detector_manager);

        Ok(())
    }

    /// Auto-discover and load plugins from the plugin directory
    async fn auto_load_plugins(&mut self) -> Result<(), UveddiError> {
        debug!(
            "Auto-loading plugins from: {}",
            self.config.plugin_directory.display()
        );

        // Check if plugin directory exists
        if !self.config.plugin_directory.exists() {
            info!(
                "Plugin directory doesn't exist, creating: {}",
                self.config.plugin_directory.display()
            );
            tokio::fs::create_dir_all(&self.config.plugin_directory)
                .await
                .map_err(|e| {
                    UveddiError::io_error(
                        "create plugin directory",
                        &self.config.plugin_directory.to_string_lossy(),
                        e,
                    )
                })?;
            return Ok(()); // No plugins to load
        }

        // Scan for plugin files
        let mut plugin_files = Vec::new();
        let mut dir_entries = tokio::fs::read_dir(&self.config.plugin_directory)
            .await
            .map_err(|e| {
                UveddiError::io_error(
                    "read plugin directory",
                    &self.config.plugin_directory.to_string_lossy(),
                    e,
                )
            })?;

        while let Some(entry) = dir_entries
            .next_entry()
            .await
            .map_err(|e| UveddiError::io_error("read directory entry", "plugin directory", e))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                // Look for corresponding manifest file
                let manifest_path = path.with_extension("toml");
                if manifest_path.exists() {
                    plugin_files.push((path, manifest_path));
                } else {
                    warn!("Found WASM file without manifest: {}", path.display());
                }
            }
        }

        info!("Found {} plugin(s) to load", plugin_files.len());

        // Load each plugin
        let plugin_manager = self.plugin_manager.as_mut().ok_or_else(|| {
            UveddiError::config_error("Plugin manager not initialized", "auto load plugins")
        })?;

        for (binary_path, manifest_path) in plugin_files {
            match plugin_manager
                .install_plugin(&binary_path, &manifest_path)
                .await
            {
                Ok(plugin_id) => {
                    info!(
                        "Successfully loaded plugin: {} from {}",
                        plugin_id,
                        binary_path.display()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to load plugin from {}: {}",
                        binary_path.display(),
                        e
                    );
                    // Continue loading other plugins
                }
            }
        }

        Ok(())
    }

    /// Get the initialized database
    pub fn database(&self) -> Option<Arc<Database>> {
        self.database.clone()
    }

    /// Get the repository manager
    pub fn repository_manager(&self) -> Option<Arc<RepositoryManager>> {
        self.repository_manager.clone()
    }

    /// Get the initialized analysis engine
    pub fn analysis_engine(&self) -> Option<Arc<RwLock<AnalysisEngine>>> {
        self.analysis_engine.clone()
    }

    /// Get the initialized plugin manager
    pub fn plugin_manager(&mut self) -> Option<&mut ApplicationPluginManager> {
        self.plugin_manager.as_mut()
    }

    /// Get the plugin detector manager
    pub fn plugin_detector_manager(&self) -> Option<&PluginDetectorManager> {
        self.plugin_detector_manager.as_ref()
    }

    /// Get the plugin runtime
    pub fn plugin_runtime(&self) -> Option<Arc<RwLock<PluginRuntime>>> {
        self.plugin_runtime.clone()
    }

    /// Check if plugins are enabled and available
    pub fn plugins_enabled(&self) -> bool {
        self.config.enable_plugins && self.plugin_manager.is_some()
    }

    /// Get startup configuration
    pub fn config(&self) -> &StartupConfig {
        &self.config
    }

    /// Shutdown all components
    pub async fn shutdown(&mut self) -> Result<(), UveddiError> {
        info!("Shutting down Uveddi application");

        // Shutdown plugin system
        if let Some(runtime) = self.plugin_runtime.take() {
            if let Ok(mut runtime_guard) = Arc::try_unwrap(runtime).map(|r| r.into_inner()) {
                runtime_guard.shutdown().await?;
            }
        }

        // Clear other components
        self.plugin_manager = None;
        self.plugin_detector_manager = None;
        self.analysis_engine = None;
        self.repository_manager = None;
        self.database = None;

        info!("Uveddi application shutdown complete");
        Ok(())
    }
}

impl Default for StartupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create and initialize the application
pub async fn initialize_application() -> Result<StartupManager, UveddiError> {
    let mut manager = StartupManager::new();
    manager.initialize().await?;
    Ok(manager)
}

/// Convenience function to initialize application with custom configuration
pub async fn initialize_application_with_config(
    config: StartupConfig,
) -> Result<StartupManager, UveddiError> {
    let mut manager = StartupManager::with_config(config);
    manager.initialize().await?;
    Ok(manager)
}
