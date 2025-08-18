use crate::database::models::ArchitecturalIssue;
use serde::{Deserialize, Serialize};

/// Represents an AI insight generated from analyzing architectural issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInsight {
    /// Unique identifier for the insight
    pub id: String,
    /// Related issue ID (if applicable)
    pub related_issue_id: Option<String>,
    /// Title of the insight
    pub title: String,
    /// Description of the insight
    pub description: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Suggested solution (if available)
    pub suggestion: Option<String>,
    /// Tags/categories for the insight
    pub tags: Vec<String>,
}

/// Result of AI analysis containing insights and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisResult {
    /// List of insights generated
    pub insights: Vec<AiInsight>,
    /// Model used for analysis
    pub model: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of tokens processed
    pub tokens_processed: Option<u32>,
}
