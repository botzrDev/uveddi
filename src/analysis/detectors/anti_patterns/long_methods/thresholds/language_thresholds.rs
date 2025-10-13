//! Language-specific threshold configurations

use crate::analysis::detectors::anti_patterns::long_methods::types::LanguageThresholds;
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::collections::HashMap;

/// Get default thresholds for a specific language
pub fn get_language_thresholds(language: SourceLanguage) -> LanguageThresholds {
    match language {
        SourceLanguage::Rust => rust_thresholds(),
        SourceLanguage::Python => python_thresholds(),
        SourceLanguage::JavaScript | SourceLanguage::TypeScript => javascript_thresholds(),
    }
}

/// Get thresholds for Rust (balanced for production code)
fn rust_thresholds() -> LanguageThresholds {
    LanguageThresholds {
        max_logical_loc: 80,  // Balanced for real-world Rust code
        max_statements: 60,   // Allow for complex but readable functions
        max_parameters: 7,    // Rust type system helps with this
        max_nesting_depth: 5, // Match-based patterns can be deep
        max_cyclomatic_complexity: 20,  // More lenient for complex logic
        max_cognitive_complexity: 18,   // Allow reasonable complexity
    }
}

/// Get thresholds for Python (based on Pylint defaults)
fn python_thresholds() -> LanguageThresholds {
    LanguageThresholds {
        max_logical_loc: 100,          // Pylint default
        max_statements: 50,            // Python readability standards
        max_parameters: 5,             // PEP 8 guidance
        max_nesting_depth: 5,          // Python nesting conventions
        max_cyclomatic_complexity: 10, // Common Python standard
        max_cognitive_complexity: 15,
    }
}

/// Get thresholds for JavaScript (framework-aware)
fn javascript_thresholds() -> LanguageThresholds {
    LanguageThresholds {
        max_logical_loc: 80,  // ESLint complexity defaults
        max_statements: 40,   // JavaScript function standards
        max_parameters: 4,    // JavaScript callback patterns
        max_nesting_depth: 4, // Callback hell prevention
        max_cyclomatic_complexity: 12,
        max_cognitive_complexity: 18,
    }
}

/// Create a map of all language thresholds
pub fn create_threshold_map() -> HashMap<SourceLanguage, LanguageThresholds> {
    let mut thresholds = HashMap::new();
    thresholds.insert(SourceLanguage::Rust, rust_thresholds());
    thresholds.insert(SourceLanguage::Python, python_thresholds());
    thresholds.insert(SourceLanguage::JavaScript, javascript_thresholds());
    thresholds.insert(SourceLanguage::TypeScript, javascript_thresholds());
    thresholds
}

/// Adjust thresholds based on project characteristics
pub fn adjust_thresholds_for_project(
    base_thresholds: LanguageThresholds,
    project_type: &str,
) -> LanguageThresholds {
    match project_type {
        "test" | "tests" => LanguageThresholds {
            max_logical_loc: base_thresholds.max_logical_loc * 2,
            max_statements: base_thresholds.max_statements * 2,
            ..base_thresholds
        },
        "generated" => LanguageThresholds {
            max_logical_loc: base_thresholds.max_logical_loc * 3,
            max_statements: base_thresholds.max_statements * 3,
            max_cyclomatic_complexity: base_thresholds.max_cyclomatic_complexity * 2,
            ..base_thresholds
        },
        _ => base_thresholds,
    }
}
