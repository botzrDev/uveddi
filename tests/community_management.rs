//! Integration tests for the community management system

use tempfile::NamedTempFile;
use uveddi::community::{CommunityDatabase, MemberRole, ActivityType, MemberProfile};

#[test]
fn test_community_database_full_workflow() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = CommunityDatabase::new(temp_file.path()).unwrap();

    // Test member registration
    let member = db.register_member("test@example.com", "Test User", MemberRole::Member).unwrap();
    assert_eq!(member.email, "test@example.com");
    assert_eq!(member.name, "Test User");
    assert_eq!(member.role, MemberRole::Member);
    assert!(member.is_active);
    assert!(!member.email_verified);

    // Test member retrieval
    let retrieved = db.get_member_by_email("test@example.com").unwrap();
    assert!(retrieved.is_some());
    let retrieved_member = retrieved.unwrap();
    assert_eq!(retrieved_member.id, member.id);

    // Test profile update
    let mut profile = MemberProfile::default();
    profile.company = Some("Test Corp".to_string());
    profile.job_title = Some("Developer".to_string());
    profile.languages = vec!["Rust".to_string(), "Python".to_string()];
    profile.experience_level = Some("intermediate".to_string());
    profile.newsletter_subscribed = true;
    profile.marketing_consent = true;
    profile.custom_fields.insert("team_size".to_string(), "5".to_string());

    let updated = db.update_member_profile(&member.id, &profile).unwrap();
    assert!(updated);

    // Verify profile was updated
    let updated_member = db.get_member_by_id(&member.id).unwrap().unwrap();
    assert_eq!(updated_member.profile.company, Some("Test Corp".to_string()));
    assert_eq!(updated_member.profile.languages.len(), 2);
    assert!(updated_member.profile.newsletter_subscribed);

    // Test role update
    let role_updated = db.update_member_role(&member.id, MemberRole::Developer).unwrap();
    assert!(role_updated);

    let role_checked = db.get_member_by_id(&member.id).unwrap().unwrap();
    assert_eq!(role_checked.role, MemberRole::Developer);

    // Test activity logging
    let activity_id = db.log_activity(
        &member.id,
        ActivityType::Login,
        Some("Test login"),
        Some("127.0.0.1"),
        Some("test-agent"),
    ).unwrap();
    assert!(activity_id > 0);

    // Log more activities
    db.log_activity(&member.id, ActivityType::DashboardView, None, None, None).unwrap();
    db.log_activity(&member.id, ActivityType::ReportGenerated, Some("Test report"), None, None).unwrap();

    // Test activity retrieval
    let activities = db.get_member_activity(&member.id, Some(10)).unwrap();
    assert!(activities.len() >= 4); // Registration + Profile Update + Role Change + Login + Dashboard + Report

    // Check for specific activities
    let has_login = activities.iter().any(|a| matches!(a.activity_type, ActivityType::Login));
    let has_registration = activities.iter().any(|a| matches!(a.activity_type, ActivityType::Registration));
    assert!(has_login);
    assert!(has_registration);

    // Test member listing
    let all_members = db.list_members(None, true, None, None).unwrap();
    assert_eq!(all_members.len(), 1);

    let developers = db.list_members(Some(MemberRole::Developer), true, None, None).unwrap();
    assert_eq!(developers.len(), 1);

    let admins = db.list_members(Some(MemberRole::Admin), true, None, None).unwrap();
    assert_eq!(admins.len(), 0);

    // Test member deactivation
    let deactivated = db.deactivate_member(&member.id).unwrap();
    assert!(deactivated);

    let inactive_member = db.get_member_by_id(&member.id).unwrap().unwrap();
    assert!(!inactive_member.is_active);

    // Test that deactivated member doesn't appear in active listings
    let active_members = db.list_members(None, true, None, None).unwrap();
    assert_eq!(active_members.len(), 0);

    // But appears when including inactive
    let all_including_inactive = db.list_members(None, false, None, None).unwrap();
    assert_eq!(all_including_inactive.len(), 1);
}

#[test]
fn test_member_roles() {
    // Test role parsing
    assert_eq!(MemberRole::from_str("admin"), MemberRole::Admin);
    assert_eq!(MemberRole::from_str("developer"), MemberRole::Developer);
    assert_eq!(MemberRole::from_str("member"), MemberRole::Member);
    assert_eq!(MemberRole::from_str("unknown"), MemberRole::Member); // defaults to member

    // Test role permissions
    assert!(MemberRole::Admin.is_admin());
    assert!(!MemberRole::Developer.is_admin());
    assert!(!MemberRole::Member.is_admin());

    assert!(MemberRole::Admin.is_developer_or_higher());
    assert!(MemberRole::Developer.is_developer_or_higher());
    assert!(!MemberRole::Member.is_developer_or_higher());

    // Test role string conversion
    assert_eq!(MemberRole::Admin.as_str(), "admin");
    assert_eq!(MemberRole::Developer.as_str(), "developer");
    assert_eq!(MemberRole::Member.as_str(), "member");
}

