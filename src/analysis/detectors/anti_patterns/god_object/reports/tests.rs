//! Tests for report formatting

#[cfg(test)]
mod tests {
    use super::super::GodObjectReportFormatter;
    use crate::database::models::ArchitecturalIssue;

    fn create_test_issue(severity: &str, file_path: &str, description: &str) -> ArchitecturalIssue {
        let mut issue = ArchitecturalIssue::new(
            1,
            1,
            file_path.to_string(),
            Some(10),
            description.to_string(),
            "GodObjectDetector".to_string(),
            severity.to_string(),
            description.to_string(),
        );
        issue.start_line = Some(10);
        issue.end_line = Some(50);
        issue
    }

    #[test]
    fn test_severity_distribution() {
        let issues = vec![
            create_test_issue("Critical", "test1.rs", "Test issue 1"),
            create_test_issue("High", "test2.rs", "Test issue 2"),
            create_test_issue("Critical", "test3.rs", "Test issue 3"),
        ];

        let distribution = GodObjectReportFormatter::calculate_severity_distribution(&issues);
        assert_eq!(distribution.get("Critical"), Some(&2));
        assert_eq!(distribution.get("High"), Some(&1));
    }

    #[test]
    fn test_class_name_extraction() {
        let description = "God Object detected: 'UserManager' has 15 methods and 10 fields.";
        let class_name = super::utils::extract_class_name(description);
        assert_eq!(class_name, "UserManager");
    }

    #[test]
    fn test_metrics_extraction() {
        let description = "God Object detected: 'UserManager' has 15 methods and 10 fields. LCOM4 score: 2";
        let metrics = super::utils::extract_metrics_from_description(description);

        assert_eq!(metrics["method_count"], 15);
        assert_eq!(metrics["field_count"], 10);
        assert_eq!(metrics["lcom4_score"], 2);
    }

    #[test]
    fn test_empty_issues_markdown() {
        let issues = vec![];
        let markdown = GodObjectReportFormatter::format_as_markdown(&issues);
        assert!(markdown.contains("No God Objects detected"));
    }

    #[test]
    fn test_json_formatting() {
        let issues = vec![
            create_test_issue("Critical", "test.rs", "Test issue"),
        ];

        let json_result = GodObjectReportFormatter::format_as_json(&issues);
        assert!(json_result.is_ok());

        let json_string = json_result.unwrap();
        assert!(json_string.contains("GodObjectDetector"));
        assert!(json_string.contains("total_issues"));
    }
}