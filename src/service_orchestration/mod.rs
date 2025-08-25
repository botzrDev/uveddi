//! Service management module for Uveddi
//!
//! Manages the lifecycle of all background services including API servers,
//! rendering services, and database connections.

pub mod orchestrator;

pub use orchestrator::{
    start_default_services, start_development_services, OrchestratorConfig, ServiceOrchestrator,
};
