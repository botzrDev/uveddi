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
