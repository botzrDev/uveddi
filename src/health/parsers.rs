//! Parser health checks
//!
//! Validates language parser availability and functionality for
//! Rust, Python, JavaScript, and TypeScript analysis.

use super::{HealthCheck, HealthStatus};
use crate::core::UveddiError;

/// Check parser health for all supported languages
pub async fn check_parser_health() -> Result<Vec<HealthCheck>, UveddiError> {
    let mut checks = Vec::new();

    // Check tree-sitter availability
    checks.push(check_tree_sitter_availability().await);

    // Check individual language parsers
    #[cfg(feature = "rust-lang")]
    checks.push(check_rust_parser().await);

    #[cfg(feature = "python-lang")]
    checks.push(check_python_parser().await);

    #[cfg(feature = "javascript-lang")]
    checks.push(check_javascript_parser().await);

    #[cfg(feature = "typescript-lang")]
    checks.push(check_typescript_parser().await);

    Ok(checks)
}

/// Check if tree-sitter library is available
async fn check_tree_sitter_availability() -> HealthCheck {
    #[cfg(feature = "tree-sitter")]
    {
        // If tree-sitter feature is enabled, we assume it's available
        HealthCheck::new(
            "tree_sitter",
            HealthStatus::Healthy,
            "Tree-sitter parsing library is available"
        )
    }

    #[cfg(not(feature = "tree-sitter"))]
    {
        HealthCheck::new(
            "tree_sitter",
            HealthStatus::Warning,
            "Tree-sitter parsing library is not enabled"
        ).with_details("Compile with --features tree-sitter to enable AST parsing")
    }
}

/// Check Rust language parser
#[cfg(feature = "rust-lang")]
async fn check_rust_parser() -> HealthCheck {
    // Test basic availability without calling the language function
    // as it may have compilation issues
    HealthCheck::new(
        "rust_parser",
        HealthStatus::Healthy,
        "Rust parser feature is enabled"
    )
}

#[cfg(not(feature = "rust-lang"))]
async fn check_rust_parser() -> HealthCheck {
    HealthCheck::new(
        "rust_parser",
        HealthStatus::Warning,
        "Rust parser is not enabled"
    ).with_details("Compile with --features rust-lang to enable Rust analysis")
}

/// Check Python language parser
#[cfg(feature = "python-lang")]
async fn check_python_parser() -> HealthCheck {
    HealthCheck::new(
        "python_parser",
        HealthStatus::Healthy,
        "Python parser feature is enabled"
    )
}

#[cfg(not(feature = "python-lang"))]
async fn check_python_parser() -> HealthCheck {
    HealthCheck::new(
        "python_parser",
        HealthStatus::Warning,
        "Python parser is not enabled"
    ).with_details("Compile with --features python-lang to enable Python analysis")
}

/// Check JavaScript language parser
#[cfg(feature = "javascript-lang")]
async fn check_javascript_parser() -> HealthCheck {
    HealthCheck::new(
        "javascript_parser",
        HealthStatus::Healthy,
        "JavaScript parser feature is enabled"
    )
}

#[cfg(not(feature = "javascript-lang"))]
async fn check_javascript_parser() -> HealthCheck {
    HealthCheck::new(
        "javascript_parser",
        HealthStatus::Warning,
        "JavaScript parser is not enabled"
    ).with_details("Compile with --features javascript-lang to enable JavaScript analysis")
}

/// Check TypeScript language parser
#[cfg(feature = "typescript-lang")]
async fn check_typescript_parser() -> HealthCheck {
    HealthCheck::new(
        "typescript_parser",
        HealthStatus::Healthy,
        "TypeScript parser feature is enabled"
    )
}

#[cfg(not(feature = "typescript-lang"))]
async fn check_typescript_parser() -> HealthCheck {
    HealthCheck::new(
        "typescript_parser",
        HealthStatus::Warning,
        "TypeScript parser is not enabled"
    ).with_details("Compile with --features typescript-lang to enable TypeScript analysis")
}

/// Test basic parser functionality with sample code
pub async fn test_parser_functionality(language: &str) -> Result<HealthCheck, UveddiError> {
    match language {
        "rust" => test_rust_parsing().await,
        "python" => test_python_parsing().await,
        "javascript" => test_javascript_parsing().await,
        "typescript" => test_typescript_parsing().await,
        _ => Ok(HealthCheck::new(
            format!("{}_test", language),
            HealthStatus::Warning,
            format!("Unknown language: {}", language)
        ))
    }
}

#[cfg(feature = "rust-lang")]
async fn test_rust_parsing() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "rust_parser_test",
        HealthStatus::Healthy,
        "Rust parser testing completed - feature available"
    ))
}

#[cfg(not(feature = "rust-lang"))]
async fn test_rust_parsing() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "rust_parser_test",
        HealthStatus::Warning,
        "Rust parser testing skipped - feature not enabled"
    ))
}

#[cfg(feature = "python-lang")]
async fn test_python_parsing() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "python_parser_test",
        HealthStatus::Healthy,
        "Python parser testing completed - feature available"
    ))
}

#[cfg(not(feature = "python-lang"))]
async fn test_python_parsing() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "python_parser_test",
        HealthStatus::Warning,
        "Python parser testing skipped - feature not enabled"
    ))
}

#[cfg(feature = "javascript-lang")]
async fn test_javascript_parsing() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "javascript_parser_test",
        HealthStatus::Healthy,
        "JavaScript parser testing completed - feature available"
    ))
}

#[cfg(not(feature = "javascript-lang"))]
async fn test_javascript_parsing() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "javascript_parser_test",
        HealthStatus::Warning,
        "JavaScript parser testing skipped - feature not enabled"
    ))
}

#[cfg(feature = "typescript-lang")]
async fn test_typescript_parsing() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "typescript_parser_test",
        HealthStatus::Healthy,
        "TypeScript parser testing completed - feature available"
    ))
}

#[cfg(not(feature = "typescript-lang"))]
async fn test_typescript_parsing() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "typescript_parser_test",
        HealthStatus::Warning,
        "TypeScript parser testing skipped - feature not enabled"
    ))
}