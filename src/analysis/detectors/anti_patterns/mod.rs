//! Anti-pattern detectors
//!
//! This module contains detectors for various anti-patterns and code smells
//! included in the Uveddi Community Edition.

// Community Edition anti-pattern detectors
pub mod code_duplication;
pub mod cyclic_dependencies;
pub mod dead_code;
pub mod god_object;
pub mod large_classes;
pub mod leaky_abstraction;
pub mod long_methods;
pub mod magic_values;
pub mod tight_coupling;
// Optionally include state_synchronization if desired
// pub mod state_synchronization;

// Re-exports for community detectors
pub use code_duplication::CodeDuplicationDetector;
pub use cyclic_dependencies::CyclicDependenciesDetector;
pub use dead_code::DeadCodeDetector;
pub use god_object::GodObjectDetector;
// pub use large_classes::LargeClassesDetector; // TODO: Implement
pub use leaky_abstraction::LeakyAbstractionDetector;
// pub use long_methods::LongMethodsDetector; // TODO: Implement
pub use magic_values::MagicValuesDetector;
pub use tight_coupling::TightCouplingDetector;
// pub use state_synchronization::StateSynchronizationDetector;
