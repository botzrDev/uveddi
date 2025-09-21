//! Security pattern detection modules
//!
//! This module contains the split pattern detection logic organized by
//! pattern type: detection coordination, threat patterns, and behavioral patterns.

pub mod behavioral_patterns;
pub mod detection_patterns;
pub mod threat_patterns;

pub use detection_patterns::{
    DetectionConfig, PatternApplicationResult, PatternMatch, SecurityPattern,
    SecurityPatternDetector, SecurityPatternResult,
};

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, StructuralSemanticGraph,
};
use crate::analysis::AnalysisError;

/// Main entry point for security pattern detection
pub async fn detect_security_patterns(
    graph: &StructuralSemanticGraph,
    entities: &[CodeEntity],
) -> Result<SecurityPatternResult, AnalysisError> {
    let detector = SecurityPatternDetector::new();
    detector.detect_patterns(graph, entities).await
}
