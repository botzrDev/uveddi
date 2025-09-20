//! Credential scanning utilities for structured data

use super::validator;
use crate::analysis::AnalysisError;
use super::super::super::types::{ConfigIssue, ConfigSeverity};

/// Analyze structured credentials in YAML/JSON content
pub fn analyze_structured_credentials(content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
    let mut issues = Vec::new();

    // Look for structured credential patterns
    // e.g., username/password pairs, connection objects
    if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(content) {
        issues.extend(analyze_yaml_credentials(&yaml_value)?);
    }

    Ok(issues)
}

fn analyze_yaml_credentials(value: &serde_yaml::Value) -> Result<Vec<ConfigIssue>, AnalysisError> {
    let mut issues = Vec::new();

    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    // Check for credential-related keys
                    if is_credential_key(key_str) {
                        if let Some(val_str) = val.as_str() {
                            if validator::looks_like_credential(val_str) {
                                issues.push(ConfigIssue::new(
                                    ConfigSeverity::High,
                                    0.8,
                                    "Structured Credential Found",
                                    format!("Found credential in structured configuration: {}", key_str),
                                )
                                .with_tag("structured-credential")
                                .with_cwe(798));
                            }
                        }
                    }
                }

                // Recursively check nested structures
                issues.extend(analyze_yaml_credentials(val)?);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                issues.extend(analyze_yaml_credentials(item)?);
            }
        }
        _ => {}
    }

    Ok(issues)
}

fn is_credential_key(key: &str) -> bool {
    let credential_keys = [
        "password", "passwd", "pwd", "secret", "key", "token", "auth",
        "credential", "private_key", "api_key", "access_key", "secret_key"
    ];

    let key_lower = key.to_lowercase();
    credential_keys.iter().any(|&cred_key| key_lower.contains(cred_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_credential_key() {
        assert!(is_credential_key("password"));
        assert!(is_credential_key("api_key"));
        assert!(is_credential_key("database_password"));
        assert!(!is_credential_key("username"));
        assert!(!is_credential_key("host"));
    }

    #[test]
    fn test_structured_credential_analysis() {
        let content = r#"
database:
  host: localhost
  password: secret123
  username: admin
"#;

        let issues = analyze_structured_credentials(content).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("Structured Credential")));
    }
}