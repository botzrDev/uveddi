//! # Context Detector Factory
//!
//! Factory for creating new context-aware detectors alongside legacy detector adapters.

use super::pipeline::{AnalysisPipeline, Detector};
use crate::analysis::detector_factory::DetectorFactory as LegacyFactory;
use crate::analysis::detectors::anti_patterns::code_duplication::context_detector::ContextCodeDuplicationDetector;
use crate::analysis::detectors::anti_patterns::dead_code::context_detector::ContextDeadCodeDetector;
use crate::analysis::detectors::anti_patterns::god_object::context_detector::ContextGodObjectDetector;
use crate::analysis::detectors::anti_patterns::god_object::GodObjectConfig;
use crate::ast::compatibility_shim::DetectorAdapter;
use std::sync::Arc;

/// Factory for creating both new context detectors and legacy detector adapters
pub struct ContextDetectorFactory {
    pipeline: Arc<AnalysisPipeline>,
}

impl ContextDetectorFactory {
    /// Create a new context detector factory
    pub fn new(pipeline: Arc<AnalysisPipeline>) -> Self {
        Self { pipeline }
    }

    /// Create all available context detectors
    pub fn create_all_context_detectors(&self) -> Vec<Box<dyn Detector>> {
        vec![
            Box::new(ContextGodObjectDetector::default()),
            Box::new(ContextCodeDuplicationDetector::default()),
            Box::new(ContextDeadCodeDetector::default()),
        ]
    }

    /// Create specific context detector by name
    pub fn create_context_detector(&self, name: &str) -> Option<Box<dyn Detector>> {
        match name {
            "context_god_object" => Some(Box::new(ContextGodObjectDetector::default())),
            "context_code_duplication" => Some(Box::new(ContextCodeDuplicationDetector::default())),
            "context_dead_code" => Some(Box::new(ContextDeadCodeDetector::default())),
            _ => None,
        }
    }

    /// Create legacy detector adapters for all default detectors
    #[cfg(feature = "engine-integration")]
    pub fn create_legacy_adapters(&self) -> Vec<Box<dyn Detector>> {
        let legacy_detectors = LegacyFactory::create_default_detectors();
        let mut adapters = Vec::new();

        for legacy_detector in legacy_detectors {
            let adapter = DetectorAdapter::new(legacy_detector, self.pipeline.clone());
            adapters.push(Box::new(adapter) as Box<dyn Detector>);
        }

        adapters
    }

    /// Create a hybrid set containing both context detectors and legacy adapters
    #[cfg(feature = "engine-integration")]
    pub fn create_hybrid_detector_set(&self) -> Vec<Box<dyn Detector>> {
        let mut detectors = Vec::new();

        // Add new context detectors (prioritized)
        detectors.extend(self.create_all_context_detectors());

        // Add legacy adapters for detectors not yet migrated
        let legacy_detectors = LegacyFactory::create_default_detectors();
        for legacy_detector in legacy_detectors {
            let detector_name = legacy_detector.get_detector_name();

            // Skip legacy detectors that we have context versions for
            match detector_name {
                "GodObjectDetector" | "CodeDuplicationDetector" | "DeadCodeDetector" => {
                    // Skip - we have context versions
                    continue;
                }
                _ => {
                    // Include legacy detector via adapter
                    let adapter = DetectorAdapter::new(legacy_detector, self.pipeline.clone());
                    detectors.push(Box::new(adapter) as Box<dyn Detector>);
                }
            }
        }

        detectors
    }

