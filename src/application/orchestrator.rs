//! Core orchestrator for coordinating analysis workflows
//!
//! This module contains the main AnalysisOrchestrator that coordinates
//! the high-level workflow for codebase analysis, integrating database,
//! analysis engine, and report generation components.

use crate::analysis::AnalysisEngine;
use crate::core::logging::{debug, error, info, warn};
use crate::database::{Database, DatabaseConfig, DatabaseType, RepositoryManager, create_repository_factory};
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::error::UveddiError;
use crate::resource_management::{ResourceConfig, ResourceManager};
use anyhow::Context;
use std::time::Duration;

#[cfg(feature = "memory-optimization")]
use crate::analysis::memory::MemoryOptimizationConfig;

/// Application layer orchestrator for analysis workflows
///
/// This struct coordinates the analysis process by managing dependencies
/// and orchestrating the workflow between different system components.
pub struct AnalysisOrchestrator {
    /// The database connection for storing and retrieving analysis results
    database: Database,
    /// Repository manager for data access
    repository_manager: RepositoryManager,
    /// The core analysis engine that performs code parsing and issue detection
    analysis_engine: AnalysisEngine,
    /// Resource manager for memory and system resource control
    resource_manager: Option<ResourceManager>,
}

/// Metadata about a completed analysis operation
#[derive(Debug, Clone)]
pub struct AnalysisMetadata {
    /// The number of files that were analyzed
    pub files_analyzed: usize,
    /// The total number of issues that were found
    pub issues_found: usize,
    /// The duration of the analysis
    pub analysis_duration: Duration,
    /// A flag indicating whether AI enhancement was used
    pub ai_enhanced: bool,
}

/// Result of a completed analysis operation
#[derive(Debug)]
pub struct AnalysisResult {
    /// The analysis run record
    pub analysis_run: AnalysisRun,
    /// The detected issues
    pub issues: Vec<ArchitecturalIssue>,
    /// Analysis metadata
    pub metadata: AnalysisMetadata,
}

impl AnalysisOrchestrator {
    /// Creates a new AnalysisOrchestrator with a database at the specified path
    pub async fn with_db_path(db_path: &std::path::Path) -> Result<Self, UveddiError> {
        let config = DatabaseConfig {
            database_type: DatabaseType::SQLite,
            connection_string: db_path.to_string_lossy().to_string(),
            ..Default::default()
        };

        let database = Database::new_with_repositories(Some(config.clone())).await
            .context("Failed to initialize database with path")?;

        let repository_factory = create_repository_factory(&config).await
            .context("Failed to create repository factory")?;
        let repository_manager = RepositoryManager::new(repository_factory);

        Self::initialize_with_database(database, repository_manager).await
    }

    /// Creates a new AnalysisOrchestrator with an in-memory database
    pub async fn new() -> Result<Self, UveddiError> {
        let config = DatabaseConfig {
            database_type: DatabaseType::SQLite,
            connection_string: ":memory:".to_string(),
            ..Default::default()
        };

        let database = Database::new_with_repositories(Some(config.clone())).await
            .context("Failed to initialize in-memory database")?;

        let repository_factory = create_repository_factory(&config).await
            .context("Failed to create repository factory")?;
        let repository_manager = RepositoryManager::new(repository_factory);

        Self::initialize_with_database(database, repository_manager).await
    }

