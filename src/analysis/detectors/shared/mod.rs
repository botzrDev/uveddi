//! Shared utilities for detectors
//!
//! This module contains common utilities and helper functions that are used across
//! multiple detector implementations to reduce code duplication and improve consistency.

pub mod query_helper;
pub mod extractor;

pub use query_helper::TreeSitterQueryHelper;
pub use extractor::{LanguageSpecificExtractor, ExtractedSymbol, SymbolType};