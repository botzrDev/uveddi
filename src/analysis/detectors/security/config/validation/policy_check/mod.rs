//! Security policy validation for configuration analysis
//!
//! This module validates configuration security findings against
//! organizational security policies and filtering rules.

pub mod enforcer;
pub mod rules;
pub mod validator;

// Re-export main types
pub use rules::SecurityPolicy;
pub use validator::PolicyValidator;

use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;
use std::collections::HashMap;
