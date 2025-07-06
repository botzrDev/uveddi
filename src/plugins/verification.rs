//! Plugin verification system for security and integrity

use crate::plugins::{errors::*, registry::PluginManifest, security::SecurityPolicy};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Plugin verification system
#[derive(Debug)]
pub struct PluginVerifier {
    static_analyzer: StaticAnalyzer,
    signature_verifier: SignatureVerifier,
    manifest_validator: ManifestValidator,
}

impl PluginVerifier {
    /// Create a new plugin verifier
    pub fn new() -> Self {
        Self {
            static_analyzer: StaticAnalyzer::new(),
            signature_verifier: SignatureVerifier::new(),
            manifest_validator: ManifestValidator::new(),
        }
    }
    
    /// Verify a plugin binary and manifest
    pub async fn verify_plugin(
        &self,
        binary: &[u8],
        manifest: &PluginManifest,
        policy: &SecurityPolicy,
    ) -> Result<VerificationReport, VerificationError> {
        let mut report = VerificationReport::new(binary, manifest);
        
        // 1. Static analysis
        report.static_analysis = self.static_analyzer.analyze(binary).await?;
        
        // 2. Signature verification (if required)
        if policy.require_code_signing {
            report.signature_verification = Some(
                self.signature_verifier.verify(binary, manifest).await?
            );
        }
        
        // 3. Manifest validation
        report.manifest_validation = self.manifest_validator.validate(manifest, policy)?;
        
        // 4. Capability audit
        report.capability_audit = self.audit_capabilities(manifest, policy)?;
        
        // 5. Overall assessment
        report.overall_status = self.assess_overall_status(&report);
        
        Ok(report)
    }
    
    /// Audit plugin capabilities against security policy
    fn audit_capabilities(
        &self,
        manifest: &PluginManifest,
        policy: &SecurityPolicy,
    ) -> Result<CapabilityAudit, VerificationError> {
        let mut audit = CapabilityAudit {
            requested_permissions: manifest.permissions.clone(),
            granted_permissions: Vec::new(),
            denied_permissions: Vec::new(),
            warnings: Vec::new(),
        };
        
        for permission in &manifest.permissions {
            if policy.has_permission(permission) {
                audit.granted_permissions.push(permission.clone());
            } else {
                audit.denied_permissions.push(permission.clone());
                audit.warnings.push(format!(
                    "Permission denied by policy: {:?}",
                    permission
                ));
            }
        }
        
        // Check for excessive permissions
        if audit.granted_permissions.len() > 10 {
            audit.warnings.push(
                "Plugin requests an unusually high number of permissions".to_string()
            );
        }
        
        Ok(audit)
    }
    
    /// Assess overall verification status
    fn assess_overall_status(&self, report: &VerificationReport) -> VerificationStatus {
        // Check for critical security issues
        if report.static_analysis.vulnerabilities.iter().any(|v| v.severity == VulnerabilitySeverity::Critical) {
            return VerificationStatus::Rejected("Critical vulnerabilities found".to_string());
        }
        
        // Check signature verification if present
        if let Some(ref sig_verification) = report.signature_verification {
            if !sig_verification.is_valid {
                return VerificationStatus::Rejected("Invalid signature".to_string());
            }
        }
        
        // Check manifest validation
        if !report.manifest_validation.is_valid {
            return VerificationStatus::Rejected("Invalid manifest".to_string());
        }
        
        // Check for high-severity vulnerabilities
        let high_vuln_count = report.static_analysis.vulnerabilities
            .iter()
            .filter(|v| v.severity == VulnerabilitySeverity::High)
            .count();
        
        if high_vuln_count > 0 {
            return VerificationStatus::Warning(format!(
                "{} high-severity vulnerabilities found",
                high_vuln_count
            ));
        }
        
        VerificationStatus::Approved
    }
}

impl Default for PluginVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Static analysis component
#[derive(Debug)]
pub struct StaticAnalyzer {
    // In a real implementation, this would integrate with tools like Wasmati
}

