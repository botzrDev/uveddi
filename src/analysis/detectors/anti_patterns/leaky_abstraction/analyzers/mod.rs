//! Analysis modules for leaky abstraction detection.

pub mod implementation;
pub mod interface;
pub mod leak_detector;
pub mod validation;

pub use implementation::ImplementationAnalyzer;
pub use interface::InterfaceAnalyzer;
pub use leak_detector::LeakDetector;
pub use validation::AbstractionValidator;
