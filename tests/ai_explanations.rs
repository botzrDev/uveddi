//! AI explanation and prompt template tests

#[cfg(test)]
mod tests {
    use uveddi::ai::prompts::prompt_templates;
    use uveddi::database::models::ArchitecturalIssue;

    #[test]
    fn prompt_template_renders_correctly() {
        let issue = ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "high".to_string(),
            description: "God Object with too many methods".to_string(),
            code_snippet: Some("struct GodObject { ... }".to_string()),
            ai_explanation: None,
        };
        let prompt = prompt_templates::for_issue(&issue);
        assert!(prompt.contains("God Object"));
        assert!(prompt.contains("src/main.rs"));
        assert!(prompt.contains("struct GodObject"));
    }

    #[tokio::test]
    async fn ai_explanation_integration() {
        use uveddi::ai::engine::AiAnalysisEngine;
        let mut issue = ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "high".to_string(),
            description: "God Object with too many methods".to_string(),
            code_snippet: Some("struct GodObject { ... }".to_string()),
            ai_explanation: None,
        };
        // No API key, should fallback gracefully
        let ai_engine = AiAnalysisEngine::new();
        let result = ai_engine.analyze_issue(&mut issue).await;
        assert!(result.is_ok());
        // Should not panic or set explanation
        assert!(issue.ai_explanation.is_none());
    }

    #[tokio::test]
    async fn ai_fallback_on_missing_key() {
        use uveddi::ai::engine::AiAnalysisEngine;
        let mut issue = ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "high".to_string(),
            description: "God Object with too many methods".to_string(),
            code_snippet: Some("struct GodObject { ... }".to_string()),
            ai_explanation: None,
        };
        // No API key, should fallback gracefully
        let ai_engine = AiAnalysisEngine::new();
        let result = ai_engine.analyze_issue(&mut issue).await;
        assert!(result.is_ok());
        assert!(issue.ai_explanation.is_none());
    }
}
