//! Unit test for AiAnalysisEngine when no provider is configured

use codeatlas::ai::engine::AiAnalysisEngine;
use codeatlas::database::models::ArchitecturalIssue;
use codeatlas::ast::CustomAst;

#[tokio::test]
async fn analyze_issue_no_provider_does_not_set_explanation() {
    let mut issue = ArchitecturalIssue {
        issue_id: None,
        analysis_run_id: 1,
        anti_pattern_type_id: 1,
        file_path: "src/main.rs".to_string(),
        start_line: Some(1),
        end_line: Some(10),
        severity: "high".to_string(),
        description: "God Object with too many methods".to_string(),
        code_snippet: Some("struct GodObject { ... }".to_string()),
        ai_explanation: None,
    };
    let ai_engine = AiAnalysisEngine::new();
    let dummy_ast = CustomAst::default();
    let result = ai_engine.analyze_issue(&mut issue, &dummy_ast).await;
    assert!(result.is_ok());
    assert!(issue.ai_explanation.is_none(), "AI explanation should not be set if no provider is configured");
}
