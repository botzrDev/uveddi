//! Web framework security analysis
//!
//! This module provides security analysis for popular web frameworks
//! across different programming languages.

use crate::ast::SourceLanguage;
use std::collections::HashMap;

/// Analyzer for web framework security patterns
pub struct WebFrameworkAnalyzer {
    framework_patterns: HashMap<String, FrameworkSecurityProfile>,
}

#[derive(Debug, Clone)]
pub struct FrameworkSecurityProfile {
    pub name: String,
    pub language: SourceLanguage,
    pub common_vulnerabilities: Vec<String>,
    pub security_features: Vec<String>,
    pub best_practices: Vec<String>,
}

impl WebFrameworkAnalyzer {
    pub fn new() -> Self {
        let mut framework_patterns = HashMap::new();

        // Rust frameworks
        framework_patterns.insert(
            "actix-web".to_string(),
            FrameworkSecurityProfile {
                name: "Actix Web".to_string(),
                language: SourceLanguage::Rust,
                common_vulnerabilities: vec![
                    "Missing authentication middleware".to_string(),
                    "Unvalidated input in handlers".to_string(),
                    "Improper error handling".to_string(),
                ],
                security_features: vec![
                    "Built-in CSRF protection".to_string(),
                    "Secure session management".to_string(),
                    "Request rate limiting".to_string(),
                ],
                best_practices: vec![
                    "Use authentication middleware for protected routes".to_string(),
                    "Validate all input parameters".to_string(),
                    "Implement proper error handling".to_string(),
                ],
            },
        );

        framework_patterns.insert(
            "rocket".to_string(),
            FrameworkSecurityProfile {
                name: "Rocket".to_string(),
                language: SourceLanguage::Rust,
                common_vulnerabilities: vec![
                    "Missing request guards".to_string(),
                    "Unvalidated route parameters".to_string(),
                ],
                security_features: vec![
                    "Type-safe request guards".to_string(),
                    "Built-in parameter validation".to_string(),
                ],
                best_practices: vec![
                    "Use request guards for authentication".to_string(),
                    "Validate all route parameters".to_string(),
                ],
            },
        );

        // Python frameworks
        framework_patterns.insert(
            "flask".to_string(),
            FrameworkSecurityProfile {
                name: "Flask".to_string(),
                language: SourceLanguage::Python,
                common_vulnerabilities: vec![
                    "Missing CSRF protection".to_string(),
                    "Insecure session configuration".to_string(),
                    "Unvalidated template rendering".to_string(),
                ],
                security_features: vec![
                    "Flask-WTF for CSRF protection".to_string(),
                    "Secure session cookies".to_string(),
                    "Jinja2 template security".to_string(),
                ],
                best_practices: vec![
                    "Enable CSRF protection for all forms".to_string(),
                    "Use secure session configuration".to_string(),
                    "Validate template variables".to_string(),
                ],
            },
        );

        framework_patterns.insert(
            "django".to_string(),
            FrameworkSecurityProfile {
                name: "Django".to_string(),
                language: SourceLanguage::Python,
                common_vulnerabilities: vec![
                    "DEBUG mode in production".to_string(),
                    "Missing authentication decorators".to_string(),
                    "Insecure middleware configuration".to_string(),
                ],
                security_features: vec![
                    "Built-in CSRF protection".to_string(),
                    "Authentication framework".to_string(),
                    "Security middleware".to_string(),
                ],
                best_practices: vec![
                    "Disable DEBUG in production".to_string(),
                    "Use authentication decorators".to_string(),
                    "Configure security middleware".to_string(),
                ],
            },
        );

        // JavaScript/TypeScript frameworks
        framework_patterns.insert(
            "express".to_string(),
            FrameworkSecurityProfile {
                name: "Express.js".to_string(),
                language: SourceLanguage::JavaScript,
                common_vulnerabilities: vec![
                    "Missing helmet middleware".to_string(),
                    "Unvalidated input parameters".to_string(),
                    "Missing authentication middleware".to_string(),
                ],
                security_features: vec![
                    "Helmet.js for security headers".to_string(),
                    "Express-validator for input validation".to_string(),
                    "Passport.js for authentication".to_string(),
                ],
                best_practices: vec![
                    "Use helmet for security headers".to_string(),
                    "Validate all input parameters".to_string(),
                    "Implement proper authentication".to_string(),
                ],
            },
        );

        Self { framework_patterns }
    }

    pub fn get_framework_profile(&self, framework_name: &str) -> Option<&FrameworkSecurityProfile> {
        self.framework_patterns.get(framework_name)
    }

    pub fn detect_framework_from_content(
        &self,
        content: &str,
        language: &SourceLanguage,
    ) -> Option<String> {
        for (framework_name, profile) in &self.framework_patterns {
            if profile.language == *language {
                // Check for both hyphen and underscore variants (e.g., "actix-web" vs "actix_web")
                let underscore_variant = framework_name.replace('-', "_");
                if content.contains(framework_name) || content.contains(&underscore_variant) {
                    return Some(framework_name.clone());
                }
            }
        }
        None
    }

    pub fn get_security_recommendations(&self, framework_name: &str) -> Vec<String> {
        if let Some(profile) = self.framework_patterns.get(framework_name) {
            profile.best_practices.clone()
        } else {
            vec!["Follow general web security best practices".to_string()]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_detection() {
        let analyzer = WebFrameworkAnalyzer::new();

        let rust_content = "use actix_web::{web, App, HttpResponse};";
        let detected = analyzer.detect_framework_from_content(rust_content, &SourceLanguage::Rust);
        assert_eq!(detected, Some("actix-web".to_string()));

        let python_content = "from flask import Flask, request";
        let detected =
            analyzer.detect_framework_from_content(python_content, &SourceLanguage::Python);
        assert_eq!(detected, Some("flask".to_string()));
    }

    #[test]
    fn test_security_recommendations() {
        let analyzer = WebFrameworkAnalyzer::new();
        let recommendations = analyzer.get_security_recommendations("flask");
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("CSRF")));
    }
}
