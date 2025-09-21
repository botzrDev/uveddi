//! Malicious pattern detection
//!
//! This module provides malicious pattern detection capabilities including
//! signature-based detection and behavioral analysis.

use super::{PatternConfig, PatternMatch, PatternMatcher};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::{
    SecurityIssue, SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityType,
};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Include the split modules as submodules
#[path = "malicious/behaviors.rs"]
mod behaviors;
#[path = "malicious/signatures.rs"]
mod signatures;

// Re-export from submodules
pub use behaviors::MaliciousPatternDatabase;
pub use signatures::{load_default_patterns, MaliciousPattern, MaliciousPatternType, ThreatLevel};
