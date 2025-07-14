//! Community Database Example
//!
//! This example demonstrates how to use the community member management system.
//! Run with: `cargo run --example community_demo`

use std::collections::HashMap;
use uveddi::community::{ActivityType, CommunityDatabase, MemberProfile, MemberRole};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Uveddi Community Database Demo");
    println!("==================================\n");

    // Initialize the community database
    let db = CommunityDatabase::new("community_demo.db")?;
    println!("✅ Community database initialized");

    // Register some demo members
    println!("\n📝 Registering demo members...");

    let alice = db.register_member("alice@techcorp.com", "Alice Johnson", MemberRole::Developer)?;
    println!("   Registered: {} ({})", alice.name, alice.email);

    let bob = db.register_member("bob@startup.io", "Bob Smith", MemberRole::Member)?;
    println!("   Registered: {} ({})", bob.name, bob.email);

    let admin = db.register_member("admin@uveddi.com", "Phillip Green", MemberRole::Admin)?;
    println!("   Registered: {} ({})", admin.name, admin.email);

    // Update Alice's profile with marketing information
    println!("\n📋 Updating member profiles...");
    let mut alice_profile = MemberProfile {
        company: Some("TechCorp Inc.".to_string()),
        job_title: Some("Senior Software Engineer".to_string()),
        languages: vec![
            "Rust".to_string(),
            "Python".to_string(),
            "TypeScript".to_string(),
        ],
        experience_level: Some("advanced".to_string()),
        use_case: Some("Code quality analysis for large codebase".to_string()),
        referral_source: Some("GitHub".to_string()),
        newsletter_subscribed: true,
        marketing_consent: true,
        custom_fields: HashMap::new(),
    };
    alice_profile
        .custom_fields
        .insert("team_size".to_string(), "15".to_string());
    alice_profile
        .custom_fields
        .insert("deployment_frequency".to_string(), "weekly".to_string());

    db.update_member_profile(&alice.id, &alice_profile)?;
    println!("   Updated profile for {}", alice.name);

    // Update Bob's profile
    let bob_profile = MemberProfile {
        company: Some("StartupIO".to_string()),
        job_title: Some("CTO".to_string()),
        languages: vec!["JavaScript".to_string(), "Go".to_string()],
        experience_level: Some("expert".to_string()),
        use_case: Some("Architectural review for scaling".to_string()),
        referral_source: Some("Twitter".to_string()),
        newsletter_subscribed: true,
        marketing_consent: false,
        custom_fields: HashMap::new(),
    };

    db.update_member_profile(&bob.id, &bob_profile)?;
    println!("   Updated profile for {}", bob.name);

    // Log some activities
    println!("\n📊 Logging member activities...");

    db.log_activity(
        &alice.id,
        ActivityType::Login,
        None,
        Some("192.168.1.100"),
        Some("Mozilla/5.0"),
    )?;
    db.log_activity(
        &alice.id,
        ActivityType::DashboardView,
        Some("Main dashboard accessed"),
        None,
        None,
    )?;
    db.log_activity(
        &alice.id,
        ActivityType::AnalysisDownload,
        Some("Downloaded Rust project analysis"),
        None,
        None,
    )?;
    db.log_activity(
        &alice.id,
        ActivityType::ReportGenerated,
        Some("Generated PDF report"),
        None,
        None,
    )?;

    db.log_activity(
        &bob.id,
        ActivityType::Login,
        None,
        Some("10.0.0.50"),
        Some("Chrome/91.0"),
    )?;
    db.log_activity(
        &bob.id,
        ActivityType::DashboardView,
        Some("Dashboard viewed"),
        None,
        None,
    )?;
    db.log_activity(
        &bob.id,
        ActivityType::DocumentationView,
        Some("API documentation"),
        None,
        None,
    )?;

    println!("   Logged activities for Alice and Bob");

    // Promote Bob to Developer role
    println!("\n👑 Updating member roles...");
    db.update_member_role(&bob.id, MemberRole::Developer)?;
    println!("   Promoted {} to Developer role", bob.name);

    // Display member list
    println!("\n👥 Current Community Members:");
    let members = db.list_members(None, true, None, None)?;
    for member in &members {
        let role_emoji = match member.role {
            MemberRole::Admin => "👑",
            MemberRole::Developer => "💻",
            MemberRole::Member => "👤",
        };
        println!(
            "   {} {} ({}) - {} - Last active: {:?}",
            role_emoji,
            member.name,
            member.email,
            member.role.as_str(),
            member
                .last_active
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or("Never".to_string())
        );
    }

    // Show recent activity for Alice
    println!("\n📈 Recent Activity for {}:", alice.name);
    let activities = db.get_member_activity(&alice.id, Some(5))?;
    for activity in activities {
        println!(
            "   {} - {} {}",
            activity.timestamp.format("%Y-%m-%d %H:%M"),
            activity.activity_type.as_str(),
            activity
                .description
                .map(|d| format!("({})", d))
                .unwrap_or_default()
        );
    }

    // Generate analytics
    println!("\n📊 Community Analytics:");
    println!("   Total Members: {}", members.len());

    let developers = db.list_members(Some(MemberRole::Developer), true, None, None)?;
    let admins = db.list_members(Some(MemberRole::Admin), true, None, None)?;
    let regular_members = db.list_members(Some(MemberRole::Member), true, None, None)?;

    println!("   Members by Role:");
    println!("     member: {}", regular_members.len());
    println!("     developer: {}", developers.len());
    println!("     admin: {}", admins.len());

    println!("\n✅ Demo completed successfully!");
    println!("💾 Database saved to: community_demo.db");
    println!("🔍 You can inspect the database using any SQLite browser");

    Ok(())
}
