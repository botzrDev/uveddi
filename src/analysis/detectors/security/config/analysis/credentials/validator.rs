//! Validation and filtering logic for credentials

use super::CredentialPattern;

/// Calculate confidence for a credential match
pub fn calculate_confidence(pattern: &CredentialPattern, matched_text: &str, line: &str) -> f64 {
    let mut confidence = pattern.confidence_base;

    // Reduce confidence for common test/example values
    let test_indicators = ["test", "example", "demo", "placeholder", "xxx", "***"];
    if test_indicators.iter().any(|&indicator| {
        matched_text.to_lowercase().contains(indicator) || line.to_lowercase().contains(indicator)
    }) {
        confidence *= 0.3;
    }

    // Reduce confidence for obviously fake values
    if matched_text.chars().all(|c| c == 'x' || c == '*' || c == '0') {
        confidence *= 0.1;
    }

    // Increase confidence for production-like contexts
    let prod_indicators = ["prod", "production", "live", "release"];
    if prod_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator)) {
        confidence = (confidence * 1.2).min(1.0);
    }

    confidence
}

/// Check if a value looks like a real credential
pub fn looks_like_credential(value: &str) -> bool {
    // Skip obviously fake or empty values
    if value.is_empty() || value.len() < 6 {
        return false;
    }

    let fake_indicators = ["test", "example", "demo", "placeholder", "xxx", "***", "changeme"];
    if fake_indicators.iter().any(|&indicator| value.to_lowercase().contains(indicator)) {
        return false;
    }

    // Look for credential-like patterns
    let has_mixed_case = value.chars().any(|c| c.is_uppercase()) && value.chars().any(|c| c.is_lowercase());
    let has_numbers = value.chars().any(|c| c.is_numeric());
    let has_special = value.chars().any(|c| !c.is_alphanumeric());
    let reasonable_length = value.len() >= 8 && value.len() <= 256;

    (has_mixed_case || has_numbers || has_special) && reasonable_length
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::super::types::ConfigSeverity;
    use regex::Regex;

    #[test]
    fn test_confidence_calculation() {
        let pattern = CredentialPattern {
            name: "Test Pattern".to_string(),
            regex: Regex::new(r"password").unwrap(),
            severity: ConfigSeverity::High,
            confidence_base: 0.8,
            cwe_id: Some(798),
            owasp_category: None,
        };

        // Test password should have low confidence
        let confidence1 = calculate_confidence(&pattern, "test123", "password: test123");
        assert!(confidence1 < 0.5);

        // Production password should have higher confidence
        let confidence2 = calculate_confidence(&pattern, "Xy9$kL2mN8pQ", "prod_password: Xy9$kL2mN8pQ");
        assert!(confidence2 > 0.7);
    }

    #[test]
    fn test_looks_like_credential() {
        // Should return true for real-looking credentials
        assert!(looks_like_credential("MySecretPassword123"));
        assert!(looks_like_credential("sk-1234567890abcdef"));
        assert!(looks_like_credential("Xy9$kL2mN8pQ"));

        // Should return false for fake credentials
        assert!(!looks_like_credential("test123"));
        assert!(!looks_like_credential("example"));
        assert!(!looks_like_credential("xxx"));
        assert!(!looks_like_credential("short"));
        assert!(!looks_like_credential(""));
    }
}