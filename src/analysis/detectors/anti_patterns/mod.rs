//! Anti-pattern detectors
//!
//! This module contains detectors for various anti-patterns and code smells
//! included in the Uveddi Community Edition.

// Community Edition anti-pattern detectors
pub mod code_duplication;
pub mod cyclic_dependencies;
// pub mod data_clumps;
pub mod dead_code;
// pub mod feature_envy;
pub mod god_object;
pub mod large_classes;
pub mod leaky_abstraction;
pub mod long_methods;
pub mod magic_values;
// pub mod shotgun_surgery;
pub mod tight_coupling;
// Optionally include state_synchronization if desired
// pub mod state_synchronization;

// Re-exports for community detectors
pub use code_duplication::{CodeDuplicationDetector, DuplicationConfig};
pub use cyclic_dependencies::CyclicDependenciesDetector;
// pub use data_clumps::{DataClumpsConfig, DataClumpsDetector};
pub use dead_code::{DeadCodeConfig, DeadCodeDetector};
// pub use feature_envy::{FeatureEnvyConfig, FeatureEnvyDetector};
pub use god_object::GodObjectDetector;
pub use large_classes::{ClassMetrics, LanguageThresholds, LargeClassConfig, LargeClassDetector};
pub use leaky_abstraction::LeakyAbstractionDetector;
pub use long_methods::LongMethodsDetector;
pub use magic_values::MagicValuesDetector;
// pub use shotgun_surgery::{ShotgunSurgeryConfig, ShotgunSurgeryDetector};
pub use tight_coupling::{CouplingMetrics, TightCouplingConfig, TightCouplingDetector};
// pub use state_synchronization::StateSynchronizationDetector;

// Security detector integration
pub use crate::analysis::detectors::security::SecurityDetector;
