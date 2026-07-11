//! License types and tier definitions
//!
//! This module defines the core license structures and tier levels.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// License tier levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum LicenseTier {
    /// Free tier - JavaScript/TypeScript, core anti-pattern detectors, security scanning, Markdown output
    #[default]
    Free,
    /// Pro tier ($199/year) - 10 languages, all detectors, all outputs
    Pro,
    /// Team tier ($99/month) - 14 languages, 5 seats
    Team,
    /// Enterprise tier (custom) - 16+ languages, unlimited seats
    Enterprise,
}

impl LicenseTier {
    /// Get the display name for this tier
    pub fn display_name(&self) -> &'static str {
        match self {
            LicenseTier::Free => "Free",
            LicenseTier::Pro => "Pro",
            LicenseTier::Team => "Team",
            LicenseTier::Enterprise => "Enterprise",
        }
    }
    
    /// Get the number of languages available in this tier
    pub fn language_count(&self) -> u32 {
        match self {
            LicenseTier::Free => 2,       // JS, TS
            LicenseTier::Pro => 10,       // + Python, Rust, Go, Java, C, C++, PHP, Ruby
            LicenseTier::Team => 14,      // + C#, Kotlin, Swift, Scala
            LicenseTier::Enterprise => 16, // + Lua, SQL, custom
        }
    }
    
    /// Get the number of detectors available in this tier
    pub fn detector_count(&self) -> u32 {
        match self {
            LicenseTier::Free => 8,       // Core anti-pattern detectors + security scanning
            LicenseTier::Pro => 9,        // All detectors (adds cyclic-dependency detection)
            LicenseTier::Team => 9,       // All detectors
            LicenseTier::Enterprise => 9, // All + custom
        }
    }
    
    /// Get the number of seats available in this tier
    pub fn seat_count(&self) -> u32 {
        match self {
            LicenseTier::Free => 1,
            LicenseTier::Pro => 1,
            LicenseTier::Team => 5,
            LicenseTier::Enterprise => u32::MAX, // Unlimited
        }
    }
    
    /// Parse tier from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "free" => Some(LicenseTier::Free),
            "pro" => Some(LicenseTier::Pro),
            "team" => Some(LicenseTier::Team),
            "enterprise" => Some(LicenseTier::Enterprise),
            _ => None,
        }
    }
}

impl std::fmt::Display for LicenseTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A validated license
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// License key (format: UVDD-XXXX-XXXX-XXXX-XXXX)
    pub key: String,
    
    /// License tier
    pub tier: LicenseTier,
    
    /// Expiration date (None for lifetime licenses)
    pub valid_until: Option<DateTime<Utc>>,
    
    /// Number of seats (for team licenses)
    pub seats: u32,
    
    /// Machine ID this license is activated on
    pub machine_id: String,
    
    /// When the license was activated
    pub activated_at: DateTime<Utc>,
    
    /// Customer email (optional)
    pub email: Option<String>,
    
    /// License signature for offline validation
    pub signature: String,
}

impl License {
    /// Create a new license
    pub fn new(
        key: String,
        tier: LicenseTier,
        valid_until: Option<DateTime<Utc>>,
        seats: u32,
        machine_id: String,
        signature: String,
    ) -> Self {
        Self {
            key,
            tier,
            valid_until,
            seats,
            machine_id,
            activated_at: Utc::now(),
            email: None,
            signature,
        }
    }
    
    /// Check if the license is expired
    pub fn is_expired(&self) -> bool {
        if let Some(valid_until) = self.valid_until {
            Utc::now() > valid_until
        } else {
            false // Lifetime license never expires
        }
    }
    
    /// Get days until expiration (None if lifetime or already expired)
    pub fn days_until_expiration(&self) -> Option<i64> {
        if let Some(valid_until) = self.valid_until {
            let now = Utc::now();
            if valid_until > now {
                Some((valid_until - now).num_days())
            } else {
                None // Already expired
            }
        } else {
            None // Lifetime license
        }
    }
    
    /// Check if license is in grace period (expired but within 7 days)
    pub fn is_in_grace_period(&self) -> bool {
        if let Some(valid_until) = self.valid_until {
            let now = Utc::now();
            let grace_end = valid_until + chrono::Duration::days(7);
            now > valid_until && now <= grace_end
        } else {
            false
        }
    }
    
