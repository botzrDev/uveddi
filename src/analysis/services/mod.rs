//! Analysis Services Module
//!
//! This module contains the decomposed services that replace the monolithic AnalysisEngine.
//! Each service has a single responsibility and can be tested and maintained independently.

pub mod analysis_service;
pub mod dependency_service;
pub mod performance_service;

pub use analysis_service::AnalysisService;
pub use dependency_service::DependencyAnalysisService;
pub use performance_service::PerformanceAnalysisService;

// Re-export common types used by services
pub use crate::analysis::errors::AnalysisError;
pub use crate::analysis::graph::dependency::LocalDependencyGraph;
pub use crate::database::models::ArchitecturalIssue;

/// Common result type for all analysis services
pub type AnalysisResult<T> = Result<T, AnalysisError>;

/// Service configuration trait for dependency injection
pub trait ServiceConfiguration: Send + Sync {
    /// Get the cache directory path
    fn cache_dir(&self) -> Option<&std::path::Path>;

    /// Get the maximum memory limit in bytes
    fn memory_limit(&self) -> Option<usize>;

    /// Get the number of parallel workers
    fn parallel_workers(&self) -> usize;
}

/// Default service configuration
#[derive(Debug, Clone)]
pub struct DefaultServiceConfig {
    pub cache_dir: Option<std::path::PathBuf>,
    pub memory_limit: Option<usize>,
    pub parallel_workers: usize,
}

impl Default for DefaultServiceConfig {
    fn default() -> Self {
        Self {
            cache_dir: None,
            memory_limit: Some(2 * 1024 * 1024 * 1024), // 2GB default
            parallel_workers: num_cpus::get(),
        }
    }
}

impl ServiceConfiguration for DefaultServiceConfig {
    fn cache_dir(&self) -> Option<&std::path::Path> {
        self.cache_dir.as_deref()
    }

    fn memory_limit(&self) -> Option<usize> {
        self.memory_limit
    }

    fn parallel_workers(&self) -> usize {
        self.parallel_workers
    }
}