    /// Creates a new AnalysisOrchestrator with custom memory optimization configuration
    pub async fn with_memory_config(
        db_path: Option<&std::path::Path>,
        memory_config: Option<crate::analysis::memory::MemoryOptimizationConfig>,
    ) -> Result<Self, UveddiError> {
        let config = if let Some(path) = db_path {
            DatabaseConfig {
                database_type: DatabaseType::SQLite,
                connection_string: path.to_string_lossy().to_string(),
                ..Default::default()
            }
        } else {
            DatabaseConfig {
                database_type: DatabaseType::SQLite,
                connection_string: ":memory:".to_string(),
                ..Default::default()
            }
        };

        let database = Database::new_with_repositories(Some(config.clone())).await
            .context("Failed to initialize database")?;

        let repository_factory = create_repository_factory(&config).await
            .context("Failed to create repository factory")?;
        let repository_manager = RepositoryManager::new(repository_factory);

        let mut orchestrator = Self::initialize_with_database(database, repository_manager).await?;

        // Initialize memory optimization with custom configuration
        #[cfg(feature = "memory-optimization")]
        if let Some(config) = memory_config {
            if let Err(e) = crate::analysis::memory::initialize_memory_optimization(config) {
                tracing::warn!("Failed to initialize memory optimization: {}", e);
            } else {
                tracing::info!("Memory optimization initialized with custom configuration");
            }
        }

        Ok(orchestrator)
    }

