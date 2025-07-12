//! Error types for the WASM plugin system

use thiserror::Error;

/// Result type for plugin operations
/// 
/// **DEPRECATED**: Use `crate::error::Result<T>` instead for consistency.
/// This type alias is maintained for backward compatibility but will be removed
/// in a future version. All new code should use the unified Result type.
#[deprecated(since = "0.1.0", note = "Use crate::error::Result<T> instead")]
pub type PluginResult<T> = Result<T, PluginError>;

/// Comprehensive error types for the plugin system
#[derive(Error, Debug)]
pub enum PluginError {
    /// Wasmtime runtime errors
    #[cfg(feature = "wasm-plugins")]
    #[error("WASM runtime error: {0}")]
    Runtime(#[from] wasmtime::Error),

    /// Component instantiation errors
    #[error("Component instantiation error: {0}")]
    Instantiation(String),

    /// Plugin verification errors
    #[error("Plugin verification failed: {0}")]
    Verification(#[from] VerificationError),

    /// Plugin loading errors
    #[error("Plugin loading error: {0}")]
    Loading(String),

    /// Plugin execution errors
    #[error("Plugin execution error: {0}")]
    Execution(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    /// Security policy violation
    #[error("Security policy violation: {0}")]
    SecurityViolation(String),

    /// Data serialization/deserialization errors
    #[error("Data plane error: {0}")]
    DataPlane(#[from] DataPlaneError),

    /// Plugin registry errors
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Plugin already exists
    #[error("Plugin already exists: {0}")]
    AlreadyExists(String),

    /// Invalid plugin state
    #[error("Invalid plugin state: expected {expected}, found {actual}")]
    InvalidState { expected: String, actual: String },

    /// Timeout error
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

/// Plugin verification specific errors
#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Static analysis failed: {0}")]
    StaticAnalysis(String),

    #[error("Code signature verification failed: {0}")]
    SignatureVerification(String),

    #[error("Manifest validation failed: {0}")]
    ManifestValidation(String),

    #[error("Capability audit failed: {0}")]
    CapabilityAudit(String),

    #[error("Malicious code detected: {0}")]
    MaliciousCode(String),

    #[error("Unsupported WASM feature: {0}")]
    UnsupportedFeature(String),
}

/// Data plane specific errors
#[derive(Error, Debug)]
pub enum DataPlaneError {
    #[error("AST conversion error: {0}")]
    AstConversion(String),

    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    #[error("Data size limit exceeded: {limit} bytes")]
    SizeLimit { limit: u64 },

    #[error("Handle not found: {handle_id}")]
    HandleNotFound { handle_id: u32 },
}

/// Plugin registry specific errors
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Plugin manifest parse error: {0}")]
    ManifestParse(String),

    #[error("Plugin discovery error: {0}")]
    Discovery(String),

    #[error("Plugin validation error: {0}")]
    Validation(String),

    #[error("Registry database error: {0}")]
    Database(String),

    #[error("Plugin index corruption: {0}")]
    IndexCorruption(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Helper macros for error creation
#[macro_export]
macro_rules! plugin_error {
    ($kind:ident, $msg:expr) => {
        $crate::plugins::errors::PluginError::$kind($msg.to_string())
    };
    ($kind:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::plugins::errors::PluginError::$kind(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! verification_error {
    ($kind:ident, $msg:expr) => {
        $crate::plugins::errors::VerificationError::$kind($msg.to_string())
    };
    ($kind:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::plugins::errors::VerificationError::$kind(format!($fmt, $($arg)*))
    };
}

/// Placeholder documentation for public items
impl PluginError {
    /// Returns a string describing the error
    pub fn description(&self) -> String {
        match self {
            #[cfg(feature = "wasm-plugins")]
            PluginError::Runtime(ref err) => err.to_string(),
            PluginError::Instantiation(ref desc) => desc.clone(),
            PluginError::Verification(ref err) => err.to_string(),
            PluginError::Loading(ref desc) => desc.clone(),
            PluginError::Execution(ref desc) => desc.clone(),
            PluginError::ResourceLimit(ref desc) => desc.clone(),
            PluginError::SecurityViolation(ref desc) => desc.clone(),
            PluginError::DataPlane(ref err) => err.to_string(),
            PluginError::Registry(ref err) => err.to_string(),
            PluginError::Configuration(ref desc) => desc.clone(),
            PluginError::Io(ref err) => err.to_string(),
            PluginError::Json(ref err) => err.to_string(),
            PluginError::NotFound(ref desc) => desc.clone(),
            PluginError::AlreadyExists(ref desc) => desc.clone(),
            PluginError::InvalidState { expected, actual } => {
                format!("Invalid state: expected {}, found {}", expected, actual)
            }
            PluginError::Timeout { timeout_ms } => {
                format!("Operation timed out after {}ms", timeout_ms)
            }
            PluginError::Unsupported(ref desc) => desc.clone(),
        }
    }
}
