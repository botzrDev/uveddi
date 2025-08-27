//! Security stub implementations for alpha testing
//!
//! This module provides no-op implementations of security functions
//! to allow building without the full security module.

use std::path::Path;

pub const MAX_FILES_PER_ANALYSIS: usize = 10000;

#[derive(Debug)]
pub enum SecurityError {
    Stub,
    InvalidInput { field: String, reason: String },
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::Stub => write!(f, "Security stub error"),
            SecurityError::InvalidInput { field, reason } => {
                write!(f, "Invalid input for field '{}': {}", field, reason)
            }
        }
    }
}

impl std::error::Error for SecurityError {}

pub fn validate_input(_input: &str, _field_name: &str) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_model_name(_name: &str) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_numeric_range(
    _value: i64,
    _min: i64,
    _max: i64,
    _field_name: &str,
) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_url(_url: &str) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_file_size(_path: &Path) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_file_type(_path: &Path) -> Result<(), SecurityError> {
    Ok(())
}

pub fn sanitize_description(desc: &str) -> String {
    desc.to_string()
}

pub fn validate_directory_depth(_depth: usize) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_file_count(_count: usize) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_code_analysis_data(
    _data: &str,
    _field_name: &str,
    _max_length: Option<usize>,
) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_file_path_for_storage(
    _file_path: &str,
    _field_name: &str,
) -> Result<(), SecurityError> {
    Ok(())
}

pub struct HttpSecurityConfig {
    pub timeout_seconds: u64,
    pub read_timeout_seconds: u64,
    pub connect_timeout_seconds: u64,
}

impl Default for HttpSecurityConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            read_timeout_seconds: 30,
            connect_timeout_seconds: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CliArgumentType {
    String,
    Path,
    Number,
    Boolean,
    FilePath,
    Generic,
}

pub fn validate_api_request(_content_type: Option<&str>, _content_length: Option<u64>, _user_agent: Option<&str>) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_cli_argument(_value: &str, _field_name: &str, _arg_type: CliArgumentType) -> Result<(), SecurityError> {
    Ok(())
}

pub fn validate_config_file_path(_path: &Path, _allowed_dirs: Option<&[&Path]>) -> Result<(), SecurityError> {
    Ok(())
}

pub struct SecureHttpClient;

impl SecureHttpClient {
    pub fn new(_config: HttpSecurityConfig) -> Result<Self, SecurityError> {
        Ok(Self)
    }

    pub async fn get(&self, _url: &str) -> Result<reqwest::Response, String> {
        Err("SecureHttpClient stub: get method not implemented".to_string())
    }

    pub async fn post(&self, _url: &str, _body: String) -> Result<reqwest::Response, String> {
        Err("SecureHttpClient stub: post method not implemented".to_string())
    }
}