impl StaticAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Perform static analysis on WASM binary
    pub async fn analyze(&self, binary: &[u8]) -> Result<StaticAnalysisReport, VerificationError> {
        let mut report = StaticAnalysisReport {
            binary_hash: self.calculate_hash(binary),
            binary_size: binary.len(),
            vulnerabilities: Vec::new(),
            code_quality_issues: Vec::new(),
            complexity_metrics: ComplexityMetrics::default(),
        };
        
        // Basic WASM structure validation
        self.validate_wasm_structure(binary, &mut report)?;
        
        // Check for suspicious patterns
        self.check_suspicious_patterns(binary, &mut report)?;
        
        // Analyze imports/exports
        self.analyze_imports_exports(binary, &mut report)?;
        
        Ok(report)
    }
    
    fn calculate_hash(&self, binary: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(binary);
        format!("{:x}", hasher.finalize())
    }
    
    fn validate_wasm_structure(
        &self,
        binary: &[u8],
        report: &mut StaticAnalysisReport,
    ) -> Result<(), VerificationError> {
        // Check WASM magic number and version
        if binary.len() < 8 {
            report.vulnerabilities.push(Vulnerability {
                id: "WASM_001".to_string(),
                description: "Binary too small to be valid WASM".to_string(),
                severity: VulnerabilitySeverity::Critical,
                location: None,
            });
            return Ok(());
        }
        
        if &binary[0..4] != b"\0asm" {
            report.vulnerabilities.push(Vulnerability {
                id: "WASM_002".to_string(),
                description: "Invalid WASM magic number".to_string(),
                severity: VulnerabilitySeverity::Critical,
                location: Some("offset 0".to_string()),
            });
        }
        
        if &binary[4..8] != &[1, 0, 0, 0] {
            report.vulnerabilities.push(Vulnerability {
                id: "WASM_003".to_string(),
                description: "Unsupported WASM version".to_string(),
                severity: VulnerabilitySeverity::High,
                location: Some("offset 4".to_string()),
            });
        }
        
        Ok(())
    }
    
    fn check_suspicious_patterns(
        &self,
        binary: &[u8],
        report: &mut StaticAnalysisReport,
    ) -> Result<(), VerificationError> {
        // Check for suspicious string patterns
        let suspicious_strings = [
            b"eval",
            b"exec",
            b"system",
            b"shell",
            b"cmd",
            b"/bin/sh",
            b"powershell",
        ];
        
        for pattern in &suspicious_strings {
            if let Some(pos) = binary.windows(pattern.len())
                .position(|window| window == *pattern) {
                report.vulnerabilities.push(Vulnerability {
                    id: "SUSPICIOUS_001".to_string(),
                    description: format!("Suspicious string pattern found: {:?}", 
                                       std::str::from_utf8(pattern).unwrap_or("binary")),
                    severity: VulnerabilitySeverity::Medium,
                    location: Some(format!("offset {}", pos)),
                });
            }
        }
        
        // Check for excessive binary size
        if binary.len() > 100 * 1024 * 1024 {
            report.vulnerabilities.push(Vulnerability {
                id: "SIZE_001".to_string(),
                description: "Binary size is unusually large".to_string(),
                severity: VulnerabilitySeverity::Low,
                location: None,
            });
        }
        
        Ok(())
    }
    
    fn analyze_imports_exports(
        &self,
        _binary: &[u8],
        _report: &mut StaticAnalysisReport,
    ) -> Result<(), VerificationError> {
        // In a real implementation, this would parse WASM sections
        // and analyze the import/export tables for suspicious functions
        Ok(())
    }
}

/// Signature verification component
#[derive(Debug)]
pub struct SignatureVerifier {
    // In a real implementation, this would store trusted public keys
    trusted_keys: HashMap<String, Vec<u8>>,
}

impl SignatureVerifier {
    pub fn new() -> Self {
        Self {
            trusted_keys: HashMap::new(),
        }
    }
    
    /// Verify plugin signature
    pub async fn verify(
        &self,
        binary: &[u8],
        manifest: &PluginManifest,
    ) -> Result<SignatureVerification, VerificationError> {
        let mut verification = SignatureVerification {
            is_valid: false,
            signer: None,
            algorithm: None,
            timestamp: None,
            trust_chain: Vec::new(),
        };
        
        // Check if signature is present
        if let Some(ref signature) = manifest.signature {
            verification.algorithm = Some("ECDSA-SHA256".to_string());
            verification.signer = Some(manifest.author.clone());
            
            // In a real implementation, this would:
            // 1. Extract signature from manifest or binary
            // 2. Verify signature against binary hash
            // 3. Check certificate chain
            // 4. Validate timestamp
            
            // For now, we'll do a simple check
            verification.is_valid = self.verify_simple_signature(binary, signature)?;
        }
        
        Ok(verification)
    }
    
    fn verify_simple_signature(&self, binary: &[u8], signature: &str) -> Result<bool, VerificationError> {
        // Calculate binary hash
        let mut hasher = Sha256::new();
        hasher.update(binary);
        let hash = format!("{:x}", hasher.finalize());
        
        // In a real implementation, this would verify the signature
        // For now, we'll just check if the signature looks valid
        Ok(signature.len() >= 64 && signature.chars().all(|c| c.is_ascii_hexdigit()))
    }
    
    /// Add a trusted public key
    pub fn add_trusted_key(&mut self, identifier: String, key: Vec<u8>) {
        self.trusted_keys.insert(identifier, key);
    }
}

/// Manifest validation component
#[derive(Debug)]
pub struct ManifestValidator;

impl ManifestValidator {
    pub fn new() -> Self {
        Self
    }
    
