//! License validation
//!
//! This module handles license key validation, tier checking, and feature gating.

use super::errors::{LicenseError, LicenseErrorKind};
use super::license::{License, LicenseTier};
use super::storage::load_license;
use super::features;
use std::sync::OnceLock;

/// Cached current tier for performance
static CACHED_TIER: OnceLock<LicenseTier> = OnceLock::new();

/// Validate a license key format
///
/// Valid format: UVDD-XXXX-XXXX-XXXX-XXXX
/// Where X is alphanumeric (A-Z, 0-9)
pub fn validate_license_key(key: &str) -> Result<(), LicenseError> {
    let key = key.trim().to_uppercase();
    
    // Check overall format
    let parts: Vec<&str> = key.split('-').collect();
    if parts.len() != 5 {
        return Err(LicenseError::invalid_key_format(
            "License key must be in format UVDD-XXXX-XXXX-XXXX-XXXX"
        ));
    }
    
    // Check prefix
    if parts[0] != "UVDD" {
        return Err(LicenseError::invalid_key_format(
            "License key must start with 'UVDD-'"
        ));
    }
    
    // Check each segment is 4 alphanumeric characters
    for (i, part) in parts.iter().enumerate().skip(1) {
        if part.len() != 4 {
            return Err(LicenseError::invalid_key_format(
                format!("Segment {} must be exactly 4 characters", i + 1)
            ));
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(LicenseError::invalid_key_format(
                format!("Segment {} must contain only letters and numbers", i + 1)
            ));
        }
    }
    
    Ok(())
}

/// Check if a license is valid (not expired and correctly signed)
pub fn is_license_valid(license: &License) -> bool {
    // Check expiration (with grace period)
    if license.is_expired() && !license.is_in_grace_period() {
        return false;
    }
    
    // TODO: Verify signature when Keygen.sh integration is complete
    // For now, accept any license with valid structure
    
    true
}

/// Get the current license tier
///
/// Returns Free if no license is found or if the license is invalid.
/// Caches the result for performance.
pub fn get_current_tier() -> LicenseTier {
    // Return cached tier if available
    if let Some(&tier) = CACHED_TIER.get() {
        return tier;
    }
    
    // Try to load and validate license
    let tier = match load_license() {
        Ok(license) => {
            if is_license_valid(&license) {
                license.tier
            } else {
                LicenseTier::Free
            }
        }
        Err(_) => LicenseTier::Free,
    };
    
    // Cache the result (ignore if already set by another thread)
    let _ = CACHED_TIER.set(tier);
    
    tier
}

/// Force refresh of the cached tier
pub fn refresh_tier_cache() -> LicenseTier {
    // We can't clear OnceLock, so we'll need to check directly
    match load_license() {
        Ok(license) if is_license_valid(&license) => license.tier,
        _ => LicenseTier::Free,
    }
}

/// Check if a specific feature is allowed for the given tier
pub fn is_feature_allowed(feature: &str, tier: &LicenseTier) -> bool {
    match feature {
        // Free tier languages (always allowed)
        "javascript-lang" | "typescript-lang" => true,
        
        // Pro tier languages
        features::LANG_RUST |
        features::LANG_PYTHON |
        features::LANG_GO |
        features::LANG_JAVA |
        features::LANG_C |
        features::LANG_CPP |
        features::LANG_PHP |
        features::LANG_RUBY => *tier >= LicenseTier::Pro,
        
        // Team tier languages
        features::LANG_CSHARP |
        features::LANG_KOTLIN |
        features::LANG_SWIFT |
        features::LANG_SCALA => *tier >= LicenseTier::Team,
        
        // Enterprise tier languages
        features::LANG_LUA |
        features::LANG_SQL => *tier >= LicenseTier::Enterprise,
        
        // Free tier detectors (core anti-pattern detectors and security scanning
        // are always available, even without a license)
        features::DETECTOR_DEAD_CODE |
        features::DETECTOR_LARGE_CLASSES |
        features::DETECTOR_TIGHT_COUPLING |
        features::DETECTOR_LONG_METHODS |
        features::DETECTOR_MAGIC_VALUES |
        features::DETECTOR_SECURITY => true,

        // Pro tier detectors
        features::DETECTOR_CYCLIC_DEPS => *tier >= LicenseTier::Pro,
        
        // Pro tier output formats
        features::OUTPUT_JSON |
        features::OUTPUT_HTML |
        features::OUTPUT_SVG |
        features::OUTPUT_SARIF => *tier >= LicenseTier::Pro,
        
        // Pro tier features
        features::AI_INSIGHTS => *tier >= LicenseTier::Pro,
        
        // Team tier features
        features::MULTI_SEAT => *tier >= LicenseTier::Team,
        
        // Unknown features default to Pro tier requirement
        _ => *tier >= LicenseTier::Pro,
    }
}

