//! Error types for resource management

use std::fmt;

/// Result type for resource management operations
pub type ResourceResult<T> = Result<T, ResourceError>;

/// Errors that can occur during resource management operations
#[derive(Debug, Clone)]
pub enum ResourceError {
    /// Memory allocation failed due to exhaustion
    MemoryExhausted {
        requested: u64,
        available: u64,
        limit: u64,
    },
    
    /// Resource limit exceeded (non-memory)
    ResourceLimitExceeded {
        resource_type: String,
        limit: u64,
        requested: u64,
    },
    
    /// Configuration is invalid
    InvalidConfiguration(String),
    
    /// System resource unavailable
    ResourceUnavailable(String),
    
    /// Analysis timeout
    AnalysisTimeout {
        duration_ms: u64,
        timeout_ms: u64,
    },
    
    /// File processing error
    FileProcessingError {
        file_path: String,
        error: String,
    },
    
    /// Monitoring system error
    MonitoringError(String),
    
    /// Graceful degradation error
    DegradationError(String),
    
    /// I/O error during resource operations
    IoError(String),
    
    /// System error (from OS or runtime)
    SystemError(String),
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::MemoryExhausted { requested, available, limit } => {
                write!(
                    f,
                    "Memory exhausted: requested {} bytes, available {} bytes (limit: {} bytes)",
                    requested, available, limit
                )
            },
            ResourceError::ResourceLimitExceeded { resource_type, limit, requested } => {
                write!(
                    f,
                    "Resource limit exceeded for {}: requested {}, limit {}",
                    resource_type, requested, limit
                )
            },
            ResourceError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            },
            ResourceError::ResourceUnavailable(msg) => {
                write!(f, "Resource unavailable: {}", msg)
            },
            ResourceError::AnalysisTimeout { duration_ms, timeout_ms } => {
                write!(
                    f,
                    "Analysis timeout: took {} ms, limit {} ms",
                    duration_ms, timeout_ms
                )
            },
            ResourceError::FileProcessingError { file_path, error } => {
                write!(f, "File processing error for {}: {}", file_path, error)
            },
            ResourceError::MonitoringError(msg) => {
                write!(f, "Monitoring error: {}", msg)
            },
            ResourceError::DegradationError(msg) => {
                write!(f, "Degradation error: {}", msg)
            },
            ResourceError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            },
            ResourceError::SystemError(msg) => {
                write!(f, "System error: {}", msg)
            },
        }
    }
}

impl std::error::Error for ResourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<std::io::Error> for ResourceError {
    fn from(err: std::io::Error) -> Self {
        ResourceError::IoError(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for ResourceError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        ResourceError::AnalysisTimeout {
            duration_ms: 0, // We don't have the actual duration here
            timeout_ms: 0,  // We don't have the timeout here
        }
    }
}

/// Specific error types for memory management
#[derive(Debug, Clone)]
pub enum MemoryError {
    /// Allocation failed
    AllocationFailed {
        component: String,
        requested: u64,
        available: u64,
    },
    
    /// Memory guard already released
    GuardReleased,
    
    /// Memory tracker destroyed
    TrackerDestroyed,
    
    /// Invalid allocation size
    InvalidSize(u64),
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::AllocationFailed { component, requested, available } => {
                write!(
                    f,
                    "Memory allocation failed for {}: requested {} bytes, available {} bytes",
                    component, requested, available
                )
            },
            MemoryError::GuardReleased => {
                write!(f, "Memory guard has already been released")
            },
            MemoryError::TrackerDestroyed => {
                write!(f, "Memory tracker has been destroyed")
            },
            MemoryError::InvalidSize(size) => {
                write!(f, "Invalid allocation size: {} bytes", size)
            },
        }
    }
}

impl std::error::Error for MemoryError {}

impl From<MemoryError> for ResourceError {
    fn from(err: MemoryError) -> Self {
        match err {
            MemoryError::AllocationFailed { requested, available, .. } => {
                ResourceError::MemoryExhausted {
                    requested,
                    available,
                    limit: available + requested,
                }
            },
            _ => ResourceError::SystemError(err.to_string()),
        }
    }
}

/// Helper macro for creating resource errors
#[macro_export]
macro_rules! resource_error {
    ($kind:ident, $($arg:expr),*) => {
        ResourceError::$kind($($arg),*)
    };
    ($kind:ident { $($field:ident: $value:expr),* }) => {
        ResourceError::$kind { $($field: $value),* }
    };
}

/// Helper macro for early return on resource errors
#[macro_export]
macro_rules! resource_try {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(err.into()),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_exhausted_error() {
        let error = ResourceError::MemoryExhausted {
            requested: 1000,
            available: 500,
            limit: 2000,
        };
        
        assert!(error.to_string().contains("Memory exhausted"));
        assert!(error.to_string().contains("1000"));
        assert!(error.to_string().contains("500"));
        assert!(error.to_string().contains("2000"));
    }
    
    #[test]
    fn test_resource_limit_exceeded_error() {
        let error = ResourceError::ResourceLimitExceeded {
            resource_type: "file_handles".to_string(),
            limit: 100,
            requested: 150,
        };
        
        assert!(error.to_string().contains("Resource limit exceeded"));
        assert!(error.to_string().contains("file_handles"));
    }
    
    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let resource_error: ResourceError = io_error.into();
        
        match resource_error {
            ResourceError::IoError(_) => {},
            _ => panic!("Expected IoError variant"),
        }
    }
}