    /// Validate plugin manifest
    pub fn validate(
        &self,
        manifest: &PluginManifest,
        policy: &SecurityPolicy,
    ) -> Result<ManifestValidation, VerificationError> {
        let mut validation = ManifestValidation {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Validate required fields
        if manifest.name.is_empty() {
            validation.errors.push("Plugin name is required".to_string());
            validation.is_valid = false;
        }
        
        if manifest.version.is_empty() {
            validation.errors.push("Plugin version is required".to_string());
            validation.is_valid = false;
        }
        
        if manifest.author.is_empty() {
            validation.errors.push("Plugin author is required".to_string());
            validation.is_valid = false;
        }
        
        // Validate version format (semver)
        if !self.is_valid_semver(&manifest.version) {
            validation.errors.push("Plugin version must be valid semver".to_string());
            validation.is_valid = false;
        }
        
        // Validate supported languages
        if manifest.supported_languages.is_empty() {
            validation.warnings.push("No supported languages specified".to_string());
        }
        
        // Validate anti-pattern types
        if manifest.anti_pattern_types.is_empty() {
            validation.warnings.push("No anti-pattern types specified".to_string());
        }
        
        // Check against policy
        if policy.require_code_signing && manifest.signature.is_none() {
            validation.errors.push("Code signing required but no signature found".to_string());
            validation.is_valid = false;
        }
        
        Ok(validation)
    }
    
    fn is_valid_semver(&self, version: &str) -> bool {
        // Simple semver validation
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        
        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }
}

/// Complete verification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub plugin_hash: String,
    pub plugin_name: String,
    pub plugin_version: String,
    pub verification_timestamp: chrono::DateTime<chrono::Utc>,
    pub static_analysis: StaticAnalysisReport,
    pub signature_verification: Option<SignatureVerification>,
    pub manifest_validation: ManifestValidation,
    pub capability_audit: CapabilityAudit,
    pub overall_status: VerificationStatus,
}

impl VerificationReport {
    fn new(binary: &[u8], manifest: &PluginManifest) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(binary);
        
        Self {
            plugin_hash: format!("{:x}", hasher.finalize()),
            plugin_name: manifest.name.clone(),
            plugin_version: manifest.version.clone(),
            verification_timestamp: chrono::Utc::now(),
            static_analysis: StaticAnalysisReport::default(),
            signature_verification: None,
            manifest_validation: ManifestValidation::default(),
            capability_audit: CapabilityAudit::default(),
            overall_status: VerificationStatus::Pending,
        }
    }
}

/// Static analysis report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StaticAnalysisReport {
    pub binary_hash: String,
    pub binary_size: usize,
    pub vulnerabilities: Vec<Vulnerability>,
    pub code_quality_issues: Vec<CodeQualityIssue>,
    pub complexity_metrics: ComplexityMetrics,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub description: String,
    pub severity: VulnerabilitySeverity,
    pub location: Option<String>,
}

/// Vulnerability severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Code quality issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityIssue {
    pub description: String,
    pub severity: String,
    pub location: Option<String>,
}

/// Code complexity metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: u32,
    pub function_count: u32,
    pub instruction_count: u32,
}

/// Signature verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub is_valid: bool,
    pub signer: Option<String>,
    pub algorithm: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub trust_chain: Vec<String>,
}

/// Manifest validation result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Capability audit result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityAudit {
    pub requested_permissions: Vec<crate::plugins::security::Permission>,
    pub granted_permissions: Vec<crate::plugins::security::Permission>,
    pub denied_permissions: Vec<crate::plugins::security::Permission>,
    pub warnings: Vec<String>,
}

/// Overall verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Approved,
    Warning(String),
    Rejected(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::security::Permission;
    use std::path::PathBuf;
    
    #[test]
    fn test_static_analyzer() {
        let analyzer = StaticAnalyzer::new();
        
        // Valid WASM binary
        let valid_binary = b"\0asm\x01\0\0\0";
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let report = runtime.block_on(analyzer.analyze(valid_binary)).unwrap();
        
        assert!(report.vulnerabilities.is_empty());
        assert_eq!(report.binary_size, 8);
    }
    
    #[test]
    fn test_manifest_validator() {
        let validator = ManifestValidator::new();
        let policy = SecurityPolicy::default();
        
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "Test plugin".to_string(),
            permissions: vec![Permission::Logging],
            supported_languages: vec!["rust".to_string()],
            anti_pattern_types: vec!["test-pattern".to_string()],
            signature: None,
        };
        
        let validation = validator.validate(&manifest, &policy).unwrap();
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
    }
    
    #[test]
    fn test_semver_validation() {
        let validator = ManifestValidator::new();
        
        assert!(validator.is_valid_semver("1.0.0"));
        assert!(validator.is_valid_semver("0.1.2"));
        assert!(!validator.is_valid_semver("1.0"));
        assert!(!validator.is_valid_semver("invalid"));
    }
}