//! Community Database Models
//!
//! Data structures for community member management, separate from the main analysis database.
//! These models are optimized for web access, admin management, and marketing analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Placeholder documentation for public items
/// Member roles with hierarchical permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemberRole {
    /// Regular community member - basic access
    Member,
    /// Developer - additional access to developer resources
    Developer,
    /// Administrator - full access to all features and admin panel
    Admin,
}

impl MemberRole {
    /// Check if this role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, MemberRole::Admin)
    }

    /// Check if this role has developer privileges (Developer or Admin)
    pub fn is_developer_or_higher(&self) -> bool {
        matches!(self, MemberRole::Developer | MemberRole::Admin)
    }

    /// Get role as string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberRole::Member => "member",
            MemberRole::Developer => "developer",
            MemberRole::Admin => "admin",
        }
    }

    /// Parse role from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => MemberRole::Admin,
            "developer" => MemberRole::Developer,
            _ => MemberRole::Member,
        }
    }
}

/// Placeholder documentation for public items
/// Core community member entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMember {
    /// Unique member identifier (UUID)
    pub id: String,
    /// Email address (unique, used for login)
    pub email: String,
    /// Display name
    pub name: String,
    /// Member role and permissions
    pub role: MemberRole,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_active: Option<DateTime<Utc>>,
    /// Account status
    pub is_active: bool,
    /// Profile information for marketing
    pub profile: MemberProfile,
    /// Email verification status
    pub email_verified: bool,
    /// Optional avatar URL
    pub avatar_url: Option<String>,
}

/// Placeholder documentation for public items
/// Member profile information for marketing analytics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemberProfile {
    /// Company/Organization (optional)
    pub company: Option<String>,
    /// Job title (optional)
    pub job_title: Option<String>,
    /// Programming languages of interest
    pub languages: Vec<String>,
    /// Experience level (beginner, intermediate, advanced, expert)
    pub experience_level: Option<String>,
    /// Primary use case for Uveddi
    pub use_case: Option<String>,
    /// How they heard about Uveddi
    pub referral_source: Option<String>,
    /// Newsletter subscription
    pub newsletter_subscribed: bool,
    /// Marketing consent
    pub marketing_consent: bool,
    /// Additional custom fields as JSON
    pub custom_fields: HashMap<String, String>,
}

/// Placeholder documentation for public items
/// Activity types for tracking member engagement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    /// Account related activities
    Registration,
    Login,
    Logout,
    ProfileUpdate,
    EmailVerified,
    PasswordReset,

    /// Feature usage
    DashboardView,
    AnalysisDownload,
    ReportGenerated,
    ApiCall,
    PluginInstall,

    /// Community engagement
    ForumPost,
    CommentPosted,
    DocumentationView,
    TutorialComplete,

    /// Admin activities
    AdminAction,
    RoleChanged,
    AccountDeactivated,

    /// Custom activity (with description)
    Custom(String),
}

impl ActivityType {
    /// Get activity type as string for database storage
    pub fn as_str(&self) -> String {
        match self {
            ActivityType::Registration => "registration".to_string(),
            ActivityType::Login => "login".to_string(),
            ActivityType::Logout => "logout".to_string(),
            ActivityType::ProfileUpdate => "profile_update".to_string(),
            ActivityType::EmailVerified => "email_verified".to_string(),
            ActivityType::PasswordReset => "password_reset".to_string(),
            ActivityType::DashboardView => "dashboard_view".to_string(),
            ActivityType::AnalysisDownload => "analysis_download".to_string(),
            ActivityType::ReportGenerated => "report_generated".to_string(),
            ActivityType::ApiCall => "api_call".to_string(),
            ActivityType::PluginInstall => "plugin_install".to_string(),
            ActivityType::ForumPost => "forum_post".to_string(),
            ActivityType::CommentPosted => "comment_posted".to_string(),
            ActivityType::DocumentationView => "documentation_view".to_string(),
            ActivityType::TutorialComplete => "tutorial_complete".to_string(),
            ActivityType::AdminAction => "admin_action".to_string(),
            ActivityType::RoleChanged => "role_changed".to_string(),
            ActivityType::AccountDeactivated => "account_deactivated".to_string(),
            ActivityType::Custom(desc) => format!("custom:{}", desc),
        }
    }

    /// Parse activity type from string
    pub fn from_str(s: &str) -> Self {
        if let Some(custom_desc) = s.strip_prefix("custom:") {
            return ActivityType::Custom(custom_desc.to_string());
        }

        match s {
            "registration" => ActivityType::Registration,
            "login" => ActivityType::Login,
            "logout" => ActivityType::Logout,
            "profile_update" => ActivityType::ProfileUpdate,
            "email_verified" => ActivityType::EmailVerified,
            "password_reset" => ActivityType::PasswordReset,
            "dashboard_view" => ActivityType::DashboardView,
            "analysis_download" => ActivityType::AnalysisDownload,
            "report_generated" => ActivityType::ReportGenerated,
            "api_call" => ActivityType::ApiCall,
            "plugin_install" => ActivityType::PluginInstall,
            "forum_post" => ActivityType::ForumPost,
            "comment_posted" => ActivityType::CommentPosted,
            "documentation_view" => ActivityType::DocumentationView,
            "tutorial_complete" => ActivityType::TutorialComplete,
            "admin_action" => ActivityType::AdminAction,
            "role_changed" => ActivityType::RoleChanged,
            "account_deactivated" => ActivityType::AccountDeactivated,
            _ => ActivityType::Custom(s.to_string()),
        }
    }
}

