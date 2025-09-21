//! Database security pattern analysis
//!
//! This module provides security analysis for database interactions
//! and data access patterns.

use std::collections::HashMap;

/// Analyzer for database security patterns and vulnerabilities
pub struct DatabaseSecurityAnalyzer {
    db_patterns: HashMap<String, DatabaseSecurityPattern>,
}

#[derive(Debug, Clone)]
pub struct DatabaseSecurityPattern {
    pub database_type: DatabaseType,
    pub security_concerns: Vec<String>,
    pub detection_patterns: Vec<String>,
    pub remediation_advice: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum DatabaseType {
    SQL,
    NoSQL,
    InMemory,
    GraphDatabase,
}

impl DatabaseSecurityAnalyzer {
    pub fn new() -> Self {
        let mut db_patterns = HashMap::new();

        // SQL database security patterns
        db_patterns.insert(
            "sql_database".to_string(),
            DatabaseSecurityPattern {
                database_type: DatabaseType::SQL,
                security_concerns: vec![
                    "SQL injection vulnerabilities".to_string(),
                    "Unencrypted database connections".to_string(),
                    "Excessive database privileges".to_string(),
                    "Missing input validation".to_string(),
                    "Unprotected database credentials".to_string(),
                ],
                detection_patterns: vec![
                    "SELECT".to_string(),
                    "INSERT".to_string(),
                    "UPDATE".to_string(),
                    "DELETE".to_string(),
                    "execute(".to_string(),
                    "query(".to_string(),
                ],
                remediation_advice: vec![
                    "Use parameterized queries and prepared statements".to_string(),
                    "Enable SSL/TLS for database connections".to_string(),
                    "Follow principle of least privilege for database users".to_string(),
                    "Validate and sanitize all database inputs".to_string(),
                    "Store database credentials securely".to_string(),
                ],
            },
        );

        // NoSQL database security patterns
        db_patterns.insert(
            "nosql_database".to_string(),
            DatabaseSecurityPattern {
                database_type: DatabaseType::NoSQL,
                security_concerns: vec![
                    "NoSQL injection attacks".to_string(),
                    "Unvalidated query objects".to_string(),
                    "Missing authentication".to_string(),
                    "Insecure data serialization".to_string(),
                ],
                detection_patterns: vec![
                    "mongodb".to_string(),
                    "redis".to_string(),
                    "cassandra".to_string(),
                    "dynamodb".to_string(),
                    "find(".to_string(),
                    "findOne(".to_string(),
                ],
                remediation_advice: vec![
                    "Validate NoSQL query structures".to_string(),
                    "Use type-safe query builders".to_string(),
                    "Implement proper authentication and authorization".to_string(),
                    "Validate data types before database operations".to_string(),
                ],
            },
        );

        // In-memory database security patterns
        db_patterns.insert(
            "inmemory_database".to_string(),
            DatabaseSecurityPattern {
                database_type: DatabaseType::InMemory,
                security_concerns: vec![
                    "Data persistence in memory dumps".to_string(),
                    "Lack of encryption at rest".to_string(),
                    "Missing access controls".to_string(),
                ],
                detection_patterns: vec![
                    "redis".to_string(),
                    "memcached".to_string(),
                    "sqlite::memory".to_string(),
                ],
                remediation_advice: vec![
                    "Implement memory encryption where possible".to_string(),
                    "Clear sensitive data from memory when no longer needed".to_string(),
                    "Implement proper access controls".to_string(),
                ],
            },
        );

        Self { db_patterns }
    }

    /// Detect database patterns in code content
    pub fn detect_database_patterns(&self, content: &str) -> Vec<String> {
        let mut detected_patterns = Vec::new();

        for (pattern_name, pattern) in &self.db_patterns {
            for detection_pattern in &pattern.detection_patterns {
                if content
                    .to_lowercase()
                    .contains(&detection_pattern.to_lowercase())
                {
                    detected_patterns.push(pattern_name.clone());
                    break;
                }
            }
        }

        detected_patterns
    }