#[test]
fn test_activity_types() {
    // Test activity type string conversion
    assert_eq!(ActivityType::Login.as_str(), "login");
    assert_eq!(ActivityType::Registration.as_str(), "registration");
    assert_eq!(ActivityType::Custom("test".to_string()).as_str(), "custom:test");

    // Test activity type parsing
    assert!(matches!(ActivityType::from_str("login"), ActivityType::Login));
    assert!(matches!(ActivityType::from_str("registration"), ActivityType::Registration));
    
    if let ActivityType::Custom(desc) = ActivityType::from_str("custom:test") {
        assert_eq!(desc, "test");
    } else {
        panic!("Expected Custom activity type");
    }

    // Unknown activity types become Custom
    if let ActivityType::Custom(desc) = ActivityType::from_str("unknown_activity") {
        assert_eq!(desc, "unknown_activity");
    } else {
        panic!("Expected Custom activity type for unknown activity");
    }
}

#[test]
fn test_analytics_generation() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = CommunityDatabase::new(temp_file.path()).unwrap();

    // Create test data
    let member1 = db.register_member("user1@example.com", "User One", MemberRole::Member).unwrap();
    let member2 = db.register_member("user2@example.com", "User Two", MemberRole::Developer).unwrap();
    let admin = db.register_member("admin@example.com", "Admin", MemberRole::Admin).unwrap();

    // Add some profile data
    let mut profile1 = MemberProfile::default();
    profile1.company = Some("Company A".to_string());
    profile1.languages = vec!["Rust".to_string(), "Go".to_string()];
    profile1.newsletter_subscribed = true;
    profile1.marketing_consent = true;
    db.update_member_profile(&member1.id, &profile1).unwrap();

    let mut profile2 = MemberProfile::default();
    profile2.company = Some("Company B".to_string());
    profile2.languages = vec!["Python".to_string(), "Rust".to_string()];
    profile2.newsletter_subscribed = false;
    profile2.marketing_consent = true;
    db.update_member_profile(&member2.id, &profile2).unwrap();

    // Log some activities
    db.log_activity(&member1.id, ActivityType::Login, None, None, None).unwrap();
    db.log_activity(&member1.id, ActivityType::DashboardView, None, None, None).unwrap();
    db.log_activity(&member2.id, ActivityType::Login, None, None, None).unwrap();
    db.log_activity(&member2.id, ActivityType::ReportGenerated, None, None, None).unwrap();

    // Generate analytics
    let analytics = db.generate_analytics().unwrap();

    // Verify basic counts
    assert_eq!(analytics.total_members, 3);
    assert_eq!(analytics.active_members, 3); // All are recently active

    // Verify role distribution
    assert_eq!(analytics.members_by_role.get("member"), Some(&1));
    assert_eq!(analytics.members_by_role.get("developer"), Some(&1));
    assert_eq!(analytics.members_by_role.get("admin"), Some(&1));

    // Verify growth metrics
    assert_eq!(analytics.growth_metrics.new_members_week, 3);
    assert_eq!(analytics.growth_metrics.new_members_month, 3);

    // Verify engagement metrics
    assert_eq!(analytics.engagement_metrics.monthly_active_users, 3);
    assert!(analytics.engagement_metrics.popular_activities.len() > 0);

    // Verify demographic insights
    assert_eq!(analytics.demographic_insights.top_companies.len(), 2);
    assert!(analytics.demographic_insights.newsletter_subscription_rate > 0.0);
    assert!(analytics.demographic_insights.marketing_consent_rate > 0.0);

    // Check language interests
    assert!(analytics.demographic_insights.language_interests.len() > 0);
    let rust_stats = analytics.demographic_insights.language_interests
        .iter()
        .find(|lang| lang.language == "Rust");
    assert!(rust_stats.is_some());
    assert_eq!(rust_stats.unwrap().member_count, 2); // Both members interested in Rust
}

#[test]
fn test_member_listing_with_pagination() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = CommunityDatabase::new(temp_file.path()).unwrap();

    // Create multiple members
    for i in 1..=5 {
        db.register_member(
            &format!("user{}@example.com", i),
            &format!("User {}", i),
            MemberRole::Member,
        ).unwrap();
    }

    // Test pagination
    let page1 = db.list_members(None, true, Some(2), Some(0)).unwrap();
    assert_eq!(page1.len(), 2);

    let page2 = db.list_members(None, true, Some(2), Some(2)).unwrap();
    assert_eq!(page2.len(), 2);

    let page3 = db.list_members(None, true, Some(2), Some(4)).unwrap();
    assert_eq!(page3.len(), 1);

    // Ensure no overlap between pages
    let page1_emails: Vec<&str> = page1.iter().map(|m| m.email.as_str()).collect();
    let page2_emails: Vec<&str> = page2.iter().map(|m| m.email.as_str()).collect();
    
    for email in &page1_emails {
        assert!(!page2_emails.contains(email));
    }
}
