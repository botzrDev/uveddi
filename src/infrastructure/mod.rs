//! Infrastructure Layer
//!
//! This module contains concrete implementations of the core interfaces,
//! implementing the dependency inversion principle to break circular dependencies.

pub mod database;

pub use database::*;
