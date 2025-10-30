//! Comprehensive Cryptographic Security Detector Module
//!
//! This module provides a complete cryptographic security analysis framework that detects
//! weak algorithms, implementation vulnerabilities, and cryptographic misuse across
//! multiple programming languages.
//!
//! ## Architecture
//!
//! The crypto detector follows a modular architecture with specialized analyzers:
//!
//! - **Algorithm Analysis**: Detects weak, deprecated, or broken cryptographic algorithms
//! - **Implementation Security**: Analyzes TLS configuration, certificate validation, and key management
//! - **Vulnerability Detection**: Identifies timing attacks, side-channel vulnerabilities, and entropy issues
//! - **Language Support**: Provides language-specific analysis for Rust, Python, and JavaScript/TypeScript
//!
//! ## Usage
//!
//! ```rust
//! use uveddi::analysis::detectors::security::crypto::{CryptoDetector, CryptoConfig};
//!
//! // Create detector with default configuration
//! let detector = CryptoDetector::new()?;
//!
//! // Create detector with custom configuration
//! let config = CryptoConfig {
//!     confidence_threshold: 0.8,
//!     enable_algorithm_analysis: true,
//!     enable_implementation_security: true,
//!     ..Default::default()
//! };
//! let detector = CryptoDetector::with_config(config)?;
//!
//! // Analyze a file
//! let vulnerabilities = detector.detect(&parsed_file).await?;
//! ```

pub mod algorithms;
pub mod implementations;
pub mod vulnerabilities;
pub mod language_support;
pub mod config;
pub mod types;
pub mod detector;

// Re-export main components
pub use detector::{CryptoDetector, AnalysisStats, SecurityPosture};
pub use config::CryptoConfig;
pub use types::{
    CryptoFinding, CryptoFindingType, AlgorithmType, SecurityLevel,
    ImplementationIssueType, VulnerabilityType, AttackVector,
    CryptoMetadata, Priority, FixEffort,
};

// Re-export specialized analyzers
pub use algorithms::{
    AlgorithmAnalyzer, SymmetricAnalyzer, AsymmetricAnalyzer,
    HashingAnalyzer, KeyDerivationAnalyzer, RandomGenerationAnalyzer,
};
pub use implementations::{
    ImplementationAnalyzer, TlsAnalyzer, CertificateAnalyzer,
    KeyManagementAnalyzer, PaddingAnalyzer, ModeAnalyzer,
};
pub use vulnerabilities::{
    VulnerabilityAnalyzer, WeakCryptoDetector, TimingAttackDetector,
    SideChannelDetector, EntropyAnalyzer, CryptoMisuseDetector,
    VulnerabilityStats, SecurityAssessment,
};
pub use language_support::{
    LanguageAnalyzerCoordinator, RustCryptoAnalyzer,
    PythonCryptoAnalyzer, JavaScriptCryptoAnalyzer,
};

use crate::analysis::AnalysisError;

/// Initialize the crypto detector module
pub fn initialize() -> Result<(), AnalysisError> {
    // Perform any necessary initialization
    tracing::info!("Initializing cryptographic security detector module");

    // Validate that required dependencies are available
    validate_dependencies()?;

    tracing::info!("Cryptographic security detector module initialized successfully");
    Ok(())
}

/// Validate that required dependencies are available
fn validate_dependencies() -> Result<(), AnalysisError> {
    // Check for regex support
    if regex::Regex::new(r"test").is_err() {
        return Err(AnalysisError::initialization_error(
            "Regex support is required for crypto detection".to_string()
        ));
    }

    Ok(())
}

/// Get version information for the crypto detector
pub fn get_version_info() -> ModuleVersionInfo {
    ModuleVersionInfo {
        module_name: "crypto_detector".to_string(),
        version: "1.0.0".to_string(),
        supported_languages: vec![
            "Rust".to_string(),
            "Python".to_string(),
            "JavaScript".to_string(),
            "TypeScript".to_string(),
        ],
        detection_categories: vec![
            "Weak Algorithms".to_string(),
            "Implementation Security".to_string(),
            "Vulnerability Detection".to_string(),
            "Entropy Analysis".to_string(),
        ],
        compliance_frameworks: vec![
            "NIST Cybersecurity Framework".to_string(),
            "OWASP Top 10".to_string(),
            "FIPS 140-2".to_string(),
            "Common Criteria".to_string(),
        ],
    }
}

/// Version and capability information for the crypto detector module
#[derive(Debug, Clone)]
pub struct ModuleVersionInfo {
    pub module_name: String,
    pub version: String,
    pub supported_languages: Vec<String>,
    pub detection_categories: Vec<String>,
    pub compliance_frameworks: Vec<String>,
}

/// Create a crypto detector with recommended security settings
pub fn create_recommended_detector() -> Result<CryptoDetector, AnalysisError> {
    let config = CryptoConfig {
        enable_algorithm_analysis: true,
        enable_implementation_security: true,
        enable_vulnerability_detection: true,
        enable_entropy_analysis: true,
        confidence_threshold: 0.7,
        check_weak_algorithms: true,
        check_deprecated_functions: true,
        check_hardcoded_keys: true,
        check_weak_random: true,
        check_certificate_validation: true,
        check_tls_configuration: true,
        check_key_management: true,
        check_padding_attacks: true,
        check_timing_attacks: true,
        check_side_channel: true,
    };

    CryptoDetector::with_config(config)
}

/// Create a minimal crypto detector for performance-critical scenarios
pub fn create_minimal_detector() -> Result<CryptoDetector, AnalysisError> {
    CryptoDetector::with_config(CryptoConfig::minimal())
}

/// Create a comprehensive crypto detector for thorough security analysis
pub fn create_comprehensive_detector() -> Result<CryptoDetector, AnalysisError> {
    CryptoDetector::with_config(CryptoConfig::comprehensive())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_initialization() {
        assert!(initialize().is_ok());
    }

    #[test]
    fn test_dependency_validation() {
        assert!(validate_dependencies().is_ok());
    }

    #[test]
    fn test_version_info() {
        let version_info = get_version_info();
        assert_eq!(version_info.module_name, "crypto_detector");
        assert!(!version_info.supported_languages.is_empty());
        assert!(!version_info.detection_categories.is_empty());
    }

    #[test]
    fn test_recommended_detector_creation() {
        let detector = create_recommended_detector();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_minimal_detector_creation() {
        let detector = create_minimal_detector();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_comprehensive_detector_creation() {
        let detector = create_comprehensive_detector();
        assert!(detector.is_ok());
    }
}