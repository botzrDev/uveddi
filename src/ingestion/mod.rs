//! Data ingestion module for Uveddi
//!
//! This module provides utilities for scanning and walking codebases asynchronously to collect files for analysis.
//! The primary components are the [`AsyncWalker`] and the file scanner utilities.

pub mod async_walker;
pub mod file_scanner;

pub use async_walker::AsyncWalker;
pub use file_scanner::*;
