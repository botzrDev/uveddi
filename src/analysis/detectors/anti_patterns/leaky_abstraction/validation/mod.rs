//! Validation modules for leaky abstraction detection.

pub mod abstraction_scorer;
pub mod boundary_validator;
pub mod encapsulation_checker;

pub use abstraction_scorer::{AbstractionQualityScore, AbstractionScorer};
pub use boundary_validator::{BoundaryIntegrityScore, BoundaryValidator};
pub use encapsulation_checker::{EncapsulationChecker, EncapsulationScore};
