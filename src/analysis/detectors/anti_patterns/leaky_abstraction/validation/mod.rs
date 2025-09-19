//! Validation modules for leaky abstraction detection.

pub mod encapsulation_checker;
pub mod boundary_validator;
pub mod abstraction_scorer;

pub use encapsulation_checker::{EncapsulationChecker, EncapsulationScore};
pub use boundary_validator::{BoundaryValidator, BoundaryIntegrityScore};
pub use abstraction_scorer::{AbstractionScorer, AbstractionQualityScore};