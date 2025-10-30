use tokio::time::{timeout, Duration};
use uveddi::ai::ollama_provider::{OllamaProvider, OllamaConfig};
use uveddi::ai::prompts::smart_prompting::SmartPromptBuilder;
use uveddi::database::models::ArchitecturalIssue;

#[tokio::test]
async fn test_ollama_availability() {
    // This test verifies if Ollama is running and the model is available
    // It's designed to be skipped if Ollama is not available (soft dependency)
    
    let config = OllamaConfig {
        model: "deepseek-coder:6.7b-instruct-q4_0".to_string(),
        api_url: "http://localhost:11434".to_string(),
        timeout_seconds: 10, // Short timeout for test
        temperature: 0.1,
        max_tokens: 100,
    };
    
    let provider = OllamaProvider::new(config);
    
    // Use a short timeout for CI environments where Ollama might not be available
    match timeout(Duration::from_secs(5), provider.check_availability()).await {
        Ok(result) => match result {
            Ok(available) => {
                if available {
                    println!("✅ Ollama is available with the deepseek-coder model");
                } else {
                    println!("⚠️ Ollama is not available or the model is not loaded - skipping test");
                    return;
                }
            },
            Err(e) => {
                println!("⚠️ Error checking Ollama availability: {} - skipping test", e);
                return;
            }
        },
        Err(_) => {
            println!("⚠️ Timeout checking Ollama availability - skipping test");
            return;
        }
    }
    
    // Only continue with inference test if Ollama is available
    let prompt = "What is a cyclic dependency in software architecture?";
    
    match timeout(Duration::from_secs(15), provider.infer(prompt)).await {
        Ok(result) => match result {
            Ok(response) => {
                assert!(!response.is_empty(), "Response should not be empty");
                assert!(response.len() > 20, "Response should be substantial");
                println!("✅ Received valid Ollama response");
            },
            Err(e) => {
                panic!("Failed to get inference response: {}", e);
            }
        },
        Err(_) => {
            panic!("Inference request timed out");
        }
    }
}

#[tokio::test]
async fn test_smart_prompting_with_issue() {
    // Create a sample architectural issue
    let issue = ArchitecturalIssue {
        issue_id: Some(1),
        analysis_run_id: 1,
        anti_pattern_type_id: 1, // Cyclic dependency
        file_path: "src/cyclic_a.rs".to_string(),
        start_line: Some(10),
        end_line: Some(15),
        severity: "high".to_string(),
        description: "Cyclic dependency detected between modules cyclic_a and cyclic_b".to_string(),
        code_snippet: Some("pub fn function_a() {\n    println!(\"Function A called\");\n    crate::cyclic_b::function_b();\n}".to_string()),
        ai_explanation: None,
    };
    
    // Test the smart prompt builder
    let prompt_builder = SmartPromptBuilder::new();
    let prompt = prompt_builder.build_prompt_for_issue(&issue);
    
    // Verify prompt contains key elements
    assert!(prompt.contains("Cyclic dependency detected"), "Prompt should contain issue description");
    assert!(prompt.contains("cyclic_a and cyclic_b"), "Prompt should contain module names");
    assert!(prompt.contains("function_a"), "Prompt should contain code snippet");
    assert!(prompt.contains("DO NOT invent or hallucinate"), "Prompt should contain anti-hallucination guidance");
    assert!(prompt.contains("JSON format"), "Prompt should request JSON response format");
    
    println!("✅ Smart prompting builds valid prompts with anti-hallucination measures");
}