    /// Get list of available context detector names
    pub fn available_context_detectors(&self) -> Vec<&'static str> {
        vec![
            "context_god_object",
            "context_code_duplication",
            "context_dead_code",
        ]
    }

    /// Check if a detector has been migrated to context
    pub fn is_migrated(&self, detector_name: &str) -> bool {
        match detector_name {
            "GodObjectDetector" => true,
            "CodeDuplicationDetector" => true,
            "DeadCodeDetector" => true,
            _ => false,
        }
    }

    /// Get migration status report
    pub fn migration_status(&self) -> DetectorMigrationStatus {
        let legacy_detectors = LegacyFactory::create_default_detectors();
        let total_detectors = legacy_detectors.len();
        let migrated_count = legacy_detectors
            .iter()
            .filter(|d| self.is_migrated(d.get_detector_name()))
            .count();

        let migrated_names: Vec<String> = legacy_detectors
            .iter()
            .filter(|d| self.is_migrated(d.get_detector_name()))
            .map(|d| d.get_detector_name().to_string())
            .collect();

        let remaining_names: Vec<String> = legacy_detectors
            .iter()
            .filter(|d| !self.is_migrated(d.get_detector_name()))
            .map(|d| d.get_detector_name().to_string())
            .collect();

        DetectorMigrationStatus {
            total_detectors,
            migrated_count,
            remaining_count: total_detectors - migrated_count,
            migrated_detectors: migrated_names,
            remaining_detectors: remaining_names,
        }
    }
}

/// Status of detector migration to context interface
#[derive(Debug, Clone)]
pub struct DetectorMigrationStatus {
    pub total_detectors: usize,
    pub migrated_count: usize,
    pub remaining_count: usize,
    pub migrated_detectors: Vec<String>,
    pub remaining_detectors: Vec<String>,
}

impl DetectorMigrationStatus {
    /// Get migration progress as a percentage
    pub fn progress_percentage(&self) -> f64 {
        if self.total_detectors == 0 {
            return 100.0;
        }
        (self.migrated_count as f64 / self.total_detectors as f64) * 100.0
    }

    /// Generate a progress report string
    pub fn progress_report(&self) -> String {
        format!(
            "Detector Migration Progress: {}/{} ({:.1}%)\n\
            Migrated: {}\n\
            Remaining: {}",
            self.migrated_count,
            self.total_detectors,
            self.progress_percentage(),
            self.migrated_detectors.join(", "),
            self.remaining_detectors.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::analysis::pipeline::AnalysisPipeline;
    use crate::engine::parsing::AstBuilder;
    use std::sync::Arc;

    #[test]
    fn test_context_detector_creation() {
        let ast_builder = Arc::new(AstBuilder::new().unwrap());
        let pipeline = Arc::new(AnalysisPipeline::new(ast_builder));
        let factory = ContextDetectorFactory::new(pipeline);

        let detectors = factory.create_all_context_detectors();
        assert_eq!(detectors.len(), 3);

        let detector_names: Vec<&str> = detectors.iter().map(|d| d.name()).collect();
        assert!(detector_names.contains(&"ContextGodObjectDetector"));
        assert!(detector_names.contains(&"ContextCodeDuplicationDetector"));
        assert!(detector_names.contains(&"ContextDeadCodeDetector"));
    }

    #[test]
    fn test_migration_status() {
        let ast_builder = Arc::new(AstBuilder::new().unwrap());
        let pipeline = Arc::new(AnalysisPipeline::new(ast_builder));
        let factory = ContextDetectorFactory::new(pipeline);

        let status = factory.migration_status();
        assert!(status.migrated_count > 0);
        assert!(status.total_detectors > status.migrated_count);
        assert!(status.progress_percentage() > 0.0);
        assert!(status.progress_percentage() < 100.0);
    }

    #[test]
    fn test_specific_detector_creation() {
        let ast_builder = Arc::new(AstBuilder::new().unwrap());
        let pipeline = Arc::new(AnalysisPipeline::new(ast_builder));
        let factory = ContextDetectorFactory::new(pipeline);

        let god_detector = factory.create_context_detector("context_god_object");
        assert!(god_detector.is_some());
        assert_eq!(god_detector.unwrap().name(), "ContextGodObjectDetector");

        let unknown_detector = factory.create_context_detector("unknown_detector");
        assert!(unknown_detector.is_none());
    }
}
