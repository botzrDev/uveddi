//! Main SQL injection detector that orchestrates specialized analyzers.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::sql_injection::config::SqlInjectionConfig;
use crate::analysis::detectors::security::sql_injection::detection::{
    DetectionContext, DetectionPipeline,
};
use crate::analysis::detectors::security::sql_injection::language_support;
use crate::analysis::detectors::security::sql_injection::types::{
    DetectionFinding, SanitizerStatus, SqlInjectionType,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use async_trait::async_trait;
pub struct SqlInjectionDetector {
    config: SqlInjectionConfig,
    pipeline: DetectionPipeline,
}

impl SqlInjectionDetector {
    pub fn new() -> Self {
        Self::with_config(SqlInjectionConfig::default())
    }

    pub fn with_config(config: SqlInjectionConfig) -> Self {
        let pipeline = DetectionPipeline::new(config.clone());

        Self { config, pipeline }
    }

    fn build_vulnerability(
        &self,
        finding: DetectionFinding,
        file: &ParsedFile,
    ) -> OwaspVulnerability {
        let DetectionFinding {
            pattern,
            metadata,
            sanitizer,
        } = finding;

        let mut location = SecurityLocation::new(
            file.file_path.to_path_buf(),
            metadata.line_number,
            metadata.column as u32,
        );

        let mut metadata_map = VulnerabilityMetadata::new();
        metadata_map.add_metadata(
            "injection_type".to_string(),
            pattern.injection_type.description().to_string(),
        );
        metadata_map.add_metadata("context".to_string(), pattern.context.to_string());
        metadata_map.add_metadata("pattern_matched".to_string(), pattern.regex.to_string());
        metadata_map.add_metadata(
            "sanitizer_status".to_string(),
            sanitizer_label(sanitizer).to_string(),
        );
        metadata_map.add_metadata("line_excerpt".to_string(), metadata.line_excerpt);

        OwaspVulnerability::new(
            OwaspCategory::Injection,
            SecurityIssueType::Injection,
            format!("SQL Injection: {}", pattern.injection_type.description()),
            pattern.description.to_string(),
            location,
        )
        .with_severity(pattern.severity)
        .with_confidence(pattern.confidence)
        .with_remediation(generate_remediation(
            &pattern.injection_type,
            pattern.context,
        ))
        .with_metadata(metadata_map)
    }
}

fn sanitizer_label(status: SanitizerStatus) -> &'static str {
    match status {
        SanitizerStatus::Unknown => "unknown",
        SanitizerStatus::Sanitized => "sanitized",
        SanitizerStatus::Unsanitized => "unsanitized",
    }
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
        c if c.contains("string") => " Avoid string concatenation and formatting in SQL queries.",
        c if c.contains("template") => " Use parameterized queries instead of template literals.",
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

#[async_trait]
impl OwaspCategoryDetector for SqlInjectionDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let mut vulnerabilities = Vec::new();
        let patterns = language_support::patterns_for(file.language);

        if patterns.is_empty() {
            return Ok(vulnerabilities);
        }

        for (line_number, line) in file.source.lines().enumerate() {
            let context = DetectionContext {
                line,
                line_number: line_number + 1,
                language: file.language,
            };

            let findings = self.pipeline.analyze_line(context, &patterns);

            for finding in findings {
                vulnerabilities.push(self.build_vulnerability(finding, file));
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
    use std::path::PathBuf;
    use std::sync::Arc;

    fn parsed_file(language: crate::ast::SourceLanguage, content: &str) -> ParsedFile {
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
        let content = r#"
            let query = format!("SELECT * FROM users WHERE id = {}", user_id);
            let update = format!("UPDATE users SET name = '{}'", name);
        "#;

        let file = parsed_file(crate::ast::SourceLanguage::Rust, content);
        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 2);
    }

    #[tokio::test]
    async fn detects_python_sql_injection() {
        let detector = SqlInjectionDetector::new();
        let content = r#"
            cursor.execute(f"SELECT * FROM users WHERE name = '{user_name}'")
            cursor.execute("INSERT INTO log VALUES (%s)" % log_entry)
        "#;

        let file = parsed_file(crate::ast::SourceLanguage::Python, content);
        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 2);
        assert!(vulnerabilities
            .iter()
            .all(|v| matches!(v.severity, SecuritySeverity::Critical)));
    }

    #[tokio::test]
    async fn detects_nosql_injection_patterns() {
        let detector = SqlInjectionDetector::new();
        let content = r#"
            db.users.find({ $where: user_input });
            db.collection.aggregate([{ $match: dynamic_filter }]);
        "#;

        let file = parsed_file(crate::ast::SourceLanguage::JavaScript, content);
        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.iter().any(|v| v
            .metadata
            .get_metadata("injection_type")
            .unwrap()
            .contains("NoSQL")));
    }

    #[test]
    fn remediation_includes_parameterized_queries() {
        let remediation =
            generate_remediation(&SqlInjectionType::Classical, "String concatenation");
        assert!(remediation.contains("parameterized queries"));
    }
}
