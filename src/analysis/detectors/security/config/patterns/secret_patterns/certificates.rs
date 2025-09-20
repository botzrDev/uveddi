//! Certificate and key pattern detection

use super::SecretPattern;
use super::super::super::types::ConfigSeverity;
use super::super::utils;
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Build patterns for certificate and private key detection
pub fn build_certificate_patterns() -> Result<HashMap<String, SecretPattern>, AnalysisError> {
    let mut patterns = HashMap::new();

    // Private Keys
    patterns.insert("private_key".to_string(), SecretPattern {
        name: "Private Key".to_string(),
        regex: utils::compile_pattern(r"-----BEGIN[A-Z\s]*PRIVATE KEY-----")?,
        severity: ConfigSeverity::Critical,
        confidence: 0.99,
        description: "Private key detected in configuration".to_string(),
        remediation: "Store private keys in secure key management systems, never in configuration files".to_string(),
        cwe_id: Some(798),
        tags: vec!["private-key".to_string(), "encryption".to_string(), "credential".to_string()],
    });

    Ok(patterns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_key_pattern() {
        let patterns = build_certificate_patterns().unwrap();
        let pattern = patterns.get("private_key").unwrap();

        let test_content = "-----BEGIN RSA PRIVATE KEY-----";
        assert!(pattern.regex.is_match(test_content));

        let test_content2 = "-----BEGIN PRIVATE KEY-----";
        assert!(pattern.regex.is_match(test_content2));
    }
}