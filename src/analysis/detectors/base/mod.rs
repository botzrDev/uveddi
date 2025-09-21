//! Base detector framework
//!
//! This module provides the foundational traits and types for all detectors in the system.
//! It establishes a consistent interface and shared functionality across different detector types.

pub mod config;
pub mod metrics;
pub mod traits;
pub mod types;

pub use config::BaseConfig;
pub use metrics::BaseMetrics;
pub use traits::{Detector, DetectorConfig, DetectorOutput};
pub use types::{AnalysisContext, DetectionMetrics, DetectorCategory, Issue, Severity};
