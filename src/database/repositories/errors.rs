//! Standardized error types for repository operations
//!
//! This module provides consistent error handling across all repository implementations

use std::fmt;

/// Primary repository error type
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    /// Database operation failed
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Connection pool error
    #[error("Connection pool error: {0}")]
    Pool(String),

    /// Entity not found
    #[error("Entity not found: {entity_type} with {identifier}")]
    NotFound {
        entity_type: String,
        identifier: String,
    },

    /// Validation error (invalid data)
    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    /// Conflict error (e.g., unique constraint violation)
    #[error("Conflict error: {message}")]
    Conflict { message: String },

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network/connection timeout
    #[error("Timeout error: operation took longer than {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },

    /// Permission/access denied
    #[error("Access denied: {operation} on {resource}")]
    AccessDenied { operation: String, resource: String },

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Runtime error (async, threading issues)
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Migration-specific error
    #[error("Migration error: {0}")]
    Migration(String),
}

impl RepositoryError {
    /// Create a database error with context
    pub fn database<E>(message: impl Into<String>, source: Option<E>) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Database {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Create a not found error
    pub fn not_found(entity_type: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            identifier: identifier.into(),
        }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(timeout_seconds: u64) -> Self {
        Self::Timeout { timeout_seconds }
    }

    /// Create an access denied error
    pub fn access_denied(operation: impl Into<String>, resource: impl Into<String>) -> Self {
        Self::AccessDenied {
            operation: operation.into(),
            resource: resource.into(),
        }
    }

    /// Check if error is recoverable (can retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            RepositoryError::Pool(_)
                | RepositoryError::Timeout { .. }
                | RepositoryError::Runtime(_)
        )
    }

    /// Check if error indicates missing data
    pub fn is_not_found(&self) -> bool {
        matches!(self, RepositoryError::NotFound { .. })
    }

    /// Check if error is a conflict
    pub fn is_conflict(&self) -> bool {
        matches!(self, RepositoryError::Conflict { .. })
    }

    /// Check if error is a validation failure
    pub fn is_validation(&self) -> bool {
        matches!(self, RepositoryError::Validation { .. })
    }
}

/// Convenience type alias for repository results
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Trait for converting database-specific errors to RepositoryError
pub trait IntoRepositoryError<T> {
    fn into_repository_error(self) -> RepositoryResult<T>;
}

// Implement conversions from common error types

impl From<rusqlite::Error> for RepositoryError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::SqliteFailure(sqlite_err, ref msg) => match sqlite_err.code {
                rusqlite::ErrorCode::DatabaseBusy => Self::Timeout {
                    timeout_seconds: 30,
                },
                rusqlite::ErrorCode::ConstraintViolation => Self::Conflict {
                    message: msg.clone().unwrap_or_else(|| "Constraint violation".to_string()),
                },
                _ => Self::Database {
                    message: format!("SQLite error: {:?}", sqlite_err),
                    source: Some(Box::new(err)),
                },
            },
            rusqlite::Error::InvalidColumnType(_, column, _) => Self::Validation {
                field: column,
                message: "Invalid column type".to_string(),
            },
            _ => Self::Database {
                message: "Database operation failed".to_string(),
                source: Some(Box::new(err)),
            },
        }
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for RepositoryError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::TimedOut => Self::Timeout {
                timeout_seconds: 30,
            },
            std::io::ErrorKind::PermissionDenied => Self::AccessDenied {
                operation: "file_access".to_string(),
                resource: err.to_string(),
            },
            _ => Self::Runtime(format!("IO error: {}", err)),
        }
    }
}

// TODO: Implement ConnectionError type in connection module
// impl From<crate::database::connection::ConnectionError> for RepositoryError {
//     fn from(err: crate::database::connection::ConnectionError) -> Self {
//         Self::Pool(err.to_string())
//     }
// }

impl From<tokio::task::JoinError> for RepositoryError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Runtime(format!("Task join error: {}", err))
    }
}

impl From<crate::error::UveddiError> for RepositoryError {
    fn from(err: crate::error::UveddiError) -> Self {
        use crate::error::UveddiError;
        match err {
            UveddiError::DatabaseError { message, .. } => {
                Self::Database { message, source: None }
            }
            UveddiError::DatabaseConnection(msg) => {
                Self::Pool(msg)
            }
            UveddiError::ConfigError { message, .. } => {
                Self::Validation { field: "config".to_string(), message }
            }
            UveddiError::Configuration(msg) => {
                Self::Validation { field: "config".to_string(), message: msg }
            }
            UveddiError::IoError { message, .. } => {
                Self::Runtime(format!("IO error: {}", message))
            }
            UveddiError::SerializationError { message, .. } => {
                Self::Serialization(message)
            }
            _ => Self::Runtime(err.to_string())
        }
    }
}

// Implement convenience conversion for Results
impl<T> IntoRepositoryError<T> for rusqlite::Result<T> {
    fn into_repository_error(self) -> RepositoryResult<T> {
        self.map_err(RepositoryError::from)
    }
}

impl<T> IntoRepositoryError<T> for serde_json::Result<T> {
    fn into_repository_error(self) -> RepositoryResult<T> {
        self.map_err(RepositoryError::from)
    }
}

/// Helper macro for creating repository errors with context
#[macro_export]
macro_rules! repo_error {
    (not_found, $entity:expr, $id:expr) => {
        $crate::database::repositories::errors::RepositoryError::not_found($entity, $id)
    };
    (validation, $field:expr, $msg:expr) => {
        $crate::database::repositories::errors::RepositoryError::validation($field, $msg)
    };
    (conflict, $msg:expr) => {
        $crate::database::repositories::errors::RepositoryError::conflict($msg)
    };
    (database, $msg:expr) => {
        $crate::database::repositories::errors::RepositoryError::database(
            $msg,
            None::<std::io::Error>,
        )
    };
    (runtime, $msg:expr) => {
        $crate::database::repositories::errors::RepositoryError::Runtime($msg.to_string())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_error_creation() {
        let not_found = RepositoryError::not_found("User", "123");
        assert!(not_found.is_not_found());

        let validation = RepositoryError::validation("email", "Invalid format");
        assert!(validation.is_validation());

        let conflict = RepositoryError::conflict("Username already exists");
        assert!(conflict.is_conflict());
    }

    #[test]
    fn test_error_recoverability() {
        let timeout = RepositoryError::timeout(30);
        assert!(timeout.is_recoverable());

        let not_found = RepositoryError::not_found("User", "123");
        assert!(!not_found.is_recoverable());
    }

    #[test]
    fn test_rusqlite_error_conversion() {
        let sqlite_err = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            None,
        );

        let repo_err = RepositoryError::from(sqlite_err);
        assert!(matches!(repo_err, RepositoryError::Timeout { .. }));
    }

    #[test]
    fn test_macro_usage() {
        let error = repo_error!(not_found, "User", "123");
        assert!(error.is_not_found());

        let error = repo_error!(validation, "email", "Required field");
        assert!(error.is_validation());

        let error = repo_error!(conflict, "Duplicate entry");
        assert!(error.is_conflict());
    }
}
