//! Rust taint analysis patterns

pub mod sanitizers;
pub mod sinks;
pub mod sources;

pub use sanitizers::get_rust_sanitizers;
pub use sinks::get_rust_sinks;
pub use sources::get_rust_sources;
