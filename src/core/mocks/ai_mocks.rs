//! Mock AI services for testing without AI feature
//!
//! This module provides mock implementations of AI services that can be used
//! when AI features are disabled or for deterministic testing.

use crate::database::models::ArchitecturalIssue;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Mock AI service that provides deterministic responses for testing
#[derive(Debug, Clone)]
pub struct MockAiService {
    responses: Vec<MockAiInsight>,
}

/// AI service trait for both real and mock implementations
#[async_trait]
pub trait AiServiceTrait: Send + Sync {
    async fn analyze_issues(
        &self,
        issues: &[ArchitecturalIssue],
    ) -> Result<Vec<AiInsight>, MockAiError>;
    async fn suggest_fixes(
        &self,
        issue: &ArchitecturalIssue,
    ) -> Result<Vec<FixSuggestion>, MockAiError>;
    async fn analyze_issue(&self, issue: &mut ArchitecturalIssue) -> Result<(), MockAiError>;
}

#[derive(Debug, Clone)]
pub struct MockAiInsight {
    pub issue_id: String,
    pub confidence: f64,
    pub suggestion: String,
}

#[derive(Debug, Clone)]
pub struct AiInsight {
    pub issue_id: String,
    pub confidence: f64,
    pub suggestion: String,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
pub struct FixSuggestion {
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub code_changes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum MockAiError {
    #[error("Mock AI service error: {0}")]
    ServiceError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl MockAiService {
    pub fn new() -> Self {
        Self {
            responses: vec![
                MockAiInsight {
                    issue_id: "mock-issue-1".to_string(),
                    confidence: 0.85,
                    suggestion: "Consider refactoring this method to reduce complexity".to_string(),
                },
                MockAiInsight {
                    issue_id: "mock-issue-2".to_string(),
                    confidence: 0.75,
                    suggestion: "This class might benefit from the Single Responsibility Principle"
                        .to_string(),
                },
                MockAiInsight {
                    issue_id: "mock-issue-3".to_string(),
                    confidence: 0.90,
                    suggestion: "Consider extracting this functionality into a separate module"
                        .to_string(),
                },
            ],
        }
    }

    pub fn with_responses(responses: Vec<MockAiInsight>) -> Self {
        Self { responses }
    }

    pub fn empty() -> Self {
        Self { responses: vec![] }
    }
}

impl Default for MockAiService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiServiceTrait for MockAiService {
    async fn analyze_issues(
        &self,
        issues: &[ArchitecturalIssue],
    ) -> Result<Vec<AiInsight>, MockAiError> {
        tracing::debug!("Mock AI service analyzing {} issues", issues.len());

        let insights: Vec<AiInsight> = self
            .responses
            .iter()
            .take(issues.len())
            .enumerate()
            .map(|(i, mock)| {
                let issue_id = issues
                    .get(i)
                    .and_then(|issue| issue.issue_id)
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| mock.issue_id.clone());

                AiInsight {
                    issue_id,
                    confidence: mock.confidence,
                    suggestion: mock.suggestion.clone(),
                    metadata: serde_json::json!({
                        "mock": true,
                        "timestamp": chrono::Utc::now().timestamp(),
                        "version": "mock-1.0"
                    }),
                }
            })
            .collect();

        Ok(insights)
    }

    async fn suggest_fixes(
        &self,
        issue: &ArchitecturalIssue,
    ) -> Result<Vec<FixSuggestion>, MockAiError> {
        let issue_type = &issue.description;

        let suggestion = if issue_type.to_lowercase().contains("god object") {
            FixSuggestion {
                title: "Split God Object".to_string(),
                description: "Break down this large class into smaller, focused classes"
                    .to_string(),
                confidence: 0.8,
                code_changes: vec![
                    "Extract related methods into separate classes".to_string(),
                    "Use composition instead of inheritance".to_string(),
                    "Apply Single Responsibility Principle".to_string(),
                ],
            }
        } else if issue_type.to_lowercase().contains("circular") {
            FixSuggestion {
                title: "Remove Circular Dependency".to_string(),
                description: "Introduce abstractions to break circular dependencies".to_string(),
                confidence: 0.9,
                code_changes: vec![
                    "Extract interface or trait".to_string(),
                    "Use dependency injection".to_string(),
                    "Consider architectural patterns like Observer".to_string(),
                ],
            }
        } else {
            FixSuggestion {
                title: "General Refactoring".to_string(),
                description: "Apply general refactoring principles to improve code quality"
                    .to_string(),
                confidence: 0.7,
                code_changes: vec![
                    "Review and simplify the implementation".to_string(),
                    "Add appropriate tests".to_string(),
                    "Consider design patterns".to_string(),
                ],
            }
        };

        Ok(vec![suggestion])
    }

    async fn analyze_issue(&self, issue: &mut ArchitecturalIssue) -> Result<(), MockAiError> {
        tracing::debug!(
            "Mock AI service analyzing single issue: {}",
            issue.description
        );

        // Generate a mock explanation based on the issue type
        let explanation = if issue.description.to_lowercase().contains("god object") {
            format!(
                "This appears to be a God Object anti-pattern. The class '{}' likely has too many responsibilities. \
                Consider breaking it down into smaller, focused classes. Mock confidence: 85%",
                issue.file_path.split('/').last().unwrap_or("unknown")
            )
        } else if issue.description.to_lowercase().contains("circular") {
            format!(
                "Circular dependency detected in '{}'. This can lead to tight coupling and make the code \
                harder to maintain. Consider introducing abstractions or using dependency injection. Mock confidence: 90%",
                issue.file_path
            )
        } else {
            format!(
                "Code quality issue detected in '{}': {}. Consider refactoring to improve maintainability. \
                Mock confidence: 75%",
                issue.file_path, issue.description
            )
        };

        issue.ai_explanation = Some(explanation);
        Ok(())
    }
}

/// Mock AI engine for when AI features are disabled
#[derive(Debug, Clone)]
pub struct MockAiEngine {
    service: MockAiService,
}

impl MockAiEngine {
    pub fn new() -> Self {
        Self {
            service: MockAiService::new(),
        }
    }

