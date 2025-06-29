//! Common types for AI Reasoning Engine outputs

/// AI-generated suggestion for an architectural issue
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiSuggestion {
    pub title: String,
    pub description: String,
    pub explanation: String,
    pub refactoring: String,
    pub confidence: Option<String>,
}
