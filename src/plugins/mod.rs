//! # WASM Plugin System
//!
//! This module provides a comprehensive WebAssembly plugin system for Uveddi,
//! enabling secure, performant, and extensible third-party code analysis plugins.
//!
//! ## Architecture Overview
//!
//! The plugin system is built on the WebAssembly Component Model using Wasmtime
//! as the runtime. It provides:
//!
//! - **Security**: Capability-based security model with WASI
//! - **Performance**: Zero-copy data exchange with Apache Arrow
//! - **Extensibility**: Well-defined WIT interfaces for plugin development
//! - **Integration**: Seamless integration with existing analysis engine
//!
//! ## Key Components
//!
//! - `WasmPluginEngine`: Main plugin runtime and orchestrator
//! - `WasmPlugin`: Individual plugin wrapper implementing AnalysisDetector
//! - `PluginRegistry`: Plugin discovery and metadata management
//! - `SecurityPolicy`: Capability-based security configuration
//! - `DataPlane`: Apache Arrow-based data serialization
//!
//! ## Usage Example
//!
//! ```rust
//! use uveddi::plugins::{WasmPluginEngine, PluginRegistry};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut engine = WasmPluginEngine::new().await?;
//!     let registry = PluginRegistry::new("plugins/").await?;
//!     
//!     // Load all plugins from registry
//!     for plugin_id in registry.list_plugins() {
//!         engine.load_plugin(&plugin_id).await?;
//!     }
//!     
//!     // Plugins are now available as AnalysisDetectors
//!     Ok(())
//! }
//! ```

pub mod data_plane;
pub mod engine;
pub mod errors;
pub mod lifecycle;
pub mod registry;
pub mod security;
pub mod types;
pub mod verification;

// Re-exports for convenience
pub use data_plane::AstDataPlane;
pub use engine::WasmPluginEngine;
pub use errors::PluginError;
// Note: PluginResult is deprecated - use crate::error::Result<T> instead
pub use lifecycle::{PluginLifecycleManager, ResourceReport};
pub use registry::{PluginManifest, PluginMetadata, PluginRegistry};
pub use security::{Permission, SecurityPolicy};
pub use types::*;
pub use verification::{PluginVerifier, VerificationReport};

// Feature gate for WASM plugin system
#[cfg(feature = "wasm-plugins")]
pub mod wasm {
    pub use super::*;

    // Generate bindings from WIT file
    wasmtime::component::bindgen!({
        path: "wit/plugin.wit",
        world: "code-analyzer",
        async: true,
    });

    // pub use self::exports::uveddi::plugins::*;
}

#[cfg(not(feature = "wasm-plugins"))]
pub mod wasm {
    //! Stub implementation when WASM plugins are disabled

    use crate::error::UveddiError;

    pub struct WasmPluginEngine;

    impl WasmPluginEngine {
        pub async fn new() -> Result<Self, UveddiError> {
            Err(UveddiError::PluginError(
                crate::plugins::errors::PluginError::Execution(
                    "WASM plugins not enabled. Compile with --features wasm-plugins".to_string(),
                ),
            ))
        }
    }
}
