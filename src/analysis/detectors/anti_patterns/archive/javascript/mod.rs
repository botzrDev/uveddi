//! JavaScript-specific anti-pattern detectors
//!
//! This module contains detectors for anti-patterns specific to JavaScript code,
//! including async/await misuse, DOM manipulation issues, module dependency problems,
//! type coercion pitfalls, and scope/context issues.

pub mod async_antipatterns;
pub mod dom_issues;
pub mod module_dependency;
pub mod scope_issues;
pub mod type_coercion;

pub use async_antipatterns::JsAsyncAntipatternsDetector;
pub use dom_issues::JsDomIssuesDetector;
pub use module_dependency::JsModuleDependencyDetector;
pub use scope_issues::JsScopeIssuesDetector;
pub use type_coercion::JsTypeCoercionDetector;
