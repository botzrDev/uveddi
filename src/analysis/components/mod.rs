//! Component-based architecture for the AnalysisEngine
//!
//! This module contains the trait definitions and implementations for the
//! decomposed components of the AnalysisEngine, following the principles
//! outlined in UV-289.

pub mod analysis_aggregator;
pub mod ast_provider;
pub mod cache_manager;
pub mod config_service;
pub mod dependency_graph_builder;
pub mod detector_scheduler;
#[cfg(feature = "wasm-plugins")]
pub mod plugin_manager;
pub mod traits;

#[cfg(test)]
pub mod tests;

// Re-export core traits for easier access
pub use traits::*;

// Re-export component implementations
pub use analysis_aggregator::AnalysisAggregator;
pub use ast_provider::AstProviderImpl;
pub use cache_manager::{CacheManager, CacheManagerImpl, CacheStats};
pub use config_service::ConfigurationService;
pub use dependency_graph_builder::DependencyGraphBuilderImpl;
pub use detector_scheduler::DetectorScheduler;
#[cfg(feature = "wasm-plugins")]
pub use plugin_manager::{PluginManager, PluginManagerHandle};

// Aliases for acceptance criteria naming
pub use cache_manager::CacheManagerImpl as CacheManagerComponent;
pub use dependency_graph_builder::DependencyGraphBuilderImpl as DependencyAnalyzer;
pub use detector_scheduler::DetectorScheduler as DetectorManager;
