//! Detection algorithms and analyzers
//!
//! This module contains all the detection logic for anti-patterns, architectural issues,
//! and other code quality problems. It includes both pattern-specific detectors and
//! infrastructure for dependency analysis.

pub mod anti_patterns;
pub mod base;
#[cfg(feature = "analysis-cache")]
pub mod cache_factory;
#[cfg(feature = "analysis-cache")]
pub mod cache_wrapper;
pub mod cycle;
pub mod dependency;
pub mod registry;
pub mod security;

pub use base::{
    AnalysisContext, BaseConfig, BaseMetrics, DetectionMetrics, Detector, DetectorCategory,
    DetectorConfig, DetectorOutput, Issue, Severity,
};
#[cfg(feature = "analysis-cache")]
pub use cache_factory::{
    CacheAwareDetectorFactory, CacheConfiguration, DetectorCacheSettings, DetectorMigrationHelper,
    MixedDetectorCollection,
};
#[cfg(feature = "analysis-cache")]
pub use cache_wrapper::CachedDetector;
pub use cycle::CycleDetector;
pub use dependency::{Dependency, DependencyExtractor};
pub use registry::{DetectorRegistry, DetectorRegistryFactory, RegistryConfig, RegistryResults};
pub use security::{MainSecurityDetector, SecurityConfig, SecurityDetector};
