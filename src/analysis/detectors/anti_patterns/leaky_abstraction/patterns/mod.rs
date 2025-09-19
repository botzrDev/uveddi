//! Pattern detection modules for specific types of leaky abstractions.

pub mod exposed_internals;
pub mod tight_coupling_patterns;
pub mod state_exposure;
pub mod data_structure_leaks;

pub use exposed_internals::ExposedInternalsPattern;
pub use tight_coupling_patterns::TightCouplingPattern;
pub use state_exposure::StateExposurePattern;
pub use data_structure_leaks::DataStructureLeaksPattern;