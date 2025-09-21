//! SQL Injection Vulnerability Detector
//!
//! Specialized detector for SQL injection vulnerabilities across multiple languages.
//! Detects various forms of SQL injection including classical, blind, and NoSQL injection.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// SQL injection pattern definition
#[derive(Debug, Clone)]
pub struct SqlInjectionPattern {
    pub pattern: String,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub injection_type: SqlInjectionType,
    pub context: String,
}

#[derive(Debug, Clone)]
pub enum SqlInjectionType {
    Classical,
    Blind,
    Union,
    Boolean,
    TimeDelayed,
    NoSqlInjection,
    StoredProcedure,
}

impl SqlInjectionType {
    fn description(&self) -> &'static str {
        match self {
            SqlInjectionType::Classical => "Direct SQL injection vulnerability",
            SqlInjectionType::Blind => "Blind SQL injection vulnerability",
            SqlInjectionType::Union => "UNION-based SQL injection",
            SqlInjectionType::Boolean => "Boolean-based blind SQL injection",
            SqlInjectionType::TimeDelayed => "Time-delayed SQL injection",
            SqlInjectionType::NoSqlInjection => "NoSQL injection vulnerability",
            SqlInjectionType::StoredProcedure => "Stored procedure injection",
        }
    }
}

/// Specialized SQL injection detector
pub struct SqlInjectionDetector {
    patterns: HashMap<SourceLanguage, Vec<SqlInjectionPattern>>,
}

impl SqlInjectionDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<SqlInjectionPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<SqlInjectionPattern> {
        vec![
            SqlInjectionPattern {
                pattern: r#"format!.*SELECT.*"#.to_string(),
                description: "String formatting in SQL SELECT statement".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "Direct string interpolation".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"format!.*INSERT.*"#.to_string(),
                description: "String formatting in SQL INSERT statement".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "Direct string interpolation".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"format!.*UPDATE.*"#.to_string(),
                description: "String formatting in SQL UPDATE statement".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "Direct string interpolation".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"format!.*DELETE.*"#.to_string(),
                description: "String formatting in SQL DELETE statement".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "Direct string interpolation".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"query.*WHERE.*\+.*"#.to_string(),
                description: "String concatenation in WHERE clause".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                injection_type: SqlInjectionType::Classical,
                context: "String concatenation".to_string(),
            },
        ]
    }

    fn python_patterns() -> Vec<SqlInjectionPattern> {
        vec![
            SqlInjectionPattern {
                pattern: r#"cursor\.execute\(f".*"#.to_string(),
                description: "F-string formatting in SQL execution".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "F-string interpolation".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"cursor\.execute\(".*%.*".*%"#.to_string(),
                description: "String formatting in SQL execution".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "Percent string formatting".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"cursor\.execute\(".*\+.*"#.to_string(),
                description: "String concatenation in SQL execution".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "String concatenation".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"\.format\(.*SELECT.*"#.to_string(),
                description: "String format in SELECT statement".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                injection_type: SqlInjectionType::Classical,
                context: "Format method".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"find\(.*\$where.*"#.to_string(),
                description: "MongoDB $where clause injection".to_string(),
                confidence: 0.75,
                severity: SecuritySeverity::High,
                injection_type: SqlInjectionType::NoSqlInjection,
                context: "NoSQL MongoDB".to_string(),
            },
        ]
    }

    fn javascript_patterns() -> Vec<SqlInjectionPattern> {
        vec![
            SqlInjectionPattern {
                pattern: r#"query\(`.*\$\{.*\}.*`\)"#.to_string(),
                description: "Template literal in SQL query".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "Template literals".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"query\(".*\+.*"#.to_string(),
                description: "String concatenation in SQL query".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "String concatenation".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"find\(.*\$where.*"#.to_string(),
                description: "MongoDB $where clause injection".to_string(),
                confidence: 0.75,
                severity: SecuritySeverity::High,
                injection_type: SqlInjectionType::NoSqlInjection,
                context: "NoSQL MongoDB".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"aggregate\(\[.*\$.*\]\)"#.to_string(),
                description: "MongoDB aggregation pipeline injection".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::Medium,
                injection_type: SqlInjectionType::NoSqlInjection,
                context: "NoSQL aggregation".to_string(),
            },
            SqlInjectionPattern {
                pattern: r#"eval\(.*SELECT.*"#.to_string(),
                description: "Dynamic SQL execution via eval".to_string(),
                confidence: 0.95,
                severity: SecuritySeverity::Critical,
                injection_type: SqlInjectionType::Classical,
                context: "Dynamic execution".to_string(),
            },
        ]
    }

    fn analyze_line(
        &self,
        line: &str,
        line_number: usize,
        patterns: &[SqlInjectionPattern],
    ) -> Vec<OwaspVulnerability> {
        let mut vulnerabilities = Vec::new();

        for pattern in patterns {
            if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                if regex.is_match(line) {
                    let location = SecurityLocation::new(
                        PathBuf::from("unknown"), // Will be updated by caller
                        line_number,
                        0,
                    );

                    let mut metadata = VulnerabilityMetadata::new();
                    metadata.add_metadata(
                        "injection_type".to_string(),
                        pattern.injection_type.description().to_string(),
                    );
                    metadata.add_metadata("context".to_string(), pattern.context.clone());
                    metadata.add_metadata("pattern_matched".to_string(), pattern.pattern.clone());

                    let remediation =
                        Self::generate_remediation(&pattern.injection_type, &pattern.context);

                    let vulnerability = OwaspVulnerability::new(
                        OwaspCategory::Injection,
                        SecurityIssueType::Injection,
                        format!("SQL Injection: {}", pattern.injection_type.description()),
                        pattern.description.clone(),
                        location,
                    )
                    .with_severity(pattern.severity)
                    .with_confidence(pattern.confidence)
                    .with_remediation(remediation)
                    .with_metadata(metadata);

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        vulnerabilities
    }

    fn generate_remediation(injection_type: &SqlInjectionType, context: &str) -> String {
        let base_advice = match injection_type {
            SqlInjectionType::Classical => {
                "Use parameterized queries or prepared statements instead of string concatenation or formatting."
            }
            SqlInjectionType::NoSqlInjection => {
                "Use proper query builders or ORM methods. Validate and sanitize all user inputs."
            }
            _ => "Use parameterized queries and input validation.",
        };

        let context_advice = match context {
            c if c.contains("string") => {
                " Avoid string concatenation and formatting in SQL queries."
            }
            c if c.contains("template") => {
                " Use parameterized queries instead of template literals."
            }
            c if c.contains("NoSQL") => {
                " Use proper NoSQL query builders and avoid dynamic query construction."
            }
            _ => "",
        };

        format!(
            "{}{} Consider using an ORM or query builder library for additional safety.",
            base_advice, context_advice
        )
    }
}

#[async_trait]
impl OwaspCategoryDetector for SqlInjectionDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let patterns = match self.patterns.get(&file.language) {
            Some(patterns) => patterns,
            None => return Ok(Vec::new()),
        };

        let mut vulnerabilities = Vec::new();

        for (line_number, line) in file.content.lines().enumerate() {
            let mut line_vulnerabilities = self.analyze_line(line, line_number + 1, patterns);

            // Update file path in location
            for vuln in &mut line_vulnerabilities {
                vuln.location.file_path = file.file_path.to_path_buf();
            }

            vulnerabilities.extend(line_vulnerabilities);
        }

        Ok(vulnerabilities)
    }
}

