//! Pattern detection modules for specific types of leaky abstractions.

pub mod data_structure_leaks;
pub mod exposed_internals;
pub mod state_exposure;
pub mod tight_coupling_patterns;

pub use data_structure_leaks::DataStructureLeaksPattern;
pub use exposed_internals::ExposedInternalsPattern;
pub use state_exposure::StateExposurePattern;
pub use tight_coupling_patterns::TightCouplingPattern;
