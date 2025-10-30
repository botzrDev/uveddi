//! Test utilities for the Uveddi project
//!
//! This module provides comprehensive testing infrastructure including:
//! - Mock implementations for external dependencies
//! - Test fixtures for common scenarios
//! - Helper functions for test setup and teardown
//! - Async test utilities for tokio-based testing

pub mod fixtures;
pub mod helpers;
pub mod mocks;
pub mod parser_helpers;

#[cfg(test)]
mod validation_test;

pub use fixtures::*;
pub use helpers::*;
pub use mocks::*;
pub use parser_helpers::*;