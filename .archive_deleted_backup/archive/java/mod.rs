//! Java-specific anti-pattern detectors
//!
//! This module contains detectors for anti-patterns specific to Java code,
//! including concurrency issues, OOP misuse, resource management problems,
//! and type system violations.

pub mod concurrency;
pub mod oop_issues;
pub mod resource_management;
pub mod type_system;

pub use concurrency::JavaConcurrencyDetector;
pub use oop_issues::JavaOopIssuesDetector;
pub use resource_management::JavaResourceManagementDetector;
pub use type_system::JavaTypeSystemDetector;