/// Check if a feature is allowed and return an error if not
pub fn require_feature(feature: &str) -> Result<(), LicenseError> {
    let tier = get_current_tier();
    
    if is_feature_allowed(feature, &tier) {
        Ok(())
    } else {
        let required_tier = get_required_tier_for_feature(feature);
        Err(LicenseError::feature_not_available(feature, required_tier))
    }
}

/// Get the minimum tier required for a feature
pub fn get_required_tier_for_feature(feature: &str) -> &'static str {
    match feature {
        "javascript-lang" | "typescript-lang" => "Free",

        features::DETECTOR_DEAD_CODE |
        features::DETECTOR_LARGE_CLASSES |
        features::DETECTOR_TIGHT_COUPLING |
        features::DETECTOR_LONG_METHODS |
        features::DETECTOR_MAGIC_VALUES |
        features::DETECTOR_SECURITY => "Free",

        features::LANG_RUST |
        features::LANG_PYTHON |
        features::LANG_GO |
        features::LANG_JAVA |
        features::LANG_C |
        features::LANG_CPP |
        features::LANG_PHP |
        features::LANG_RUBY |
        features::DETECTOR_CYCLIC_DEPS |
        features::OUTPUT_JSON |
        features::OUTPUT_HTML |
        features::OUTPUT_SVG |
        features::OUTPUT_SARIF |
        features::AI_INSIGHTS => "Pro",
        
        features::LANG_CSHARP |
        features::LANG_KOTLIN |
        features::LANG_SWIFT |
        features::LANG_SCALA |
        features::MULTI_SEAT => "Team",
        
        features::LANG_LUA |
        features::LANG_SQL => "Enterprise",
        
        _ => "Pro",
    }
}

/// Get the machine ID for this system
pub fn get_machine_id() -> String {
    // Use environment variables and system info for a simple machine ID
    // This is a basic implementation; production would use hardware IDs
    
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string());
    
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown-user".to_string());
    
    format!("{}-{}", hostname, username)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_key_format() {
        assert!(validate_license_key("UVDD-ABCD-1234-EFGH-5678").is_ok());
        assert!(validate_license_key("uvdd-abcd-1234-efgh-5678").is_ok()); // Case insensitive
    }
    
    #[test]
    fn test_invalid_key_format() {
        assert!(validate_license_key("INVALID-KEY").is_err());
        assert!(validate_license_key("XXXX-ABCD-1234-EFGH-5678").is_err()); // Wrong prefix
        assert!(validate_license_key("UVDD-ABC-1234-EFGH-5678").is_err()); // Segment too short
        assert!(validate_license_key("UVDD-ABCD-12!4-EFGH-5678").is_err()); // Invalid character
    }
    
    #[test]
    fn test_feature_allowed() {
        // Free tier
        assert!(is_feature_allowed("javascript-lang", &LicenseTier::Free));
        assert!(is_feature_allowed("typescript-lang", &LicenseTier::Free));
        assert!(!is_feature_allowed(features::LANG_RUST, &LicenseTier::Free));

        // Core detectors and security scanning are available on the free tier
        assert!(is_feature_allowed(features::DETECTOR_DEAD_CODE, &LicenseTier::Free));
        assert!(is_feature_allowed(features::DETECTOR_LARGE_CLASSES, &LicenseTier::Free));
        assert!(is_feature_allowed(features::DETECTOR_TIGHT_COUPLING, &LicenseTier::Free));
        assert!(is_feature_allowed(features::DETECTOR_LONG_METHODS, &LicenseTier::Free));
        assert!(is_feature_allowed(features::DETECTOR_MAGIC_VALUES, &LicenseTier::Free));
        assert!(is_feature_allowed(features::DETECTOR_SECURITY, &LicenseTier::Free));
        // ...but advanced detectors still require Pro
        assert!(!is_feature_allowed(features::DETECTOR_CYCLIC_DEPS, &LicenseTier::Free));
        assert!(is_feature_allowed(features::DETECTOR_CYCLIC_DEPS, &LicenseTier::Pro));
        
        // Pro tier
        assert!(is_feature_allowed(features::LANG_RUST, &LicenseTier::Pro));
        assert!(is_feature_allowed(features::LANG_GO, &LicenseTier::Pro));
        assert!(!is_feature_allowed(features::LANG_CSHARP, &LicenseTier::Pro));
        
        // Team tier
        assert!(is_feature_allowed(features::LANG_CSHARP, &LicenseTier::Team));
        assert!(is_feature_allowed(features::LANG_KOTLIN, &LicenseTier::Team));
        
        // Enterprise tier
        assert!(is_feature_allowed(features::LANG_LUA, &LicenseTier::Enterprise));
    }
}
