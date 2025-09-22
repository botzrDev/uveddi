//! Modular SQL injection detector components.
//!
//! This module exposes the public API for the SQL injection detector while the
//! implementation details live in the specialized submodules. Consumers should
//! interact with [`SqlInjectionDetector`] and the associated configuration types.

pub mod config;
pub mod detector;
pub mod detection;
pub mod language_support;
pub mod patterns;
pub mod sanitizers;
pub mod types;

pub use config::SqlInjectionConfig;
pub use detector::SqlInjectionDetector;
pub use types::{DetectionFinding, DetectionMetadata, SqlInjectionPattern, SqlInjectionType};
