//! Anti-Pattern Knowledge Modules
//!
//! This module contains the comprehensive anti-pattern knowledge base with both
//! universal patterns and language-specific variations for AI-powered analysis.

pub mod universal;
pub mod language_specific;

// Re-export key functionality
pub use universal::create_universal_patterns;
pub use language_specific::create_language_specific_libraries;