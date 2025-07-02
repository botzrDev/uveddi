//! Python-specific anti-pattern detectors
//!
//! This module contains detectors for anti-patterns specific to Python code,
//! including data structure misuse, exception handling problems, OOP issues,
//! and performance-related anti-patterns.

pub mod data_structure_misuse;
pub mod exception_handling;
pub mod oop_issues;
pub mod performance_issues;

pub use data_structure_misuse::PyDataStructureMisuseDetector;
pub use exception_handling::PyExceptionHandlingDetector;
pub use oop_issues::PyOopIssuesDetector;
pub use performance_issues::PyPerformanceIssuesDetector;
