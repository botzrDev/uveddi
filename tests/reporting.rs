//! Reporting output and summary tests

#[cfg(test)]
mod tests {
    use super::*;
    use codeatlas::report::ReportGenerator;
    use codeatlas::database::models::{AnalysisRun, ArchitecturalIssue};
    use chrono::Utc;

    #[test]
    fn markdown_report_output() {
        let analysis_run = AnalysisRun {
            run_id: Some(1),
            project_id: 1,
            start_time: Utc::now(),
            end_time: None,
            status: "completed".to_string(),
            total_files_analyzed: Some(1),
            total_issues_found: Some(1),
            analysis_config: "{}".to_string(),
        };
        let issues = vec![ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "high".to_string(),
            description: "God Object with too many methods".to_string(),
            code_snippet: Some("struct GodObject { ... }".to_string()),
            ai_explanation: Some("This is a God Object because...".to_string()),
        }];
        let generator = ReportGenerator::new();
        let report = generator.generate_markdown_report(&analysis_run, &issues).unwrap();
        assert!(report.contains("God Object"));
        assert!(report.contains("AI Analysis"));
        assert!(report.contains("src/main.rs"));
        assert!(report.contains("struct GodObject"));
    }

    #[test]
    fn json_report_output() {
        // TODO: Test JSON report output
        unimplemented!();
    }

    #[test]
    fn report_summary_statistics() {
        // TODO: Test summary statistics in reports
        unimplemented!();
    }

    #[test]
    fn report_includes_code_and_ai_context() {
        // TODO: Test report includes code and AI context
        unimplemented!();
    }
}
