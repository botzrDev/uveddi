//! License management command
//!
//! This module provides CLI commands for managing UVEDDI licenses:
//! - `activate`: Activate a license key
//! - `status`: Show current license status
//! - `deactivate`: Remove license from this machine
//! - `info`: Show detailed license information

use clap::{Args, Subcommand};

use crate::error::UveddiError;
use crate::licensing::{
    activate_license, deactivate_license, check_activation_status, print_activation_status,
    load_license, get_current_tier, validate_license_key, LicenseInfo, ActivationStatus,
};

/// License management command
#[derive(Args, Debug, Clone)]
pub struct LicenseCommand {
    #[command(subcommand)]
    pub action: LicenseAction,
}

/// License subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum LicenseAction {
    /// Activate a license key on this machine
    Activate {
        /// License key in format UVDD-XXXX-XXXX-XXXX-XXXX
        #[arg(value_name = "KEY")]
        key: String,
    },
    
    /// Show current license status
    Status,
    
    /// Deactivate and remove license from this machine
    Deactivate,
    
    /// Show detailed license information
    Info,
}

impl LicenseCommand {
    /// Execute the license command
    pub async fn execute(&self) -> Result<(), UveddiError> {
        match &self.action {
            LicenseAction::Activate { key } => self.activate(key).await,
            LicenseAction::Status => self.status().await,
            LicenseAction::Deactivate => self.deactivate().await,
            LicenseAction::Info => self.info().await,
        }
    }
    
    /// Activate a license key
    async fn activate(&self, key: &str) -> Result<(), UveddiError> {
        println!("\n🔑 Activating license...\n");
        
        // Validate format first
        if let Err(e) = validate_license_key(key) {
            eprintln!("❌ Invalid license key format: {}", e);
            eprintln!("\n   Expected format: UVDD-XXXX-XXXX-XXXX-XXXX");
            eprintln!("   Example: UVDD-PRO1-A2B3-C4D5-E6F7");
            return Err(UveddiError::config_error(&e.to_string(), "license activation"));
        }
        
        match activate_license(key).await {
            Ok(license) => {
                println!("✅ License activated successfully!\n");
                println!("══════════════════════════════════════");
                println!("  Tier:      {}", license.tier.display_name());
                println!("  Key:       {}", license.masked_key());
                if let Some(days) = license.days_until_expiration() {
                    println!("  Expires:   in {} days", days);
                } else {
                    println!("  Expires:   Never (lifetime license)");
                }
                println!("  Languages: {} supported", license.tier.language_count());
                println!("  Detectors: {} available", license.tier.detector_count());
                println!();
                println!("🎉 You now have access to premium features!");
                println!("   Run 'uveddi license info' to see all unlocked features.");
                println!();
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Activation failed: {}", e);
                eprintln!("\n   If this persists, please contact support@botzr.com");
                Err(UveddiError::config_error(&e.to_string(), "license activation"))
            }
        }
    }
    
    /// Show license status
    async fn status(&self) -> Result<(), UveddiError> {
        print_activation_status();
        Ok(())
    }
    
    /// Deactivate and remove license
    async fn deactivate(&self) -> Result<(), UveddiError> {
        let status = check_activation_status();
        
        if matches!(status, ActivationStatus::NotActivated) {
            println!("\nℹ️  No license is currently activated on this machine.");
            return Ok(());
        }
        
        println!("\n⚠️  Deactivating license...\n");
        println!("   This will remove the license from this machine.");
        println!("   You can re-activate it later with your license key.");
        println!();
        
        match deactivate_license().await {
            Ok(()) => {
                println!("✅ License deactivated successfully.");
                println!();
                println!("   You are now on the Free tier.");
                println!("   To re-activate, run: uveddi license activate <KEY>");
                println!();
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Deactivation failed: {}", e);
                Err(UveddiError::config_error(&e.to_string(), "license deactivation"))
            }
        }
    }
    
    /// Show detailed license info
    async fn info(&self) -> Result<(), UveddiError> {
        let status = check_activation_status();
        let tier = get_current_tier();
        
        println!("\n📋 UVEDDI License Information");
        println!("══════════════════════════════════════════════════════════");
        
        match load_license() {
            Ok(license) => {
                let info = LicenseInfo::from_license(&license);
                
                println!();
                println!("  License Key:     {}", license.masked_key());
                println!("  Tier:            {}", license.tier.display_name());
                println!("  Status:          {}", if info.is_active { "✅ Active" } else { "❌ Inactive" });
                
                if let Some(days) = info.days_remaining {
                    if days > 30 {
                        println!("  Expires in:      {} days", days);
                    } else if days > 0 {
                        println!("  Expires in:      ⚠️  {} days (renew soon!)", days);
                    } else {
                        println!("  Expires in:      ❌ Expired");
                    }
                } else {
                    println!("  Expires:         Never (lifetime)");
                }
                
                println!("  Seats:           {}/{}", info.seats_used, info.seats_total);
                println!("  Machine ID:      {}", license.machine_id);
                println!("  Activated:       {}", license.activated_at.format("%Y-%m-%d %H:%M UTC"));
                
                println!();
                println!("  Unlocked Features:");
                println!("  ──────────────────────────────────────────────────────");
                for feature in info.features.iter() {
                    println!("    ✓ {}", feature);
                }
            }
            Err(_) => {
                println!();
                println!("  Tier:            Free");
                println!("  Status:          No license activated");
                println!();
                println!("  Available Features (Free Tier):");
                println!("  ──────────────────────────────────────────────────────");
                println!("    ✓ JavaScript analysis");
                println!("    ✓ TypeScript analysis");
                println!("    ✓ Core anti-pattern detectors");
                println!("    ✓ Security scanning");
                println!("    ✓ Markdown output");
                println!();
                println!("  💡 Upgrade to Pro for 10 languages, cyclic-dependency detection, and more:");
                println!("     https://uveddi.org/pricing");
            }
        }
        
        println!();
        println!("══════════════════════════════════════════════════════════");
        println!();
        
        Ok(())
    }
}
