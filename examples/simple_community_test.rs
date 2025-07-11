//! Simple Community Test
//!
//! Basic test of the community database functionality without the complex analytics.

use std::collections::HashMap;
use uveddi::community::{ActivityType, CommunityDatabase, MemberProfile, MemberRole};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Community Database Test");
    println!("==================================\n");

    // Initialize the community database
    let db = CommunityDatabase::new("simple_community_test.db")?;
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

    db.update_member_profile(&alice.id, &alice_profile)?;
    println!("   Updated profile for {}", alice.name);

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
        &bob.id,
        ActivityType::Login,
        None,
        Some("10.0.0.50"),
        Some("Chrome/91.0"),
    )?;

    println!("   Logged activities for Alice and Bob");

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
            "   {} {} ({}) - {}",
            role_emoji,
            member.name,
            member.email,
            member.role.as_str()
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

    println!("\n✅ Test completed successfully!");
    println!("💾 Database saved to: simple_community_test.db");

    Ok(())
}
