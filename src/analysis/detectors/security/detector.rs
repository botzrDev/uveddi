//! Main security detector implementation
//!
//! This module provides the concrete implementation of the SecurityDetector
//! that integrates with Uveddi's existing AnalysisDetector trait system.

use crate::analysis::detectors::security::{
    MainSecurityDetector, SecurityConfig
};

// Re-export the main detector as SecurityDetector for convenience
pub use crate::analysis::detectors::security::MainSecurityDetector as SecurityDetector;

impl SecurityDetector {
    /// Create a new security detector with minimal configuration for development
    pub fn minimal() -> Result<Self, crate::analysis::AnalysisError> {
        let config = SecurityConfig::development();
        Self::with_config(config)
    }

    /// Create a new security detector optimized for CI/CD environments
    pub fn for_ci_cd() -> Result<Self, crate::analysis::AnalysisError> {
        let config = SecurityConfig::ci_cd();
        Self::with_config(config)
    }

    /// Create a new security detector with production settings
    pub fn for_production() -> Result<Self, crate::analysis::AnalysisError> {
        let config = SecurityConfig::production();
        Self::with_config(config)
    }
}