//! API key pattern detection

use super::SecretPattern;
use super::super::super::types::ConfigSeverity;
use super::super::utils;
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Build patterns for API key detection
pub fn build_api_key_patterns() -> Result<HashMap<String, SecretPattern>, AnalysisError> {
    let mut patterns = HashMap::new();

    // AWS Credentials
    patterns.insert("aws_access_key".to_string(), SecretPattern {
        name: "AWS Access Key ID".to_string(),
        regex: utils::compile_pattern(r"(?i)(aws_access_key_id|AKIA[0-9A-Z]{16})")?,
        severity: ConfigSeverity::Critical,
        confidence: 0.95,
        description: "AWS Access Key ID detected in configuration".to_string(),
        remediation: "Use AWS IAM roles or environment variables for AWS credentials".to_string(),
        cwe_id: Some(798),
        tags: vec!["aws".to_string(), "cloud".to_string(), "credential".to_string()],
    });

    patterns.insert("aws_secret_key".to_string(), SecretPattern {
        name: "AWS Secret Access Key".to_string(),
        regex: utils::compile_pattern(r"(?i)(aws_secret_access_key|[A-Za-z0-9/+=]{40})")?,
        severity: ConfigSeverity::Critical,
        confidence: 0.90,
        description: "AWS Secret Access Key detected in configuration".to_string(),
        remediation: "Use AWS IAM roles or AWS Secrets Manager for credentials".to_string(),
        cwe_id: Some(798),
        tags: vec!["aws".to_string(), "cloud".to_string(), "credential".to_string()],
    });

    // Generic API Keys
    patterns.insert("api_key".to_string(), SecretPattern {
        name: "API Key".to_string(),
        regex: utils::compile_pattern(r#"(?i)(api[_-]?key|secret[_-]?key)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_-]{20,})['"]?"#)?,
        severity: ConfigSeverity::High,
        confidence: 0.85,
        description: "API key detected in configuration".to_string(),
        remediation: "Store API keys in environment variables or secure secret management".to_string(),
        cwe_id: Some(798),
        tags: vec!["api-key".to_string(), "credential".to_string()],
    });

    // Slack Tokens
    patterns.insert("slack_token".to_string(), SecretPattern {
        name: "Slack Token".to_string(),
        regex: utils::compile_pattern(r"xox[baprs]-[0-9]{12}-[0-9]{12}-[a-zA-Z0-9]{24}")?,
        severity: ConfigSeverity::High,
        confidence: 0.95,
        description: "Slack API token detected in configuration".to_string(),
        remediation: "Store Slack tokens in environment variables or secure token management".to_string(),
        cwe_id: Some(798),
        tags: vec!["slack".to_string(), "token".to_string(), "api".to_string()],
    });

    // GitHub Tokens
    patterns.insert("github_token".to_string(), SecretPattern {
        name: "GitHub Token".to_string(),
        regex: utils::compile_pattern(r"(?i)(github[_-]?token|gh[ps]_[a-zA-Z0-9]{36})")?,
        severity: ConfigSeverity::High,
        confidence: 0.90,
        description: "GitHub access token detected in configuration".to_string(),
        remediation: "Use GitHub Apps or store tokens in secure secret management".to_string(),
        cwe_id: Some(798),
        tags: vec!["github".to_string(), "token".to_string(), "git".to_string()],
    });

    Ok(patterns)
}