//! Rust taint analysis patterns

pub mod sources;
pub mod sinks;
pub mod sanitizers;

pub use sources::get_rust_sources;
pub use sinks::get_rust_sinks;
pub use sanitizers::get_rust_sanitizers;