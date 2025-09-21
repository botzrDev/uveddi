//! Threshold management for long methods detection

pub mod adaptive_thresholds;
pub mod language_thresholds;

pub use adaptive_thresholds::AdaptiveThresholdCalculator;
pub use language_thresholds::get_language_thresholds;
