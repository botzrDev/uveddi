//! Database Infrastructure Module
//!
//! This module contains concrete implementations of persistence interfaces
//! that break the circular dependency between analysis and database layers.

pub mod persistence_provider;

pub use persistence_provider::*;
