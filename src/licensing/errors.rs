//! Licensing error types
//!
//! This module defines error types for license operations.

use std::fmt;

/// The kind of licensing error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseErrorKind {
    /// License key format is invalid
    InvalidKeyFormat,
    /// License key not found or doesn't exist
    KeyNotFound,
    /// License has expired
    Expired,
    /// License is not activated on this machine
    NotActivated,
    /// Machine ID mismatch
    MachineIdMismatch,
    /// License signature verification failed
    InvalidSignature,
    /// Network error during activation
    NetworkError,
    /// API error from license server
    ApiError,
    /// Storage error (reading/writing license file)
    StorageError,
    /// Feature not available in current tier
    FeatureNotAvailable,
    /// Seat limit exceeded
    SeatLimitExceeded,
    /// Unknown or internal error
    Unknown,
}

/// A licensing error
#[derive(Debug)]
pub struct LicenseError {
    /// The kind of error
    pub kind: LicenseErrorKind,
    /// Human-readable error message
    pub message: String,
    /// Optional source error
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl LicenseError {
    /// Create a new license error
    pub fn new(kind: LicenseErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    /// Create an error with a source
    pub fn with_source(
        kind: LicenseErrorKind,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an invalid key format error
    pub fn invalid_key_format(message: impl Into<String>) -> Self {
        Self::new(LicenseErrorKind::InvalidKeyFormat, message)
    }

    /// Create a key not found error
    pub fn key_not_found() -> Self {
        Self::new(LicenseErrorKind::KeyNotFound, "License key not found")
    }

    /// Create an expired error
    pub fn expired() -> Self {
        Self::new(
            LicenseErrorKind::Expired,
            "License has expired. Please renew at uveddi.org/pricing",
        )
    }

    /// Create a not activated error
    pub fn not_activated() -> Self {
        Self::new(
            LicenseErrorKind::NotActivated,
            "License not activated. Run 'uveddi license activate <KEY>'",
        )
    }

    /// Create a feature not available error
    pub fn feature_not_available(feature: &str, required_tier: &str) -> Self {
        Self::new(
            LicenseErrorKind::FeatureNotAvailable,
            format!(
                "'{}' requires {} tier or higher. Upgrade at uveddi.org/pricing",
                feature, required_tier
            ),
        )
    }

    /// Create a storage error
    pub fn storage_error(message: impl Into<String>) -> Self {
        Self::new(LicenseErrorKind::StorageError, message)
    }

    /// Create a network error
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::new(LicenseErrorKind::NetworkError, message)
    }

    /// Create an API error
    pub fn api_error(message: impl Into<String>) -> Self {
        Self::new(LicenseErrorKind::ApiError, message)
    }

    /// Get a user-friendly error message with upgrade suggestion
    pub fn user_message(&self) -> String {
        match self.kind {
            LicenseErrorKind::FeatureNotAvailable => {
                format!(
                    "⚠️  {}\n\n💡 Upgrade your license to unlock this feature.",
                    self.message
                )
            }
            LicenseErrorKind::Expired => {
                format!(
                    "⚠️  {}\n\n💡 Renew your license to continue using premium features.",
                    self.message
                )
            }
            LicenseErrorKind::NotActivated => {
                format!("ℹ️  {}\n\n💡 Already have a license? Activate it with: uveddi license activate <KEY>", self.message)
            }
            _ => self.message.clone(),
        }
    }
}

impl fmt::Display for LicenseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LicenseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for LicenseError {
    fn from(err: std::io::Error) -> Self {
        Self::with_source(
            LicenseErrorKind::StorageError,
            format!("License storage error: {}", err),
            err,
        )
    }
}

impl From<serde_json::Error> for LicenseError {
    fn from(err: serde_json::Error) -> Self {
        Self::with_source(
            LicenseErrorKind::StorageError,
            format!("License data error: {}", err),
            err,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = LicenseError::feature_not_available("Rust analysis", "Pro");
        assert_eq!(err.kind, LicenseErrorKind::FeatureNotAvailable);
        assert!(err.message.contains("Rust analysis"));
        assert!(err.message.contains("Pro"));
    }

    #[test]
    fn test_user_message() {
        let err = LicenseError::expired();
        let msg = err.user_message();
        assert!(msg.contains("⚠️"));
        assert!(msg.contains("💡"));
    }
}
