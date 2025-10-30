//! tests/ai/hallucination_mitigation.rs

use uveddi::ai::prompts::prompt_templates;
use uveddi::database::models::ArchitecturalIssue;

#[test]
fn test_hallucination_mitigation_in_prompt() {
    let issue = ArchitecturalIssue {
        issue_id: Some(1),
        analysis_run_id: 1,
        anti_pattern_type_id: 1, // God Object
        file_path: "src/main.rs".to_string(),
        start_line: Some(10),
        end_line: Some(100),
        severity: "high".to_string(),
        description: "This is a god object".to_string(),
        code_snippet: Some("...".to_string()),
        ai_explanation: None,
    };

    let prompt = prompt_templates::for_issue(&issue);

    // Assert that the prompt contains instructions to mitigate hallucinations
    assert!(prompt.contains("If you are unsure or lack enough context, say \"Not enough information to provide a reliable explanation.\""));
    assert!(prompt.contains("Do not make up details or speculate."));
    assert!(prompt.contains("Only use information present in the description and code snippet above."));
}