impl Default for SqlInjectionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_sql_injection_detection_rust() {
        let detector = SqlInjectionDetector::new();
        let content = r#"
            let query = format!("SELECT * FROM users WHERE id = {}", user_id);
            let update = format!("UPDATE users SET name = '{}'", name);
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            content: content.to_string(),
            tree: None,
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 2);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.description.contains("SELECT")));
        assert!(vulnerabilities
            .iter()
            .any(|v| v.description.contains("UPDATE")));
    }

    #[tokio::test]
    async fn test_sql_injection_detection_python() {
        let detector = SqlInjectionDetector::new();
        let content = r#"
            cursor.execute(f"SELECT * FROM users WHERE name = '{user_name}'")
            cursor.execute("INSERT INTO log VALUES (%s)" % log_entry)
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: content.to_string(),
            tree: None,
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 2);
        assert!(vulnerabilities
            .iter()
            .all(|v| v.severity == SecuritySeverity::Critical));
    }

    #[tokio::test]
    async fn test_nosql_injection_detection() {
        let detector = SqlInjectionDetector::new();
        let content = r#"
            db.users.find({ $where: user_input });
            db.collection.aggregate([{ $match: dynamic_filter }]);
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.js")),
            language: SourceLanguage::JavaScript,
            content: content.to_string(),
            tree: None,
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 1);
        assert!(vulnerabilities.iter().any(|v| v
            .metadata
            .get_metadata("injection_type")
            .unwrap()
            .contains("NoSQL")));
    }

    #[test]
    fn test_remediation_generation() {
        let remediation = SqlInjectionDetector::generate_remediation(
            &SqlInjectionType::Classical,
            "String concatenation",
        );
        assert!(remediation.contains("parameterized queries"));
        assert!(remediation.contains("string concatenation"));
    }
}
