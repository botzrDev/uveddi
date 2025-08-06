use crate::ai::engine::AiAnalysisEngine;
use crate::ai::analysis::AiInsight;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use async_trait::async_trait;

/// AI service trait defining the interface for AI-powered analysis
#[async_trait]
pub trait AiService: Send + Sync {
    /// Analyze a list of architectural issues to produce AI insights
    async fn analyze_issues(&self, issues: &[ArchitecturalIssue]) -> Result<Vec<AiInsight>, UveddiError>;
}

/// Concrete implementation of AiService using AiAnalysisEngine
pub struct AiAnalysisService {
    engine: AiAnalysisEngine,
}

impl AiAnalysisService {
    /// Create a new AI analysis service
    pub fn new(engine: AiAnalysisEngine) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl AiService for AiAnalysisService {
    async fn analyze_issues(&self, issues: &[ArchitecturalIssue]) -> Result<Vec<AiInsight>, UveddiError> {
        self.engine.analyze_issues(issues).await
    }
}