    pub async fn analyze_issue(&self, issue: &mut ArchitecturalIssue) -> Result<(), MockAiError> {
        self.service.analyze_issue(issue).await
    }

    pub async fn analyze_issues(
        &self,
        issues: &[ArchitecturalIssue],
    ) -> Result<Vec<AiInsight>, MockAiError> {
        self.service.analyze_issues(issues).await
    }
}

impl Default for MockAiEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Conditional type alias based on feature
#[cfg(any(feature = "ai", feature = "local-ai"))]
pub type DefaultAiService = crate::ai::engine::AiAnalysisEngine;

#[cfg(not(any(feature = "ai", feature = "local-ai")))]
pub type DefaultAiService = MockAiService;

#[cfg(any(feature = "ai", feature = "local-ai"))]
pub type DefaultAiEngine = crate::ai::engine::AiAnalysisEngine;

#[cfg(not(any(feature = "ai", feature = "local-ai")))]
pub type DefaultAiEngine = MockAiEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_ai_service_basic() {
        let service = MockAiService::new();
        let result = service.analyze_issues(&[]).await;
        assert!(
            result.is_ok(),
            "Mock AI service should always succeed with empty input"
        );
    }

    #[tokio::test]
    async fn test_mock_ai_service_with_issues() {
        let service = MockAiService::new();
        let mut issue = ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "test.rs".to_string(),
            start_line: Some(1),
            end_line: Some(10),
            line_number: Some(1),
            column_number: Some(1),
            message: "Test issue".to_string(),
            metadata: "{}".to_string(),
            detector_name: "MockDetector".to_string(),
            created_at: chrono::Utc::now(),
            severity: "high".to_string(),
            description: "God Object detected".to_string(),
            code_snippet: Some("struct GodObject { ... }".to_string()),
            ai_explanation: None,
        };

        service.analyze_issue(&mut issue).await.unwrap();
        assert!(
            issue.ai_explanation.is_some(),
            "Mock should provide explanation"
        );
        assert!(
            issue
                .ai_explanation
                .as_ref()
                .unwrap()
                .contains("God Object"),
            "Explanation should be relevant"
        );
    }

    #[tokio::test]
    async fn test_mock_ai_engine() {
        let engine = MockAiEngine::new();
        let mut issue = ArchitecturalIssue {
            issue_id: Some(2),
            analysis_run_id: 1,
            anti_pattern_type_id: 2,
            file_path: "circular.rs".to_string(),
            start_line: Some(5),
            end_line: Some(15),
            line_number: Some(5),
            column_number: Some(1),
            message: "Test issue".to_string(),
            metadata: "{}".to_string(),
            detector_name: "MockDetector".to_string(),
            created_at: chrono::Utc::now(),
            severity: "medium".to_string(),
            description: "Circular dependency".to_string(),
            code_snippet: Some("mod a { use super::b; }".to_string()),
            ai_explanation: None,
        };

        engine.analyze_issue(&mut issue).await.unwrap();
        assert!(
            issue.ai_explanation.is_some(),
            "Mock engine should provide explanation"
        );
    }
}
