//! License activation
//!
//! This module handles license activation and deactivation via the Keygen.sh API.

use super::errors::{LicenseError, LicenseErrorKind};
use super::license::{License, LicenseTier};
use super::storage::{save_license, delete_license, load_license};
use super::validation::{validate_license_key, get_machine_id};

use chrono::{DateTime, Utc, Duration};

/// Activation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationStatus {
    /// License is activated and valid
    Active,
    /// License is activated but expired
    Expired,
    /// License is in grace period (expired but still functional)
    GracePeriod { days_remaining: i64 },
    /// No license is activated
    NotActivated,
    /// License is invalid or corrupted
    Invalid,
}

impl ActivationStatus {
    /// Check if the status allows using premium features
    pub fn allows_premium_features(&self) -> bool {
        matches!(self, ActivationStatus::Active | ActivationStatus::GracePeriod { .. })
    }
}

/// Activate a license key
///
/// This validates the key format, contacts the license server (when available),
/// and stores the license locally.
pub async fn activate_license(key: &str) -> Result<License, LicenseError> {
    // Validate key format first
    validate_license_key(key)?;
    
    let key = key.trim().to_uppercase();
    let machine_id = get_machine_id();
    
    // TODO: When Keygen.sh is configured, make actual API call here
    // For now, we'll create a demo license based on key pattern
    
    let license = create_demo_license(&key, &machine_id)?;
    
    // Save the license to disk
    save_license(&license)?;
    
    Ok(license)
}

/// Create a demo license for development/testing
/// In production, this would be replaced with actual Keygen.sh API call
fn create_demo_license(key: &str, machine_id: &str) -> Result<License, LicenseError> {
    // Determine tier based on key pattern (for demo purposes)
    // In production, the server would return the actual tier
    let tier = if key.contains("ENTR") || key.contains("ENT") {
        LicenseTier::Enterprise
    } else if key.contains("TEAM") || key.contains("TM") {
        LicenseTier::Team
    } else if key.contains("PRO") || key.starts_with("UVDD-P") {
        LicenseTier::Pro
    } else {
        // Default to Pro for valid keys during demo period
        LicenseTier::Pro
    };
    
    // Set expiration (1 year for annual, 30 days for demo)
    let valid_until = Some(Utc::now() + Duration::days(365));
    
    // Determine seats
    let seats = match tier {
        LicenseTier::Free => 1,
        LicenseTier::Pro => 1,
        LicenseTier::Team => 5,
        LicenseTier::Enterprise => u32::MAX,
    };
    
    // Generate a simple signature (in production, this would be cryptographic)
    let signature = format!(
        "demo-sig-{}-{}",
        &key[5..9],
        machine_id.chars().take(8).collect::<String>()
    );
    
    Ok(License::new(
        key.to_string(),
        tier,
        valid_until,
        seats,
        machine_id.to_string(),
        signature,
    ))
}

/// Deactivate the current license
///
/// This removes the license from this machine and (when API is available)
/// frees up a seat on the license server.
pub async fn deactivate_license() -> Result<(), LicenseError> {
    // Load current license to get key for API call
    let license = load_license().ok();
    
    // TODO: When Keygen.sh is configured, make API call to release seat
    if let Some(_lic) = license {
        // Would call: keygen_api.deactivate(&lic.key, &lic.machine_id).await?;
    }
    
    // Delete local license file
    delete_license()?;
    
    Ok(())
}

/// Check the current activation status
pub fn check_activation_status() -> ActivationStatus {
    match load_license() {
        Ok(license) => {
            if license.is_expired() {
                if license.is_in_grace_period() {
                    // Calculate days remaining in grace period
                    if let Some(valid_until) = license.valid_until {
                        let grace_end = valid_until + Duration::days(7);
                        let days = (grace_end - Utc::now()).num_days();
                        ActivationStatus::GracePeriod { days_remaining: days }
                    } else {
                        ActivationStatus::Expired
                    }
                } else {
                    ActivationStatus::Expired
                }
            } else {
                ActivationStatus::Active
            }
        }
        Err(e) => {
            match e.kind {
                LicenseErrorKind::NotActivated => ActivationStatus::NotActivated,
                _ => ActivationStatus::Invalid,
            }
        }
    }
}

/// Print activation status to console with formatting
pub fn print_activation_status() {
    let status = check_activation_status();
    
    match status {
        ActivationStatus::Active => {
            if let Ok(license) = load_license() {
                println!("\n✅ License Status: Active");
                println!("══════════════════════════════════════");
                println!("  Tier:         {}", license.tier.display_name());
                println!("  Key:          {}", license.masked_key());
                if let Some(days) = license.days_until_expiration() {
                    println!("  Expires in:   {} days", days);
                } else {
                    println!("  Expires:      Never (lifetime)");
                }
                println!("  Seats:        1/{}", license.seats);
                println!("  Machine:      {}", license.machine_id);
                println!();
                println!("  Features unlocked:");
                for feature in super::license::get_tier_features(license.tier).iter().take(8) {
                    println!("    ✓ {}", feature);
                }
                if license.tier >= LicenseTier::Pro {
                    println!("    ... and more!");
                }
                println!();
            }
        }
        ActivationStatus::GracePeriod { days_remaining } => {
            println!("\n⚠️  License Status: Grace Period");
            println!("══════════════════════════════════════");
            println!("  Your license has expired but you have {} days", days_remaining);
            println!("  remaining in the grace period.");
            println!();
            println!("  💡 Renew at: https://uveddi.org/pricing");
            println!();
        }
        ActivationStatus::Expired => {
            println!("\n❌ License Status: Expired");
            println!("══════════════════════════════════════");
            println!("  Your license has expired and the grace period has ended.");
            println!("  Premium features are no longer available.");
            println!();
            println!("  💡 Renew at: https://uveddi.org/pricing");
            println!();
        }
        ActivationStatus::NotActivated => {
            println!("\nℹ️  License Status: Free Tier");
            println!("══════════════════════════════════════");
            println!("  No license activated. Using free tier features:");
            println!("    ✓ JavaScript analysis");
            println!("    ✓ TypeScript analysis");
            println!("    ✓ GodObject detector");
            println!("    ✓ Markdown output");
            println!();
            println!("  💡 Upgrade for 10+ languages, all detectors, security scanning, and AI:");
            println!("     https://uveddi.org/pricing");
            println!();
            println!("  Already have a key? Run: uveddi license activate <KEY>");
            println!();
        }
        ActivationStatus::Invalid => {
            println!("\n❌ License Status: Invalid");
            println!("══════════════════════════════════════");
            println!("  The stored license is invalid or corrupted.");
            println!();
            println!("  Try re-activating: uveddi license activate <KEY>");
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_activation_status_allows_premium() {
        assert!(ActivationStatus::Active.allows_premium_features());
        assert!(ActivationStatus::GracePeriod { days_remaining: 3 }.allows_premium_features());
        assert!(!ActivationStatus::Expired.allows_premium_features());
        assert!(!ActivationStatus::NotActivated.allows_premium_features());
    }
    
    #[tokio::test]
    async fn test_demo_license_creation() {
        let license = create_demo_license("UVDD-PRO1-TEST-DEMO-1234", "test-machine").unwrap();
        assert_eq!(license.tier, LicenseTier::Pro);
        
        let team_license = create_demo_license("UVDD-TEAM-TEST-DEMO-1234", "test-machine").unwrap();
        assert_eq!(team_license.tier, LicenseTier::Team);
    }
}