    /// Analyze database security issues in code content
    pub fn analyze_database_security(&self, content: &str) -> Vec<DatabaseSecurityIssue> {
        let mut issues = Vec::new();

        // Check for SQL injection patterns
        if self.contains_sql_injection_pattern(content) {
            issues.push(DatabaseSecurityIssue {
                issue_type: "SQL Injection".to_string(),
                description: "Potential SQL injection vulnerability detected".to_string(),
                remediation: "Use parameterized queries or prepared statements".to_string(),
                severity: "Critical".to_string(),
            });
        }

        // Check for hardcoded database credentials
        if self.contains_hardcoded_credentials(content) {
            issues.push(DatabaseSecurityIssue {
                issue_type: "Hardcoded Credentials".to_string(),
                description: "Database credentials appear to be hardcoded".to_string(),
                remediation: "Store credentials in environment variables or secure configuration"
                    .to_string(),
                severity: "High".to_string(),
            });
        }

        // Check for unencrypted database connections
        if self.contains_unencrypted_connection(content) {
            issues.push(DatabaseSecurityIssue {
                issue_type: "Unencrypted Connection".to_string(),
                description: "Database connection may not be encrypted".to_string(),
                remediation: "Enable SSL/TLS for database connections".to_string(),
                severity: "Medium".to_string(),
            });
        }

        issues
    }

    fn contains_sql_injection_pattern(&self, content: &str) -> bool {
        let injection_patterns = [
            "format!(\"SELECT",
            "format!(\"INSERT",
            "format!(\"UPDATE",
            "format!(\"DELETE",
            "f\"SELECT",
            "f\"INSERT",
            "f\"UPDATE",
            "f\"DELETE",
            "+ \"SELECT",
            "+ \"INSERT",
            "+ \"UPDATE",
            "+ \"DELETE",
        ];

        injection_patterns
            .iter()
            .any(|pattern| content.contains(pattern))
    }

    fn contains_hardcoded_credentials(&self, content: &str) -> bool {
        let credential_patterns = [
            "password=",
            "pwd=",
            "user=",
            "username=",
            "DATABASE_URL=\"",
            "CONNECTION_STRING=\"",
        ];

        credential_patterns.iter().any(|pattern| {
            content.to_lowercase().contains(&pattern.to_lowercase())
                && !content.contains("env::var")
                && !content.contains("os.environ")
                && !content.contains("process.env")
        })
    }

    fn contains_unencrypted_connection(&self, content: &str) -> bool {
        content.contains("sslmode=disable")
            || content.contains("ssl=false")
            || (content.contains("mysql://") && !content.contains("ssl"))
            || (content.contains("postgres://") && !content.contains("ssl"))
    }

    /// Get security recommendations for a specific database type
    pub fn get_security_recommendations(&self, pattern_name: &str) -> Vec<String> {
        self.db_patterns
            .get(pattern_name)
            .map(|pattern| pattern.remediation_advice.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseSecurityIssue {
    pub issue_type: String,
    pub description: String,
    pub remediation: String,
    pub severity: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_pattern_detection() {
        let analyzer = DatabaseSecurityAnalyzer::new();

        let sql_content = "SELECT * FROM users WHERE id = ?";
        let patterns = analyzer.detect_database_patterns(sql_content);
        assert!(patterns.contains(&"sql_database".to_string()));

        let nosql_content = "db.users.find({name: userName})";
        let patterns = analyzer.detect_database_patterns(nosql_content);
        assert!(patterns.contains(&"nosql_database".to_string()));
    }

    #[test]
    fn test_sql_injection_detection() {
        let analyzer = DatabaseSecurityAnalyzer::new();

        let vulnerable_content = r#"query = format!("SELECT * FROM users WHERE id = {}", user_id)"#;
        let issues = analyzer.analyze_database_security(vulnerable_content);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == "SQL Injection"));

        let safe_content =
            "query = \"SELECT * FROM users WHERE id = ?\"; execute(query, [user_id])";
        let issues = analyzer.analyze_database_security(safe_content);
        assert!(issues.iter().all(|i| i.issue_type != "SQL Injection"));
    }

    #[test]
    fn test_hardcoded_credentials_detection() {
        let analyzer = DatabaseSecurityAnalyzer::new();

        let vulnerable_content = r#"CONNECTION_STRING="mysql://user:password@localhost/db""#;
        let issues = analyzer.analyze_database_security(vulnerable_content);
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "Hardcoded Credentials"));

        let safe_content = r#"connection_string = env::var("DATABASE_URL").unwrap()"#;
        let issues = analyzer.analyze_database_security(safe_content);
        assert!(issues
            .iter()
            .all(|i| i.issue_type != "Hardcoded Credentials"));
    }
}
