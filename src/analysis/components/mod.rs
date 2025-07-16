//! Component-based architecture for the AnalysisEngine
//!
//! This module contains the trait definitions and implementations for the
//! decomposed components of the AnalysisEngine, following the principles
//! outlined in UV-289.

pub mod traits;
pub mod config_service;
pub mod ast_provider;
pub mod plugin_manager;
pub mod analysis_aggregator;
pub mod dependency_graph_builder;
pub mod detector_scheduler;

#[cfg(test)]
pub mod tests;

// Re-export core traits for easier access
pub use traits::*;

// Re-export component implementations
pub use config_service::ConfigurationService;
pub use ast_provider::AstProviderImpl;
pub use plugin_manager::{PluginManager, PluginManagerHandle};
pub use analysis_aggregator::AnalysisAggregator;
pub use dependency_graph_builder::DependencyGraphBuilderImpl;
pub use detector_scheduler::DetectorScheduler;