//! Enhanced Community Database Example with Admin and Developer Management
//!
//! This example demonstrates the complete community management system including
//! role-specific functionality for admins and developers.
//! Run with: `cargo run --example enhanced_community_demo`

use chrono::Utc;
use std::collections::HashMap;
use uveddi::community::{
    CommunityDatabase, MemberRole, ActivityType, MemberProfile,
};
use uveddi::community::models::{
    AdminPermissions, AdminLevel, DeveloperType, BadgeType
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced Uveddi Community Database Demo");
    println!("============================================\n");

    // Initialize the community database
    let db = CommunityDatabase::new("enhanced_community_demo.db")?;
    println!("✅ Enhanced community database initialized");

    // Register members with different roles
    println!("\n📝 Registering community members with different roles...");
    
    let alice = db.register_member("alice@techcorp.com", "Alice Johnson", MemberRole::Developer)?;
    println!("   Registered: {} ({})", alice.name, alice.role.as_str());

    let bob = db.register_member("bob@startup.io", "Bob Smith", MemberRole::Member)?;
    println!("   Registered: {} ({})", bob.name, bob.role.as_str());

    let admin_user = db.register_member("admin@uveddi.com", "Phillip Green", MemberRole::Admin)?;
    println!("   Registered: {} ({})", admin_user.name, admin_user.role.as_str());

    let carol = db.register_member("carol@enterprise.com", "Carol Williams", MemberRole::Developer)?;
    println!("   Registered: {} ({})", carol.name, carol.role.as_str());

    // Create admin profile for the admin user
    println!("\n👑 Setting up admin profiles...");
    let admin_permissions = AdminPermissions::super_admin();
    let admin_profile = db.create_admin_profile(
        &admin_user.id,
        admin_permissions,
        AdminLevel::SuperAdmin,
        &admin_user.id, // Self-appointed for demo
    )?;
    println!("   Created super admin profile for {}", admin_user.name);

    // Create developer profiles
    println!("\n💻 Setting up developer profiles...");
    
    let alice_dev_profile = db.create_developer_profile(
        &alice.id,
        DeveloperType::Enterprise,
        vec!["Rust".to_string(), "Python".to_string(), "DevOps".to_string()],
    )?;
    println!("   Created enterprise developer profile for {}", alice.name);

    let carol_dev_profile = db.create_developer_profile(
        &carol.id,
        DeveloperType::OpenSource,
        vec!["JavaScript".to_string(), "TypeScript".to_string(), "React".to_string()],
    )?;
    println!("   Created open source developer profile for {}", carol.name);

    // Award some developer badges
    println!("\n🏆 Awarding developer badges...");
    
    db.award_developer_badge(
        &alice.id,
        BadgeType::EarlyAdopter,
        "Early adopter of Uveddi platform",
        Some(r#"{"join_date": "2025-01-01", "feature": "beta_testing"}"#),
    )?;
    
    db.award_developer_badge(
        &alice.id,
        BadgeType::Contributor,
        "Contributed to core platform improvements",
        Some(r#"{"contributions": 15, "area": "performance"}"#),
    )?;

    db.award_developer_badge(
        &carol.id,
        BadgeType::BugFinder,
        "Reported critical security vulnerability",
        Some(r#"{"severity": "high", "component": "auth"}"#),
    )?;

    println!("   Awarded badges to Alice and Carol");

    // Update member profiles with role-specific information
    println!("\n📋 Updating member profiles...");
    let mut alice_profile = MemberProfile {
        company: Some("TechCorp Inc.".to_string()),
        job_title: Some("Senior Software Engineer".to_string()),
        languages: vec!["Rust".to_string(), "Python".to_string(), "TypeScript".to_string()],
        experience_level: Some("advanced".to_string()),
        use_case: Some("Enterprise code quality analysis".to_string()),
        referral_source: Some("GitHub".to_string()),
        newsletter_subscribed: true,
        marketing_consent: true,
        custom_fields: HashMap::new(),
    };
    alice_profile.custom_fields.insert("team_size".to_string(), "25".to_string());
    alice_profile.custom_fields.insert("deployment_env".to_string(), "kubernetes".to_string());

    db.update_member_profile(&alice.id, &alice_profile)?;
    println!("   Updated profile for {} (Developer)", alice.name);

    // Promote Bob to Developer after he shows interest
    println!("\n⬆️ Promoting member roles...");
    db.update_member_role(&bob.id, MemberRole::Developer)?;
    
    // Create developer profile for the newly promoted Bob
    let bob_dev_profile = db.create_developer_profile(
        &bob.id,
        DeveloperType::Startup,
        vec!["Go".to_string(), "JavaScript".to_string()],
    )?;
    
    println!("   Promoted {} to Developer and created profile", bob.name);

    // Log various activities showing role-specific actions
    println!("\n📊 Logging role-specific activities...");
    
    // Regular user activities
    db.log_activity(&alice.id, ActivityType::Login, None, Some("192.168.1.100"), Some("Mozilla/5.0"))?;
    db.log_activity(&alice.id, ActivityType::ApiCall, Some("Analyzed large codebase"), None, None)?;
    db.log_activity(&alice.id, ActivityType::ReportGenerated, Some("Generated enterprise report"), None, None)?;
    
    // Developer-specific activities
    db.log_activity(&carol.id, ActivityType::Login, None, Some("10.0.0.50"), Some("Chrome/91.0"))?;
    db.log_activity(&carol.id, ActivityType::PluginInstall, Some("Installed custom linting plugin"), None, None)?;
    db.log_activity(&carol.id, ActivityType::ForumPost, Some("Helped community member with setup"), None, None)?;
    
    // Admin activities
    db.log_activity(&admin_user.id, ActivityType::AdminAction, Some("Reviewed user permissions"), None, None)?;
    db.log_activity(&admin_user.id, ActivityType::RoleChanged, Some("Promoted user to developer"), None, None)?;

    println!("   Logged {} activities across different roles", 7);

    // Display enhanced member information by role
    println!("\n👥 Community Members by Role:");
    
    // List all admins with their permissions
    let admins = db.list_admins()?;
    println!("\n   🔧 Administrators ({}):", admins.len());
    for (member, admin_profile) in &admins {
        println!("     {} {} ({}) - Level: {:?}", 
                 if admin_profile.is_super_admin { "👑" } else { "🔧" },
                 member.name, 
                 member.email,
                 admin_profile.admin_level
        );
        println!("       Appointed: {}", admin_profile.appointed_at.format("%Y-%m-%d"));
        println!("       Can manage users: {}", admin_profile.permissions.can_manage_users);
        println!("       Can manage admins: {}", admin_profile.permissions.can_manage_admins);
    }

    // List all developers with their profiles
    let all_members = db.list_members(None, true, None, None)?;
    let developers: Vec<_> = all_members.iter()
        .filter(|m| m.role.is_developer_or_higher() && !m.role.is_admin())
        .collect();
    
    println!("\n   💻 Developers ({}):", developers.len());
    for member in &developers {
        if let Ok(Some(dev_profile)) = db.get_developer_profile(&member.id) {
            println!("     💻 {} ({}) - Type: {:?}", 
                     member.name, 
                     member.email,
                     dev_profile.developer_type
            );
            println!("       Specializations: {}", dev_profile.specializations.join(", "));
            println!("       API Access: {:?}", dev_profile.api_access_level);
            println!("       Contribution Score: {}", dev_profile.contribution_score);
            println!("       GitHub Verified: {}", dev_profile.github_verified);
            println!("       Mentor Status: {}", dev_profile.mentor_status);
        }
    }

    // Show role permissions
    println!("\n🔐 Role Permissions Overview:");
    for role in [MemberRole::Member, MemberRole::Developer, MemberRole::Admin] {
        if let Ok(Some(permissions)) = db.get_role_permissions(&role) {
            println!("   {} Role: {}", 
                     match role {
                         MemberRole::Member => "👤",
                         MemberRole::Developer => "💻",
                         MemberRole::Admin => "👑",
                     },
                     role.as_str()
            );
            println!("     API Access: {}", permissions.can_access_api);
            println!("     View Source: {}", permissions.can_view_source);
            println!("     Rate Limit Tier: {}", permissions.rate_limit_tier);
            println!("     Storage Quota: {} MB", permissions.storage_quota_mb);
            println!("     Features: {}", permissions.features.join(", "));
            println!();
        }
    }

    // Generate analytics including role-specific metrics
    println!("\n📊 Enhanced Community Analytics:");
    let all_members = db.list_members(None, true, None, None)?;
    println!("   Total Active Members: {}", all_members.len());
    
    let member_count = all_members.iter().filter(|m| matches!(m.role, MemberRole::Member)).count();
    let dev_count = all_members.iter().filter(|m| matches!(m.role, MemberRole::Developer)).count();
    let admin_count = all_members.iter().filter(|m| matches!(m.role, MemberRole::Admin)).count();
    
    println!("   Members by Role:");
    println!("     👤 Members: {}", member_count);
    println!("     💻 Developers: {}", dev_count);
    println!("     👑 Admins: {}", admin_count);

    // Show recent activities
    println!("\n📈 Recent Community Activity:");
    let recent_activities = db.get_member_activity(&alice.id, Some(3))?;
    for activity in recent_activities {
        println!("   {} - {} {}",
                 activity.timestamp.format("%Y-%m-%d %H:%M"),
                 activity.activity_type.as_str(),
                 activity.description.map(|d| format!("({})", d)).unwrap_or_default()
        );
    }

    println!("\n✅ Enhanced demo completed successfully!");
    println!("💾 Database saved to: enhanced_community_demo.db");
    println!("🔍 You can inspect the enhanced database using any SQLite browser");
    println!("\n📋 New Tables Created:");
    println!("   - admin_profiles (admin permissions and settings)");
    println!("   - developer_profiles (developer info and access levels)");
    println!("   - developer_badges (achievements and recognition)");
    println!("   - role_permissions (granular role-based permissions)");

    Ok(())
}
