//! Core Module
//!
//! This module contains the core abstractions and interfaces that enable
//! dependency inversion and break circular dependencies (UV-105).

pub mod interfaces;
pub mod features;
pub mod mocks;

pub use interfaces::*;
pub use features::*;
pub use mocks::*;
