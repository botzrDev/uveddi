//! Centralized AI feature configuration and conditional compilation

use serde::{Deserialize, Serialize};

/// Centralized configuration for AI feature detection and management
#[derive(Debug, Clone)]
pub struct AiFeatureConfig;

impl AiFeatureConfig {
    /// Check if AI features are enabled at compile time
    pub const fn is_enabled() -> bool {
        cfg!(feature = "ai") || cfg!(feature = "local-ai")
    }

    /// Get AI feature status for runtime checks
    pub fn status() -> AiFeatureStatus {
        if Self::is_enabled() {
            AiFeatureStatus::Enabled
        } else {
            AiFeatureStatus::Disabled
        }
    }

    /// Check if local AI (Ollama) is enabled
    pub const fn is_local_ai_enabled() -> bool {
        cfg!(feature = "local-ai")
    }

    /// Get human-readable description of current AI feature status
    pub fn description() -> String {
        match Self::status() {
            AiFeatureStatus::Enabled => {
                if Self::is_local_ai_enabled() {
                    "AI features enabled with local Ollama support".to_string()
                } else {
                    "AI features enabled".to_string()
                }
            }
            AiFeatureStatus::Disabled => {
                "AI features disabled - using mock implementations".to_string()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiFeatureStatus {
    Enabled,
    Disabled,
}

/// Macro for consistent AI feature conditional compilation
///
/// Usage: ai_feature!({ /* enabled code */ }, { /* disabled code */ })
/// or:    ai_feature!({ /* enabled code */ })
#[macro_export]
macro_rules! ai_feature {
    ($enabled_code:block, $disabled_code:block) => {
        #[cfg(any(feature = "ai", feature = "local-ai"))]
        $enabled_code

        #[cfg(not(any(feature = "ai", feature = "local-ai")))]
        $disabled_code
    };
    ($enabled_code:block) => {
        #[cfg(any(feature = "ai", feature = "local-ai"))]
        $enabled_code
    };
}

/// Conditional AI type definitions
#[cfg(any(feature = "ai", feature = "local-ai"))]
pub type AiAnalysisResult = Vec<crate::core::mocks::ai_mocks::AiInsight>;

#[cfg(not(any(feature = "ai", feature = "local-ai")))]
pub type AiAnalysisResult = Vec<crate::core::mocks::ai_mocks::AiInsight>;

#[cfg(any(feature = "ai", feature = "local-ai"))]
pub type AiService = crate::ai::engine::AiAnalysisEngine;

#[cfg(not(any(feature = "ai", feature = "local-ai")))]
pub type AiService = crate::core::mocks::MockAiService;

/// Mock AI analysis result for when AI features are disabled
#[derive(Debug, Clone, Default)]
pub struct MockAiAnalysisResult {
    pub confidence: f64,
    pub suggestion: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl MockAiAnalysisResult {
    pub fn new() -> Self {
        Self {
            confidence: 0.5,
            suggestion: "Mock AI analysis - enable AI features for real analysis".to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_feature_detection() {
        #[cfg(any(feature = "ai", feature = "local-ai"))]
        {
            assert!(
                AiFeatureConfig::is_enabled(),
                "AI features should be detected when enabled"
            );
        }

        #[cfg(not(any(feature = "ai", feature = "local-ai")))]
        {
            assert!(
                !AiFeatureConfig::is_enabled(),
                "AI features should not be detected when disabled"
            );
        }
    }

    #[test]
    fn test_ai_feature_status() {
        let status = AiFeatureConfig::status();
        assert!(matches!(
            status,
            AiFeatureStatus::Enabled | AiFeatureStatus::Disabled
        ));
    }

    #[test]
    fn test_ai_feature_description() {
        let description = AiFeatureConfig::description();
        assert!(!description.is_empty());
        assert!(description.contains("AI features"));
    }
}
