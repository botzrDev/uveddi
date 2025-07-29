//! Anti-Pattern Knowledge Modules
//!
//! This module contains the comprehensive anti-pattern knowledge base with both
//! universal patterns and language-specific variations for AI-powered analysis.

pub mod language_specific;
pub mod universal;

// Re-export key functionality
pub use language_specific::create_language_specific_libraries;
pub use universal::create_universal_patterns;
