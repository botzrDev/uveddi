//! Core Types for Cryptographic Analysis
//!
//! This module defines the core data structures used throughout the crypto detector.

use crate::analysis::detectors::security::types::SecuritySeverity;
use serde::{Deserialize, Serialize};

/// A cryptographic finding from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoFinding {
    /// Type of finding
    pub finding_type: CryptoFindingType,

    /// Line number where the issue was found
    pub line_number: usize,

    /// Column number where the issue was found
    pub column: usize,

    /// Description of the issue
    pub description: String,

    /// Severity level
    pub severity: SecuritySeverity,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,

    /// Algorithm or component involved
    pub algorithm: Option<String>,

    /// Recommendation for fixing the issue
    pub recommendation: String,

    /// CWE identifier if applicable
    pub cwe_id: Option<u32>,
}

/// Types of crypto findings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CryptoFindingType {
    /// Weak or broken algorithm detected
    WeakAlgorithm,

    /// Deprecated function usage
    DeprecatedFunction,

    /// Hardcoded cryptographic key
    HardcodedKey,

    /// Weak random number generation
    WeakRandom,

    /// Insecure TLS configuration
    InsecureTls,

    /// Invalid certificate validation
    InvalidCertValidation,

    /// Weak key management
    WeakKeyManagement,

    /// Timing attack vulnerability
    TimingVulnerability,

    /// Side-channel information leak
    SideChannelLeak,

    /// Entropy-related issue
    EntropyIssue,
}

/// Algorithm-specific pattern matching
#[derive(Debug, Clone)]
pub struct AlgorithmPattern {
    pub name: String,
    pub pattern: String,
    pub algorithm_type: AlgorithmType,
    pub security_level: SecurityLevel,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub deprecated: bool,
    pub replacements: Vec<String>,
}

/// Types of cryptographic algorithms
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlgorithmType {
    /// Symmetric encryption algorithms
    SymmetricEncryption,

    /// Asymmetric encryption algorithms
    AsymmetricEncryption,

    /// Hash functions
    HashFunction,

    /// Key derivation functions
    KeyDerivation,

    /// Message authentication codes
    MessageAuthentication,

    /// Digital signature algorithms
    DigitalSignature,

    /// Random number generators
    RandomGeneration,
}

/// Security levels for algorithms
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Cryptographically secure
    Secure,

    /// Weak but potentially acceptable in some contexts
    Weak,

    /// Cryptographically broken
    Broken,

    /// Deprecated but not necessarily broken
    Deprecated,

    /// Unknown security status
    Unknown,
}

/// Implementation security issue types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplementationIssueType {
    /// Weak TLS configuration
    WeakTlsConfiguration,

    /// Invalid certificate validation
    InvalidCertificateValidation,

    /// Insecure key management
    InsecureKeyManagement,

    /// Weak padding scheme
    WeakPaddingScheme,

    /// Insecure encryption mode
    InsecureMode,

    /// Hardcoded secrets
    HardcodedSecrets,

    /// Weak randomness
    WeakRandomness,

    /// Timing vulnerability
    TimingVulnerability,

    /// Side-channel leak
    SideChannelLeak,
}

/// Cryptographic vulnerability types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VulnerabilityType {
    /// Weak cryptography usage
    WeakCryptography,

    /// Timing attack vulnerability
    TimingAttack,

    /// Side-channel attack vulnerability
    SideChannelAttack,

    /// Entropy weakness
    EntropyWeakness,

    /// Cryptographic misuse
    CryptoMisuse,

    /// Key management failure
    KeyManagementFailure,

    /// Certificate validation failure
    CertificateValidationFailure,

    /// TLS misconfiguration
    TlsMisconfiguration,

    /// Padding oracle vulnerability
    PaddingOracle,

    /// Hash collision vulnerability
    HashCollision,
}

/// Attack vectors for crypto vulnerabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttackVector {
    /// Remote network attack
    Remote,

    /// Local system attack
    Local,

    /// Adjacent network attack
    Adjacent,

    /// Physical access attack
    Physical,

    /// Unknown attack vector
    Unknown,
}

/// Metadata for crypto findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoMetadata {
    /// Category of the crypto issue
    pub category: String,

    /// Subcategory for more specific classification
    pub subcategory: Option<String>,

    /// NIST framework alignment
    pub nist_category: Option<String>,

    /// OWASP category alignment
    pub owasp_category: Option<String>,

    /// Compliance frameworks affected
    pub compliance_impact: Vec<String>,

    /// Remediation priority
    pub priority: Priority,

    /// Estimated fix effort
    pub fix_effort: FixEffort,
}

/// Priority levels for remediation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    /// Immediate attention required
    Critical,

    /// High priority
    High,

    /// Medium priority
    Medium,

    /// Low priority
    Low,
}

/// Estimated effort required to fix an issue
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FixEffort {
    /// Minimal effort (configuration change)
    Minimal,

    /// Low effort (simple code change)
    Low,

    /// Medium effort (moderate refactoring)
    Medium,

    /// High effort (significant redesign)
    High,

    /// Very high effort (architectural changes)
    VeryHigh,
}

impl CryptoFinding {
    /// Create a new crypto finding
    pub fn new(
        finding_type: CryptoFindingType,
        line_number: usize,
        description: String,
        severity: SecuritySeverity,
        confidence: f64,
    ) -> Self {
        Self {
            finding_type,
            line_number,
            column: 0,
            description,
            severity,
            confidence,
            algorithm: None,
            recommendation: String::new(),
            cwe_id: None,
        }
    }

    /// Add algorithm information
    pub fn with_algorithm(mut self, algorithm: String) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    /// Add recommendation
    pub fn with_recommendation(mut self, recommendation: String) -> Self {
        self.recommendation = recommendation;
        self
    }

    /// Add CWE identifier
    pub fn with_cwe_id(mut self, cwe_id: u32) -> Self {
        self.cwe_id = Some(cwe_id);
        self
    }

    /// Add column information
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = column;
        self
    }

    /// Check if this finding meets a confidence threshold
    pub fn meets_confidence_threshold(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }

    /// Get a risk score for this finding
    pub fn get_risk_score(&self) -> f64 {
        let severity_weight = match self.severity {
            SecuritySeverity::Critical => 1.0,
            SecuritySeverity::High => 0.8,
            SecuritySeverity::Medium => 0.6,
            SecuritySeverity::Low => 0.4,
        };

        severity_weight * self.confidence
    }
}