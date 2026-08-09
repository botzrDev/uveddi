//! # UVEDDI Licensing Module
//!
//! This module provides license validation, activation, and tier-based feature gating
//! for the UVEDDI premium service tiers.
//!
//! ## Tier Structure
//!
//! - **Free**: JavaScript/TypeScript only, GodObjectDetector only, Markdown output
//! - **Pro** ($199/year): 10 languages, all detectors, all outputs, security, AI
//! - **Team** ($99/month): 14 languages, 5 seats, priority support
//! - **Enterprise** (Custom): 16+ languages, SSO, compliance, dedicated support
//!
//! ## Usage
//!
//! ```rust,no_run
//! use uveddi::licensing::{get_current_tier, LicenseTier, is_feature_allowed};
//!
//! // Check current license tier
//! let tier = get_current_tier();
//!
//! // Check if a specific feature is allowed
//! if is_feature_allowed("rust-lang", &tier) {
//!     // Proceed with Rust analysis
//! }
//! ```

mod activation;
mod errors;
mod license;
mod storage;
mod validation;

pub use activation::{
    activate_license, check_activation_status, deactivate_license, print_activation_status,
    ActivationStatus,
};
pub use errors::{LicenseError, LicenseErrorKind};
pub use license::{License, LicenseInfo, LicenseTier};
pub use storage::{delete_license, get_license_path, load_license, save_license};
pub use validation::{
    get_current_tier, get_required_tier_for_feature, is_feature_allowed, is_license_valid,
    require_feature, validate_license_key,
};

/// Features that can be gated by license tier
pub mod features {
    /// Language features
    pub const LANG_RUST: &str = "rust-lang";
    pub const LANG_PYTHON: &str = "python-lang";
    pub const LANG_GO: &str = "go-lang";
    pub const LANG_JAVA: &str = "java-lang";
    pub const LANG_C: &str = "c-lang";
    pub const LANG_CPP: &str = "cpp-lang";
    pub const LANG_CSHARP: &str = "csharp-lang";
    pub const LANG_PHP: &str = "php-lang";
    pub const LANG_RUBY: &str = "ruby-lang";
    pub const LANG_KOTLIN: &str = "kotlin-lang";
    pub const LANG_SWIFT: &str = "swift-lang";
    pub const LANG_SCALA: &str = "scala-lang";
    pub const LANG_LUA: &str = "lua-lang";
    pub const LANG_SQL: &str = "sql-lang";

    /// Detector features
    pub const DETECTOR_DEAD_CODE: &str = "dead-code-detector";
    pub const DETECTOR_LARGE_CLASSES: &str = "large-classes-detector";
    pub const DETECTOR_TIGHT_COUPLING: &str = "tight-coupling-detector";
    pub const DETECTOR_LONG_METHODS: &str = "long-methods-detector";
    pub const DETECTOR_MAGIC_VALUES: &str = "magic-values-detector";
    pub const DETECTOR_CYCLIC_DEPS: &str = "cyclic-deps-detector";
    pub const DETECTOR_SECURITY: &str = "security-detector";

    /// Output format features
    pub const OUTPUT_JSON: &str = "output-json";
    pub const OUTPUT_HTML: &str = "output-html";
    pub const OUTPUT_SVG: &str = "output-svg";
    pub const OUTPUT_SARIF: &str = "output-sarif";

    /// Other features
    pub const AI_INSIGHTS: &str = "ai-insights";
    pub const MULTI_SEAT: &str = "multi-seat";
}