/// Placeholder documentation for public items
/// Member activity tracking for engagement analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberActivity {
    /// Unique activity identifier
    pub id: Option<i64>,
    /// Member who performed the activity
    pub member_id: String,
    /// Type of activity performed
    pub activity_type: ActivityType,
    /// Activity timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional activity description or metadata
    pub description: Option<String>,
    /// IP address (for security tracking)
    pub ip_address: Option<String>,
    /// User agent (for analytics)
    pub user_agent: Option<String>,
    /// Additional metadata as JSON
    pub metadata: HashMap<String, String>,
}

/// Placeholder documentation for public items
/// Member session tracking for authentication and security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberSession {
    /// Unique session identifier
    pub id: String,
    /// Member who owns this session
    pub member_id: String,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Last activity in this session
    pub last_activity: DateTime<Utc>,
    /// IP address for this session
    pub ip_address: Option<String>,
    /// User agent for this session
    pub user_agent: Option<String>,
    /// Whether session is currently active
    pub is_active: bool,
}

/// Placeholder documentation for public items
/// Admin-specific privileges and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminProfile {
    pub member_id: String,
    pub permissions: AdminPermissions,
    pub admin_level: AdminLevel,
    pub assigned_regions: Vec<String>, // Geographic or functional regions they manage
    pub assigned_teams: Vec<String>,   // Teams they oversee
    pub admin_notes: Option<String>,   // Internal admin notes
    pub appointed_by: Option<String>,  // Admin who granted permissions
    pub appointed_at: DateTime<Utc>,
    pub last_admin_action: Option<DateTime<Utc>>,
    pub is_super_admin: bool, // Root admin privileges
}

/// Placeholder documentation for public items
/// Granular admin permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPermissions {
    pub can_manage_users: bool,
    pub can_manage_content: bool,
    pub can_view_analytics: bool,
    pub can_manage_settings: bool,
    pub can_manage_admins: bool, // Only for super admins
    pub can_export_data: bool,
    pub can_delete_users: bool,
    pub can_manage_billing: bool, // For subscription management
    pub can_access_logs: bool,
}

impl Default for AdminPermissions {
    fn default() -> Self {
        Self {
            can_manage_users: false,
            can_manage_content: false,
            can_view_analytics: false,
            can_manage_settings: false,
            can_manage_admins: false,
            can_export_data: false,
            can_delete_users: false,
            can_manage_billing: false,
            can_access_logs: false,
        }
    }
}

impl AdminPermissions {
    /// Create basic moderator permissions
    pub fn moderator() -> Self {
        Self {
            can_manage_users: true,
            can_manage_content: true,
            can_view_analytics: true,
            can_manage_settings: false,
            can_manage_admins: false,
            can_export_data: false,
            can_delete_users: false,
            can_manage_billing: false,
            can_access_logs: false,
        }
    }

    /// Create full administrator permissions
    pub fn administrator() -> Self {
        Self {
            can_manage_users: true,
            can_manage_content: true,
            can_view_analytics: true,
            can_manage_settings: true,
            can_manage_admins: false,
            can_export_data: true,
            can_delete_users: true,
            can_manage_billing: true,
            can_access_logs: true,
        }
    }

    /// Create super admin permissions (all access)
    pub fn super_admin() -> Self {
        Self {
            can_manage_users: true,
            can_manage_content: true,
            can_view_analytics: true,
            can_manage_settings: true,
            can_manage_admins: true,
            can_export_data: true,
            can_delete_users: true,
            can_manage_billing: true,
            can_access_logs: true,
        }
    }
}

/// Placeholder documentation for public items
/// Admin hierarchy levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdminLevel {
    Moderator,     // Basic admin rights
    Administrator, // Full admin rights
    SuperAdmin,    // Root level access
}

/// Placeholder documentation for public items
/// Developer-specific profile and access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperProfile {
    pub member_id: String,
    pub developer_type: DeveloperType,
    pub specializations: Vec<String>, // Programming languages, frameworks
    pub github_verified: bool,
    pub contribution_score: i32, // Based on community contributions
    pub api_access_level: ApiAccessLevel,
    pub repositories: Vec<String>, // Associated repositories
    pub badges: Vec<DeveloperBadge>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verification_method: Option<String>, // How they were verified
    pub mentor_status: bool,                 // Can mentor other developers
    pub beta_tester: bool,                   // Access to beta features
}

/// Placeholder documentation for public items
/// Types of developers in the community
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeveloperType {
    OpenSource, // Open source contributor
    Enterprise, // Enterprise user
    Student,    // Student developer
    Freelancer, // Independent developer
    Startup,    // Startup team member
    Corporate,  // Large company developer
}

/// Placeholder documentation for public items
/// API access levels for developers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiAccessLevel {
    Basic,      // Rate limited, basic endpoints
    Premium,    // Higher limits, more endpoints
    Enterprise, // Full access, custom limits
}

/// Placeholder documentation for public items
/// Developer badges and achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperBadge {
    pub badge_type: BadgeType,
    pub earned_at: DateTime<Utc>,
    pub description: String,
    pub metadata: Option<String>, // Additional context
}

/// Placeholder documentation for public items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BadgeType {
    EarlyAdopter,
    Contributor,
    Mentor,
    BugFinder,
    FeatureRequester,
    BetaTester,
    CommunityHelper,
    CodeReviewer,
    Documentation,
    Custom(String),
}

/// Placeholder documentation for public items
/// Role-specific settings and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissions {
    pub role: MemberRole,
    pub can_access_api: bool,
    pub can_view_source: bool,
    pub can_download_reports: bool,
    pub can_create_projects: bool,
    pub can_share_publicly: bool,
    pub can_invite_members: bool,
    pub rate_limit_tier: i32,  // API rate limiting tier
    pub storage_quota_mb: i64, // File storage quota
    pub features: Vec<String>, // Enabled features list
}
