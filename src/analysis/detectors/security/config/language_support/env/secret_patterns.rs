//! Secret pattern detection for environment variables
//!
//! This module contains patterns for detecting various types of secrets
//! and credentials in environment variable files.

use crate::analysis::AnalysisError;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::super::config::ConfigSecurityConfig;
use super::super::utils;
use regex::Regex;

/// Pattern for detecting secrets in environment variables
pub struct EnvSecretPattern {
    pub name: String,
    pub pattern: Regex,
    pub severity: ConfigSeverity,
    pub description: String,
    pub cwe_id: Option<u32>,
}

/// Secret pattern checker for environment variables
pub struct EnvSecretPatternChecker {
    patterns: Vec<EnvSecretPattern>,
    config: ConfigSecurityConfig,
}

impl EnvSecretPatternChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let patterns = Self::build_secret_patterns()?;
        Ok(Self {
            patterns,
            config: config.clone(),
        })
    }

    pub fn check_variable(&self, key: &str, value: &str, line_number: usize) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        let full_line = format!("{}={}", key, value);
        for pattern in &self.patterns {
            if pattern.pattern.is_match(&full_line) {
                let confidence = self.calculate_confidence(key, value);

                if confidence >= self.config.confidence_threshold {
                    let issue = utils::create_config_issue(
                        pattern.severity,
                        format!("Environment Variable: {}", pattern.name),
                        pattern.description.clone(),
                        Some(line_number),
                        "Store sensitive values in secure secret management systems",
                        vec!["environment".to_string(), "credential".to_string()],
                    );

                    let final_issue = if let Some(cwe_id) = pattern.cwe_id {
                        issue.with_cwe(cwe_id)
                    } else {
                        issue.with_cwe(798)
                    };

                    issues.push(final_issue);
                }
            }
        }

        issues
    }

    fn build_secret_patterns() -> Result<Vec<EnvSecretPattern>, AnalysisError> {
        let patterns = vec![
            EnvSecretPattern {
                name: "AWS Access Key".to_string(),
                pattern: Regex::new(r"^AWS_ACCESS_KEY.*=.*AKIA[0-9A-Z]{16}")?,
                severity: ConfigSeverity::Critical,
                description: "AWS access key found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "AWS Secret Key".to_string(),
                pattern: Regex::new(r"^AWS_SECRET.*=.*[A-Za-z0-9/+=]{40}")?,
                severity: ConfigSeverity::Critical,
                description: "AWS secret key found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "Database Password".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*PASSWORD.*=.*[^\s]{6,}")?,
                severity: ConfigSeverity::High,
                description: "Database password found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "API Key".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*API[_-]?KEY.*=.*[A-Za-z0-9_-]{20,}")?,
                severity: ConfigSeverity::High,
                description: "API key found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "JWT Secret".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*JWT[_-]?SECRET.*=.*[A-Za-z0-9_+-=]{20,}")?,
                severity: ConfigSeverity::High,
                description: "JWT secret found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "Private Key".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*PRIVATE[_-]?KEY.*=.*-----BEGIN.*PRIVATE KEY-----")?,
                severity: ConfigSeverity::Critical,
                description: "Private key found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "Database URL".to_string(),
                pattern: Regex::new(r"^DATABASE_URL.*=.*(postgres|mysql|mongodb)://[^/]*:[^@]*@")?,
                severity: ConfigSeverity::Critical,
                description: "Database URL with credentials found".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "GitHub Token".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*GITHUB[_-]?TOKEN.*=.*gh[ps]_[A-Za-z0-9]{36}")?,
                severity: ConfigSeverity::High,
                description: "GitHub token found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "Slack Token".to_string(),
                pattern: Regex::new(r"^SLACK[_-]?TOKEN.*=.*xox[baprs]-[0-9]{12}-[0-9]{12}-[a-zA-Z0-9]{24}")?,
                severity: ConfigSeverity::High,
                description: "Slack token found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "Discord Bot Token".to_string(),
                pattern: Regex::new(r"^DISCORD[_-]?TOKEN.*=.*[MN][A-Za-z\d]{23}\.[\w-]{6}\.[\w-]{27}")?,
                severity: ConfigSeverity::High,
                description: "Discord bot token found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "Firebase Key".to_string(),
                pattern: Regex::new(r"^FIREBASE[_-]?KEY.*=.*AIza[0-9A-Za-z-_]{35}")?,
                severity: ConfigSeverity::High,
                description: "Firebase API key found in environment variable".to_string(),
                cwe_id: Some(798),
            },

            EnvSecretPattern {
                name: "Stripe Key".to_string(),
                pattern: Regex::new(r"^STRIPE[_-]?KEY.*=.*(sk|pk)_(test|live)_[0-9a-zA-Z]{24}")?,
                severity: ConfigSeverity::High,
                description: "Stripe API key found in environment variable".to_string(),
                cwe_id: Some(798),
            },
        ];

        Ok(patterns)
    }

    fn calculate_confidence(&self, key: &str, value: &str) -> f64 {
        let mut confidence = 0.8;

        // Higher confidence for keys that clearly indicate secrets
        let secret_indicators = ["secret", "key", "token", "password", "pwd", "pass"];
        if secret_indicators.iter().any(|&indicator| key.to_lowercase().contains(indicator)) {
            confidence += 0.1;
        }

        // Lower confidence for obvious test/example values
        let test_indicators = ["test", "example", "demo", "placeholder", "xxx", "***"];
        if test_indicators.iter().any(|&indicator| value.to_lowercase().contains(indicator)) {
            confidence -= 0.3;
        }

        // Lower confidence for very short values
        if value.len() < 8 {
            confidence -= 0.2;
        }

        // Higher confidence for values with good entropy
        if Self::has_good_entropy(value) {
            confidence += 0.1;
        }

        confidence.max(0.0).min(1.0)
    }

    fn has_good_entropy(value: &str) -> bool {
        if value.len() < 10 {
            return false;
        }

        let mut char_counts = std::collections::HashMap::new();
        for ch in value.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        let total_chars = value.len() as f64;
        let entropy: f64 = char_counts.values()
            .map(|&count| {
                let p = count as f64 / total_chars;
                -p * p.log2()
            })
            .sum();

        entropy > 3.0 // Threshold for good entropy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_key_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = EnvSecretPatternChecker::new(&config).unwrap();

        let issues = checker.check_variable("AWS_ACCESS_KEY_ID", "AKIAIOSFODNN7EXAMPLE", 1);
        assert!(!issues.is_empty());
        assert!(issues[0].title.contains("AWS Access Key"));
    }

    #[test]
    fn test_database_password_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = EnvSecretPatternChecker::new(&config).unwrap();

        let issues = checker.check_variable("DB_PASSWORD", "supersecret123", 1);
        assert!(!issues.is_empty());
        assert!(issues[0].title.contains("Database Password"));
    }

    #[test]
    fn test_api_key_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = EnvSecretPatternChecker::new(&config).unwrap();

        let issues = checker.check_variable("API_KEY", "sk-1234567890abcdefghij", 1);
        assert!(!issues.is_empty());
        assert!(issues[0].title.contains("API Key"));
    }

    #[test]
    fn test_confidence_calculation() {
        let config = ConfigSecurityConfig::default();
        let checker = EnvSecretPatternChecker::new(&config).unwrap();

        // Test value should have lower confidence
        let low_conf = checker.calculate_confidence("API_KEY", "test123");
        assert!(low_conf < 0.7);

        // Real-looking value should have higher confidence
        let high_conf = checker.calculate_confidence("API_KEY", "sk-1234567890abcdefghij");
        assert!(high_conf > 0.8);
    }

    #[test]
    fn test_entropy_calculation() {
        assert!(EnvSecretPatternChecker::has_good_entropy("Xy9$kL2mN8pQ"));
        assert!(!EnvSecretPatternChecker::has_good_entropy("aaaaaaaaaa"));
        assert!(!EnvSecretPatternChecker::has_good_entropy("short"));
    }
}