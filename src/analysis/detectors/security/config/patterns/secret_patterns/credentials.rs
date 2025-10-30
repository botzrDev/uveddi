//! Password and username pattern detection

use super::super::super::types::ConfigSeverity;
use super::super::utils;
use super::SecretPattern;
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Build patterns for credential detection
pub fn build_credential_patterns() -> Result<HashMap<String, SecretPattern>, AnalysisError> {
    let mut patterns = HashMap::new();

    // Database Passwords
    patterns.insert(
        "database_password".to_string(),
        SecretPattern {
            name: "Database Password".to_string(),
            regex: utils::compile_pattern(
                r#"(?i)(password|passwd|pwd)[\s]*[:=][\s]*['"]?([^'\s\n]{6,})['"]?"#,
            )?,
            severity: ConfigSeverity::High,
            confidence: 0.80,
            description: "Database password detected in configuration".to_string(),
            remediation: "Use environment variables or database credential management systems"
                .to_string(),
            cwe_id: Some(798),
            tags: vec![
                "database".to_string(),
                "password".to_string(),
                "credential".to_string(),
            ],
        },
    );

    // JWT Secrets
    patterns.insert("jwt_secret".to_string(), SecretPattern {
        name: "JWT Secret".to_string(),
        regex: utils::compile_pattern(r#"(?i)(jwt[_-]?secret|token[_-]?secret)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_+-=]{20,})['"]?"#)?,
        severity: ConfigSeverity::High,
        confidence: 0.88,
        description: "JWT signing secret detected in configuration".to_string(),
        remediation: "Use cryptographically strong, random JWT secrets stored securely".to_string(),
        cwe_id: Some(798),
        tags: vec!["jwt".to_string(), "token".to_string(), "secret".to_string()],
    });

    // OAuth Tokens
    patterns.insert("oauth_token".to_string(), SecretPattern {
        name: "OAuth Token".to_string(),
        regex: utils::compile_pattern(r#"(?i)(access[_-]?token|bearer[_-]?token|oauth[_-]?token)[\s]*[:=][\s]*['"]?([a-zA-Z0-9_.-]{32,})['"]?"#)?,
        severity: ConfigSeverity::High,
        confidence: 0.85,
        description: "OAuth access token detected in configuration".to_string(),
        remediation: "Use OAuth refresh token flow and store tokens securely".to_string(),
        cwe_id: Some(798),
        tags: vec!["oauth".to_string(), "token".to_string(), "credential".to_string()],
    });

    // Database Connection Strings
    patterns.insert("db_connection_string".to_string(), SecretPattern {
        name: "Database Connection String".to_string(),
        regex: utils::compile_pattern(r"(?i)(mongodb|mysql|postgresql|postgres|mssql)://[^/]*:[^@]*@")?,
        severity: ConfigSeverity::Critical,
        confidence: 0.95,
        description: "Database connection string with embedded credentials detected".to_string(),
        remediation: "Use connection strings without embedded credentials and store credentials separately".to_string(),
        cwe_id: Some(798),
        tags: vec!["database".to_string(), "connection-string".to_string(), "credential".to_string()],
    });

    Ok(patterns)
}
