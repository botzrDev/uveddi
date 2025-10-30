//! Database default detection
//!
//! This module detects default database credentials, connection strings,
//! and other database-related default configurations.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use std::collections::HashSet;

/// Pattern for detecting database default values
pub struct DatabaseDefaultPattern {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub values: HashSet<String>,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}

/// Checker for database default security issues
pub struct DatabaseDefaultChecker {
    patterns: Vec<DatabaseDefaultPattern>,
}

impl DatabaseDefaultChecker {
    pub fn new(_config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let patterns = Self::build_database_patterns();
        Ok(Self { patterns })
    }

    pub fn check_line(&self, line: &str, line_number: usize) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Look for database-related keys
        if Self::contains_database_key(line) {
            for pattern in &self.patterns {
                for default_value in &pattern.values {
                    if Self::line_contains_value(line, default_value) {
                        let issue = ConfigIssue::new(
                            pattern.severity,
                            0.95, // High confidence for exact matches
                            pattern.name.clone(),
                            format!("{}: {}", pattern.description, default_value),
                        )
                        .with_location(line_number, 1)
                        .with_snippet(line.to_string())
                        .with_remediation(pattern.remediation.clone());

                        let mut final_issue = if let Some(cwe_id) = pattern.cwe_id {
                            issue.with_cwe(cwe_id)
                        } else {
                            issue
                        };

                        if let Some(ref owasp_cat) = pattern.owasp_category {
                            final_issue = final_issue.with_tag(owasp_cat.clone());
                        }

                        issues.push(final_issue);
                        break; // Only report one issue per line
                    }
                }
            }
        }

        issues
    }

    fn build_database_patterns() -> Vec<DatabaseDefaultPattern> {
        vec![
            DatabaseDefaultPattern {
                name: "Default Database Credentials".to_string(),
                description: "Default database username or password detected".to_string(),
                severity: ConfigSeverity::High,
                values: [
                    "sa",       // SQL Server default
                    "postgres", // PostgreSQL default
                    "mysql",    // MySQL common default
                    "oracle",   // Oracle default
                    "admin",    // Common admin user
                    "dba",      // Database admin
                    "scott",    // Oracle example user
                    "hr",       // Oracle example user
                    "root",     // MySQL root user
                    "user",     // Generic user
                    "guest",    // Guest user
                    "test",     // Test user
                    "demo",     // Demo user
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation:
                    "Create unique database usernames and strong passwords for each environment."
                        .to_string(),
                cwe_id: Some(798),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
            DatabaseDefaultPattern {
                name: "Default Database Name".to_string(),
                description: "Default or example database name detected".to_string(),
                severity: ConfigSeverity::Medium,
                values: [
                    "northwind",      // Example database
                    "sakila",         // MySQL example database
                    "chinook",        // SQLite example database
                    "adventureworks", // SQL Server example
                    "dvdrental",      // PostgreSQL example
                    "employees",      // Common example
                    "test",           // Test database
                    "demo",           // Demo database
                    "sample",         // Sample database
                    "example",        // Example database
                    "tutorial",       // Tutorial database
                    "training",       // Training database
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Use application-specific database names instead of examples."
                    .to_string(),
                cwe_id: Some(200),
                owasp_category: Some(
                    "A09:2021 - Security Logging and Monitoring Failures".to_string(),
                ),
            },
            DatabaseDefaultPattern {
                name: "Default Connection String".to_string(),
                description: "Default or example connection string detected".to_string(),
                severity: ConfigSeverity::High,
                values: [
                    "localhost",
                    "127.0.0.1",
                    "example.com",
                    "test.db",
                    "sample.db",
                    "demo.db",
                    "database.db",
                    "data.db",
                    "app.db",
                    "main.db",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Use environment-specific database connection details.".to_string(),
                cwe_id: Some(200),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },
            DatabaseDefaultPattern {
                name: "Default Database Port".to_string(),
                description: "Database using well-known default port".to_string(),
                severity: ConfigSeverity::Low,
                values: [
                    "3306",  // MySQL
                    "5432",  // PostgreSQL
                    "1433",  // SQL Server
                    "1521",  // Oracle
                    "27017", // MongoDB
                    "6379",  // Redis
                    "5984",  // CouchDB
                    "9042",  // Cassandra
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Consider using non-default ports to reduce attack surface."
                    .to_string(),
                cwe_id: Some(1188),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },
            DatabaseDefaultPattern {
                name: "Default Schema/Table Names".to_string(),
                description: "Default or example schema/table names detected".to_string(),
                severity: ConfigSeverity::Low,
                values: [
                    "public",             // Default PostgreSQL schema
                    "dbo",                // Default SQL Server schema
                    "sys",                // System schema
                    "information_schema", // Standard schema
                    "users",              // Common table name
                    "customers",          // Example table
                    "orders",             // Example table
                    "products",           // Example table
                    "test_table",         // Test table
                    "sample_table",       // Sample table
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                remediation: "Use application-specific schema and table names.".to_string(),
                cwe_id: Some(200),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },
        ]
    }

    fn contains_database_key(line: &str) -> bool {
        let line_lower = line.to_lowercase();
        let database_keys = [
            "database",
            "db",
            "sql",
            "mysql",
            "postgres",
            "oracle",
            "mongo",
            "redis",
            "cassandra",
            "couchdb",
            "user",
            "username",
            "password",
            "host",
            "port",
            "connection",
            "server",
            "schema",
            "table",
        ];

        database_keys.iter().any(|&key| line_lower.contains(key))
    }

    fn line_contains_value(line: &str, value: &str) -> bool {
        let line_lower = line.to_lowercase();
        let value_lower = value.to_lowercase();

        // Check for exact matches with common delimiters
        let patterns = [
            format!("\"{}\"", value_lower), // "value"
            format!("'{}'", value_lower),   // 'value'
            format!(": {}", value_lower),   // : value
            format!("= {}", value_lower),   // = value
            format!(":{}", value_lower),    // :value
            format!("={}", value_lower),    // =value
            format!("//{}", value_lower),   // //value (in URLs)
            format!("@{}", value_lower),    // @value (in connection strings)
        ];

        patterns.iter().any(|pattern| line_lower.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_database_user_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseDefaultChecker::new(&config).unwrap();

        let line = "db_username: sa";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Database Credentials")));
    }

    #[test]
    fn test_default_database_name_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseDefaultChecker::new(&config).unwrap();

        let line = "database_name: \"northwind\"";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Database Name")));
    }

    #[test]
    fn test_localhost_connection_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseDefaultChecker::new(&config).unwrap();

        let line = "db_host: localhost";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Connection String")));
    }

    #[test]
    fn test_default_port_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseDefaultChecker::new(&config).unwrap();

        let line = "mysql_port: 3306";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Database Port")));
    }

    #[test]
    fn test_non_database_line_ignored() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseDefaultChecker::new(&config).unwrap();

        let line = "timeout: 30";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_custom_database_config_no_issues() {
        let config = ConfigSecurityConfig::default();
        let checker = DatabaseDefaultChecker::new(&config).unwrap();

        let line = "db_user: myapp_prod_user";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }
}
