//! Detector tolerance calibration and standardization system
//!
//! This module provides a systematic approach to managing and calibrating detector
//! thresholds across the codebase, ensuring consistency, context-awareness, and
//! data-driven fine-tuning.

pub mod confidence_bands;
pub mod context_multipliers;
pub mod feedback;
pub mod profiles;
pub mod severity_scoring;
pub mod testing;

pub use confidence_bands::{ConfidenceBand, ConfidenceLevel};
pub use context_multipliers::{CodeContext, ContextAwareThresholds};
pub use feedback::{FeedbackCollector, UserVerdict};
pub use profiles::{DetectorProfile, ProfileConfig};
pub use severity_scoring::{SeverityScore, StandardSeverity};
pub use testing::{ABTest, CalibrationMetrics, ThresholdOptimizer};
