//! Database security patterns for YAML/JSON configuration
//!
//! This module contains security patterns specific to database
//! configurations in YAML and JSON files.

use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::utils;
use super::parser::YamlParser;
use serde_yaml::Value as YamlValue;

/// Database security pattern checker
pub struct DatabasePatternChecker;

impl DatabasePatternChecker {
    /// Check for database security issues
    pub fn check_database_security(value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();
        Self::check_database_recursive(value, "", &mut issues);
        issues
    }

    fn check_database_recursive(value: &YamlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                // Check if this looks like a database configuration
                let is_db_config = map.keys().any(|k| {
                    if let Some(key_str) = k.as_str() {
                        let key_lower = key_str.to_lowercase();
                        key_lower.contains("database") || key_lower.contains("db") ||
                        key_lower == "host" || key_lower == "port" || key_lower == "username"
                    } else {
                        false
                    }
                });

                if is_db_config {
                    // Check for SSL disabled
                    if let Some(ssl_val) = YamlParser::get_mapping_value(map, &["ssl", "ssl_enabled", "use_ssl"]) {
                        if ssl_val.as_bool() == Some(false) {
                            issues.push(utils::create_config_issue(
                                ConfigSeverity::High,
                                "Database SSL Disabled",
                                "Database connection does not use SSL encryption",
                                None,
                                "Enable SSL/TLS for database connections",
                                vec!["database".to_string(), "encryption".to_string()],
                            ).with_cwe(319));
                        }
                    }

                    // Check for default ports
                    if let Some(port_val) = YamlParser::get_mapping_value(map, &["port"]) {
                        if let Some(port_num) = port_val.as_u64() {
                            let default_ports = [3306, 5432, 1433, 27017, 6379]; // MySQL, PostgreSQL, SQL Server, MongoDB, Redis
                            if default_ports.contains(&(port_num as u16)) {
                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::Low,
                                    "Default Database Port",
                                    format!("Database using default port: {}", port_num),
                                    None,
                                    "Consider using non-default ports to reduce attack surface",
                                    vec!["database".to_string(), "default-config".to_string()],
                                ).with_cwe(1188));
                            }
                        }
                    }

                    // Check for missing connection limits
                    if YamlParser::get_mapping_value(map, &["max_connections", "connection_limit", "pool_size"]).is_none() {
                        issues.push(utils::create_config_issue(
                            ConfigSeverity::Medium,
                            "Missing Connection Limits",
                            "Database configuration lacks connection limits",
                            None,
                            "Set appropriate connection limits to prevent resource exhaustion",
                            vec!["database".to_string(), "resource-management".to_string()],
                        ).with_cwe(770));
                    }

                    // Check for weak authentication methods
                    if let Some(auth_method) = YamlParser::get_mapping_value(map, &["auth_method", "authentication", "auth"]) {
                        if let Some(auth_str) = auth_method.as_str() {
                            let auth_lower = auth_str.to_lowercase();
                            if auth_lower.contains("trust") || auth_lower.contains("none") {
                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::Critical,
                                    "Weak Database Authentication",
                                    format!("Weak authentication method: {}", auth_str),
                                    None,
                                    "Use strong authentication methods like password, certificate, or LDAP",
                                    vec!["database".to_string(), "authentication".to_string()],
                                ).with_cwe(287));
                            }
                        }
                    }

                    // Check for backup encryption
                    if let Some(backup_config) = YamlParser::get_mapping_value(map, &["backup", "backups"]) {
                        if let YamlValue::Mapping(backup_map) = backup_config {
                            if let Some(encryption) = YamlParser::get_mapping_value(backup_map, &["encryption", "encrypt"]) {
                                if encryption.as_bool() == Some(false) {
                                    issues.push(utils::create_config_issue(
                                        ConfigSeverity::High,
                                        "Unencrypted Database Backups",
                                        "Database backups are not encrypted",
                                        None,
                                        "Enable encryption for database backups",
                                        vec!["database".to_string(), "backup".to_string(), "encryption".to_string()],
                                    ).with_cwe(311));
                                }
                            }
                        }
                    }

                    // Check for logging of sensitive data
                    if let Some(logging_config) = YamlParser::get_mapping_value(map, &["logging", "log_config"]) {
                        if let YamlValue::Mapping(log_map) = logging_config {
                            if let Some(log_statements) = YamlParser::get_mapping_value(log_map, &["log_statements", "log_queries"]) {
                                if log_statements.as_bool() == Some(true) {
                                    issues.push(utils::create_config_issue(
                                        ConfigSeverity::Medium,
                                        "Database Query Logging Enabled",
                                        "Database configured to log all queries which may expose sensitive data",
                                        None,
                                        "Disable query logging or ensure logs are properly secured",
                                        vec!["database".to_string(), "logging".to_string(), "privacy".to_string()],
                                    ).with_cwe(532));
                                }
                            }
                        }
                    }
                }

                // Recurse into nested structures
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
                        let new_path = if path.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", path, key_str)
                        };
                        Self::check_database_recursive(val, &new_path, issues);
                    }
                }
            }
            YamlValue::Sequence(seq) => {
                for (index, item) in seq.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, index);
                    Self::check_database_recursive(item, &new_path, issues);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::parser::YamlParser;

    #[test]
    fn test_ssl_disabled_detection() {
        let yaml_content = r#"
database:
  host: localhost
  port: 5432
  ssl: false
"#;
        let parsed = YamlParser::parse_content(yaml_content).unwrap();
        let issues = DatabasePatternChecker::check_database_security(&parsed);
        assert!(issues.iter().any(|i| i.title.contains("SSL Disabled")));
    }

    #[test]
    fn test_default_port_detection() {
        let yaml_content = r#"
mysql:
  host: localhost
  port: 3306
  username: admin
"#;
        let parsed = YamlParser::parse_content(yaml_content).unwrap();
        let issues = DatabasePatternChecker::check_database_security(&parsed);
        assert!(issues.iter().any(|i| i.title.contains("Default Database Port")));
    }

    #[test]
    fn test_weak_authentication() {
        let yaml_content = r#"
database:
  host: localhost
  auth_method: trust
"#;
        let parsed = YamlParser::parse_content(yaml_content).unwrap();
        let issues = DatabasePatternChecker::check_database_security(&parsed);
        assert!(issues.iter().any(|i| i.title.contains("Weak Database Authentication")));
    }

    #[test]
    fn test_unencrypted_backups() {
        let yaml_content = r#"
database:
  host: localhost
  backup:
    enabled: true
    encryption: false
"#;
        let parsed = YamlParser::parse_content(yaml_content).unwrap();
        let issues = DatabasePatternChecker::check_database_security(&parsed);
        assert!(issues.iter().any(|i| i.title.contains("Unencrypted Database Backups")));
    }
}