//! Database configuration security checks
//!
//! This module provides specialized checks for database connection and
//! configuration security issues.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use serde_yaml::Value as YamlValue;

/// Checker for database-related misconfigurations
pub struct DatabaseChecker {
    config: ConfigSecurityConfig,
}

impl DatabaseChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check for insecure database configuration
    pub fn check_insecure_database_config(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    if key_str.to_lowercase().contains("database")
                        || key_str.to_lowercase().contains("db")
                    {
                        if let YamlValue::Mapping(db_config) = val {
                            issues.extend(self.check_db_security(db_config));
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    fn check_db_security(&self, db_config: &serde_yaml::Mapping) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for SSL disabled
        if let Some(ssl_val) = db_config.get(&YamlValue::String("ssl".to_string())) {
            if ssl_val.as_bool() == Some(false) {
                issues.push(
                    ConfigIssue::new(
                        ConfigSeverity::High,
                        0.85,
                        "Database SSL Disabled",
                        "Database connection does not use SSL encryption",
                    )
                    .with_tag("database-security")
                    .with_remediation("Enable SSL for database connections")
                    .with_cwe(319),
                );
            }
        }

        // Check for weak authentication
        if let Some(auth_val) = db_config.get(&YamlValue::String("auth_plugin".to_string())) {
            if let Some(auth_str) = auth_val.as_str() {
                if auth_str.to_lowercase().contains("native_password") {
                    issues.push(
                        ConfigIssue::new(
                            ConfigSeverity::Medium,
                            0.7,
                            "Weak Database Authentication",
                            "Database uses weak native password authentication",
                        )
                        .with_tag("database-security")
                        .with_remediation(
                            "Use stronger authentication methods like certificate-based auth",
                        ),
                    );
                }
            }
        }

        // Check for default ports
        if let Some(port_val) = db_config.get(&YamlValue::String("port".to_string())) {
            if let Some(port) = port_val.as_i64() {
                let default_ports = [3306, 5432, 1433, 27017, 6379]; // MySQL, PostgreSQL, SQL Server, MongoDB, Redis
                if default_ports.contains(&(port as u16)) {
                    issues.push(
                        ConfigIssue::new(
                            ConfigSeverity::Low,
                            0.5,
                            "Default Database Port",
                            format!("Database is using default port: {}", port),
                        )
                        .with_tag("database-security")
                        .with_remediation(
                            "Consider using non-default ports to reduce attack surface",
                        ),
                    );
                }
            }
        }

        issues
    }

    /// Check for database connection pooling issues
    pub fn check_connection_pooling(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    if key_str.to_lowercase().contains("pool") {
                        if let YamlValue::Mapping(pool_config) = val {
                            issues.extend(self.check_pool_config(pool_config));
                        }
                    }
                }
            }
        }

        issues
    }

    fn check_pool_config(&self, pool_config: &serde_yaml::Mapping) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for excessive max connections
        if let Some(max_val) = pool_config.get(&YamlValue::String("max_connections".to_string())) {
            if let Some(max_conn) = max_val.as_i64() {
                if max_conn > 1000 {
                    issues.push(
                        ConfigIssue::new(
                            ConfigSeverity::Medium,
                            0.6,
                            "Excessive Database Connections",
                            "Connection pool allows too many concurrent connections",
                        )
                        .with_tag("database-performance")
                        .with_remediation("Limit max connections to prevent resource exhaustion"),
                    );
                }
            }
        }

        // Check for missing connection timeouts
        if !pool_config.contains_key(&YamlValue::String("timeout".to_string())) {
            issues.push(
                ConfigIssue::new(
                    ConfigSeverity::Low,
                    0.4,
                    "Missing Connection Timeout",
                    "Database connection pool lacks timeout configuration",
                )
                .with_tag("database-reliability")
                .with_remediation("Configure connection timeouts to prevent hanging connections"),
            );
        }

        issues
    }

    /// Check for database backup and recovery configuration
    pub fn check_backup_config(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    if key_str.to_lowercase().contains("backup") {
                        if let YamlValue::Mapping(backup_config) = val {
                            issues.extend(self.check_backup_security(backup_config));
                        }
                    }
                }
            }
        }

        issues
    }

    fn check_backup_security(&self, backup_config: &serde_yaml::Mapping) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for unencrypted backups
        if let Some(encrypt_val) = backup_config.get(&YamlValue::String("encrypt".to_string())) {
            if encrypt_val.as_bool() == Some(false) {
                issues.push(
                    ConfigIssue::new(
                        ConfigSeverity::High,
                        0.8,
                        "Unencrypted Database Backups",
                        "Database backups are not encrypted",
                    )
                    .with_tag("backup-security")
                    .with_remediation("Enable backup encryption to protect sensitive data")
                    .with_cwe(311),
                );
            }
        }

        // Check for backup retention policy
        if !backup_config.contains_key(&YamlValue::String("retention_days".to_string())) {
            issues.push(
                ConfigIssue::new(
                    ConfigSeverity::Low,
                    0.3,
                    "Missing Backup Retention Policy",
                    "No backup retention policy configured",
                )
                .with_tag("backup-management")
                .with_remediation("Configure appropriate backup retention policies"),
            );
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_security() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseChecker::new(&config);

        let yaml_content = r#"
database:
  ssl: false
  port: 3306
  auth_plugin: mysql_native_password
"#;
        let value: YamlValue = serde_yaml::from_str(yaml_content).unwrap();
        let issues = checker.check_insecure_database_config(&value).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Database SSL Disabled")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Database Port")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Weak Database Authentication")));
    }

    #[test]
    fn test_connection_pooling() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseChecker::new(&config);

        let yaml_content = r#"
connection_pool:
  max_connections: 1500
"#;
        let value: YamlValue = serde_yaml::from_str(yaml_content).unwrap();
        let issues = checker.check_connection_pooling(&value);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Excessive Database Connections")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Missing Connection Timeout")));
    }

    #[test]
    fn test_backup_config() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseChecker::new(&config);

        let yaml_content = r#"
backup:
  encrypt: false
"#;
        let value: YamlValue = serde_yaml::from_str(yaml_content).unwrap();
        let issues = checker.check_backup_config(&value);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Unencrypted Database Backups")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Missing Backup Retention Policy")));
    }
}
