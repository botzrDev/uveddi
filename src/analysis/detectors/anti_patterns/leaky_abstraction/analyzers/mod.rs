//! Analysis modules for leaky abstraction detection.

pub mod interface;
pub mod implementation;
pub mod validation;
pub mod leak_detector;

pub use interface::InterfaceAnalyzer;
pub use implementation::ImplementationAnalyzer;
pub use validation::AbstractionValidator;
pub use leak_detector::LeakDetector;