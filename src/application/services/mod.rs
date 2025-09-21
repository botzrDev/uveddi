//! Service layer for dependency management and processing coordination
//!
//! This module provides service abstractions and dependency injection
//! patterns for managing analysis workflows and system resources.

pub mod file_processor;
pub mod progress_tracker;
pub mod registry;

pub use file_processor::FileProcessor;
pub use progress_tracker::{ProgressTracker, ProgressUpdate};
pub use registry::{ServiceBuilder, ServiceRegistry};

/// Common service traits and interfaces
pub mod traits {
    use crate::error::UveddiError;

    /// Trait for services that can be started and stopped
    pub trait Service: Send + Sync {
        /// Start the service
        async fn start(&mut self) -> Result<(), UveddiError>;

        /// Stop the service
        async fn stop(&mut self) -> Result<(), UveddiError>;

        /// Check if the service is running
        fn is_running(&self) -> bool;

        /// Get the service name
        fn name(&self) -> &str;
    }

    /// Trait for services that can be configured
    pub trait Configurable<T> {
        /// Configure the service with the given configuration
        fn configure(&mut self, config: T) -> Result<(), UveddiError>;

        /// Get the current configuration
        fn get_config(&self) -> Option<&T>;
    }

    /// Trait for services that provide health checks
    pub trait HealthCheck: Send + Sync {
        /// Check the health of the service
        async fn health_check(&self) -> Result<ServiceHealth, UveddiError>;
    }

    /// Service health status
    #[derive(Debug, Clone, PartialEq)]
    pub enum ServiceHealth {
        Healthy,
        Degraded(String),
        Unhealthy(String),
    }
}
