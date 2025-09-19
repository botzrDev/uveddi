//! Analysis modules for agent detection and behavior analysis
//!
//! This module contains the core analysis logic for detecting and analyzing
//! agent patterns in security-related code.

pub mod agent_detector;
pub mod behavior_analyzer;
pub mod pattern_matcher;

pub use agent_detector::AgentDetector;
pub use behavior_analyzer::BehaviorAnalyzer;
pub use pattern_matcher::PatternMatcher;

use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;

/// Core analysis interface for agent detection
pub trait AnalysisModule: Send + Sync {
    /// Analyze security context for agent patterns
    async fn analyze(&self, context: &SecurityContext) -> Result<Vec<SecurityIssue>, AnalysisError>;

    /// Get the name of this analysis module
    fn module_name(&self) -> &'static str;

    /// Check if this module can analyze the given context
    fn can_analyze(&self, context: &SecurityContext) -> bool;
}

/// Analysis configuration for agent detection
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub enable_deep_analysis: bool,
    pub confidence_threshold: f64,
    pub max_analysis_depth: u32,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            enable_deep_analysis: true,
            confidence_threshold: 0.7,
            max_analysis_depth: 10,
        }
    }
}

/// Analysis context wrapper for agent detection
pub struct AnalysisContext<'a> {
    pub security_context: &'a SecurityContext,
    pub config: &'a AnalysisConfig,
}

impl<'a> AnalysisContext<'a> {
    pub fn new(security_context: &'a SecurityContext, config: &'a AnalysisConfig) -> Self {
        Self {
            security_context,
            config,
        }
    }
}