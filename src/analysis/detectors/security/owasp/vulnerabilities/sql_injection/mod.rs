//! Modular SQL injection vulnerability detector.
//!
//! This module exposes the public interface for the SQL injection detector
//! after refactoring the original monolithic implementation into specialized
//! components for detection, pattern management, sanitization validation, and
//! language-specific analysis.

pub mod config;
pub mod detection;
pub mod detector;
pub mod language_support;
pub mod patterns;
pub mod sanitizers;
pub mod types;

pub use config::SqlInjectionDetectorConfig;
pub use detector::SqlInjectionDetector;
pub use types::{
    DetectionContext, DetectionFinding, PatternCategory, SanitizationStatus, SqlInjectionPattern,
    SqlInjectionType,
};
