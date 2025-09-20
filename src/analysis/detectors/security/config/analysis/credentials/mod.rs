//! Credential exposure detection in configuration files
//!
//! This module detects hardcoded passwords, API keys, database connection
//! strings, cloud service credentials, and certificate/private keys in
//! configuration files.

pub mod detector;
pub mod scanner;
pub mod validator;

// Re-export main types
pub use detector::CredentialAnalyzer;

use crate::analysis::AnalysisError;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity};
use regex::Regex;
use std::collections::HashMap;

/// Pattern for detecting different types of credentials
pub struct CredentialPattern {
    pub name: String,
    pub regex: Regex,
    pub severity: ConfigSeverity,
    pub confidence_base: f64,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}