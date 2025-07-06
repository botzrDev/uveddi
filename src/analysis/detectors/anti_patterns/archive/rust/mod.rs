//! Rust-specific anti-pattern detectors
//!
//! This module contains detectors for anti-patterns specific to Rust code,
//! including clone abuse, lifetime complexity, memory management issues,
//! and unsafe unwrap usage.

pub mod clone_abuse;
pub mod lifetime_complexity;
pub mod memory_management;
pub mod unwrap_abuse;

pub use clone_abuse::CloneAbuseDetector;
pub use lifetime_complexity::LifetimeComplexityDetector;
pub use memory_management::MemoryManagementDetector;
pub use unwrap_abuse::UnwrapAbuseDetector;
