//! TypeScript/JavaScript taint analysis patterns

pub mod sources;
pub mod sinks;
pub mod sanitizers;

pub use sources::get_typescript_sources;
pub use sinks::get_typescript_sinks;
pub use sanitizers::get_typescript_sanitizers;