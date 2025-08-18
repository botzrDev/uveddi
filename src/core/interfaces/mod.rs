//! Core Interfaces Module
//!
//! This module defines the abstract interfaces that break circular dependencies
//! as part of UV-105 implementation. These interfaces implement the dependency
//! inversion principle to create clean architecture boundaries.

pub mod events;
pub mod persistence;

pub use events::*;
pub use persistence::*;
