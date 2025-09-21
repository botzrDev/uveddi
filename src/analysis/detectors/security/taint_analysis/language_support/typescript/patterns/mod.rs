//! TypeScript/JavaScript taint analysis patterns

pub mod sanitizers;
pub mod sinks;
pub mod sources;

pub use sanitizers::get_typescript_sanitizers;
pub use sinks::get_typescript_sinks;
pub use sources::get_typescript_sources;
