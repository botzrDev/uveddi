//! Security policy validation for configuration analysis
//!
//! This module validates configuration security findings against
//! organizational security policies and filtering rules.

pub mod enforcer;
pub mod rules;
pub mod validator;

// Re-export main types
pub use validator::PolicyValidator;
pub use rules::SecurityPolicy;

use crate::analysis::AnalysisError;
use crate::analysis::detectors::security::types::SecurityIssue;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity};
use std::collections::HashMap;