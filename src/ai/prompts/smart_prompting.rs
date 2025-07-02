//! Smart prompting and RAG strategy implementation

use crate::ast::CustomAst;

/// Builds a prompt embedding code snippets and AST structure.
pub fn build_prompt_from_ast(ast: &CustomAst, issue_context: &str) -> String {
    // Extract a summary of the AST structure (e.g., node types, relationships)
    let ast_summary = ast.summary(); // Assumes a summary() method exists or is implemented
                                     // Extract relevant code snippets (e.g., lines around the detected issue)
    let code_snippet = ast
        .extract_relevant_code(issue_context)
        .unwrap_or_else(|| "<code unavailable>".to_string());
    // Format the prompt using a template
    format!(
        "You are an expert software architect.\n\
Given the following code and context, explain the architectural issue.\n\
\nCode:\n{code_snippet}\n\
Context:\n{ast_summary}\n\
Issue:\n{issue_context}\n\
Respond in the following JSON format:\n{{\n  \"title\": \"...\",\n  \"description\": \"...\",\n  \"explanation\": \"...\",\n  \"refactoring\": \"...\",\n  \"confidence\": \"...\"\n}}\n",
        code_snippet = code_snippet,
        ast_summary = ast_summary,
        issue_context = issue_context
    )
}

/// Builds a prompt embedding ranked context snippets and AST structure.
pub fn build_prompt_with_context(
    context_snippets: &[String],
    ast: &CustomAst,
    issue_context: &str,
) -> String {
    let ast_summary = ast.summary();
    let context = if context_snippets.is_empty() {
        "<no relevant context>".to_string()
    } else {
        context_snippets.join("\n---\n")
    };
    format!(
        "You are an expert software architect.\n\
Given the following ranked context and code structure, explain the architectural issue.\n\
\nContext Snippets:\n{context}\n\
AST Structure:\n{ast_summary}\n\
Issue:\n{issue_context}\n\
Respond in the following JSON format:\n{{\n  \"title\": \"...\",\n  \"description\": \"...\",\n  \"explanation\": \"...\",\n  \"refactoring\": \"...\",\n  \"confidence\": \"...\"\n}}\n",
        context = context,
        ast_summary = ast_summary,
        issue_context = issue_context
    )
}

/// Hallucination mitigation: structured prompting, uncertainty handling, and output schema enforcement
pub fn add_hallucination_mitigation(prompt: &str) -> String {
    let mitigation_instructions = r#"
---
INSTRUCTIONS FOR AI:
- If you are not certain, respond with "I don’t know." or indicate uncertainty.
- Only use facts present in the provided code and context. Do not invent details.
- Follow the required JSON output schema exactly. If you cannot answer, set fields to null or "unknown".
- If the issue is ambiguous, explain your uncertainty in the explanation field.
- Do not provide information not grounded in the input.
---
"#;
    format!("{}\n{}", prompt, mitigation_instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::CustomAst;

    #[test]
    fn test_build_prompt_from_ast_basic() {
        let ast = CustomAst::default(); // Assuming Default is implemented for CustomAst
        let issue_context = "God Object detected in module foo.rs";
        let prompt = build_prompt_from_ast(&ast, issue_context);
        assert!(prompt.contains(issue_context));
    }

    #[test]
    fn test_build_prompt_with_context_ranked_snippets() {
        let ast = CustomAst::default();
        let issue_context = "God Object detected in module foo.rs";
        let context_snippets = vec![
            "Snippet 1: Related to the issue.".to_string(),
            "Snippet 2: Provides additional context.".to_string(),
        ];
        let prompt = build_prompt_with_context(&context_snippets, &ast, issue_context);
        assert!(prompt.contains("Snippet 1: Related to the issue."));
        assert!(prompt.contains("Snippet 2: Provides additional context."));
    }

    #[test]
    fn test_add_hallucination_mitigation_appends_instruction() {
        let prompt = "Explain the issue.";
        let mitigated = add_hallucination_mitigation(prompt);
        assert!(mitigated.contains("respond with \"I don’t know.\""));
    }
}
