//! Cryptographic Detector Configuration

use serde::{Deserialize, Serialize};

/// Configuration for the crypto detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub enable_algorithm_analysis: bool,
    pub enable_implementation_security: bool,
    pub enable_vulnerability_detection: bool,
    pub enable_entropy_analysis: bool,
    pub confidence_threshold: f64,
    pub check_weak_algorithms: bool,
    pub check_deprecated_functions: bool,
    pub check_hardcoded_keys: bool,
    pub check_weak_random: bool,
    pub check_certificate_validation: bool,
    pub check_tls_configuration: bool,
    pub check_key_management: bool,
    pub check_padding_attacks: bool,
    pub check_timing_attacks: bool,
    pub check_side_channel: bool,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl CryptoConfig {
    pub fn minimal() -> Self {
        Self {
            enable_implementation_security: false,
            enable_vulnerability_detection: false,
            enable_entropy_analysis: false,
            confidence_threshold: 0.8,
            ..Default::default()
        }
    }

    pub fn comprehensive() -> Self {
        Self {
            confidence_threshold: 0.5,
            ..Default::default()
        }
    }
}