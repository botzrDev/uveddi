//! tests/ai/provider_abstraction.rs

use uveddi::ai::engine::{AiAnalysisEngine, AiError};
use uveddi::ai::api::llm_provider::LlmProvider;
use uveddi::database::models::ArchitecturalIssue;
use async_trait::async_trait;
use mockall::mock;
use anyhow::Result;

mock! {
    pub LlmProvider {}

    #[async_trait]
    impl LlmProvider for LlmProvider {
        async fn generate_explanation(&self, prompt: &str) -> Result<String>;
        fn get_provider_name(&self) -> &'static str;
    }
}

fn create_test_issue() -> ArchitecturalIssue {
    ArchitecturalIssue {
        issue_id: Some(1),
        analysis_run_id: 1,
        anti_pattern_type_id: 1,
        file_path: "test.rs".to_string(),
        start_line: Some(1),
        end_line: Some(1),
        severity: "high".to_string(),
        description: "test".to_string(),
        code_snippet: Some("test".to_string()),
        ai_explanation: None,
    }
}

#[tokio::test]
async fn test_provider_fallback() {
    let mut issue = create_test_issue();

    let mut failing_provider = MockLlmProvider::new();
    failing_provider.expect_generate_explanation()
        .returning(|_| Err(anyhow::anyhow!("Failed")));

    let mut succeeding_provider = MockLlmProvider::new();
    succeeding_provider.expect_generate_explanation()
        .returning(|_| Ok("Success".to_string()));

    let engine = AiAnalysisEngine::new()
        .with_openai_api("dummy_key".to_string()) // This will be the failing provider
        .with_anthropic("dummy_key".to_string()); // This will be the succeeding provider

    // This is a bit of a hack, but it's the easiest way to test the fallback
    // without changing the engine's internal structure.
    let mut engine_with_mocks = AiAnalysisEngine::new();
    engine_with_mocks.api_provider = Some(Box::new(failing_provider));
    engine_with_mocks.anthropic_provider = Some(Box::new(succeeding_provider));


    let result = engine_with_mocks.analyze_issue(&mut issue).await;

    assert!(result.is_ok());
    assert_eq!(issue.ai_explanation, Some("Success".to_string()));
}

#[tokio::test]
async fn test_no_provider_available() {
    let mut issue = create_test_issue();
    let engine = AiAnalysisEngine::new();

    let result = engine.analyze_issue(&mut issue).await;

    assert!(result.is_ok());
    assert!(issue.ai_explanation.is_none());
}