    /// Get a masked version of the license key for display
    pub fn masked_key(&self) -> String {
        if self.key.len() > 10 {
            let parts: Vec<&str> = self.key.split('-').collect();
            if parts.len() >= 2 {
                return format!("{}-****-****-****-{}", parts[0], parts.last().unwrap_or(&"****"));
            }
        }
        "UVDD-****-****-****-****".to_string()
    }
}

/// License information for display
#[derive(Debug, Clone)]
pub struct LicenseInfo {
    pub tier: LicenseTier,
    pub is_active: bool,
    pub days_remaining: Option<i64>,
    pub seats_used: u32,
    pub seats_total: u32,
    pub features: Vec<String>,
}

impl LicenseInfo {
    /// Create license info from a license
    pub fn from_license(license: &License) -> Self {
        let features = get_tier_features(license.tier);
        
        Self {
            tier: license.tier,
            is_active: !license.is_expired() || license.is_in_grace_period(),
            days_remaining: license.days_until_expiration(),
            seats_used: 1, // TODO: Track actual seat usage
            seats_total: license.seats,
            features,
        }
    }
    
    /// Create info for free tier (no license)
    pub fn free_tier() -> Self {
        Self {
            tier: LicenseTier::Free,
            is_active: true,
            days_remaining: None,
            seats_used: 1,
            seats_total: 1,
            features: get_tier_features(LicenseTier::Free),
        }
    }
}

/// Get the list of features available for a tier
pub fn get_tier_features(tier: LicenseTier) -> Vec<String> {
    let mut features = vec![
        "JavaScript analysis".to_string(),
        "TypeScript analysis".to_string(),
        "Core anti-pattern detectors".to_string(),
        "Security scanning".to_string(),
        "Markdown output".to_string(),
    ];
    
    if tier >= LicenseTier::Pro {
        features.extend([
            "Python analysis".to_string(),
            "Rust analysis".to_string(),
            "Go analysis".to_string(),
            "Java analysis".to_string(),
            "C analysis".to_string(),
            "C++ analysis".to_string(),
            "PHP analysis".to_string(),
            "Ruby analysis".to_string(),
            "Cyclic-dependency detector".to_string(),
            "JSON/HTML/SVG output".to_string(),
            "SARIF export".to_string(),
            "AI-powered insights".to_string(),
        ]);
    }
    
    if tier >= LicenseTier::Team {
        features.extend([
            "C# analysis".to_string(),
            "Kotlin analysis".to_string(),
            "Swift analysis".to_string(),
            "Scala analysis".to_string(),
            "5 seats included".to_string(),
            "Priority support".to_string(),
        ]);
    }
    
    if tier >= LicenseTier::Enterprise {
        features.extend([
            "Lua analysis".to_string(),
            "SQL analysis".to_string(),
            "Custom language support".to_string(),
            "Unlimited seats".to_string(),
            "On-prem deployment".to_string(),
            "SSO/SAML".to_string(),
            "Compliance reports".to_string(),
            "Dedicated support".to_string(),
        ]);
    }
    
    features
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tier_ordering() {
        assert!(LicenseTier::Free < LicenseTier::Pro);
        assert!(LicenseTier::Pro < LicenseTier::Team);
        assert!(LicenseTier::Team < LicenseTier::Enterprise);
    }
    
    #[test]
    fn test_tier_from_str() {
        assert_eq!(LicenseTier::from_str("free"), Some(LicenseTier::Free));
        assert_eq!(LicenseTier::from_str("PRO"), Some(LicenseTier::Pro));
        assert_eq!(LicenseTier::from_str("Team"), Some(LicenseTier::Team));
        assert_eq!(LicenseTier::from_str("invalid"), None);
    }
    
    #[test]
    fn test_license_expiration() {
        let mut license = License::new(
            "UVDD-TEST-TEST-TEST-TEST".to_string(),
            LicenseTier::Pro,
            Some(Utc::now() + chrono::Duration::days(30)),
            1,
            "test-machine".to_string(),
            "test-sig".to_string(),
        );
        
        assert!(!license.is_expired());
        assert!(license.days_until_expiration().unwrap() >= 29);
        
        // Test expired license
        license.valid_until = Some(Utc::now() - chrono::Duration::days(1));
        assert!(license.is_expired());
    }
    
    #[test]
    fn test_masked_key() {
        let license = License::new(
            "UVDD-ABCD-EFGH-IJKL-MNOP".to_string(),
            LicenseTier::Pro,
            None,
            1,
            "test".to_string(),
            "sig".to_string(),
        );
        
        assert_eq!(license.masked_key(), "UVDD-****-****-****-MNOP");
    }
}
