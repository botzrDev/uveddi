//! Centralized prompt templates for AI Reasoning Engine

/// Example template for architectural issue explanation
pub const EXPLANATION_TEMPLATE: &str = r#"
You are an expert software architect. Given the following code and context, explain the architectural issue:

Code:
{{code_snippet}}

Context:
{{ast_structure}}

Respond in the following JSON format:
{
  "title": "...",
  "description": "...",
  "explanation": "...",
  "refactoring": "...",
  "confidence": "..."
}
"#;

/// Critic prompt template for self-correction loop
pub const CRITIC_TEMPLATE: &str = r#"
You are a highly critical software architecture reviewer. Given the following prompt, AI answer, and context, identify any errors, hallucinations, or unsupported claims. Suggest corrections or improvements. If the answer is correct and well-grounded, state 'Valid'.

Prompt:
{{prompt}}

AI Answer:
{{ai_answer}}

Context:
{{context}}

Respond in the following JSON format:
{
  "verdict": "Valid | Needs Correction | Unverifiable",
  "explanation": "...",
  "corrected_answer": "..."
}
"#;
