use super::config::SqlInjectionDetectorConfig;
use super::detection;
use super::language_support;
use super::sanitizers;
use super::types::{
    DetectionContext, DetectionFinding, SanitizationStatus, SqlInjectionPattern, SqlInjectionType,
};
use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use async_trait::async_trait;
use std::path::PathBuf;

/// Modular SQL injection detector.
pub struct SqlInjectionDetector {
    config: SqlInjectionDetectorConfig,
}

impl SqlInjectionDetector {
    pub fn new() -> Self {
        Self::with_config(SqlInjectionDetectorConfig::default())
    }

    pub fn with_config(config: SqlInjectionDetectorConfig) -> Self {
        Self { config }
    }

    fn analyze_line(
        &self,
        context: &DetectionContext,
        patterns: &[SqlInjectionPattern],
    ) -> Vec<DetectionFinding> {
        let sanitization = if self.config.enable_sanitizer_validation {
            sanitizers::evaluate(context)
        } else {
            SanitizationStatus::default()
        };

        detection::analyze(context, patterns, &sanitization, &self.config)
    }

    fn build_vulnerability(
        &self,
        file_path: &PathBuf,
        finding: DetectionFinding,
    ) -> OwaspVulnerability {
        let mut location = SecurityLocation::new(
            file_path.clone(),
            finding.line_number as i32,
            finding.line_number as i32,
        );
        if let Some(column) = finding.column {
            let end_col = column + finding.match_length;
            location.start_column = Some(column as i32);
            location.end_column = Some(end_col as i32);
        }

        let adjusted_severity =
            self.adjust_severity(finding.pattern.severity, &finding.sanitization);
        let remediation =
            Self::generate_remediation(&finding.pattern.injection_type, &finding.pattern.context);

        let mut metadata = VulnerabilityMetadata::new();
        metadata.add_metadata("pattern_id".to_string(), finding.pattern.id.to_string());
        metadata.add_metadata("pattern".to_string(), finding.pattern.pattern.clone());
        metadata.add_metadata(
            "pattern_matched".to_string(),
            finding.pattern.pattern.clone(),
        );
        metadata.add_metadata("context".to_string(), finding.pattern.context.clone());
        metadata.add_metadata(
            "injection_type".to_string(),
            finding.pattern.injection_type.description().to_string(),
        );
        for (key, value) in finding.sanitization.to_metadata_pairs() {
            if value {
                metadata.add_metadata(key.to_string(), value.to_string());
            }
        }

        OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            format!(
                "SQL Injection: {}",
                finding.pattern.injection_type.description()
            ),
            finding.pattern.description.clone(),
            location,
        )
        .with_severity(adjusted_severity)
        .with_confidence(finding.pattern.confidence)
        .with_remediation(remediation)
        .with_metadata(metadata)
    }

    fn adjust_severity(
        &self,
        severity: SecuritySeverity,
        sanitization: &SanitizationStatus,
    ) -> SecuritySeverity {
        if !sanitization.any() {
            return severity;
        }

        match severity {
            SecuritySeverity::Critical => SecuritySeverity::High,
            SecuritySeverity::High => SecuritySeverity::Medium,
            SecuritySeverity::Medium => SecuritySeverity::Low,
            SecuritySeverity::Low | SecuritySeverity::Info => severity,
        }
    }

    fn generate_remediation(injection_type: &SqlInjectionType, context: &str) -> String {
        let base_advice = match injection_type {
            SqlInjectionType::Classical => "Use parameterized queries or prepared statements instead of string concatenation or formatting.",
            SqlInjectionType::NoSqlInjection => "Use the driver/ORM query builder APIs and avoid dynamic string construction.",
            SqlInjectionType::Union => "Ensure queries do not concatenate user-influenced UNION clauses; prefer parameterized queries.",
            SqlInjectionType::Blind | SqlInjectionType::Boolean => "Remove boolean-based conditions influenced by user input and enforce strict validation.",
            SqlInjectionType::TimeDelayed => "Disallow execution of time-delay primitives from user-controlled inputs.",
            SqlInjectionType::StoredProcedure => "Use stored procedure parameters and validate all inputs before invocation.",
        };

        let lower_context = context.to_lowercase();
        let context_advice = if lower_context.contains("template") {
            " Prefer parameterized APIs over template literals."
        } else if lower_context.contains("concatenation") {
            " Avoid string concatenation when building SQL statements."
        } else if lower_context.contains("orm") {
            " Use ORM-safe methods for filtering and parameter binding."
        } else {
            ""
        };

        format!(
            "{}{} Consider introducing centralized sanitization helpers if not already present.",
            base_advice, context_advice
        )
    }
}

#[async_trait]
impl OwaspCategoryDetector for SqlInjectionDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let patterns = language_support::patterns_for(&file.language);
        if patterns.is_empty() {
            return Ok(Vec::new());
        }

        let mut vulnerabilities = Vec::new();
        for (index, line) in file.content.lines().enumerate() {
            let context =
                DetectionContext::new(file.file_path.as_path(), file.language, line, index + 1);

            for finding in self.analyze_line(&context, &patterns) {
                let vulnerability = self.build_vulnerability(&file.file_path, finding);
                vulnerabilities.push(vulnerability);
            }
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

    fn build_file(language: SourceLanguage, content: &str) -> ParsedFile {
        ParsedFile {
            file_path: Arc::new(PathBuf::from("test")),
            language,
            content: content.to_string(),
            tree: None,
        }
    }

    #[tokio::test]
    async fn detects_rust_sql_injection() {
        let detector = SqlInjectionDetector::new();
        let file = build_file(
            SourceLanguage::Rust,
            r#"
            let query = format!("SELECT * FROM users WHERE id = {}", user_id);
            sqlx::query(&format!("DELETE FROM accounts WHERE name = '{}'", name));
        "#,
        );

        let vulns = detector.detect(&file).await.unwrap();
        assert!(vulns.iter().any(|v| v.title.contains("SQL Injection")));
        assert!(vulns.iter().any(|v| v.description.contains("SELECT")));
    }

    #[tokio::test]
    async fn detects_python_sql_injection() {
        let detector = SqlInjectionDetector::new();
        let file = build_file(
            SourceLanguage::Python,
            r#"
            cursor.execute(f"SELECT * FROM users WHERE name = '{user_name}'")
            session.execute(text(f"UPDATE accounts SET balance = {balance}"))
        "#,
        );

        let vulns = detector.detect(&file).await.unwrap();
        assert!(vulns.len() >= 2);
        assert!(vulns.iter().all(|v| v.severity >= SecuritySeverity::High));
    }

    #[tokio::test]
    async fn detects_nosql_injection() {
        let detector = SqlInjectionDetector::new();
        let file = build_file(
            SourceLanguage::JavaScript,
            r#"
            db.users.find({ $where: user_input });
            sequelize.query(`SELECT * FROM users WHERE role = ${role}`);
        "#,
        );

        let vulns = detector.detect(&file).await.unwrap();
        assert!(vulns.iter().any(|v| v
            .metadata
            .get_metadata("injection_type")
            .unwrap()
            .contains("NoSQL")));
    }

    #[test]
    fn remediation_includes_parameterization_advice() {
        let advice = SqlInjectionDetector::generate_remediation(
            &SqlInjectionType::Classical,
            "String concatenation",
        );
        assert!(advice.contains("parameterized"));
        assert!(advice.contains("centralized"));
    }
}
