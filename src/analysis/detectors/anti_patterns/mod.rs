//! Anti-pattern detectors
//!
//! This module contains detectors for various anti-patterns and code smells,
//! including God Objects, inappropriate inheritance, and other structural issues.

pub mod god_object;

pub use god_object::GodObjectDetector;