    /// Initialize orchestrator with given database and repository manager
    async fn initialize_with_database(database: Database, repository_manager: RepositoryManager) -> Result<Self, UveddiError> {
        // Initialize memory optimization with default configuration
        #[cfg(feature = "memory-optimization")]
        {
            let memory_config = crate::analysis::memory::MemoryOptimizationConfig::default();
            if let Err(e) = crate::analysis::memory::initialize_memory_optimization(memory_config) {
                tracing::warn!("Failed to initialize memory optimization: {}", e);
            } else {
                tracing::info!("Memory optimization initialized successfully");
            }
        }

        let analysis_engine =
            AnalysisEngine::new().context("Failed to initialize analysis engine")?;

        // Initialize resource manager if enabled
        let resource_manager = if std::env::var("UVEDDI_ENABLE_RESOURCE_MANAGEMENT")
            .unwrap_or_default()
            .parse::<bool>()
            .unwrap_or(false)
        {
            let resource_config = ResourceConfig::development();
            match ResourceManager::new(resource_config) {
                Ok(manager) => {
                    tracing::info!("Resource management enabled");
                    Some(manager)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize resource manager: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            database,
            repository_manager,
            analysis_engine,
            resource_manager,
        })
    }

    /// Execute the core analysis workflow
    pub async fn execute_core_analysis(
        &mut self,
        config: &super::configuration::AnalysisConfig,
    ) -> Result<AnalysisResult, UveddiError> {
        let start_time = std::time::Instant::now();
        debug!("Starting core analysis execution");

        // Initialize or update resource management if enabled
        if config.enable_resource_management {
            self.setup_resource_management(config).await?;
        }

        // Recreate analysis engine with memory optimization if enabled
        if config.enable_memory_optimization {
            self.setup_memory_optimization(config)?;
        }

        // Validate input path
        self.validate_target_path(config)?;

        // Configure detectors
        self.configure_detectors(config)?;

        // Initialize database schema
        self.initialize_database_schema().await?;

        // Create analysis run record
        let mut analysis_run = self.create_analysis_run(config).await?;

        // Execute core analysis
        let (mut issues, _dependency_graph) = self
            .analysis_engine
            .analyze(&config.target_path)
            .await
            .context("Analysis failed")?;

        debug!("Core analysis completed - found {} issues", issues.len());

        // Update analysis run with results
        let analysis_duration = start_time.elapsed();
        self.finalize_analysis_run(&mut analysis_run, &issues, analysis_duration)
            .await?;

        // Store results in database
        self.store_analysis_results(&mut analysis_run, &mut issues)
            .await?;

        let metadata = AnalysisMetadata {
            files_analyzed: self.analysis_engine.get_files_analyzed() as usize,
            issues_found: issues.len(),
            analysis_duration,
            ai_enhanced: false, // This will be updated by the AI workflow
        };

        Ok(AnalysisResult {
            analysis_run,
            issues,
            metadata,
        })
    }

    /// Get the database instance
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Get a mutable reference to the database
    pub fn database_mut(&mut self) -> &mut Database {
        &mut self.database
    }

    /// Get the analysis engine
    pub fn analysis_engine(&self) -> &AnalysisEngine {
        &self.analysis_engine
    }

    /// Get a mutable reference to the analysis engine
    pub fn analysis_engine_mut(&mut self) -> &mut AnalysisEngine {
        &mut self.analysis_engine
    }

    /// Setup resource management
    async fn setup_resource_management(
        &mut self,
        config: &super::configuration::AnalysisConfig,
    ) -> Result<(), UveddiError> {
        debug!("Resource management enabled, initializing resource manager");

        let resource_config = config
            .resource_config
            .clone()
            .unwrap_or_else(|| ResourceConfig::production());

        match ResourceManager::new(resource_config) {
            Ok(manager) => {
                // Start monitoring in the background
                if let Err(e) = manager.start_monitoring().await {
                    warn!("Failed to start resource monitoring: {}", e);
                } else {
                    info!("Resource monitoring started successfully");
                }

                self.resource_manager = Some(manager);
                debug!("Resource manager initialized successfully");
            }
            Err(e) => {
                warn!("Failed to initialize resource manager: {}", e);
                warn!("Continuing without resource management");
            }
        }

        Ok(())
    }

    /// Setup memory optimization
    fn setup_memory_optimization(
        &mut self,
        config: &super::configuration::AnalysisConfig,
    ) -> Result<(), UveddiError> {
        debug!("Memory optimization enabled, creating optimized analysis engine");
        self.analysis_engine = Self::create_analysis_engine_with_memory_optimization(config)?;
        debug!("Memory-optimized analysis engine created successfully");
        Ok(())
    }

    /// Validate the target path
    fn validate_target_path(
        &self,
        config: &super::configuration::AnalysisConfig,
    ) -> Result<(), UveddiError> {
        debug!("Validating input path: {}", config.target_path.display());
        if !config.target_path.exists() {
            error!("Path does not exist: {}", config.target_path.display());
            return Err(UveddiError::PathError {
                path: config.target_path.display().to_string(),
                reason: "Path does not exist".to_string(),
                suggestion: "Verify the path exists and is accessible".to_string(),
            });
        }
        debug!("Input path validated successfully");
        Ok(())
    }

    /// Configure analysis detectors
    fn configure_detectors(
        &mut self,
        config: &super::configuration::AnalysisConfig,
    ) -> Result<(), UveddiError> {
        debug!("Configuring detectors");

        if let Some(dead_code_config) = config.create_dead_code_config() {
            self.analysis_engine
                .configure_dead_code_detector(dead_code_config);
            info!("Dead code detector configured with custom settings");
        }

        if let Some(large_class_config) = config.create_large_class_config() {
            self.analysis_engine
                .configure_large_classes_detector(large_class_config);
            info!("Large classes detector configured with custom settings");
        }

        debug!("Detectors configured successfully");
        Ok(())
    }

    /// Initialize database schema with anti-pattern types
    async fn initialize_database_schema(&mut self) -> Result<(), UveddiError> {
        debug!("Initializing database schema");
        for mut anti_pattern_type in self.analysis_engine.get_anti_pattern_types() {
            self.database
                .store_anti_pattern_type(&mut anti_pattern_type)
                .context("Failed to store anti-pattern type")?;
        }
        debug!("Database schema initialized successfully");
        Ok(())
    }

    /// Create a new analysis run record
    async fn create_analysis_run(
        &mut self,
        config: &super::configuration::AnalysisConfig,
    ) -> Result<AnalysisRun, UveddiError> {
        debug!("Creating analysis run record");

        // Get project repository to create project if needed
        let project_repo = self.repository_manager.project_repository();
        let project_id = project_repo.get_or_create_project_id(&config.target_path).await
            .context("Failed to get or create project")?;

        // Get analysis repository and create analysis run
        let analysis_repo = self.repository_manager.analysis_repository();
        let analysis_run = analysis_repo.create_analysis_run(project_id).await
            .context("Failed to create analysis run")?;

        debug!(
            "Analysis run record created with ID: {:?}",
            analysis_run.run_id
        );
        Ok(analysis_run)
    }

    /// Finalize the analysis run record with results
    async fn finalize_analysis_run(
        &mut self,
        analysis_run: &mut AnalysisRun,
        issues: &[ArchitecturalIssue],
        duration: Duration,
    ) -> Result<(), UveddiError> {
        analysis_run.total_files_analyzed = Some(self.analysis_engine.get_files_analyzed());
        analysis_run.total_issues_found = Some(issues.len() as i32);
        analysis_run.end_time = Some(chrono::Utc::now());
        analysis_run.status = "completed".to_string();

        // Use analysis repository to update
        let analysis_repo = self.repository_manager.analysis_repository();
        analysis_repo.update(analysis_run).await
            .context("Failed to update analysis run")?;
        Ok(())
    }

    /// Store analysis results in the database
    async fn store_analysis_results(
        &mut self,
        analysis_run: &AnalysisRun,
        issues: &mut [ArchitecturalIssue],
    ) -> Result<(), UveddiError> {
        info!("Starting to store {} issues to database", issues.len());

        // Set the correct analysis_run_id for all issues
        let analysis_run_id = analysis_run.run_id.ok_or_else(|| {
            error!("Analysis run missing ID after creation");
            UveddiError::database_error_msg(
                "Analysis run should have an ID after database insertion",
            )
        })?;

        for issue in issues.iter_mut() {
            issue.analysis_run_id = analysis_run_id;
        }

        self.database
            .store_issues(issues)
            .map_err(|e| {
                error!("Database storage failure - detailed error: {:#}", e);
                e
            })
            .context("Failed to store analysis issues")?;

        info!("Issues stored to database successfully");
        Ok(())
    }

    /// Create analysis engine with memory optimization support
    fn create_analysis_engine_with_memory_optimization(
        config: &super::configuration::AnalysisConfig,
    ) -> Result<AnalysisEngine, UveddiError> {
        #[cfg(feature = "memory-optimization")]
        {
            if let Some(memory_config) = config.memory_optimization.clone() {
                // Validate provided memory config
                if let Err(validation_error) = memory_config.validate() {
                    warn!(
                        "Invalid memory optimization configuration: {}",
                        validation_error
                    );
                    warn!("Falling back to standard analysis mode");
                    return Self::create_fallback_engine();
                }

                AnalysisEngine::new()
                    .context("Failed to initialize analysis engine with memory optimization")
                    .or_else(|_| Self::create_fallback_engine())
            } else {
                // Create memory optimization config based on profile and limits
                let mut memory_config = match config.memory_profile.as_deref() {
                    Some("small") => MemoryOptimizationConfig::small_project(),
                    Some("large") => MemoryOptimizationConfig::large_codebase(),
                    _ => MemoryOptimizationConfig::default(),
                };

                // Apply memory limit if specified
                if let Some(limit_gb) = config.memory_limit_gb {
                    if limit_gb > 0.0 {
                        memory_config.target_max_memory_bytes =
                            (limit_gb * 1024.0 * 1024.0 * 1024.0) as usize;
                    }
                }

                // Validate the created config
                if let Err(validation_error) = memory_config.validate() {
                    warn!(
                        "Generated memory optimization configuration is invalid: {}",
                        validation_error
                    );
                    warn!("Falling back to standard analysis mode");
                    return Self::create_fallback_engine();
                }

                AnalysisEngine::new()
                    .context("Failed to initialize analysis engine with memory optimization")
                    .or_else(|_| Self::create_fallback_engine())
            }
        }
        #[cfg(not(feature = "memory-optimization"))]
        {
            warn!("Memory optimization requested but feature not enabled");
            warn!("Falling back to standard analysis mode");
            Self::create_fallback_engine()
        }
    }

    /// Create fallback analysis engine
    fn create_fallback_engine() -> Result<AnalysisEngine, UveddiError> {
        AnalysisEngine::new()
            .context("Failed to initialize analysis engine")
            .map_err(|e| UveddiError::config_error(&e.to_string(), "analysis engine"))
    }
}

// Remove Default implementation as async construction is now required
// Use AnalysisOrchestrator::new().await instead
